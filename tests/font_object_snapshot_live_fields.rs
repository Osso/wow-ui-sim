//! `SetFontObject` snapshots the font object's fields into the FontString. A
//! font object carries two sets: `__font` / `__height` / `__outline`, written
//! by the XML loader with a default height of 12 for fonts that inherit their
//! size, and `__fontPath` / `__fontHeight` / `__fontFlags`, kept by `SetFont`
//! and the inheritance copy and read back by `GetFont`. The snapshot preferred
//! the loader fields, so `GameFontNormalSmall` reported 10 through `GetFont`
//! while every FontString using it rendered at 12.

use crate::common::blizzard_addon_harness::with_blizzard_addon_closure;
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn set_font_object_takes_the_size_get_font_reports() {
    let env = WowLuaEnv::new().expect("env");
    let (object_size, string_size): (f64, f64) = env
        .eval(
            r#"
            local font = CreateFont("SnapshotProbeFont")
            font:SetFont("Fonts\\FRIZQT__.TTF", 10, "")
            font.__height = 12  -- the stale loader default an inheriting <Font> keeps
            local fs = UIParent:CreateFontString(nil, "OVERLAY")
            fs:SetFontObject(font)
            return select(2, font:GetFont()), select(2, fs:GetFont())
            "#,
        )
        .expect("font probe");
    assert_eq!(object_size, 10.0);
    assert_eq!(string_size, 10.0, "the FontString must take the size the font object reports");
}

#[test]
fn blizzard_small_fonts_reach_font_strings_at_ten() {
    with_blizzard_addon_closure(&["Blizzard_Fonts_Shared"], &[], |env, _| {
        let (small, number, via_inherits): (f64, f64, f64) = env
            .eval(
                r#"
                local a = UIParent:CreateFontString(nil, "OVERLAY"); a:SetFontObject(GameFontNormalSmall)
                local b = UIParent:CreateFontString(nil, "OVERLAY"); b:SetFontObject(GameNormalNumberFont)
                local c = UIParent:CreateFontString(nil, "OVERLAY", "GameFontNormalSmall")
                return select(2, a:GetFont()), select(2, b:GetFont()), select(2, c:GetFont())
                "#,
            )
            .expect("font strings");
        assert_eq!(small, 10.0, "GameFontNormalSmall is FRIZQT 10 (SystemFont_Shadow_Small)");
        assert_eq!(number, 10.0, "GameNormalNumberFont is FRIZQT 10 (NumberFont_GameNormal)");
        assert_eq!(via_inherits, 10.0, "CreateFontString(..., inherits) takes the same size");
    });
}
