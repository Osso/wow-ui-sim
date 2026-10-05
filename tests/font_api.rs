//! Tests for font API functions (font_api.rs).

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

// ============================================================================
// CreateFont
// ============================================================================

#[test]
fn test_create_font_named() {
    let env = env();
    let is_table: bool = env
        .eval("local f = CreateFont('TestFont'); return type(f) == 'table'")
        .unwrap();
    assert!(is_table);
}

#[test]
fn test_create_font_registers_global() {
    let env = env();
    env.eval::<()>("CreateFont('MyFont')").unwrap();
    let exists: bool = env.eval("return MyFont ~= nil").unwrap();
    assert!(exists);
}

#[test]
fn test_create_font_set_get_font() {
    let env = env();
    let (path, height, flags): (String, f64, String) = env
        .eval(
            r#"
            local f = CreateFont('TestFont2')
            f:SetFont("Fonts\\Arial.ttf", 16, "OUTLINE")
            return f:GetFont()
            "#,
        )
        .unwrap();
    assert_eq!(path, "Fonts\\Arial.ttf");
    assert_eq!(height, 16.0);
    assert_eq!(flags, "OUTLINE");
}

#[test]
fn test_create_font_default_values() {
    let env = env();
    let (is_nil, height): (bool, f64) = env
        .eval(
            r#"
            local f = CreateFont('TestDefault')
            local p, h = f:GetFont()
            return p == nil, h
            "#,
        )
        .unwrap();
    assert!(is_nil, "GetFont path should be nil on fresh CreateFont");
    assert_eq!(height, 0.0);
}

#[test]
fn test_copy_font_object_accepts_xml_style_font_fields() {
    let env = env();
    let (path, height, flags): (String, f64, String) = env
        .eval(
            r#"
            local source = {
                __font = "Fonts\\ARIALN.TTF",
                __height = 14,
                __outline = "OUTLINE",
            }
            local copy = CreateFont("XmlStyleFontCopyProbe")
            copy:CopyFontObject(source)
            return copy:GetFont()
            "#,
        )
        .unwrap();

    assert_eq!(path, "Fonts\\ARIALN.TTF");
    assert_eq!(height, 14.0);
    assert_eq!(flags, "OUTLINE");
}

#[test]
fn test_set_font_object_prefers_canonical_fields_over_legacy_aliases() {
    let env = env();
    let (path, height, flags): (String, f64, String) = env
        .eval(
            r#"
            local font = {
                __fontPath = "Fonts\\ARIALN.TTF",
                __font = "Fonts\\FRIZQT__.TTF",
                __fontHeight = 24,
                __height = 12,
                __fontFlags = "OUTLINE",
                __outline = "",
            }
            local fontString = UIParent:CreateFontString("CanonicalFontFieldsProbe", "ARTWORK")
            fontString:SetFontObject(font)
            return fontString:GetFont()
            "#,
        )
        .unwrap();

    assert_eq!(path, "Fonts\\ARIALN.TTF");
    assert_eq!(height, 24.0);
    assert_eq!(flags, "OUTLINE");
}

#[test]
fn test_set_font_nil_path_does_not_clear_existing_font() {
    let env = env();
    let (result, path, height): (bool, String, f64) = env
        .eval(
            r#"
            local f = UIParent:CreateFontString("SetFontNilPathProbe", "ARTWORK", "ChatFontNormal")
            local result = f:SetFont(nil, 0)
            local path, height = f:GetFont()
            return result, path, height
            "#,
        )
        .unwrap();

    assert!(!result, "SetFont(nil, ...) should report failure");
    assert_eq!(path, "Fonts\\FRIZQT__.TTF");
    assert_eq!(height, 14.0);
}

#[test]
fn test_message_frame_set_font_object_uses_chat_font_height() {
    let env = env();
    let height: f64 = env
        .eval(
            r#"
            local f = CreateFrame("ScrollingMessageFrame", "MessageFrameFontObjectProbe", UIParent)
            f:SetFontObject(ChatFontNormal)
            local _, height = f:GetFont()
            return height
            "#,
        )
        .unwrap();

    assert_eq!(height, 14.0);
}

