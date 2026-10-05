//! Spell-bound action queries. Range snapshots are host inputs keyed by unit GUID.
use crate::lua_api::methods::{borrow_state, val_to_string};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct ActionSpellRange {
    pub has_range_requirements: bool,
    pub in_range_by_guid: HashMap<String, bool>,
}

pub(crate) fn register(state: &mut LuaState, ns: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, ns, "GetActionAutocast", autocast)?;
    table_set_rust_fn_static(state, ns, "HasRangeRequirements", range_required)?;
    table_set_rust_fn_static(state, ns, "IsActionInRange", in_range)
}
fn read_slot(state: &LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    match value {
        Val::Num(n) if n > 0.0 && n <= u32::MAX as f64 && n.fract() == 0.0 => Ok(n as u32),
        _ => Err(rilua::runtime_error("actionID must be a positive integer")),
    }
}
fn autocast(state: &mut LuaState) -> LuaResult<u32> {
    let slot = read_slot(state)?;
    let (allowed, enabled) = {
        let sim = borrow_state(state)?;
        // INFERRED: a directly bound pet spell shares its existing pet-bar autocast state.
        // Macros/items/outfits have no resolved pet action in the current model.
        let pet = sim.action_bars.get(&slot).and_then(|id| {
            sim.pet_actions
                .iter()
                .find(|pet| pet.has_action && pet.spell_id == Some(*id))
        });
        pet.map_or((false, false), |pet| {
            (
                pet.auto_cast_allowed,
                pet.auto_cast_allowed && pet.auto_cast_enabled,
            )
        })
    };
    state.push(Val::Bool(allowed));
    state.push(Val::Bool(enabled));
    Ok(2)
}
fn range_required(state: &mut LuaState) -> LuaResult<u32> {
    let slot = read_slot(state)?;
    let value = {
        let sim = borrow_state(state)?;
        sim.action_bars
            .get(&slot)
            .and_then(|id| sim.action_spell_ranges.get(id))
            .is_some_and(|range| range.has_range_requirements)
    };
    state.push(Val::Bool(value));
    Ok(1)
}
fn read_target(state: &LuaState) -> LuaResult<String> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 2))?;
    match value {
        Val::Nil => Ok("target".into()),
        Val::Str(_) => {
            val_to_string(state, value).ok_or_else(|| rilua::runtime_error("target must be UTF-8"))
        }
        _ => Err(rilua::runtime_error("target must be a unit token or nil")),
    }
}
fn in_range(state: &mut LuaState) -> LuaResult<u32> {
    let slot = read_slot(state)?;
    let token = read_target(state)?;
    let result = {
        let sim = borrow_state(state)?;
        let unit = crate::lua_api::globals::targeting_verbs::resolve_unit_snapshot(&sim, &token);
        unit.and_then(|unit| {
            sim.action_bars
                .get(&slot)
                .and_then(|id| sim.action_spell_ranges.get(id))
                .and_then(|range| range.in_range_by_guid.get(&unit.guid).copied())
        })
    };
    // INFERRED: absent host range data is indeterminate even for unranged actions.
    state.push(result.map_or(Val::Nil, Val::Bool));
    Ok(1)
}
