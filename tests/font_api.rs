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
    let (path, height): (String, f64) = env
        .eval(
            r#"
            local f = CreateFont('TestDefault')
            local p, h = f:GetFont()
            return p, h
            "#,
        )
        .unwrap();
    assert_eq!(path, "Fonts\\FRIZQT__.TTF");
    assert_eq!(height, 12.0);
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

// ============================================================================
// GetFonts / GetFontInfo
// ============================================================================

#[test]
fn test_get_fonts_returns_table() {
    let env = env();
    let is_table: bool = env
        .eval("return type(GetFonts()) == 'table'")
        .unwrap();
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
        "GameFontHighlight",
        "GameFontDisable",
        "NumberFontNormal",
        "SystemFont_Med1",
        "GameTooltipText",
    ] {
        let exists: bool = env
            .eval(&format!("return {} ~= nil", name))
            .unwrap();
        assert!(exists, "{} should exist", name);
    }
}

#[test]
fn test_standard_font_has_methods() {
    let env = env();
    let (path, height, _flags): (String, f64, String) = env
        .eval("return GameFontNormal:GetFont()")
        .unwrap();
    assert_eq!(path, "Fonts\\FRIZQT__.TTF");
    assert_eq!(height, 12.0);
}

#[test]
fn test_standard_font_gold_color() {
    let env = env();
    let (r, g, _b, _a): (f64, f64, f64, f64) = env
        .eval("return GameFontNormal:GetTextColor()")
        .unwrap();
    assert_eq!(r, 1.0);
    assert!((g - 0.82).abs() < 0.01);
}
