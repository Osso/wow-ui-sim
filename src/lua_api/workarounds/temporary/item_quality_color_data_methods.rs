//! Temporary item-quality color data method repair.
//!
//! ITEM_QUALITY_COLORS entries are plain tables during startup. Keep these
//! GetRGB/GetRGBA method repairs isolated until the color data surface is
//! registered with the same methods Blizzard expects.

use crate::lua_api::LoaderEnv;

const ITEM_QUALITY_COLOR_DATA_METHODS_WORKAROUND_LUA: &str = r#"
if rawget(_G, "__wow_item_quality_color_data_methods_wrapped") then
    return
end

local function ensureColorDataMethods(colorData)
    if type(colorData) ~= "table" then
        return
    end

    if type(colorData.GetRGB) ~= "function" then
        function colorData:GetRGB()
            return self.r, self.g, self.b
        end
    end

    if type(colorData.GetRGBA) ~= "function" then
        function colorData:GetRGBA()
            return self.r, self.g, self.b, self.a or 1
        end
    end
end

local function ensureAllItemQualityColorMethods()
    if type(ITEM_QUALITY_COLORS) ~= "table" then
        return
    end

    for _, colorData in pairs(ITEM_QUALITY_COLORS) do
        ensureColorDataMethods(colorData)
    end
end

ensureAllItemQualityColorMethods()

if type(ColorManager) == "table" and type(ColorManager.UpdateColorsForItemQuality) == "function" then
    local originalUpdateColorsForItemQuality = ColorManager.UpdateColorsForItemQuality
    function ColorManager.UpdateColorsForItemQuality(...)
        local result = originalUpdateColorsForItemQuality(...)
        ensureAllItemQualityColorMethods()
        return result
    end
end

rawset(_G, "__wow_item_quality_color_data_methods_wrapped", true)
"#;

pub(crate) fn patch(env: &LoaderEnv<'_>) {
    let _ = env.exec(ITEM_QUALITY_COLOR_DATA_METHODS_WORKAROUND_LUA);
}

#[cfg(all(
    test,
    not(any(feature = "client-retail", feature = "client-wowforever"))
))]
mod tests {
    use super::*;
    use crate::lua_api::WowLuaEnv;

    #[test]
    fn adds_rgb_and_rgba_methods_to_existing_item_quality_colors() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            ITEM_QUALITY_COLORS = {
                [1] = { r = 0.1, g = 0.2, b = 0.3, a = 0.4 },
                [2] = { r = 0.5, g = 0.6, b = 0.7 },
            }
            "#,
        )
        .expect("item quality colors should install");

        patch(&env.loader_env());

        let (r, g, b, a): (f64, f64, f64, f64) = env
            .eval(
                r#"
                local r, g, b = ITEM_QUALITY_COLORS[1]:GetRGB()
                local _, _, _, a = ITEM_QUALITY_COLORS[2]:GetRGBA()
                return r, g, b, a
                "#,
            )
            .expect("item quality color methods should be readable");

        assert_eq!((r, g, b, a), (0.1, 0.2, 0.3, 1.0));
    }

    #[test]
    fn color_manager_update_repairs_new_item_quality_color_tables() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        env.exec(
            r#"
            ITEM_QUALITY_COLORS = {}
            ColorManager = {
                UpdateColorsForItemQuality = function()
                    ITEM_QUALITY_COLORS[3] = { r = 0.8, g = 0.7, b = 0.6, a = 0.5 }
                    return "updated"
                end,
            }
            "#,
        )
        .expect("color manager should install");

        patch(&env.loader_env());

        let (result, r, g, b, a): (String, f64, f64, f64, f64) = env
            .eval(
                r#"
                local result = ColorManager.UpdateColorsForItemQuality()
                local r, g, b, a = ITEM_QUALITY_COLORS[3]:GetRGBA()
                return result, r, g, b, a
                "#,
            )
            .expect("color-manager repaired color methods should be readable");

        assert_eq!(result, "updated");
        assert_eq!((r, g, b, a), (0.8, 0.7, 0.6, 0.5));
    }
}
