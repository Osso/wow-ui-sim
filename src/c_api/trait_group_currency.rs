//! WM102 group selection with explicitly supplied, already-resolved currency inputs.
use std::collections::{HashMap, HashSet};

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{
    borrow_state, call_function_state, create_table, table_set, table_set_num,
};
use crate::lua_bridge::stack_val;
use rilua::vm::closure::{Closure, RustClosure};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const USAGE: &str =
    "Usage: local groupCurrencyInfos = C_Traits.GetGroupCurrencyInfo(configID, groupIDs)";

#[derive(Clone, Debug)]
pub struct RankedNode {
    pub node_id: i32,
    pub rank: u32,
}

/// UNKNOWN native cost/holdings acquisition seam. Values are explicit resolved
/// signed-32 amounts, not inferred per-rank costs or fabricated player holdings.
#[derive(Clone, Debug)]
pub struct CurrencyAmounts {
    pub quantity: i32,
    pub max_quantity: Option<i32>,
    pub spent: i32,
    pub spent_in_tree: Option<i32>,
}

#[derive(Clone, Debug, Default)]
pub struct ConfigCurrencyInputs {
    pub nodes: Vec<RankedNode>,
    /// Supplied accumulator bucket/chain order; never request order or sorting.
    pub accumulator_order: Vec<i32>,
    pub amounts: HashMap<(i32, i32), CurrencyAmounts>,
}

#[derive(Clone, Debug, Default)]
pub struct GroupCurrencyState {
    pub configs: HashMap<i32, ConfigCurrencyInputs>,
    pub group_trees: HashMap<i32, i32>,
    pub node_groups: HashMap<i32, Vec<i32>>,
    /// Supplied tree-currency provider order, independent of group request order.
    pub tree_currencies: HashMap<i32, Vec<i32>>,
}

struct GroupSnapshot {
    group_id: i32,
    currencies: Vec<(i32, CurrencyAmounts)>,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if ACTIVE != ClientProfile::WowForever {
        return Ok(());
    }
    // Forever 70707 SharedTraitsDocumentation.lua:281-295,972-980,1188-1200.
    let namespace = super::ensure_namespace(state, "C_Traits")?;
    crate::lua_bridge::table_set_rust_fn_static(
        state,
        namespace,
        "GetGroupCurrencyInfo",
        get_group_currency_info,
    )
}

fn read_signed_selector(state: &mut LuaState, value: Val) -> LuaResult<i32> {
    let value = rilua::table_security::unwrap_secret(state, value)?;
    let number = match value {
        Val::Num(number) => number,
        Val::Str(_) => {
            // Use the VM's numeric-string grammar without consulting an
            // addon-replaceable global tonumber function.
            let converter = Val::Function(state.gc.alloc_closure(Closure::Rust(RustClosure::new(
                rilua::stdlib::base::lua_tonumber,
                "trait_group_currency_number",
            ))));
            let Val::Num(number) = call_function_state(state, converter, &[value])? else {
                return Err(rilua::runtime_error(USAGE));
            };
            number
        }
        _ => return Err(rilua::runtime_error(USAGE)),
    };
    if !number.is_finite() || !(f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&number) {
        return Err(rilua::runtime_error(format!(
            "{USAGE} - outside of expected range -2147483648 to 2147483647"
        )));
    }
    Ok(number.trunc() as i32)
}

fn read_group_ids(state: &mut LuaState) -> LuaResult<Vec<i32>> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 2))?;
    let Val::Table(table_ref) = value else {
        return Err(rilua::runtime_error(USAGE));
    };
    let length = state
        .gc
        .tables
        .get(table_ref)
        .ok_or_else(|| rilua::runtime_error("GetGroupCurrencyInfo: invalid group array"))?
        .len(&state.gc.string_arena);
    (1..=length)
        .map(|index| {
            let value = state
                .gc
                .tables
                .get(table_ref)
                .ok_or_else(|| rilua::runtime_error("GetGroupCurrencyInfo: invalid group array"))?
                .get_int(index as i64);
            read_signed_selector(state, value)
        })
        .collect()
}

/// INFERRED WM102 public seam: native empty output publishes one empty array.
fn infer_empty_group_currency_result() -> Vec<GroupSnapshot> {
    Vec::new()
}

