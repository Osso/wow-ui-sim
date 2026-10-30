//! Original Blizzard TargetFrame receives admin target and health changes.
#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

#[test]
fn target_frame_tracks_admin_target_and_health_events() {
    crate::common::with_timeout(90, || {
        crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape(
            &["Blizzard_UnitFrame"],
            &[],
            |env, _| {
                let errors_before = env.state().borrow().lua_errors.len();
                env.exec(r#"
                    A_Admin.ClearTarget()
                    assert(TargetFrame and not TargetFrame:IsShown(), 'target frame starts hidden')
                    A_Admin.SetTarget('Admin target', 80, 1, true)
                    assert(TargetFrame:IsShown(), 'target event must show Blizzard frame')
                    local health = TargetFrame.TargetFrameContent.TargetFrameContentMain.HealthBarsContainer.HealthBar
                    A_Admin.SetTargetHealth(123, 456)
                    assert(health:GetValue() == 123, 'health event must update Blizzard bar')
                    local _, maximum = health:GetMinMaxValues()
                    assert(maximum == 456, 'max-health event must update Blizzard bar')
                    A_Admin.ClearTarget()
                    assert(not TargetFrame:IsShown(), 'target event must hide Blizzard frame')
                "#).expect("original TargetFrame consumes admin events");
                let state = env.state();
                let state = state.borrow();
                assert_eq!(
                    state.lua_errors.len(),
                    errors_before,
                    "TargetFrame events logged Lua errors: {:?}",
                    &state.lua_errors[errors_before..]
                );
            },
        );
    });
}
