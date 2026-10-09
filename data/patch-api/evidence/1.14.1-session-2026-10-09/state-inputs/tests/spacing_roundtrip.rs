//! `SetSpacing` / `GetSpacing` round-trip.
//!
//! The value is stored on `Frame.text_line_spacing` (for FontString /
//! EditBox widgets) or the font-object `__spacing` slot (for
//! GameFontNormal-style Font tables). These tests cover API round trips
//! and rendered text measurement without loading Blizzard UI.

use std::cell::RefCell;
use std::rc::Rc;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::render::font::WowFontSystem;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("WowLuaEnv init")
}

#[test]
fn font_string_spacing_defaults_to_zero() {
    let env = env();
    let spacing: f64 = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", nil, UIParent)
            local fs = frame:CreateFontString(nil, "ARTWORK", "GameFontNormal")
            return fs:GetSpacing()
            "#,
        )
        .unwrap();
    assert_eq!(spacing, 0.0);
}

#[test]
fn font_string_set_spacing_round_trips() {
    let env = env();
    let spacing: f64 = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", nil, UIParent)
            local fs = frame:CreateFontString(nil, "ARTWORK", "GameFontNormal")
            fs:SetSpacing(7.5)
            return fs:GetSpacing()
            "#,
        )
        .unwrap();
    assert!((spacing - 7.5).abs() < 1e-4, "got {spacing}");
}

#[test]
fn edit_box_set_spacing_round_trips() {
    let env = env();
    let spacing: f64 = env
        .eval(
            r#"
            local eb = CreateFrame("EditBox", nil, UIParent)
            eb:SetSpacing(3)
            return eb:GetSpacing()
            "#,
        )
        .unwrap();
    assert_eq!(spacing, 3.0);
}

#[test]
fn font_object_set_spacing_round_trips() {
    let env = env();
    let spacing: f64 = env
        .eval(
            r#"
            GameFontNormal:SetSpacing(4.25)
            return GameFontNormal:GetSpacing()
            "#,
        )
        .unwrap();
    assert!((spacing - 4.25).abs() < 1e-4, "got {spacing}");
}

#[test]
fn font_string_and_font_object_are_independent() {
    let env = env();
    let (fs_val, font_val): (f64, f64) = env
        .eval(
            r#"
            -- Use a fresh font so prior tests in this file don't leak state.
            local MyFont = CreateFont("SpacingIndependenceFont")
            MyFont:SetSpacing(11)
            local frame = CreateFrame("Frame", nil, UIParent)
            local fs = frame:CreateFontString(nil, "ARTWORK", "SpacingIndependenceFont")
            fs:SetSpacing(2)
            return fs:GetSpacing(), MyFont:GetSpacing()
            "#,
        )
        .unwrap();
    assert_eq!(fs_val, 2.0, "FontString value");
    assert_eq!(font_val, 11.0, "Font object value");
}

#[test]
fn font_string_creation_snapshots_explicit_font_object_spacing() {
    let env = env();
    env.set_font_system(Rc::new(RefCell::new(WowFontSystem::new_without_casc())));
    let (baseline, spaced, auto_height, spacing): (f64, f64, f64, f64) = env
        .eval(
            r#"
            local plain = CreateFont("PlainSpacingSnapshotFont")
            plain:SetFont("Fonts\\FRIZQT__.TTF", 16)
            local font = CreateFont("SpacedSnapshotFont")
            font:CopyFontObject(plain)
            font:SetSpacing(6)
            local parent = CreateFrame("Frame", nil, UIParent)
            local control = parent:CreateFontString(nil, "ARTWORK", "PlainSpacingSnapshotFont")
            local fs = parent:CreateFontString(nil, "ARTWORK", "SpacedSnapshotFont")
            control:SetText("H\nH")
            fs:SetText("H\nH")
            return control:GetStringHeight(), fs:GetStringHeight(), fs:GetHeight(), fs:GetSpacing()
            "#,
        )
        .unwrap();
    assert!(baseline > 0.0, "must shape real text");
    assert!((spaced - baseline - 6.0).abs() < 0.01, "{baseline} -> {spaced}");
    assert!((auto_height - spaced).abs() < 0.01);
    assert_eq!(spacing, 6.0);
}

#[test]
fn set_font_object_snapshots_spacing_then_local_spacing_is_independent() {
    let env = env();
    env.set_font_system(Rc::new(RefCell::new(WowFontSystem::new_without_casc())));
    let (before, after, auto_height, inherited, local, font_spacing):
        (f64, f64, f64, f64, f64, f64) = env
        .eval(
            r#"
            local font = CreateFont("ExplicitSpacingSnapshotFont")
            font:SetFont("Fonts\\FRIZQT__.TTF", 16)
            font:SetSpacing(5)
            local fs = CreateFrame("Frame", nil, UIParent):CreateFontString(nil, "ARTWORK")
            fs:SetFont("Fonts\\FRIZQT__.TTF", 16)
            fs:SetText("H\nH")
            local before = fs:GetStringHeight()
            fs:SetFontObject(font)
            local after = fs:GetStringHeight()
            local auto_height = fs:GetHeight()
            local inherited = fs:GetSpacing()
            fs:SetSpacing(2)
            return before, after, auto_height, inherited, fs:GetSpacing(), font:GetSpacing()
            "#,
        )
        .unwrap();
    assert!(before > 0.0, "must shape real text");
    assert!((after - before - 5.0).abs() < 0.01, "{before} -> {after}");
    assert!((auto_height - after).abs() < 0.01);
    assert_eq!(inherited, 5.0);
    assert_eq!(local, 2.0);
    assert_eq!(font_spacing, 5.0);
}

