//! Integration tests for the key dispatch subsystem.
//!
//! Covers: ESC dispatch, keybinding lookup + execution, EditBox text input,
//! and unbound key as a silent no-op.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("WowLuaEnv init")
}

#[test]
fn key_down_state_tracks_real_dispatch_and_release() {
    let env = env();
    env.exec(
        r#"
        local frame = CreateFrame("Frame", "KeyDownStateFrame", UIParent)
        frame:EnableKeyboard(true)
        frame:SetScript("OnKeyDown", function(_, key)
            _G.key_down_seen = { key, IsKeyDown(key), IsKeyDown("F2") }
        end)
        "#,
    )
    .unwrap();
    let id = env
        .state()
        .borrow()
        .widgets
        .get_id_by_name("KeyDownStateFrame")
        .unwrap();
    env.state().borrow_mut().focused_frame_id = Some(id);

    assert!(!env.eval::<bool>("return IsKeyDown('Q')").unwrap());
    env.send_key_down("Q", None).unwrap();
    let seen: (String, bool, bool) = env.eval("return unpack(_G.key_down_seen)").unwrap();
    assert_eq!(seen, ("Q".into(), true, false));
    assert!(env.eval::<bool>("return IsKeyDown('Q')").unwrap());
    env.send_key_up("Q");
    assert!(!env.eval::<bool>("return IsKeyDown('Q')").unwrap());
}

#[test]
fn synthetic_key_press_releases_and_headless_keys_start_up() {
    let env = env();
    assert!(!env.eval::<bool>("return IsKeyDown('F2')").unwrap());
    env.send_key_press("F2", None).unwrap();
    assert!(!env.eval::<bool>("return IsKeyDown('F2')").unwrap());
}

// ── ESC dispatch ──────────────────────────────────────────────────────────────

#[test]
fn esc_with_focused_editbox_fires_on_escape_pressed_and_returns_early() {
    let env = env();
    // Create an editbox, set an OnEscapePressed that sets a flag and returns true.
    env.exec(
        r#"
        _G.escaped = false
        local eb = CreateFrame("EditBox", "TestEscEB", UIParent)
        eb:SetScript("OnEscapePressed", function(self)
            _G.escaped = true
            return true
        end)
        "#,
    )
    .unwrap();

    // Focus the editbox by clicking it.
    let eb_id = {
        let state = env.state();
        let sim = state.borrow();
        sim.widgets.get_id_by_name("TestEscEB")
    };
    if let Some(id) = eb_id {
        env.state().borrow_mut().focused_frame_id = Some(id);
    }

    env.send_key_press("ESCAPE", None).unwrap();

    let escaped: bool = env.eval("return _G.escaped").unwrap();
    assert!(escaped, "OnEscapePressed should have fired");
}

#[test]
fn esc_does_not_error_with_no_focus() {
    let env = env();
    // Sending ESC with no focus and no target should be a no-op (or toggle
    // GameMenuFrame) without panicking or returning an error.
    let result = env.send_key_press("ESCAPE", None);
    assert!(result.is_ok(), "ESC with no focus should not error");
}

#[test]
fn esc_dispatches_blizzard_toggle_game_menu_binding() {
    let env = env();
    env.exec(
        r#"
        _G.escape_count = 0
        function ToggleGameMenu()
            _G.escape_count = _G.escape_count + 1
        end
        "#,
    )
    .unwrap();

    env.send_key_press("ESCAPE", None).unwrap();

    let count: i32 = env.eval("return _G.escape_count").unwrap();
    assert_eq!(count, 1, "ESC should run the TOGGLEGAMEMENU binding");
}