#[test]
fn test_create_font_text_color() {
    let env = env();
    let (r, g, b, a): (f64, f64, f64, f64) = env
        .eval(
            r#"
            local f = CreateFont('TestColor')
            f:SetTextColor(0.5, 0.6, 0.7, 0.8)
            return f:GetTextColor()
            "#,
        )
        .unwrap();
    assert_eq!((r, g, b, a), (0.5, 0.6, 0.7, 0.8));
}

#[test]
fn test_create_font_shadow() {
    let env = env();
    let (r, g, b, a, x, y): (f64, f64, f64, f64, f64, f64) = env
        .eval(
            r#"
            local f = CreateFont('TestShadow')
            f:SetShadowColor(0.1, 0.2, 0.3, 0.4)
            f:SetShadowOffset(2, -2)
            local r, g, b, a = f:GetShadowColor()
            local x, y = f:GetShadowOffset()
            return r, g, b, a, x, y
            "#,
        )
        .unwrap();
    assert_eq!((r, g, b, a), (0.1, 0.2, 0.3, 0.4));
    assert_eq!((x, y), (2.0, -2.0));
}

#[test]
fn test_standard_font_objects_expose_methods_through_metatable_index() {
    let env = env();
    let result: (String, String, String, bool) = env
        .eval(
            r#"
            local mt = getmetatable(GameFontNormal)
            local index = mt and mt.__index
            GameFontNormal:SetShadowColor(0.2, 0.3, 0.4, 0.5)
            local r, g, b, a = GameFontNormal:GetShadowColor()
            return type(mt), type(index), type(index and index.SetShadowColor),
                r == 0.2 and g == 0.3 and b == 0.4 and a == 0.5
            "#,
        )
        .unwrap();

    assert_eq!(
        result,
        (
            "table".to_string(),
            "table".to_string(),
            "function".to_string(),
            true
        )
    );
}

#[test]
fn test_font_string_shadow_methods() {
    let env = env();
    let (r, g, b, a, x, y): (f64, f64, f64, f64, f64, f64) = env
        .eval(
            r#"
            local f = UIParent:CreateFontString("FontStringShadowProbe", "ARTWORK")
            f:SetShadowColor(0.2, 0.3, 0.4, 0.5)
            f:SetShadowOffset(3, -4)
            local r, g, b, a = f:GetShadowColor()
            local x, y = f:GetShadowOffset()
            return r, g, b, a, x, y
            "#,
        )
        .unwrap();

    assert!((r - 0.2).abs() < 0.0001);
    assert!((g - 0.3).abs() < 0.0001);
    assert!((b - 0.4).abs() < 0.0001);
    assert!((a - 0.5).abs() < 0.0001);
    assert_eq!((x, y), (3.0, -4.0));
}

#[test]
fn test_create_font_justify() {
    let env = env();
    let (h, v): (String, String) = env
        .eval(
            r#"
            local f = CreateFont('TestJustify')
            f:SetJustifyH("LEFT")
            f:SetJustifyV("TOP")
            return f:GetJustifyH(), f:GetJustifyV()
            "#,
        )
        .unwrap();
    assert_eq!(h, "LEFT");
    assert_eq!(v, "TOP");
}

#[test]
fn test_create_font_spacing() {
    let env = env();
    let spacing: f64 = env
        .eval(
            r#"
            local f = CreateFont('TestSpacing')
            f:SetSpacing(3.5)
            return f:GetSpacing()
            "#,
        )
        .unwrap();
    assert_eq!(spacing, 3.5);
}

#[test]
fn test_create_font_get_name() {
    let env = env();
    let name: String = env
        .eval("local f = CreateFont('NamedFont'); return f:GetName()")
        .unwrap();
    assert_eq!(name, "NamedFont");
}

#[test]
fn test_create_font_copy_font_object() {
    let env = env();
    let (path, height): (String, f64) = env
        .eval(
            r#"
            local src = CreateFont('SrcFont')
            src:SetFont("Fonts\\Custom.ttf", 24, "THICKOUTLINE")
            local dst = CreateFont('DstFont')
            dst:CopyFontObject(src)
            return dst:GetFont()
            "#,
        )
        .unwrap();
    assert_eq!(path, "Fonts\\Custom.ttf");
    assert_eq!(height, 24.0);
}

