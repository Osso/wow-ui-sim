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

#[test]
fn atlas_icon_adds_its_width() {
    let (plain, with_icon) = widths("3 2", "3 |A:hud-backpack:14:20|a2");
    assert!(
        (with_icon - plain - 20.0).abs() < 0.01,
        "plain {plain}, with icon {with_icon}"
    );
}

#[test]
fn texture_icon_of_size_zero_is_as_wide_as_a_line_is_high() {
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

#[test]
fn texture_icon_without_size_takes_no_room() {
    let (plain, with_icon) = widths("ab", "a|TInterface\\Icons\\INV_Misc_QuestionMark|tb");
    assert!(
        (with_icon - plain).abs() < 0.01,
        "plain {plain}, with icon {with_icon}"
    );
}

#[test]
fn atlas_icon_without_width_is_a_line_high_square() {
    let (plain, with_icon) = widths("ab", "a|A:hud-backpack:24:0|ab");
    let (_, line) = widths("ab", "|A:hud-backpack:0:0|a");
    assert!(
        line > 0.0 && (with_icon - plain - line).abs() < 0.01,
        "plain {plain}, with icon {with_icon}, line {line}"
    );
}

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
    assert!(text < 32.0 && (tall - 32.0).abs() < 0.01, "text {text}, tall {tall}");
}

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
