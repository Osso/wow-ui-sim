#![cfg(all(
    feature = "gui",
    any(feature = "client-retail", feature = "client-ptr")
))]

mod tooltip_full_env_helpers;

use tooltip_full_env_helpers::setup_full_env;

#[test]
fn blizzard_tooltip_wrapped_content_uses_custom_width_as_cap() {
    let env = setup_full_env();
    env.exec(r#"
        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        GameTooltip:SetCustomWordWrapMinWidth(350)
        assert(GameTooltip:ProcessInfo({ tooltipData = { lines = {
            { type = 0, leftText = "Short", wrapText = true },
        } } }))
        assert(GameTooltip:GetWidth() < 150, "short wrapped content must not expand to the custom cap")
    "#).unwrap();
}

#[test]
fn blizzard_tooltip_wrap_cap_and_minimum_set_line_width() {
    let env = setup_full_env();
    env.exec(
        r#"
        local text = string.rep("many words ", 40)
        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        GameTooltip:SetShrinkToFitWrapped(false)
        GameTooltip:SetCustomWordWrapMinWidth(120)
        assert(GameTooltip:ProcessInfo({ tooltipData = { lines = {
            { type = 0, leftText = text, wrapText = true },
        } } }))
        assert(math.abs(GameTooltip:GetWidth() - 140.48) < 0.1)
        assert(math.abs(GameTooltip:GetLeftLine(1):GetWidth() - 120) < 0.1)
        GameTooltip:SetCustomWordWrapMinWidth(200)
        GameTooltip:Show()
        assert(math.abs(GameTooltip:GetWidth() - 220.48) < 0.1)
        assert(math.abs(GameTooltip:GetLeftLine(1):GetWidth() - 200) < 0.1)
        GameTooltip:SetMinimumWidth(300, true)
        GameTooltip:Show()
        assert(math.abs(GameTooltip:GetWidth() - 320.48) < 0.1)
        assert(math.abs(GameTooltip:GetLeftLine(1):GetWidth() - 300) < 0.1)
    "#,
    )
    .unwrap();
}

#[test]
fn blizzard_tooltip_height_tracks_fontstring_font_size() {
    let env = setup_full_env();
    env.exec(
        r#"
        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        assert(GameTooltip:ProcessInfo({ tooltipData = { lines = {
            { type = 0, leftText = "A line of text" },
        } } }))
        local line = GameTooltip:GetLeftLine(1)
        local font = line:GetFont()
        line:SetFont(font, 12)
        GameTooltip:Show()
        local small = GameTooltip:GetHeight()
        line:SetFont(font, 28)
        GameTooltip:Show()
        assert(GameTooltip:GetHeight() > small + 8, "tooltip height must follow the line font size")
    "#,
    )
    .unwrap();
}
