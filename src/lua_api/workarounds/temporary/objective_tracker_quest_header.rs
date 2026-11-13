//! Temporary Objective Tracker quest-header surface workaround.
//!
//! Legacy profiles retain the existing quest-header surface workaround.
//! Retail uses the original selected Lua/XML animation without forced Stop/alpha.
//! Startup can leave the quest header partially faded after its add animation.
//! Keep this isolated until Objective Tracker layout/animation state is modeled
//! closely enough that the Blizzard code reaches the same visible surface.

use crate::lua_api::WowLuaEnv;

const OBJECTIVE_TRACKER_QUEST_HEADER_WORKAROUND_LUA: &str = r#"
local function module_has_visible_contents(module)
    if not module then
        return false
    end
    if module.GetContentsHeight and module:GetContentsHeight() > 0 then
        return true
    end
    local used = module.usedBlocks
    if type(used) ~= "table" then
        return false
    end
    for _, blocks in pairs(used) do
        if type(blocks) == "table" and next(blocks) ~= nil then
            return true
        end
    end
    return false
end

local function resolve_quest_header()
    if QuestObjectiveTracker and QuestObjectiveTracker.Header then
        return QuestObjectiveTracker.Header
    end
    local legacy = ObjectiveTrackerBlocksFrame and ObjectiveTrackerBlocksFrame.QuestHeader
    if not legacy then
        return nil
    end
    return legacy.Header or legacy
end

local function set_region_alpha(region, target_alpha)
    if type(region) ~= "table"
        or type(region.GetAlpha) ~= "function"
        or type(region.SetAlpha) ~= "function" then
        return
    end
    local current = region:GetAlpha()
    if type(current) ~= "number" or math.abs(current - target_alpha) > 0.001 then
        region:SetAlpha(target_alpha)
    end
end

local function normalize_quest_header_surface(module, header)
    if type(header) ~= "table" then
        return
    end
    -- During startup the AddAnim can leave the quest header in a half-faded
    -- state (dim background + fully lit shine/glow). Force the expanded visuals
    -- once the module has quest content so the texture stack matches Blizzard.
    if module and module.collapsed then
        return
    end
    if not module_has_visible_contents(module) then
        return
    end
    if type(header.AddAnim) == "table" and type(header.AddAnim.Stop) == "function" then
        header.AddAnim:Stop()
    end
    set_region_alpha(header, 1)
    set_region_alpha(header.Background, 1)
    set_region_alpha(header.Shine, 0)
    set_region_alpha(header.Glow, 0)
    set_region_alpha(header.MinimizeButton, 1)
end

local function ensure_quest_header_text()
    local module = QuestObjectiveTracker
    local header = resolve_quest_header()
    local textRegion = header and header.Text
    if not textRegion then
        return
    end

    normalize_quest_header_surface(module, header)

    if header.Show and module_has_visible_contents(module) and not header:IsShown() then
        header:Show()
    end

    local text = textRegion.GetText and textRegion:GetText() or nil
    if type(text) ~= "string" or text == "" then
        local fallback = TRACKER_HEADER_QUESTS or QUESTS or "Quests"
        textRegion:SetText(fallback)
    end

    if textRegion.Show and not textRegion:IsShown() then
        textRegion:Show()
    end

    if textRegion.GetAlpha and textRegion.SetAlpha and textRegion:GetAlpha() <= 0 then
        textRegion:SetAlpha(1)
    end

    if textRegion.GetTextColor and textRegion.SetTextColor then
        local r, g, b, a = textRegion:GetTextColor()
        local effectively_black = (r or 0) < 0.02 and (g or 0) < 0.02 and (b or 0) < 0.02
        local fully_transparent = a ~= nil and a <= 0
        if effectively_black or fully_transparent then
            local color =
                (type(OBJECTIVE_TRACKER_COLOR) == "table" and OBJECTIVE_TRACKER_COLOR["Header"])
                or NORMAL_FONT_COLOR
            if type(color) == "table" and color.r and color.g and color.b then
                textRegion:SetTextColor(color.r, color.g, color.b, color.a or 1)
            elseif fully_transparent then
                textRegion:SetTextColor(r or 1, g or 0.82, b or 0.0, 1)
            end
        end
    end
    normalize_quest_header_surface(module, header)
end

if not rawget(_G, "__wow_objective_tracker_quest_header_update_wrapper")
    and ObjectiveTrackerContainerMixin
    and type(ObjectiveTrackerContainerMixin.Update) == "function" then
    local originalUpdate = ObjectiveTrackerContainerMixin.Update
    ObjectiveTrackerContainerMixin.Update = function(self, dirtyUpdate)
        local result = originalUpdate(self, dirtyUpdate)
        pcall(ensure_quest_header_text)
        return result
    end
    rawset(_G, "__wow_objective_tracker_quest_header_update_wrapper", true)
end

