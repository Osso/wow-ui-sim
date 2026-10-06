//! B08 cached Type/Nilable/InnerType contracts through public producers and inputs.
#![cfg(feature = "retail-12-0-0")]
use wow_ui_sim::c_api::crafting_reagents::*;
use wow_ui_sim::lua_api::{WowLuaEnv, state::CurrencyInfo};

#[test]
fn crafting_schematic_reagents_are_nested_identities() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local slot = C_TradeSkillUI.GetRecipeSchematic(100001).reagentSlotSchematics[1]
        assert(slot.dataSlotType == Enum.TradeskillSlotDataType.Reagent)
        assert(slot.reagents[1].itemID == 210934)
        assert(slot.reagents[1].quantityRequired == nil)
        assert(slot.reagents[1].reagentType == nil)
    "#,
    )
    .unwrap();
}

#[test]
fn crafting_order_rejects_legacy_flat_reagents_without_posting() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local ok = pcall(C_CraftingOrders.PlaceNewOrder, {
            skillLineAbilityID = 100001, orderType = 0, orderDuration = 0,
            tipAmount = 10, customerNotes = 'bad identity',
            reagentInfos = {{itemID=210934, quantity=2}}, craftingReagentItems={},
        })
        assert(not ok)
        assert(#C_CraftingOrders.GetMyOrders() == 0)
    "#,
    )
    .unwrap();
}

#[test]
fn crafting_order_requires_host_placement_metadata() {
    let env = WowLuaEnv::new().unwrap();
    env.state()
        .borrow_mut()
        .crafting
        .reagents
        .order_recipes
        .insert(9001, 100001);
    env.exec(r#"
        assert(not pcall(C_CraftingOrders.PlaceNewOrder, {
            skillLineAbilityID=9001,orderType=0,orderDuration=0,tipAmount=0,customerNotes='',
            reagentInfos={{reagent={itemID=210934},quantity=12},{reagent={itemID=210937},quantity=2}},
            craftingReagentItems={},
        }))
        assert(#C_CraftingOrders.GetMyOrders()==0)
    "#).unwrap();
}

fn placement_details(
    min_quality: i32,
    crafter_name: Option<&str>,
    expiration_time: f64,
) -> CraftingOrderDetails {
    CraftingOrderDetails {
        order_state: 1,
        expiration_time,
        claim_end_time: 345.5,
        min_quality,
        consortium_cut: 77.0,
        is_fulfillable: false,
        reagent_state: 2,
        customer_guid: Some("Player-1-42".into()),
        customer_name: Some("Customer-Realm".into()),
        crafter_guid: None,
        crafter_name: crafter_name.map(str::to_owned),
        npc_customer_creature_id: None,
        output_item_hyperlink: None,
        output_item_guid: None,
        recraft_item_hyperlink: None,
        npc_order_rewards: vec![OrderReward {
            item_link: None,
            currency_type: Some(2803),
            count: 3,
        }],
        npc_crafting_order_set_id: 18,
        npc_treasure_id: 19,
    }
}

fn fixture() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut state = env.state().borrow_mut();
        let inputs = &mut state.crafting.reagents;
        let mut slot = ReagentSlotSchematic::required_item(1, 210934, 4);
        slot.reagents.push(CraftingReagent::Currency(2803));
        slot.variable_quantities = vec![
            RegularReagentInfo {
                reagent: CraftingReagent::Item(210934),
                quantity: 4,
            },
            RegularReagentInfo {
                reagent: CraftingReagent::Currency(2803),
                quantity: 6,
            },
        ];
        slot.slot_info = Some(ReagentSlotInfo {
            mcr_slot_id: 42,
            required_skill_rank: 50,
            slot_text: Some("Choose metal or currency".into()),
        });
        slot.order_source = Some(1);
        inputs.recipe_slots.insert(
            100001,
            vec![slot, ReagentSlotSchematic::required_item(2, 210937, 2)],
        );
        inputs.order_recipes.insert(9001, 100001);
        inputs.placement_results.extend([
            placement_details(4, Some("ActualCrafter-Realm"), 1234.5),
            placement_details(0, None, 2345.5),
        ]);
        inputs.item_modifications.insert(
            "Item-1-42".into(),
            vec![
                ItemSlotModification {
                    data_slot_index: 1,
                    reagent: CraftingReagent::Currency(2803),
                },
                ItemSlotModification {
                    data_slot_index: 2,
                    reagent: CraftingReagent::Item(210937),
                },
            ],
        );
        inputs.resource_returns.insert(
            100001,
            vec![
                RegularReagentInfo {
                    reagent: CraftingReagent::Currency(2803),
                    quantity: 2,
                },
                RegularReagentInfo {
                    reagent: CraftingReagent::Item(210937),
                    quantity: 1,
                },
            ],
        );
        state.currency_info.insert(
            2803,
            CurrencyInfo {
                currency_id: 2803,
                name: "Fixture currency".into(),
                quantity: 20,
                ..Default::default()
            },
        );
    }
    env.exec("A_Admin.ClearBags(); A_Admin.AddBagItem(0, 1, 210937, 10)")
        .unwrap();
    env
}

