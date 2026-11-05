//! WM94 §6: the original `SetEquipmentSet` accessor summarises a saved set:
//! name plus item count, equipped/inventory/ignored counts and the missing-slot
//! list in reverse slot order.

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

tooltip_selection_getters_fixture! {
    fn setup_sets(env: &wow_ui_sim::lua_api::WowLuaEnv) {
        env.exec(
            r#"
            A_Admin.EquipItem(1, 221125)
            A_Admin.EquipItem(16, 231268)
            C_EquipmentSet.CreateEquipmentSet('Raid', 'Interface\\Icons\\INV_Misc_QuestionMark')
            A_Admin.EquipItem(5, 221176)
            C_EquipmentSet.CreateEquipmentSet('Banked', 'Interface\\Icons\\INV_Misc_QuestionMark')
            A_Admin.UnequipItem(5)
            A_Admin.AddBagItem(0, 3, 221176, 1)
            A_Admin.EquipItem(3, 221140)
            C_EquipmentSet.CreateEquipmentSet('Mythic', 'Interface\\Icons\\INV_Misc_QuestionMark')
            C_EquipmentSet.IgnoreSlotForSave(3)
            C_EquipmentSet.SaveEquipmentSet(3)
            "#,
        )
        .unwrap();
    }

    #[test]
    fn tooltip_equipment_set_original_accessor_reports_the_saved_set() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        setup_sets(&env);
        env.exec(
            r#"
            local ids = C_EquipmentSet.GetEquipmentSetIDs()
            assert(#ids == 3, 'three saved sets')
            for index, case in ipairs({
                {1, 'Raid', 2, 2, 0, 0, 17},
                {2, 'Banked', 3, 2, 1, 0, 16},
                {3, 'Mythic', 3, 3, 0, 1, 16},
            }) do
                local set_id, name, items, equipped, in_bags, ignored, missing = unpack(case)
                GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
                GameTooltip:SetEquipmentSet(set_id)
                local info = GameTooltip:GetPrimaryTooltipInfo()
                local data = GameTooltip:GetPrimaryTooltipData()
                assert(info and info.getterName == 'GetEquipmentSet', 'equipment-set accessor missing')
                assert(data and data.type == Enum.TooltipDataType.EquipmentSet and data.id == set_id)
                local header = data.lines[1]
                assert(header.leftText == name, 'set name must lead the summary')
                assert(header.rightText == string.format(ITEMS_VARIABLE_QUANTITY, items),
                    'header right side is the item count')
                local counts = 0
                for _, line in ipairs(data.lines) do
                    if line == header then
                    elseif line.leftText == string.format(ITEMS_EQUIPPED, equipped)
                        or line.leftText == string.format(ITEMS_IN_INVENTORY, in_bags)
                        or line.leftText == string.format(ITEM_SLOTS_IGNORED, ignored) then
                        counts = counts + 1
                    end
                end
                local expected_counts = (equipped > 0 and 1 or 0)
                    + (in_bags > 0 and 1 or 0)
                    + (ignored > 0 and 1 or 0)
                assert(counts == expected_counts, 'only nonzero detail counts are published')
                assert(equipped + in_bags + ignored > 0, 'fixture must publish at least one count')
                if equipped > 0 then
                    local found = false
                    for _, line in ipairs(data.lines) do
                        found = found or line.leftText == string.format(ITEMS_EQUIPPED, equipped)
                    end
                    assert(found, 'equipped count line')
                end
                local missing_lines = #data.lines - 1 - expected_counts
                assert(missing_lines == missing,
                    ('%s missing-slot lines: got %d, want %d'):format(name, missing_lines, missing))
            end
            "#,
        )
        .unwrap();
    }

    #[test]
    fn tooltip_equipment_set_missing_slots_use_labels_in_reverse_order() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        setup_sets(&env);
        env.exec(
            r#"
            local labels = {
                HEADSLOT, NECKSLOT, SHOULDERSLOT, SHIRTSLOT, CHESTSLOT, WAISTSLOT, LEGSSLOT,
                FEETSLOT, WRISTSLOT, HANDSSLOT, FINGER0SLOT, FINGER1SLOT, TRINKET0SLOT,
                TRINKET1SLOT, BACKSLOT, MAINHANDSLOT, SECONDARYHANDSLOT, RANGEDSLOT, TABARDSLOT,
            }
            local function expected_missing(filled)
                local out = {}
                for slot = #labels, 1, -1 do
                    if not filled[slot] then
                        out[#out + 1] = string.format(ITEM_MISSING, labels[slot])
                    end
                end
                return out
            end
            local cases = {
                {1, {[1] = true, [16] = true}, 2},
                {2, {[1] = true, [5] = true, [16] = true}, 2},
                {3, {[1] = true, [3] = true, [16] = true}, 3},
            }
            for _, case in ipairs(cases) do
                local data = C_TooltipInfo.GetEquipmentSet(case[1])
                local expected = expected_missing(case[2])
                local start = #data.lines - #expected + 1
                for offset, text in ipairs(expected) do
                    assert(data.lines[start + offset - 1].leftText == text,
                        ('missing slot order at line %d'):format(start + offset - 1))
                end
                local red = data.lines[#data.lines]
                if data.dataInstanceID then
                    local r, g, b = red.leftColor:GetRGB()
                    assert(r == ERROR_COLOR.r and g == ERROR_COLOR.g and b == ERROR_COLOR.b,
                        'missing-slot text uses the error color')
                end
            end
            assert(select('#', C_TooltipInfo.GetEquipmentSet(99)) == 0,
                'unknown saved set must take the missing route')
            "#,
        )
        .unwrap();
    }
}
