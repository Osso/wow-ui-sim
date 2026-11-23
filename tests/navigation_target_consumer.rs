#![cfg(all(feature = "gui", feature = "client-retail"))]

use crate::common;

fn update_navigation_target(env: &WowLuaEnv, target: &str) -> (String, i32, bool) {
    env.exec(target).expect("supply selected navigation target");
    env.fire_on_update(0.016).expect("tick navigation consumer");
    env.eval(
        "return SuperTrackedFrame.DistanceText:GetText(), C_Navigation.GetTargetState(), C_Navigation.GetFrame() ~= nil",
    )
    .expect("read navigation consumer")
}

prefork_full_ui_case! {
fn navigation_target_drives_super_tracked_frame(env: &WowLuaEnv) {
    let first = update_navigation_target(
        env,
        r#"
        C_SuperTrack.SetSuperTrackedQuestID(987654)
        A_Admin.SetNavigationTarget({
            state = Enum.NavigationState.InRange,
            distance = 125,
            validScreenPosition = true,
            clamped = false,
            screenX = 320,
            screenY = 240,
        })
        "#,
    );
    let second = update_navigation_target(
        env,
        r#"
        A_Admin.SetNavigationTarget({
            state = Enum.NavigationState.Occluded,
            distance = 40,
            validScreenPosition = true,
            clamped = false,
            screenX = 180,
            screenY = 100,
        })
        "#,
    );
    assert!(first.0.contains("125"), "first distance text: {:?}", first.0);
    assert!(second.0.contains("40"), "second distance text: {:?}", second.0);
    assert_ne!(first.0, second.0, "target distances must change consumer text");
    assert_eq!(first.1, 2, "first target is InRange");
    assert_eq!(second.1, 1, "second target is Occluded");
    assert!(first.2 && second.2, "selected targets have a navigation frame");

    env.exec("A_Admin.SetNavigationTarget(nil)")
        .expect("clear supplied navigation target");
    env.fire_on_update(0.016).expect("tick destroyed target consumer");
    let destroyed: (bool, i32) = env
        .eval("return C_Navigation.GetFrame() == nil, C_Navigation.GetTargetState()")
        .expect("read destroyed navigation state");
    assert_eq!(destroyed, (true, 0), "cleared target is no longer navigable");
}
}
