#![cfg(feature = "client-wowforever")]

use wow_ui_sim::c_api::trait_group_currency::{ConfigCurrencyInputs, CurrencyAmounts, RankedNode};
use wow_ui_sim::lua_api::WowLuaEnv;

fn configured_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        let model = &mut sim.talents.forever.group_currency;
        model.group_trees.extend([(10, 500), (20, 500), (30, 600), (40, 500)]);
        model.node_groups.insert(100, vec![10, 20, 999]);
        model.node_groups.insert(101, vec![20]);
        model.node_groups.insert(102, vec![30]);
        model.tree_currencies.insert(500, vec![80, 70]);
        let mut config = ConfigCurrencyInputs {
            nodes: vec![RankedNode { node_id: 100, rank: 1 }, RankedNode { node_id: 101, rank: 2 }, RankedNode { node_id: 102, rank: 0 }],
            accumulator_order: vec![20, 10],
            ..Default::default()
        };
        for (group, currency, quantity, max_quantity, spent) in [
            (20, 80, -4, Some(10), 14), (20, 70, 2, None, 9),
            (10, 80, 8, Some(10), 2), (10, 70, 3, None, 1),
        ] {
            config.amounts.insert((group, currency), CurrencyAmounts { quantity, max_quantity, spent, spent_in_tree: None });
        }
        let mut other = config.clone();
        other.accumulator_order = vec![10, 20];
        other.amounts.get_mut(&(10, 80)).unwrap().quantity = 27;
        model.configs.insert(7, config);
        model.configs.insert(8, other);
    }
    env
}

#[test]
fn group_currency_projects_selected_config_nodes_and_provider_order() {
    let env = configured_env();
    env.exec(r#"
        local rows = C_Traits.GetGroupCurrencyInfo(7, {10, 20})
        assert(#rows == 2 and rows[1].traitNodeGroupID == 20 and rows[2].traitNodeGroupID == 10)
        local first, second = rows[1].currencyInfos[1], rows[1].currencyInfos[2]
        assert(#rows[1].currencyInfos == 2)
        assert(first.traitCurrencyID == 80 and first.quantity == -4 and first.maxQuantity == 10 and first.spent == 14)
        assert(first.spentInTree == nil)
        assert(second.traitCurrencyID == 70 and second.quantity == 2 and second.maxQuantity == nil and second.spent == 9)
        rows[1].currencyInfos[1].quantity = 999
        assert(C_Traits.GetGroupCurrencyInfo(7, {20})[1].currencyInfos[1].quantity == -4)
        local other = C_Traits.GetGroupCurrencyInfo(8, {20, 10})
        assert(other[1].traitNodeGroupID == 10 and other[1].currencyInfos[1].quantity == 27)
        local repeated = C_Traits.GetGroupCurrencyInfo('7.9', {'0xa', '10.8', 999, extra = false, [0] = false})
        assert(#repeated == 1 and repeated[1].traitNodeGroupID == 10)
        assert(repeated[1].currencyInfos[1].spent == 2, 'duplicate requested IDs must not multiply membership')
        assert(select('#', C_Traits.GetGroupCurrencyInfo(7, {10})) == 1)
    "#).unwrap();
    {
        let state = env.state();
        let mut sim = state.borrow_mut();
        for node in &mut sim.talents.forever.group_currency.configs.get_mut(&7).unwrap().nodes {
            node.rank = 0;
        }
    }
    env.exec("assert(#C_Traits.GetGroupCurrencyInfo(7, {10,20}) == 0)").unwrap();
}

#[test]
fn group_currency_empty_cases_do_not_fabricate_group_rows() {
    let env = configured_env();
    env.exec(r#"
        assert(#C_Traits.GetGroupCurrencyInfo(99, {10}) == 0)
        assert(#C_Traits.GetGroupCurrencyInfo(7, {}) == 0)
        assert(#C_Traits.GetGroupCurrencyInfo(7, {999}) == 0)
        assert(#C_Traits.GetGroupCurrencyInfo(7, {10,30}) == 0, 'mixed known trees are empty even if one node has rank zero')
        assert(#C_Traits.GetGroupCurrencyInfo(7, {40}) == 0, 'no positive ranked membership')
        assert(#C_Traits.GetGroupCurrencyInfo(7, {30}) == 0, 'rank zero cannot create an accumulator')
        assert(#C_Traits.GetGroupCurrencyInfo(7, {10,999}) == 1, 'unknown IDs do not invalidate known groups')
    "#).unwrap();
    env.state().borrow_mut().talents.forever.group_currency.tree_currencies.insert(500, vec![]);
    env.exec("local rows=C_Traits.GetGroupCurrencyInfo(7,{10}); assert(#rows==1 and type(rows[1].currencyInfos)=='table' and #rows[1].currencyInfos==0)").unwrap();
}

#[test]
fn group_currency_requires_full_argument_array_before_missing_state_lookup() {
    let env = WowLuaEnv::new().unwrap();
    for call in [
        "C_Traits.GetGroupCurrencyInfo()", "C_Traits.GetGroupCurrencyInfo(nil, {})",
        "C_Traits.GetGroupCurrencyInfo(99)", "C_Traits.GetGroupCurrencyInfo(99, nil)",
        "C_Traits.GetGroupCurrencyInfo(99, '10')", "C_Traits.GetGroupCurrencyInfo(99, {10,false})",
        "C_Traits.GetGroupCurrencyInfo('no', {})", "C_Traits.GetGroupCurrencyInfo(2147483648, {})",
        "C_Traits.GetGroupCurrencyInfo(99, {-2147483649})",
    ] {
        let error = env.exec(call).expect_err("required native selectors").to_string();
        assert!(error.contains("Usage: local groupCurrencyInfos = C_Traits.GetGroupCurrencyInfo(configID, groupIDs)"), "{error}");
    }
    env.exec("assert(#C_Traits.GetGroupCurrencyInfo('-2147483648', {'2147483647'}) == 0)").unwrap();
}

#[test]
fn group_currency_missing_populated_sources_stay_named_unknowns() {
    let env = configured_env();
    env.state().borrow_mut().talents.forever.group_currency.configs.get_mut(&7).unwrap().amounts.remove(&(20,80));
    let error = env.exec("C_Traits.GetGroupCurrencyInfo(7,{20})").expect_err("unknown resolved native cost input").to_string();
    assert!(error.contains("UNKNOWN GetGroupCurrencyInfo resolved cost/holdings input"), "{error}");
    let env = configured_env();
    env.state().borrow_mut().talents.forever.group_currency.configs.get_mut(&7).unwrap().accumulator_order.clear();
    let error = env.exec("C_Traits.GetGroupCurrencyInfo(7,{20})").expect_err("unknown native bucket order").to_string();
    assert!(error.contains("UNKNOWN GetGroupCurrencyInfo accumulator traversal order"), "{error}");
}
