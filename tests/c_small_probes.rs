//! Tests for C_Timer profile compatibility, C_System.GetFrameStack, and
//! C_AddOnProfiler.CheckForPerformanceMessage.

use wow_ui_sim::lua_api::state::AddonRuntimeMetrics;
use wow_ui_sim::lua_api::{AddonInfo, WowLuaEnv};

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

// ── C_Timer.NewTimerID profile compatibility ──────────────────────────────────

#[cfg(feature = "client-retail")]
#[test]
fn retail_does_not_publish_new_timer_id() {
    let env = env();
    let absent: bool = env.eval("return C_Timer.NewTimerID == nil").unwrap();
    assert!(absent, "Retail has no native C_Timer.NewTimerID entry");
}

#[cfg(not(feature = "client-retail"))]
#[test]
fn non_retail_preserves_new_timer_id() {
    let env = env();
    let present: bool = env
        .eval("return type(C_Timer.NewTimerID) == 'function'")
        .unwrap();
    assert!(
        present,
        "non-Retail profiles retain NewTimerID as preservation"
    );
}

// ── C_System.GetFrameStack ────────────────────────────────────────────────────

#[test]
fn get_frame_stack_returns_empty_when_no_hover() {
    let env = env();
    let count: i32 = env.eval("return #C_System.GetFrameStack()").unwrap();
    assert_eq!(count, 0, "no hover → empty stack");
}

#[test]
fn get_frame_stack_returns_table() {
    let env = env();
    let ty: String = env.eval("return type(C_System.GetFrameStack())").unwrap();
    assert_eq!(ty, "table");
}

#[test]
fn get_frame_stack_returns_one_frame_when_hovered() {
    let env = env();
    let _: () = env
        .eval(r#"CreateFrame("Frame", "TestHoverFrame", UIParent)"#)
        .unwrap();
    {
        let state = env.state().borrow();
        let id = state.widgets.get_id_by_name("TestHoverFrame");
        drop(state);
        if let Some(id) = id {
            env.state().borrow_mut().hovered_frame = Some(id);
        }
    }
    let count: i32 = env.eval("return #C_System.GetFrameStack()").unwrap();
    assert_eq!(count, 1, "one hovered frame → stack has one element");
}

// ── C_AddOnProfiler.CheckForPerformanceMessage ───────────────────────────────

#[test]
fn check_for_performance_message_returns_nil_by_default() {
    let env = env();
    // No cvars set → thresholds are disabled → no performance message.
    let is_nil: bool = env
        .eval("return C_AddOnProfiler.CheckForPerformanceMessage() == nil")
        .unwrap();
    assert!(is_nil, "no cvars set → nil (no message)");
}

#[test]
fn check_for_performance_message_returns_table_when_threshold_exceeded() {
    let env = env();
    seed_heavy_addon_performance_message(&env);

    let is_table: bool = env
        .eval(
            r#"
            local msg = C_AddOnProfiler.CheckForPerformanceMessage()
            return type(msg) == "table"
            "#,
        )
        .unwrap();
    assert!(is_table, "threshold exceeded → returns a message table");
}

#[test]
fn add_performance_message_shown_suppresses_matching_message() {
    let env = env();
    seed_heavy_addon_performance_message(&env);

    let result: String = env
        .eval(
            r#"
            local msg = C_AddOnProfiler.CheckForPerformanceMessage()
            if type(msg) ~= "table" then return "missing" end
            C_AddOnProfiler.AddPerformanceMessageShown(msg)
            return C_AddOnProfiler.CheckForPerformanceMessage() == nil and "ok" or "repeat"
            "#,
        )
        .unwrap();

    assert_eq!(result, "ok");
}

fn seed_heavy_addon_performance_message(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state.cvars.set("addonPerformanceMsgWarning", "0.01");
    state.app_frame_metrics.session_frame_count = 1;
    state.app_frame_metrics.session_total_ms = 1000.0;
    state.app_frame_metrics.recent_frame_ms.push_back(1000.0);
    state.addons.push(AddonInfo {
        folder_name: "HeavyAddon".into(),
        title: "Heavy".into(),
        enabled: true,
        loaded: true,
        runtime: AddonRuntimeMetrics {
            session_frame_count: 1,
            session_total_ms: 900.0,
            recent_frames: {
                let mut frames = std::collections::VecDeque::new();
                frames.push_back(900.0);
                frames
            },
            ..Default::default()
        },
        ..Default::default()
    });
}