fn select_tree(model: &GroupCurrencyState, requested: &HashSet<i32>) -> Option<i32> {
    let mut tree = None;
    for group in requested {
        let Some(&candidate) = model.group_trees.get(group) else {
            continue;
        };
        if tree.is_some_and(|tree| tree != candidate) {
            return None;
        }
        tree = Some(candidate);
    }
    tree
}

fn collect_snapshots(
    model: &GroupCurrencyState,
    config_id: i32,
    group_ids: &[i32],
) -> LuaResult<Vec<GroupSnapshot>> {
    let Some(config) = model.configs.get(&config_id) else {
        return Ok(infer_empty_group_currency_result());
    };
    let requested: HashSet<_> = group_ids.iter().copied().collect();
    let Some(tree_id) = select_tree(model, &requested) else {
        return Ok(infer_empty_group_currency_result());
    };
    let mut eligible: HashSet<_> = config
        .nodes
        .iter()
        .filter(|node| node.rank > 0)
        .filter_map(|node| model.node_groups.get(&node.node_id))
        .flatten()
        .copied()
        .filter(|group| requested.contains(group) && model.group_trees.get(group) == Some(&tree_id))
        .collect();
    if eligible.is_empty() {
        return Ok(infer_empty_group_currency_result());
    }
    let currency_order = model.tree_currencies.get(&tree_id).ok_or_else(|| {
        rilua::runtime_error(
            "UNKNOWN GetGroupCurrencyInfo tree-currency provider order is not supplied",
        )
    })?;
    let mut result = Vec::new();
    for &group_id in &config.accumulator_order {
        if !eligible.remove(&group_id) {
            continue;
        }
        let currencies = currency_order.iter().map(|&currency_id| {
            let amounts = config.amounts.get(&(group_id, currency_id)).cloned()
                .ok_or_else(|| rilua::runtime_error("UNKNOWN GetGroupCurrencyInfo resolved cost/holdings input is not supplied"))?;
            Ok((currency_id, amounts))
        }).collect::<LuaResult<_>>()?;
        result.push(GroupSnapshot {
            group_id,
            currencies,
        });
    }
    if !eligible.is_empty() {
        return Err(rilua::runtime_error(
            "UNKNOWN GetGroupCurrencyInfo accumulator traversal order is not supplied",
        ));
    }
    Ok(result)
}

fn publish_currency(state: &mut LuaState, currency_id: i32, amounts: &CurrencyAmounts) -> Val {
    let row = create_table(state);
    for (key, value) in [
        ("traitCurrencyID", currency_id),
        ("quantity", amounts.quantity),
        ("spent", amounts.spent),
    ] {
        table_set(state, row, key, Val::Num(f64::from(value)));
    }
    table_set(
        state,
        row,
        "maxQuantity",
        amounts
            .max_quantity
            .map_or(Val::Nil, |n| Val::Num(f64::from(n))),
    );
    table_set(
        state,
        row,
        "spentInTree",
        amounts
            .spent_in_tree
            .map_or(Val::Nil, |n| Val::Num(f64::from(n))),
    );
    row
}

fn publish_groups(state: &mut LuaState, groups: &[GroupSnapshot]) -> Val {
    let result = create_table(state);
    let Val::Table(result_ref) = result else {
        unreachable!("create_table returns a table")
    };
    for (index, group) in groups.iter().enumerate() {
        let row = create_table(state);
        table_set(
            state,
            row,
            "traitNodeGroupID",
            Val::Num(f64::from(group.group_id)),
        );
        let currencies = create_table(state);
        let Val::Table(currency_ref) = currencies else {
            unreachable!("create_table returns a table")
        };
        for (index, (currency_id, amounts)) in group.currencies.iter().enumerate() {
            let currency = publish_currency(state, *currency_id, amounts);
            table_set_num(state, currency_ref, (index + 1) as f64, currency);
        }
        table_set(state, row, "currencyInfos", currencies);
        table_set_num(state, result_ref, (index + 1) as f64, row);
    }
    result
}

fn get_group_currency_info(state: &mut LuaState) -> LuaResult<u32> {
    let config = read_signed_selector(state, stack_val(state, 1))?;
    let groups = read_group_ids(state)?;
    let snapshots = collect_snapshots(
        &borrow_state(state)?.talents.forever.group_currency,
        config,
        &groups,
    )?;
    let result = publish_groups(state, &snapshots);
    state.push(result);
    Ok(1)
}
