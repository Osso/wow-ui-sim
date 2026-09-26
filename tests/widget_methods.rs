//! Tests for EditBox, CheckButton, SimpleHTML, and frame property methods.
//! Widget misc tests are in `widget_misc_methods.rs`.

use wow_ui_sim::lua_api::WowLuaEnv;

// ============================================================================
// EditBox: SetFocus / ClearFocus / HasFocus
// ============================================================================

#[test]
fn test_editbox_focus() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(r#"local eb = CreateFrame("EditBox", "TestEB", UIParent)"#)
        .unwrap();

    let has_focus: bool = env.eval("return TestEB:HasFocus()").unwrap();
    assert!(!has_focus, "EditBox should not have focus initially");

    env.exec("TestEB:SetFocus()").unwrap();
    let has_focus: bool = env.eval("return TestEB:HasFocus()").unwrap();
    assert!(has_focus, "EditBox should have focus after SetFocus");

    env.exec("TestEB:ClearFocus()").unwrap();
    let has_focus: bool = env.eval("return TestEB:HasFocus()").unwrap();
    assert!(!has_focus, "EditBox should not have focus after ClearFocus");
}

#[test]
fn test_editbox_focus_switches_between_frames() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local eb1 = CreateFrame("EditBox", "TestEB1", UIParent)
        local eb2 = CreateFrame("EditBox", "TestEB2", UIParent)
        eb1:SetFocus()
    "#,
    )
    .unwrap();

    let eb1_focus: bool = env.eval("return TestEB1:HasFocus()").unwrap();
    assert!(eb1_focus);

    env.exec("TestEB2:SetFocus()").unwrap();
    // Only eb2 should have focus (eb1 doesn't auto-lose)
    let eb2_focus: bool = env.eval("return TestEB2:HasFocus()").unwrap();
    assert!(eb2_focus);
}

