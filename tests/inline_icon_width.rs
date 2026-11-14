//! Inline `|T`/`|A` icons take their width in measured text, as the client's
//! `GetStringWidth` reports it, and GameTooltip lines with icons widen the
//! tooltip by the icons' widths.
use wow_ui_sim::lua_api::WowLuaEnv;

fn widths(text_a: &str, text_b: &str) -> (f64, f64) {
    let env = WowLuaEnv::new().unwrap();
    env.eval(&format!(
        r#"
        local fs = UIParent:CreateFontString(nil, "ARTWORK")
        fs:SetFont("Fonts\\FRIZQT__.TTF", 12, "")
        fs:SetText({text_a:?})
        local a = fs:GetStringWidth()
        fs:SetText({text_b:?})
        return a, fs:GetStringWidth()
        "#
    ))
    .unwrap()
}

// VERIFIED R88e.6: accepted width contributes. The exact 20-unit size is INFERRED.
#[test]
fn atlas_icon_adds_its_width() {
    let (plain, with_icon) = widths("3 2", "3 |A:hud-backpack:14:20|a2");
    assert!(
        (with_icon - plain - 20.0).abs() < 0.01,
        "plain {plain}, with icon {with_icon}"
    );
}

// VERIFIED R88f.L.2: the absolute right edge, not the icon width, is snapped.
#[test]
fn atlas_icon_snaps_absolute_right_edge_before_following_text() {
    let env = WowLuaEnv::new().unwrap();
    let (prefix, suffix, total): (f64, f64, f64) = env
        .eval(
            r#"
            local fs = UIParent:CreateFontString(nil, "ARTWORK")
            fs:SetFont("Fonts\\FRIZQT__.TTF", 13.7, "")
            fs:SetTextScale(1.3)
            fs:SetText("W")
            local prefix = fs:GetStringWidth()
            fs:SetText("i")
            local suffix = fs:GetStringWidth()
            fs:SetText("W|A:hud-backpack:10:10.25|ai")
            return prefix, suffix, fs:GetStringWidth()
            "#,
        )
        .unwrap();
    let unsnapped_right_edge = prefix + 10.25 * 1.3;
    let expected = (unsnapped_right_edge + 0.5).floor() + suffix;
    assert!(
        (unsnapped_right_edge - unsnapped_right_edge.round()).abs() > 0.1,
        "fixture right edge must be fractional: {unsnapped_right_edge}"
    );
    assert!(
        (total - expected).abs() < 0.01,
        "prefix {prefix}, suffix {suffix}, total {total}, expected {expected}"
    );
}

// VERIFIED R88d: zero height uses the caller metric; mapping it to line height is INFERRED.
#[test]
fn texture_zero_height_uses_inferred_line_metric() {
    let env = WowLuaEnv::new().unwrap();
    let (plain, with_icon, line): (f64, f64, f64) = env
        .eval(
            r#"
            local fs = UIParent:CreateFontString(nil, "ARTWORK")
            fs:SetFont("Fonts\\FRIZQT__.TTF", 12, "")
            fs:SetText("ab")
            local a, h = fs:GetStringWidth(), fs:GetStringHeight()
            fs:SetText("a|TInterface\\Icons\\INV_Misc_QuestionMark:0|tb")
            return a, fs:GetStringWidth(), h
            "#,
        )
        .unwrap();
    assert!(
        line > 0.0 && (with_icon - plain - line).abs() < 0.01,
        "plain {plain}, with icon {with_icon}, line {line}"
    );
}

// VERIFIED R88d: a missing height rejects the token; its zero-width measurement is INFERRED.
#[test]
fn texture_icon_without_size_takes_no_room() {
    let (plain, with_icon) = widths("ab", "a|TInterface\\Icons\\INV_Misc_QuestionMark|tb");
    assert!(
        (with_icon - plain).abs() < 0.01,
        "plain {plain}, with icon {with_icon}"
    );
}

// VERIFIED R88d: width zero uses the caller metric; mapping it to line height is INFERRED.
#[test]
fn atlas_zero_width_uses_inferred_line_metric() {
    let (plain, with_icon) = widths("ab", "a|A:hud-backpack:24:0|ab");
    let (_, line) = widths("ab", "|A:hud-backpack:0:0|a");
    assert!(
        line > 0.0 && (with_icon - plain - line).abs() < 0.01,
        "plain {plain}, with icon {with_icon}, line {line}"
    );
}

// VERIFIED R88e.6: a taller accepted icon raises line height; exact size 32 is INFERRED.
#[test]
fn a_tall_icon_raises_the_line() {
    let env = WowLuaEnv::new().unwrap();
    let (text, tall): (f64, f64) = env
        .eval(
            r#"
            local fs = UIParent:CreateFontString(nil, "ARTWORK")
            fs:SetFont("Fonts\\FRIZQT__.TTF", 12, "")
            fs:SetText("X")
            local a = fs:GetStringHeight()
            fs:SetText("X|TInterface\\Icons\\INV_Misc_QuestionMark:32|t")
            return a, fs:GetStringHeight()
            "#,
        )
        .unwrap();
    assert!(
        text < 32.0 && (tall - 32.0).abs() < 0.01,
        "text {text}, tall {tall}"
    );
}

