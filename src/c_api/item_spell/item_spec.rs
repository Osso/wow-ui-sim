//! Supplied derived item-specialization records.

use crate::lua_api::methods::borrow_state;
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
    let result = if class == 0 {
        true
    } else {
        let sim = borrow_state(state)?;
        sim.player.present
            && sim
                .item_spec_records
                .get(&item)
                .is_none_or(|record| record_contains_spec(record, class, spec))
    };
    state.push(Val::Bool(result));
    Ok(1)
}