#[test]
fn editbox_lua_focus_transitions_dispatch_once_with_current_state() {
    let env = WowLuaEnv::new().unwrap();
    let calls: String = env.eval(r#"
        local first = CreateFrame('EditBox')
        local second = CreateFrame('EditBox')
        local calls = {}
        local function record(label, frame, event)
            table.insert(calls, label .. ':' .. event .. ':' .. tostring(frame:HasFocus()))
        end
        first:SetScript('OnEditFocusGained', function(self) record('first', self, 'gained') end)
        first:HookScript('OnEditFocusLost', function(self) record('first', self, 'lost') end)
        second:SetScript('OnEditFocusGained', function(self) record('second', self, 'gained') end)
        second:SetScript('OnEditFocusLost', function(self) record('second', self, 'lost') end)
        first:SetFocus()
        first:SetFocus()
        second:SetFocus()
        first:ClearFocus()
        second:SetFocus()
        second:ClearFocus()
        second:ClearFocus()
        return table.concat(calls, ',')
    "#).unwrap();
    assert_eq!(calls, "first:gained:true,first:lost:false,second:gained:true,second:lost:false");
}

#[test]
fn editbox_focus_lost_redirect_does_not_gain_stale_target() {
    let env = WowLuaEnv::new().unwrap();
    let calls: String = env.eval(r#"
        local first = CreateFrame('EditBox')
        local requested = CreateFrame('EditBox')
        local redirected = CreateFrame('EditBox')
        local calls = {}
        first:SetFocus()
        first:SetScript('OnEditFocusLost', function(self)
            table.insert(calls, 'lost:' .. tostring(self:HasFocus()))
            redirected:SetFocus()
        end)
        requested:SetScript('OnEditFocusGained', function() table.insert(calls, 'stale') end)
        redirected:SetScript('OnEditFocusGained', function(self)
            table.insert(calls, 'redirected:' .. tostring(self:HasFocus()))
        end)
        requested:SetFocus()
        assert(redirected:HasFocus() and not requested:HasFocus())
        return table.concat(calls, ',')
    "#).unwrap();
    assert_eq!(calls, "lost:false,redirected:true");
}

#[test]
fn editbox_focus_dispatches_intrinsic_bindings_in_order() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("FocusBindingProbe = function() end").unwrap();
    let root = tempfile::tempdir().unwrap();
    let toc = root.path().join("FocusBindings.toc");
    std::fs::write(&toc, "## Title: Focus bindings\nBindings.xml\n").unwrap();
    std::fs::write(root.path().join("Bindings.xml"), r#"
        <Ui>
            <Frame name="FocusPrecall" intrinsic="true"><Scripts>
                <OnEditFocusLost intrinsicOrder="precall">FocusBindingProbe('pre', self)</OnEditFocusLost>
            </Scripts></Frame>
            <Frame name="FocusPostcall" virtual="true"><Scripts>
                <OnEditFocusLost intrinsicOrder="postcall">FocusBindingProbe('post', self)</OnEditFocusLost>
            </Scripts></Frame>
        </Ui>
    "#).unwrap();
    let loaded = wow_ui_sim::loader::load_addon(&env.loader_env(), &toc).unwrap();
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    let bindings: String = env.eval(r#"
        local calls = {}
        FocusBindingProbe = function(binding, self)
            table.insert(calls, binding .. ':' .. tostring(self:HasFocus()))
        end
        local edit = CreateFrame('EditBox', nil, UIParent, 'FocusPrecall,FocusPostcall')
        assert(type(edit:GetScript('OnEditFocusLost', 0)) == 'function')
        assert(type(edit:GetScript('OnEditFocusLost', 2)) == 'function')
        edit:SetScript('OnEditFocusLost', function(self)
            table.insert(calls, 'normal:' .. tostring(self:HasFocus()))
        end)
        edit:SetFocus()
        edit:ClearFocus()
        return table.concat(calls, ',')
    "#).unwrap();
    assert_eq!(bindings, "pre:false,normal:false,post:false");
}

#[test]
fn editbox_focus_error_reports_and_other_focus_callbacks_continue() {
    let env = WowLuaEnv::new().unwrap();
    let hidden: bool = env.eval(r#"
        local popup = CreateFrame('Frame')
        local first = CreateFrame('EditBox')
        local second = CreateFrame('EditBox')
        first:SetScript('OnEditFocusLost', function() error('focus cleanup failure') end)
        second:SetScript('OnEditFocusGained', function() popup:Hide() end)
        first:SetFocus()
        second:SetFocus()
        return not popup:IsShown() and second:HasFocus()
    "#).unwrap();
    assert!(hidden, "gained callback must run despite former owner's failing lost callback");
    assert!(env.state().borrow().lua_errors.iter().any(|error| error.contains("focus cleanup failure")));
}

#[test]
fn test_button_get_text_height_measures_button_text() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local button = CreateFrame("Button", "TextHeightButton", UIParent)
        button:SetText("Quest")
    "#,
    )
    .unwrap();

    let height: f64 = env.eval("return TextHeightButton:GetTextHeight()").unwrap();
    assert!(
        height > 0.0,
        "Button:GetTextHeight should measure button text"
    );
}

// ============================================================================
// EditBox: SetNumber / GetNumber
// ============================================================================

#[test]
fn test_editbox_set_get_number() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local eb = CreateFrame("EditBox", "TestEBNum", UIParent)
        eb:SetNumber(42.5)
    "#,
    )
    .unwrap();

    let num: f64 = env.eval("return TestEBNum:GetNumber()").unwrap();
    assert!((num - 42.5).abs() < 0.01);
}

#[test]
fn test_editbox_get_number_default() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(r#"local eb = CreateFrame("EditBox", "TestEBNumDef", UIParent)"#)
        .unwrap();

    let num: f64 = env.eval("return TestEBNumDef:GetNumber()").unwrap();
    assert_eq!(num, 0.0, "GetNumber should return 0 when no text set");
}

// ============================================================================
// CheckButton: SetChecked / GetChecked
// ============================================================================

#[test]
fn test_checkbutton_checked_state() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(r#"local cb = CreateFrame("CheckButton", "TestCB", UIParent)"#)
        .unwrap();

    let checked: bool = env.eval("return TestCB:GetChecked()").unwrap();
    assert!(!checked, "CheckButton should be unchecked initially");

    env.exec("TestCB:SetChecked(true)").unwrap();
    let checked: bool = env.eval("return TestCB:GetChecked()").unwrap();
    assert!(
        checked,
        "CheckButton should be checked after SetChecked(true)"
    );

    env.exec("TestCB:SetChecked(false)").unwrap();
    let checked: bool = env.eval("return TestCB:GetChecked()").unwrap();
    assert!(
        !checked,
        "CheckButton should be unchecked after SetChecked(false)"
    );
}

// ============================================================================
// SimpleHTML: SetHyperlinkFormat / GetHyperlinkFormat
// ============================================================================

