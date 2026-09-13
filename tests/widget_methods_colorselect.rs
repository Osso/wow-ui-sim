//! Tests for ColorSelect and StatusBar widget methods.

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
#[cfg(feature = "retail-12-0-0")]
fn statusbar_fill_style_enum_numeric_values() {
    WowLuaEnv::new()
        .unwrap()
        .exec(
            r#"
            local styles = Enum.StatusBarFillStyle
            for name, expected in pairs({Standard = 0, StandardNoRangeFill = 1, Center = 2, Reverse = 3}) do
                assert(type(styles[name]) == "number", name .. " must be numeric")
                assert(styles[name] == expected, name .. " has wrong enum value")
            end
            "#,
        )
        .unwrap();
}

#[test]
#[cfg(feature = "retail-12-0-0")]
fn statusbar_fill_style_numeric_roundtrip_and_getter_arity() {
    let failures: String = WowLuaEnv::new()
        .unwrap()
        .eval(
            r#"
            local bar = CreateFrame("StatusBar")
            local failures = {}
            local function check(expected, ...)
                local actual = ...
                if select('#', ...) ~= 1 or type(actual) ~= "number" or actual ~= expected then
                    failures[#failures + 1] = string.format(
                        "style %d: expected one numeric return, got %d returns, %s %s",
                        expected, select('#', ...), type(actual), tostring(actual))
                end
            end
            for _, name in ipairs({"Standard", "StandardNoRangeFill", "Center", "Reverse"}) do
                local style = Enum.StatusBarFillStyle[name]
                bar:SetFillStyle(style)
                check(style, bar:GetFillStyle())
            end
            return table.concat(failures, "\n")
            "#,
        )
        .unwrap();
    assert_eq!(failures, "");
}

#[test]
#[cfg(feature = "retail-12-0-0")]
fn statusbar_fill_style_setter_returns_zero_values() {
    WowLuaEnv::new()
        .unwrap()
        .exec(
            r#"
            local bar = CreateFrame("StatusBar")
            for _, name in ipairs({"Standard", "StandardNoRangeFill", "Center", "Reverse"}) do
                assert(select('#', bar:SetFillStyle(Enum.StatusBarFillStyle[name])) == 0,
                    name .. " setter must return zero values")
            end
            "#,
        )
        .unwrap();
}

#[test]
#[cfg(feature = "retail-12-0-0")]
fn statusbar_fill_style_later_mutation_and_instance_isolation() {
    WowLuaEnv::new()
        .unwrap()
        .exec(
            r#"
            local first = CreateFrame("StatusBar")
            local second = CreateFrame("StatusBar")
            local styles = Enum.StatusBarFillStyle
            first:SetFillStyle(styles.Center)
            second:SetFillStyle(styles.Reverse)
            assert(first:GetFillStyle() == styles.Center, "first explicitly set style lost")
            assert(second:GetFillStyle() == styles.Reverse, "second explicitly set style lost")
            first:SetFillStyle(styles.StandardNoRangeFill)
            assert(first:GetFillStyle() == styles.StandardNoRangeFill, "first mutation lost")
            assert(second:GetFillStyle() == styles.Reverse, "first mutation changed second")
            second:SetFillStyle(styles.Standard)
            assert(second:GetFillStyle() == styles.Standard, "second mutation lost")
            assert(first:GetFillStyle() == styles.StandardNoRangeFill, "second mutation changed first")
            "#,
        )
        .unwrap();
}