if not rawget(_G, "__wow_objective_tracker_header_play_add_anim_wrapper")
    and ObjectiveTrackerModuleHeaderMixin
    and type(ObjectiveTrackerModuleHeaderMixin.PlayAddAnimation) == "function" then
    local originalPlayAddAnimation = ObjectiveTrackerModuleHeaderMixin.PlayAddAnimation
    ObjectiveTrackerModuleHeaderMixin.PlayAddAnimation = function(self, ...)
        local result = originalPlayAddAnimation(self, ...)
        local module = self.GetParent and self:GetParent() or nil
        if module == QuestObjectiveTracker then
            pcall(normalize_quest_header_surface, module, self)
        end
        return result
    end
    rawset(_G, "__wow_objective_tracker_header_play_add_anim_wrapper", true)
end

if not rawget(_G, "__wow_objective_tracker_module_end_layout_wrapper")
    and ObjectiveTrackerModuleMixin
    and type(ObjectiveTrackerModuleMixin.EndLayout) == "function" then
    local originalEndLayout = ObjectiveTrackerModuleMixin.EndLayout
    ObjectiveTrackerModuleMixin.EndLayout = function(self, ...)
        local result = originalEndLayout(self, ...)
        if self == QuestObjectiveTracker then
            pcall(normalize_quest_header_surface, self, self.Header)
        end
        return result
    end
    rawset(_G, "__wow_objective_tracker_module_end_layout_wrapper", true)
end

pcall(ensure_quest_header_text)
"#;

pub(crate) fn patch(env: &WowLuaEnv) {
    // Retail's original PlayAddAnimation restarts the XML AddAnim. Applying this
    // surface override immediately stops that animation and forces terminal alphas.
    // Do not replace the selected original consumer with a guessed presentation.
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Retail {
        let _ = env.exec(OBJECTIVE_TRACKER_QUEST_HEADER_WORKAROUND_LUA);
    }
}

// These synthetic tests preserve only the unchanged legacy workaround, not
// a native animation contract. Retail is covered by the original-consumer test.
#[cfg(all(test, not(feature = "client-retail")))]
mod tests {
    use super::*;

    const REGION_HELPERS_LUA: &str = r#"
        TRACKER_HEADER_QUESTS = "Quests"
        NORMAL_FONT_COLOR = { r = 1, g = 0.82, b = 0, a = 1 }

        local function region(alpha)
            return {
                alpha = alpha,
                GetAlpha = function(self)
                    return self.alpha
                end,
                SetAlpha = function(self, value)
                    self.alpha = value
                end,
            }
        end
        __test_objective_tracker_region = region
    "#;

    const TEXT_REGION_LUA: &str = r#"
        local text = __test_objective_tracker_region(0)
        text.text = ""
        text.shown = false
        text.r = 0
        text.g = 0
        text.b = 0
        text.a = 0
        text.GetText = function(self)
            return self.text
        end
        text.SetText = function(self, value)
            self.text = value
        end
        text.Show = function(self)
            self.shown = true
        end
        text.IsShown = function(self)
            return self.shown
        end
        text.GetTextColor = function(self)
            return self.r, self.g, self.b, self.a
        end
        text.SetTextColor = function(self, r, g, b, a)
            self.r = r
            self.g = g
            self.b = b
            self.a = a
        end
        __test_objective_tracker_text = text
    "#;

    const HEADER_METHODS_LUA: &str = r#"
        header.AddAnim = {
            Stop = function(self)
                self.stopped = true
            end,
        }
        header.Show = function(self)
            self.shown = true
        end
        header.IsShown = function(self)
            return self.shown
        end
        header.GetParent = function()
            return QuestObjectiveTracker
        end
    "#;

    #[test]
    fn seeds_visible_populated_quest_header_surface() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        install_objective_tracker_surface(&env, "false", "0", "0", "0", "0");

        patch(&env);

        let (header_shown, text, text_shown, text_alpha, header_alpha, background_alpha): (
            bool,
            String,
            bool,
            i64,
            i64,
            i64,
        ) = env
            .eval(
                r#"
                return QuestObjectiveTracker.Header.shown,
                    QuestObjectiveTracker.Header.Text.text,
                    QuestObjectiveTracker.Header.Text.shown,
                    QuestObjectiveTracker.Header.Text.alpha,
                    QuestObjectiveTracker.Header.alpha,
                    QuestObjectiveTracker.Header.Background.alpha
                "#,
            )
            .expect("quest header state should be readable");