#[test]
fn test_simplehtml_hyperlink_format() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local sh = CreateFrame("SimpleHTML", "TestSH", UIParent)
        sh:SetHyperlinkFormat("|H%s|h[%s]|h")
    "#,
    )
    .unwrap();

    let fmt: String = env.eval("return TestSH:GetHyperlinkFormat()").unwrap();
    assert_eq!(fmt, "|H%s|h[%s]|h");
}

// ============================================================================
// SimpleHTML: SetHyperlinksEnabled / GetHyperlinksEnabled
// ============================================================================

#[test]
fn test_simplehtml_hyperlinks_enabled() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local sh = CreateFrame("SimpleHTML", "TestSHEnabled", UIParent)
        sh:SetHyperlinksEnabled(false)
    "#,
    )
    .unwrap();

    let enabled: bool = env
        .eval("return TestSHEnabled:GetHyperlinksEnabled()")
        .unwrap();
    assert!(!enabled);
}

// ============================================================================
// SimpleHTML: SetText strips HTML tags
// ============================================================================

#[test]
fn test_simplehtml_settext_strips_tags() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local sh = CreateFrame("SimpleHTML", "TestSHText", UIParent)
        sh:SetText("<p>Hello <b>World</b></p>")
    "#,
    )
    .unwrap();

    let text: String = env.eval("return TestSHText:GetText()").unwrap();
    assert_eq!(text, "Hello World", "HTML tags should be stripped");
}

#[test]
fn test_simplehtml_indented_word_wrap_round_trips_by_text_type() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local sh = CreateFrame("SimpleHTML", "TestSHIndentWrap", UIParent)
        sh:SetIndentedWordWrap("p", true)
    "#,
    )
    .unwrap();

    let enabled: bool = env
        .eval(r#"return TestSHIndentWrap:GetIndentedWordWrap("p")"#)
        .unwrap();
    assert!(
        enabled,
        "SimpleHTML should report the stored indented word wrap state for the text type"
    );

    let default_other: bool = env
        .eval(r#"return TestSHIndentWrap:GetIndentedWordWrap("h1")"#)
        .unwrap();
    assert!(
        !default_other,
        "SimpleHTML should default to false for text types without stored wrap state"
    );
}

// ============================================================================
// Drag/Moving: SetMovable / IsMovable / StartMoving / StopMovingOrSizing
// ============================================================================

#[test]
fn test_movable_set_get() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("Frame", "TestMovable", UIParent)
        f:SetMovable(true)
    "#,
    )
    .unwrap();

    let movable: bool = env.eval("return TestMovable:IsMovable()").unwrap();
    assert!(movable);
}

#[test]
fn test_resizable_set_get() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("Frame", "TestResizable", UIParent)
        f:SetResizable(true)
    "#,
    )
    .unwrap();

    let resizable: bool = env.eval("return TestResizable:IsResizable()").unwrap();
    assert!(resizable);
}

#[test]
fn test_clamped_to_screen_set_get() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("Frame", "TestClamped", UIParent)
        f:SetClampedToScreen(true)
    "#,
    )
    .unwrap();

    let clamped: bool = env.eval("return TestClamped:IsClampedToScreen()").unwrap();
    assert!(clamped);
}

// ============================================================================
// StartSizing / StopMovingOrSizing / Resize bounds
// ============================================================================

#[test]
fn test_start_sizing_requires_resizable() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("Frame", "TestSizingNotResizable", UIParent)
        f:SetSize(200, 100)
        f:StartSizing("BOTTOMRIGHT")
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let id = state
        .widgets
        .get_id_by_name("TestSizingNotResizable")
        .unwrap();
    let frame = state.widgets.get(id).unwrap();
    assert!(
        !frame.is_sizing,
        "StartSizing should be ignored when frame is not resizable"
    );
}

#[test]
fn test_start_sizing_sets_state() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("Frame", "TestSizingState", UIParent)
        f:SetSize(200, 100)
        f:SetResizable(true)
        f:StartSizing("BOTTOMRIGHT")
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name("TestSizingState").unwrap();
    let frame = state.widgets.get(id).unwrap();
    assert!(frame.is_sizing, "StartSizing should set is_sizing flag");
    assert_eq!(
        frame.sizing_point,
        wow_ui_sim::widget::AnchorPoint::BottomRight
    );
}

