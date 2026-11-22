use crate::common;
use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;
use wow_ui_sim::lua_api::WowLuaEnv;

const ROOT: &str = "Blizzard_ActionBarController";

#[test]
fn possess_buttons_use_supplied_slot_info() {
    test_timeout! {
        with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
            supply_possess_slots_and_show_bar(env);
            assert_possess_buttons_show_supplied_slots(env);
        });
    }
}

fn supply_possess_slots_and_show_bar(env: &WowLuaEnv) {
    env.exec(
        r#"
        A_Admin.SetPossessActionSlot(1, "Interface\\Icons\\Ability_Hunter_BeastCall", 883, true)
        A_Admin.SetPossessActionSlot(2, "Interface\\Icons\\Spell_Shadow_MindControl", 605, false)
        "#,
    )
    .expect("possess slots should be supplied through the admin model");
    env.state()
        .borrow_mut()
        .action_bar_state
        .possess_bar_visible = true;
    env.fire_event("UPDATE_POSSESS_BAR")
        .expect("possess-bar update event should dispatch");
}

fn assert_possess_buttons_show_supplied_slots(env: &WowLuaEnv) {
    let supplied_slots_reached_original_consumer: bool = env
        .eval(
            r#"
            local first = PossessButton1:GetActionButtonInfo()
            local second = PossessButton2:GetActionButtonInfo()
            return PossessButton1.icon:GetTexture() == "Interface\\Icons\\Ability_Hunter_BeastCall"
                and PossessButton2.icon:GetTexture() == "Interface\\Icons\\Spell_Shadow_MindControl"
                and PossessButton1:HasAction()
                and not PossessButton2:HasAction()
                and first.id == 883 and first.enabled == true
                and second.id == 605 and second.enabled == false
                and PossessButton1:IsEnabled() and PossessButton2:IsEnabled()
                and not PossessButton1.icon:IsDesaturated()
                and not PossessButton2.icon:IsDesaturated()
            "#,
        )
        .expect("original possess-button consumer should expose supplied state");
    assert!(
        supplied_slots_reached_original_consumer,
        "each Blizzard possess button must use its own supplied texture, spell ID, and enabled value while retaining UpdateState's enabled, non-desaturated presentation"
    );
}
