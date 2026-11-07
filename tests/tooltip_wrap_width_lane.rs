//! Tooltip wrapped-content width: intrinsic cap, unwrapped/minimum floors,
//! shrink-to-fit and the forced wrapped size, plus the width assigned to the
//! wrapped left FontString.
//!
//! Native contract: `tooltip/WM63-L23-wrap-spacing.md` section 2 (available wrap
//! width) and `WM72-tooltip-text-anchors.md` (assignment and public queries),
//! build 70688.
//!
//! Original consumer: Mainline `Blizzard_SharedXMLGame/Tooltip/TooltipDataHandler.lua:309`
//! `ProcessLines` -> `:338 AddLineDataText`, which calls
//! `GameTooltip_AddColoredLine` / `GameTooltip_AddColoredDoubleLine`
//! (`Blizzard_SharedXML/SharedTooltipTemplates.lua:166-181`). Runtime selection:
//! `GameTooltipTemplate` carries `GameTooltipDataMixin`
//! (`Blizzard_GameTooltip/Mainline/GameTooltip.xml:4`), created from
//! `TooltipDataHandlerMixin` (`Blizzard_GameTooltip/Mainline/GameTooltip.lua:951`),
//! and the tests assert that `GameTooltip.ProcessInfo` is that consumer function.
#![cfg(all(
    feature = "gui",
    any(feature = "client-retail", feature = "client-ptr")
))]

mod tooltip_full_env_helpers;

use tooltip_full_env_helpers::setup_full_env;

const CONSUMER_CHECK: &str = r#"
    assert(GameTooltip.ProcessInfo == TooltipDataHandlerMixin.ProcessInfo,
        "GameTooltip must run the TooltipDataHandler consumer")
"#;

/// WM63 L23 section 2: the widest wrapped contribution is capped by the custom
/// wrap value and shrunk to its fitted width. A right column on a different row
/// only raises the unwrapped floor; it must not disable the shrink of the
/// wrapped row, which keeps the selected width (WM72).
#[test]
fn blizzard_tooltip_shrink_survives_a_sibling_right_column_row() {
    let env = setup_full_env();
    env.exec(&format!(
        r#"
        {CONSUMER_CHECK}

        -- One word per line at cap 200: the widest fitted segment stays well
        -- below the cap, so shrink-to-fit reduces the content width.
        local word = string.rep("a", 16)
        local body = word .. " " .. word .. " " .. word .. " " .. word

        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        GameTooltip:SetCustomWordWrapMinWidth(200)
        assert(GameTooltip:ProcessInfo({{ tooltipData = {{ lines = {{
            {{ type = 0, leftText = body, wrapText = true }},
        }} }} }}))
        local wrapped_only = GameTooltip:GetWidth() - 20.48

        -- The same wrapped row plus an ordinary row that carries a right column.
        assert(GameTooltip:ProcessInfo({{ tooltipData = {{ lines = {{
            {{ type = 0, leftText = body, wrapText = true }},
            {{ type = 0, leftText = "B", rightText = "1" }},
        }} }} }}))
        local with_right = GameTooltip:GetWidth() - 20.48
        local row_width = GameTooltip:GetLeftLine(1):GetWidth()

        assert(wrapped_only <= 200.01,
            string.format("custom wrap value caps the wrapped candidate: %.3f", wrapped_only))
        assert(math.abs(with_right - wrapped_only) < 0.5, string.format(
            "a sibling right column must not disable shrink-to-fit: wrapped only %.3f, with right row %.3f",
            wrapped_only, with_right))
        assert(math.abs(row_width - with_right) < 0.5, string.format(
            "wrapped left FontString keeps the selected width: row %.3f, content %.3f",
            row_width, with_right))
    "#
    ))
    .unwrap();
}

/// WM63 L23 section 2: "The reduction cannot cross the unwrapped-row or
/// explicit-minimum floor". The unwrapped double row sets that floor and stays
/// in the first row slot of both tooltips, so its measured font is identical.
#[test]
fn blizzard_tooltip_shrink_does_not_cross_the_unwrapped_floor() {
    let env = setup_full_env();
    env.exec(&format!(
        r#"
        {CONSUMER_CHECK}

        local word = string.rep("a", 16)
        local body = word .. " " .. word .. " " .. word .. " " .. word
        local wide = string.rep("W", 40)

        -- Floor probe: the unwrapped double row on its own.
        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        assert(GameTooltip:ProcessInfo({{ tooltipData = {{ lines = {{
            {{ type = 0, leftText = wide, rightText = "1" }},
        }} }} }}))
        local floor = GameTooltip:GetWidth() - 20.48

        assert(GameTooltip:ProcessInfo({{ tooltipData = {{ lines = {{
            {{ type = 0, leftText = wide, rightText = "1" }},
            {{ type = 0, leftText = body, wrapText = true }},
        }} }} }}))
        local mixed = GameTooltip:GetWidth() - 20.48

        assert(math.abs(mixed - floor) < 0.5, string.format(
            "shrink stops at the unwrapped floor: floor %.3f, mixed %.3f", floor, mixed))
    "#
    ))
    .unwrap();
}
