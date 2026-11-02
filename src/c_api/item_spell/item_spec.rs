//! Item-specialization membership over cached derived records (WM37, WM42).

use crate::lua_api::methods::{borrow_state_mut, create_table, table_set_num};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemSpecRecord {
    /// Native permissive field, inferred as no traits and no specializations.
    pub no_traits_and_specializations: bool,
    /// Native permissive field, inferred as no stat-derived traits.
    pub no_stat_traits: bool,
    /// Native field selecting rejection of an unmatched specialization.
    pub reject_unmatched: bool,
    /// Native itemFlag27, copied from ItemSparse flag 134217728 by normal
    /// derivation; read by GetItemSpecInfo only (WM48).
    pub item_flag_27: bool,
    pub specialization_ids: Vec<u32>,
}

fn numeric_id(value: Val, name: &str) -> LuaResult<i32> {
    match value {
        Val::Num(number)
            if number.is_finite() && number >= i32::MIN as f64 && number <= i32::MAX as f64 =>
        {
            Ok(number as i32)
        }
        _ => Err(runtime_error(format!(
            "C_Item.DoesItemContainSpec: expected {name}"
        ))),
    }
}

fn record_contains_spec(record: &ItemSpecRecord, class: i32, spec: i32) -> bool {
    if record.no_traits_and_specializations || record.no_stat_traits {
        return true;
    }
    let matched = record.specialization_ids.iter().any(|id| {
        if spec != 0 {
            i64::from(*id) == i64::from(spec)
        } else {
            crate::specializations::spec_by_id(*id).map_or(0, |info| info.class_id as i32) == class
        }
    });
    matched || !record.reject_unmatched
}

pub(super) fn does_item_contain_spec(state: &mut LuaState) -> LuaResult<u32> {
    let item = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let class = rilua::table_security::unwrap_secret(state, stack_val(state, 2))?;
    let spec = rilua::table_security::unwrap_secret(state, stack_val(state, 3))?;
    let item = super::parse_item_id_key(state, item)
        .ok_or_else(|| runtime_error("C_Item.DoesItemContainSpec: expected itemInfo"))?;
    let class = numeric_id(class, "classID")?;
    let spec = if matches!(spec, Val::Nil) {
        0
    } else {
        numeric_id(spec, "specID")?
    };
    let result = class == 0 || player_item_contains_spec(state, item, class, spec)?;
    state.push(Val::Bool(result));
    Ok(1)
}

/// Cache band for the player-level selector: below 41, 41–109, 110 and above.
pub fn level_band(level: i32) -> u8 {
    match level {
        110.. => 2,
        41.. => 1,
        _ => 0,
    }
}

/// A cache hit is reused within its band. A miss builds once; missing item
/// data or selector zero builds nothing, and the query accepts.
fn player_item_contains_spec(
    state: &LuaState,
    item: u32,
    class: i32,
    spec: i32,
) -> LuaResult<bool> {
    let mut sim = borrow_state_mut(state)?;
    if !sim.player.present {
        return Ok(false);
    }
    let level = sim.player.level;
    let key = (item, level_band(level));
    if !sim.item_spec_records.contains_key(&key) {
        let built = (level != 0)
            .then(|| super::item_spec_derive::derive_record(item, level))
            .flatten();
        let Some(record) = built else {
            return Ok(true);
        };
        sim.item_spec_records.insert(key, record);
    }
    Ok(record_contains_spec(
        &sim.item_spec_records[&key],
        class,
        spec,
    ))
}

/// WM48: serialize a fresh stable filtered subsequence of the shared record.
pub(super) fn get_item_spec_info(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let item = super::parse_item_id_key(state, value)
        .ok_or_else(|| runtime_error("C_Item.GetItemSpecInfo: expected itemInfo"))?;
    let filtered = {
        let mut sim = borrow_state_mut(state)?;
        let level = sim.player.level;
        let key = (item, level_band(level));
        if !sim.item_spec_records.contains_key(&key) {
            let Some(record) = (level != 0)
                .then(|| super::item_spec_derive::derive_record(item, level))
                .flatten()
            else {
                return Ok(0);
            };
            sim.item_spec_records.insert(key, record);
        }
        let record = &sim.item_spec_records[&key];
        if record.no_traits_and_specializations || record.item_flag_27 {
            return Ok(0);
        }
        let class = if sim
            .game_rules
            .rules
            .get("CharacterlessLogin")
            .and_then(|rule| rule.integer_value)
            .is_some_and(|v| v != 0)
        {
            14
        } else {
            sim.player.class_index
        };
        let bonding = super::item_spec_derive::item_bonding(item);
        record
            .specialization_ids
            .iter()
            .copied()
            .filter(|&id| {
                let spec = super::item_spec_derive::specialization_row(id);
                if spec.is_some_and(|(_, order)| order == 4) {
                    return false;
                }
                if bonding.is_some_and(|value| value != 1) {
                    return true;
                }
                spec.is_some_and(|(spec_class, _)| i32::from(spec_class) == class)
            })
            .collect::<Vec<_>>()
    };
    let Val::Table(table) = create_table(state) else {
        unreachable!("create_table returns a table")
    };
    for (index, id) in filtered.into_iter().enumerate() {
        table_set_num(state, table, (index + 1) as f64, Val::Num(f64::from(id)));
    }
    state.push(Val::Table(table));
    Ok(1)
}