#[test]
fn fontstring_spacing_changes_multiline_height_but_not_single_line() {
    let env = env();
    env.set_font_system(Rc::new(RefCell::new(WowFontSystem::new_without_casc())));
    let (before, after, single_before, single_after, lines): (f64, f64, f64, f64, f64) = env
        .eval(
            r#"
            local fs = CreateFrame("Frame", nil, UIParent):CreateFontString(nil, "ARTWORK")
            fs:SetFont("Fonts\\FRIZQT__.TTF", 16)
            fs:SetText("H\nH")
            local before = fs:GetStringHeight()
            fs:SetSpacing(6)
            local after = fs:GetStringHeight()
            local lines = fs:GetNumLines()
            fs:SetText("H")
            local single_after = fs:GetStringHeight()
            fs:SetSpacing(0)
            return before, after, fs:GetStringHeight(), single_after, lines
            "#,
        )
        .unwrap();
    assert!(before > 0.0, "must shape real text");
    assert!((after - before - 6.0).abs() < 0.01, "{before} -> {after}");
    assert_eq!(single_before, single_after);
    assert_eq!(lines, 2.0);
}

#[test]
fn fontstring_spacing_keeps_line_count_without_font_system() {
    let env = env();
    let (before, after): (f64, f64) = env
        .eval(
            r#"
            local fs = CreateFrame("Frame", nil, UIParent):CreateFontString(nil, "ARTWORK")
            fs:SetFont("Fonts\\FRIZQT__.TTF", 16)
            fs:SetText("H\nH")
            local before = fs:GetNumLines()
            fs:SetSpacing(6)
            return before, fs:GetNumLines()
            "#,
        )
        .unwrap();
    assert_eq!(before, 2.0);
    assert_eq!(after, 2.0);
}

#[test]
fn fontstring_spacing_updates_auto_text_height() {
    let env = env();
    env.set_font_system(Rc::new(RefCell::new(WowFontSystem::new_without_casc())));
    let (before, after, measured): (f64, f64, f64) = env
        .eval(
            r#"
            local fs = CreateFrame("Frame", nil, UIParent):CreateFontString(nil, "ARTWORK")
            fs:SetFont("Fonts\\FRIZQT__.TTF", 16)
            fs:SetText("H\nH")
            local before = fs:GetHeight()
            fs:SetSpacing(6)
            return before, fs:GetHeight(), fs:GetStringHeight()
            "#,
        )
        .unwrap();
    assert!(before > 0.0);
    assert!((after - before - 6.0).abs() < 0.01, "{before} -> {after}");
    assert!((after - measured).abs() < 0.01);
}

#[test]
fn fontstring_spacing_changes_wrapped_height_with_text_scale() {
    let env = env();
    let fonts = Rc::new(RefCell::new(WowFontSystem::new_without_casc()));
    let width = fonts.borrow_mut().measure_text_width("H ", None, 16.0) + 1.0;
    env.set_font_system(fonts);
    let (before, after, unwrapped, after_unwrapped): (f64, f64, f64, f64) = env
        .eval(&format!(
            r#"
            local fs = CreateFrame("Frame", nil, UIParent):CreateFontString(nil, "ARTWORK")
            fs:SetFont("Fonts\\FRIZQT__.TTF", 16)
            fs:SetText("H H")
            fs:SetWidth({width})
            fs:SetTextScale(1.5)
            local before = fs:GetStringHeight()
            fs:SetSpacing(6)
            local after = fs:GetStringHeight()
            fs:SetWidth(400)
            local after_unwrapped = fs:GetStringHeight()
            fs:SetSpacing(0)
            return before, after, fs:GetStringHeight(), after_unwrapped
            "#,
        ))
        .unwrap();
    assert!(before > 0.0);
    assert!((after - before - 9.0).abs() < 0.01, "{before} -> {after}");
    assert_eq!(unwrapped, after_unwrapped);
}

#[test]
fn spacing_accepts_zero_and_negative_values() {
    let env = env();
    let (zero, neg): (f64, f64) = env
        .eval(
            r#"
            local frame = CreateFrame("Frame", nil, UIParent)
            local fs = frame:CreateFontString(nil, "ARTWORK", "GameFontNormal")
            fs:SetSpacing(0)
            local z = fs:GetSpacing()
            fs:SetSpacing(-2.5)
            local n = fs:GetSpacing()
            return z, n
            "#,
        )
        .unwrap();
    assert_eq!(zero, 0.0);
    assert!((neg - (-2.5)).abs() < 1e-4, "got {neg}");
}
