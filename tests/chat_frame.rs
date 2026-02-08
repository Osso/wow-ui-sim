//! Integration test for the Blizzard chat frame.
//!
//! Loads the Blizzard UI, clicks on ChatFrame1EditBox, types a message,
//! presses Enter, and verifies the message was submitted via
//! C_ChatInfo.SendChatMessage.

use std::path::PathBuf;
use wow_ui_sim::loader::{discover_blizzard_addons, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;

fn blizzard_ui_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Interface/BlizzardUI")
}

/// Create a fully loaded environment with all Blizzard addons and startup events.
fn setup_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);

    let ui = blizzard_ui_dir();
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![ui.clone()];
    }

    let addons = discover_blizzard_addons(&ui);
    for (name, toc_path) in &addons {
        if let Err(e) = load_addon(&env.loader_env(), toc_path) {
            eprintln!("[load {name}] FAILED: {e}");
        }
    }

    env.apply_post_load_workarounds();
    fire_startup_events(&env);
    env
}

/// Fire startup events (same sequence as main.rs).
fn fire_startup_events(env: &WowLuaEnv) {
    let lua = env.lua();
    let _ = env.fire_event_with_args(
        "ADDON_LOADED",
        &[mlua::Value::String(lua.create_string("WoWUISim").unwrap())],
    );
    for event in ["VARIABLES_LOADED", "PLAYER_LOGIN"] {
        let _ = env.fire_event(event);
    }
    if let Ok(f) = lua.globals().get::<mlua::Function>("RequestTimePlayed") {
        let _ = f.call::<()>(());
    }
    let _ = env.fire_event_with_args(
        "PLAYER_ENTERING_WORLD",
        &[mlua::Value::Boolean(true), mlua::Value::Boolean(false)],
    );
    for event in [
        "UPDATE_BINDINGS",
        "DISPLAY_SIZE_CHANGED",
        "UI_SCALE_CHANGED",
    ] {
        let _ = env.fire_event(event);
    }
}

/// Hook C_ChatInfo.SendChatMessage to capture submitted messages.
fn hook_send_chat_message(env: &WowLuaEnv) {
    env.exec(
        r#"
        _G.__test_sent_messages = {}
        local orig = C_ChatInfo.SendChatMessage
        C_ChatInfo.SendChatMessage = function(msg, chatType, language, target)
            table.insert(_G.__test_sent_messages, {
                message = msg,
                chatType = chatType,
                language = language,
                target = target,
            })
            if orig then orig(msg, chatType, language, target) end
        end
    "#,
    )
    .expect("Failed to hook SendChatMessage");
}

/// Type a string into the focused EditBox character by character.
fn type_text(env: &WowLuaEnv, text: &str) {
    for ch in text.chars() {
        let s = ch.to_string();
        let key = if ch == ' ' {
            "SPACE".to_string()
        } else {
            s.to_uppercase()
        };
        env.send_key_press(&key, Some(&s)).unwrap();
    }
}

/// Click on ChatFrame1EditBox and verify it gains focus.
fn click_chat_editbox(env: &WowLuaEnv) {
    let frame_id = env
        .state()
        .borrow()
        .widgets
        .get_id_by_name("ChatFrame1EditBox")
        .expect("ChatFrame1EditBox not found in widget registry");
    env.send_click(frame_id).expect("send_click failed");

    let has_focus: bool = env
        .eval("return ChatFrame1EditBox:HasFocus()")
        .expect("HasFocus failed");
    assert!(has_focus, "ChatFrame1EditBox should have focus after click");
}

/// Assert exactly one message was sent with expected text and chat type.
fn assert_message_sent(env: &WowLuaEnv, expected_text: &str, expected_type: &str) {
    let count: i32 = env
        .eval("return #_G.__test_sent_messages")
        .expect("eval failed");
    assert_eq!(count, 1, "Exactly one message should have been sent");

    let message: String = env
        .eval("return _G.__test_sent_messages[1].message")
        .expect("eval failed");
    assert_eq!(message, expected_text, "Sent message should match typed text");

    let chat_type: String = env
        .eval("return _G.__test_sent_messages[1].chatType")
        .expect("eval failed");
    assert_eq!(chat_type, expected_type, "Chat type should match expected");

    let text_after: String = env
        .eval("return ChatFrame1EditBox:GetText() or ''")
        .expect("GetText failed");
    assert_eq!(text_after, "", "EditBox should be cleared after submit");
}

#[test]
fn test_chat_editbox_click_type_and_submit() {
    let env = setup_env();

    let exists: bool = env
        .eval("return ChatFrame1EditBox ~= nil")
        .expect("eval failed");
    assert!(exists, "ChatFrame1EditBox should exist after loading Blizzard UI");

    hook_send_chat_message(&env);

    let has_focus: bool = env
        .eval("return ChatFrame1EditBox:HasFocus()")
        .expect("HasFocus failed");
    assert!(!has_focus, "ChatFrame1EditBox should not have focus initially");

    click_chat_editbox(&env);
    type_text(&env, "hello world");

    let text: String = env
        .eval("return ChatFrame1EditBox:GetText()")
        .expect("GetText failed");
    assert_eq!(text, "hello world", "EditBox should contain typed text");

    env.send_key_press("ENTER", None)
        .expect("ENTER key press failed");

    assert_message_sent(&env, "hello world", "SAY");

    let message: String = env
        .eval("return _G.__test_sent_messages[1].message")
        .expect("eval failed");
    assert_eq!(message, "hello world", "Sent message should match typed text");

    let chat_type: String = env
        .eval("return _G.__test_sent_messages[1].chatType")
        .expect("eval failed");
    assert_eq!(chat_type, "SAY", "Default chat type should be SAY");

    let text_after: String = env
        .eval("return ChatFrame1EditBox:GetText() or ''")
        .expect("GetText failed");
    assert_eq!(text_after, "", "EditBox should be cleared after submit");
}

#[test]
fn test_chat_editbox_text_color_after_activation() {
    let env = setup_env();

    click_chat_editbox(&env);

    // After activation, ActivateChat should have called UpdateHeader
    // which sets text color to white (ChatTypeInfo default = 1.0, 1.0, 1.0)
    let (r, g, b): (f64, f64, f64) = env
        .eval("return ChatFrame1EditBox:GetTextColor()")
        .expect("GetTextColor failed");
    assert!(
        (r - 1.0).abs() < 0.01 && (g - 1.0).abs() < 0.01 && (b - 1.0).abs() < 0.01,
        "EditBox text color should be white after activation, got ({r}, {g}, {b})"
    );

    // Alpha should be 1.0 after activation
    let alpha: f64 = env
        .eval("return ChatFrame1EditBox:GetAlpha()")
        .expect("GetAlpha failed");
    assert!(
        (alpha - 1.0).abs() < 0.01,
        "EditBox alpha should be 1.0 after activation, got {alpha}"
    );
}