// VERIFIED R88e.6: accepted width contributes. The exact 110-unit size is INFERRED.
#[test]
fn texture_icon_width_overrides_its_height() {
    let (plain, with_icon) = widths(
        "x",
        "x|TInterface\\Buttons\\WHITE8X8:1:110:0:0:8:8:0:8:0:8:110:110:110|t",
    );
    assert!(
        (with_icon - plain - 110.0).abs() < 0.01,
        "plain {plain}, with icon {with_icon}"
    );
}

// INFERRED: the simulator's text-scale mapping retains the contributor's Forever measurements.
#[test]
fn text_scale_scales_the_icons_with_the_glyphs() {
    let env = WowLuaEnv::new().unwrap();
    let (plain, with_icon): (f64, f64) = env
        .eval(
            r#"
            local fs = UIParent:CreateFontString(nil, "ARTWORK")
            fs:SetFont("Fonts\\FRIZQT__.TTF", 12, "")
            fs:SetTextScale(2)
            fs:SetText("ab")
            local a = fs:GetStringWidth()
            fs:SetText("a|A:hud-backpack:10:10|ab")
            return a, fs:GetStringWidth()
            "#,
        )
        .unwrap();
    assert!(
        (with_icon - plain - 20.0).abs() < 0.01,
        "plain {plain}, with icon {with_icon}"
    );
}

// R88d verifies cached width and height division by FontString scale.
#[test]
fn string_metrics_are_local_to_fontstring_scale() {
    let env = WowLuaEnv::new().unwrap();
    let (width_before, height_before, width_after, height_after): (f64, f64, f64, f64) = env
        .eval(
            r#"
            local fs = UIParent:CreateFontString(nil, "ARTWORK")
            fs:SetFont("Fonts\\FRIZQT__.TTF", 12, "")
            fs:SetText("R88d scale probe")
            local width, height = fs:GetStringWidth(), fs:GetStringHeight()
            fs:SetScale(2)
            return width, height, fs:GetStringWidth(), fs:GetStringHeight()
            "#,
        )
        .unwrap();
    assert!(
        (width_before - width_after).abs() < 0.01,
        "width before {width_before}, after {width_after}"
    );
    assert!(
        (height_before - height_after).abs() < 0.01,
        "height before {height_before}, after {height_after}"
    );
}

// R88e.6 verifies that each shown left FontString region height enters the row total.
#[test]
fn tooltip_height_adds_each_left_fontstring_row() {
    let env = WowLuaEnv::new().unwrap();
    let (one_row, two_rows, three_rows, first_height, second_height, third_height): (
        f64,
        f64,
        f64,
        f64,
        f64,
        f64,
    ) = env
        .eval(
            r#"
            GameTooltip:ClearLines()
            GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
            GameTooltip:AddLine("Repeated row")
            GameTooltip:Show()
            local one = GameTooltip:GetHeight()
            local first_height = GameTooltip:GetLeftLine(1):GetHeight()
            GameTooltip:AddLine("Repeated row")
            GameTooltip:Show()
            local two = GameTooltip:GetHeight()
            local second_height = GameTooltip:GetLeftLine(2):GetHeight()
            GameTooltip:AddLine("Repeated row")
            GameTooltip:Show()
            local three = GameTooltip:GetHeight()
            local third_height = GameTooltip:GetLeftLine(3):GetHeight()
            return one, two, three, first_height, second_height, third_height
            "#,
        )
        .unwrap();
    let second_row_spacing = two_rows - one_row - second_height;
    let third_row_spacing = three_rows - two_rows - third_height;
    assert!(
        first_height > 0.0,
        "first left region height {first_height}"
    );
    assert!(
        second_height > 0.0,
        "second left region height {second_height}"
    );
    assert!(
        third_height > 0.0,
        "third left region height {third_height}"
    );
    assert!(
        (second_row_spacing - third_row_spacing).abs() < 0.01,
        "second row spacing {second_row_spacing}, third row spacing {third_row_spacing}"
    );
}

// INFERRED: icon-only line height and the default native vertical mode remain unknown.
#[test]
fn icon_only_text_is_one_line_high() {
    let env = WowLuaEnv::new().unwrap();
    let (text, icon_only): (f64, f64) = env
        .eval(
            r#"
            local fs = UIParent:CreateFontString(nil, "ARTWORK")
            fs:SetFont("Fonts\\FRIZQT__.TTF", 12, "")
            fs:SetText("X")
            local a = fs:GetStringHeight()
            fs:SetText("|TInterface\\Buttons\\WHITE8X8:1:110|t")
            return a, fs:GetStringHeight()
            "#,
        )
        .unwrap();
    assert!(
        icon_only > 0.0 && (icon_only - text).abs() < 0.01,
        "text {text}, icon only {icon_only}"
    );
}
