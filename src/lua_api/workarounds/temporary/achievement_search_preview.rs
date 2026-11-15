//! Temporary AchievementUI search-preview surface repair.
//!
//! Blizzard_AchievementUI expects the search-preview array to be ready
//! after the addon loads. Keep the repair in this
//! addon-specific temporary owner instead of the generic runtime bootstrap.

const ACHIEVEMENT_SEARCH_PREVIEW_LUA: &str = r##"
local function ensure_search_previews()
    local frame = AchievementFrame
    local container = frame and frame.SearchPreviewContainer
    if type(container) ~= "table" and type(container) ~= "userdata" then
        return
    end

    local previews = container.searchPreviews
    if type(previews) ~= "table" then
        previews = {}
        container.searchPreviews = previews
    end

    local count = ACHIEVEMENT_FRAME_NUM_SEARCH_PREVIEWS or 5
    for index = 1, count do
        if previews[index] == nil then
            previews[index] = container["SearchPreview" .. index]
        end
    end
end

local function patch_search_preview_selection()
    if rawget(_G, "__wow_achievement_search_preview_patched") then
        return
    end
    if type(AchievementFrame_SetSearchPreviewSelection) ~= "function" then
        return
    end

    local original = AchievementFrame_SetSearchPreviewSelection
    AchievementFrame_SetSearchPreviewSelection = function(selectedIndex)
        ensure_search_previews()
        return original(selectedIndex)
    end
    __wow_achievement_search_preview_patched = true
end

ensure_search_previews()
patch_search_preview_selection()
"##;

pub(crate) fn apply_bootstrap(lua: &mut rilua::Lua) -> crate::Result<()> {
    lua.exec(ACHIEVEMENT_SEARCH_PREVIEW_LUA)?;
    Ok(())
}

/// Re-apply the search-preview repair after Blizzard_AchievementUI loads.
/// Wired through `apply_blizzard_post_load_patches`; the Lua
/// `hooksecurefunc(C_AddOns, "LoadAddOn", ...)` route no longer works because
/// the shared bootstrap hooksecurefunc deliberately refuses that target.
pub(crate) fn patch_for_addon_load(env: &crate::lua_api::LoaderEnv<'_>) -> crate::Result<()> {
    env.exec(ACHIEVEMENT_SEARCH_PREVIEW_LUA)
}

#[cfg(test)]
mod tests {
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn patches_achievement_search_preview_after_addon_load() {
        let env = WowLuaEnv::new().expect("lua env should initialize");

        // Install the pre-patch surface the way Blizzard_AchievementUI would
        // have left it, then run the same patch entry the loader fires from
        // apply_blizzard_post_load_patches. Loading the real addon here would
        // overwrite these probes with the genuine Blizzard functions.
        env.exec(
            r#"
                calls = {}
                AchievementFrame = {
                    SearchPreviewContainer = {
                        SearchPreview1 = "one",
                        SearchPreview2 = "two",
                    },
                }
                ACHIEVEMENT_FRAME_NUM_SEARCH_PREVIEWS = 2
                AchievementFrame_SetSearchPreviewSelection = function(selectedIndex)
                    calls[#calls + 1] = selectedIndex
                    return "selected"
                end
                AchievementFrameSummaryAchievementsEmptyText = {
                    shown = nil,
                    SetShown = function(self, shown) self.shown = shown end,
                }
                AchievementFrameSummaryAchievements = { buttons = {} }
                AchievementFrameSummary_UpdateAchievements = function(numAchievements)
                    return numAchievements
                end
                originalSummary = AchievementFrameSummary_UpdateAchievements
            "#,
        )
        .expect("achievement fixture should install");

        super::patch_for_addon_load(&env.loader_env())
            .expect("achievement post-load patch should apply");

        let result: String = env
            .eval(
                r#"
                local selectionResult = AchievementFrame_SetSearchPreviewSelection(2)
                local summaryResult = AchievementFrameSummary_UpdateAchievements(0)
                local previews = AchievementFrame.SearchPreviewContainer.searchPreviews

                if selectionResult ~= "selected" or summaryResult ~= 0 then return "return" end
                if calls[1] ~= 2 then return "selection" end
                if previews[1] ~= "one" or previews[2] ~= "two" then return "previews" end
                if AchievementFrameSummaryAchievementsEmptyText.shown ~= nil then return "summary_changed" end
                if AchievementFrameSummary_UpdateAchievements ~= originalSummary then return "summary_owner" end
                if not __wow_achievement_search_preview_patched then return "search_patch" end
                if __wow_achievement_summary_empty_text_patched ~= nil then return "summary_patch" end

                return "ok"
                "#,
            )
            .expect("achievement search preview probe should run");

        assert_eq!(result, "ok");
    }
}
