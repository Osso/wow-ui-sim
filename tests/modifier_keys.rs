//! Modifier-key probes — SimState-backed round-trip.

use std::path::PathBuf;

use wow_ui_sim::lua_api::WowLuaEnv;

fn source_path(relative_path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative_path)
}

fn all(env: &WowLuaEnv) -> (bool, bool, bool, bool, bool) {
    env.eval(
        r#"
        return IsShiftKeyDown(), IsControlKeyDown(), IsAltKeyDown(),
               IsMetaKeyDown(), IsModifierKeyDown()
        "#,
    )
    .unwrap()
}

#[test]
fn modifier_keys_live_under_real_globals_boundary() {
    assert!(
        !source_path("src/lua_api/globals/modifier_keys.rs").exists(),
        "modifier-key globals are modeled through SimState and belong under globals::real",
    );
    assert!(
        source_path("src/lua_api/globals/real/modifier_keys.rs").exists(),
        "modifier-key globals should stay classified as real modeled Lua globals",
    );
}

#[test]
fn defaults_all_false() {
    let env = WowLuaEnv::new().unwrap();
    let (shift, ctrl, alt, meta, any) = all(&env);
    assert!(!shift);
    assert!(!ctrl);
    assert!(!alt);
    assert!(!meta);
    assert!(!any);
}

#[test]
fn set_shift_flips_shift_and_any_modifier() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("A_Admin.SetShiftKeyDown(true)").unwrap();
    let (shift, _, _, _, any) = all(&env);
    assert!(shift);
    assert!(
        any,
        "IsModifierKeyDown should report true when shift is held"
    );
}

#[test]
fn set_control_contributes_to_any_modifier() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("A_Admin.SetControlKeyDown(true)").unwrap();
    let (_, ctrl, _, _, any) = all(&env);
    assert!(ctrl);
    assert!(any);
}

#[test]
fn set_alt_contributes_to_any_modifier() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("A_Admin.SetAltKeyDown(true)").unwrap();
    let (_, _, alt, _, any) = all(&env);
    assert!(alt);
    assert!(any);
}

#[test]
fn meta_does_not_contribute_to_any_modifier() {
    // Real WoW: IsModifierKeyDown is shift || control || alt, NOT meta.
    let env = WowLuaEnv::new().unwrap();
    env.exec("A_Admin.SetMetaKeyDown(true)").unwrap();
    let (_, _, _, meta, any) = all(&env);
    assert!(meta);
    assert!(
        !any,
        "IsModifierKeyDown should NOT include meta — matches WoW semantics",
    );
}

#[test]
fn no_arg_defaults_to_true_for_all_setters() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("A_Admin.SetShiftKeyDown()").unwrap();
    env.exec("A_Admin.SetControlKeyDown()").unwrap();
    env.exec("A_Admin.SetAltKeyDown()").unwrap();
    env.exec("A_Admin.SetMetaKeyDown()").unwrap();
    let (shift, ctrl, alt, meta, any) = all(&env);
    assert!(shift);
    assert!(ctrl);
    assert!(alt);
    assert!(meta);
    assert!(any);
}

#[test]
fn setters_can_release_keys() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("A_Admin.SetShiftKeyDown(true)").unwrap();
    env.exec("A_Admin.SetShiftKeyDown(false)").unwrap();
    let (shift, _, _, _, any) = all(&env);
    assert!(!shift);
    assert!(!any);
}

#[test]
fn multiple_modifiers_set_simultaneously() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("A_Admin.SetShiftKeyDown(true)").unwrap();
    env.exec("A_Admin.SetAltKeyDown(true)").unwrap();
    let (shift, ctrl, alt, _, any) = all(&env);
    assert!(shift);
    assert!(!ctrl);
    assert!(alt);
    assert!(any);
}

#[test]
fn is_modified_click_follows_any_non_meta_modifier() {
    let env = WowLuaEnv::new().unwrap();

    let before: bool = env.eval("return IsModifiedClick('CHATLINK')").unwrap();
    assert!(
        !before,
        "IsModifiedClick should start false when no modifiers are held"
    );

    env.exec("A_Admin.SetControlKeyDown(true)").unwrap();
    let during: bool = env.eval("return IsModifiedClick('CHATLINK')").unwrap();
    assert!(
        during,
        "IsModifiedClick should report true when a non-meta modifier is held"
    );

    env.exec("A_Admin.SetControlKeyDown(false)").unwrap();
    env.exec("A_Admin.SetMetaKeyDown(true)").unwrap();
    let meta_only: bool = env.eval("return IsModifiedClick('CHATLINK')").unwrap();
    assert!(
        !meta_only,
        "IsModifiedClick should mirror IsModifierKeyDown semantics and ignore meta-only input"
    );
}
