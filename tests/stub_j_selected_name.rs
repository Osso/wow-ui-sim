//! J retirement: preserve the original selected-name consumer on actual XML
//! frames. Behavior checks are preservation; only ownership is retirement RED.
#![cfg(all(feature = "optional-tests", feature = "client-retail"))]

use crate::common;
use crate::common::blizzard_addon_harness::blizzard_ui_dir;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::settle_headless_startup;

fn original_selected_name_env(empty_roster: bool) -> common::LockedEnv {
    common::lock_env(move || {
        let ui = blizzard_ui_dir();
        let toc = find_toc_file(&ui.join("Blizzard_GlueXML")).unwrap();
        assert!(toc.ends_with("Blizzard_GlueXML_Mainline.toc"));
        let contents = std::fs::read_to_string(&toc).unwrap();
        assert!(contents.contains("## AllowLoad: Glue"));
        assert!(contents.contains("## AllowLoadGameType: mainline"));
        assert!(contents.contains("Mainline\\CharacterSelect.lua"));
        assert!(contents.contains("Mainline\\CharacterSelect.xml"));
        let env = WowLuaEnv::new().unwrap();
        // Direct-screen startup's roster is the existing social host fixture.
        // Empty host input is not a replacement native API or simulated event.
        if empty_roster {
            env.state().borrow_mut().bnet_friends.clear();
        }
        env.set_screen_size(1024.0, 768.0);
        env.set_screen_mode(ScreenKind::CharacterSelect);
        let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::CharacterSelect);
        assert!(addons.iter().any(|(name, path)| name == "Blizzard_GlueXML" && path == &toc));
        for (name, path) in addons {
            load_addon(&env.loader_env(), &path)
                .unwrap_or_else(|error| panic!("[load {name}] {error}"));
        }
        env.apply_post_load_workarounds();
        settle_headless_startup(&env);
        env.exec(r#"
            assert(CharSelectCharacterName and CharacterSelect and GlueTooltip,
                'original CharacterSelect XML objects must exist')
            assert(debug.getinfo(CharacterSelectUtil.FormatCharacterName, 'S').source:find(
                'Mainline/CharacterSelect/CharacterSelectUtil.lua', 1, true),
                'actual selected original formatter must own formatting')
            assert(debug.getinfo(CharacterSelectFrameMixin.OnEvent, 'S').source:find(
                'Mainline/CharacterSelect.lua', 1, true),
                'actual selected original event consumer must exist')
        "#).unwrap();
        env
    })
}

#[test]
fn selected_name_default_roster_original_consumer_preservation() {
    test_timeout! {
        let env = original_selected_name_env(false);
        env.exec(r#"
            assert(GetNumCharacters() > 0, 'existing default host roster expected')
            local guid = GetCharacterGUID(GetCharacterSelection())
            assert(guid, 'existing default selection must resolve')
            local info = GetBasicCharacterInfo(guid)
            assert(info and info.name, 'existing default record must resolve')
            local season = GetCharacterTimerunningSeasonID(guid)
            assert(not season, 'fixture has no timerunning season; do not invent one')
            CharacterSelect_SetSelectedCharacterName(info.name, season)
            assert(CharSelectCharacterName:GetText() == CharacterSelectUtil.FormatCharacterName(info.name, season, nil, 2))
            assert(CharSelectCharacterName:GetScript('OnEnter') == nil)
            assert(CharSelectCharacterName:GetScript('OnLeave') == nil)
            -- The original no-selection branch passes this exact empty sentinel.
            -- Direct consumer invocation tests UI clearing, not native event order.
            CharacterSelect_SetSelectedCharacterName('')
            assert(CharSelectCharacterName:GetText() == '')
            assert(CharSelectCharacterName:GetScript('OnEnter') == nil)
            assert(CharSelectCharacterName:GetScript('OnLeave') == nil)
            CharacterSelect_SetSelectedCharacterName(info.name, season)
            assert(CharSelectCharacterName:GetText() == info.name)
        "#).unwrap();
    }
}

#[test]
fn selected_name_empty_roster_original_consumer_preservation() {
    test_timeout! {
        let env = original_selected_name_env(true);
        env.exec(r#"
            assert(GetNumCharacters() == 0, 'empty host roster must remain empty')
            CharacterSelect_SetSelectedCharacterName('')
            assert(CharSelectCharacterName:GetText() == '')
            assert(CharSelectCharacterName:GetScript('OnEnter') == nil)
            assert(CharSelectCharacterName:GetScript('OnLeave') == nil)
            assert(GetNumCharacters() == 0)
        "#).unwrap();
    }
}

#[test]
fn selected_name_original_ownership_after_postload_retirement_red() {
    test_timeout! {
        let env = original_selected_name_env(false);
        env.exec(r#"
            local source = debug.getinfo(CharacterSelect_SetSelectedCharacterName, 'S').source
            assert(source:find('Mainline/CharacterSelect.lua', 1, true),
                'selected-name original ownership replaced by ' .. source)
            assert(rawget(_G, '__wow_character_select_selected_name_patched') == nil,
                'retired guard marker must not be installed')
        "#).unwrap();
    }
}
