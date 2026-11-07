//! Retail/PTR tooltip texture and atlas row attachments: the seven anchor
//! ordinals and the attachment settings.
//!
//! Native contract: WM63 S8L lines 11-35 (anchor table, settings, 30 per
//! tooltip, four width groups, top/bottom overflow) and WM72 line 30
//! (attachment reservations shift the row's left constraint).
//!
//! The producer is already on master (`7fee0f82b`, spec
//! `docs/specs/tooltip-attachments.md`). These cases cover the anchor
//! ordinals and setting conversions that had no direct test. Original
//! consumer: Mainline `Blizzard_SharedXMLGame/Tooltip/TooltipDataRules.lua`
//! post-callbacks, which call `AddTexture` with the default settings,
//! `RightCenter` and typed icon settings.
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
    function TTLeftX(tooltip, row)
        return select(4, tooltip:GetLeftLine(row):GetPoint(2))
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

/// WM63 S8L anchor table: texture point, text point and horizontal offset for
/// ordinals 0 through 6. Only `LeftTop` and `RightCenter` are used by
/// `TooltipDataRules`, so the remaining rows of the table are asserted
/// directly from the native evidence.
#[test]
fn tooltip_attachment_seven_anchor_ordinals_place_the_texture() {
    let env = env_with_helpers();
    env.exec(
        r#"
        local tip = GameTooltip
        tip:SetOwner(UIParent, "ANCHOR_NONE")
        tip:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
        local A = Enum.TooltipTextureAnchor
        local settings = {
            width = 10, height = 6, verticalOffset = 2, margin = { left = 3, right = 5 },
        }
        local order = { A.LeftTop, A.LeftCenter, A.LeftBottom, A.RightTop, A.RightCenter, A.RightBottom }
        local labels = { "LeftTop", "LeftCenter", "LeftBottom", "RightTop", "RightCenter", "RightBottom" }
        for index = 1, 6 do
            tip:AddLine("Row" .. index)
            local rowSettings = { width = 10, height = 6, verticalOffset = 2,
                margin = { left = 3, right = 5 }, anchor = order[index] }
            tip:AddTexture(134400, rowSettings)
        end
        tip:Show()
        assert(tip:NumLines() == 6, "no attachment appends a row")

        -- Left-side ordinals move left by the right margin; right-side ordinals
        -- move right by the left margin. Every ordinal raises by the offset.
        local expected = {
            { point = "TOPRIGHT", relative = "TOPLEFT", x = -5 },
            { point = "RIGHT", relative = "LEFT", x = -5 },
            { point = "BOTTOMRIGHT", relative = "BOTTOMLEFT", x = -5 },
            { point = "TOPLEFT", relative = "TOPRIGHT", x = 3 },
            { point = "LEFT", relative = "RIGHT", x = 3 },
            { point = "BOTTOMLEFT", relative = "BOTTOMRIGHT", x = 3 },
        }
        for row = 1, 6 do
            local line = tip:GetLeftLine(row)
            local attached = TTAttached(tip, line)
            assert(#attached == 1, labels[row] .. " attaches one texture")
            local texture = attached[1]
            local want = expected[row]
            local point, relativeTo, relativePoint, x, y = texture:GetPoint(1)
            assert(point == want.point and relativeTo == line and relativePoint == want.relative,
                labels[row] .. " points")
            TTNear(x, want.x, labels[row] .. " x")
            TTNear(y, 2, labels[row] .. " vertical offset")
            TTNear(texture:GetWidth(), 10, labels[row] .. " width")
            TTNear(texture:GetHeight(), 6, labels[row] .. " height")
        end

        -- Left-side ordinals reserve 10 + 3 + 5 outward on the left. The
        -- right-side ordinals reserve only an inward group, which does not
        -- move the left constraint.
        for row = 1, 3 do
            TTNear(TTLeftX(tip, row), 28, "row " .. row .. " outward-left reservation")
        end
        for row = 4, 6 do
            TTNear(TTLeftX(tip, row), 10, "row " .. row .. " inward reservation stays inward")
        end

        -- Bottom-aligned ordinals resolve against the text bottom.
        local rowThree, rowSix = tip:GetLeftLine(3), tip:GetLeftLine(6)
        TTNear(TTAttached(tip, rowThree)[1]:GetBottom(), rowThree:GetBottom() + 2, "LeftBottom bottom")
        TTNear(TTAttached(tip, rowThree)[1]:GetRight(), rowThree:GetLeft() - 5, "LeftBottom right")
        TTNear(TTAttached(tip, rowSix)[1]:GetBottom(), rowSix:GetBottom() + 2, "RightBottom bottom")
        TTNear(TTAttached(tip, rowSix)[1]:GetLeft(), rowSix:GetRight() + 3, "RightBottom left")

        -- All: two points, no horizontal offset, displacement on both.
        tip:ClearLines()
        tip:AddLine("Spanned", 1, 1, 1, true)
        tip:AddTexture(134400, {
            width = 10, height = 6, verticalOffset = 2, anchor = A.All,
            margin = { left = 3, right = 5 },
        })
        tip:Show()
        local line = tip:GetLeftLine(1)
        local spanned = TTAttached(tip, line)[1]
        assert(spanned:GetNumPoints() == 2, "All uses two constraints")
        TTNear(spanned:GetLeft(), line:GetLeft(), "All spans text left")
        TTNear(spanned:GetRight(), line:GetRight(), "All spans text right")
        TTNear(spanned:GetTop(), line:GetTop() + 2, "All raised top")
        TTNear(spanned:GetBottom(), line:GetBottom() + 2, "All raised bottom")
        "#,
    )
    .unwrap();
}

/// WM63 S8L settings: defaults 12 by 12 with a right margin of 8, ordinal
/// truncation toward zero, out-of-range ordinals keeping zero, tint and
/// desaturation clamped to zero through one, explicit texture coordinates
/// passed to the texture, and a zero dimension using the resolved extent.
#[test]
fn tooltip_attachment_settings_follow_native_conversions() {
    let env = env_with_helpers();
    env.exec(
        r#"
        local tip = GameTooltip
        tip:SetOwner(UIParent, "ANCHOR_NONE")
        tip:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)

        -- Defaults: 12 by 12, right margin 8, no offset, white, opaque.
        tip:AddLine("Defaults")
        tip:AddTexture(134400)
        tip:Show()
        local attached = TTAttached(tip, tip:GetLeftLine(1))
        assert(#attached == 1, "bare AddTexture attaches one texture")
        local texture = attached[1]
        local point, _, relativePoint, x, y = texture:GetPoint(1)
        assert(point == "TOPRIGHT" and relativePoint == "TOPLEFT", "default LeftTop points")
        TTNear(x, -8, "default right margin")
        TTNear(y, 0, "default vertical offset")
        TTNear(texture:GetWidth(), 12, "default width")
        TTNear(texture:GetHeight(), 12, "default height")
        TTNear(TTLeftX(tip, 1), 30, "default 12 + 0 + 8 reservation")
        local r, g, b, a = texture:GetVertexColor()
        TTNear(r, 1, "default tint red")
        TTNear(g, 1, "default tint green")
        TTNear(b, 1, "default tint blue")
        TTNear(a, 1, "default tint alpha")
        assert(not texture:IsDesaturated(), "default not desaturated")

        -- Ordinals truncate toward zero: 2.9 is LeftBottom, 1.9 is RightLine.
        tip:ClearLines()
        tip:AddDoubleLine("Left", "Right")
        tip:AddTexture(134400, {
            width = 6, height = 6, anchor = 2.9, region = 1.9, verticalOffset = -1,
        })
        tip:Show()
        local left, right = tip:GetLeftLine(1), tip:GetRightLine(1)
        local truncated = TTAttached(tip, right)
        assert(#truncated == 1, "region 1.9 selects the right line")
        assert(#TTAttached(tip, left) == 0, "region 1.9 leaves the left line alone")
        local point, _, relativePoint, x, y = truncated[1]:GetPoint(1)
        assert(point == "BOTTOMRIGHT" and relativePoint == "BOTTOMLEFT", "anchor 2.9 is LeftBottom")
        TTNear(x, -8, "LeftBottom right margin")
        TTNear(y, -1, "LeftBottom vertical offset")

        -- Out-of-range ordinals keep zero: anchor 9 is LeftTop, region 2 the
        -- left line.
        tip:ClearLines()
        tip:AddDoubleLine("Left", "Right")
        tip:AddTexture(134400, { width = 6, height = 6, anchor = 9, region = 2 })
        tip:Show()
        left, right = tip:GetLeftLine(1), tip:GetRightLine(1)
        local fallback = TTAttached(tip, left)
        assert(#fallback == 1, "region 2 falls back to the left line")
        assert(#TTAttached(tip, right) == 0, "region 2 leaves the right line alone")
        local point, _, relativePoint = fallback[1]:GetPoint(1)
        assert(point == "TOPRIGHT" and relativePoint == "TOPLEFT", "anchor 9 falls back to LeftTop")

        -- Tint and desaturation clamp to zero through one.
        tip:ClearLines()
        tip:AddLine("Clamped")
        tip:AddTexture(134400, {
            vertexColor = { r = 1.5, g = -0.5, b = 0.25, a = 2 },
            desaturation = 2,
        })
        tip:Show()
        local clamped = TTAttached(tip, tip:GetLeftLine(1))[1]
        local r, g, b, a = clamped:GetVertexColor()
        TTNear(r, 1, "tint red clamp")
        TTNear(g, 0, "tint green clamp")
        TTNear(b, 0.25, "tint blue kept")
        TTNear(a, 1, "tint alpha clamp")
        assert(clamped:IsDesaturated(), "desaturation 2 clamps to on")
        tip:ClearLines()
        tip:AddLine("Not clamped")
        tip:AddTexture(134400, { desaturation = -1 })
        tip:Show()
        assert(not TTAttached(tip, tip:GetLeftLine(1))[1]:IsDesaturated(),
            "desaturation -1 clamps to off")

        -- Explicit texture coordinates reach the texture.
        tip:ClearLines()
        tip:AddLine("Cropped")
        tip:AddTexture(134400, {
            texCoords = { left = 0.171875, right = 0.828125, top = 0.171875, bottom = 0.828125 },
        })
        tip:Show()
        local coords = { TTAttached(tip, tip:GetLeftLine(1))[1]:GetTexCoord() }
        assert(#coords == 8, "eight texture coordinates")
        local corners = { 0.171875, 0.171875, 0.171875, 0.828125, 0.828125, 0.171875, 0.828125, 0.828125 }
        for index = 1, 8 do
            TTNear(coords[index], corners[index], "texture coordinate " .. index)
        end

        -- A zero dimension uses the resolved atlas extent: questlog-tab-side
        -- is 51 by 60.
        tip:ClearLines()
        tip:AddLine("Extent")
        tip:AddAtlas("questlog-tab-side", { width = 0, height = 0 })
        tip:Show()
        local atlas = TTAttached(tip, tip:GetLeftLine(1))[1]
        TTNear(atlas:GetWidth(), 51, "zero width uses the atlas width")
        TTNear(atlas:GetHeight(), 60, "zero height uses the atlas height")
        TTNear(TTLeftX(tip, 1), 10 + 51 + 8, "extent feeds the left reservation")
        "#,
    )
    .unwrap();
}

/// Original consumer: `TooltipDataRules.AzeriteEssenceSlot` attaches the
/// essence icon and then the border to the same row, with cropped texture
/// coordinates and desaturation from the disabled flag.
#[test]
fn blizzard_azerite_essence_slot_attaches_two_textures_to_one_row() {
    let env = env_with_helpers();
    env.exec(
        r#"
        assert(TooltipDataRules and TooltipDataRules.AzeriteEssenceSlot,
            "Mainline TooltipDataRules is loaded")
        local tip = GameTooltip
        tip:SetOwner(UIParent, "ANCHOR_NONE")
        tip:SetPoint("TOPLEFT", UIParent, "TOPLEFT", 100, -100)
        assert(tip:ProcessInfo({ tooltipData = { lines = {
            { type = 0, leftText = "Header" },
            { type = Enum.TooltipDataLineType.AzeriteEssenceSlot, leftText = "Slot",
              slotType = "major", essenceIcon = 134402, disabled = true },
        } } }))
        assert(tip:NumLines() == 2, "attachments must not append a row")

        local line = tip:GetLeftLine(2)
        local attached = TTAttached(tip, line)
        assert(#attached == 2, "the essence and the border share the row")
        local essence, border = attached[1], attached[2]

        -- Essence: 10 by 10, offset -1, cropped coordinates, desaturated.
        TTNear(essence:GetWidth(), 10, "essence width")
        TTNear(essence:GetHeight(), 10, "essence height")
        TTNear(select(5, essence:GetPoint(1)), -1, "essence vertical offset")
        local coords = { essence:GetTexCoord() }
        local corners = { 0.171875, 0.171875, 0.171875, 0.828125, 0.828125, 0.171875, 0.828125, 0.828125 }
        for index = 1, 8 do
            TTNear(coords[index], corners[index], "essence texture coordinate " .. index)
        end
        assert(essence:IsDesaturated(), "disabled essence is desaturated")

        -- Border: 18 by 12, right margin 4, desaturated.
        local point, _, relativePoint, x, y = border:GetPoint(1)
        assert(point == "TOPRIGHT" and relativePoint == "TOPLEFT", "border LeftTop points")
        TTNear(x, -4, "border right margin")
        TTNear(y, 0, "border has no vertical offset")
        TTNear(border:GetWidth(), 18, "border width")
        TTNear(border:GetHeight(), 12, "border height")
        assert(border:IsDesaturated(), "disabled border is desaturated")

        -- Outward-left is the maximum of 10 + 0 + 8 and 18 + 0 + 4, not the sum.
        TTNear(TTLeftX(tip, 2), 32, "outward-left maximum")
        -- Bottom overflow uses the maximum protrusion below the text.
        local textHeight = line:GetHeight()
        local header = tip:GetLeftLine(1):GetHeight()
        TTNear(tip:GetHeight(), 20.48 + header + 2 + textHeight + math.max(0, 12 - textHeight),
            "row height adds the maximum bottom overflow")
        "#,
    )
    .unwrap();
}