#[test]
fn test_create_font_copy_by_name() {
    let env = env();
    let height: f64 = env
        .eval(
            r#"
            local src = CreateFont('CopySrc')
            src:SetFont("Fonts\\Big.ttf", 32)
            local dst = CreateFont('CopyDst')
            dst:CopyFontObject("CopySrc")
            local _, h = dst:GetFont()
            return h
            "#,
        )
        .unwrap();
    assert_eq!(height, 32.0);
}

#[test]
fn test_create_font_get_font_object_for_alphabet() {
    let env = env();
    let same: bool = env
        .eval(
            r#"
            local f = CreateFont('AlphaFont')
            return f:GetFontObjectForAlphabet("roman") == f
            "#,
        )
        .unwrap();
    assert!(same);
}

#[test]
fn text_region_reports_truncation_when_text_exceeds_width() {
    let env = env();
    let truncated: bool = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", "FontTruncationProbeParent", UIParent)
            local fs = frame:CreateFontString("FontTruncationProbe", "ARTWORK", "GameFontNormal")
            fs:SetWidth(20)
            fs:SetText("This is wider than twenty pixels")
            return fs:IsTruncated()
            "#,
        )
        .unwrap();
    assert!(truncated, "FontString:IsTruncated() should report overflow");
}

#[test]
fn font_string_get_wrapped_width_uses_active_wrap_width() {
    let env = env();
    let (string_width, wrapped_width): (f64, f64) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", "WrappedWidthProbeParent", UIParent)
            local fs = frame:CreateFontString("WrappedWidthProbe", "ARTWORK", "GameFontNormal")
            fs:SetText("This is long enough to wrap")
            fs:SetWordWrap(true)
            fs:SetWidth(60)
            return fs:GetStringWidth(), fs:GetWrappedWidth()
            "#,
        )
        .unwrap();
    assert!(
        string_width > 60.0,
        "test text should be wider than the wrap width"
    );
    assert_eq!(wrapped_width, 60.0);
}

#[cfg(feature = "retail-12-0-5")]
#[test]
fn font_string_unbounded_width_for_text_measures_caller_text_in_own_font() {
    let env = env();
    let (for_text, own_width, reference_width, short_width, text): (f64, f64, f64, f64, String) =
        env.eval(
            r#"
            local frame = CreateFrame("Frame", "WidthForTextProbeParent", UIParent)
            local fs = frame:CreateFontString("WidthForTextProbe", "ARTWORK", "GameFontNormal")
            fs:SetText("Tier 1")
            fs:SetWidth(20)
            fs:SetWordWrap(true)
            local shortWidth = fs:GetUnboundedStringWidth()
            local forText = fs:GetUnboundedStringWidthForText("|cffff0000Tier 11 (Lair Challenges)|r")
            local ownWidth = fs:GetUnboundedStringWidth()
            local text = fs:GetText()
            local reference = frame:CreateFontString(nil, "ARTWORK", "GameFontNormal")
            reference:SetText("Tier 11 (Lair Challenges)")
            return forText, ownWidth, reference:GetUnboundedStringWidth(), shortWidth, text
            "#,
        )
        .unwrap();
    assert_eq!(for_text, reference_width, "markup is stripped and the own font is used");
    assert!(for_text > 20.0, "measurement is not bounded by the frame width");
    assert!(for_text > short_width);
    assert_eq!(own_width, short_width, "own text measurement is unchanged");
    assert_eq!(text, "Tier 1");
}

#[test]
fn create_font_string_inherits_named_font_object_properties() {
    let env = env();
    let (path, height, flags): (String, f64, String) = env
        .eval(
            r#"
            local inherited = CreateFont("InheritedFontObject")
            inherited:SetFont("Fonts\\Custom.ttf", 18, "OUTLINE")
            local frame = CreateFrame("Frame", "FontInheritanceProbeParent", UIParent)
            local fs = frame:CreateFontString("FontInheritanceProbe", "ARTWORK", "InheritedFontObject")
            return fs:GetFont()
            "#,
        )
        .unwrap();
    assert_eq!(path, "Fonts\\Custom.ttf");
    assert_eq!(height, 18.0);
    assert_eq!(flags, "OUTLINE");
}

// ============================================================================
// GetFonts / GetFontInfo
// ============================================================================

