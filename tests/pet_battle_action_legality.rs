#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;

fn load_game_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    let ui =
        wow_ui_sim::paths::default_blizzard_ui_addons_path().expect("Blizzard UI cache synced");
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    wow_ui_sim::xml::register_intrinsic_templates();
    let addons = wow_ui_sim::loader::discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    for (name, path) in addons {
        wow_ui_sim::loader::load_addon(&env.loader_env(), &path)
            .unwrap_or_else(|error| panic!("[load {name}] FAILED: {error}"));
    }
    env.apply_post_load_workarounds();
    wow_ui_sim::startup::fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

const ACTION_BUTTON_STATE: &str = r#"
    local switch = PetBattleFrame.BottomFrame.SwitchPetButton
    local skip = PetBattleFrame.BottomFrame.TurnTimer.SkipButton
    PetBattleActionButton_UpdateState(switch)
    PetBattleActionButton_UpdateState(skip)
    return tostring(switch:IsEnabled()) .. "," .. tostring(skip:IsEnabled())
"#;

fn action_buttons_after(env: &WowLuaEnv, legality: &str) -> String {
    env.exec(&format!("A_Admin.SetPetBattleActionLegality({legality})"))
        .unwrap();
    env.eval(ACTION_BUTTON_STATE).unwrap()
}

#[test]
fn blizzard_pet_battle_ui_action_buttons_follow_supplied_turn_legality() {
    let env = load_game_ui();
    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_PetBattleUI')")
        .expect("consumer load probe succeeds");
    assert!(
        loaded,
        "Blizzard_PetBattleUI must be selected for this profile"
    );
    env.exec("A_Admin.SetPetBattleCounts(3, 2); A_Admin.SetPetBattleState(1)")
        .unwrap();
    env.exec("assert(Enum.BattlePetOwner.Ally == 1 and Enum.BattlePetOwner.Enemy == 2)")
        .unwrap();
    let first = action_buttons_after(&env, "false, false, false, false, false");
    assert_eq!(first, "false,false");
    let second = action_buttons_after(&env, "true, false, false, true, true");
    assert_eq!(second, "true,true");
}
