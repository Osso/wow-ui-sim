#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use wow_ui_sim::lua_api::WowLuaEnv;

prefork_full_ui_case! {
fn timer_tracker_parent_tracks_commentator_spectating(env: &WowLuaEnv) {
    env.exec(
        r#"
        assert(TimerTracker and TimerTracker:GetScript("OnEvent"), "real TimerTracker frame loaded")
        local onEvent = TimerTracker:GetScript("OnEvent")

        A_Admin.SetSpectating(false)
        onEvent(TimerTracker, "PLAYER_ENTERING_WORLD")
        assert(TimerTracker:GetParent() == UIParent, "non-spectating timer stays under UIParent")

        A_Admin.SetSpectating(true)
        onEvent(TimerTracker, "PLAYER_ENTERING_WORLD")
        assert(TimerTracker:GetParent() == WorldFrame, "spectating timer moves under WorldFrame")
        "#,
    )
    .expect("original TimerTracker event handler follows supplied commentator state");
}
}
