//! Integration tests for `src/lua_api/globals/real/mouse_probes.rs`.

use std::path::PathBuf;

use wow_ui_sim::lua_api::WowLuaEnv;

fn source_path(relative_path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative_path)
}

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("WowLuaEnv init")
}

// ── GetCursorPosition ─────────────────────────────────────────────────────────

#[test]
fn get_cursor_position_defaults_to_zero() {
    let env = env();
    let (x, y): (f64, f64) = env.eval("return GetCursorPosition()").unwrap();
    assert_eq!(x, 0.0);
    assert_eq!(y, 0.0);
}

#[test]
fn get_cursor_position_reads_mouse_position() {
    let env = env();
    env.set_screen_size(1600.0, 1200.0);
    env.state()
        .borrow_mut()
        .set_mouse_position(Some((320.5, 240.25)));
    let (x, y): (f64, f64) = env.eval("return GetCursorPosition()").unwrap();
    assert!((x - 320.5).abs() < 1e-3);
    assert!((y - 959.75).abs() < 1e-3);
}

#[test]
fn get_cursor_position_flips_renderer_y_to_lua_screen_y() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        state.screen_height = 768.0;
        state.set_mouse_position(Some((12.0, 24.0)));
    }

    let (x, y): (f64, f64) = env.eval("return GetCursorPosition()").unwrap();
    assert_eq!(x, 12.0);
    assert_eq!(y, 744.0);
}

// ── GetMouseFocus ─────────────────────────────────────────────────────────────

#[test]
fn get_mouse_focus_nil_when_no_hovered_frame() {
    let env = env();
    let v: Option<String> = env.eval("return GetMouseFocus()").unwrap();
    assert_eq!(v, None);
}

#[test]
fn get_mouse_focus_returns_hovered_frame_userdata() {
    let env = env();
    let frame_id: i64 = env
        .eval(r#"local f = CreateFrame("Frame", "MouseFocusTestFrame"); return f:GetID()"#)
        .unwrap();

    {
        let mut st = env.state().borrow_mut();
        let id = st
            .widgets
            .get_id_by_name("MouseFocusTestFrame")
            .expect("test frame should exist");
        st.hovered_frame = Some(id);
        let _ = frame_id; // silence unused when GetID returned 0
    }

    let name: String = env.eval("return GetMouseFocus():GetName()").unwrap();
    assert_eq!(name, "MouseFocusTestFrame");
}

// ── IsMouseButtonDown ────────────────────────────────────────────────────────

#[test]
fn is_mouse_button_down_defaults_false() {
    let env = env();
    let (any, left, right, middle, button4): (bool, bool, bool, bool, bool) = env
        .eval(
            r#"
            return IsMouseButtonDown(),
                   IsMouseButtonDown("LeftButton"),
                   IsMouseButtonDown("RightButton"),
                   IsMouseButtonDown("MiddleButton"),
                   IsMouseButtonDown("Button4")
            "#,
        )
        .unwrap();
    assert!(!any);
    assert!(!left);
    assert!(!right);
    assert!(!middle);
    assert!(!button4);
}

#[test]
fn is_mouse_button_down_reads_named_button_state() {
    let env = env();
    {
        let mut state = env.state().borrow_mut();
        let _ = state.set_mouse_button_down("LeftButton", true);
    }

    let (any, left, right, unknown): (bool, bool, bool, bool) = env
        .eval(
            r#"
            return IsMouseButtonDown(),
                   IsMouseButtonDown("LeftButton"),
                   IsMouseButtonDown("RightButton"),
                   IsMouseButtonDown("NoSuchButton")
            "#,
        )
        .unwrap();
    assert!(any);
    assert!(left);
    assert!(!right);
    assert!(!unknown);
}

#[test]
fn mouse_probe_globals_live_under_real_globals_boundary() {
    assert!(
        !source_path("src/lua_api/globals/mouse_probes.rs").exists(),
        "mouse probe globals are modeled through simulator input state and belong under globals::real",
    );
    assert!(
        source_path("src/lua_api/globals/real/mouse_probes.rs").exists(),
        "mouse probe globals should stay classified as real modeled Lua globals",
    );
}
