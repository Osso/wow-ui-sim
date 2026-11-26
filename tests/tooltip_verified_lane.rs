#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

const SELL_PRICE_ROUTE_PROBE: &str = r#"
A_Admin.SetMerchantItems({184318, 164374})
A_Admin.AddBagItem(0, 1, 184318, 1)
A_Admin.StartLootRoll(71, 60000, 'Roll One', '', 4, 0, 'item:184318')
A_Admin.StartLootRoll(72, 60000, 'Roll Two', '', 4, 0, 'item:164374')
GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')

local function sellPriceLine(data)
    for _, line in ipairs(data.lines) do
        if line.type == Enum.TooltipDataLineType.SellPrice then
            return line
        end
    end
end

local function visibleText(text)
    for index = 1, GameTooltip:NumLines() do
        if GameTooltip:GetLeftLine(index):GetText() == text then
            return true
        end
    end
    return false
end

for _, case in ipairs({{184318, 1}, {164374, 50}}) do
    GameTooltip:SetItemByID(case[1])
    local data = assert(GameTooltip:GetPrimaryTooltipData())
    local line = assert(sellPriceLine(data), 'eligible item must publish SellPrice')
    assert(line.price == case[2] and line.maxPrice == -1)
    local expected = SELL_PRICE .. ': ' .. MoneyFormatterUtil.FormatMoney(case[2], GameTooltipMoneyFormat)
    assert(visibleText(expected), 'original item consumer must render the distinct price')
end

GameTooltip:SetHyperlink('item:184318', nil, nil, false)
assert(sellPriceLine(GameTooltip:GetPrimaryTooltipData()).price == 1)
GameTooltip:SetHyperlink('item:184318', nil, nil, true)
assert(sellPriceLine(GameTooltip:GetPrimaryTooltipData()) == nil)
GameTooltip:SetMerchantItem(1)
assert(sellPriceLine(GameTooltip:GetPrimaryTooltipData()) == nil)
GameTooltip:SetBagItem(0, 1)
assert(sellPriceLine(GameTooltip:GetPrimaryTooltipData()) == nil,
    'only the bag route supplies the unresolved live-instance status input')
GameTooltip:SetLootRollItem(71)
assert(sellPriceLine(GameTooltip:GetPrimaryTooltipData()).price == 1)
GameTooltip:SetLootRollItem(72)
assert(sellPriceLine(GameTooltip:GetPrimaryTooltipData()).price == 50)
"#;

const SELL_PRICE_MAX_PROBE: &str = r#"
A_Admin.SetItemTooltipSellPriceQuantities(184318, 2, 5)
A_Admin.SetItemTooltipSellPriceQuantities(164374, 3, 100000000)
GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')

local function assertMaximum(itemID, current, maximum, expectedMax)
    A_Admin.SetItemTooltipSellPriceQuantities(itemID, current, maximum)
    GameTooltip:SetItemByID(itemID)
    local data = assert(GameTooltip:GetPrimaryTooltipData())
    local line
    for _, candidate in ipairs(data.lines) do
        if candidate.type == Enum.TooltipDataLineType.SellPrice then
            line = candidate
        end
    end
    assert(line and line.maxPrice == expectedMax,
        'maxPrice mismatch for ' .. itemID .. ': ' .. tostring(line and line.maxPrice))
    local visible = {}
    for index = 1, GameTooltip:NumLines() do
        visible[GameTooltip:GetLeftLine(index):GetText()] = true
    end
    local money = function(value)
        return MoneyFormatterUtil.FormatMoney(value, GameTooltipMoneyFormat)
    end
    assert(visible[SELL_PRICE .. ': ' .. money(line.price)], 'minimum SellPrice row not rendered')
    if expectedMax > 0 then
        assert(visible['    ' .. MINIMUM .. ': ' .. money(line.price)], 'MINIMUM row not rendered')
        assert(visible['    ' .. MAXIMUM .. ': ' .. money(expectedMax)], 'MAXIMUM row not rendered')
    else
        assert(not visible['    ' .. MAXIMUM .. ': ' .. money(line.price * maximum)])
    end
    return line.price, line.maxPrice
end

local priceOne, maxOne = assertMaximum(184318, 2, 5, 5)
local priceTwo, maxTwo = assertMaximum(164374, 3, 100000000, 705032704)
assert(priceOne ~= priceTwo and maxOne ~= maxTwo)
assertMaximum(184318, 5, 5, -1)
assertMaximum(184318, 6, 5, -1)
A_Admin.SetItemTooltipSellPriceQuantities(184318, nil, nil)
assertMaximum(184318, 0, 0, -1)
"#;