#[test]
#[cfg(feature = "retail-12-0-0")]
fn statusbar_fill_style_strict_numeric_validation_simulator_policy() {
    let failures: String = WowLuaEnv::new()
        .unwrap()
        .eval(
            r#"
            local bar = CreateFrame("StatusBar")
            local failures = {}
            local cases = {
                {"nil", nil}, {"legacy string", "REVERSE"}, {"numeric string", "2"},
                {"boolean", true}, {"table", {}}, {"negative", -1}, {"above range", 4},
                {"fraction", 1.5}, {"NaN", 0/0}, {"infinity", math.huge},
            }
            for _, case in ipairs(cases) do
                bar:SetFillStyle(Enum.StatusBarFillStyle.Center)
                local accepted = pcall(function() bar:SetFillStyle(case[2]) end)
                if accepted then
                    failures[#failures + 1] = case[1] .. " accepted against simulator policy"
                elseif bar:GetFillStyle() ~= Enum.StatusBarFillStyle.Center then
                    failures[#failures + 1] = case[1] .. " rejection changed prior valid style"
                end
            end
            return table.concat(failures, "\n")
            "#,
        )
        .unwrap();
    assert_eq!(failures, "");
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

    let (r, g, b): (f64, f64, f64) = env.eval("return TestCS:GetColorRGB()").unwrap();
    assert!((r - 0.5).abs() < 0.001);
    assert!((g - 0.6).abs() < 0.001);
    assert!((b - 0.7).abs() < 0.001);
}

#[test]
fn test_colorselect_rgb_defaults() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(r#"local cs = CreateFrame("ColorSelect", "TestCSDef", UIParent)"#)
        .unwrap();

    let (r, g, b): (f64, f64, f64) = env.eval("return TestCSDef:GetColorRGB()").unwrap();
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

    let (h, s, v): (f64, f64, f64) = env.eval("return TestCSHSV:GetColorHSV()").unwrap();
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
    let (r, g, b): (f64, f64, f64) = env.eval("return TestCSRed:GetColorRGB()").unwrap();
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
    let (r, g, b): (f64, f64, f64) = env.eval("return TestCSGreen:GetColorRGB()").unwrap();
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
    let (h, s, v): (f64, f64, f64) = env.eval("return TestCSConv:GetColorHSV()").unwrap();
    assert!(h.abs() < 0.01, "Hue for red should be ~0, got {}", h);
    assert!(
        (s - 1.0).abs() < 0.01,
        "Saturation for red should be 1, got {}",
        s
    );
    assert!(
        (v - 1.0).abs() < 0.01,
        "Value for red should be 1, got {}",
        v
    );
}

#[test]
fn test_colorselect_alpha_defaults_to_one_and_round_trips() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local cs = CreateFrame("ColorSelect", "TestCSAlpha", UIParent)
    "#,
    )
    .unwrap();

    let default_alpha: f64 = env.eval("return TestCSAlpha:GetColorAlpha()").unwrap();
    assert!(
        (default_alpha - 1.0).abs() < 0.001,
        "ColorSelect alpha should default to fully opaque"
    );

    env.exec("TestCSAlpha:SetColorAlpha(0.35)").unwrap();

    let alpha: f64 = env.eval("return TestCSAlpha:GetColorAlpha()").unwrap();
    assert!((alpha - 0.35).abs() < 0.001);
}

#[test]
fn test_colorselect_alpha_is_preserved_across_rgb_and_hsv_updates() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local cs = CreateFrame("ColorSelect", "TestCSAlphaPreserve", UIParent)
        cs:SetColorRGB(0.2, 0.3, 0.4)
        cs:SetColorAlpha(0.6)
        cs:SetColorHSV(120, 0.5, 0.8)
    "#,
    )
    .unwrap();

    let (alpha, r, g, b, h, s, v): (f64, f64, f64, f64, f64, f64, f64) = env
        .eval(
            r#"
            local r, g, b = TestCSAlphaPreserve:GetColorRGB()
            local h, s, v = TestCSAlphaPreserve:GetColorHSV()
            return TestCSAlphaPreserve:GetColorAlpha(), r, g, b, h, s, v
            "#,
        )
        .unwrap();

    assert!(
        (alpha - 0.6).abs() < 0.001,
        "SetColorHSV should not wipe stored alpha"
    );
    assert!((r - 0.4).abs() < 0.01);
    assert!((g - 0.8).abs() < 0.01);
    assert!((b - 0.4).abs() < 0.01);
    assert!((h - 120.0).abs() < 0.01);
    assert!((s - 0.5).abs() < 0.01);
    assert!((v - 0.8).abs() < 0.01);

    env.exec("TestCSAlphaPreserve:SetColorRGB(0.9, 0.1, 0.2)")
        .unwrap();
    let (alpha_after_rgb, r_after_rgb, g_after_rgb, b_after_rgb): (f64, f64, f64, f64) = env
        .eval(
            r#"
            local r, g, b = TestCSAlphaPreserve:GetColorRGB()
            return TestCSAlphaPreserve:GetColorAlpha(), r, g, b
            "#,
        )
        .unwrap();

    assert!(
        (alpha_after_rgb - 0.6).abs() < 0.001,
        "SetColorRGB should not wipe stored alpha"
    );
    assert!((r_after_rgb - 0.9).abs() < 0.01);
    assert!((g_after_rgb - 0.1).abs() < 0.01);
    assert!((b_after_rgb - 0.2).abs() < 0.01);
}

