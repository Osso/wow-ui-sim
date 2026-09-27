use wow_ui_sim::lua_api::WowLuaEnv;

#[path = "editbox_stub_family/max_letters.rs"]
mod max_letters;

#[test]
fn editbox_set_text_clamps_internal_positions_before_callbacks_and_keyboard_edits() {
    let env = env();
    env.exec(r#"
        local eb = CreateFrame("EditBox", "SetTextBoundsEB", UIParent)
        eb:SetText("aé猫z")
        eb:SetCursorPosition(7)
        eb:HighlightText(3, 7)
        local observations = {}
        eb:SetScript("OnTextSet", function(self)
            observations[#observations + 1] = {"set", self:GetText(), self:GetCursorPosition(), self:GetUTF8CursorPosition()}
        end)
        eb:SetScript("OnTextChanged", function(self, userInput)
            observations[#observations + 1] = {"changed", self:GetText(), self:GetCursorPosition(), self:GetUTF8CursorPosition(), userInput}
        end)
        eb:SetText("éX")
        assert(#observations == 2)
        assert(observations[1][1] == "set" and observations[1][2] == "éX" and observations[1][3] == 3 and observations[1][4] == 2)
        assert(observations[2][1] == "changed" and observations[2][2] == "éX" and observations[2][3] == 3 and observations[2][4] == 2 and observations[2][5] == false)
        eb:Insert("Y")
        assert(eb:GetText() == "éXY", "shortened selection must not delete new text")
        eb:SetFocus()
    "#).unwrap();
    env.send_key_press("LEFT", None).unwrap();
    env.send_key_press("BACKSPACE", None).unwrap();
    let result: String = env
        .eval(r#"return SetTextBoundsEB:GetText() .. ":" .. SetTextBoundsEB:GetCursorPosition()"#)
        .unwrap();
    assert_eq!(result, "éY:2");
}

#[test]
fn editbox_set_text_preserves_in_bounds_caret_and_clips_selection() {
    let env = env();
    env.exec(
        r#"
        local eb = CreateFrame("EditBox", "SetTextClippedEB", UIParent)
        eb:SetText("é猫abc")
        eb:SetCursorPosition(2)
        eb:HighlightText(2, 8)
        eb:SetText("éZ")
        assert(eb:GetCursorPosition() == 2 and eb:GetUTF8CursorPosition() == 1)
    "#,
    )
    .unwrap();
    let state = env.state();
    let state = state.borrow();
    let id = state.widgets.get_id_by_name("SetTextClippedEB").unwrap();
    let frame = state.widgets.get(id).unwrap();
    assert_eq!(frame.editbox_cursor_pos, 1);
    assert_eq!(frame.editbox_highlight_range, Some((1, 2)));
}

#[test]
fn editbox_set_formatted_text_dispatches_hooks_and_routes_errors_without_reentry_loop() {
    env().exec(r#"
        local eb = CreateFrame("EditBox", nil, UIParent)
        local events, errors = {}, {}
        seterrorhandler(function(message) errors[#errors + 1] = tostring(message) end)
        eb:SetScript("OnChar", function() error("unexpected char") end)
        eb:SetScript("OnTextSet", function(self)
            events[#events + 1] = "set:" .. self:GetText()
            self:SetText(self:GetText())
        end)
        eb:HookScript("OnTextSet", function(self) events[#events + 1] = "set hook:" .. self:GetText() end)
        eb:SetScript("OnTextChanged", function(self, userInput)
            events[#events + 1] = "changed:" .. self:GetText() .. ":" .. tostring(userInput)
        end)
        eb:HookScript("OnTextChanged", function() events[#events + 1] = "changed hook" end)
        eb:SetFormattedText("%s%d", "é", 2)
        assert(eb:GetText() == "é2")
        assert(table.concat(events, ",") == "set:é2,set hook:é2,changed:é2:false,changed hook", table.concat(events, ","))
        eb:SetScript("OnTextSet", function() error("text set failure") end)
        eb:SetText("next")
        assert(eb:GetText() == "next")
        assert(events[#events - 1] == "changed:next:false" and events[#events] == "changed hook")
        assert(#errors == 1 and string.find(errors[1], "text set failure", 1, true))
    "#).unwrap();
}

#[test]
fn non_editbox_set_text_does_not_dispatch_editbox_callbacks() {
    env()
        .exec(
            r#"
        local widgets = {
            CreateFrame("Button", nil, UIParent),
            CreateFrame("GameTooltip", nil, UIParent),
            UIParent:CreateFontString(nil, "ARTWORK"),
        }
        for _, widget in ipairs(widgets) do
            widget:SetText("old")
            widget:SetText("new")
            assert(widget:GetText() == "new")
        end
    "#,
        )
        .unwrap();
}

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn editbox_insert_clamps_cursor_after_text_shortens() {
    env()
        .exec(
            r#"
            local eb = CreateFrame("EditBox", nil, UIParent)
            eb:SetText("abcd")
            eb:SetCursorPosition(4)
            eb:SetText("A")
            eb:Insert("X")
            assert(eb:GetText() == "AX")
            assert(eb:GetCursorPosition() == 2)
            assert(eb:GetUTF8CursorPosition() == 2, "cursor must follow the actual clamped insertion")

            eb:SetText("é猫A")
            eb:HighlightText(5, 6)
            eb:SetText("é")
            eb:Insert("X")
            assert(eb:GetText() == "éX")
            assert(eb:GetCursorPosition() == 3)
            assert(eb:GetUTF8CursorPosition() == 2, "stale selection must not leave the cursor past the text")
            eb:Insert("Y")
            assert(eb:GetText() == "éXY")
            assert(eb:GetUTF8CursorPosition() == 3)
            "#,
        )
        .unwrap();
}

#[test]
fn editbox_stub_family_methods_persist_runtime_state() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            local eb = CreateFrame("EditBox", "StubFamilyEB", UIParent)

            if eb:GetAltArrowKeyMode() ~= false then
                return "alt_arrow_should_default_false"
            end
            if eb:IsAlphabeticOnly() ~= false then
                return "alphabetic_only_should_default_false"
            end
            if eb:IsNumericFullRange() ~= false then
                return "numeric_full_range_should_default_false"
            end
            if eb:IsSecureText() ~= false then
                return "secure_text_should_default_false"
            end
            if eb:GetVisibleTextByteLimit() ~= 0 then
                return "visible_text_byte_limit_should_default_zero"
            end
            if eb:GetInputLanguage() ~= "ROMAN" then
                return "input_language_should_default_roman"
            end
            if eb:HasText() ~= false then
                return "has_text_should_default_false"
            end
            if eb:IsCountInvisibleLetters() ~= false then
                return "count_invisible_letters_should_default_false"
            end
            local r, g, b, a = eb:GetHighlightColor()
            if r ~= 1 or g ~= 1 or b ~= 1 or a ~= 1 then
                return "highlight_color_should_default_white"
            end

            eb:SetAltArrowKeyMode(true)
            eb:SetAlphabeticOnly(true)
            eb:SetNumericFullRange(true)
            eb:SetSecureText(true)
            eb:SetSecurityDisableSetText()
            eb:SetVisibleTextByteLimit(32)
            eb:SetSecurityDisablePaste()
            eb:SetCountInvisibleLetters(true)
            eb:SetHighlightColor(0.25, 0.5, 0.75, 0.9)

            eb:AddHistoryLine("one")
            eb:AddHistoryLine("two")
            if eb:GetHistoryLines() ~= 2 then
                return "history_should_have_two_lines"
            end
            eb:ClearHistory()
            if eb:GetHistoryLines() ~= 0 then
                return "clear_history_should_empty_history"
            end

            eb:ToggleInputLanguage()
            if eb:GetInputLanguage() ~= "NATIVE" then
                return "toggle_input_language_should_switch_to_native"
            end
            eb:ResetInputMode()

            if eb:GetAltArrowKeyMode() ~= true then
                return "alt_arrow_state_not_persisted"
            end
            if eb:IsAlphabeticOnly() ~= true then
                return "alphabetic_only_state_not_persisted"
            end
            if eb:IsNumericFullRange() ~= true then
                return "numeric_full_range_state_not_persisted"
            end
            if eb:IsSecureText() ~= true then
                return "secure_text_state_not_persisted"
            end
            eb:SetDesiredWidth(222)
            if eb:GetDesiredWidth() ~= 222 then
                return "desired_width_should_round_trip"
            end
            if eb:GetWidth() ~= 222 then
                return "desired_width_should_update_width"
            end
            if eb:GetVisibleTextByteLimit() ~= 32 then
                return "visible_text_byte_limit_not_persisted"
            end
            if eb:IsCountInvisibleLetters() ~= true then
                return "count_invisible_letters_not_persisted"
            end
            if eb:GetInputLanguage() ~= "ROMAN" then
                return "reset_input_mode_should_restore_roman"
            end

            eb:SetText("Visible text")
            if eb:HasText() ~= true then
                return "has_text_should_be_true_after_set_text"
            end
            eb:SetText("aé🙂b")
            eb:SetCursorPosition(2)
            if eb:GetUTF8CursorPosition() ~= 1 then
                return "utf8_cursor_position_should_track_character_offset_after_accent"
            end
            eb:SetCursorPosition(7)
            if eb:GetUTF8CursorPosition() ~= 3 then
                return "utf8_cursor_position_should_track_character_offset_after_emoji"
            end
            eb:SetText("Visible text")
            if eb:GetDisplayText() ~= "Visible text" then
                return "display_text_should_reflect_current_text"
            end
            local hr, hg, hb, ha = eb:GetHighlightColor()
            local function approx_eq(a, b)
                return math.abs(a - b) < 0.0001
            end
            if not approx_eq(hr, 0.25) or not approx_eq(hg, 0.5) or not approx_eq(hb, 0.75) or not approx_eq(ha, 0.9) then
                return "highlight_color_should_round_trip"
            end
            eb:HighlightText(2, 7)
            eb:ClearHighlightText()
            eb:HighlightText()

            return "ok"
            "#,
        )
        .unwrap();

    assert_eq!(
        result, "ok",
        "EditBox stub-family methods should persist real state instead of no-oping"
    );

    let frame_id = env
        .state()
        .borrow()
        .widgets
        .get_id_by_name("StubFamilyEB")
        .expect("StubFamilyEB should exist");
    let state = env.state().borrow();
    let frame = state
        .widgets
        .get(frame_id)
        .expect("StubFamilyEB frame should exist");
    assert!(
        frame.editbox_security_disable_set_text,
        "SetSecurityDisableSetText should persist the editbox set-text-disable flag"
    );
    assert!(
        frame.editbox_security_disable_paste,
        "SetSecurityDisablePaste should persist the editbox paste-disable flag"
    );
    assert_eq!(
        frame.editbox_highlight_color,
        wow_ui_sim::widget::Color::new(0.25, 0.5, 0.75, 0.9),
        "SetHighlightColor should persist the editbox highlight color"
    );
    assert_eq!(
        frame.editbox_highlight_range,
        Some((0, "Visible text".chars().count() as i32)),
        "HighlightText() without args should select the full current text"
    );
}

#[test]
fn editbox_public_cursor_offsets_bridge_utf8_boundaries() {
    let env = env();
    let result: String = env.eval(r#"
        local eb = CreateFrame("EditBox", nil, UIParent)
        eb:SetText("é猫A")
        eb:SetCursorPosition(2)
        if eb:GetCursorPosition() ~= 2 or eb:GetUTF8CursorPosition() ~= 1 then return "after accent" end
        eb:Insert("X")
        if eb:GetText() ~= "éX猫A" then return "insert placement" end
        if eb:GetCursorPosition() ~= 3 or eb:GetUTF8CursorPosition() ~= 2 then return "after insert" end
        eb:SetCursorPosition(6)
        if eb:GetCursorPosition() ~= 6 or eb:GetUTF8CursorPosition() ~= 3 then return "after ideograph" end
        eb:SetCursorPosition(100)
        if eb:GetCursorPosition() ~= 7 or eb:GetUTF8CursorPosition() ~= 4 then return "end clamp" end
        return "ok"
    "#).unwrap();
    assert_eq!(result, "ok");
}

#[test]
fn editbox_insert_replaces_or_deletes_ascii_selection() {
    let env = env();
    let result: String = env
        .eval(
            r#"
        local eb = CreateFrame("EditBox", nil, UIParent)
        eb:SetText("abcdef")
        eb:HighlightText(1, 4)
        eb:Insert("X")
        if eb:GetText() ~= "aXef" or eb:GetCursorPosition() ~= 2 then return "replace" end
        eb:HighlightText(1, 2)
        eb:Insert("")
        if eb:GetText() ~= "aef" or eb:GetCursorPosition() ~= 1 then return "delete" end
        eb:Insert("Z")
        if eb:GetText() ~= "aZef" then return "selection not cleared" end
        return "ok"
    "#,
        )
        .unwrap();
    assert_eq!(result, "ok");
}

#[test]
fn editbox_insert_replaces_valid_utf8_byte_selection() {
    let env = env();
    let result: String = env
        .eval(
            r#"
        local eb = CreateFrame("EditBox", nil, UIParent)
        eb:SetText("é猫A")
        eb:HighlightText(2, 5)
        eb:Insert("界")
        if eb:GetText() ~= "é界A" then return "replacement" end
        if eb:GetCursorPosition() ~= 5 or eb:GetUTF8CursorPosition() ~= 2 then return "cursor" end
        eb:HighlightText(0, 2)
        eb:Insert("")
        if eb:GetText() ~= "界A" or eb:GetCursorPosition() ~= 0 then return "deletion" end
        return "ok"
    "#,
        )
        .unwrap();
    assert_eq!(result, "ok");
}

#[test]
fn editbox_ime_composition_getter_reflects_runtime_state() {
    let env = env();
    env.eval::<()>(r#"CreateFrame("EditBox", "StubFamilyIME", UIParent)"#)
        .unwrap();

    let frame_id = env
        .state()
        .borrow()
        .widgets
        .get_id_by_name("StubFamilyIME")
        .expect("StubFamilyIME should exist");

    {
        let state_rc = env.state();
        let mut state = state_rc.borrow_mut();
        let frame = state
            .widgets
            .get_mut_visual(frame_id)
            .expect("StubFamilyIME frame should exist");
        frame.editbox_in_ime_composition_mode = true;
    }

    let in_ime: bool = env
        .eval(r#"return StubFamilyIME:IsInIMECompositionMode()"#)
        .unwrap();
    assert!(
        in_ime,
        "IsInIMECompositionMode should reflect the backing editbox runtime state"
    );
}

#[test]
fn editbox_insert_refreshes_stripped_render_text() {
    let env = env();
    env.eval::<()>(
        r#"
        local eb = CreateFrame("EditBox", "StubFamilyInsertedText", UIParent)
        eb:SetText("")
        eb:Insert("Visible")
        "#,
    )
    .unwrap();

    let frame_id = env
        .state()
        .borrow()
        .widgets
        .get_id_by_name("StubFamilyInsertedText")
        .expect("StubFamilyInsertedText should exist");
    let state = env.state().borrow();
    let frame = state
        .widgets
        .get(frame_id)
        .expect("StubFamilyInsertedText frame should exist");

    assert_eq!(frame.text.as_deref(), Some("Visible"));
    assert_eq!(
        frame.text_stripped.as_deref(),
        Some("Visible"),
        "Insert should keep renderer-facing stripped text in sync with typed text"
    );
}
