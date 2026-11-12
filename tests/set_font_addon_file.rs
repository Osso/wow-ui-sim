//! VERIFIED (Retail 12.1.5.70688 and Forever 1.60.1.70707): a missing addon font reports
//! `Invalid font asset (<path>): file not found`. UNKNOWN: the exact SetFont owner and whether
//! a failed call preserves the previous font; the retention assertion remains optional.

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
#[test]
fn set_font_with_a_missing_addon_font_file_reports_native_detail() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    let (ok, err): (bool, String) = env
        .eval(
            r#"
            local fs = UIParent:CreateFontString()
            local ok, err = pcall(fs.SetFont, fs, "Interface\\AddOns\\NOSUCHADDON\\X.TTF", 16, "")
            return ok, tostring(err)
            "#,
        )
        .expect("missing font probe should execute");
    assert!(!ok, "SetFont should reject a missing font");
    assert!(
        err.ends_with("Invalid font asset (Interface\\AddOns\\NOSUCHADDON\\X.TTF): file not found"),
        "unexpected native error detail: {err}"
    );
}

// UNKNOWN: L2 does not prove that a failed native SetFont call keeps the old font.
#[cfg(feature = "optional-tests")]
#[test]
fn set_font_with_a_missing_addon_font_file_keeps_the_previous_font_unknown_native() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
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
        err.ends_with("Invalid font asset (Interface\\AddOns\\NoSuchAddon\\x.ttf): file not found"),
        "unexpected native error detail: {err}"
    );
    assert_eq!((path.as_str(), height), ("Fonts\\ARIALN.TTF", 12.0));
}
