//! Temporary post-event frame layout refreshes.
//!
//! These layout nudges compensate for startup ordering/state gaps after the
//! core login events fire. Keep them isolated until the underlying Objective
//! Tracker, party-frame, casting-bar, and chat edit-box state flows are modeled.

use crate::lua_api::WowLuaEnv;

const POST_EVENT_FRAME_LAYOUT_WORKAROUND_LUA: &str = r#"
local function reanchor_default_objective_tracker(frame)
    if not frame:IsInDefaultPosition() then
        return
    end
    local container = GetRightManagedFrameContainer()
    local height = math.min(836.5, container:GetTop())
    frame:ClearAllPoints()
    frame:SetPoint(
        "TOPRIGHT",
        container,
        "TOPRIGHT",
        0,
        0
    )
    frame:SetHeight(height)
end

if UpdateRaidAndPartyFrames then
    pcall(UpdateRaidAndPartyFrames)
end
if PartyFrame and PartyFrame.UpdatePaddingAndLayout then
    pcall(PartyFrame.UpdatePaddingAndLayout, PartyFrame)
end
if CompactPartyFrame and CompactPartyFrame.UpdateVisibility then
    pcall(CompactPartyFrame.UpdateVisibility, CompactPartyFrame)
end
if ObjectiveTrackerFrame then
    if ObjectiveTrackerFrame.Update then
        pcall(ObjectiveTrackerFrame.Update, ObjectiveTrackerFrame)
    end
    if ObjectiveTrackerFrame.UpdateHeight then
        pcall(ObjectiveTrackerFrame.UpdateHeight, ObjectiveTrackerFrame)
    end
    reanchor_default_objective_tracker(ObjectiveTrackerFrame)
end
if not rawget(_G, "__wow_objective_tracker_resize_event_frame")
    and CreateFrame
    and ObjectiveTrackerFrame then
    local resizeEventFrame = CreateFrame("Frame")
    resizeEventFrame:RegisterEvent("DISPLAY_SIZE_CHANGED")
    resizeEventFrame:RegisterEvent("UI_SCALE_CHANGED")
    resizeEventFrame:SetScript("OnEvent", function()
        reanchor_default_objective_tracker(ObjectiveTrackerFrame)
    end)
    rawset(_G, "__wow_objective_tracker_resize_event_frame", resizeEventFrame)
end
if CompactPartyFrame then
    CompactPartyFrame:SetHeight(234)
end
if PlayerCastingBarFrame then
    PlayerCastingBarFrame:SetAlpha(1)
end
if not rawget(_G, "__wow_objective_tracker_update_height_wrapper")
    and ObjectiveTrackerContainerMixin
    and type(ObjectiveTrackerContainerMixin.UpdateHeight) == "function" then
    local originalUpdateHeight = ObjectiveTrackerContainerMixin.UpdateHeight
    function ObjectiveTrackerContainerMixin:UpdateHeight()
        originalUpdateHeight(self)
        if self == ObjectiveTrackerFrame then
            reanchor_default_objective_tracker(self)
        end
    end
    rawset(_G, "__wow_objective_tracker_update_height_wrapper", true)
end
if not rawget(_G, "__wow_compact_party_update_layout_wrapper")
    and CompactPartyFrameMixin
    and type(CompactPartyFrameMixin.UpdateLayout) == "function" then
    local originalUpdateLayout = CompactPartyFrameMixin.UpdateLayout
    function CompactPartyFrameMixin:UpdateLayout()
        originalUpdateLayout(self)
        self:SetHeight(234)
    end
    rawset(_G, "__wow_compact_party_update_layout_wrapper", true)
end
if not rawget(_G, "__wow_casting_bar_apply_alpha_wrapper")
    and CastingBarMixin
    and type(CastingBarMixin.ApplyAlpha) == "function" then
    local originalApplyAlpha = CastingBarMixin.ApplyAlpha
    function CastingBarMixin:ApplyAlpha(alpha)
        if self == PlayerCastingBarFrame then
            alpha = 1
        end
        originalApplyAlpha(self, alpha)
    end
    rawset(_G, "__wow_casting_bar_apply_alpha_wrapper", true)
end
if ChatFrame1EditBox and ChatFrame1 then
    ChatFrame1EditBox:SetWidth(447)
end
"#;

pub(crate) fn patch(env: &WowLuaEnv) {
    let _ = env.exec(POST_EVENT_FRAME_LAYOUT_WORKAROUND_LUA);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn post_event_layout_preserves_saved_raid_style_party_frame_setting() {
        let env = WowLuaEnv::new().expect("create Lua env");
        env.exec(
            r#"
            Enum = {
                EditModeSystem = { UnitFrame = 3 },
                EditModeUnitFrameSystemIndices = { Party = 4 },
                EditModeUnitFrameSetting = { UseRaidStylePartyFrames = 4 },
            }

            partySystem = {
                systemInfo = {
                    settings = {
                        { setting = Enum.EditModeUnitFrameSetting.UseRaidStylePartyFrames, value = 1 },
                    },
                },
            }

            EditModeManagerFrame = {
                GetRegisteredSystemFrame = function()
                    return partySystem
                end,
            }
            "#,
        )
        .expect("install party system stub");

        patch(&env);

        let value: i32 = env
            .eval("return partySystem.systemInfo.settings[1].value")
            .expect("read party style setting");

        assert_eq!(
            value, 1,
            "post-event layout refresh must not force raid-style party frames off"
        );
    }
}
