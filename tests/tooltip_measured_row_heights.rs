//! Retail/PTR tooltip row heights are the row FontString's own measured height.
//!
//! Native contract: WM63 L23 section 3 (lines 42-44). Height adds attachment
//! overflow around the left FontString's measured height; layout uses
//! FontString measurements, not a fixed font-size pair or a universal 1.2
//! line-height multiplier. The exact native header/body font files, glyph
//! advances and rendered line heights are UNKNOWN, so these cases assert only
//! that the height follows the row's own FontString.
//!
//! Original consumer: Mainline
//! `Blizzard_SharedXMLGame/Tooltip/TooltipDataHandler.lua` line 309
//! `ProcessLines` (reached by `SetItemByID`/`SetSpellByID` through
//! `InternalProcessInfo`) calls `AddLineDataText`, which calls
//! `GameTooltip_AddColoredLine` (`Blizzard_SharedXML/SharedTooltipTemplates.lua`
//! line 166) and so `GameTooltip:AddLine`. The row fonts are the tooltip
//! template's declared slots: `textLeft1Font` and `textLeft2Font`
//! (`Blizzard_SharedXML/SharedTooltipTemplates.xml` lines 12-15), applied by
//! `SharedTooltip_OnLoad` (`SharedTooltipTemplates.lua` lines 3-7 and 16-19).
#![cfg(all(
    feature = "gui",
    any(feature = "client-retail", feature = "client-ptr")
))]

mod tooltip_full_env_helpers;

use tooltip_full_env_helpers::setup_full_env;

const HELPERS: &str = r#"
    -- WM63 L23: content width and height add 20.48 plus explicit padding, and
    -- inter-row spacing is 2 between visible rows.
    local OUTER = 20.48
    local SPACING = 2

    function MTNear(actual, expected, label)
        assert(math.abs(actual - expected) < 0.01,
            label .. ": expected " .. tostring(expected) .. ", got " .. tostring(actual))
    end

    function MTRowHeights(tip)
        local heights = {}
        for index = 1, tip:NumLines() do
            heights[index] = tip:GetLeftLine(index):GetHeight()
        end
        return heights
    end

    function MTNearMeasuredHeight(tip, label)
        local heights = MTRowHeights(tip)
        local expected = OUTER
        for index, height in ipairs(heights) do
            if index > 1 then
                expected = expected + SPACING
            end
            expected = expected + height
        end
        MTNear(tip:GetHeight(), expected, label .. " rows sum to the measured height")
        return heights
    end
"#;

#[test]
fn blizzard_item_tooltip_body_rows_measure_the_declared_body_font() {
    let env = setup_full_env();
    env.exec(HELPERS).unwrap();
    env.exec(
        r#"
        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        assert(GameTooltip:SetItemByID(6948), "item data must reach the tooltip")
        GameTooltip:Show()
        assert(GameTooltip:NumLines() == 3, "item 6948 has a header and two body rows")

        local heights = MTNearMeasuredHeight(GameTooltip, "GameTooltip")
        local header, headerSize = GameTooltip:GetLeftLine(1):GetFont()
        local body, bodySize = GameTooltip:GetLeftLine(2):GetFont()
        assert(headerSize > bodySize, "row 1 uses textLeft1Font, row 2 textLeft2Font")
        assert(heights[1] > heights[2],
            "the header row must be taller than the body row it measures")

        -- Rows after the two declared font slots have no font of their own, so
        -- the synthesized row measures the tooltip's declared body font.
        local later, laterSize = GameTooltip:GetLeftLine(3):GetFont()
        assert(later == body and laterSize == bodySize,
            "row 3 must keep the body font: expected " .. tostring(body) .. "/" ..
            tostring(bodySize) .. ", got " .. tostring(later) .. "/" .. tostring(laterSize))
        MTNear(heights[3], heights[2], "row 3 measured like row 2")
    "#,
    )
    .unwrap();
}

#[test]
fn blizzard_tooltip_wrapped_row_height_follows_the_font_string_measurement() {
    let env = setup_full_env();
    env.exec(HELPERS).unwrap();
    env.exec(
        r#"
        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        assert(GameTooltip:ProcessInfo({ tooltipData = { lines = {
            { type = 0, leftText = "Summary" },
            { type = 0, leftText = "Short body" },
            { type = 0, leftText = string.rep("wrapped description ", 12), wrapText = true },
        } } }))

        local heights = MTNearMeasuredHeight(GameTooltip, "GameTooltip")
        -- A fixed single-line constant cannot produce a multi-line row.
        assert(heights[3] > heights[2] * 2,
            "the wrapped row must measure several text lines: " .. tostring(heights[3]))
        MTNear(GameTooltip:GetLeftLine(3):GetHeight(), heights[3],
            "the wrapped row height is the FontString height")
    "#,
    )
    .unwrap();
}