#[test]
fn test_get_fonts_returns_table() {
    let env = env();
    let is_table: bool = env.eval("return type(GetFonts()) == 'table'").unwrap();
    assert!(is_table);
}

#[test]
fn test_get_font_info_by_name() {
    let env = env();
    let height: f64 = env
        .eval(
            r#"
            CreateFont('InfoFont')
            InfoFont:SetFont("Fonts\\Test.ttf", 20)
            local info = GetFontInfo("InfoFont")
            return info.height
            "#,
        )
        .unwrap();
    assert_eq!(height, 20.0);
}

#[test]
fn test_get_font_info_by_object() {
    let env = env();
    let height: f64 = env
        .eval(
            r#"
            local f = CreateFont('ObjInfoFont')
            f:SetFont("Fonts\\Test.ttf", 18)
            local info = GetFontInfo(f)
            return info.height
            "#,
        )
        .unwrap();
    assert_eq!(height, 18.0);
}

// ============================================================================
// CreateFontFamily
// ============================================================================

#[test]
fn test_create_font_family() {
    let env = env();
    let (path, height): (String, f64) = env
        .eval(
            r#"
            local ff = CreateFontFamily("TestFamily", {
                {alphabet = "roman", file = "Fonts\\Custom.ttf", height = 14}
            })
            return ff:GetFont()
            "#,
        )
        .unwrap();
    assert_eq!(path, "Fonts\\Custom.ttf");
    assert_eq!(height, 14.0);
}

#[test]
fn test_create_font_family_registers_global() {
    let env = env();
    env.eval::<()>(
        r#"CreateFontFamily("FamilyGlobal", {{alphabet = "roman", file = "Fonts\\X.ttf", height = 10}})"#,
    )
    .unwrap();
    let exists: bool = env.eval("return FamilyGlobal ~= nil").unwrap();
    assert!(exists);
}

// ============================================================================
// Standard font objects
// ============================================================================

#[test]
fn test_standard_fonts_exist() {
    let env = env();
    for name in &[
        "GameFontNormal",
        "GameFontNormalSmall",
        "GameFontNormalLarge",
        "GameFontNormalMed1",
        "GameFontNormalMed2",
        "GameFontNormalMed3",
        "GameFontHighlight",
        "GameFontHighlightMedium",
        "GameFontHighlightMed2",
        "GameFontDisable",
        "GameTooltipHeaderText",
        "ObjectiveFont",
        "NumberFontNormal",
        "SystemFont_Med1",
        "GameTooltipText",
    ] {
        let exists: bool = env.eval(&format!("return {} ~= nil", name)).unwrap();
        assert!(exists, "{} should exist", name);
    }
}

#[test]
fn test_standard_font_has_methods() {
    let env = env();
    let (path, height): (String, f64) = env
        .eval("local p, h = GameFontNormal:GetFont(); return p, h")
        .unwrap();
    assert_eq!(path, "Fonts\\FRIZQT__.TTF");
    assert_eq!(height, 12.0);
}

#[test]
fn test_standard_font_gold_color() {
    let env = env();
    let (r, g, _b, _a): (f64, f64, f64, f64) =
        env.eval("return GameFontNormal:GetTextColor()").unwrap();
    assert_eq!(r, 1.0);
    assert!((g - 0.82).abs() < 0.01);
}

#[test]
fn test_named_medium_font_objects_work_with_set_font_object() {
    let env = env();
    let (height, r, g): (f64, f64, f64) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", "NamedMediumFontProbe", UIParent)
            local fs = frame:CreateFontString(nil, "ARTWORK")
            fs:SetFontObject("GameFontNormalMed1")
            local _, height = fs:GetFont()
            local r, g = fs:GetTextColor()
            return height, r, g
            "#,
        )
        .unwrap();

    assert_eq!(height, 13.0);
    assert_eq!(r, 1.0);
    assert!((g - 0.82).abs() < 0.01);
}

// ============================================================================
// MessageFrame / EditBox auto-created FontObject
// ============================================================================

#[test]
fn test_messageframe_get_font_object_auto_creates() {
    let env = env();
    let is_table: bool = env
        .eval(
            r#"
            local mf = CreateFrame('MessageFrame')
            local fo = mf:GetFontObject()
            return type(fo) == 'table'
            "#,
        )
        .unwrap();
    assert!(
        is_table,
        "MessageFrame:GetFontObject() should auto-create a Font object"
    );
}