#[test]
fn spell_stop_casting_clears_active_cast() {
    let env = env();
    let (first_stop, casting_after_first, second_stop): (bool, bool, bool) = env
        .eval(
            r#"
            CastSpellByID(19750)
            local firstStop = SpellStopCasting()
            local castingAfterFirst = UnitCastingInfo("player") ~= nil
            local secondStop = SpellStopCasting()
            return firstStop, castingAfterFirst, secondStop
            "#,
        )
        .unwrap();

    assert!(
        first_stop,
        "SpellStopCasting should report an interrupted cast"
    );
    assert!(
        !casting_after_first,
        "SpellStopCasting should clear casting state"
    );
    assert!(
        !second_stop,
        "SpellStopCasting should return false when nothing is casting"
    );
}

// ── Keybinding dispatch ───────────────────────────────────────────────────────

#[test]
fn bound_key_dispatches_lua_code() {
    let env = env();
    // Set a custom binding whose Lua code sets a global flag.
    env.exec(
        r#"
        _G.binding_fired = false
        SetBinding("CTRL-X", "TOGGLEBACKPACK")
        -- Override ToggleBackpack to set our flag instead
        function ToggleBackpack() _G.binding_fired = true end
        "#,
    )
    .unwrap();

    env.send_key_press("CTRL-X", None).unwrap();

    let fired: bool = env.eval("return _G.binding_fired").unwrap();
    assert!(fired, "bound key should have dispatched its Lua code");
}

#[test]
fn unbound_key_is_silent_no_op() {
    let env = env();
    // A key that has no binding should do nothing and not error.
    let result = env.send_key_press("F12", None);
    assert!(result.is_ok(), "unbound key should be a silent no-op");
}

#[test]
fn default_keybindings_dispatch_on_init() {
    let env = env();
    env.exec(
        r#"
        _G.open_all_bags_fired = false
        function ToggleAllBags()
            _G.open_all_bags_fired = true
        end
        "#,
    )
    .unwrap();

    env.send_key_press("B", None).unwrap();

    let fired: bool = env.eval("return _G.open_all_bags_fired").unwrap();
    assert!(fired, "B should dispatch the default OPENALLBAGS binding");
}

#[test]
fn default_ctrl_q_binding_requests_simulator_exit() {
    let env = env();

    env.send_key_press("CTRL-Q", None).unwrap();

    let requested: bool = env
        .eval("return A_Admin.IsSimulatorExitRequested()")
        .unwrap();
    assert!(requested, "CTRL-Q should dispatch the default Quit binding");
}

#[test]
fn ctrl_q_control_character_without_modifier_requests_simulator_exit() {
    let env = env();

    env.send_key_press("\u{11}", None).unwrap();

    let requested: bool = env
        .eval("return A_Admin.IsSimulatorExitRequested()")
        .unwrap();
    assert!(
        requested,
        "Ctrl+Q may arrive from iced as ASCII DC1 without a modifier flag"
    );
}

#[test]
fn quit_game_global_requests_simulator_exit() {
    let env = env();

    env.exec("QuitGame()").unwrap();

    let requested: bool = env
        .eval("return A_Admin.IsSimulatorExitRequested()")
        .unwrap();
    assert!(requested, "QuitGame should use the simulator exit path");
}

#[test]
fn action_button_key_dispatch_casts_once_through_button_down() {
    let env = env();
    env.exec(
        r#"
        A_Admin.SetActionSlot(1, 19750)
        local button = CreateFrame("Button", "ActionButton1", UIParent, "SecureActionButtonTemplate")
        button.action = 1
        button:SetButtonState("NORMAL")

        _G.spellcast_start_count = 0
        local listener = CreateFrame("Frame")
        listener:RegisterEvent("UNIT_SPELLCAST_START")
        listener:SetScript("OnEvent", function()
            _G.spellcast_start_count = _G.spellcast_start_count + 1
        end)
        "#,
    )
    .unwrap();

    env.send_key_press("1", None).unwrap();

    let count: i64 = env.eval("return _G.spellcast_start_count").unwrap();
    assert_eq!(
        count, 1,
        "ACTIONBUTTON1 key dispatch should not call both ActionButtonDown and UseAction"
    );
}

