//! Pinned 70268 localization through the selected original Mists settings consumer.
#![cfg(feature = "client-mists")]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::loader::find_toc_file;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

#[test]
fn original_mists_display_settings_reads_70268_enemy_castbar_name() {
    assert_eq!(ACTIVE, ClientProfile::Mists);
    let ui = blizzard_ui_dir();
    let toc_path = find_toc_file(&ui.join("Blizzard_SettingsDefinitions_Frame")).unwrap();
    assert_eq!(
        toc_path.file_name().unwrap(),
        "Blizzard_SettingsDefinitions_Frame_Classic.toc"
    );
    let toc = TocFile::from_file(&toc_path).unwrap();
    assert!(toc.allows_screen(ScreenKind::Game));
    assert!(!toc.is_game_type_restricted());
    assert_eq!(
        toc.files[1],
        std::path::Path::new("Classic/InterfaceOverrides.lua")
    );
    let env = new_blizzard_addon_env(&ui);
    // Supply only the preceding availability branch. Engraving uses the seeded
    // Mists native defaults, so its original line 138 no longer stops this probe.
    env.state().borrow_mut().transmog_availability.enabled = Some(true);
    load_blizzard_addon_closure_into_env(
        &env,
        &ui,
        &[
            "Blizzard_SharedXMLBase",
            "Blizzard_SharedXML",
            "Blizzard_SharedXMLGame",
            "Blizzard_LoadLocale",
            "Blizzard_SettingsDefinitions_Frame",
        ],
        &[],
    );
    let errors = env.state().borrow().lua_errors.clone();
    assert!(errors.is_empty(), "{errors:#?}");
    env.exec(
        r#"
        local function original(func, suffix, line)
            local info = debug.getinfo(func, 'S')
            local source = info.source:gsub('\\', '/')
            assert(info.what == 'Lua' and info.linedefined == line, source)
            assert(source:sub(-#suffix) == suffix, source)
        end
        original(InterfaceOverrides.AdjustDisplaySettings,
            'Blizzard_SettingsDefinitions_Frame/Classic/InterfaceOverrides.lua', 3)
        original(Settings.SetupCVarCheckbox,
            'Blizzard_Settings_Shared/Blizzard_Settings.lua', 561)
        original(CVarSettingMixin.Init,
            'Blizzard_Settings_Shared/Blizzard_Setting.lua', 273)
        MistsStringsCategory, MistsStringsLayout =
            Settings.RegisterVerticalLayoutCategory('70268 global strings probe')
        MistsDisplayLines = {}
        local display = InterfaceOverrides.AdjustDisplaySettings
        -- Observe, but never replace or stop, the original consumer.
        debug.sethook(function(_, line)
            if debug.getinfo(2, 'f').func == display then
                MistsDisplayLines[line] = true
            end
        end, 'l')
        "#,
    )
    .unwrap();
    // The seeded Mists engraving defaults let the original consumer pass line 138.
    env.exec("InterfaceOverrides.AdjustDisplaySettings(MistsStringsCategory)")
        .expect("seeded engraving availability advances the original consumer past line 138");
    env.exec("debug.sethook()").unwrap();
    env.exec(
        r#"
        assert(MistsDisplayLines[131] and MistsDisplayLines[138])
        assert(SHOW_ENEMY_CASTBARS == 'Show Enemy Cast Bar') -- GlobalStrings ID 51586
        assert(OPTION_TOOLTIP_SHOW_ENEMY_CASTBARS == nil)
        local setting = Settings.GetSetting('showTargetCastbar')
        assert(setting:GetName() == 'Show Enemy Cast Bar')
        assert(setting:GetVariableType() == Settings.VarType.Boolean)
        assert(Settings.GetSetting('showDynamicBuffSize') ~= nil)
        assert(Settings.GetSetting('unitFramesDisplayIncomingHeals') ~= nil)
        "#,
    )
    .expect("original settings registered the concrete 70268 name and advanced past 131");
}
