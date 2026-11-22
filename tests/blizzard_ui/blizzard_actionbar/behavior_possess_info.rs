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
    let observed: String = env
        .eval(
            r#"
            local first = PossessButton1:GetActionButtonInfo()
            local second = PossessButton2:GetActionButtonInfo()
            return table.concat({
                tostring(PossessButton1.icon:GetTexture()),
                tostring(PossessButton2.icon:GetTexture()),
                tostring(PossessButton1:HasAction()),
                tostring(PossessButton2:HasAction()),
                tostring(first.id), tostring(first.enabled),
                tostring(second.id), tostring(second.enabled),
                tostring(PossessButton1:IsEnabled()),
                tostring(PossessButton2:IsEnabled()),
                tostring(PossessButton1.icon:IsDesaturated()),
                tostring(PossessButton2.icon:IsDesaturated())
            }, "|")
            "#,
        )
        .expect("original possess-button consumer should expose supplied state");
    assert_eq!(
        observed,
        "132161|Interface\\Icons\\Spell_Shadow_MindControl|true|false|883|true|605|false|true|true|false|false"
    );
}
