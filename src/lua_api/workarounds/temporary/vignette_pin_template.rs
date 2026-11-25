//! Temporary Vignette data-provider pin-template workaround.
//!
//! Legacy profiles lack the Retail/Forever vignette model and retain the
//! nil-safe bridge for the original provider refresh path.

use crate::lua_api::WowLuaEnv;

const VIGNETTE_PIN_TEMPLATE_WORKAROUND_LUA: &str = r###"
local function __wow_patch_vignette_provider(provider)
    if type(provider) ~= "table" then
        return
    end
    if type(provider.GetPinTemplate) ~= "function" then
        return
    end
    if type(provider.GetDefaultPinTemplate) ~= "function" then
        return
    end
    if provider.__wow_ui_sim_nil_safe_get_pin_template then
        return
    end
    if provider:GetDefaultPinTemplate() ~= "VignettePinTemplate" then
        return
    end

    local original = provider.GetPinTemplate
    function provider:GetPinTemplate(vignetteInfo)
        if vignetteInfo == nil then
            return self:GetDefaultPinTemplate()
        end
        return original(self, vignetteInfo)
    end
    provider.__wow_ui_sim_nil_safe_get_pin_template = true
end

__wow_patch_vignette_provider(VignetteDataProviderMixin)

for _, mapName in ipairs({"WorldMapFrame", "BattlefieldMapFrame", "FlightMapFrame"}) do
    local map = _G[mapName]
    if map and type(map.dataProviders) == "table" then
        for provider in pairs(map.dataProviders) do
            __wow_patch_vignette_provider(provider)
        end
    end
end
"###;

pub(crate) fn patch(env: &WowLuaEnv) {
    let _ = env.exec(VIGNETTE_PIN_TEMPLATE_WORKAROUND_LUA);
}

#[cfg(all(
    test,
    not(feature = "client-retail"),
    not(feature = "client-wowforever")
))]
mod tests {
    use super::*;

    #[test]
    fn global_vignette_provider_returns_default_template_for_nil_info() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        install_provider(&env, "VignetteDataProviderMixin", "VignettePinTemplate");
        patch(&env);
        let (nil_template, explicit_template, patched): (String, String, bool) = env
            .eval("return VignetteDataProviderMixin:GetPinTemplate(nil), VignetteDataProviderMixin:GetPinTemplate({ pinTemplate = 'CustomPinTemplate' }), VignetteDataProviderMixin.__wow_ui_sim_nil_safe_get_pin_template == true")
            .expect("patched global vignette provider should be readable");
        assert_eq!(nil_template, "VignettePinTemplate");
        assert_eq!(explicit_template, "CustomPinTemplate");
        assert!(patched);
    }

    #[test]
    fn patches_vignette_provider_attached_to_map() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        install_provider(&env, "mapProvider", "VignettePinTemplate");
        env.exec("WorldMapFrame = { dataProviders = { [mapProvider] = true } }")
            .expect("map provider table should install");
        patch(&env);
        let nil_template: String = env
            .eval("return mapProvider:GetPinTemplate(nil)")
            .expect("patched map provider should be readable");
        assert_eq!(nil_template, "VignettePinTemplate");
    }

    #[test]
    fn ignores_non_vignette_default_templates() {
        let env = WowLuaEnv::new().expect("lua env should initialize");
        install_provider(&env, "VignetteDataProviderMixin", "OtherPinTemplate");
        patch(&env);
        let (nil_template, patched): (String, bool) = env
            .eval("return VignetteDataProviderMixin:GetPinTemplate(nil), VignetteDataProviderMixin.__wow_ui_sim_nil_safe_get_pin_template == true")
            .expect("unpatched non-vignette provider should be readable");
        assert_eq!(nil_template, "original:nil");
        assert!(!patched);
    }

    fn install_provider(env: &WowLuaEnv, name: &str, default_template: &str) {
        let lua = format!(
            "{name} = {{ GetDefaultPinTemplate = function() return '{default_template}' end, GetPinTemplate = function(self, vignetteInfo) if vignetteInfo == nil then return 'original:nil' end return vignetteInfo.pinTemplate end }}"
        );
        env.exec(&lua)
            .expect("vignette test provider should install");
    }
}
