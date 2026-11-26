#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

const SET_OWNER_RELEASE_PROBE: &str = r#"
local function attachedCount()
    local count = 0
    for _, region in ipairs({ GameTooltip:GetRegions() }) do
        if region:GetObjectType() == 'Texture' and region:IsShown() then
            local _, relativeTo = region:GetPoint(1)
            if relativeTo and relativeTo:GetObjectType() == 'FontString' then
                count = count + 1
            end
        end
    end
    return count
end

GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
GameTooltip:AddLine('Attachment release')
GameTooltip:AddTexture(136243, { width = 12, height = 12 })
GameTooltip:Show()
assert(attachedCount() == 1, 'original AddTexture must attach one visible texture')
GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
assert(GameTooltip:NumLines() == 0, 'SetOwner must clear the old row')
assert(attachedCount() == 0, 'SetOwner must release the old attachment')
"#;

const HIDE_RELEASE_PROBE: &str = r#"
local function attachedCount()
    local count = 0
    for _, region in ipairs({ GameTooltip:GetRegions() }) do
        if region:GetObjectType() == 'Texture' and region:IsShown() then
            local _, relativeTo = region:GetPoint(1)
            if relativeTo and relativeTo:GetObjectType() == 'FontString' then
                count = count + 1
            end
        end
    end
    return count
end

GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
GameTooltip:AddLine('Attachment release')
GameTooltip:AddTexture(136243, { width = 12, height = 12 })
GameTooltip:Show()
assert(attachedCount() == 1, 'original AddTexture must attach one visible texture')
GameTooltip:Hide()
assert(not GameTooltip:IsShown(), 'Hide must hide the tooltip')
assert(attachedCount() == 0, 'Hide must release the old attachment')
"#;

const SOCIAL_SEPARATOR_PROBE: &str = r#"
assert(type(SocialUIUtil) == 'table'
    and type(SocialUIUtil.AddSeparatorToTooltip) == 'function',
    'the unchanged cached SocialUIUtil consumer must be loaded')
GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
GameTooltip:AddLine('Header')
GameTooltip:Show()
local rowCount = GameTooltip:NumLines()
SocialUIUtil.AddSeparatorToTooltip(GameTooltip)
assert(GameTooltip:NumLines() == rowCount + 1,
    'the helper adds one blank row and AddTexture adds no further row')
local blank = GameTooltip:GetLeftLine(rowCount + 1)
assert(blank:GetText() == ' ', 'the separator row must stay blank')
local matches = 0
for _, region in ipairs({ GameTooltip:GetRegions() }) do
    if region:GetObjectType() == 'Texture' and region:IsShown()
        and region:GetTextureFilePath() == 'Interface\\Common\\UI-TooltipDivider-Transparent' then
        local _, relativeTo = region:GetPoint(1)
        if relativeTo == blank then
            matches = matches + 1
            assert(math.abs(region:GetHeight() - 10) < 0.01,
                'the original options must set divider height 10')
            assert(region:GetNumPoints() == 2,
                'the original All option must install two constraints')
            local firstPoint, firstTarget, firstRelative = region:GetPoint(1)
            local secondPoint, secondTarget, secondRelative = region:GetPoint(2)
            assert(firstTarget == blank and secondTarget == blank,
                'both All constraints must target the wrapped blank row')
            assert(
                (firstPoint == 'BOTTOMRIGHT' and secondPoint == 'TOPLEFT')
                or (firstPoint == 'TOPLEFT' and secondPoint == 'BOTTOMRIGHT'),
                'the All constraints must span the top and bottom edges')
            assert(
                (firstRelative == 'BOTTOMRIGHT' and secondRelative == 'TOPLEFT')
                or (firstRelative == 'TOPLEFT' and secondRelative == 'BOTTOMRIGHT'),
                'both All constraints must use matching blank-row edges')
        end
    end
end
assert(matches == 1, 'the original helper must attach exactly one divider')
"#;

const KEYSTONE_LINK_PROBE: &str = r#"
local function sellPriceLine(data)
    for _, line in ipairs(data.lines) do
        if line.type == Enum.TooltipDataLineType.SellPrice then
            return line
        end
    end
end

GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
for _, case in ipairs({
    {'keystone:184318:0:0:0:0:0:0:0', 184318, 1},
    {'keystone:164374:0:0:0:0:0:0:0', 164374, 50},
}) do
    GameTooltip:SetHyperlink(case[1], 17, 29, true)
    local info = GameTooltip:GetPrimaryTooltipInfo()
    local data = GameTooltip:GetPrimaryTooltipData()
    assert(info and info.getterName == 'GetHyperlink',
        'the original SetHyperlink accessor must select GetHyperlink')
    assert(data and data.id == case[2], 'each keystone link must retain its distinct item ID')
    local line = assert(sellPriceLine(data),
        'mode zero and clear preview must leave the distinct SellPrice row visible')
    assert(line.price == case[3] and line.maxPrice == -1,
        'the keystone route must use the no-instance mode-zero request')
    local expected = SELL_PRICE .. ': ' .. MoneyFormatterUtil.FormatMoney(case[3], GameTooltipMoneyFormat)
    local visible = false
    for index = 1, GameTooltip:NumLines() do
        visible = visible or GameTooltip:GetLeftLine(index):GetText() == expected
    end
    assert(visible, 'the original tooltip consumer must render the SellPrice row')
end
"#;

const EQUIPPED_GUID_PROBE: &str = r#"
A_Admin.EquipItem(16, 231268)
A_Admin.SetEquippedItemGUID(16, 'supplied-equipped-guid')
assert(not pcall(A_Admin.SetEquippedItemGUID, -1, 'missing-slot-guid'),
    'the GUID input must reject an empty equipment slot')
assert(not pcall(A_Admin.SetEquippedItemGUID, 16, ''),
    'the GUID input must reject an empty string')
GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
GameTooltip:SetInventoryItemByID(231268)
local info = GameTooltip:GetPrimaryTooltipInfo()
local data = GameTooltip:GetPrimaryTooltipData()
assert(info and info.getterName == 'GetInventoryItemByID',
    'the original inventory accessor must select GetInventoryItemByID')
assert(data and data.id == 231268 and data.guid == 'supplied-equipped-guid',
    'an equipped copy must publish its supplied instance GUID')
assert(GameTooltip:GetLeftLine(1):GetText() == 'Blastfurious Machete',
    'the equipped copy must retain its ordinary item tooltip')
"#;

prefork_full_ui_case! {
fn tooltip_residual_set_owner_releases_attachments(env: &WowLuaEnv) {
    env.exec(SET_OWNER_RELEASE_PROBE).unwrap();
}
}

prefork_full_ui_case! {
fn tooltip_residual_hide_releases_attachments(env: &WowLuaEnv) {
    env.exec(HIDE_RELEASE_PROBE).unwrap();
}
}

prefork_full_ui_case! {
fn tooltip_residual_original_social_separator_attaches_to_blank_row(env: &WowLuaEnv) {
    env.exec(SOCIAL_SEPARATOR_PROBE).unwrap();
}
}

prefork_full_ui_case! {
fn tooltip_residual_keystone_links_keep_sell_price_preview_clear(env: &WowLuaEnv) {
    env.exec(KEYSTONE_LINK_PROBE).unwrap();
}
}

prefork_full_ui_case! {
fn tooltip_residual_equipped_copy_publishes_supplied_guid(env: &WowLuaEnv) {
    env.exec(EQUIPPED_GUID_PROBE).unwrap();
}
}