#[test]
fn test_editbox_get_font_object_auto_creates() {
    let env = env();
    let is_table: bool = env
        .eval(
            r#"
            local eb = CreateFrame('EditBox')
            local fo = eb:GetFontObject()
            return type(fo) == 'table'
            "#,
        )
        .unwrap();
    assert!(
        is_table,
        "EditBox:GetFontObject() should auto-create a Font object"
    );
}

#[test]
fn test_set_font_object_nil_errors() {
    let env = env();
    let errors: bool = env
        .eval(
            r#"
            local mf = CreateFrame('MessageFrame')
            local ok = pcall(mf.SetFontObject, mf, nil)
            return not ok
            "#,
        )
        .unwrap();
    assert!(errors, "SetFontObject(nil) should throw an error");
}

// ============================================================================
// XML Font table: SetFontObject / GetFontObject
// ============================================================================

#[test]
fn test_create_font_set_font_object_by_name() {
    let env = env();
    let got_it: bool = env
        .eval(
            r#"
            local src = CreateFont('NameTarget')
            src:SetFont("Fonts/Big.ttf", 24)
            local dst = CreateFont('NameCaller')
            dst:SetFontObject("NameTarget")
            return dst:GetFontObject() == src
            "#,
        )
        .unwrap();
    assert!(got_it, "SetFontObject(name) should resolve and store");
}

#[test]
fn test_xml_font_set_font_object_by_table() {
    let env = env();
    let got_it: bool = env
        .eval(
            r#"
            local src = CreateFont('TableTarget')
            src:SetFont("Fonts/Table.ttf", 18)
            local dst = CreateFont('TableCaller')
            dst:SetFontObject(src)
            return dst:GetFontObject() == src
            "#,
        )
        .unwrap();
    assert!(got_it, "SetFontObject(table) should store and retrieve");
}

#[test]
fn test_xml_font_get_object_type() {
    let env = env();
    let obj_type: String = env.eval("return GameFontNormal:GetObjectType()").unwrap();
    assert_eq!(obj_type, "Font");
}

#[test]
fn test_xml_font_is_object_type() {
    let env = env();
    let (is_font, is_frame): (bool, bool) = env
        .eval(
            r#"
            return GameFontNormal:IsObjectType("Font"),
                   GameFontNormal:IsObjectType("Frame")
            "#,
        )
        .unwrap();
    assert!(is_font);
    assert!(!is_frame);
}

// ============================================================================
// FontString:SetFixedColor (Blizzard_AccessibilityTemplates dependency)
// ============================================================================

#[test]
fn test_fontstring_set_fixed_color_is_callable() {
    let env = env();
    let ok: bool = env
        .eval(
            r#"
            local parent = CreateFrame("Frame", "SetFixedColorParent")
            local fs = parent:CreateFontString(nil, "OVERLAY", "GameFontNormal")
            return type(fs.SetFixedColor) == "function"
            "#,
        )
        .unwrap();
    assert!(ok, "FontString should expose SetFixedColor");
}

#[test]
fn test_fontstring_set_fixed_color_does_not_error() {
    let env = env();
    env.eval::<()>(
        r#"
        local parent = CreateFrame("Frame", "SetFixedColorNoError")
        local fs = parent:CreateFontString(nil, "OVERLAY", "GameFontNormal")
        fs:SetFixedColor(true)
        fs:SetFixedColor(false)
        "#,
    )
    .unwrap();
}

#[test]
fn test_fontstring_set_fixed_color_then_set_text_color_still_applies() {
    // Real WoW: SetFixedColor(true) locks against state-driven recoloring,
    // but explicit SetTextColor calls still apply. AccessibilityTemplates
    // and QuestFrame both rely on this ordering.
    let env = env();
    let (r, g, b, a): (f64, f64, f64, f64) = env
        .eval(
            r#"
            local parent = CreateFrame("Frame", "SetFixedColorThenSetTextColor")
            local fs = parent:CreateFontString(nil, "OVERLAY", "GameFontNormal")
            fs:SetFixedColor(true)
            fs:SetTextColor(0.25, 0.5, 0.75, 1.0)
            return fs:GetTextColor()
            "#,
        )
        .unwrap();
    assert_eq!((r, g, b, a), (0.25, 0.5, 0.75, 1.0));
}

