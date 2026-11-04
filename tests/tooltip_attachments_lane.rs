//! Retail/PTR texture and atlas attachments on existing tooltip rows.
//! Native contract: WM63 S8L (anchors, settings, cap) and WM84 (reservation
//! groups, overflow, row inset). Original consumers: Mainline
//! `Blizzard_SharedXMLGame/Tooltip/TooltipDataRules.lua:18-223` post-callbacks
//! and `Blizzard_SharedXML/SharedTooltipTemplates.lua:166-181` inset forwarding.
#![cfg(all(
    feature = "gui",
    any(feature = "client-retail", feature = "client-ptr")
))]

mod tooltip_full_env_helpers;

use tooltip_full_env_helpers::setup_full_env;
use wow_ui_sim::lua_api::WowLuaEnv;

const HELPERS: &str = r#"
    -- Shown Texture regions of `tooltip` whose first anchor targets `line`.
    function TTAttached(tooltip, line)
        local found = {}
        for _, region in ipairs({ tooltip:GetRegions() }) do
            if region:GetObjectType() == "Texture" and region:IsShown() then
                local _, relativeTo = region:GetPoint(1)
                if relativeTo == line then
                    table.insert(found, region)
                end
            end
        end
        return found
    end
    -- Shown Texture regions anchored to any of the tooltip's FontStrings.
    function TTAttachedCount(tooltip)
        local count = 0
        for _, region in ipairs({ tooltip:GetRegions() }) do
            if region:GetObjectType() == "Texture" and region:IsShown() then
                local _, relativeTo = region:GetPoint(1)
                if relativeTo and relativeTo:GetObjectType() == "FontString" then
                    count = count + 1
                end
            end
        end
        return count
    end
    function TTLeftX(tooltip, row)
        return select(4, tooltip:GetLeftLine(row):GetPoint(2))
    end
    function TTTopY(tooltip, row)
        return select(5, tooltip:GetLeftLine(row):GetPoint(1))
    end
    function TTNear(actual, expected, label)
        assert(math.abs(actual - expected) < 0.01,
            string.format("%s: actual %.4f, expected %.4f", label, actual, expected))
    end
"#;

fn env_with_helpers() -> WowLuaEnv {
    let env = setup_full_env();
    env.exec(HELPERS).expect("install observation helpers");
    env
}