const PVP_REPLACEMENT_PROBE: &str = r#"
A_Admin.SetPlayerLevel(70)
A_Admin.SetSelectedPvpTalentIDs({})
A_Admin.SetPvpTalentTooltipRecord(91001, 116, 853, 75, 1)
A_Admin.SetPvpTalentTooltipRecord(91002, 116, 853, 75, 1)
A_Admin.SetPvpTalentTooltipRecord(91003, 116, 4294967295, 75, 1)
A_Admin.SetPvpTalentTooltipResolution(116, 133)
A_Admin.SetPvpTalentTooltipResolution(853, 133)
GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')

local function replacementCount(data)
    local count = 0
    for _, line in ipairs(data.lines) do
        if line.leftText and line.leftText:find(REPLACES_SPELL:format(C_TooltipInfo.GetSpellByID(133).lines[1].leftText), 1, true) then
            count = count + 1
        end
    end
    return count
end

GameTooltip:SetPvpTalent(91001)
local positive = assert(GameTooltip:GetPrimaryTooltipData())
assert(positive.id == 133 and replacementCount(positive) == 1)
assert(positive.lines[#positive.lines].leftText == UNLOCKED_AT_LEVEL:format(75))
assert(GameTooltip:GetLeftLine(GameTooltip:NumLines()):GetText() == UNLOCKED_AT_LEVEL:format(75))

A_Admin.SetPvpTalentTooltipResolution(116, nil)
A_Admin.SetPvpTalentTooltipResolution(853, 4294967295)
GameTooltip:SetPvpTalent(91002)
local missingName = assert(GameTooltip:GetPrimaryTooltipData())
assert(missingName.id == 116 and replacementCount(missingName) == 0)
assert(missingName.lines[#missingName.lines].leftText == UNLOCKED_AT_LEVEL:format(75))

A_Admin.SetPvpTalentTooltipResolution(853, nil)
GameTooltip:SetPvpTalent(91003)
local missingOriginal = assert(GameTooltip:GetPrimaryTooltipData())
assert(missingOriginal.id == 116 and replacementCount(missingOriginal) == 0)
assert(missingOriginal.lines[#missingOriginal.lines].leftText == UNLOCKED_AT_LEVEL:format(75))
"#;

const PVP_MISS_ANNOTATION_PROBE: &str = r#"
A_Admin.SetPlayerLevel(70)
A_Admin.SetSelectedPvpTalentIDs({})
A_Admin.SetPvpTalentTooltipRecord(91004, 116, 0, 75, 1)
A_Admin.SetPvpTalentTooltipRecord(91005, 116, 0, 80, 1)
A_Admin.SetPvpTalentTooltipResolution(116, 4294967295)
GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')

for _, case in ipairs({{91004, 75}, {91005, 80}}) do
    GameTooltip:SetPvpTalent(case[1])
    local data = assert(GameTooltip:GetPrimaryTooltipData(), 'annotations must still publish data')
    assert(#data.lines == 1)
    assert(data.lines[1].leftText == UNLOCKED_AT_LEVEL:format(case[2]))
    assert(data.lines[1].wrapText == nil)
    assert(GameTooltip:GetLeftLine(1):GetText() == UNLOCKED_AT_LEVEL:format(case[2]))
end
"#;

const SOCKET_PROJECTION_PROBE: &str = r#"
A_Admin.AddBagItem(0, 1, 1728, 1)
A_Admin.AddBagItem(0, 2, 1728, 1)
A_Admin.SetItemTooltipSocketProjection(0, 1, {
    {socketType = 'Meta', gemIcon = 'Interface\\Icons\\INV_Misc_Gem_01', gemItemID = 6948},
    {socketType = 'Red', gemItemID = 0},
})
A_Admin.SetItemTooltipSocketProjection(0, 2, {
    {socketType = 'Blue', gemIcon = 'Interface\\Icons\\INV_Misc_Gem_02', gemItemID = 127829},
})
GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')

local function assertProjection(slot, expected)
    GameTooltip:SetInventoryItemByID(1728)
    local data = assert(GameTooltip:GetPrimaryTooltipData())
    assert(data.guid == 'Item-0-' .. slot .. '-1728')
    local rows = {}
    for _, line in ipairs(data.lines) do
        if line.type == Enum.TooltipDataLineType.GemSocket then
            table.insert(rows, line)
        end
    end
    assert(#rows == #expected)
    for index, row in ipairs(rows) do
        assert(row.socketType == expected[index][1])
        assert(row.gemIcon == expected[index][2])
        assert(row.gemItemID == expected[index][3])
    end
    local textures = {}
    for _, child in ipairs({GameTooltip:GetChildren()}) do
        if child:GetObjectType() == 'Texture' and child:IsShown() then
            textures[child:GetTexture()] = true
        end
    end
    assert(textures['Interface\\Icons\\INV_Misc_Gem_01'] == (slot == 1))
    assert(textures['Interface\\Icons\\INV_Misc_Gem_02'] == (slot == 2))
    assert(textures['Interface\\ItemSocketingFrame\\UI-EmptySocket-Meta'] == (slot == 1))
    assert(textures['Interface\\ItemSocketingFrame\\UI-EmptySocket-Red'] == (slot == 1))
    assert(textures['Interface\\ItemSocketingFrame\\UI-EmptySocket-Blue'] == false)
end

assertProjection(1, {
    {'Meta', 'Interface\\Icons\\INV_Misc_Gem_01', 6948},
    {'Red', nil, 0},
})
A_Admin.RemoveBagItem(0, 1)
assertProjection(2, {
    {'Blue', 'Interface\\Icons\\INV_Misc_Gem_02', 127829},
})
"#;

const PENDING_ITEM_PROBE: &str = r#"
A_Admin.AddBagItem(0, 1, 128910, 1)
A_Admin.AddBagItem(0, 2, 158923, 1)
A_Admin.SetItemLocationInputs(105, false, {})
A_Admin.SetItemInteractionScenario(2, {128910}, 4, 7)
PendingSelectionEvents = {}
PendingOrderEvents = {}
PendingProbe = CreateFrame('Frame')
for _, event in ipairs({
    'ITEM_INTERACTION_ITEM_SELECTION_UPDATED', 'ITEM_LOCK_CHANGED',
    'ITEM_LOCKED', 'ITEM_UNLOCKED', 'CURSOR_CHANGED',
}) do
    PendingProbe:RegisterEvent(event)
end
PendingProbe:SetScript('OnEvent', function(_, event, payload)
    table.insert(PendingOrderEvents, event)
    if event == 'ITEM_INTERACTION_ITEM_SELECTION_UPDATED' then
        local itemID
        if payload and payload.bagID ~= nil and payload.slotIndex ~= nil then
            local key = tostring(payload.bagID) .. ':' .. tostring(payload.slotIndex)
            itemID = key == '0:1' and 128910 or key == '0:2' and 158923 or nil
        end
        table.insert(PendingSelectionEvents, itemID or false)
    end
end)

local first = ItemLocation:CreateFromBagAndSlot(0, 1)
local second = ItemLocation:CreateFromBagAndSlot(0, 2)
assert(select('#', C_ItemInteraction.SetPendingItem(first)) == 1)
assert(C_ItemInteraction.SetPendingItem(first) == true)
assert(C_Item.IsLocked(first) and PendingSelectionEvents[1] == 128910)
assert(not CursorHasItem())

C_Container.PickupContainerItem(0, 1)
PendingSelectionEvents = {}
PendingOrderEvents = {}
assert(C_ItemInteraction.SetPendingItem(first) == true)
assert(CursorHasItem() and #PendingSelectionEvents == 0 and #PendingOrderEvents == 0)
ClearCursor()

PendingSelectionEvents = {}
assert(C_ItemInteraction.SetPendingItem(second) == false)
assert(not C_Item.IsLocked(first) and not C_Item.IsLocked(second))
assert(PendingSelectionEvents[1] == false)
assert(C_ItemInteraction.SetPendingItem(nil) == false)
assert(PendingSelectionEvents[2] == false)

local ok, reason = pcall(C_ItemInteraction.SetPendingItem, {})
assert(not ok and tostring(reason):find('Usage: local success = C_ItemInteraction.SetPendingItem([item])', 1, true))
assert(#PendingSelectionEvents == 2)

A_Admin.SetItemInteractionScenario(2, {158923}, 4, 7)
PendingSelectionEvents = {}
assert(C_ItemInteraction.SetPendingItem(second) == true)
assert(C_Item.IsLocked(second) and PendingSelectionEvents[1] == 158923)
PendingSelectionEvents = {}
C_ItemInteraction.InitializeFrame()
assert(select('#', C_TooltipInfo.GetItemInteractionItem()) == 0)
assert(C_Item.IsLocked(second) and #PendingSelectionEvents == 0)
C_ItemInteraction.Reset()
assert(C_Item.IsLocked(second) and #PendingSelectionEvents == 0)

A_Admin.SetItemInteractionScenario(2, {128910}, 4, 7)
assert(C_ItemInteraction.SetPendingItem(first) == true)
C_ItemInteraction.ClearPendingItem()
assert(not C_Item.IsLocked(first) and PendingSelectionEvents[#PendingSelectionEvents] == false)
"#;

const POSSESSION_PROBE: &str = r#"
PossessionEvents = {}
PossessionProbe = CreateFrame('Frame')
PossessionProbe:RegisterEvent('UPDATE_POSSESS_BAR')
PossessionProbe:SetScript('OnEvent', function()
    local data = C_TooltipInfo.GetPossession(1)
    table.insert(PossessionEvents, data and data.id or 0)
end)

A_Admin.SetPossessionSpellID(116)
assert(C_TooltipInfo.GetPossession(1).id == 116)
assert(PossessionEvents[1] == 116)
A_Admin.SetPossessionSpellID(116)
assert(#PossessionEvents == 1)

A_Admin.SetPossessionObject({
    identityFirst = 11, identitySecond = 22,
    candidates = {
        {spellID = 4294967295, tested = false, effects = {2}, identityTypePresent = false, comparedIdentityFirst = 0, comparedIdentitySecond = 0},
        {spellID = 116, tested = true, effects = {2}, identityTypePresent = false, comparedIdentityFirst = 0, comparedIdentitySecond = 0},
        {spellID = 116, tested = false, effects = {}, identityTypePresent = false, comparedIdentityFirst = 0, comparedIdentitySecond = 0},
        {spellID = 133, tested = false, effects = {2}, identityTypePresent = false, comparedIdentityFirst = 0, comparedIdentitySecond = 0},
        {spellID = 116, tested = false, effects = {6}, identityTypePresent = false, comparedIdentityFirst = 0, comparedIdentitySecond = 0},
    },
})
A_Admin.UpdatePossessionPlayer(11, 22)
assert(C_TooltipInfo.GetPossession(1).id == 133 and PossessionEvents[2] == 133)
A_Admin.UpdatePossessionPlayer(12, 22)
assert(C_TooltipInfo.GetPossession(1).id == 133 and #PossessionEvents == 2)

A_Admin.SetPossessionObject({
    identityFirst = 11, identitySecond = 22,
    candidates = {
        {spellID = 116, tested = false, effects = {236}, identityTypePresent = true, comparedIdentityFirst = 11, comparedIdentitySecond = 22},
    },
})
A_Admin.UpdatePossessionPlayer(11, 22)
assert(C_TooltipInfo.GetPossession(1).id == 116 and PossessionEvents[3] == 116)
A_Admin.UpdatePossessionPlayer(0, 0)
assert(select('#', C_TooltipInfo.GetPossession(1)) == 0 and PossessionEvents[4] == 0)
assert(select('#', C_TooltipInfo.GetPossession(2)) == 0)
"#;

prefork_full_ui_case! {
fn sell_price_routes_pass_mode_preview_and_instance_inputs(env: &WowLuaEnv) {
    env.exec(SELL_PRICE_ROUTE_PROBE).unwrap();
}
}

prefork_full_ui_case! {
fn prepared_max_price_uses_quantity_and_wraps_at_32_bits(env: &WowLuaEnv) {
    env.exec(SELL_PRICE_MAX_PROBE).unwrap();
}
}

prefork_full_ui_case! {
fn pvp_replacement_misses_reach_the_original_talent_consumer(env: &WowLuaEnv) {
    env.exec(PVP_REPLACEMENT_PROBE).unwrap();
}
}

prefork_full_ui_case! {
fn pvp_annotations_continue_after_primary_spell_name_miss(env: &WowLuaEnv) {
    env.exec(PVP_MISS_ANNOTATION_PROBE).unwrap();
}
}

prefork_full_ui_case! {
fn inventory_tooltip_copies_each_instance_socket_projection(env: &WowLuaEnv) {
    env.exec(SOCKET_PROJECTION_PROBE).unwrap();
}
}

prefork_full_ui_case! {
fn pending_item_local_lifecycle_uses_public_apis(env: &WowLuaEnv) {
    env.exec(PENDING_ITEM_PROBE).unwrap();
    let input = env
        .state()
        .borrow()
        .item_interaction_tooltip_input
        .clone()
        .expect("scenario input");
    assert_eq!((input.selection_id, input.record_id), (-1, 0));
}
}

prefork_full_ui_case! {
fn possession_local_rescan_writes_before_change_events(env: &WowLuaEnv) {
    env.exec(POSSESSION_PROBE).unwrap();
}
}
