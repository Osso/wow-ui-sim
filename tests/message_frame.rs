//! Tests for MessageFrame / ScrollingMessageFrame implementation.

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn test_create_message_frame_type() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(r#"local f = CreateFrame("ScrollingMessageFrame", "TestMF", UIParent)"#)
        .unwrap();

    let obj_type: String = env.eval("return TestMF:GetObjectType()").unwrap();
    assert_eq!(obj_type, "ScrollingMessageFrame");
}

#[test]
fn test_message_frame_is_object_type_frame() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(r#"local f = CreateFrame("MessageFrame", "TestMF2", UIParent)"#)
        .unwrap();

    let is_frame: bool = env.eval("return TestMF2:IsObjectType('Frame')").unwrap();
    assert!(is_frame);

    let is_mf: bool = env
        .eval("return TestMF2:IsObjectType('MessageFrame')")
        .unwrap();
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

fn assert_clear_preserves_other_widgets(widget_type: &str) {
    let env = WowLuaEnv::new().unwrap();
    env.exec(&format!(
        r#"local frame = CreateFrame("{widget_type}")
        local other = CreateFrame("{widget_type}")
        local cooldown = CreateFrame("Cooldown")
        cooldown:SetCooldown(12, 8, 2)
        frame:AddMessage("First")
        frame:AddMessage("Second")
        other:AddMessage("Keep")
        assert(frame:GetNumMessages() == 2)
        assert(select('#', frame:Clear()) == 0)
        assert(frame:GetNumMessages() == 0)
        assert(frame:GetScrollOffset() == 0)
        assert(other:GetNumMessages() == 1)
        assert(other:GetMessageInfo(1) == "Keep")
        local start, duration = cooldown:GetCooldownTimes()
        assert(start == 12000 and duration == 8000)
        assert(cooldown:GetCooldownDisplayDuration() == 8000)
        frame:AddMessage("After clear")
        assert(frame:GetNumMessages() == 1)
        assert(frame:GetMessageInfo(1) == "After clear")
        frame:ClearText()
        assert(frame:GetNumMessages() == 0)
    "#
    ))
    .unwrap();
}

#[test]
fn clear_message_frame_history_preserves_other_widgets() {
    assert_clear_preserves_other_widgets("MessageFrame");
}

#[test]
fn clear_scrolling_message_frame_history_preserves_other_widgets() {
    assert_clear_preserves_other_widgets("ScrollingMessageFrame");
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

#[test]
fn test_message_frame_indented_word_wrap_round_trips() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("ScrollingMessageFrame", "TestMFIndentedWrap", UIParent)
        f:SetIndentedWordWrap(true)
    "#,
    )
    .unwrap();

    let wrapped: bool = env
        .eval("return TestMFIndentedWrap:GetIndentedWordWrap()")
        .unwrap();
    assert!(wrapped);
}

#[test]
fn test_backfill_message_prepends() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("ScrollingMessageFrame", "TestMFBack", UIParent)
        f:AddMessage("First")
        f:BackFillMessage("BackFilled")
    "#,
    )
    .unwrap();

    let count: i32 = env.eval("return TestMFBack:GetNumMessages()").unwrap();
    assert_eq!(count, 2);

    // BackFilled should be at index 1 (inserted at front)
    let text: String = env
        .eval("local t = TestMFBack:GetMessageInfo(1); return t")
        .unwrap();
    assert_eq!(text, "BackFilled");
}

#[test]
fn test_fade_power_set_get() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("ScrollingMessageFrame", "TestMFPower", UIParent)
        f:SetFadePower(2.5)
    "#,
    )
    .unwrap();

    let power: f64 = env.eval("return TestMFPower:GetFadePower()").unwrap();
    assert_eq!(power, 2.5);
}

