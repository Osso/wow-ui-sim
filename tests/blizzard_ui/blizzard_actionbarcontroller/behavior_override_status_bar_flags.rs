//! Behavior pin: override bars show health/power bars from the override flags.

use crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape;

const ROOT: &str = "Blizzard_ActionBarController";

#[test]
fn override_bar_shows_status_bars_from_override_flags() {
    test_timeout! {
    for (health, mana) in [(true, false), (false, true)] {
        with_blizzard_addon_startup_shape(&[ROOT], &[], |env, _loaded| {
            {
                let mut state = env.state().borrow_mut();
                state.has_override_action_bar = true;
                state.override_bar_skin = Some(1);
                state.override_bar_shows_health_bar = health;
                state.override_bar_shows_mana_bar = mana;
            }
            env.exec(
                r#"
                ActionBarController:GetScript("OnEvent")(
                    ActionBarController,
                    "UPDATE_OVERRIDE_ACTIONBAR"
                )
                "#,
            )
            .expect("UPDATE_OVERRIDE_ACTIONBAR dispatch must run cleanly");

            let shown: (bool, bool) = env
                .eval(
                    "return OverrideActionBarHealthBar:IsShown(), \
                     OverrideActionBarPowerBar:IsShown()",
                )
                .expect("status bar visibility probe must run cleanly");
            assert_eq!(
                shown,
                (health, mana),
                "OverrideActionBar:Setup must show exactly the flagged status bars"
            );
        });
    }
    }
}