// ── EditBox text input ────────────────────────────────────────────────────────

#[test]
fn editbox_receives_typed_text() {
    let env = env();
    env.exec(
        r#"
        local eb = CreateFrame("EditBox", "TestTypeEB", UIParent)
        eb:SetText("")
        "#,
    )
    .unwrap();

    // Focus the editbox.
    let eb_id = {
        let state = env.state();
        let sim = state.borrow();
        sim.widgets.get_id_by_name("TestTypeEB")
    };
    if let Some(id) = eb_id {
        env.state().borrow_mut().focused_frame_id = Some(id);
    }

    // Send individual characters.
    env.send_key_press("a", Some("a")).unwrap();
    env.send_key_press("b", Some("b")).unwrap();
    env.send_key_press("c", Some("c")).unwrap();

    // Read back via GetText.
    let text: String = env
        .eval(r#"return TestTypeEB:GetText()"#)
        .unwrap_or_default();
    assert_eq!(text, "abc", "EditBox should contain the typed characters");
}

#[test]
fn focused_editbox_inserts_printable_key_when_text_payload_is_missing() {
    let env = env();
    env.exec(
        r#"
        local eb = CreateFrame("EditBox", "TestFallbackTextEB", UIParent)
        eb:SetText("")
        eb:SetFocus()
        "#,
    )
    .unwrap();

    env.send_key_press("A", None).unwrap();
    env.send_key_press("B", None).unwrap();

    let text: String = env.eval(r#"return TestFallbackTextEB:GetText()"#).unwrap();
    assert_eq!(
        text, "ab",
        "focused EditBox should insert printable key names when the GUI event has no text payload"
    );
}

#[test]
fn editbox_on_text_changed_fires_on_input() {
    let env = env();
    env.exec(
        r#"
        _G.change_count = 0
        local eb = CreateFrame("EditBox", "TestChangedEB", UIParent)
        eb:SetScript("OnTextChanged", function(self, userInput)
            if userInput then _G.change_count = _G.change_count + 1 end
        end)
        "#,
    )
    .unwrap();

    let eb_id = {
        let state = env.state();
        let sim = state.borrow();
        sim.widgets.get_id_by_name("TestChangedEB")
    };
    if let Some(id) = eb_id {
        env.state().borrow_mut().focused_frame_id = Some(id);
    }

    env.send_key_press("x", Some("x")).unwrap();
    env.send_key_press("y", Some("y")).unwrap();

    let count: i64 = env.eval("return _G.change_count").unwrap_or(0);
    assert_eq!(count, 2, "OnTextChanged should fire once per character");
}

#[test]
fn editbox_backspace_removes_last_character() {
    let env = env();
    env.exec(
        r#"
        local eb = CreateFrame("EditBox", "TestBkspEB", UIParent)
        eb:SetText("hello")
        eb:SetCursorPosition(5)
        "#,
    )
    .unwrap();

    let eb_id = {
        let state = env.state();
        let sim = state.borrow();
        sim.widgets.get_id_by_name("TestBkspEB")
    };
    if let Some(id) = eb_id {
        env.state().borrow_mut().focused_frame_id = Some(id);
    }

    env.send_key_press("BACKSPACE", None).unwrap();

    let text: String = env
        .eval(r#"return TestBkspEB:GetText()"#)
        .unwrap_or_default();
    assert_eq!(text, "hell", "Backspace should remove the last character");
}

#[test]
fn focused_editbox_typing_replaces_selection_and_consumes_it_before_public_insert() {
    let env = env();
    env.exec(
        r#"
        local eb = CreateFrame("EditBox", "SelectedTypeEB", UIParent)
        eb:SetText("AB")
        eb:HighlightText(0, 1)
        eb:SetCursorPosition(1)
        eb:SetFocus()
        "#,
    )
    .unwrap();

    env.send_key_press("X", Some("X")).unwrap();
    let after_key: (String, i64, i64) = env
        .eval("return SelectedTypeEB:GetText(), SelectedTypeEB:GetCursorPosition(), SelectedTypeEB:GetUTF8CursorPosition()")
        .unwrap();
    assert_eq!(after_key, ("XB".into(), 1, 1));

    env.exec("SelectedTypeEB:Insert('Y')").unwrap();
    let after_insert: (String, i64, i64) = env
        .eval("return SelectedTypeEB:GetText(), SelectedTypeEB:GetCursorPosition(), SelectedTypeEB:GetUTF8CursorPosition()")
        .unwrap();
    assert_eq!(after_insert, ("XYB".into(), 2, 2));
}

#[test]
fn focused_editbox_typing_replaces_utf8_byte_selection() {
    let env = env();
    env.exec(
        r#"
        local eb = CreateFrame("EditBox", "SelectedUnicodeEB", UIParent)
        eb:SetText("aé猫z")
        eb:HighlightText(1, 6)
        eb:SetCursorPosition(6)
        eb:SetFocus()
        "#,
    )
    .unwrap();

    env.send_key_press("X", Some("界")).unwrap();
    let result: (String, i64, i64) = env
        .eval("return SelectedUnicodeEB:GetText(), SelectedUnicodeEB:GetCursorPosition(), SelectedUnicodeEB:GetUTF8CursorPosition()")
        .unwrap();
    assert_eq!(result, ("a界z".into(), 4, 2));
}

#[test]
fn focused_editbox_deletion_keys_remove_selected_range_and_consume_selection() {
    let mut results = Vec::new();
    for key in ["BACKSPACE", "DELETE"] {
        let env = env();
        env.exec(
            r#"
            local eb = CreateFrame("EditBox", "SelectedDeletionEB", UIParent)
            eb:SetText("aé猫z")
            eb:HighlightText(1, 6)
            eb:SetCursorPosition(6)
            eb:SetFocus()
            "#,
        )
        .unwrap();

        env.send_key_press(key, None).unwrap();
        let after_key: (String, i64, i64) = env
            .eval("return SelectedDeletionEB:GetText(), SelectedDeletionEB:GetCursorPosition(), SelectedDeletionEB:GetUTF8CursorPosition()")
            .unwrap();
        env.exec("SelectedDeletionEB:Insert('Q')").unwrap();
        let after_insert: (String, i64) = env
            .eval("return SelectedDeletionEB:GetText(), SelectedDeletionEB:GetCursorPosition()")
            .unwrap();
        results.push((key, after_key, after_insert));
    }
    assert_eq!(
        results,
        vec![
            ("BACKSPACE", ("az".into(), 1, 1), ("aQz".into(), 2)),
            ("DELETE", ("az".into(), 1, 1), ("aQz".into(), 2)),
        ]
    );
}

#[test]
fn selected_keyboard_edits_dispatch_callbacks_after_committing_text_and_caret() {
    let env = env();
    env.exec(
        r#"
        local eb = CreateFrame("EditBox", "SelectedCallbackEB", UIParent)
        eb:SetText("AB")
        eb:HighlightText(0, 1)
        eb:SetCursorPosition(1)
        _G.selected_edit_events = {}
        eb:SetScript("OnChar", function(self, char)
            table.insert(_G.selected_edit_events, table.concat({"char", char, self:GetText(), self:GetCursorPosition(), self:GetUTF8CursorPosition()}, ":"))
        end)
        eb:SetScript("OnTextChanged", function(self, userInput)
            table.insert(_G.selected_edit_events, table.concat({"changed", tostring(userInput), self:GetText(), self:GetCursorPosition(), self:GetUTF8CursorPosition()}, ":"))
        end)
        eb:SetFocus()
        "#,
    )
    .unwrap();

    env.send_key_press("X", Some("X")).unwrap();
    env.exec("SelectedCallbackEB:HighlightText(0, 1); SelectedCallbackEB:SetCursorPosition(1)")
        .unwrap();
    env.send_key_press("BACKSPACE", None).unwrap();
    let events: String = env
        .eval("return table.concat(_G.selected_edit_events, ';')")
        .unwrap();
    assert_eq!(
        events,
        "char:X:XB:1:1;changed:true:XB:1:1;changed:true:B:0:0"
    );
}

#[test]
fn rejected_numeric_key_preserves_selection_for_next_valid_key() {
    let env = env();
    env.exec(
        r#"
        local eb = CreateFrame("EditBox", "SelectedNumericEB", UIParent)
        eb:SetText("12")
        eb:HighlightText(0, 1)
        eb:SetCursorPosition(1)
        eb:SetNumeric(true)
        eb:SetFocus()
        "#,
    )
    .unwrap();

    env.send_key_press("X", Some("X")).unwrap();
    let rejected: (String, i64) = env
        .eval("return SelectedNumericEB:GetText(), SelectedNumericEB:GetCursorPosition()")
        .unwrap();
    assert_eq!(rejected, ("12".into(), 1));

    env.send_key_press("9", Some("9")).unwrap();
    let accepted: (String, i64) = env
        .eval("return SelectedNumericEB:GetText(), SelectedNumericEB:GetCursorPosition()")
        .unwrap();
    assert_eq!(accepted, ("92".into(), 1));
    env.exec("SelectedNumericEB:Insert('8')").unwrap();
    assert_eq!(
        env.eval::<String>("return SelectedNumericEB:GetText()")
            .unwrap(),
        "982"
    );
}

#[test]
fn editbox_cursor_keys_move_within_text_bounds() {
    let env = env();
    env.exec(
        r#"
        local eb = CreateFrame("EditBox", "TestCursorKeysEB", UIParent)
        eb:SetText("hello")
        eb:SetCursorPosition(2)
        eb:SetFocus()
        "#,
    )
    .unwrap();

    env.send_key_press("LEFT", None).unwrap();
    let left_pos: i64 = env
        .eval(r#"return TestCursorKeysEB:GetCursorPosition()"#)
        .unwrap();
    env.send_key_press("RIGHT", None).unwrap();
    let right_pos: i64 = env
        .eval(r#"return TestCursorKeysEB:GetCursorPosition()"#)
        .unwrap();
    env.send_key_press("END", None).unwrap();
    let end_pos: i64 = env
        .eval(r#"return TestCursorKeysEB:GetCursorPosition()"#)
        .unwrap();
    env.send_key_press("HOME", None).unwrap();
    let home_pos: i64 = env
        .eval(r#"return TestCursorKeysEB:GetCursorPosition()"#)
        .unwrap();

    assert_eq!(left_pos, 1);
    assert_eq!(right_pos, 2);
    assert_eq!(end_pos, 5);
    assert_eq!(home_pos, 0);
}

// ── OnKeyDown propagation ─────────────────────────────────────────────────────

#[test]
fn on_key_down_fires_on_focused_frame() {
    let env = env();
    env.exec(
        r#"
        _G.key_down = nil
        local f = CreateFrame("Frame", "TestKeyDownFrame", UIParent)
        f:EnableKeyboard(true)
        f:SetScript("OnKeyDown", function(self, key)
            _G.key_down = key
        end)
        "#,
    )
    .unwrap();

    let frame_id = {
        let state = env.state();
        let sim = state.borrow();
        sim.widgets.get_id_by_name("TestKeyDownFrame")
    };
    if let Some(id) = frame_id {
        env.state().borrow_mut().focused_frame_id = Some(id);
    }

    env.send_key_press("Q", None).unwrap();

    let key: Option<String> = env.eval("return _G.key_down").unwrap_or(None);
    assert_eq!(
        key.as_deref(),
        Some("Q"),
        "OnKeyDown should receive the key"
    );
}
