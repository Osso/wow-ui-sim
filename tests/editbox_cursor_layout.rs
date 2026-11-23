//! Shared clean-room cursor geometry contract, not native WoW geometry.
//! Uses installed simulator fonts through the existing backend and aliases.
//! No manual callback dispatch, synthetic geometry injection, or new geometry API.

use std::cell::RefCell;
use std::rc::Rc;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::render::font::{DEFAULT_WOW_FONT, WowFontSystem};

const TICK_SECONDS: f64 = 0.016;

#[test]
fn headless_cursor_uses_real_font_geometry_without_explicit_injection() {
    let env = crate::common::new_headless_cursor_env();
    env.exec(CREATE_FIXTURE)
        .expect("create existing cursor fixture");
    env.fire_on_update(TICK_SECONDS).unwrap();
    env.exec(
        r#"
        assert(#CursorRecords == 1, 'first shaped cursor must be delivered')
        local first = CursorRecords[1]
        assert(first.x == 5 and first.y < 0 and first.height > 0)
        CursorBox:SetCursorPosition(2)
        "#,
    )
    .expect("first headless cursor delivery");
    env.fire_on_update(TICK_SECONDS).unwrap();
    env.exec(
        r#"
        assert(#CursorRecords == 2, 'cursor movement must deliver new geometry')
        assert(CursorRecords[2].x > CursorRecords[1].x)
        "#,
    )
    .expect("headless cursor must use shaped glyph advances");
}

fn create_env_with_installed_fonts() -> WowLuaEnv {
    let fonts = WowFontSystem::new_without_casc();
    assert!(
        fonts.font_system.db().faces().next().is_some(),
        "SETUP FAILURE: installed simulator font database has no face; not behavioral RED"
    );
    assert!(
        fonts.family_name(Some(DEFAULT_WOW_FONT)).is_some(),
        "SETUP FAILURE: existing default simulator font alias is unavailable"
    );
    let env = WowLuaEnv::new().expect("SETUP FAILURE: create real Lua environment");
    env.set_font_system(Rc::new(RefCell::new(fonts)));
    env.exec(CREATE_FIXTURE)
        .expect("SETUP FAILURE: create public EditBox and FontString fixtures");
    env
}

const CREATE_FIXTURE: &str = r#"
    CursorBox = CreateFrame('EditBox', 'C2CursorLayoutBox', UIParent)
    CursorBox:SetAutoFocus(false)
    CursorBox:SetMultiLine(true)
    CursorBox:SetSize(400, 120)
    CursorBox:SetPoint('TOPLEFT', UIParent, 'TOPLEFT', 113, -79)
    assert(CursorBox:SetFont('Fonts\\FRIZQT__.TTF', 20))
    CursorBox:SetJustifyH('LEFT')
    CursorBox:SetJustifyV('TOP')
    CursorBox:SetTextInsets(5, 7, 3, 4)
    CursorBox:SetText('é\n猫z')
    CursorBox:SetCursorPosition(0)
    CursorBox:Show()

    local parent = CreateFrame('Frame', nil, UIParent)
    CursorEWidth = parent:CreateFontString(nil, 'ARTWORK')
    CursorCatWidth = parent:CreateFontString(nil, 'ARTWORK')
    assert(CursorEWidth:SetFont('Fonts\\FRIZQT__.TTF', 20))
    assert(CursorCatWidth:SetFont('Fonts\\FRIZQT__.TTF', 20))
    CursorEWidth:SetText('é')
    CursorCatWidth:SetText('猫')

    CursorRecords, CursorUpdateObservations = {}, {}
    CursorBox:SetScript('OnCursorChanged', function(self, ...)
        local x, y, width, height = ...
        CursorRecords[#CursorRecords + 1] = {
            x = x, y = y, width = width, height = height,
            arity = select('#', ...),
            byte = self:GetCursorPosition(),
            scalar = self:GetUTF8CursorPosition(), text = self:GetText(),
        }
    end)
    -- Small ordinary OnUpdate observer, NOT a cached ScrollingEdit consumer.
    CursorBox:SetScript('OnUpdate', function(self)
        CursorUpdateObservations[#CursorUpdateObservations + 1] = {
            record = CursorRecords[#CursorRecords], count = #CursorRecords,
            byte = self:GetCursorPosition(),
            scalar = self:GetUTF8CursorPosition(), text = self:GetText(),
        }
    end)

    function AssertCursorPending(count, byte, scalar)
        assert(#CursorRecords == count, 'cursor callback must wait for layout tick')
        assert(CursorBox:GetCursorPosition() == byte, 'public byte cursor committed')
        assert(CursorBox:GetUTF8CursorPosition() == scalar, 'public scalar cursor committed')
        assert(CursorBox:GetText() == 'é\n猫z')
    end

    function AssertCursorDelivery(count, byte, scalar, x, y)
        assert(#CursorRecords == count, 'exactly one delivery per changed tick')
        local record = CursorRecords[count]
        assert(record ~= nil and record.arity == 4, 'four geometry arguments')
        local function near(actual, expected, tolerance)
            assert(type(actual) == 'number', 'numeric geometry required')
            assert(math.abs(actual - expected) < (tolerance or 0.001),
                tostring(actual) .. ' != ' .. tostring(expected))
        end
        -- Library Edit::cursor_position returns integer logical coordinates;
        -- FontString advances can be fractional. Line starts stay exact.
        if x > 5 then
            near(record.x, x, 1)
            assert(record.x > 5, 'caret advances past the shaped glyph')
        else
            near(record.x, x)
        end
        near(record.y, y)
        near(record.width, 1.5)
        near(record.height, 24)
        assert(record.byte == byte and record.scalar == scalar)
        assert(record.text == 'é\n猫z', 'callback sees committed text')
        local observation = CursorUpdateObservations[#CursorUpdateObservations]
        assert(observation ~= nil, 'ordinary OnUpdate actually ran')
        assert(observation.count == count and observation.record == record,
            'cursor delivery must precede ordinary OnUpdate in this tick')
        assert(observation.byte == byte and observation.scalar == scalar)
        assert(observation.text == record.text)
        AssertCursorPending(count, byte, scalar)
    end
"#;

#[test]
fn programmatic_cursor_layout_delivers_before_onupdate_and_suppresses_unchanged_ticks() {
    let env = create_env_with_installed_fonts();
    env.exec("AssertCursorPending(0, 0, 0); assert(#CursorUpdateObservations == 0)")
        .unwrap();

    // First actual layout delivery must already be visible to the first OnUpdate.
    env.fire_on_update(TICK_SECONDS).unwrap();
    env.exec("AssertCursorDelivery(1, 0, 0, 5, -3); assert(#CursorUpdateObservations == 1)")
        .unwrap();

    env.exec("CursorBox:SetCursorPosition(3); AssertCursorPending(1, 3, 2)")
        .unwrap();
    env.fire_on_update(TICK_SECONDS).unwrap();
    // Proposed policy: local x = 5 + shaped line-start x (0);
    // local y = -(3 + shaped second-line top (24)). No glyph advance oracle needed.
    env.exec("AssertCursorDelivery(2, 3, 2, 5, -27); assert(#CursorUpdateObservations == 2)")
        .unwrap();

    env.exec("CursorBox:SetCursorPosition(3); AssertCursorPending(2, 3, 2)")
        .unwrap();
    for _ in 0..2 {
        env.fire_on_update(TICK_SECONDS).unwrap();
        env.exec("AssertCursorDelivery(2, 3, 2, 5, -27)").unwrap();
    }
    env.exec("assert(#CursorUpdateObservations == 4)").unwrap();
}

#[test]
fn programmatic_text_lifecycle_dispatches_changed_at_host_return_before_cursor_layout() {
    for assignment in [
        "CursorBox:SetText('x\\né猫')",
        "CursorBox:SetFormattedText('%s\\n%s', 'x', 'é猫')",
    ] {
        let env = create_env_with_installed_fonts();
        env.exec("CursorBox:SetCursorPosition(3)").unwrap();
        env.fire_on_update(TICK_SECONDS).unwrap();
        assert!(
            env.state().borrow().lua_errors.is_empty(),
            "setup tick recorded Lua errors: {:?}",
            env.state().borrow().lua_errors
        );
        env.exec(
            r#"
            CursorRecords, CursorUpdateObservations, LifecycleTrace = {}, {}, {}
            LifecycleErrors = {}
            seterrorhandler(function(message)
                LifecycleErrors[#LifecycleErrors + 1] = tostring(message)
            end)

            local function assert_current_cursor(self)
                assert(self:GetText() == 'x\né猫', 'callback sees current text')
                assert(self:GetCursorPosition() == 2, 'callback sees byte cursor')
                assert(self:GetUTF8CursorPosition() == 2, 'callback sees scalar cursor')
                assert(#CursorRecords == 1, 'one actual cursor delivery')
                local record = CursorRecords[1]
                assert(record.arity == 4, 'four geometry arguments')
                assert(record.text == self:GetText())
                assert(record.byte == 2 and record.scalar == 2)
                local function near(actual, expected)
                    assert(type(actual) == 'number', 'numeric geometry required')
                    assert(math.abs(actual - expected) < 0.001,
                        tostring(actual) .. ' != ' .. tostring(expected))
                end
                near(record.x, 5)
                near(record.y, -27)
                near(record.width, 1.5)
                near(record.height, 24)
            end

            CursorBox:SetScript('OnTextSet', function(self)
                LifecycleTrace[#LifecycleTrace + 1] = 'set'
                assert(self:GetText() == 'x\né猫', 'OnTextSet sees committed text')
                assert(#LifecycleTrace == 1, 'OnTextSet is synchronous and nonreentrant')
                assert(#CursorRecords == 0, 'OnTextSet does not force layout')
            end)
            CursorBox:HookScript('OnCursorChanged', function(self)
                LifecycleTrace[#LifecycleTrace + 1] = 'cursor'
                assert_current_cursor(self)
                assert(table.concat(LifecycleTrace, ',') == 'set,returned,changed,cursor')
            end)
            -- Public lifecycle observer, NOT a genuine cached ScrollingEdit consumer.
            CursorBox:SetScript('OnTextChanged', function(self, userInput)
                LifecycleTrace[#LifecycleTrace + 1] = 'changed'
                assert(self:GetText() == 'x\né猫', 'callback sees current text')
                assert(self:GetCursorPosition() == 2, 'callback sees byte cursor')
                assert(self:GetUTF8CursorPosition() == 2, 'callback sees scalar cursor')
                assert(#CursorRecords == 0, 'text notification does not force cursor presentation')
                assert(userInput == false, 'programmatic assignment is not user input')
                assert(table.concat(LifecycleTrace, ',') == 'set,returned,changed')
            end)
            CursorBox:HookScript('OnUpdate', function(self)
                LifecycleTrace[#LifecycleTrace + 1] = 'update'
                assert_current_cursor(self)
                assert(table.concat(LifecycleTrace, ',') == 'set,returned,changed,cursor,update')
                assert(#CursorUpdateObservations == 1, 'existing OnUpdate observer preserved')
                local observation = CursorUpdateObservations[1]
                assert(observation.record == CursorRecords[1] and observation.count == 1)
                assert(observation.text == self:GetText())
                assert(observation.byte == 2 and observation.scalar == 2)
            end)
            "#,
        )
        .unwrap();
        let assignment_result = env.exec(&format!(
            r#"
            {assignment}
            assert(CursorBox:GetText() == 'x\né猫', 'text commits before method returns')
            assert(table.concat(LifecycleTrace, ',') == 'set', 'only OnTextSet before return')
            LifecycleTrace[#LifecycleTrace + 1] = 'returned'
            assert(CursorBox:GetCursorPosition() == 2, 'logical byte cursor commits immediately')
            assert(CursorBox:GetUTF8CursorPosition() == 2, 'logical scalar cursor commits immediately')
            assert(#CursorRecords == 0, 'text assignment must not force cursor delivery')
            assert(#CursorUpdateObservations == 0, 'no update before actual tick')
            assert(table.concat(LifecycleTrace, ',') == 'set,returned', 'no immediate notification')
            "#
        ));
        env.exec("assert(#LifecycleErrors == 0, table.concat(LifecycleErrors, '\\n'))")
            .unwrap();
        assert!(
            env.state().borrow().lua_errors.is_empty(),
            "{assignment}: assignment recorded Lua errors: {:?}",
            env.state().borrow().lua_errors
        );
        assignment_result.unwrap();
        env.exec(
            r#"
            assert(table.concat(LifecycleTrace, ',') == 'set,returned,changed',
                table.concat(LifecycleTrace, ','))
            assert(#CursorRecords == 0 and #CursorUpdateObservations == 0)
            "#,
        )
        .unwrap();

        let tick_result = env.fire_on_update(TICK_SECONDS);
        env.exec("assert(#LifecycleErrors == 0, table.concat(LifecycleErrors, '\\n'))")
            .unwrap();
        assert!(
            env.state().borrow().lua_errors.is_empty(),
            "{assignment}: lifecycle tick recorded Lua errors: {:?}",
            env.state().borrow().lua_errors
        );
        tick_result.unwrap();
        env.exec(
            r#"
            assert(table.concat(LifecycleTrace, ',') == 'set,returned,changed,cursor,update',
                table.concat(LifecycleTrace, ','))
            assert(#CursorRecords == 1 and #CursorUpdateObservations == 1)
            assert(CursorBox:GetText() == 'x\né猫')
            assert(CursorBox:GetCursorPosition() == 2 and CursorBox:GetUTF8CursorPosition() == 2)
            "#,
        )
        .unwrap();
    }
}

#[test]
fn keyboard_cursor_layout_crosses_multibyte_and_newline_boundaries_on_real_ticks() {
    let env = create_env_with_installed_fonts();
    env.exec("CursorBox:SetFocus(); AssertCursorPending(0, 0, 0)")
        .unwrap();
    env.fire_on_update(TICK_SECONDS).unwrap();
    env.exec("AssertCursorDelivery(1, 0, 0, 5, -3); assert(#CursorUpdateObservations == 1)")
        .unwrap();

    // Whole-document UTF-8 boundaries: 0 --é--> 2 --newline--> 3 --猫--> 6.
    // Glyph-dependent x uses public FontString measurement with the same backend.
    let movements = [
        ("RIGHT", 2, 1, "5 + CursorEWidth:GetStringWidth()", -3),
        ("RIGHT", 3, 2, "5", -27),
        ("RIGHT", 6, 3, "5 + CursorCatWidth:GetStringWidth()", -27),
        ("LEFT", 3, 2, "5", -27),
        ("LEFT", 2, 1, "5 + CursorEWidth:GetStringWidth()", -3),
        ("LEFT", 0, 0, "5", -3),
    ];
    for (index, (key, byte, scalar, x, y)) in movements.iter().enumerate() {
        let previous_count = index + 1;
        let delivered_count = previous_count + 1;
        env.send_key_press(key, None).unwrap();
        env.exec(&format!(
            "AssertCursorPending({previous_count}, {byte}, {scalar})"
        ))
        .unwrap();
        env.fire_on_update(TICK_SECONDS).unwrap();
        env.exec(&format!(
            "AssertCursorDelivery({delivered_count}, {byte}, {scalar}, {x}, {y}); \
             assert(#CursorUpdateObservations == {delivered_count})"
        ))
        .unwrap();
    }

    for _ in 0..2 {
        env.fire_on_update(TICK_SECONDS).unwrap();
        env.exec("AssertCursorDelivery(7, 0, 0, 5, -3)").unwrap();
    }
    env.exec("assert(#CursorUpdateObservations == 9)").unwrap();
}

// Append after the existing cursor-layout tests. This is a public lifecycle
// observer, not a real ScrollingEdit consumer or a native WoW geometry oracle.
const DIRTY_FLUSH_OBSERVERS: &str = r#"
    CursorRecords, CursorUpdateObservations = {}, {}
    DirtyTrace, DirtyErrors = {}, {}
    seterrorhandler(function(message)
        DirtyErrors[#DirtyErrors + 1] = tostring(message)
    end)

    function RecordDirtyEntry(kind, self, value, arity)
        local entry = { kind = kind, value = value, arity = arity,
            text = self:GetText(), byte = self:GetCursorPosition(),
            scalar = self:GetUTF8CursorPosition(), cursorCount = #CursorRecords }
        DirtyTrace[#DirtyTrace + 1] = entry
        return entry
    end

    CursorBox:SetScript('OnTextSet', function(self, ...)
        RecordDirtyEntry('set', self, nil, select('#', ...))
    end)
    CursorBox:SetScript('OnChar', function(self, ...)
        local entry = RecordDirtyEntry('char', self, ..., select('#', ...))
        assert(entry.arity == 1, 'existing keyboard payload: one scalar string')
        assert(type(entry.value) == 'string' and entry.value ~= '', 'nonempty OnChar')
    end)
    CursorBox:SetScript('OnKeyDown', function(self, key, ...)
        RecordDirtyEntry('key', self, key, 1 + select('#', ...))
    end)
    -- Preserve the fixture's actual geometry observer; never manufacture records.
    CursorBox:HookScript('OnCursorChanged', function(self, ...)
        local entry = RecordDirtyEntry('cursor', self, nil, select('#', ...))
        assert(entry.arity == 4, 'actual four-argument presentation callback')
    end)
    CursorBox:SetScript('OnTextChanged', function(self, ...)
        local entry = RecordDirtyEntry('changed', self, ..., select('#', ...))
        assert(entry.arity == 1 and type(entry.value) == 'boolean')
        assert(entry.text == self:GetText(), 'notification sees current text')
        assert(entry.byte == self:GetCursorPosition(), 'notification sees byte cursor')
        assert(entry.scalar == self:GetUTF8CursorPosition(),
            'notification sees scalar cursor')
    end)

    function AssertDirtyTrace(expected)
        local labels = {}
        for _, entry in ipairs(DirtyTrace) do
            if entry.kind == 'set' then
                labels[#labels + 1] = 'set:' .. entry.text
            elseif entry.kind == 'char' or entry.kind == 'key'
                or entry.kind == 'changed' then
                labels[#labels + 1] = entry.kind .. ':' .. tostring(entry.value)
                    .. ':' .. entry.text
            end
        end
        local actual = table.concat(labels, '|')
        assert(actual == expected, 'trace: ' .. actual .. ' expected: ' .. expected)
    end

    function AssertDirtyText(text, byte, scalar)
        assert(CursorBox:GetText() == text, 'public current text')
        assert(CursorBox:GetCursorPosition() == byte, 'public byte cursor')
        assert(CursorBox:GetUTF8CursorPosition() == scalar, 'public scalar cursor')
    end
"#;

fn assert_dirty_flush_errors_empty(env: &WowLuaEnv) {
    env.exec("assert(#DirtyErrors == 0, table.concat(DirtyErrors, '\\n'))")
        .unwrap();
    assert!(
        env.state().borrow().lua_errors.is_empty(),
        "dirty-flush callbacks recorded simulator errors: {:?}",
        env.state().borrow().lua_errors
    );
}

fn exec_dirty_flush_lua(env: &WowLuaEnv, script: &str) {
    let result = env.exec(script);
    assert_dirty_flush_errors_empty(env);
    result.unwrap();
}

fn tick_dirty_flush(env: &WowLuaEnv) {
    let result = env.fire_on_update(TICK_SECONDS);
    assert_dirty_flush_errors_empty(env);
    result.unwrap();
}

fn create_dirty_flush_env() -> WowLuaEnv {
    let env = create_env_with_installed_fonts();
    env.exec("CursorBox:SetFocus(); AssertCursorPending(0, 0, 0)")
        .unwrap();
    env.fire_on_update(TICK_SECONDS).unwrap();
    assert!(
        env.state().borrow().lua_errors.is_empty(),
        "setup tick recorded Lua errors: {:?}",
        env.state().borrow().lua_errors
    );
    // Reset only after the fixture's real initial tick. Preserve its Unicode
    // text, cursor, fonts, geometry and both original observers.
    env.exec(DIRTY_FLUSH_OBSERVERS).unwrap();
    assert_dirty_flush_errors_empty(&env);
    env
}

fn assert_dirty_flush_stable(env: &WowLuaEnv, expected: &str) {
    exec_dirty_flush_lua(env, &format!("AssertDirtyTrace('{expected}')"));
    tick_dirty_flush(env);
    exec_dirty_flush_lua(env, &format!("AssertDirtyTrace('{expected}')"));
}

#[test]
fn dirty_flush_insert_then_changed_settext_is_one_programmatic_change() {
    let env = create_dirty_flush_env();
    exec_dirty_flush_lua(
        &env,
        r#"
        CursorBox:HighlightText(0)
        CursorBox:Insert('a')
        AssertDirtyTrace('char:a:a')
        CursorBox:SetText('b')
        AssertDirtyTrace('char:a:a|set:b')
        AssertDirtyText('b', 1, 1)
        "#,
    );
    tick_dirty_flush(&env);
    assert_dirty_flush_stable(&env, "char:a:a|set:b|changed:false:b");
}

#[test]
fn dirty_flush_changed_settext_then_insert_is_one_user_change() {
    let env = create_dirty_flush_env();
    exec_dirty_flush_lua(
        &env,
        r#"
        CursorBox:SetText('a')
        AssertDirtyTrace('set:a')
        CursorBox:SetCursorPosition(1)
        CursorBox:Insert('b')
        AssertDirtyTrace('set:a|char:b:ab')
        AssertDirtyText('ab', 2, 2)
        "#,
    );
    tick_dirty_flush(&env);
    assert_dirty_flush_stable(&env, "set:a|char:b:ab|changed:true:ab");
}

#[test]
fn dirty_flush_same_value_settext_preserves_insert_origin_without_callbacks() {
    let env = create_dirty_flush_env();
    exec_dirty_flush_lua(
        &env,
        r#"
        CursorBox:HighlightText(0)
        CursorBox:Insert('a')
        AssertDirtyTrace('char:a:a')
        CursorBox:SetText('a')
        AssertDirtyTrace('char:a:a')
        AssertDirtyText('a', 1, 1)
        "#,
    );
    tick_dirty_flush(&env);
    assert_dirty_flush_stable(&env, "char:a:a|changed:true:a");
    // Once clean, a same-value assignment must not schedule a new dirty flush.
    exec_dirty_flush_lua(&env, "CursorBox:SetText('a')");
    assert_dirty_flush_stable(&env, "char:a:a|changed:true:a");
}

#[test]
fn dirty_flush_two_unicode_inserts_emit_synchronous_scalars_and_one_change() {
    let env = create_dirty_flush_env();
    exec_dirty_flush_lua(
        &env,
        r#"
        CursorBox:HighlightText(0)
        CursorBox:Insert('é')
        AssertDirtyTrace('char:é:é')
        AssertDirtyText('é', 2, 1)
        CursorBox:Insert('猫')
        AssertDirtyTrace('char:é:é|char:猫:é猫')
        AssertDirtyText('é猫', 5, 2)
        "#,
    );
    tick_dirty_flush(&env);
    assert_dirty_flush_stable(&env, "char:é:é|char:猫:é猫|changed:true:é猫");
}

#[test]
fn dirty_flush_scripted_full_deletion_preserves_true_and_emits_no_empty_char() {
    let env = create_dirty_flush_env();
    exec_dirty_flush_lua(
        &env,
        r#"
        CursorBox:HighlightText(0)
        CursorBox:Insert('a')
        AssertDirtyTrace('char:a:a')
        CursorBox:HighlightText(0)
        CursorBox:Insert('')
        AssertDirtyTrace('char:a:a')
        AssertDirtyText('', 0, 0)
        "#,
    );
    tick_dirty_flush(&env);
    assert_dirty_flush_stable(&env, "char:a:a|changed:true:");
}

#[test]
fn dirty_flush_scripted_partial_deletion_preserves_false() {
    let env = create_dirty_flush_env();
    exec_dirty_flush_lua(
        &env,
        r#"
        CursorBox:SetText('ab')
        AssertDirtyTrace('set:ab')
        CursorBox:HighlightText(0, 1)
        CursorBox:Insert('')
        AssertDirtyTrace('set:ab')
        AssertDirtyText('b', 0, 0)
        "#,
    );
    tick_dirty_flush(&env);
    assert_dirty_flush_stable(&env, "set:ab|changed:false:b");
}

#[test]
fn dirty_flush_keydown_presents_pending_text_before_keydown_and_char() {
    let env = create_dirty_flush_env();
    exec_dirty_flush_lua(
        &env,
        r#"
        CursorBox:SetText('ab')
        CursorBox:SetCursorPosition(2)
        AssertDirtyTrace('set:ab')
        AssertDirtyText('ab', 2, 2)
        "#,
    );
    // No intervening tick: the key-down boundary itself must present and flush ab.
    let result = env.send_key_down("X", Some("x"));
    assert_dirty_flush_errors_empty(&env);
    result.unwrap();
    env.send_key_up("X");
    exec_dirty_flush_lua(
        &env,
        r#"
        AssertDirtyTrace('set:ab|changed:false:ab|key:X:ab|char:x:abx|changed:true:abx')
        AssertDirtyText('abx', 3, 3)
        "#,
    );
    tick_dirty_flush(&env);
    assert_dirty_flush_stable(
        &env,
        "set:ab|changed:false:ab|key:X:ab|char:x:abx|changed:true:abx",
    );
}

#[test]
fn dirty_flush_settext_inside_single_keyboard_onchar_sets_programmatic_origin() {
    let env = create_dirty_flush_env();
    exec_dirty_flush_lua(
        &env,
        r#"
        CursorBox:HighlightText(0)
        CursorBox:HookScript('OnChar', function(self, char)
            -- Base observer records callback entry before any protected assertion.
            assert(char == 'x')
            self:SetText('b')
        end)
        "#,
    );
    // Exactly one key; a second key could hide the bug with a pre-key flush.
    let result = env.send_key_press("X", Some("x"));
    assert_dirty_flush_errors_empty(&env);
    result.unwrap();
    exec_dirty_flush_lua(
        &env,
        r#"
        AssertDirtyTrace('key:X:é\n猫z|char:x:x|set:b|changed:false:b')
        AssertDirtyText('b', 1, 1)
        "#,
    );
    tick_dirty_flush(&env);
    assert_dirty_flush_stable(&env, "key:X:é\\n猫z|char:x:x|set:b|changed:false:b");
}

#[test]
fn dirty_flush_settext_inside_keydown_then_keyboard_insert_sets_user_origin() {
    let env = create_dirty_flush_env();
    exec_dirty_flush_lua(
        &env,
        r#"
        CursorBox:HookScript('OnKeyDown', function(self, key)
            -- Base observer records callback entry before any protected assertion.
            assert(key == 'X')
            self:SetText('ab')
            self:SetCursorPosition(2)
        end)
        "#,
    );
    let result = env.send_key_press("X", Some("x"));
    assert_dirty_flush_errors_empty(&env);
    result.unwrap();
    exec_dirty_flush_lua(
        &env,
        r#"
        AssertDirtyTrace('key:X:é\n猫z|set:ab|changed:false:ab|char:x:abx|changed:true:abx')
        AssertDirtyText('abx', 3, 3)
        "#,
    );
    tick_dirty_flush(&env);
    assert_dirty_flush_stable(&env, "key:X:é\\n猫z|set:ab|char:x:abx|changed:true:abx");
}

#[cfg(feature = "gui")]
mod visible_text_publication {
    use super::*;
    use wow_ui_sim::iced_app::{RegistryQuadBatchParams, build_quad_batch_for_registry};
    use wow_ui_sim::render::glyph::GlyphAtlas;

    fn render_glyph_positions(
        env: &WowLuaEnv,
        fonts: &Rc<RefCell<WowFontSystem>>,
        atlas: &mut GlyphAtlas,
        root: &str,
    ) -> Vec<[f32; 2]> {
        let buckets = {
            let mut state = env.state().borrow_mut();
            state.ensure_layout_rects();
            let _ = state.get_strata_buckets();
            state.strata_buckets.as_ref().unwrap().clone()
        };
        let state = env.state().borrow();
        let mut fonts = fonts.borrow_mut();
        let batch = build_quad_batch_for_registry(
            RegistryQuadBatchParams::new(&state.widgets, (1024.0, 768.0), &buckets)
                .root_name(Some(root))
                .text_ctx(Some((&mut fonts, atlas))),
        );
        batch
            .vertices
            .iter()
            .filter(|vertex| vertex.tex_index == wow_ui_sim::render::shader::GLYPH_ATLAS_TEX_INDEX)
            .map(|vertex| vertex.position)
            .collect()
    }

    fn normalize_glyph_positions(positions: &[[f32; 2]]) -> Vec<[f32; 2]> {
        assert!(
            !positions.is_empty(),
            "expected actual emitted glyph vertices"
        );
        assert!(
            positions
                .iter()
                .flatten()
                .all(|coordinate| coordinate.is_finite())
        );
        let min_x = positions.iter().map(|p| p[0]).fold(f32::INFINITY, f32::min);
        let min_y = positions.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min);
        positions
            .iter()
            .map(|p| [p[0] - min_x, p[1] - min_y])
            .collect()
    }

    #[test]
    fn changed_settext_keeps_emitted_glyphs_until_tick_then_matches_fontstring() {
        let env = create_env_with_installed_fonts();
        // Replace the fixture's font handle before any tick or batch; retain and
        // use this exact shared system for both environment layout and rendering.
        let fonts = Rc::new(RefCell::new(WowFontSystem::new_without_casc()));
        assert!(fonts.borrow().font_system.db().faces().next().is_some());
        assert!(fonts.borrow().family_name(Some(DEFAULT_WOW_FONT)).is_some());
        env.set_font_system(Rc::clone(&fonts));
        env.set_screen_size(1024.0, 768.0);
        env.exec(
            r#"
            CursorGlyphReferenceParent = CreateFrame('Frame', 'C2GlyphReference', UIParent)
            CursorGlyphReferenceParent:SetSize(400, 120)
            CursorGlyphReferenceParent:SetPoint('TOPLEFT', UIParent, 'TOPLEFT', 113, -79)
            CursorGlyphReferenceParent:Show()
            local reference = CursorGlyphReferenceParent:CreateFontString(nil, 'ARTWORK')
            reference:SetPoint('TOPLEFT', CursorGlyphReferenceParent, 'TOPLEFT', 5, -3)
            reference:SetSize(388, 113)
            assert(reference:SetFont('Fonts\\FRIZQT__.TTF', 20))
            reference:SetJustifyH('LEFT')
            reference:SetJustifyV('TOP')
            reference:SetText('WWWWWW')
            reference:Show()
            assert(CursorBox:GetText() == 'é\n猫z')
            "#,
        )
        .unwrap();
        let mut atlas = GlyphAtlas::new();
        env.fire_on_update(TICK_SECONDS).unwrap();
        assert!(
            env.state().borrow().lua_errors.is_empty(),
            "setup tick Lua errors"
        );
        let before = render_glyph_positions(&env, &fonts, &mut atlas, "C2CursorLayoutBox");
        let expected = render_glyph_positions(&env, &fonts, &mut atlas, "C2GlyphReference");
        let expected_normalized = normalize_glyph_positions(&expected);
        assert_ne!(
            normalize_glyph_positions(&before),
            expected_normalized,
            "initial multiline Unicode glyphs must distinguish the replacement"
        );

        env.exec("CursorBox:SetText('WWWWWW'); assert(CursorBox:GetText() == 'WWWWWW')")
            .unwrap();
        // Building a real batch must not publish the newly committed logical text.
        let pending = render_glyph_positions(&env, &fonts, &mut atlas, "C2CursorLayoutBox");
        assert_eq!(
            pending, before,
            "pre-tick emitted glyph positions must remain old"
        );

        env.fire_on_update(TICK_SECONDS).unwrap();
        assert!(
            env.state().borrow().lua_errors.is_empty(),
            "flush tick Lua errors"
        );
        let flushed = render_glyph_positions(&env, &fonts, &mut atlas, "C2CursorLayoutBox");
        assert_ne!(
            flushed, before,
            "actual tick must publish changed visible glyphs"
        );
        let flushed_normalized = normalize_glyph_positions(&flushed);
        assert_eq!(
            flushed_normalized.len(),
            expected_normalized.len(),
            "flushed glyph vertex count must match ordinary FontString output"
        );
        for (actual, expected) in flushed_normalized.iter().zip(&expected_normalized) {
            for axis in 0..2 {
                assert!(
                    (actual[axis] - expected[axis]).abs() < 0.001,
                    "flushed glyph position {actual:?} != FontString {expected:?}"
                );
            }
        }
    }
}

// Public full-content alignment oracle using actual ScrollFrame clipping.
#[cfg(feature = "gui")]
mod overflow_vertical_alignment {
    use super::*;
    use wow_ui_sim::iced_app::{RegistryQuadBatchParams, build_quad_batch_for_registry};
    use wow_ui_sim::render::glyph::GlyphAtlas;

    fn render_visible_absolute_glyph_positions(
        env: &WowLuaEnv,
        fonts: &Rc<RefCell<WowFontSystem>>,
        atlas: &mut GlyphAtlas,
        root: &str,
    ) -> Vec<[f32; 2]> {
        let buckets = {
            let mut state = env.state().borrow_mut();
            state.ensure_layout_rects();
            let _ = state.get_strata_buckets();
            state.strata_buckets.as_ref().unwrap().clone()
        };
        let state = env.state().borrow();
        let mut fonts = fonts.borrow_mut();
        let batch = build_quad_batch_for_registry(
            RegistryQuadBatchParams::new(&state.widgets, (1024.0, 768.0), &buckets)
                .root_name(Some(root))
                .text_ctx(Some((&mut fonts, atlas))),
        );
        // Clipping retains fully excluded vertices with alpha zero. Exclude
        // those, but retain partially clipped glyph edges at absolute positions.
        batch
            .vertices
            .iter()
            .filter(|vertex| {
                vertex.tex_index == wow_ui_sim::render::shader::GLYPH_ATLAS_TEX_INDEX
                    && vertex.color[3] > 0.0
            })
            .map(|vertex| vertex.position)
            .collect()
    }

    #[test]
    fn multiline_overflow_center_and_bottom_match_full_content_scroll_viewport() {
        let outcomes = ["CENTER", "BOTTOM"].map(|alignment| {
            let env = create_env_with_installed_fonts();
            let fonts = Rc::new(RefCell::new(WowFontSystem::new_without_casc()));
            assert!(fonts.borrow().font_system.db().faces().next().is_some());
            assert!(fonts.borrow().family_name(Some(DEFAULT_WOW_FONT)).is_some());
            env.set_font_system(Rc::clone(&fonts));
            env.set_screen_size(1024.0, 768.0);
            env.exec(&format!(
                r#"
                local alignment = '{alignment}'
                local text = table.concat({{
                    'A', 'BB', 'CCC', 'DDDD', 'EEEEE',
                    'FFFFFF', 'GGGGGGG', 'HHHHHHHH', 'IIIIIIIII', 'JJJJJJJJJJ',
                }}, '\n')
                CursorBox:SetSize(400, 72)
                CursorBox:SetJustifyH('LEFT')
                CursorBox:SetJustifyV(alignment == 'CENTER' and 'MIDDLE' or alignment)
                CursorBox:SetText(text)
                CursorBox:SetCursorPosition(0)
                CursorBox:ClearFocus()
                CursorBox:SetScript('OnCursorChanged', nil)
                CursorBox:SetScript('OnUpdate', nil)

                OverflowViewport = CreateFrame('ScrollFrame', 'C2OverflowViewport', UIParent)
                OverflowViewport:SetPoint('TOPLEFT', CursorBox, 'TOPLEFT', 5, -3)
                OverflowViewport:SetSize(388, 65)
                OverflowViewport:Show()
                OverflowContent = CreateFrame('Frame', nil, OverflowViewport)
                -- Simulator-configured font20 line height is ceil(20 * 1.2) = 24.
                OverflowContent:SetSize(388, 10 * 24)
                OverflowContent:SetPoint(alignment, OverflowViewport, alignment, 0, 0)
                OverflowViewport:SetScrollChild(OverflowContent)
                assert(OverflowContent:GetPoint(1) == alignment,
                    'SETUP FAILURE: SetScrollChild changed explicit alignment anchor')
                OverflowViewport:SetHorizontalScroll(0)
                OverflowViewport:SetVerticalScroll(0)
                OverflowContent:Show()
                local reference = OverflowContent:CreateFontString(nil, 'ARTWORK')
                reference:SetAllPoints(OverflowContent)
                assert(reference:SetFont('Fonts\\FRIZQT__.TTF', 20))
                reference:SetJustifyH('LEFT')
                reference:SetJustifyV('TOP')
                reference:SetText(text)
                reference:Show()
                assert(CursorBox:GetText() == reference:GetText())
                "#,
            ))
            .expect("SETUP FAILURE: create public overflow fixtures");
            env.fire_on_update(TICK_SECONDS).unwrap();
            assert!(
                env.state().borrow().lua_errors.is_empty(),
                "SETUP FAILURE: {alignment} layout tick Lua errors: {:?}",
                env.state().borrow().lua_errors
            );
            env.exec(&format!(
                r#"
                local function near(a, b)
                    assert(math.abs(a - b) < 0.001, 'SETUP FAILURE: public fixture bounds')
                end
                near(OverflowViewport:GetLeft(), CursorBox:GetLeft() + 5)
                near(OverflowViewport:GetTop(), CursorBox:GetTop() - 3)
                near(OverflowViewport:GetWidth(), CursorBox:GetWidth() - 5 - 7)
                near(OverflowViewport:GetHeight(), CursorBox:GetHeight() - 3 - 4)
                near(OverflowContent:GetHeight(), 240)
                near(OverflowContent:GetLeft(), OverflowViewport:GetLeft())
                local offset = '{alignment}' == 'CENTER' and (240 - 65) / 2 or 240 - 65
                near(OverflowContent:GetTop(), OverflowViewport:GetTop() + offset)
                near(OverflowViewport:GetHorizontalScroll(), 0)
                near(OverflowViewport:GetVerticalScroll(), 0)
                "#,
            ))
            .expect("SETUP FAILURE: independent public viewport geometry");

            let mut atlas = GlyphAtlas::new();
            let actual = render_visible_absolute_glyph_positions(
                &env,
                &fonts,
                &mut atlas,
                "C2CursorLayoutBox",
            );
            let expected = render_visible_absolute_glyph_positions(
                &env,
                &fonts,
                &mut atlas,
                "C2OverflowViewport",
            );
            assert!(
                !actual.is_empty() && !expected.is_empty(),
                "SETUP FAILURE: {alignment} missing visible glyph output"
            );
            assert!(
                actual
                    .iter()
                    .chain(&expected)
                    .flatten()
                    .all(|value| value.is_finite()),
                "SETUP FAILURE: {alignment} nonfinite emitted position"
            );
            let first_difference = actual.iter().zip(&expected).position(|(actual, expected)| {
                (actual[0] - expected[0]).abs() >= 0.001 || (actual[1] - expected[1]).abs() >= 0.001
            });
            let matching = actual.len() == expected.len() && first_difference.is_none();
            let differing_positions =
                first_difference.map(|index| (actual[index], expected[index]));
            (
                alignment,
                matching,
                actual.len(),
                expected.len(),
                differing_positions,
            )
        });
        // Collect both cases before failing, without normalizing either origin.
        assert!(
            outcomes.iter().all(|outcome| outcome.1),
            "overflow glyph alignment != full-content viewport: \
             (alignment, matches, actual vertices, expected vertices, first differing positions) \
             {outcomes:?}"
        );
    }
}