#[test]
fn test_scroll_offset_set_get() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("ScrollingMessageFrame", "TestMFScroll", UIParent)
        f:AddMessage("Line")
        f:SetScrollOffset(5)
    "#,
    )
    .unwrap();

    let offset: i32 = env.eval("return TestMFScroll:GetScrollOffset()").unwrap();
    assert_eq!(offset, 5);
}

#[test]
fn test_scroll_allowed_set_get() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("ScrollingMessageFrame", "TestMFScrollAllow", UIParent)
        f:SetScrollAllowed(false)
    "#,
    )
    .unwrap();

    let allowed: bool = env
        .eval("return TestMFScrollAllow:IsScrollAllowed()")
        .unwrap();
    assert!(!allowed);
}

#[test]
fn test_text_copyable_set_get() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("ScrollingMessageFrame", "TestMFCopy", UIParent)
        f:SetTextCopyable(true)
    "#,
    )
    .unwrap();

    let copyable: bool = env.eval("return TestMFCopy:IsTextCopyable()").unwrap();
    assert!(copyable);
}

#[test]
fn test_has_message_by_id() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("ScrollingMessageFrame", "TestMFByID", UIParent)
        f:AddMessage("Tagged", 1, 1, 1, 1, 42)
        f:AddMessage("Untagged")
    "#,
    )
    .unwrap();

    let has_42: bool = env.eval("return TestMFByID:HasMessageByID(42)").unwrap();
    assert!(has_42, "Should find message with ID 42");

    let has_99: bool = env.eval("return TestMFByID:HasMessageByID(99)").unwrap();
    assert!(!has_99, "Should not find message with ID 99");
}

#[test]
fn test_add_message_stores_timestamp() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("ScrollingMessageFrame", "TestMFTimestamp", UIParent)
        _G.__before = GetTime()
        f:AddMessage("Timed message", 1, 1, 1)
        _G.__after = GetTime()
    "#,
    )
    .unwrap();

    // GetMessageInfo returns (text, r, g, b, a, timestamp) — timestamp is the 6th value
    let (before, after, ts): (f64, f64, f64) = env
        .eval(
            r#"
        local _, _, _, _, _, timestamp = TestMFTimestamp:GetMessageInfo(1)
        return _G.__before, _G.__after, timestamp
    "#,
        )
        .unwrap();

    assert!(ts >= before, "timestamp {ts} should be >= before {before}");
    assert!(ts <= after, "timestamp {ts} should be <= after {after}");
    assert!(ts > 0.0, "timestamp should be positive (GetTime-based)");
}

#[test]
fn test_max_lines_truncates_existing() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("ScrollingMessageFrame", "TestMFTrunc", UIParent)
        f:AddMessage("One")
        f:AddMessage("Two")
        f:AddMessage("Three")
        f:SetMaxLines(2)
    "#,
    )
    .unwrap();

    let count: i32 = env.eval("return TestMFTrunc:GetNumMessages()").unwrap();
    assert_eq!(count, 2, "SetMaxLines should truncate existing messages");
}

#[test]
fn test_scroll_operations_update_offset_and_clamp_to_message_history() {
    let env = WowLuaEnv::new().unwrap();

    let result: (i32, i32, i32, i32, i32, i32) = env
        .eval(
            r#"
            local f = CreateFrame("ScrollingMessageFrame", "TestMFScrollOps", UIParent)
            f:AddMessage("One")
            f:AddMessage("Two")
            f:AddMessage("Three")
            f:AddMessage("Four")

            f:ScrollUp()
            local afterScrollUp = f:GetScrollOffset()

            f:PageUp()
            local afterPageUp = f:GetScrollOffset()

            f:ScrollToBottom()
            local afterScrollToBottom = f:GetScrollOffset()

            f:ScrollToTop()
            local afterScrollToTop = f:GetScrollOffset()

            f:PageDown()
            local afterPageDown = f:GetScrollOffset()

            f:ScrollDown()
            local afterScrollDown = f:GetScrollOffset()

            return afterScrollUp, afterPageUp, afterScrollToBottom, afterScrollToTop, afterPageDown, afterScrollDown
        "#,
        )
        .unwrap();

    assert_eq!(result.0, 1, "ScrollUp should increment the scroll offset");
    assert_eq!(
        result.1, 3,
        "PageUp should clamp at the top of message history"
    );
    assert_eq!(result.2, 0, "ScrollToBottom should reset the offset");
    assert_eq!(
        result.3, 3,
        "ScrollToTop should jump to the highest valid offset"
    );
    assert_eq!(result.4, 0, "PageDown should clamp back toward the bottom");
    assert_eq!(result.5, 0, "ScrollDown should clamp at the bottom");
}

