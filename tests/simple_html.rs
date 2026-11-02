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

    let is_frame: bool = env.eval("return TestHTML2:IsObjectType('Frame')").unwrap();
    assert!(is_frame);

    let is_region: bool = env.eval("return TestHTML2:IsObjectType('Region')").unwrap();
    assert!(is_region);

    let is_html: bool = env
        .eval("return TestHTML2:IsObjectType('SimpleHTML')")
        .unwrap();
    assert!(is_html);

    let is_button: bool = env.eval("return TestHTML2:IsObjectType('Button')").unwrap();
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
fn test_settext_keeps_existing_plain_projection_not_document_geometry() {
    // GetText's existing projection is preserved. This is not rendering proof;
    // H1's original-consumer tests assert distinct presented block rectangles.
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
fn test_settext_strips_wow_markup_after_html_tags() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("SimpleHTML", "TestHTMLWowMarkup", UIParent)
        f:SetText("<p>|cFF2959D3|Hspell:1225135|h[Suppression Zones]|h|r|nNext</p>")
    "#,
    )
    .unwrap();

    let text: String = env.eval("return TestHTMLWowMarkup:GetText()").unwrap();
    assert_eq!(text, "[Suppression Zones]\nNext");
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

    let (r, g, b, a): (f32, f32, f32, f32) =
        env.eval("return TestHTMLColor:GetTextColor('h1')").unwrap();
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
fn test_per_texttype_set_font_object_by_name() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local font = CreateFont("TestHTMLHeadingFont")
        font:SetFont("Fonts\\FRIZQT__.TTF", 24, "OUTLINE")

        local f = CreateFrame("SimpleHTML", "TestHTMLFontObject", UIParent)
        f:SetFontObject("h1", "TestHTMLHeadingFont")
    "#,
    )
    .unwrap();

    let (font, size, flags): (String, f64, String) = env
        .eval(r#"return TestHTMLFontObject:GetFont("h1")"#)
        .unwrap();
    assert_eq!(font, "Fonts\\FRIZQT__.TTF");
    assert_eq!(size, 24.0);
    assert_eq!(flags, "OUTLINE");
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
    assert!(
        height > 0.0,
        "GetContentHeight should return > 0 when text is set, got {}",
        height
    );
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
fn test_get_content_height_tracks_resolved_layout_width() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local parent = CreateFrame("Frame", "TestHTMLLayoutParent", UIParent)
        parent:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 0, 0)
        parent:SetSize(400, 200)

        local f = CreateFrame("SimpleHTML", "TestHTMLLayoutWidth", parent)
        f:SetPoint("TOPLEFT", parent, "TOPLEFT", 0, 0)
        f:SetPoint("TOPRIGHT", parent, "TOPRIGHT", 0, 0)
        f:SetHeight(200)
        f:SetText("This is long SimpleHTML content that should wrap differently when the resolved layout width changes from wide to narrow. This is long SimpleHTML content that should wrap differently when the resolved layout width changes from wide to narrow. This is long SimpleHTML content that should wrap differently when the resolved layout width changes from wide to narrow.")
    "#,
    )
    .unwrap();

    let wide_height: f64 = env
        .eval("return TestHTMLLayoutWidth:GetContentHeight()")
        .unwrap();

    env.exec("TestHTMLLayoutParent:SetWidth(160)").unwrap();

    let narrow_height: f64 = env
        .eval("return TestHTMLLayoutWidth:GetContentHeight()")
        .unwrap();

    assert!(
        narrow_height > wide_height,
        "Narrower resolved layout width should increase wrapped content height (wide={}, narrow={})",
        wide_height,
        narrow_height
    );
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
fn test_get_text_data_returns_expected_fields() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local f = CreateFrame("SimpleHTML", "TestHTMLDataFields", UIParent)
        f:SetHyperlinkFormat("|cffffd200|H%s|h[%s]|h")
        f:SetHyperlinksEnabled(false)
        f:SetText("Hello <b>world</b>")
    "#,
    )
    .unwrap();

    let (hyperlink_format, hyperlinks_enabled, text, text_styles_type): (
        String,
        bool,
        String,
        String,
    ) = env
        .eval(
            r#"
            local data = TestHTMLDataFields:GetTextData()
            return data.hyperlinkFormat,
                   data.hyperlinksEnabled,
                   data.text,
                   type(data.textStyles)
        "#,
        )
        .unwrap();

    assert_eq!(hyperlink_format, "|cffffd200|H%s|h[%s]|h");
    assert!(!hyperlinks_enabled);
    assert_eq!(text, "Hello <b>world</b>");
    assert_eq!(text_styles_type, "table");
}

#[test]
fn test_get_text_data_exposes_content_nodes_and_fresh_snapshots() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local frame = CreateFrame("SimpleHTML", nil, UIParent)
        frame:SetJustifyH("CENTER")
        frame:SetText("<html><body><h1>Title</h1><p>One &amp; two<br/>Next line</p><h2>Subtitle</h2></body></html>")
        local content = frame:GetTextData()
        assert(#content == 3)
        assert(content[1].type == "H1" and content[1].text == "Title")
        assert(content[2].type == "P" and content[2].text == "One & two\nNext line")
        assert(content[3].type == "H2" and content[3].text == "Subtitle")
        for _, node in ipairs(content) do
            assert(node.align == "CENTER")
        end
        content[1].text = "mutated"
        assert(frame:GetTextData()[1].text == "Title")
        frame:SetText("<html><body><p>Replacement</p></body></html>")
        local replacement = frame:GetTextData()
        assert(#replacement == 1 and replacement[1].text == "Replacement")
        assert(#content == 3 and content[2].text == "One & two\nNext line")
        frame:SetText("")
        assert(#frame:GetTextData() == 0)
        "#,
    )
    .expect("SimpleHTML content nodes should reflect the parsed document, independently of layout");
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
    assert_eq!(
        text, "<h1>Title</h1>",
        "FontString should store HTML tags as-is"
    );
}