#[test]
fn test_fontstring_set_fixed_color_accepts_non_bool_arg_as_false() {
    // SetFixedColor takes a bool per API doc; non-true values clear the flag.
    // This matches WoW's loose argument coercion for boolean params.
    let env = env();
    env.eval::<()>(
        r#"
        local parent = CreateFrame("Frame", "SetFixedColorNonBool")
        local fs = parent:CreateFontString(nil, "OVERLAY", "GameFontNormal")
        fs:SetFixedColor(nil)
        "#,
    )
    .unwrap();
}

#[test]
fn test_accessibility_templates_set_fixed_color_pattern() {
    // Mirror UIThemeContainerMixin:UpdateFontStrings call shape:
    // for each registered fontString, SetFixedColor then SetTextColor.
    let env = env();
    let (r, g, b): (f64, f64, f64) = env
        .eval(
            r#"
            local parent = CreateFrame("Frame", "AccessibilityFixedColorPattern")
            local fs = parent:CreateFontString(nil, "OVERLAY", "GameFontNormal")
            local fixedColor = true
            fs:SetFixedColor(fixedColor)
            fs:SetTextColor(0.9, 0.8, 0.7)
            local cr, cg, cb = fs:GetTextColor()
            return cr, cg, cb
            "#,
        )
        .unwrap();
    assert!((r - 0.9).abs() < 1e-6);
    assert!((g - 0.8).abs() < 1e-6);
    assert!((b - 0.7).abs() < 1e-6);
}

// ============================================================================
// FontString:SetFont 12.0.5 shape: numeric height, nilable flags, success return
// ============================================================================

#[test]
fn font_string_set_font_returns_exactly_one_success_boolean() {
    let env = env();
    env.exec(
        r#"
        local fs = CreateFrame("Frame"):CreateFontString()
        local path = "Fonts\\FRIZQT__.TTF"
        local function check(...)
            assert(select('#', ...) == 1, "one result")
            assert((...) == true, "success")
        end
        check(fs:SetFont(path, 16, "OUTLINE"))
        check(fs:SetFont(path, 16, "OUTLINE"))
        local got, height, flags = fs:GetFont()
        assert(got == path and height == 16 and flags == "OUTLINE")
        local function refuse(...)
            assert(select('#', ...) == 1 and (...) == false, "one false")
        end
        refuse(fs:SetFont(nil, 20, ""))
        refuse(fs:SetFont({}, 20, ""))
        got, height, flags = fs:GetFont()
        assert(got == path and height == 16 and flags == "OUTLINE", "refused call changes nothing")
        "#,
    )
    .unwrap();
}

#[test]
fn font_string_set_font_accepts_omitted_and_nil_flags() {
    let env = env();
    env.exec(
        r#"
        local fs = CreateFrame("Frame"):CreateFontString()
        local path = "Fonts\\FRIZQT__.TTF"
        assert(fs:SetFont(path, 16, "OUTLINE") == true)
        assert(fs:SetFont(path, 18) == true)
        local _, height, flags = fs:GetFont()
        assert(height == 18 and flags == "OUTLINE", "omitted flags keep the outline")
        assert(fs:SetFont(path, 20, nil) == true)
        _, height, flags = fs:GetFont()
        assert(height == 20 and flags == "OUTLINE", "nil flags keep the outline")
        assert(fs:SetFont(path, 20, "") == true)
        _, height, flags = fs:GetFont()
        assert(height == 20 and flags == "", "empty flags clear the outline")
        "#,
    )
    .unwrap();
}

#[test]
fn font_string_set_font_height_is_a_plain_number() {
    let env = env();
    env.exec(
        r#"
        local fs = CreateFrame("Frame"):CreateFontString()
        local path = "Fonts\\FRIZQT__.TTF"
        for _, height in ipairs({13.2, 0.5, 40}) do
            assert(fs:SetFont(path, height, nil) == true)
            local _, got = fs:GetFont()
            assert(type(got) == "number" and math.abs(got - height) < 0.00001, "height " .. height)
        end
        "#,
    )
    .unwrap();
}