/// Cache-generated declarations are the serialized external contract. Read them
/// as test data, never inspect simulator source or internal dispatch choices.
fn install_cached_shape_checker(env: &WowLuaEnv) {
    let docs = wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(
        env!("CARGO_MANIFEST_DIR"),
    ))
    .join("Blizzard_APIDocumentationGenerated");
    let mut definitions = Vec::new();
    for (file, names) in [
        (
            "TradeSkillUITypesDocumentation.lua",
            &[
                "CraftingReagent",
                "CraftingItemSlotModification",
                "CraftingReagentInfo",
                "RegularReagentInfo",
                "CraftingReagentSlotSchematic",
                "CraftingReagentSlotInfo",
                "CraftingVariableQuantities",
                "CraftingResourceReturnInfo",
            ][..],
        ),
        (
            "CraftingOrderUISharedDocumentation.lua",
            &[
                "NewCraftingOrderInfo",
                "CraftingOrderReagentInfo",
                "CraftingOrderInfo",
                "CraftingOrderRewardInfo",
            ][..],
        ),
    ] {
        let source = std::fs::read_to_string(docs.join(file)).unwrap();
        for name in names {
            let header = format!("Name = \"{name}\",");
            let mut lines = source.lines().skip_while(|line| line.trim() != header);
            assert!(lines.next().is_some(), "{name}");
            assert_eq!(lines.next().map(str::trim), Some("Type = \"Structure\","));
            let fields = lines
                .skip_while(|line| !line.trim().starts_with("{ Name = "))
                .take_while(|line| line.trim().starts_with("{ Name = "))
                .map(|line| {
                    format!(
                        "{{ {:?}, {:?}, {}, {:?} }}",
                        attr(line, "Name").unwrap(),
                        attr(line, "Type").unwrap(),
                        line.contains("Nilable = true"),
                        attr(line, "InnerType").unwrap_or("")
                    )
                })
                .collect::<Vec<_>>();
            definitions.push(format!("[{name:?}] = {{ {} }}", fields.join(",")));
        }
    }
    env.exec(&format!(r#"
        local shapes = {{ {} }}
        function CheckCraftingShape(name, info)
            assert(type(info) == 'table', name)
            for _, field in ipairs(shapes[name]) do
                local value = info[field[1]]
                local kind = field[2]
                local expected = (kind == 'bool' and 'boolean') or
                    ((kind == 'number' or kind == 'luaIndex' or kind == 'WOWMONEY' or kind == 'time_t' or type(Enum[kind]) == 'table') and 'number') or
                    ((kind == 'string' or kind == 'cstring' or kind == 'WOWGUID' or kind == 'BigUInteger') and 'string') or 'table'
                assert((value == nil and field[3]) or type(value) == expected, name .. '.' .. field[1])
                if value ~= nil and shapes[kind] then CheckCraftingShape(kind, value) end
                if value ~= nil and field[4] ~= '' then
                    for _, nested in ipairs(value) do CheckCraftingShape(field[4], nested) end
                end
            end
        end
    "#, definitions.join(","))).unwrap();
}
fn attr<'a>(line: &'a str, name: &str) -> Option<&'a str> {
    let start = line.find(&format!("{name} = \""))? + name.len() + 4;
    let end = line[start..].find('"')?;
    Some(&line[start..start + end])
}