        assert!(header_shown);
        assert_eq!(text, "Quests");
        assert!(text_shown);
        assert_eq!(text_alpha, 1);
        assert_eq!(header_alpha, 1);
        assert_eq!(background_alpha, 1);
    }

    #[test]
    fn wrappers_normalize_header_after_blizzard_calls() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        install_objective_tracker_surface(&env, "false", "0", "1", "1", "0");
        env.exec(
            r#"
            ObjectiveTrackerContainerMixin = {
                Update = function()
                    return "updated"
                end,
            }
            ObjectiveTrackerModuleHeaderMixin = {
                PlayAddAnimation = function(self)
                    self.played = true
                    return "played"
                end,
            }
            ObjectiveTrackerModuleMixin = {
                EndLayout = function(self)
                    self.ended = true
                    return "ended"
                end,
            }
            "#,
        )
        .expect("objective tracker wrapper mixins should install");

        patch(&env);

        let (update_result, play_result, layout_result, stopped, shine_alpha, glow_alpha): (
            String,
            String,
            String,
            bool,
            i64,
            i64,
        ) = env
            .eval(
                r#"
                local updateResult = ObjectiveTrackerContainerMixin.Update({})
                QuestObjectiveTracker.Header.Shine.alpha = 1
                QuestObjectiveTracker.Header.Glow.alpha = 1
                local playResult = ObjectiveTrackerModuleHeaderMixin.PlayAddAnimation(QuestObjectiveTracker.Header)
                QuestObjectiveTracker.Header.Shine.alpha = 1
                QuestObjectiveTracker.Header.Glow.alpha = 1
                local layoutResult = ObjectiveTrackerModuleMixin.EndLayout(QuestObjectiveTracker)
                return updateResult,
                    playResult,
                    layoutResult,
                    QuestObjectiveTracker.Header.AddAnim.stopped,
                    QuestObjectiveTracker.Header.Shine.alpha,
                    QuestObjectiveTracker.Header.Glow.alpha
                "#,
            )
            .expect("wrapped objective tracker behavior should be readable");

        assert_eq!(update_result, "updated");
        assert_eq!(play_result, "played");
        assert_eq!(layout_result, "ended");
        assert!(stopped);
        assert_eq!(shine_alpha, 0);
        assert_eq!(glow_alpha, 0);
    }

    #[test]
    fn collapsed_or_empty_modules_keep_header_surface_unchanged() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        install_objective_tracker_surface(&env, "true", "0", "0", "1", "1");
        env.exec("QuestObjectiveTracker.usedBlocks = {}")
            .expect("quest tracker contents should be cleared");

        patch(&env);

        let (header_shown, header_alpha, shine_alpha, stopped): (bool, i64, i64, bool) = env
            .eval(
                r#"
                return QuestObjectiveTracker.Header.shown,
                    QuestObjectiveTracker.Header.alpha,
                    QuestObjectiveTracker.Header.Shine.alpha,
                    QuestObjectiveTracker.Header.AddAnim.stopped == true
                "#,
            )
            .expect("collapsed quest header state should be readable");

        assert!(!header_shown);
        assert_eq!(header_alpha, 0);
        assert_eq!(shine_alpha, 1);
        assert!(!stopped);
    }

    fn install_objective_tracker_surface(
        env: &WowLuaEnv,
        collapsed: &str,
        contents_height: &str,
        header_alpha: &str,
        shine_alpha: &str,
        glow_alpha: &str,
    ) {
        env.exec(REGION_HELPERS_LUA)
            .expect("objective tracker region helper should install");
        env.exec(TEXT_REGION_LUA)
            .expect("objective tracker text region should install");
        install_objective_tracker_header(env, header_alpha, shine_alpha, glow_alpha);
        install_objective_tracker_module(env, collapsed, contents_height);
    }

    fn install_objective_tracker_header(
        env: &WowLuaEnv,
        header_alpha: &str,
        shine_alpha: &str,
        glow_alpha: &str,
    ) {
        let lua = format!(
            r#"
            header = __test_objective_tracker_region({header_alpha})
            header.shown = false
            header.Text = __test_objective_tracker_text
            header.Background = __test_objective_tracker_region(0)
            header.Shine = __test_objective_tracker_region({shine_alpha})
            header.Glow = __test_objective_tracker_region({glow_alpha})
            header.MinimizeButton = __test_objective_tracker_region(0)
            {HEADER_METHODS_LUA}
            __test_objective_tracker_header = header
            "#,
        );
        env.exec(&lua)
            .expect("objective tracker header should install");
    }

    fn install_objective_tracker_module(env: &WowLuaEnv, collapsed: &str, contents_height: &str) {
        let lua = format!(
            r#"
            QuestObjectiveTracker = {{}}
            QuestObjectiveTracker.collapsed = {collapsed}
            QuestObjectiveTracker.Header = __test_objective_tracker_header
            QuestObjectiveTracker.usedBlocks = {{}}
            QuestObjectiveTracker.usedBlocks.quests = {{}}
            QuestObjectiveTracker.usedBlocks.quests.first = true
            QuestObjectiveTracker.GetContentsHeight = function()
                return {contents_height}
            end
            "#,
        );
        env.exec(&lua)
            .expect("objective tracker module should install");
    }
}
