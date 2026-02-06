//! Tests for widget-specific methods (methods_widget.rs):
//! EditBox, CheckButton, ColorSelect, SimpleHTML, Drag/Moving.

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
    assert!(checked, "CheckButton should be checked after SetChecked(true)");

    env.exec("TestCB:SetChecked(false)").unwrap();
    let checked: bool = env.eval("return TestCB:GetChecked()").unwrap();
    assert!(!checked, "CheckButton should be unchecked after SetChecked(false)");
}

// ============================================================================
// ColorSelect: SetColorRGB / GetColorRGB
// ============================================================================

#[test]
fn test_colorselect_rgb() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local cs = CreateFrame("ColorSelect", "TestCS", UIParent)
        cs:SetColorRGB(0.5, 0.6, 0.7)
    "#,
    )
    .unwrap();

    let (r, g, b): (f64, f64, f64) = env
        .eval("return TestCS:GetColorRGB()")
        .unwrap();
    assert!((r - 0.5).abs() < 0.001);
    assert!((g - 0.6).abs() < 0.001);
    assert!((b - 0.7).abs() < 0.001);
}

#[test]
fn test_colorselect_rgb_defaults() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(r#"local cs = CreateFrame("ColorSelect", "TestCSDef", UIParent)"#)
        .unwrap();

    let (r, g, b): (f64, f64, f64) = env
        .eval("return TestCSDef:GetColorRGB()")
        .unwrap();
    assert_eq!(r, 1.0);
    assert_eq!(g, 1.0);
    assert_eq!(b, 1.0);
}

// ============================================================================
// ColorSelect: SetColorHSV / GetColorHSV
// ============================================================================

#[test]
fn test_colorselect_hsv_roundtrip() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local cs = CreateFrame("ColorSelect", "TestCSHSV", UIParent)
        cs:SetColorHSV(120, 0.5, 0.8)
    "#,
    )
    .unwrap();

    let (h, s, v): (f64, f64, f64) = env
        .eval("return TestCSHSV:GetColorHSV()")
        .unwrap();
    assert!((h - 120.0).abs() < 0.01);
    assert!((s - 0.5).abs() < 0.01);
    assert!((v - 0.8).abs() < 0.01);
}

#[test]
fn test_colorselect_hsv_to_rgb_red() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local cs = CreateFrame("ColorSelect", "TestCSRed", UIParent)
        cs:SetColorHSV(0, 1, 1)
    "#,
    )
    .unwrap();

    // HSV(0, 1, 1) should be pure red RGB(1, 0, 0)
    let (r, g, b): (f64, f64, f64) = env
        .eval("return TestCSRed:GetColorRGB()")
        .unwrap();
    assert!((r - 1.0).abs() < 0.01);
    assert!(g.abs() < 0.01);
    assert!(b.abs() < 0.01);
}

#[test]
fn test_colorselect_hsv_to_rgb_green() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local cs = CreateFrame("ColorSelect", "TestCSGreen", UIParent)
        cs:SetColorHSV(120, 1, 1)
    "#,
    )
    .unwrap();

    // HSV(120, 1, 1) should be pure green RGB(0, 1, 0)
    let (r, g, b): (f64, f64, f64) = env
        .eval("return TestCSGreen:GetColorRGB()")
        .unwrap();
    assert!(r.abs() < 0.01);
    assert!((g - 1.0).abs() < 0.01);
    assert!(b.abs() < 0.01);
}

#[test]
fn test_colorselect_rgb_to_hsv_conversion() {
    let env = WowLuaEnv::new().unwrap();

    // Set via RGB, then read via HSV
    env.exec(
        r#"
        local cs = CreateFrame("ColorSelect", "TestCSConv", UIParent)
        cs:SetColorRGB(1, 0, 0)
    "#,
    )
    .unwrap();

    // Pure red should be HSV(0, 1, 1)
    let (h, s, v): (f64, f64, f64) = env
        .eval("return TestCSConv:GetColorHSV()")
        .unwrap();
    assert!(h.abs() < 0.01, "Hue for red should be ~0, got {}", h);
    assert!((s - 1.0).abs() < 0.01, "Saturation for red should be 1, got {}", s);
    assert!((v - 1.0).abs() < 0.01, "Value for red should be 1, got {}", v);
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