#[test]
fn test_scroll_position_queries_follow_history_and_truncation() {
    let env = WowLuaEnv::new().unwrap();

    let result: (i32, bool, bool, i32, bool, bool) = env
        .eval(
            r#"
            local f = CreateFrame("ScrollingMessageFrame", "TestMFScrollInfo", UIParent)
            f:AddMessage("One")
            f:AddMessage("Two")
            f:AddMessage("Three")

            local initialRange = f:GetMaxScrollRange()
            local initialAtBottom = f:AtBottom()
            local initialAtTop = f:AtTop()

            f:SetMaxLines(2)
            f:ScrollToTop()

            local truncatedRange = f:GetMaxScrollRange()
            local truncatedAtTop = f:AtTop()
            local truncatedAtBottom = f:AtBottom()

            return initialRange, initialAtBottom, initialAtTop, truncatedRange, truncatedAtTop, truncatedAtBottom
        "#,
        )
        .unwrap();

    assert_eq!(
        result.0, 2,
        "three messages should yield a max scroll range of two"
    );
    assert!(result.1, "new frames should start at the bottom");
    assert!(
        !result.2,
        "new frames with history should not start at the top"
    );
    assert_eq!(
        result.3, 1,
        "truncating to two lines should reduce max scroll range"
    );
    assert!(result.4, "ScrollToTop should move the frame to the top");
    assert!(!result.5, "top position should no longer report bottom");
}

#[test]
fn test_message_frame_callbacks_fire_for_scroll_and_display_refresh() {
    let env = WowLuaEnv::new().unwrap();

    let result: (i32, i32, i32, String) = env
        .eval(
            r#"
            local f = CreateFrame("ScrollingMessageFrame", "TestMFCallbacks", UIParent)
            f:AddMessage("One")
            f:AddMessage("Two")
            f:AddMessage("Three")

            local scrollCalls = 0
            local lastOffset = -1
            local refreshCalls = 0
            local lastRefreshName = ""

            f:SetOnScrollChangedCallback(function(self, offset)
                scrollCalls = scrollCalls + 1
                lastOffset = offset
            end)

            f:AddOnDisplayRefreshedCallback(function(self)
                refreshCalls = refreshCalls + 1
                lastRefreshName = self:GetName()
            end)

            f:SetScrollOffset(1)
            f:SetScrollOffset(1)
            f:MarkDisplayDirty()
            f:ResetAllFadeTimes()

            return scrollCalls, lastOffset, refreshCalls, lastRefreshName
        "#,
        )
        .unwrap();

    assert_eq!(
        result.0, 1,
        "scroll callback should only fire when the offset changes"
    );
    assert_eq!(result.1, 1, "scroll callback should receive the new offset");
    assert_eq!(
        result.2, 2,
        "display refresh callbacks should run for explicit dirty/reset calls"
    );
    assert_eq!(result.3, "TestMFCallbacks");
}