#[test]
fn test_colorselect_texture_getters_default_nil() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(r#"local cs = CreateFrame("ColorSelect", "TestCSTexNil", UIParent)"#)
        .unwrap();

    let getters_are_nil: (bool, bool, bool, bool, bool, bool) = env
        .eval(
            r#"
            return TestCSTexNil:GetColorAlphaTexture() == nil,
                   TestCSTexNil:GetColorAlphaThumbTexture() == nil,
                   TestCSTexNil:GetColorValueTexture() == nil,
                   TestCSTexNil:GetColorValueThumbTexture() == nil,
                   TestCSTexNil:GetColorWheelTexture() == nil,
                   TestCSTexNil:GetColorWheelThumbTexture() == nil
            "#,
        )
        .unwrap();

    assert_eq!(getters_are_nil, (true, true, true, true, true, true));
}

#[test]
fn test_colorselect_primary_texture_setters_create_child_textures() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local cs = CreateFrame("ColorSelect", "TestCSTex", UIParent)
        cs:SetColorAlphaTexture("Interface\\Buttons\\WHITE8X8")
        cs:SetColorValueTexture("Interface\\Buttons\\WHITE8X8")
        cs:SetColorWheelTexture("Interface\\Buttons\\WHITE8X8")
    "#,
    )
    .unwrap();

    let result: (String, String, String, String, String, String) = env
        .eval(
            r#"
            return TestCSTex:GetColorAlphaTexture():GetObjectType(),
                   TestCSTex:GetColorAlphaTexture():GetParent():GetName(),
                   TestCSTex:GetColorValueTexture():GetObjectType(),
                   TestCSTex:GetColorValueTexture():GetParent():GetName(),
                   TestCSTex:GetColorWheelTexture():GetObjectType(),
                   TestCSTex:GetColorWheelTexture():GetParent():GetName()
            "#,
        )
        .unwrap();

    assert_eq!(result.0, "Texture");
    assert_eq!(result.1, "TestCSTex");
    assert_eq!(result.2, "Texture");
    assert_eq!(result.3, "TestCSTex");
    assert_eq!(result.4, "Texture");
    assert_eq!(result.5, "TestCSTex");
}

#[test]
fn test_colorselect_thumb_texture_setters_round_trip_userdata() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local cs = CreateFrame("ColorSelect", "TestCSThumbs", UIParent)
        TestCSAlphaThumb = cs:CreateTexture(nil, "ARTWORK")
        TestCSValueThumb = cs:CreateTexture(nil, "ARTWORK")
        TestCSWheelThumb = cs:CreateTexture(nil, "ARTWORK")
        cs:SetColorAlphaThumbTexture(TestCSAlphaThumb)
        cs:SetColorValueThumbTexture(TestCSValueThumb)
        cs:SetColorWheelThumbTexture(TestCSWheelThumb)
    "#,
    )
    .unwrap();

    let result: (bool, bool, bool) = env
        .eval(
            r#"
            return TestCSThumbs:GetColorAlphaThumbTexture() == TestCSAlphaThumb,
                   TestCSThumbs:GetColorValueThumbTexture() == TestCSValueThumb,
                   TestCSThumbs:GetColorWheelThumbTexture() == TestCSWheelThumb
            "#,
        )
        .unwrap();

    assert_eq!(result, (true, true, true));
}

#[test]
fn test_colorselect_clear_color_wheel_texture_clears_getter() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local cs = CreateFrame("ColorSelect", "TestCSClearWheel", UIParent)
        cs:SetColorWheelTexture("Interface\\Buttons\\WHITE8X8")
        TestCSWheelBeforeClear = cs:GetColorWheelTexture()
        cs:ClearColorWheelTexture()
    "#,
    )
    .unwrap();

    let result: (bool, bool) = env
        .eval(
            r#"
            return TestCSWheelBeforeClear ~= nil,
                   TestCSClearWheel:GetColorWheelTexture() == nil
            "#,
        )
        .unwrap();
    assert!(
        result.0,
        "SetColorWheelTexture should create a retrievable wheel texture before clear"
    );
    assert!(result.1, "Cleared color wheel getter should return nil");
}