#[test]
fn profession_crafting_slot_shapes_and_variable_quantities() {
    let env = fixture();
    install_cached_shape_checker(&env);
    env.exec(r#"
        local mods = C_TradeSkillUI.GetItemSlotModifications('Item-1-42')
        assert(#mods == 2)
        for _, info in ipairs(mods) do CheckCraftingShape('CraftingItemSlotModification', info); assert(info.itemID == nil and rawget(info,'itemID') == nil) end
        assert(mods[1].dataSlotIndex == 1 and mods[1].reagent.currencyID == 2803 and mods[1].reagent.itemID == nil)
        assert(mods[2].dataSlotIndex == 2 and mods[2].reagent.itemID == 210937 and mods[2].reagent.currencyID == nil)
        assert(#C_TradeSkillUI.GetItemSlotModifications('Item-unknown') == 0)
        local slots = C_TradeSkillUI.GetRecipeSchematic(100001).reagentSlotSchematics
        assert(#slots == 2)
        for _, slot in ipairs(slots) do CheckCraftingShape('CraftingReagentSlotSchematic', slot) end
        assert(slots[1].variableQuantities[1].quantity == 4 and slots[1].variableQuantities[2].quantity == 6)
        assert(slots[1].variableQuantities[2].reagent.currencyID == 2803)
        assert(slots[1].slotInfo.mcrSlotID == 42 and slots[1].slotInfo.requiredSkillRank == 50)
        assert(slots[1].slotInfo.slotText == 'Choose metal or currency' and slots[1].orderSource == Enum.CraftingOrderReagentSource.Customer)
        assert(slots[2].slotInfo == nil and slots[2].orderSource == nil and #slots[2].variableQuantities == 0)
        assert(slots[1].reagentType == Enum.CraftingReagentType.Basic and slots[1].required and not slots[1].hiddenInCraftingForm)
        mods[1].reagent.currencyID = 999
        slots[1].variableQuantities[2].quantity = 999
        collectgarbage('collect')
        assert(C_TradeSkillUI.GetItemSlotModifications('Item-1-42')[1].reagent.currencyID == 2803)
        assert(C_TradeSkillUI.GetRecipeSchematic(100001).reagentSlotSchematics[1].variableQuantities[2].quantity == 6)
        assert(C_TradeSkillUI.GetRecipeSchematic(100002).reagentSlotSchematics[1].reagents[1].currencyID == nil)
    "#).unwrap();
}

#[test]
fn crafting_order_nested_inputs_outputs_and_snapshots() {
    let env = fixture();
    install_cached_shape_checker(&env);
    env.exec(r#"
        local input = {skillLineAbilityID=9001, orderType=Enum.CraftingOrderType.Personal,
            orderDuration=Enum.CraftingOrderDuration.Long, tipAmount=12345, customerNotes='Nested contract',
            reagentInfos={{reagent={currencyID=2803},quantity=6}},
            craftingReagentItems={{reagent={itemID=210937},dataSlotIndex=2,quantity=2}},
            minCraftingQualityID=5, orderTarget='Crafter-Realm', recraftItem='Item-1-42'}
        CheckCraftingShape('NewCraftingOrderInfo', input)
        for _, info in ipairs(input.reagentInfos) do CheckCraftingShape('RegularReagentInfo', info) end
        assert(select('#',C_CraftingOrders.PlaceNewOrder(input)) == 0)
        input.reagentInfos[1].quantity=999
        input.craftingReagentItems[1].reagent.itemID=999
        local first=C_CraftingOrders.GetMyOrders()[1]
        assert(first.skillLineAbilityID == 9001 and first.tipAmount == 12345 and first.customerNotes == 'Nested contract')
        CheckCraftingShape('CraftingOrderInfo',first)
        assert(first.crafterName == 'ActualCrafter-Realm' and first.isRecraft and first.minQuality == 4)
        assert(first.expirationTime == 1234.5 and first.claimEndTime == 345.5 and first.consortiumCut == 77)
        assert(first.orderState == 1 and not first.isFulfillable and first.reagentState == 2)
        assert(first.customerGuid == 'Player-1-42' and first.customerName == 'Customer-Realm')
        assert(first.npcCraftingOrderSetID == 18 and first.npcTreasureID == 19)
        assert(first.npcOrderRewards[1].currencyType == 2803 and first.npcOrderRewards[1].count == 3)
        assert(#first.reagents == 2)
        for _, info in ipairs(first.reagents) do
            CheckCraftingShape('CraftingOrderReagentInfo', info)
            assert(info.reagent == nil and rawget(info,'reagent') == nil)
            assert(info.reagentInfo.itemID == nil and rawget(info.reagentInfo,'itemID') == nil)
            assert(info.source == Enum.CraftingOrderReagentSource.Customer and info.isBasicReagent)
        end
        assert(first.reagents[1].reagentInfo.reagent.itemID == 210937 and first.reagents[1].reagentInfo.quantity == 2)
        assert(first.reagents[2].reagentInfo.reagent.currencyID == 2803 and first.reagents[2].reagentInfo.quantity == 6)
        CheckCraftingShape('CraftingItemSlotModification', C_TradeSkillUI.GetItemSlotModificationsForOrder(first.orderID)[1])
        first.reagents[2].reagentInfo.quantity=999
        first.expirationTime=999
        collectgarbage('collect')
        assert(C_CraftingOrders.GetMyOrders()[1].reagents[2].reagentInfo.quantity == 6)
        assert(C_CraftingOrders.GetMyOrders()[1].expirationTime == 1234.5)
        C_CraftingOrders.PlaceNewOrder({skillLineAbilityID=9001,orderType=0,orderDuration=0,tipAmount=0,customerNotes='',
            reagentInfos={{reagent={itemID=210934},quantity=4},{reagent={itemID=210937},quantity=2}},craftingReagentItems={}})
        local second=C_CraftingOrders.GetMyOrders()[2]
        assert(second.orderID ~= first.orderID and not second.isRecraft and second.crafterName == nil)
        assert(second.reagents[1].reagentInfo.reagent.itemID == 210934)
        assert(second.expirationTime == 2345.5 and second.minQuality == 0)
    "#).unwrap();
    let state = env.state().borrow();
    let request = &state.crafting.reagents.orders[&1].request;
    assert_eq!(request.order_duration, 2);
    assert_eq!(request.min_crafting_quality_id, Some(5));
    assert_eq!(request.order_target.as_deref(), Some("Crafter-Realm"));
    assert_eq!(request.recraft_item.as_deref(), Some("Item-1-42"));
    assert_eq!(request.reagent_infos[0].quantity, 6);
    assert_eq!(
        request.crafting_reagent_items[0].reagent,
        CraftingReagent::Item(210937)
    );
}

#[test]
fn crafting_nested_allocations_consume_items_currency_and_publish_returns() {
    let env = fixture();
    install_cached_shape_checker(&env);
    env.exec(r#"
        CraftingResults={}
        local frame=CreateFrame('Frame')
        frame:RegisterEvent('TRADE_SKILL_ITEM_CRAFTED_RESULT')
        frame:SetScript('OnEvent',function(_,_,data) CraftingResults[#CraftingResults+1]=data end)
        local mods=C_TradeSkillUI.GetItemSlotModifications('Item-1-42')
        local allocations={{reagent=mods[1].reagent,dataSlotIndex=mods[1].dataSlotIndex,quantity=6},
            {reagent=mods[2].reagent,dataSlotIndex=mods[2].dataSlotIndex,quantity=2}}
        for _,info in ipairs(allocations) do CheckCraftingShape('CraftingReagentInfo',info) end
        assert(C_TradeSkillUI.CraftRecipe(100001,2,allocations))
        local result=CraftingResults[1]
        assert(result.quantity == 2 and result.itemID == 211993 and #result.resourcesReturned == 2)
        assert(result.itemGUID == C_Item.GetItemGUID({bagID=0,slotIndex=2}))
        assert(string.find(result.hyperlink,'|Hitem:211993',1,true))
        for _,resource in ipairs(result.resourcesReturned) do CheckCraftingShape('CraftingResourceReturnInfo',resource); assert(resource.itemID == nil and rawget(resource,'itemID') == nil) end
        assert(result.resourcesReturned[1].reagent.currencyID == 2803 and result.resourcesReturned[1].quantity == 4)
        assert(result.resourcesReturned[2].reagent.itemID == 210937 and result.resourcesReturned[2].quantity == 2)
        assert(C_CurrencyInfo.GetCurrencyInfo(2803).quantity == 12)
        allocations[1].quantity=999
        assert(CraftingResults[1].resourcesReturned[1].quantity == 4)
    "#).unwrap();
    let state = env.state().borrow();
    assert_eq!(
        state
            .bag_items
            .values()
            .filter(|item| item.item_id == 210937)
            .map(|item| item.stack_count)
            .sum::<i32>(),
        8
    );
    assert_eq!(
        state
            .bag_items
            .values()
            .filter(|item| item.item_id == 211993)
            .map(|item| item.stack_count)
            .sum::<i32>(),
        2
    );
}

#[test]
fn crafting_order_secret_arguments_obey_caller_taint() {
    let env = fixture();
    env.exec(r#"
        local request={skillLineAbilityID=9001,orderType=0,orderDuration=0,tipAmount=0,customerNotes='',
            reagentInfos={{reagent=secretwrap({currencyID=2803}),quantity=6},
                {reagent={itemID=210937},quantity=2}},craftingReagentItems={}}
        local wrapped = secretwrap(request)
        C_CraftingOrders.PlaceNewOrder(wrapped)
        assert(#C_CraftingOrders.GetMyOrders()==1)
        local function addon()
            assert(debug.getstacktaint()=='CraftingAddon')
            assert(not pcall(C_CraftingOrders.PlaceNewOrder,wrapped))
            assert(not pcall(C_CraftingOrders.PlaceNewOrder,request))
            assert(#C_CraftingOrders.GetMyOrders()==1)
            request.reagentInfos[1].reagent={currencyID=2803}
            C_CraftingOrders.PlaceNewOrder(request)
            assert(#C_CraftingOrders.GetMyOrders()==2)
            assert(debug.getstacktaint()=='CraftingAddon')
        end
        debug.setobjecttaint(addon,'CraftingAddon')
        addon()
    "#).unwrap();
}

#[test]
fn crafting_invalid_inputs_leave_inventory_orders_and_events_unchanged() {
    let env = fixture();
    env.exec(r#"
        CraftingEventCount=0
        local frame=CreateFrame('Frame'); frame:RegisterEvent('TRADE_SKILL_ITEM_CRAFTED_RESULT')
        frame:SetScript('OnEvent',function() CraftingEventCount=CraftingEventCount+1 end)
        for _, bad in ipairs({{}, {itemID=210934}, {itemID=210934,currencyID=2803}, {currencyID=-1}}) do
            assert(not pcall(C_TradeSkillUI.CraftRecipe,100001,1,{{reagent=bad,dataSlotIndex=1,quantity=6}}))
        end
        for _, quantity in ipairs({0,-1,1.5,2147483648}) do
            assert(not pcall(C_TradeSkillUI.CraftRecipe,100001,1,{{reagent={currencyID=2803},dataSlotIndex=1,quantity=quantity}}))
        end
        assert(not pcall(C_TradeSkillUI.CraftRecipe,100001,1,{{itemID=210934,dataSlotIndex=1,quantity=6}}))
        assert(not pcall(C_TradeSkillUI.CraftRecipe,100001,1,{{reagent={currencyID=2803},dataSlotIndex=9,quantity=6}}))
        assert(not C_TradeSkillUI.CraftRecipe(100001,4,{{reagent={currencyID=2803},dataSlotIndex=1,quantity=6},
            {reagent={itemID=210937},dataSlotIndex=2,quantity=2}}))
        for _, bad in ipairs({{}, {itemID=210934,currencyID=2803}, {currencyID=999}}) do
            assert(not pcall(C_CraftingOrders.PlaceNewOrder,{skillLineAbilityID=9001,orderType=0,orderDuration=0,tipAmount=0,customerNotes='',
                reagentInfos={{reagent=bad,quantity=6}},craftingReagentItems={}}))
        end
        assert(#C_CraftingOrders.GetMyOrders()==0 and CraftingEventCount==0)
        assert(C_CurrencyInfo.GetCurrencyInfo(2803).quantity==20)
    "#).unwrap();
    let state = env.state().borrow();
    assert_eq!(state.crafting.reagents.next_order_id, 1);
    assert_eq!(state.crafting.reagents.placement_results.len(), 2);
    assert_eq!(state.bag_items.len(), 1);
    assert_eq!(state.bag_items[&(0, 1)].stack_count, 10);
}
