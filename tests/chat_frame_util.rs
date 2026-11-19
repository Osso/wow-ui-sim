//! Integration tests for `ChatFrameUtil.AddSystemMessage`.
//!
//! These tests use a bare `WowLuaEnv` (no Blizzard UI loading) and verify:
//! - The Rust function is registered on the global `ChatFrameUtil` table.
//! - Each invocation pushes the message text onto `state.system_chat_log`.
//! - When a chat frame is available, the message is routed to its
//!   `AddMessage` handler with the SYSTEM colour (yellow) and channel id.
//! - The function tolerates a missing chat frame and non-string arguments
//!   without erroring.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("WowLuaEnv init")
}

#[test]
fn add_system_message_registered_on_chat_frame_util() {
    let env = env();
    let kind: String = env
        .eval("return type(ChatFrameUtil.AddSystemMessage)")
        .unwrap();
    assert_eq!(kind, "function");
}

#[test]
fn chat_type_group_registered_on_chat_frame_surface() {
    let env = env();
    let (system_count, system_first, system_last, raid_warning): (i32, String, String, String) =
        env.eval(
            r#"
            return #ChatTypeGroup.SYSTEM,
                   ChatTypeGroup.SYSTEM[1],
                   ChatTypeGroup.SYSTEM[5],
                   ChatTypeGroup.RAID[3]
            "#,
        )
        .unwrap();

    assert_eq!(system_count, 5);
    assert_eq!(system_first, "SYSTEM");
    assert_eq!(system_last, "CHANNEL_NOTICE_USER");
    assert_eq!(raid_warning, "RAID_WARNING");
}

#[test]
fn add_system_message_pushes_to_system_chat_log() {
    let env = env();
    env.exec(r#"ChatFrameUtil.AddSystemMessage("hello, world")"#)
        .unwrap();

    let state = env.state().borrow();
    assert_eq!(state.system_chat_log, vec!["hello, world".to_string()]);
}

#[test]
fn add_system_message_preserves_order() {
    let env = env();
    env.exec(
        r#"
        ChatFrameUtil.AddSystemMessage("first")
        ChatFrameUtil.AddSystemMessage("second")
        ChatFrameUtil.AddSystemMessage("third")
        "#,
    )
    .unwrap();

    let state = env.state().borrow();
    assert_eq!(
        state.system_chat_log,
        vec![
            "first".to_string(),
            "second".to_string(),
            "third".to_string(),
        ]
    );
}

#[test]
fn add_system_message_handles_empty_string() {
    let env = env();
    env.exec(r#"ChatFrameUtil.AddSystemMessage("")"#).unwrap();

    let state = env.state().borrow();
    assert_eq!(state.system_chat_log, vec![String::new()]);
}

#[test]
fn add_system_message_routes_to_default_chat_frame() {
    let env = env();
    env.exec(
        r#"
        _G.__captured = {}
        DEFAULT_CHAT_FRAME = {
            AddMessage = function(self, text, r, g, b, id)
                table.insert(_G.__captured, {
                    text = text, r = r, g = g, b = b, id = id,
                })
            end,
        }
        ChatFrameUtil.AddSystemMessage("system warning")
        "#,
    )
    .unwrap();

    let (count, text, r, g, b, id): (i32, String, f64, f64, f64, f64) = env
        .eval(
            r#"
            return #_G.__captured,
                _G.__captured[1].text,
                _G.__captured[1].r,
                _G.__captured[1].g,
                _G.__captured[1].b,
                _G.__captured[1].id
            "#,
        )
        .unwrap();

    assert_eq!(count, 1);
    assert_eq!(text, "system warning");
    assert_eq!((r, g, b), (1.0, 1.0, 0.0));
    assert_eq!(id, 1.0);
}

#[test]
fn add_system_message_falls_back_to_chat_frame1() {
    let env = env();
    env.exec(
        r#"
        DEFAULT_CHAT_FRAME = nil
        _G.__captured_text = nil
        ChatFrame1 = {
            AddMessage = function(self, text)
                _G.__captured_text = text
            end,
        }
        ChatFrameUtil.AddSystemMessage("fallback path")
        "#,
    )
    .unwrap();

    let captured: String = env.eval("return _G.__captured_text").unwrap();
    assert_eq!(captured, "fallback path");
}

#[test]
fn add_system_message_no_chat_frame_still_logs() {
    let env = env();
    env.exec(
        r#"
        DEFAULT_CHAT_FRAME = nil
        ChatFrame1 = nil
        ChatFrameUtil.AddSystemMessage("orphan message")
        "#,
    )
    .unwrap();

    let state = env.state().borrow();
    assert_eq!(state.system_chat_log, vec!["orphan message".to_string()]);
}

#[test]
fn add_system_message_with_no_args_records_empty_string() {
    let env = env();
    env.exec("ChatFrameUtil.AddSystemMessage()").unwrap();

    let state = env.state().borrow();
    assert_eq!(state.system_chat_log, vec![String::new()]);
}

#[test]
fn add_system_message_does_not_clobber_other_chat_frame_util_entries() {
    // VERIFIED — Blizzard_ChatFrameBase.toc loads Shared/ChatFrameFilters.lua.
    let env = crate::common::panel_fixtures::setup_env();
    let process_kind: String = env
        .eval("return type(ChatFrameUtil.ProcessMessageEventFilters)")
        .unwrap();
    assert_eq!(process_kind, "function");

    env.exec("ChatFrameUtil.AddSystemMessage('entry preservation')")
        .unwrap();

    let process_kind: String = env
        .eval("return type(ChatFrameUtil.ProcessMessageEventFilters)")
        .unwrap();
    assert_eq!(
        process_kind, "function",
        "AddSystemMessage must preserve the original chat filter function"
    );
}
