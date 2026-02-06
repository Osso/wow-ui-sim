//! Tests for MessageFrame / ScrollingMessageFrame implementation.

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn test_create_message_frame_type() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(r#"local f = CreateFrame("ScrollingMessageFrame", "TestMF", UIParent)"#)
        .unwrap();

    let obj_type: String = env.eval("return TestMF:GetObjectType()").unwrap();
    assert_eq!(obj_type, "MessageFrame");
}

#[test]
fn test_message_frame_is_object_type_frame() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(r#"local f = CreateFrame("MessageFrame", "TestMF2", UIParent)"#)
        .unwrap();

    let is_frame: bool = env.eval("return TestMF2:IsObjectType('Frame')").unwrap();
    assert!(is_frame);

    let is_mf: bool = env.eval("return TestMF2:IsObjectType('MessageFrame')").unwrap();
    assert!(is_mf);
}

#[test]
fn test_add_message_and_num_messages() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("ScrollingMessageFrame", "TestMFAdd", UIParent)
        f:AddMessage("Hello", 1, 1, 1)
        f:AddMessage("World", 0, 1, 0)
    "#,
    )
    .unwrap();

    let count: i32 = env.eval("return TestMFAdd:GetNumMessages()").unwrap();
    assert_eq!(count, 2);
}

#[test]
fn test_add_msg_alias() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("ScrollingMessageFrame", "TestMFAlias", UIParent)
        f:AddMsg("Test message")
    "#,
    )
    .unwrap();

    let count: i32 = env.eval("return TestMFAlias:GetNumMessages()").unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_clear_messages() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("ScrollingMessageFrame", "TestMFClear", UIParent)
        f:AddMessage("Line 1")
        f:AddMessage("Line 2")
        f:Clear()
    "#,
    )
    .unwrap();

    let count: i32 = env.eval("return TestMFClear:GetNumMessages()").unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_set_max_lines_truncates() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("ScrollingMessageFrame", "TestMFMax", UIParent)
        f:SetMaxLines(2)
        f:AddMessage("One")
        f:AddMessage("Two")
        f:AddMessage("Three")
    "#,
    )
    .unwrap();

    let count: i32 = env.eval("return TestMFMax:GetNumMessages()").unwrap();
    assert_eq!(count, 2, "Should truncate to max_lines");

    let max: i32 = env.eval("return TestMFMax:GetMaxLines()").unwrap();
    assert_eq!(max, 2);
}

#[test]
fn test_fading_set_get() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("ScrollingMessageFrame", "TestMFFade", UIParent)
        f:SetFading(false)
    "#,
    )
    .unwrap();

    let fading: bool = env.eval("return TestMFFade:GetFading()").unwrap();
    assert!(!fading);
}

#[test]
fn test_time_visible_set_get() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("ScrollingMessageFrame", "TestMFTime", UIParent)
        f:SetTimeVisible(30)
    "#,
    )
    .unwrap();

    let time: f64 = env.eval("return TestMFTime:GetTimeVisible()").unwrap();
    assert_eq!(time, 30.0);
}

#[test]
fn test_fade_duration_set_get() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("ScrollingMessageFrame", "TestMFFadeDur", UIParent)
        f:SetFadeDuration(5)
    "#,
    )
    .unwrap();

    let dur: f64 = env.eval("return TestMFFadeDur:GetFadeDuration()").unwrap();
    assert_eq!(dur, 5.0);
}

#[test]
fn test_insert_mode_set_get() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("ScrollingMessageFrame", "TestMFInsert", UIParent)
        f:SetInsertMode("TOP")
    "#,
    )
    .unwrap();

    let mode: String = env.eval("return TestMFInsert:GetInsertMode()").unwrap();
    assert_eq!(mode, "TOP");
}

#[test]
fn test_get_message_info() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("ScrollingMessageFrame", "TestMFInfo", UIParent)
        f:AddMessage("Hello", 0.5, 0.6, 0.7)
    "#,
    )
    .unwrap();

    let (text, r, g, b): (String, f64, f64, f64) = env
        .eval("local t, r, g, b = TestMFInfo:GetMessageInfo(1); return t, r, g, b")
        .unwrap();
    assert_eq!(text, "Hello");
    assert!((r - 0.5).abs() < 0.01);
    assert!((g - 0.6).abs() < 0.01);
    assert!((b - 0.7).abs() < 0.01);
}

#[test]
fn test_builtin_default_chat_frame_exists() {
    let env = WowLuaEnv::new().unwrap();

    let exists: bool = env.eval("return DEFAULT_CHAT_FRAME ~= nil").unwrap();
    assert!(exists, "DEFAULT_CHAT_FRAME should exist");

    let obj_type: String = env
        .eval("return DEFAULT_CHAT_FRAME:GetObjectType()")
        .unwrap();
    assert_eq!(obj_type, "MessageFrame");
}

#[test]
fn test_builtin_chat_frame_accepts_messages() {
    let env = WowLuaEnv::new().unwrap();

    // Built-in frames should lazily init MessageFrameData
    env.exec(r#"DEFAULT_CHAT_FRAME:AddMessage("Test", 1, 1, 1)"#)
        .unwrap();

    let count: i32 = env
        .eval("return DEFAULT_CHAT_FRAME:GetNumMessages()")
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_insert_mode_top_prepends() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("ScrollingMessageFrame", "TestMFTop", UIParent)
        f:SetInsertMode("TOP")
        f:AddMessage("First")
        f:AddMessage("Second")
    "#,
    )
    .unwrap();

    // With TOP insert mode, "Second" should be at index 1
    let text: String = env
        .eval("local t = TestMFTop:GetMessageInfo(1); return t")
        .unwrap();
    assert_eq!(text, "Second");
}
