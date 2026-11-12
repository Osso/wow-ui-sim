//! FontString:SetFont with a font file below Interface\AddOns that does not
//! exist: the client (WoW Forever 1.60.1) raises "Invalid font asset" and
//! keeps the font.
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn set_font_with_a_missing_addon_font_file_raises_the_client_error() {
    let env = WowLuaEnv::new().unwrap();
    let (ok, err, path, height): (bool, String, String, f64) = env
        .eval(
            r#"
            local fs = UIParent:CreateFontString()
            fs:SetFont("Fonts\\ARIALN.TTF", 12, "")
            local ok, err = pcall(fs.SetFont, fs, "Interface\\AddOns\\NoSuchAddon\\x.ttf", 16, "")
            local path, height = fs:GetFont()
            return ok, tostring(err), path, height
            "#,
        )
        .expect("missing font probe should execute");
    assert!(!ok, "SetFont should raise an error");
    assert!(
        err.ends_with(
            "FontString:SetFont(): Invalid font asset (Interface\\AddOns\\NoSuchAddon\\x.ttf): file not found"
        ),
        "unexpected error: {err}"
    );
    assert_eq!((path.as_str(), height), ("Fonts\\ARIALN.TTF", 12.0));
}
