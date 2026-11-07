//! WM63 D56 tooltip-data identity, driven through the original Blizzard
//! accessors (`Blizzard_SharedXMLGame/Tooltip/TooltipDataHandler.lua:241-309`):
//! every publication is a fresh table with a fresh line array, `dataInstanceID`
//! comes from an incrementing counter, supplied sides publish packed RGB without
//! alpha, and no inline segment arrays are attached.
#![cfg(any(feature = "profile-retail", feature = "client-ptr"))]

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

/// `TooltipDataHandler.lua:525` maps `SetItemByID` to `C_TooltipInfo.GetItemByID`;
/// the pinned declaration is `TooltipInfoDocumentation.lua:439`. The item
/// producer is `tooltip_for_item_context` -> `tooltip_for_item_info`, which runs
/// the shared D56 line constructor for every row.
#[test]
fn tooltip_data_identity_item_accessor_returns_fresh_tables_per_call() {
    let env = tooltip_full_env_helpers::setup_full_env();
    env.exec(
        r#"
        GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
        GameTooltip:SetItemByID(6948)
        local first = GameTooltip:GetPrimaryTooltipData()
        GameTooltip:SetItemByID(6948)
        local second = GameTooltip:GetPrimaryTooltipData()
        assert(first and second, 'D56 publication must exist for both calls')
        assert(first ~= second, 'each call must publish a fresh data table')
        assert(first.lines ~= second.lines, 'each call must publish a fresh line array')
        assert(type(first.dataInstanceID) == 'number' and type(second.dataInstanceID) == 'number')
        assert(second.dataInstanceID == first.dataInstanceID + 1,
            'D56 counter must increase by one: ' .. tostring(first.dataInstanceID) .. ' -> ' .. tostring(second.dataInstanceID))
        assert(first.type == Enum.TooltipDataType.Item and second.type == Enum.TooltipDataType.Item)
        assert(first.id == 6948 and second.id == 6948, 'item identity id')
        assert(#first.lines > 1 and #first.lines == #second.lines, 'item rows must be published')
        for index, line in ipairs(first.lines) do
            assert(line ~= second.lines[index], 'line ' .. index .. ' must be a fresh table')
            assert(type(line.type) == 'number', 'common line constructor publishes numeric type')
            assert(line.leftText ~= nil and line.leftColor ~= nil, 'supplied left side publishes text and color')
            assert(line.rightText == nil and line.rightColor == nil, 'absent right side publishes nothing')
            assert(line.leftColorSegments == nil, 'D56 common path has no segment arrays')
            assert(line.leftColor.a == nil, 'D56 color carries no alpha')
            local r, g, b = line.leftColor:GetRGB()
            for _, component in ipairs({r, g, b}) do
                local byte = math.floor(component * 255 + 0.5)
                assert(math.abs(component - byte * 0.003921568859368563) < 0.0000001,
                    'D56 color uses packed bytes scaled by 1/255')
            end
        end
        assert(first.lines[1].leftText == 'Hearthstone', 'item name line')
        assert(first.lines[2].leftText == 'Item Level 1', 'item level line')
    "#,
    )
    .unwrap();
}

/// `TooltipDataHandler.lua:585` maps `SetEquipmentSet` to `C_TooltipInfo.GetEquipmentSet`.
/// Its header row is a double line: `publish_header` supplies both sides through
/// the shared `push_tooltip_line`/`publish_side` publication path, so the D56
/// color rule (packed RGB, no alpha) applies to the right color as well.
#[test]
fn tooltip_data_identity_double_line_right_color_is_packed() {
    let env = tooltip_full_env_helpers::setup_full_env();
    env.exec(
        r#"
        for slot = 1, 19 do
            A_Admin.UnequipItem(slot)
        end
        A_Admin.EquipItem(1, 221125)
        C_EquipmentSet.CreateEquipmentSet('Raid', 'Interface\\Icons\\INV_Misc_QuestionMark')
        local set_id = C_EquipmentSet.GetEquipmentSetIDs()[1]
        GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
        GameTooltip:SetEquipmentSet(set_id)
        local data = GameTooltip:GetPrimaryTooltipData()
        assert(data and data.type == Enum.TooltipDataType.EquipmentSet, 'equipment-set publication')
        local header = data.lines[1]
        assert(header.leftText ~= nil and header.leftColor ~= nil, 'supplied left side')
        assert(header.leftColor.a == nil, 'left color carries no alpha')
        assert(header.rightText ~= nil, 'header publishes the item count on the right')
        assert(header.rightColor ~= nil, 'supplied right side publishes its color')
        assert(header.rightColor.a == nil, 'D56 color carries no alpha')
        for _, color in ipairs({header.leftColor, header.rightColor}) do
            local r, g, b = color:GetRGB()
            for _, component in ipairs({r, g, b}) do
                local byte = math.floor(component * 255 + 0.5)
                assert(math.abs(component - byte * 0.003921568859368563) < 0.0000001,
                    'D56 color uses packed bytes scaled by 1/255')
            end
        end
    "#,
    )
    .unwrap();
}