#[test]
fn test_start_sizing_bottomleft() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("Frame", "TestSizingBL", UIParent)
        f:SetSize(200, 100)
        f:SetResizable(true)
        f:StartSizing("BOTTOMLEFT")
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name("TestSizingBL").unwrap();
    let frame = state.widgets.get(id).unwrap();
    assert!(frame.is_sizing);
    assert_eq!(
        frame.sizing_point,
        wow_ui_sim::widget::AnchorPoint::BottomLeft
    );
}

#[test]
fn test_start_sizing_defaults_to_bottomright() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("Frame", "TestSizingDefault", UIParent)
        f:SetSize(200, 100)
        f:SetResizable(true)
        f:StartSizing()
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name("TestSizingDefault").unwrap();
    let frame = state.widgets.get(id).unwrap();
    assert!(frame.is_sizing);
    assert_eq!(
        frame.sizing_point,
        wow_ui_sim::widget::AnchorPoint::BottomRight,
        "StartSizing with no argument should default to BOTTOMRIGHT"
    );
}

#[test]
fn test_stop_moving_or_sizing_clears_sizing() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("Frame", "TestStopSizing", UIParent)
        f:SetSize(200, 100)
        f:SetResizable(true)
        f:StartSizing("BOTTOMRIGHT")
    "#,
    )
    .unwrap();

    {
        let state = env.state().borrow();
        let id = state.widgets.get_id_by_name("TestStopSizing").unwrap();
        assert!(state.widgets.get(id).unwrap().is_sizing);
    }

    env.exec("TestStopSizing:StopMovingOrSizing()").unwrap();

    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name("TestStopSizing").unwrap();
    let frame = state.widgets.get(id).unwrap();
    assert!(
        !frame.is_sizing,
        "StopMovingOrSizing should clear is_sizing"
    );
    assert!(
        frame.user_placed,
        "StopMovingOrSizing should set user_placed after sizing"
    );
}

#[test]
fn test_resize_bounds_clamp() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("Frame", "TestResizeBounds", UIParent)
        f:SetSize(200, 100)
        f:SetResizable(true)
        f:SetResizeBounds(100, 50, 300, 200)
    "#,
    )
    .unwrap();

    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name("TestResizeBounds").unwrap();
    let frame = state.widgets.get(id).unwrap();
    assert_eq!(frame.resize_bounds_min, (100.0, 50.0));
    assert_eq!(frame.resize_bounds_max, Some((300.0, 200.0)));
}

// ============================================================================
// Alpha: SetAlpha / GetAlpha on frames
// ============================================================================

#[test]
fn test_set_alpha_zero_persists() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("Button", "TestAlphaZeroBtn", UIParent)
        f:SetAlpha(0)
    "#,
    )
    .unwrap();

    let alpha: f64 = env.eval("return TestAlphaZeroBtn:GetAlpha()").unwrap();
    assert!(
        alpha.abs() < 0.001,
        "Button with SetAlpha(0) should have alpha=0, got {alpha}"
    );

    let state = env.state().borrow();
    let id = state.widgets.get_id_by_name("TestAlphaZeroBtn").unwrap();
    let frame = state.widgets.get(id).unwrap();
    assert!(
        frame.alpha.abs() < 0.001,
        "Rust frame.alpha should be 0, got {}",
        frame.alpha
    );
}

// ============================================================================
// Button / CheckButton / EditBox: mouse enabled by default
// ============================================================================

#[test]
fn test_button_mouse_enabled_by_default() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"CreateFrame("Button", "TestMouseBtn", UIParent)"#)
        .unwrap();
    let enabled: bool = env
        .eval("return TestMouseBtn:IsMouseClickEnabled()")
        .unwrap();
    assert!(enabled, "Button should have mouse enabled by default");
}

#[test]
fn test_checkbutton_mouse_enabled_by_default() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"CreateFrame("CheckButton", "TestMouseCB", UIParent)"#)
        .unwrap();
    let enabled: bool = env
        .eval("return TestMouseCB:IsMouseClickEnabled()")
        .unwrap();
    assert!(enabled, "CheckButton should have mouse enabled by default");
}

#[test]
fn test_frame_mouse_disabled_by_default() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"CreateFrame("Frame", "TestMouseFrame", UIParent)"#)
        .unwrap();
    let enabled: bool = env
        .eval("return TestMouseFrame:IsMouseClickEnabled()")
        .unwrap();
    assert!(!enabled, "Frame should not have mouse enabled by default");
}
