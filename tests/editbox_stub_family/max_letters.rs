use super::env;

#[cfg(feature = "client-wowforever")]
#[test]
fn editbox_set_text_max_letters_secret_clipping_preserves_access_controls() {
    env()
        .exec(
            r#"
        local eb = CreateFrame('EditBox')
        eb:SetMaxLetters(3)
        local wrapped = secretwrap('hidden payload')
        eb:SetText(wrapped)
        assert(eb:GetText() == 'hid')
        local function addon()
            assert(not pcall(eb.GetText, eb), 'tainted caller must not read secret-origin text')
            assert(not pcall(eb.SetText, eb, wrapped), 'tainted secret write must fail')
        end
        debug.setobjecttaint(addon, 'SecretEditBoxProbe')
        addon()
        assert(eb:GetText() == 'hid', 'rejected secret write must preserve clipped text')
        assert(not pcall(eb.SetText, eb, newproxy()), 'ordinary userdata must be rejected')
        assert(eb:GetText() == 'hid', 'rejected userdata must not mutate text')
        eb:SetText('plain')
        assert(eb:GetText() == 'pla')
        local function read_plain() assert(eb:GetText() == 'pla') end
        debug.setobjecttaint(read_plain, 'SecretEditBoxProbe')
        read_plain()
    "#,
        )
        .unwrap();
}

#[test]
fn editbox_set_text_max_letters_matches_recorded_ascii_fixture() {
    env()
        .exec(
            r#"
        local eb = CreateFrame('EditBox')
        eb:SetFontObject('GameFontNormal')
        eb:SetMaxLetters(5)
        eb:SetText('hello world')
        assert(eb:GetText() == 'hello', eb:GetText())
        assert(eb:GetNumLetters() == 5)
    "#,
        )
        .unwrap();
}

#[test]
fn editbox_set_text_max_letters_clips_unicode_before_callbacks_and_render_cache() {
    let env = env();
    env.exec(r#"
        local eb = CreateFrame('EditBox', 'LimitedUnicodeEB', UIParent)
        eb:SetText('abcdef')
        eb:SetCursorPosition(6)
        eb:SetMaxLetters(3)
        local seen = {}
        eb:SetScript('OnTextSet', function(self)
            seen[#seen + 1] = 'set:' .. self:GetText() .. ':' .. self:GetCursorPosition() .. ':' .. self:GetNumLetters()
        end)
        eb:SetScript('OnTextChanged', function(self, userInput)
            seen[#seen + 1] = 'changed:' .. self:GetText() .. ':' .. self:GetCursorPosition() .. ':' .. tostring(userInput)
        end)
        eb:SetText('é猫ABC')
        assert(table.concat(seen, ',') == 'set:é猫A:6:3,changed:é猫A:6:false', table.concat(seen, ','))
        assert(eb:GetUTF8CursorPosition() == 3)
    "#).unwrap();
    let state = env.state();
    let state = state.borrow();
    let frame = state
        .widgets
        .get(state.widgets.get_id_by_name("LimitedUnicodeEB").unwrap())
        .unwrap();
    assert_eq!(frame.text.as_deref(), Some("é猫A"));
    assert_eq!(frame.text_stripped.as_deref(), Some("é猫A"));
}

#[test]
fn editbox_set_text_max_letters_repeated_clipping_is_noop_including_formatted_text() {
    env()
        .exec(
            r#"
        local eb = CreateFrame('EditBox', nil, UIParent)
        eb:SetMaxLetters(5)
        local seen = {}
        eb:SetScript('OnTextSet', function(self) seen[#seen + 1] = 'set:' .. self:GetText() end)
        eb:SetScript('OnTextChanged', function(self, userInput)
            seen[#seen + 1] = 'changed:' .. self:GetText() .. ':' .. tostring(userInput)
        end)
        eb:SetText('hello world')
        eb:SetText('hello again')
        eb:SetFormattedText('%s %s', 'hello', 'friend')
        assert(eb:GetText() == 'hello')
        assert(table.concat(seen, ',') == 'set:hello,changed:hello:false', table.concat(seen, ','))
    "#,
        )
        .unwrap();
}

#[test]
fn editbox_set_text_max_letters_preserves_unlimited_and_non_editbox_text() {
    env()
        .exec(
            r#"
        local eb = CreateFrame('EditBox', nil, UIParent)
        eb:SetMaxLetters(0)
        eb:SetText('é猫ABC')
        assert(eb:GetText() == 'é猫ABC' and eb:GetNumLetters() == 5)
        local text = UIParent:CreateFontString(nil, 'ARTWORK')
        text:SetText('hello world')
        assert(text:GetText() == 'hello world')
    "#,
        )
        .unwrap();
}