#[test]
fn test_statusbar_texture_and_color_methods_still_resolve() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local sb = CreateFrame("StatusBar", "TestStatusBarMethods", UIParent)
        sb:SetStatusBarTexture("Interface\\Buttons\\WHITE8X8")
        sb:SetStatusBarColor(0.1, 0.2, 0.3, 0.4)
        sb:SetFillStyle("REVERSE")
    "#,
    )
    .unwrap();

    let has_texture: bool = env
        .eval("return type(TestStatusBarMethods:GetStatusBarTexture()) == 'table'")
        .unwrap();
    assert!(
        has_texture,
        "StatusBar should expose StatusBarTexture child"
    );

    let (r, g, b, a): (f32, f32, f32, f32) = env
        .eval("return TestStatusBarMethods:GetStatusBarColor()")
        .unwrap();
    assert!((r - 0.1).abs() < 0.001);
    assert!((g - 0.2).abs() < 0.001);
    assert!((b - 0.3).abs() < 0.001);
    assert!((a - 0.4).abs() < 0.001);
}

#[test]
fn test_statusbar_set_color_fill_aliases_statusbar_color_state() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local sb = CreateFrame("StatusBar", "TestStatusBarColorFill", UIParent)
        sb:SetStatusBarTexture("Interface\\Buttons\\WHITE8X8")
        sb:SetColorFill(0.6, 0.5, 0.4, 0.3)
    "#,
    )
    .unwrap();

    let (r, g, b, a): (f32, f32, f32, f32) = env
        .eval("return TestStatusBarColorFill:GetStatusBarColor()")
        .unwrap();
    assert!((r - 0.6).abs() < 0.001);
    assert!((g - 0.5).abs() < 0.001);
    assert!((b - 0.4).abs() < 0.001);
    assert!((a - 0.3).abs() < 0.001);
}

#[test]
fn test_statusbar_texture_accepts_atlas_names() {
    let env = WowLuaEnv::new().unwrap();

    let result: (String, String) = env
        .eval(
            r#"
            local sb = CreateFrame("StatusBar", "TestStatusBarAtlasTexture", UIParent)
            sb:SetStatusBarTexture("UI-HUD-UnitFrame-Player-PortraitOn-Bar-Mana")
            local texture = sb:GetStatusBarTexture()
            return texture:GetAtlas(), texture:GetTextureFilePath()
        "#,
        )
        .unwrap();

    assert_eq!(result.0, "UI-HUD-UnitFrame-Player-PortraitOn-Bar-Mana");
    assert!(
        result.1.starts_with("Interface\\"),
        "atlas-backed status bar should resolve to a concrete texture path"
    );
    assert_ne!(
        result.1, result.0,
        "atlas-backed status bar should not keep the atlas name as a raw texture path"
    );
}

#[test]
fn test_statusbar_texture_userdata_preserves_existing_atlas_source() {
    let env = WowLuaEnv::new().unwrap();

    let result: (String, String) = env
        .eval(
            r#"
            local sb = CreateFrame("StatusBar", "TestStatusBarTextureUserdata", UIParent)
            local tex = sb:CreateTexture(nil, "ARTWORK")
            tex:SetAtlas("UI-HUD-UnitFrame-Party-PortraitOn-Bar-Mana")
            sb:SetStatusBarTexture(tex)
            local adopted = sb:GetStatusBarTexture()
            return adopted:GetAtlas() or "", adopted:GetTextureFilePath() or ""
        "#,
        )
        .unwrap();

    assert_eq!(
        result.0, "UI-HUD-UnitFrame-Party-PortraitOn-Bar-Mana",
        "SetStatusBarTexture(existingTexture) should preserve an atlas-backed child texture"
    );
    assert!(
        result.1.starts_with("Interface\\"),
        "atlas-backed adopted status bar texture should retain a resolved texture file path"
    );
}

