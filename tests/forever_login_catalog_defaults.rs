#![cfg(feature = "client-wowforever")]

use crate::common;
use wow_ui_sim::c_api::video_capabilities::VideoCapabilityBytes;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::settle_headless_startup;

fn load_login_screen() -> common::LockedEnv {
    load_login_screen_with_video_capabilities(None)
}

fn load_login_screen_with_video_capabilities(
    bytes: Option<VideoCapabilityBytes>,
) -> common::LockedEnv {
    common::lock_env(|| {
        let env = WowLuaEnv::new().expect("create Forever login environment");
        if let Some(bytes) = bytes {
            env.state().borrow_mut().video_capability_bytes = bytes;
        }
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
fn forever_host_credits_support_original_view_and_selector() {
    test_timeout! {
        let env = load_login_screen();
        // Explicit host fixture, not a reconstructed fresh native credits list.
        env.state().borrow_mut().credits.rows.push(wow_ui_sim::c_api::credits::CreditsRow {
            is_default: true, credits_file_id: 17,
            key_art: "CreditsScreen-Keyart-0".into(),
            background: "CreditsScreen-Background-0".into(),
            name: "Host credits".into(), ..Default::default()
        });
        env.state().borrow_mut().credits.texts.insert(17, "<html><body><p>Host-supplied credits.</p></body></html>".into());
        env.exec(r#"
            local catalog = GetCreditsDisplayInfo()
            assert(type(catalog) == "table" and #catalog == 1)
            local entry = catalog[1]
            assert(entry.isDefault == true)
            assert(type(entry.creditsFileID) == "number")
            assert(type(entry.keyArt) == "string" and type(entry.background) == "string")
            assert(entry.name == "Host credits" and entry.logo == nil)
            local text = GetCreditsTextByFileDataID(entry.creditsFileID)
            assert(type(text) == "string" and text:find("Host-supplied credits", 1, true))
            catalog[1].name = "changed"
            assert(GetCreditsDisplayInfo()[1].name == "Host credits")

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
        "#).expect("original Forever credits view and selector should accept the supplied host catalog");

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
fn forever_graphics_settings_skip_controls_for_explicit_host_capability_bytes() {
    test_timeout! {
        // INFERRED host behavior, not native default capability assertions.
        let env = load_login_screen_with_video_capabilities(Some(VideoCapabilityBytes {
            linear_enabled_on_start: 0,
            outline_disabled: 1,
            pbr_water_supported: 0,
            secondary_lighting_supported: 0,
            spell_visual_density_supported: 0,
        }));
        env.exec(r#"
            -- Separate unchanged simulator MSAA policy, not a native false claim.
            assert(select('#', MultiSampleAntiAliasingSupported()) == 1)
            assert(MultiSampleAntiAliasingSupported() == false)
            assert(Settings.GetSetting("PROXY_MSAA") ~= nil)
            assert(Settings.GetSetting("PROXY_MSAA_ALPHA") ~= nil)
            assert(C_VideoOptions.IsPBRWaterSupported() == false)
            assert(C_VideoOptions.IsOutlineModeSupported() == false)
            assert(C_VideoOptions.IsSecondaryLightingSupported() == false)
            assert(C_VideoOptions.IsLinearEnabledOnStart() == false)
            assert(C_VideoOptions.IsSpellVisualDensitySystemSupported() == false)
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
                assert(added[prefix .. "SpellDensity"] == nil)
            end
        "#).expect("original Forever graphics controls should honor explicit host bytes");

        let errors: Vec<_> = env.state().borrow().lua_errors.iter()
            .filter(|error| error.contains("Blizzard_SettingsDefinitions_Shared/Graphics.lua"))
            .cloned()
            .collect();
        assert!(errors.is_empty(), "offline graphics definition errors: {errors:#?}");
    }
}

#[test]
fn forever_graphics_settings_include_controls_for_explicit_host_capability_bytes() {
    test_timeout! {
        // Enable all five host projections, including the inverted outline byte.
        let env = load_login_screen_with_video_capabilities(Some(VideoCapabilityBytes {
            linear_enabled_on_start: 2,
            outline_disabled: 0,
            pbr_water_supported: 128,
            secondary_lighting_supported: 255,
            spell_visual_density_supported: 3,
        }));
        env.exec(r#"
            assert(C_VideoOptions.IsLinearEnabledOnStart() == true)
            assert(C_VideoOptions.IsOutlineModeSupported() == true)
            assert(C_VideoOptions.IsPBRWaterSupported() == true)
            assert(C_VideoOptions.IsSecondaryLightingSupported() == true)
            assert(C_VideoOptions.IsSpellVisualDensitySystemSupported() == true)
            assert(Settings.GetSetting("Gamma") ~= nil)
            assert(Settings.GetSetting("bloomUserMult") ~= nil)

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
                assert(added[prefix .. "PBRLiquidDetail"] == true)
                assert(added[prefix .. "OutlineMode"] == true)
                assert(added[prefix .. "LightMode"] == true)
                assert(added[prefix .. "SpellDensity"] == true)
            end
        "#).expect("original Forever graphics controls should honor explicit host bytes");

        let errors: Vec<_> = env.state().borrow().lua_errors.iter()
            .filter(|error| error.contains("Blizzard_SettingsDefinitions_Shared/Graphics.lua"))
            .cloned()
            .collect();
        assert!(errors.is_empty(), "host-enabled graphics definition errors: {errors:#?}");
    }
}
