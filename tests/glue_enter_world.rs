//! Real CharacterSelect controls must hand the selected character to Game.

#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use crate::common;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::{run_extra_update_ticks, settle_headless_startup};

const SELECTED_CHARACTER_NAME: &str = "Entrant";

fn load_character_select() -> WowLuaEnv {
    let mut env = WowLuaEnv::new().expect("create Glue environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::CharacterSelect);
    env.state().borrow_mut().bnet_friends[0].game_accounts[0].character_name =
        SELECTED_CHARACTER_NAME.into();
    let ui =
        wow_ui_sim::paths::default_blizzard_ui_addons_path().expect("verified profile UI cache");
    for (name, toc) in discover_blizzard_addons_for_screen(&ui, ScreenKind::CharacterSelect) {
        load_addon(&env.loader_env(), &toc).unwrap_or_else(|error| panic!("load {name}: {error}"));
    }
    env.apply_post_load_workarounds();
    settle_headless_startup(&env);
    env.set_reload_loader(move |env, _| {
        // Host transition; native login sequence UNKNOWN (R74).
        let screen = env.state().borrow().screen_kind;
        for (name, toc) in discover_blizzard_addons_for_screen(&ui, screen) {
            load_addon(&env.loader_env(), &toc)
                .map_err(|error| wow_ui_sim::Error::Other(format!("load {name}: {error}")))?;
        }
        env.apply_post_load_workarounds();
        Ok(())
    });
    env
}

#[test]
fn character_select_enter_world_button_loads_game_for_selected_character() {
    test_timeout! {
      common::with_shared_workload(|| {
        let mut env = load_character_select();
        env.exec(r#"
            assert(CharacterSelect:IsShown())
            assert(GetNumCharacters() > 0)
            CharacterSelect_SelectCharacter(1, true)
            assert(GetCharacterSelection() == 1)
            assert(CharacterSelect_AllowedToEnterWorld())
            assert(CharSelectEnterWorldButton:IsShown())
            assert(CharSelectEnterWorldButton:IsEnabled())
            assert(CharSelectEnterWorldButton:GetScript('OnClick') ~= nil)
        "#).expect("the original Enter World control is available for the selected character");
        #[cfg(feature = "client-retail")]
        let selected_name: String = env.eval(
            "return CharacterSelectUtil.GetCharacterInfoTable(GetCharacterSelection()).name"
        ).expect("selected Retail roster record supplies the name");
        #[cfg(feature = "client-wowforever")]
        let selected_name: String = env.eval(
            "return CharacterSelectUtil.GetCharacterInfoTable(GetCharacterSelection()).characterName"
        ).expect("selected Forever roster record supplies the name");
        assert_eq!(selected_name, SELECTED_CHARACTER_NAME);
        assert_ne!(env.state().borrow().player.name, SELECTED_CHARACTER_NAME,
            "the selected roster identity must differ from the previous player identity");
        let button = env.state().borrow().widgets
            .get_id_by_name("CharSelectEnterWorldButton")
            .expect("original XML button exists");
        let errors_before = env.state().borrow().lua_errors.len();
        env.send_click(button).expect("dispatch original Enter World OnClick");
        assert!(env.service_headless_ui_transition(&mut None)
            .expect("service host entry through the owner"),
            "original button must queue host entry; new errors: {:#?}",
            &env.state().borrow().lua_errors[errors_before..]);
        run_extra_update_ticks(&env, 3);
        assert_eq!(env.state().borrow().screen_kind, ScreenKind::Game,
            "Enter World must leave Glue; new errors: {:#?}",
            &env.state().borrow().lua_errors[errors_before..]);
        assert!(env.eval::<bool>(r#"
            return UIParent:IsShown() and IsLoggedIn()
                and (GlueParent == nil or not GlueParent:IsShown())
                and (CharacterSelect == nil or not CharacterSelect:IsShown())
        "#).unwrap(), "Game is visible and Glue no longer is");
        assert_eq!(env.eval::<String>("return UnitName('player')").unwrap(), SELECTED_CHARACTER_NAME);
        assert_eq!(env.state().borrow().lua_errors.len(), errors_before,
            "host entry must add no Lua errors: {:#?}",
            &env.state().borrow().lua_errors[errors_before..]);
      });
    }
}