#[test]
fn blizzard_tooltip_typed_icons_reserve_and_anchor_on_their_rows() {
    let env = env_with_helpers();
    env.exec(
        r#"
        assert(TooltipDataRules and TooltipDataRules.GemSocket, "Mainline TooltipDataRules is loaded")
        local L = Enum.TooltipDataLineType
        assert(L.GemSocket and L.QuestObjective and L.AzeriteEssenceSlot
            and L.AzeriteEssencePower and L.Separator, "typed line enums")
        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        GameTooltip:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
        assert(GameTooltip:ProcessInfo({ tooltipData = { lines = {
            { type = L.GemSocket, leftText = "Socket", gemIcon = 134400 },
            { type = L.QuestObjective, leftText = "Objective", completed = true },
            { type = L.AzeriteEssenceSlot, leftText = "Essence", slotType = "major", essenceIcon = 134402 },
            { type = L.AzeriteEssencePower, leftText = "Dash", powerState = "active" },
            { type = L.Separator, leftText = "Sep" },
        } } }))
        assert(GameTooltip:NumLines() == 5, "attachments must not append rows")

        -- Outward-left group per row: default 12+0+8; quest 16+0+2;
        -- essence max(10+0+8, 18+0+4), not the sum; dash 8-3+4; separator 200+0+2.
        local outward = { 20, 18, 22, 9, 202 }
        local counts = { 1, 1, 2, 1, 1 }
        for row = 1, 5 do
            local line = GameTooltip:GetLeftLine(row)
            TTNear(TTLeftX(GameTooltip, row), 10 + outward[row], "row " .. row .. " LEFT x")
            assert(#TTAttached(GameTooltip, line) == counts[row], "row " .. row .. " attachment count")
        end

        -- S8L LeftTop: texture TOPRIGHT to text TOPLEFT, left by the right
        -- margin, raised by verticalOffset.
        local placements = {
            { row = 1, x = -8, y = 0, w = 12, h = 12 },
            { row = 2, x = -2, y = 2, w = 16, h = 16 },
            { row = 4, x = -4, y = -6, w = 8, h = 4 },
        }
        for _, expected in ipairs(placements) do
            local line = GameTooltip:GetLeftLine(expected.row)
            local texture = TTAttached(GameTooltip, line)[1]
            local point, relativeTo, relativePoint, x, y = texture:GetPoint(1)
            assert(point == "TOPRIGHT" and relativeTo == line and relativePoint == "TOPLEFT",
                "row " .. expected.row .. " anchor points")
            TTNear(x, expected.x, "row " .. expected.row .. " texture x")
            TTNear(y, expected.y, "row " .. expected.row .. " texture y")
            TTNear(texture:GetWidth(), expected.w, "row " .. expected.row .. " width")
            TTNear(texture:GetHeight(), expected.h, "row " .. expected.row .. " height")
            TTNear(texture:GetRight(), line:GetLeft() + expected.x, "row " .. expected.row .. " resolved right")
            TTNear(texture:GetTop(), line:GetTop() + expected.y, "row " .. expected.row .. " resolved top")
        end

        -- WM84 overflow candidates with each row's measured left height.
        local h = {}
        for row = 1, 5 do h[row] = GameTooltip:GetLeftLine(row):GetHeight() end
        local top = { 0, 0, 0, 0, 0 }
        local bottom = {
            math.max(0, 12 - h[1]),
            math.max(0, 12 - h[2] - 2),
            math.max(0, 11 - h[3], 12 - h[3]),
            math.max(0, 10 - h[4]),
            math.max(0, 6 - h[5]),
        }
        TTNear(TTTopY(GameTooltip, 1), -10 - top[1], "row 1 TOP y")
        local height = 20.48
        for row = 1, 5 do
            if row > 1 then
                TTNear(TTTopY(GameTooltip, row), -2 - top[row] - bottom[row - 1], "row " .. row .. " TOP y")
                height = height + 2
            end
            height = height + h[row] + top[row] + bottom[row]
        end
        TTNear(GameTooltip:GetHeight(), height, "tooltip height")
        assert(GameTooltip:GetWidth() > 202 + 20.48, "separator reservation widens the tooltip")
        "#,
    )
    .unwrap();
}

#[test]
fn blizzard_tooltip_typed_line_offsets_store_row_inset() {
    let env = env_with_helpers();
    env.exec(
        r#"
        local L = Enum.TooltipDataLineType
        GameTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        assert(GameTooltip:ProcessInfo({ tooltipData = { lines = {
            { type = 0, leftText = "Header" },
            -- TooltipDataRules.GemSocketEnchantment sets leftOffset 20;
            -- GameTooltip_AddColoredDoubleLine forwards it as argument 11.
            { type = L.GemSocketEnchantment, leftText = "Enchanted", rightText = "+5" },
            -- TooltipDataRules.LeftOffsetPowerDescription sets 10;
            -- GameTooltip_AddColoredLine forwards it as argument 7.
            { type = L.AzeriteItemPowerDescription, leftText = string.rep("power words ", 30), wrapText = true },
        } } }))
        TTNear(TTLeftX(GameTooltip, 1), 10, "header LEFT x")
        TTNear(TTLeftX(GameTooltip, 2), 30, "double line LEFT x")
        TTNear(TTLeftX(GameTooltip, 3), 20, "wrapped line LEFT x")
        local content = GameTooltip:GetWidth() - 20.48
        TTNear(select(4, GameTooltip:GetRightLine(2):GetPoint(1)), content - 20, "right RIGHT x")
        TTNear(GameTooltip:GetLeftLine(3):GetWidth(), content - 10, "wrapped width")

        GameTooltip:ClearLines()
        GameTooltip:AddLine("Plain")
        GameTooltip:Show()
        TTNear(TTLeftX(GameTooltip, 1), 10, "inset does not outlive its row")
        GameTooltip:AddLine("Explicit", 1, 1, 1, false, 7.5)
        GameTooltip:Show()
        TTNear(TTLeftX(GameTooltip, 2), 17.5, "fractional explicit inset")
        GameTooltip:SetText("Title")
        GameTooltip:Show()
        TTNear(TTLeftX(GameTooltip, 1), 10, "SetText passes explicit inset zero")
        "#,
    )
    .unwrap();
}

#[test]
fn blizzard_item_ref_close_button_spacer_reserves_outward_right() {
    let env = env_with_helpers();
    env.exec(
        r#"
        assert(ItemRefTooltip and ItemRefTooltip.ProcessInfo, "Mainline ItemRefTooltip is loaded")
        ItemRefTooltip:SetOwner(UIParent, "ANCHOR_NONE")
        ItemRefTooltip:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
        assert(ItemRefTooltip:ProcessInfo({ tooltipData = { lines = {
            { type = Enum.TooltipDataLineType.ItemName, leftText = "Item name", rightText = "x" },
            { type = 0, leftText = "Body" },
        } } }))
        -- AccountForCloseButtonOnItems: RightLine, RightCenter, 10 wide,
        -- default right margin 8: outward-right 18, no left reservation.
        local right = ItemRefTooltip:GetRightLine(1)
        local content = ItemRefTooltip:GetWidth() - 20.48
        TTNear(select(4, right:GetPoint(1)), content - 18, "right RIGHT x")
        TTNear(TTLeftX(ItemRefTooltip, 1), 10, "left LEFT x")
        local spacer = TTAttached(ItemRefTooltip, right)
        assert(#spacer == 1, "spacer anchors to the right line")
        local point, _, relativePoint, x, y = spacer[1]:GetPoint(1)
        assert(point == "LEFT" and relativePoint == "RIGHT", "RightCenter points")
        TTNear(x, 0, "right by the left margin")
        TTNear(y, 0, "no vertical offset")
        TTNear(spacer[1]:GetLeft(), right:GetRight(), "resolved left edge")
        "#,
    )
    .unwrap();
}

#[test]
fn tooltip_attachment_vertical_overflow_separates_rows() {
    let env = env_with_helpers();
    env.exec(
        r#"
        local tip = GameTooltip
        tip:SetOwner(UIParent, "ANCHOR_NONE")
        tip:AddLine("Tall")
        tip:AddTexture(134400, { width = 10, height = 40, margin = { right = 2 } })
        tip:AddLine("After")
        tip:Show()
        local h1, h2 = tip:GetLeftLine(1):GetHeight(), tip:GetLeftLine(2):GetHeight()
        TTNear(TTTopY(tip, 1), -10, "top attachment has no top overflow")
        TTNear(TTTopY(tip, 2), -2 - (40 - h1), "bottom overflow of the preceding row")
        TTNear(tip:GetHeight(), 40 + 2 + h2 + 20.48, "row height includes overflow")

        tip:ClearLines()
        tip:AddLine("Raised")
        tip:AddTexture(134400, { height = 40, anchor = Enum.TooltipTextureAnchor.LeftBottom })
        tip:Show()
        local h = tip:GetLeftLine(1):GetHeight()
        TTNear(TTTopY(tip, 1), -10 - (40 - h), "bottom attachment overflows above")
        TTNear(tip:GetHeight(), 40 + 20.48, "measured text plus top overflow")
        "#,
    )
    .unwrap();
}

#[test]
fn tooltip_attachment_margins_stay_fractional_and_all_spans_text() {
    let env = env_with_helpers();
    env.exec(
        r#"
        local tip = GameTooltip
        tip:SetOwner(UIParent, "ANCHOR_NONE")
        tip:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
        tip:AddLine("Fraction")
        tip:AddTexture(134400, { width = 10, margin = { left = -2.5, right = -1.25 } })
        tip:AddLine("Spanned", 1, 1, 1, true)
        tip:AddTexture(134400, {
            height = 10,
            anchor = Enum.TooltipTextureAnchor.All,
            verticalOffset = 3,
            margin = { left = 3, right = 5, top = -2, bottom = -2 },
        })
        tip:Show()
        -- W = 10 - 2.5 - 1.25; the texture moves left by the right margin.
        TTNear(TTLeftX(tip, 1), 16.25, "fractional negative margins")
        local fraction = TTAttached(tip, tip:GetLeftLine(1))[1]
        TTNear(select(4, fraction:GetPoint(1)), 1.25, "texture x is minus the right margin")

        -- All on the left line: outward-left 3, inward-left 5; width total 8.
        local line = tip:GetLeftLine(2)
        TTNear(TTLeftX(tip, 2), 13, "All reserves only the left margin outward")
        TTNear(line:GetWidth(), tip:GetWidth() - 20.48 - 8, "wrap width subtracts all groups")
        local spanned = TTAttached(tip, line)[1]
        assert(spanned:GetNumPoints() == 2, "All uses two constraints")
        TTNear(spanned:GetLeft(), line:GetLeft(), "spans text left")
        TTNear(spanned:GetRight(), line:GetRight(), "spans text right")
        TTNear(spanned:GetTop(), line:GetTop() + 3, "raised top")
        TTNear(spanned:GetBottom(), line:GetBottom() + 3, "raised bottom")
        "#,
    )
    .unwrap();
}

#[test]
fn tooltip_attachment_inward_groups_share_first_selection() {
    let env = env_with_helpers();
    env.exec(
        r#"
        local tip = GameTooltip
        -- Left-region RightTop: 1 - 14 + 8 = -5 (inward from left).
        local inwardLeft = { width = 1, margin = { left = -14 }, anchor = Enum.TooltipTextureAnchor.RightTop }
        -- Right-region LeftTop: 1 - 16 + 8 = -7 (inward from right).
        local inwardRight = { width = 1, margin = { left = -16 }, region = Enum.TooltipTextureRelativeRegion.RightLine }
        local function wrappedWidth(first, second)
            tip:SetOwner(UIParent, "ANCHOR_NONE")
            tip:SetShrinkToFitWrapped(false)
            tip:SetCustomWordWrapMinWidth(100)
            tip:AddLine(string.rep("word ", 60), 1, 1, 1, true)
            tip:AddTexture(134400, first)
            tip:AddTexture(134400, second)
            tip:Show()
            TTNear(TTLeftX(tip, 1), 10, "no outward-left reservation")
            return tip:GetLeftLine(1):GetWidth()
        end
        -- WM84: the first inward candidate replaces zero even when negative;
        -- the other group then changes only for a larger candidate.
        TTNear(wrappedWidth(inwardLeft, inwardRight), 105, "inward results -5 and 0")
        TTNear(wrappedWidth(inwardRight, inwardLeft), 107, "inward results 0 and -7")
        "#,
    )
    .unwrap();
}

#[test]
fn tooltip_attachment_cap_is_per_tooltip_and_clear_releases_it() {
    let env = env_with_helpers();
    env.exec(
        r#"
        local tip = GameTooltip
        tip:SetOwner(UIParent, "ANCHOR_NONE")
        tip:AddTexture(134400, { width = 200 })
        assert(tip:NumLines() == 0, "an empty tooltip has no row to attach to")
        assert(TTAttachedCount(tip) == 0, "nothing attached to an empty tooltip")
        tip:AddLine("First")
        for _ = 1, 29 do tip:AddTexture(134400) end
        tip:AddLine("Second")
        tip:AddTexture(134400)
        tip:AddTexture(134400, { width = 200 })
        tip:Show()
        assert(tip:NumLines() == 2)
        assert(#TTAttached(tip, tip:GetLeftLine(1)) == 29, "equal anchors overlay on one row")
        assert(#TTAttached(tip, tip:GetLeftLine(2)) == 1, "the 31st attachment is rejected")
        TTNear(TTLeftX(tip, 2), 30, "rejected attachment reserves nothing")

        tip:ClearLines()
        assert(TTAttachedCount(tip) == 0, "ClearLines hides released attachments")
        tip:AddLine("Fresh")
        tip:AddTexture(134400, { width = 50 })
        tip:Show()
        assert(TTAttachedCount(tip) == 1, "capacity is released")
        TTNear(TTLeftX(tip, 1), 68, "new attachment reserves 50 + 0 + 8")
        "#,
    )
    .unwrap();
}
