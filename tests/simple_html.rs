//! Tests for SimpleHTML frame type implementation.

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn test_create_simple_html_correct_type() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(r#"local f = CreateFrame("SimpleHTML", "TestHTML", UIParent)"#)
        .unwrap();

    let obj_type: String = env.eval("return TestHTML:GetObjectType()").unwrap();
    assert_eq!(obj_type, "SimpleHTML");
}

#[test]
fn test_simple_html_is_object_type_frame() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(r#"local f = CreateFrame("SimpleHTML", "TestHTML2", UIParent)"#)
        .unwrap();

    let is_frame: bool = env
        .eval("return TestHTML2:IsObjectType('Frame')")
        .unwrap();
    assert!(is_frame);

    let is_region: bool = env
        .eval("return TestHTML2:IsObjectType('Region')")
        .unwrap();
    assert!(is_region);

    let is_html: bool = env
        .eval("return TestHTML2:IsObjectType('SimpleHTML')")
        .unwrap();
    assert!(is_html);

    let is_button: bool = env
        .eval("return TestHTML2:IsObjectType('Button')")
        .unwrap();
    assert!(!is_button);
}

#[test]
fn test_settext_stores_plain_text() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("SimpleHTML", "TestHTMLText", UIParent)
        f:SetText("Hello world")
    "#,
    )
    .unwrap();

    let text: String = env.eval("return TestHTMLText:GetText()").unwrap();
    assert_eq!(text, "Hello world");
}

#[test]
fn test_settext_strips_html_tags() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("SimpleHTML", "TestHTMLStrip", UIParent)
        f:SetText("<h1>Title</h1><p>Body text</p>")
    "#,
    )
    .unwrap();

    let text: String = env.eval("return TestHTMLStrip:GetText()").unwrap();
    assert_eq!(text, "TitleBody text");
}

#[test]
fn test_hyperlink_format_default() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(r#"local f = CreateFrame("SimpleHTML", "TestHTMLHyper", UIParent)"#)
        .unwrap();

    let format: String = env
        .eval("return TestHTMLHyper:GetHyperlinkFormat()")
        .unwrap();
    assert_eq!(format, "|H%s|h%s|h");
}

#[test]
fn test_set_get_hyperlink_format() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("SimpleHTML", "TestHTMLHyper2", UIParent)
        f:SetHyperlinkFormat("|cff%s|h%s|h")
    "#,
    )
    .unwrap();

    let format: String = env
        .eval("return TestHTMLHyper2:GetHyperlinkFormat()")
        .unwrap();
    assert_eq!(format, "|cff%s|h%s|h");
}

#[test]
fn test_hyperlinks_enabled_default() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(r#"local f = CreateFrame("SimpleHTML", "TestHTMLHE", UIParent)"#)
        .unwrap();

    let enabled: bool = env
        .eval("return TestHTMLHE:GetHyperlinksEnabled()")
        .unwrap();
    assert!(enabled);
}

#[test]
fn test_set_get_hyperlinks_enabled() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("SimpleHTML", "TestHTMLHE2", UIParent)
        f:SetHyperlinksEnabled(false)
    "#,
    )
    .unwrap();

    let enabled: bool = env
        .eval("return TestHTMLHE2:GetHyperlinksEnabled()")
        .unwrap();
    assert!(!enabled);
}

#[test]
fn test_per_texttype_set_text_color() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("SimpleHTML", "TestHTMLColor", UIParent)
        f:SetTextColor("h1", 1, 0, 0, 1)
    "#,
    )
    .unwrap();

    let (r, g, b, a): (f32, f32, f32, f32) = env
        .eval("return TestHTMLColor:GetTextColor('h1')")
        .unwrap();
    assert_eq!((r, g, b, a), (1.0, 0.0, 0.0, 1.0));
}

#[test]
fn test_per_texttype_set_font() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("SimpleHTML", "TestHTMLFont", UIParent)
        f:SetFont("h1", "Fonts\\FRIZQT__.TTF", 24)
    "#,
    )
    .unwrap();

    let (font, size): (String, f64) = env
        .eval(r#"local a, b = TestHTMLFont:GetFont("h1"); return a, b"#)
        .unwrap();
    assert_eq!(font, "Fonts\\FRIZQT__.TTF");
    assert_eq!(size, 24.0);
}

#[test]
fn test_get_content_height_nonzero_with_text() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("SimpleHTML", "TestHTMLHeight", UIParent)
        f:SetSize(400, 300)
        f:SetText("Some content text that should give a non-zero content height")
    "#,
    )
    .unwrap();

    let height: f64 = env
        .eval("return TestHTMLHeight:GetContentHeight()")
        .unwrap();
    assert!(height > 0.0, "GetContentHeight should return > 0 when text is set, got {}", height);
}

#[test]
fn test_get_content_height_zero_without_text() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(r#"local f = CreateFrame("SimpleHTML", "TestHTMLHeight2", UIParent)"#)
        .unwrap();

    let height: f64 = env
        .eval("return TestHTMLHeight2:GetContentHeight()")
        .unwrap();
    assert_eq!(height, 0.0);
}

#[test]
fn test_get_text_data_returns_table() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(r#"local f = CreateFrame("SimpleHTML", "TestHTMLData", UIParent)"#)
        .unwrap();

    let is_table: bool = env
        .eval("return type(TestHTMLData:GetTextData()) == 'table'")
        .unwrap();
    assert!(is_table);
}

#[test]
fn test_fontstring_settext_no_html_stripping() {
    // Verify that regular FontStrings do NOT strip HTML tags
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local parent = CreateFrame("Frame", "TestFSParent", UIParent)
        local fs = parent:CreateFontString("TestFontStr", "OVERLAY")
        fs:SetText("<h1>Title</h1>")
    "#,
    )
    .unwrap();

    // The methods_widget SetText overrides methods_text SetText, but only strips
    // HTML for SimpleHTML frames. Regular frames store as-is.
    let text: String = env.eval("return TestFontStr:GetText()").unwrap();
    assert_eq!(text, "<h1>Title</h1>", "FontString should store HTML tags as-is");
}
