#![cfg(feature = "client-wowforever")]

use crate::common;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::settle_headless_startup;

fn load_login_screen() -> common::LockedEnv {
    common::lock_env(|| {
        let env = WowLuaEnv::new().expect("create Forever login environment");
        env.set_screen_size(1024.0, 768.0);
        env.set_screen_mode(ScreenKind::Login);
        let ui = wow_ui_sim::client_profile::blizzard_ui_addons_dir();
        for (name, toc_path) in discover_blizzard_addons_for_screen(&ui, ScreenKind::Login) {
            load_addon(&env.loader_env(), &toc_path)
                .unwrap_or_else(|error| panic!("load {name}: {error}"));
        }
        env.apply_post_load_workarounds();
        settle_headless_startup(&env);
        env
    })
}

#[test]
fn forever_offline_credits_support_original_view_and_selector() {
    test_timeout! {
        let env = load_login_screen();
        env.exec(r#"
            local catalog = GetCreditsDisplayInfo()
            assert(type(catalog) == "table" and #catalog == 1)
            local entry = catalog[1]
            assert(entry.isDefault == true)
            assert(type(entry.creditsFileID) == "number")
            assert(type(entry.keyArt) == "string" and type(entry.background) == "string")
            assert(entry.name == "Offline simulator" and type(entry.logo) == "number")
            local text = GetCreditsTextByFileDataID(entry.creditsFileID)
            assert(type(text) == "string" and text:find("offline simulator", 1, true))
            assert(GetCreditsTextByFileDataID(-1) == nil)
            catalog[1].name = "changed"
            assert(GetCreditsDisplayInfo()[1].name == "Offline simulator")

            CreditsFrame:OnLoad()
            assert(CreditsFrame.selection == 1)
            CreditsFrame:Show()
            CreditsFrame:Update()
            CreditsFrame:ToggleExpansionList()
            assert(CreditsFrame.ExpansionList:IsShown())
            CreditsFrame:ToggleExpansionList()
            assert(not CreditsFrame.ExpansionList:IsShown())
            HideCursor()
            assert(IsCursorVisible() == false)
            CreditsFrame:Hide()
            assert(IsCursorVisible() == true)
            HideCursor()
            assert(IsCursorVisible() == false)
            ShowCursor()
            assert(IsCursorVisible() == true)
        "#).expect("original Forever credits view and selector should accept the offline catalog");

        let errors: Vec<_> = env.state().borrow().lua_errors.iter()
            .filter(|error| error.contains("CreditsFrame") || error.contains("GetCreditsDisplayInfo"))
            .cloned()
            .collect();
        assert!(errors.is_empty(), "offline credits errors: {errors:#?}");
    }
}

#[test]
fn forever_high_res_capability_is_read_only_and_graphics_consumer_loads() {
    test_timeout! {
        let env = load_login_screen();
        env.exec(r#"
            assert(type(C_VideoOptions.AreHighResTexturesAvailable()) == 'boolean')
            local category, layout = Settings.RegisterVerticalLayoutCategory("High-resolution textures")
            GraphicsOverrides.CreateHiResOptions(category, layout)
        "#).expect("original graphics consumer accepts the modeled capability");
    }
}

#[test]
fn forever_offline_graphics_settings_skip_unsupported_3d_controls() {
    test_timeout! {
        let env = load_login_screen();
        env.exec(r#"
            assert(select('#', MultiSampleAntiAliasingSupported()) == 1)
            assert(MultiSampleAntiAliasingSupported() == false)
            assert(Settings.GetSetting("PROXY_MSAA") ~= nil)
            assert(Settings.GetSetting("PROXY_MSAA_ALPHA") ~= nil)
            assert(C_VideoOptions.IsPBRWaterSupported() == false)
            assert(C_VideoOptions.IsOutlineModeSupported() == false)
            assert(C_VideoOptions.IsSecondaryLightingSupported() == false)
            assert(C_VideoOptions.IsLinearEnabledOnStart() == false)
            assert(Settings.GetSetting("Gamma") ~= nil)
            assert(Settings.GetSetting("bloomUserMult") == nil)

            local added = {}
            local function RecordSetting(settings, category, variable)
                added[variable] = true
                settings[#settings + 1] = variable
            end
            local normal = GraphicsOverrides.CreateAdvancedSettingsTable({}, RecordSetting)
            local raid = GraphicsOverrides.CreateAdvancedRaidSettingsTable({}, RecordSetting)
            assert(#normal > 0 and #raid > 0)
            for _, prefix in ipairs({ "graphics", "raidGraphics" }) do
                assert(added[prefix .. "LiquidDetail"] == true)
                assert(added[prefix .. "TextureResolution"] == true)
                assert(added[prefix .. "PBRLiquidDetail"] == nil)
                assert(added[prefix .. "OutlineMode"] == nil)
                assert(added[prefix .. "LightMode"] == nil)
            end
        "#).expect("original Forever graphics controls should honor unsupported 3D capabilities");

        let errors: Vec<_> = env.state().borrow().lua_errors.iter()
            .filter(|error| error.contains("Blizzard_SettingsDefinitions_Shared/Graphics.lua"))
            .cloned()
            .collect();
        assert!(errors.is_empty(), "offline graphics definition errors: {errors:#?}");
    }
}
