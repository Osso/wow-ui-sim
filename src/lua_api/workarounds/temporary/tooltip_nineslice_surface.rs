//! Temporary tooltip NineSlice surface repair.
//!
//! Some startup tooltip paths run before the simulator has created the
//! NineSlice child surfaces that Blizzard styling helpers expect.

use crate::lua_api::WowLuaEnv;

const TOOLTIP_NINESLICE_SURFACE_LUA: &str = r#"
local function ensure_tooltip_nineslice(tooltip)
    if type(tooltip) ~= "table" or tooltip.NineSlice ~= nil then
        return
    end

    if type(CreateFrame) ~= "function" or type(NineSliceUtil) ~= "table" then
        return
    end

    local nineSlice = CreateFrame("Frame", nil, tooltip, "NineSlicePanelTemplate")
    if nineSlice == nil then
        return
    end

    tooltip.NineSlice = nineSlice
    if type(nineSlice.SetParentKey) == "function" then
        pcall(nineSlice.SetParentKey, nineSlice, "NineSlice", true)
    end
    if type(NineSliceUtil.DisableSharpening) == "function" then
        NineSliceUtil.DisableSharpening(nineSlice)
    end
    if type(SharedTooltip_SetBackdropStyle) == "function" then
        pcall(SharedTooltip_SetBackdropStyle, tooltip, nil, false)
    end
end

ensure_tooltip_nineslice(GameTooltip)
ensure_tooltip_nineslice(GlueTooltip)
"#;

pub(crate) fn patch(env: &WowLuaEnv) {
    let _ = env.exec(TOOLTIP_NINESLICE_SURFACE_LUA);
}

#[cfg(all(
    test,
    not(any(feature = "client-retail", feature = "client-wowforever"))
))]
mod tests {
    use super::*;

    #[test]
    fn creates_missing_tooltip_nineslice_surfaces() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            created = {}
            sharpened = {}
            styled = {}
            GameTooltip = {}
            GlueTooltip = {}
            NineSliceUtil = {
                DisableSharpening = function(frame)
                    sharpened[frame] = true
                end,
            }
            SharedTooltip_SetBackdropStyle = function(tooltip)
                styled[tooltip] = true
            end
            CreateFrame = function(frameType, name, parent, template)
                local frame = {
                    frameType = frameType,
                    parent = parent,
                    template = template,
                    SetParentKey = function(self, key, isGlobal)
                        self.parentKey = key
                        self.parentKeyIsGlobal = isGlobal
                    end,
                }
                table.insert(created, frame)
                return frame
            end
            "#,
        )
        .expect("tooltip fixture should install");

        patch(&env);

        let (
            created_count,
            game_has_nineslice,
            glue_has_nineslice,
            parent_key,
            template,
            sharpened_game,
            styled_game,
        ): (i64, bool, bool, String, String, bool, bool) = env
            .eval(
                r#"
                return #created,
                    GameTooltip.NineSlice ~= nil,
                    GlueTooltip.NineSlice ~= nil,
                    GameTooltip.NineSlice.parentKey,
                    GameTooltip.NineSlice.template,
                    sharpened[GameTooltip.NineSlice] == true,
                    styled[GameTooltip] == true
                "#,
            )
            .expect("tooltip nineslice surfaces should be readable");

        assert_eq!(created_count, 2);
        assert!(game_has_nineslice);
        assert!(glue_has_nineslice);
        assert_eq!(parent_key, "NineSlice");
        assert_eq!(template, "NineSlicePanelTemplate");
        assert!(sharpened_game);
        assert!(styled_game);
    }

    #[test]
    fn preserves_existing_tooltip_nineslice() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            existing = {}
            create_calls = 0
            GameTooltip = { NineSlice = existing }
            GlueTooltip = nil
            NineSliceUtil = {}
            CreateFrame = function()
                create_calls = create_calls + 1
                return {}
            end
            "#,
        )
        .expect("tooltip fixture should install");

        patch(&env);

        let (same_nineslice, create_calls): (bool, i64) = env
            .eval("return GameTooltip.NineSlice == existing, create_calls")
            .expect("existing tooltip nineslice should be readable");

        assert!(same_nineslice);
        assert_eq!(create_calls, 0);
    }
}
