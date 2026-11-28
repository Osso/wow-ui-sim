#![cfg(all(feature = "gui", feature = "client-retail"))]

use crate::common;
use wow_ui_sim::lua_api::WowLuaEnv;

fn update_navigation_target(env: &WowLuaEnv, target: &str) -> (String, i32, bool, bool, bool) {
    env.exec(target).expect("supply selected navigation target");
    env.fire_on_update(0.016).expect("tick navigation consumer");
    env.eval(
        "return SuperTrackedFrame.DistanceText:GetText(), C_Navigation.GetTargetState(), C_Navigation.GetFrame() ~= nil, C_Navigation.HasValidScreenPosition(), C_Navigation.WasClampedToScreen()",
    )
    .expect("read navigation consumer")
}

fn assert_navigation_observation(
    observation: &(String, i32, bool, bool, bool),
    distance: &str,
    state: i32,
    has_valid_screen_position: bool,
) {
    assert!(observation.0.contains(distance), "distance text: {:?}", observation.0);
    assert_eq!(observation.1, state, "navigation target state");
    assert!(observation.2, "selected target has a navigation frame");
    assert_eq!(observation.3, has_valid_screen_position, "screen-position validity");
    assert!(!observation.4, "targets are not clamped");
}

fn clear_navigation_target(env: &WowLuaEnv) -> (bool, i32, i32) {
    env.exec("A_Admin.SetNavigationTarget(nil)")
        .expect("clear supplied navigation target");
    env.fire_on_update(0.016).expect("tick destroyed target consumer");
    env.eval("return C_Navigation.GetFrame() == nil, C_Navigation.GetTargetState(), SuperTrackedFrame:GetNumPoints()")
        .expect("read destroyed navigation state")
}

#[cfg(feature = "client-retail")]
prefork_full_ui_case! {
fn navigation_target_drives_super_tracked_frame(env: &WowLuaEnv) {
    let first = update_navigation_target(
        env,
        r#"A_Admin.OpenMultiQuestNpc()
        SelectAvailableQuest(1)
        ConfirmAcceptQuest()
        assert(A_Admin.SetQuestObjectives(80002, {{text='Navigate',type='progress',finished=false,numFulfilled=0,numRequired=1}}, 0))
        C_SuperTrack.SetSuperTrackedQuestID(80002)
        A_Admin.SetNavigationTarget({state=Enum.NavigationState.InRange,distance=125,clamped=false,screenX=0.0000002384185791015625,screenY=0})"#,
    );
    let second = update_navigation_target(
        env,
        r#"A_Admin.SetNavigationTarget({state=Enum.NavigationState.Occluded,distance=40,clamped=false,screenX=0,screenY=0})"#,
    );
    assert_navigation_observation(&first, "125", 2, true);
    assert_navigation_observation(&second, "40", 1, false);
    assert_ne!(first.0, second.0, "target distances must change consumer text");

    let destroyed = clear_navigation_target(env);
    assert_eq!(destroyed, (true, 0, 0), "destroyed target clears its consumer anchors");
}
}

