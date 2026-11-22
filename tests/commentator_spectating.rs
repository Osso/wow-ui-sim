#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use crate::common::blizzard_addon_harness::{blizzard_ui_dir, build_blizzard_addon_closure_env};
use wow_ui_sim::lua_api::WowLuaEnv;

fn load_frame_xml_consumer() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let (env, loaded) = build_blizzard_addon_closure_env(&ui, &["Blizzard_FrameXML"], &[]);
    assert!(
        loaded.iter().any(|name| name == "Blizzard_FrameXML"),
        "Blizzard_FrameXML loads"
    );
    env.apply_post_load_workarounds();
    wow_ui_sim::startup::settle_headless_startup(&env);
    env
}

#[test]
fn timer_tracker_parent_tracks_commentator_spectating() {
    let env = load_frame_xml_consumer();
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
