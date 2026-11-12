use crate::common;

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::startup::fire_startup_events;

fn build_action_bar_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    common::panel_fixtures::seed_addon_search_paths(&env);
    common::panel_fixtures::load_panel_addons(&env);
    env.apply_post_load_workarounds();
    fire_startup_events(&env);
    env
}

fn seed_action_slot(env: &WowLuaEnv, slot: u32, spell_id: u32) {
    env.state().borrow_mut().action_bars.insert(slot, spell_id);
}

#[test]
fn action_button_down_and_up_toggle_state_and_fire_action() {
    wrath_action_case!(key_release_cast);

    let env = build_action_bar_env();
    seed_action_slot(&env, 1, 19750);
    env.fire_event("ACTIONBAR_SLOT_CHANGED").unwrap();

    let before: (String, bool) = env
        .eval(
            r#"
            return ActionButton1:GetButtonState(), HasAction(1)
        "#,
        )
        .unwrap();
    assert_eq!(before.0, "NORMAL");
    assert!(before.1);

    env.exec("ActionButtonDown(1)").unwrap();

    let pressed: (String, bool) = env
        .eval(
            r#"
            return ActionButton1:GetButtonState(), IsCurrentAction(1)
        "#,
        )
        .unwrap();
    assert_eq!(pressed.0, "PUSHED");
    assert!(pressed.1);

    env.exec("ActionButtonUp(1)").unwrap();
    let released: String = env.eval("return ActionButton1:GetButtonState()").unwrap();
    assert_eq!(released, "NORMAL");
}

// TryUseActionButton is modern-only; native key-release casting stays covered above.
post_wrath_ui_fixture! {
#[test]
fn try_use_action_button_only_fires_from_key_down() {
    let env = build_action_bar_env();
    seed_action_slot(&env, 1, 19750);
    env.fire_event("ACTIONBAR_SLOT_CHANGED").unwrap();

    let only_down_casts: bool = env
        .eval(
            r#"
            local before = select(9, UnitCastingInfo("player"))
            TryUseActionButton(ActionButton1, false)
            local afterUp = select(9, UnitCastingInfo("player"))
            TryUseActionButton(ActionButton1, true)
            local afterDown = select(9, UnitCastingInfo("player"))
            return before == nil and afterUp == nil and afterDown == 19750
            "#,
        )
        .unwrap();
    assert!(only_down_casts);
}

}

#[test]
fn multi_and_extra_action_dispatch_use_the_same_button_fallthrough() {
    wrath_action_case!(multibar_release_cast);

    let env = build_action_bar_env();
    seed_action_slot(&env, 61, 19750);
    env.fire_event("ACTIONBAR_SLOT_CHANGED").unwrap();

    let multi_cast_spell_id: i32 = env
        .eval(
            r#"
            MultiActionButtonDown("MultiBarBottomLeft", 1)
            return select(9, UnitCastingInfo("player"))
        "#,
        )
        .unwrap();
    assert_eq!(multi_cast_spell_id, 19750);

    env.exec("SpellStopCasting()").unwrap();
    let extra_did_not_error: bool = env
        .eval(
            r#"
            local ok = pcall(ExtraActionButtonKey, 1, true)
            return ok
        "#,
        )
        .unwrap();
    assert!(extra_did_not_error);
}
