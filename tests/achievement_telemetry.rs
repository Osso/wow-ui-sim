#![cfg(not(feature = "client-wrath"))]
use wow_ui_sim::c_api::c_achievement_telemetry::AchievementTelemetryEvent;
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn achievement_telemetry_records_reports_in_call_order() {
    let env = WowLuaEnv::new().expect("env");
    env.exec(
        "C_AchievementTelemetry.ShowAchievements(); \
         C_AchievementTelemetry.LinkAchievementInWhisper(6); \
         C_AchievementTelemetry.LinkAchievementInClub(40000)",
    )
    .expect("telemetry calls should succeed");

    assert_eq!(
        env.state().borrow().achievement_telemetry,
        vec![
            AchievementTelemetryEvent::ShowAchievements,
            AchievementTelemetryEvent::LinkInWhisper(6),
            AchievementTelemetryEvent::LinkInClub(40000),
        ]
    );
}

#[test]
fn achievement_telemetry_links_require_an_achievement_id() {
    let env = WowLuaEnv::new().expect("env");
    let ok: bool = env
        .eval("return (pcall(C_AchievementTelemetry.LinkAchievementInClub))")
        .expect("pcall");
    assert!(!ok, "LinkAchievementInClub without an achievementID must error");
    assert!(env.state().borrow().achievement_telemetry.is_empty());
}
