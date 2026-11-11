//! Round 7: native argument readers, profile publication and no-value misses.
//! Populated payload coverage is limited to the existing owned-item model.

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

#[test]
fn tooltip_round7_registration_matches_native_profile_evidence() {
    use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    let modern = matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    );
    let all_getters = matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever);
    for (name, published) in [
        ("GetAzeriteEssence", modern),
        ("GetAzeriteEssenceSlot", modern),
        ("GetItemInteractionItem", all_getters),
        ("GetLFGDungeonReward", all_getters),
        ("GetLFGDungeonShortageReward", all_getters),
        ("GetPossession", all_getters),
        ("GetQuestLogSpecialItem", all_getters),
        ("GetQuestPartyProgress", all_getters),
        ("GetRecipeRankInfo", all_getters),
        ("GetRuneforgeResultItem", all_getters),
        ("GetSlottedKeystone", all_getters),
    ] {
        let source = format!("return type(rawget(C_TooltipInfo, '{name}')) == 'function'");
        let actual: bool = env.eval(&source).unwrap();
        assert_eq!(actual, published, "{ACTIVE:?}: {name}");
    }
}

tooltip_selection_getters_fixture! {
    use rilua::LuaApiMut;
    use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
    use wow_ui_sim::lua_api::WowLuaEnv;

    const CASES: &str = r#"
        local cases = {
            {'GetAzeriteEssence', {1}, 'essenceID [, rank]', 1, 2, {1}},
            {'GetAzeriteEssenceSlot', {0}, 'slot', 1, 1, {1}},
        }
        if R7AllGetters then
            for _, case in ipairs({
                {'GetLFGDungeonReward', {1, 0}, 'dungeonID, lootIndex', 2, 2, {1, 2}},
                {
                    'GetLFGDungeonShortageReward', {1, 0, 0},
                    'dungeonID, shortageSeverity, lootIndex', 3, 3, {1, 2, 3},
                },
                {'GetPossession', {0}, 'slot', 1, 1, {1}},
                {'GetQuestLogSpecialItem', {1}, 'questIndex', 1, 1, {1}},
                {'GetQuestPartyProgress', {1}, 'questID [, omitTitle, ignoreActivePlayer]', 1, 3, {1}},
                {'GetRecipeRankInfo', {1, 1}, 'recipeID, rank', 2, 2, {1, 2}},
                {
                    'GetRuneforgeResultItem', {'Opaque-1', 40},
                    'itemGUID, itemLevel [, powerID, modifiers]', 2, 4, {2},
                },
            }) do cases[#cases + 1] = case end
        end
    "#;

    fn env_with_profile() -> WowLuaEnv {
        let env = WowLuaEnv::new().unwrap();
        let all = ACTIVE != ClientProfile::Ptr;
        env.exec(&format!("R7AllGetters = {all}")).unwrap();
        env
    }

    fn publish_secret(env: &WowLuaEnv) {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        rilua::table_security::register_table_security(&mut lua).unwrap();
        let secret = rilua::table_security::wrap_host_secret_number(lua.state_mut(), 128910.0);
        lua.state_mut().push(secret);
        lua.set_global_val("R7Secret", secret).unwrap();
        lua.state_mut().pop();
    }

    #[test]
    fn tooltip_round7_required_arguments_report_native_usage() {
        let env = env_with_profile();
        env.exec(&format!(r#"{CASES}
            for _, case in ipairs(cases) do
                local name, args, signature, required = unpack(case)
                local getter = assert(rawget(C_TooltipInfo, name), name .. ' missing')
                local usage = 'Usage: local data = C_TooltipInfo.' .. name .. '(' .. signature .. ')'
                for position = 1, required do
                    local copy = {{unpack(args)}}
                    copy[position] = nil
                    local ok, message = pcall(getter, unpack(copy, 1, required))
                    assert(not ok and message:find(usage, 1, true), name .. ': ' .. tostring(message))
                    copy[position] = {{}}
                    ok, message = pcall(getter, unpack(copy, 1, required))
                    assert(not ok and message:find(usage, 1, true), name .. ': bad conversion: ' .. tostring(message))
                end
            end
        "#)).unwrap();
    }

    #[test]
    fn tooltip_round7_numeric_readers_convert_strings_and_truncate_before_lookup() {
        let env = env_with_profile();
        env.exec(&format!(r#"{CASES}
            for _, case in ipairs(cases) do
                local name, args, _, _, _, signedPositions = unpack(case)
                local getter = assert(rawget(C_TooltipInfo, name), name .. ' missing')
                for _, position in ipairs(signedPositions) do
                    if type(args[position]) == 'number' then
                        local copy = {{unpack(args)}}
                        copy[position] = tostring(args[position] + 0.9)
                        local result_count = select('#', getter(unpack(copy)))
                        assert(result_count == 0, name .. ': string/fraction must reach miss arm')
                        for _, outside in ipairs({{-2147483649, 2147483648, math.huge, 0/0}}) do
                            copy[position] = outside
                            local ok, message = pcall(getter, unpack(copy))
                            local has_range_error = message:find(
                                'outside of expected range -2147483648 to 2147483647', 1, true)
                            assert(not ok and has_range_error,
                                name .. ': range error: ' .. tostring(message))
                        end
                    end
                end
            end
            assert(select('#', C_TooltipInfo.GetAzeriteEssence(1, '0.9')) == 0)
            for _, rank in ipairs({{-1, 4294967296}}) do
                local ok, message = pcall(C_TooltipInfo.GetAzeriteEssence, 1, rank)
                local has_range_error = message:find(
                    'outside of expected range 0 to 4294967295', 1, true)
                assert(not ok and has_range_error)
            end
            assert(select('#', C_TooltipInfo.GetAzeriteEssenceSlot(259)) == 0, 'low byte 3 accepted')
            assert(select('#', C_TooltipInfo.GetAzeriteEssenceSlot(-256)) == 0, 'low byte 0 accepted')
            for _, slot in ipairs({{4, 260, -1}}) do
                local ok, message = pcall(C_TooltipInfo.GetAzeriteEssenceSlot, slot)
                local usage = 'Usage: local data = C_TooltipInfo.GetAzeriteEssenceSlot(slot)'
                assert(not ok and message:find(usage, 1, true))
            end
        "#)).unwrap();
    }

    #[test]
    fn tooltip_round7_misses_return_no_values_through_selected_original_accessors() {
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.exec(&format!("R7AllGetters = {}", ACTIVE != ClientProfile::Ptr)).unwrap();
        env.exec(&format!(r#"{CASES}
            local shared_xml_loaded = C_AddOns.IsAddOnLoaded('Blizzard_SharedXMLGame')
            assert(shared_xml_loaded, 'selected original tooltip consumer must load')
            if R7AllGetters then
                cases[#cases + 1] = {{'GetItemInteractionItem', {{}}}}
                cases[#cases + 1] = {{'GetSlottedKeystone', {{}}}}
            end
            for _, case in ipairs(cases) do
                local name, args = unpack(case)
                local getter = assert(rawget(C_TooltipInfo, name), name .. ' missing')
                assert(select('#', getter(unpack(args))) == 0, name .. ': miss is no values, not nil')
                local setter = 'Set' .. name:sub(4)
                local source = debug.getinfo(GameTooltip[setter], 'S').source
                assert(source:find('TooltipDataHandler.lua', 1, true), 'original accessor')
                GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
                GameTooltip[setter](GameTooltip, unpack(args))
                local primary_data = GameTooltip:GetPrimaryTooltipData()
                assert(primary_data == nil, name .. ': original accessor must not fabricate data')
            end
        "#)).unwrap();
    }

    #[test]
    fn tooltip_round7_secret_readers_reject_tainted_arguments_and_recover() {
        let env = env_with_profile();
        publish_secret(&env);
        env.exec(&format!(r#"{CASES}
            local function tainted()
                for _, case in ipairs(cases) do
                    local name, args, _, _, count = unpack(case)
                    local getter = assert(rawget(C_TooltipInfo, name), name .. ' missing')
                    for position = 1, count do
                        local copy = {{unpack(args)}}
                        copy[position] = R7Secret
                        local ok, message = pcall(getter, unpack(copy, 1, count))
                        local has_secret_error = message:find(
                            'Secret values are only allowed during untainted execution for this argument.',
                            1, true)
                        assert(not ok and has_secret_error,
                            name .. ' argument ' .. position .. ': ' .. tostring(message))
                    end
                    assert(select('#', getter(unpack(args))) == 0, 'ordinary arguments still work')
                end
                if R7AllGetters then
                    local interaction_count = select(
                        '#', C_TooltipInfo.GetItemInteractionItem(R7Secret))
                    local keystone_count = select(
                        '#', C_TooltipInfo.GetSlottedKeystone(R7Secret))
                    assert(interaction_count == 0, 'no argument reader')
                    assert(keystone_count == 0, 'no argument reader')
                end
            end
            debug.setobjecttaint(tainted, 'Round7Probe')
            tainted()
            assert(issecretvalue(R7Secret), 'input remains secret')
            assert(select('#', C_TooltipInfo.GetAzeriteEssence(R7Secret)) == 0, 'secure reader can unwrap')
            if R7AllGetters then
                for _, position in ipairs({{2, 3}}) do
                    local args = {{1, false, false}}
                    args[position] = 1
                    local ok = pcall(C_TooltipInfo.GetQuestPartyProgress, unpack(args))
                    assert(not ok, 'optional boolean is not truthiness')
                end
                local accepted_bad_modifiers = pcall(
                    C_TooltipInfo.GetRuneforgeResultItem, 'Opaque-1', 40, nil, 7)
                assert(not accepted_bad_modifiers, 'modifier table type')
            end
        "#)).unwrap();
    }

    #[test]
    fn tooltip_round7_runeforge_original_accessor_uses_owned_guid_and_level() {
        if ACTIVE == ClientProfile::Ptr {
            return;
        }
        let env = tooltip_full_env_helpers::load_full_env_without_post_load();
        env.exec(r#"
            A_Admin.AddBagItem(0, 3, 1728, 1)
            A_Admin.AddBagItem(1, 2, 231268, 1)
            for _, case in ipairs({
                {0, 3, 1728, 40, "Teebu's Blazing Longsword"},
                {1, 2, 231268, 80, 'Blastfurious Machete'},
            }) do
                local guid = C_Item.GetItemGUID({bagID=case[1], slotIndex=case[2]})
                GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
                GameTooltip:SetRuneforgeResultItem(guid, case[4])
                local info = GameTooltip:GetPrimaryTooltipInfo()
                local data = GameTooltip:GetPrimaryTooltipData()
                local has_runeforge_accessor = info and info.getterName == 'GetRuneforgeResultItem'
                assert(has_runeforge_accessor, 'runeforge accessor missing')
                local has_owned_identity = data and data.type == 0
                    and data.id == case[3] and data.guid == guid
                assert(has_owned_identity, 'owned instance identity')
                assert(data.overrideItemLevel == case[4], 'supplied display level')
                assert(data.lines[1].leftText == case[5], 'ordinary shared item producer')
                assert(data.lines[2].leftText == 'Item Level ' .. case[4], 'displayed level reaches lines')
                assert(GameTooltip:GetLeftLine(1):GetText() == case[5], 'original consumer presents item')
            end
            local function assert_runeforge_miss(guid, reason)
                local result_count = select(
                    '#', C_TooltipInfo.GetRuneforgeResultItem(guid, 40))
                assert(result_count == 0, reason)
            end
            assert_runeforge_miss('Item-0-4-1728', 'GUID must resolve, not item id alone')
            assert_runeforge_miss('garbage', 'unparseable GUID is a miss')
            assert_runeforge_miss('Opaque-123', 'unresolved alias is a miss')
            assert_runeforge_miss(123, 'numeric GUID conversion reaches miss')
        "#).unwrap();
    }

    #[test]
    fn tooltip_round7_item_num_sockets_checks_secret_input_without_guessing_output_policy() {
        let env = env_with_profile();
        publish_secret(&env);
        env.exec(r#"
            local function tainted()
                local ok, message = pcall(C_Item.GetItemNumSockets, R7Secret)
                local has_secret_error = message:find(
                    'Secret values are only allowed during untainted execution for this argument.',
                    1, true)
                assert(not ok and has_secret_error, tostring(message))
                assert(C_Item.GetItemNumSockets(128910) == 2, 'plain input still works')
            end
            debug.setobjecttaint(tainted, 'Round7Probe')
            tainted()
            assert(C_Item.GetItemNumSockets(R7Secret) == 2, 'secure caller must read the wrapped item ID')
            assert(issecretvalue(R7Secret), 'do not declassify caller input')
            assert(select('#', C_Item.GetItemNumSockets(999999999)) == 1, 'integer miss, not no values')
        "#).unwrap();
    }
}