#[test]
fn test_statusbar_interpolation_methods_track_target_and_displayed_value() {
    let env = WowLuaEnv::new().unwrap();

    let result: (f64, f64, bool, f64, bool) = env
        .eval(
            r#"
            local sb = CreateFrame("StatusBar", "TestStatusBarInterpolation", UIParent)
            sb:SetMinMaxValues(0, 1)
            sb:SetValue(0.25)
            sb:SetValue(0.75, "Smooth")
            local targetValue = sb:GetValue()
            local displayedValue = sb:GetInterpolatedValue()
            local isInterpolating = sb:IsInterpolating()
            sb:SetToTargetValue()
            local snappedValue = sb:GetInterpolatedValue()
            local isStillInterpolating = sb:IsInterpolating()
            return targetValue, displayedValue, isInterpolating, snappedValue, isStillInterpolating
        "#,
        )
        .unwrap();

    assert!((result.0 - 0.75).abs() < 0.001);
    assert!(
        (result.1 - 0.25).abs() < 0.001,
        "displayed value should remain at the previous bar value until snapped"
    );
    assert!(result.2, "status bar should report active interpolation");
    assert!((result.3 - 0.75).abs() < 0.001);
    assert!(
        !result.4,
        "SetToTargetValue should finish interpolation and clear the flag"
    );
}

#[test]
fn test_statusbar_set_min_max_values_clamps_existing_value() {
    let env = WowLuaEnv::new().unwrap();

    let result: (f64, f64, f64, f64, f64, f64) = env
        .eval(
            r#"
            local sb = CreateFrame("StatusBar", "TestStatusBarMinMax", UIParent)
            sb:SetMinMaxValues(0, 100)
            sb:SetValue(80)

            sb:SetMinMaxValues(10, 50)
            local upperMin, upperMax = sb:GetMinMaxValues()
            local upperValue = sb:GetValue()

            sb:SetValue(20)
            sb:SetMinMaxValues(30, 40)
            local lowerMin, lowerMax = sb:GetMinMaxValues()
            local lowerValue = sb:GetValue()
            return upperMin, upperMax, upperValue, lowerMin, lowerMax, lowerValue
        "#,
        )
        .unwrap();

    assert_eq!((result.0, result.1, result.2), (10.0, 50.0, 50.0));
    assert_eq!((result.3, result.4, result.5), (30.0, 40.0, 30.0));
}

#[test]
fn test_statusbar_desaturation_methods_share_persisted_state() {
    let env = WowLuaEnv::new().unwrap();

    let result: (f64, bool, bool, f64, bool, bool) = env
        .eval(
            r#"
            local sb = CreateFrame("StatusBar", "TestStatusBarDesaturation", UIParent)
            sb:SetStatusBarTexture("Interface\\Buttons\\WHITE8X8")
            local texture = sb:GetStatusBarTexture()

            sb:SetStatusBarDesaturation(0.4)
            local normalized = sb:GetStatusBarDesaturation()
            local isDesaturated = sb:IsStatusBarDesaturated()
            local textureIsDesaturated = texture:IsDesaturated()

            sb:SetStatusBarDesaturated(false)
            local clearedNormalized = sb:GetStatusBarDesaturation()
            local clearedBool = sb:IsStatusBarDesaturated()
            local clearedTexture = texture:IsDesaturated()

            return normalized, isDesaturated, textureIsDesaturated, clearedNormalized, clearedBool, clearedTexture
        "#,
        )
        .unwrap();

    assert!((result.0 - 0.4).abs() < 0.001);
    assert!(
        result.1,
        "normalized desaturation should mark the status bar desaturated"
    );
    assert!(
        result.2,
        "status bar texture child should inherit desaturation"
    );
    assert_eq!(result.3, 0.0);
    assert!(
        !result.4,
        "bool state should clear through SetStatusBarDesaturated(false)"
    );
    assert!(
        !result.5,
        "texture state should clear through SetStatusBarDesaturated(false)"
    );
}

#[test]
fn test_statusbar_timer_duration_round_trips_duration_object() {
    let env = WowLuaEnv::new().unwrap();

    let result: (bool, String, bool) = env
        .eval(
            r#"
            local sb = CreateFrame("StatusBar", "TestStatusBarTimerDuration", UIParent)
            local duration = C_DurationUtil.CreateDuration()
            duration.debugTag = "statusbar-timer"
            sb:SetTimerDuration(duration, Enum.StatusBarInterpolation.Immediate, Enum.StatusBarTimerDirection.RemainingTime)
            local stored = sb:GetTimerDuration()
            return rawequal(duration, stored), stored.debugTag, type(stored) == "table"
        "#,
        )
        .unwrap();

    assert!(
        result.0,
        "GetTimerDuration should return the same duration object"
    );
    assert_eq!(result.1, "statusbar-timer");
    assert!(
        result.2,
        "status bar timer should stay a LuaDurationObject table"
    );
}
