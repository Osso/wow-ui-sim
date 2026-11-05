//! `ModelScene:SetViewInsets` / `GetViewInsets` round-trip.
//!
//! Stored on `Frame.model_scene_state.view_insets` as `(l, r, t, b)`.
//! 3D rendering is intentionally out of scope (see AGENTS.md
//! "Intentional Gaps"), so the sim never consumes the value — these
//! tests pin the storage-only contract.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("WowLuaEnv init")
}

#[test]
fn default_view_insets_are_zero() {
    let env = env();
    let (l, r, t, b): (f64, f64, f64, f64) = env
        .eval(
            r#"
            local ms = CreateFrame("ModelScene", nil, UIParent)
            return ms:GetViewInsets()
            "#,
        )
        .unwrap();
    assert_eq!((l, r, t, b), (0.0, 0.0, 0.0, 0.0));
}

#[test]
fn set_view_insets_round_trips_four_values() {
    let env = env();
    let (l, r, t, b): (f64, f64, f64, f64) = env
        .eval(
            r#"
            local ms = CreateFrame("ModelScene", nil, UIParent)
            ms:SetViewInsets(1.5, 2.25, 3, 4.75)
            return ms:GetViewInsets()
            "#,
        )
        .unwrap();
    assert!((l - 1.5).abs() < 1e-4);
    assert!((r - 2.25).abs() < 1e-4);
    assert!((t - 3.0).abs() < 1e-4);
    assert!((b - 4.75).abs() < 1e-4);
}

#[test]
fn set_view_insets_overwrites_prior_values() {
    let env = env();
    let (l, r, t, b): (f64, f64, f64, f64) = env
        .eval(
            r#"
            local ms = CreateFrame("ModelScene", nil, UIParent)
            ms:SetViewInsets(10, 20, 30, 40)
            ms:SetViewInsets(0, 0, 0, 0)
            return ms:GetViewInsets()
            "#,
        )
        .unwrap();
    assert_eq!((l, r, t, b), (0.0, 0.0, 0.0, 0.0));
}

#[test]
fn negative_view_insets_accepted() {
    let env = env();
    let (l, r, t, b): (f64, f64, f64, f64) = env
        .eval(
            r#"
            local ms = CreateFrame("ModelScene", nil, UIParent)
            ms:SetViewInsets(-1, -2, -3, -4)
            return ms:GetViewInsets()
            "#,
        )
        .unwrap();
    assert_eq!((l, r, t, b), (-1.0, -2.0, -3.0, -4.0));
}

#[test]
fn independent_model_scenes_do_not_share_insets() {
    let env = env();
    let (al, ar, at, ab, bl, br, bt, bb): (f64, f64, f64, f64, f64, f64, f64, f64) = env
        .eval(
            r#"
            local a = CreateFrame("ModelScene", nil, UIParent)
            local b = CreateFrame("ModelScene", nil, UIParent)
            a:SetViewInsets(1, 2, 3, 4)
            b:SetViewInsets(5, 6, 7, 8)
            local al, ar, at, ab = a:GetViewInsets()
            local bl, br, bt, bb = b:GetViewInsets()
            return al, ar, at, ab, bl, br, bt, bb
            "#,
        )
        .unwrap();
    assert_eq!((al, ar, at, ab), (1.0, 2.0, 3.0, 4.0));
    assert_eq!((bl, br, bt, bb), (5.0, 6.0, 7.0, 8.0));
}