#[test]
fn test_message_frame_transform_and_filter_apis_update_history() {
    let env = WowLuaEnv::new().unwrap();

    let result: (i32, String, f64, f64, f64, String, f64, f64, f64) = env
        .eval(
            r#"
            local f = CreateFrame("ScrollingMessageFrame", "TestMFTransform", UIParent)
            f:AddMessage("Keep", 1, 0, 0)
            f:AddMessage("Drop", 0, 1, 0)
            f:AddMessage("Swap", 0, 0, 1)

            local refreshCalls = 0
            f:AddOnDisplayRefreshedCallback(function()
                refreshCalls = refreshCalls + 1
            end)

            f:RemoveMessagesByPredicate(function(message)
                return message == "Drop"
            end)

            f:AdjustMessageColors(function(message, r, g, b)
                if message == "Keep" then
                    return true, 0.25, 0.5, 0.75
                end
                return false
            end)

            f:TransformMessages(
                function(message)
                    return message == "Swap"
                end,
                function(message, r, g, b)
                    return "Swapped", 0.9, 0.8, 0.7
                end
            )

            local keepText, keepR, keepG, keepB = f:GetMessageInfo(1)
            local swapText, swapR, swapG, swapB = f:GetMessageInfo(2)
            return refreshCalls, keepText, keepR, keepG, keepB, swapText, swapR, swapG, swapB
        "#,
        )
        .unwrap();

    assert_eq!(
        result.0, 3,
        "each mutating API should mark the display dirty once"
    );
    assert_eq!(result.1, "Keep");
    assert!((result.2 - 0.25).abs() < 0.01);
    assert!((result.3 - 0.5).abs() < 0.01);
    assert!((result.4 - 0.75).abs() < 0.01);
    assert_eq!(result.5, "Swapped");
    assert!((result.6 - 0.9).abs() < 0.01);
    assert!((result.7 - 0.8).abs() < 0.01);
    assert!((result.8 - 0.7).abs() < 0.01);
}

#[test]
fn test_message_frame_fontstring_lookup_and_fade_reset_update_runtime_state() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("ScrollingMessageFrame", "TestMFFadeReset", UIParent)
        f:AddMessage("Repeated", 0.8, 0.4, 0.2, 1.0, 77)
    "#,
    )
    .unwrap();

    let frame_id = env
        .state()
        .borrow()
        .widgets
        .get_id_by_name("TestMFFadeReset")
        .unwrap();

    let (text, r, g, b): (String, f64, f64, f64) = env
        .eval(
            r#"
            local fontString = TestMFFadeReset:GetFontStringByID(77)
            local text = fontString and fontString:GetText() or ""
            local r, g, b = fontString:GetTextColor()
            return text, r, g, b
        "#,
        )
        .unwrap();
    assert_eq!(text, "Repeated");
    assert!((r - 0.8).abs() < 0.01);
    assert!((g - 0.4).abs() < 0.01);
    assert!((b - 0.2).abs() < 0.01);

    let original_timestamp = {
        let state = env.state().borrow();
        state.message_frames.get(&frame_id).unwrap().messages[0].timestamp
    };

    std::thread::sleep(std::time::Duration::from_millis(10));
    env.exec(r#"TestMFFadeReset:ResetMessageFadeByID(77)"#)
        .unwrap();

    let reset_timestamp = {
        let state = env.state().borrow();
        state.message_frames.get(&frame_id).unwrap().messages[0].timestamp
    };
    assert!(
        reset_timestamp > original_timestamp,
        "ResetMessageFadeByID should refresh the stored timestamp"
    );

    {
        let mut state = env.state().borrow_mut();
        let data = state.message_frames.get_mut(&frame_id).unwrap();
        data.display_dirty = false;
        data.override_fade_timestamp = 0.0;
    }

    env.exec(
        r#"
        TestMFFadeReset:MarkDisplayDirty()
        TestMFFadeReset:ResetAllFadeTimes()
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let data = state.message_frames.get(&frame_id).unwrap();
    assert!(
        data.display_dirty,
        "dirty methods should set real display-dirty state"
    );
    assert!(
        data.override_fade_timestamp > 0.0,
        "ResetAllFadeTimes should update the override fade timestamp"
    );
}
