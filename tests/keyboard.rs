//! Tests for keyboard input handling.
//!
//! Tests for key press simulation via `WowLuaEnv::send_key_press`.

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn test_escape_shows_game_menu() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        GameMenuFrame = CreateFrame("Frame", "GameMenuFrame", UIParent)
        GameMenuFrame:Hide()
    "#,
    )
    .unwrap();

    let shown: bool = env.eval("return GameMenuFrame:IsShown()").unwrap();
    assert!(!shown, "GameMenuFrame should start hidden");

    env.send_key_press("ESCAPE").unwrap();

    let shown: bool = env.eval("return GameMenuFrame:IsShown()").unwrap();
    assert!(shown, "Escape should show GameMenuFrame");
}

#[test]
fn test_escape_hides_game_menu_when_shown() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        GameMenuFrame = CreateFrame("Frame", "GameMenuFrame", UIParent)
        GameMenuFrame:Show()
    "#,
    )
    .unwrap();

    let shown: bool = env.eval("return GameMenuFrame:IsShown()").unwrap();
    assert!(shown, "GameMenuFrame should start shown");

    env.send_key_press("ESCAPE").unwrap();

    let shown: bool = env.eval("return GameMenuFrame:IsShown()").unwrap();
    assert!(!shown, "Escape should hide GameMenuFrame");
}

#[test]
fn test_escape_toggles_game_menu_twice() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        GameMenuFrame = CreateFrame("Frame", "GameMenuFrame", UIParent)
        GameMenuFrame:Hide()
    "#,
    )
    .unwrap();

    env.send_key_press("ESCAPE").unwrap();
    let shown: bool = env.eval("return GameMenuFrame:IsShown()").unwrap();
    assert!(shown, "First Escape should show menu");

    env.send_key_press("ESCAPE").unwrap();
    let shown: bool = env.eval("return GameMenuFrame:IsShown()").unwrap();
    assert!(!shown, "Second Escape should hide menu");
}

#[test]
fn test_escape_without_game_menu_frame() {
    let env = WowLuaEnv::new().unwrap();

    // Should not error when GameMenuFrame doesn't exist
    env.send_key_press("ESCAPE").unwrap();
}
