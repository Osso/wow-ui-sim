//! Forever structured views of the shared spell-flyout catalog.

use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_get, table_set,
    val_to_string,
};
use crate::lua_api::state::{SpellFlyoutInfo, SpellFlyoutSlot};
use crate::lua_bridge::{FromStack, TableBuilder, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};
use std::collections::HashMap;

// Forever 70707 FlyoutDocumentation.lua:10, :27, :43, :59, :92.
pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let table = super::ensure_namespace(state, "C_Flyout")?;
    table_set_rust_fn_static(state, table, "FlyoutHasSpell", has_spell)?;
    table_set_rust_fn_static(state, table, "GetFlyoutID", flyout_id)?;
    table_set_rust_fn_static(state, table, "GetFlyoutInfo", flyout_info)?;
    table_set_rust_fn_static(state, table, "GetFlyoutSlotInfo", flyout_slot_info)?;
    table_set_rust_fn_static(state, table, "GetNumFlyouts", num_flyouts)
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetSpellFlyouts", replace_catalog)
}

fn num_flyouts(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.spell_flyout_order.len();
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn flyout_id(state: &mut LuaState) -> LuaResult<u32> {
    let index = u32::from_stack(state, 1)?;
    let id = {
        let sim = borrow_state(state)?;
        index
            .checked_sub(1)
            .and_then(|i| sim.spell_flyout_order.get(i as usize))
            .copied()
    };
    let Some(id) = id else { return Ok(0) };
    state.push(Val::Num(f64::from(id)));
    Ok(1)
}

fn flyout_info(state: &mut LuaState) -> LuaResult<u32> {
    let id = u32::from_stack(state, 1)?;
    let info = borrow_state(state)?.spell_flyouts.get(&id).cloned();
    let Some(info) = info else { return Ok(0) };
    let row = create_table(state);
    state.push(row);
    let name = create_string(state, &info.name);
    table_set(state, row, "name", name);
    let description = create_string(state, &info.description);
    table_set(state, row, "description", description);
    table_set(state, row, "numSlots", Val::Num(info.slots.len() as f64));
    table_set(state, row, "isKnown", Val::Bool(info.is_known));
    Ok(1)
}

fn flyout_slot_info(state: &mut LuaState) -> LuaResult<u32> {
    let id = u32::from_stack(state, 1)?;
    let index = u32::from_stack(state, 2)?;
    let slot = {
        let sim = borrow_state(state)?;
        sim.spell_flyouts
            .get(&id)
            .and_then(|info| {
                index
                    .checked_sub(1)
                    .and_then(|i| info.slots.get(i as usize))
            })
            .cloned()
    };
    let Some(slot) = slot else { return Ok(0) };
    push_slot(state, slot);
    Ok(1)
}

fn push_slot(state: &mut LuaState, slot: SpellFlyoutSlot) {
    let row = create_table(state);
    state.push(row);
    table_set(state, row, "spellID", Val::Num(f64::from(slot.spell_id)));
    table_set(
        state,
        row,
        "overrideSpellID",
        Val::Num(f64::from(slot.override_spell_id)),
    );
    table_set(state, row, "isKnown", Val::Bool(slot.is_known));
    if let Some(name) = slot.spell_name {
        let name = create_string(state, &name);
        table_set(state, row, "name", name);
    }
    if let Some(spec) = slot.spec_id {
        table_set(state, row, "specID", Val::Num(f64::from(spec)));
    }
}

fn has_spell(state: &mut LuaState) -> LuaResult<u32> {
    let id = u32::from_stack(state, 1)?;
    let spell = u32::from_stack(state, 2)?;
    let found = borrow_state(state)?
        .spell_flyouts
        .get(&id)
        .map(|info| info.slots.iter().any(|slot| slot.spell_id == spell));
    let Some(found) = found else { return Ok(0) };
    state.push(Val::Bool(found));
    Ok(1)
}

fn replace_catalog(state: &mut LuaState) -> LuaResult<u32> {
    let rows = read_array(state, stack_val(state, 1))?;
    let mut order = Vec::with_capacity(rows.len());
    let mut catalog = HashMap::with_capacity(rows.len());
    for row in rows {
        require_table(row)?;
        let id = required_id(state, row, "flyoutID")?;
        let info = decode_flyout(state, row)?;
        if catalog.insert(id, info).is_some() {
            return Err(runtime_error("SetSpellFlyouts: duplicate flyoutID"));
        }
        order.push(id);
    }
    let mut sim = borrow_state_mut(state)?;
    sim.spell_flyout_order = order;
    sim.spell_flyouts = catalog;
    Ok(0)
}

fn decode_flyout(state: &mut LuaState, row: Val) -> LuaResult<SpellFlyoutInfo> {
    let name = required_text(state, row, "name")?;
    let description = required_text(state, row, "description")?;
    let is_known = required_bool(state, row, "isKnown")?;
    let slot_rows = table_get(state, row, "slots");
    let slots = read_array(state, slot_rows)?
        .into_iter()
        .map(|slot| decode_slot(state, slot))
        .collect::<LuaResult<_>>()?;
    Ok(SpellFlyoutInfo {
        name,
        description,
        is_known,
        slots,
    })
}

fn decode_slot(state: &mut LuaState, row: Val) -> LuaResult<SpellFlyoutSlot> {
    require_table(row)?;
    let spell_name = optional_name(state, row)?;
    let spec_id = optional_spec(state, row)?;
    Ok(SpellFlyoutSlot {
        spell_id: required_id(state, row, "spellID")?,
        override_spell_id: required_id(state, row, "overrideSpellID")?,
        is_known: required_bool(state, row, "isKnown")?,
        spell_name,
        spec_id,
    })
}

fn optional_name(state: &mut LuaState, row: Val) -> LuaResult<Option<String>> {
    let name = table_get(state, row, "name");
    match name {
        Val::Nil => Ok(None),
        Val::Str(_) => val_to_string(state, name)
            .map(Some)
            .ok_or_else(|| runtime_error("SetSpellFlyouts: invalid name")),
        _ => Err(runtime_error(
            "SetSpellFlyouts: name must be a string or nil",
        )),
    }
}

fn optional_spec(state: &mut LuaState, row: Val) -> LuaResult<Option<i32>> {
    let spec = table_get(state, row, "specID");
    if matches!(spec, Val::Nil) {
        return Ok(None);
    }
    i32::try_from(read_id(spec, "specID")?)
        .map(Some)
        .map_err(|_| runtime_error("SetSpellFlyouts: specID out of range"))
}

fn require_table(value: Val) -> LuaResult<()> {
    if matches!(value, Val::Table(_)) {
        Ok(())
    } else {
        Err(runtime_error("SetSpellFlyouts: expected row table"))
    }
}

fn read_array(state: &LuaState, value: Val) -> LuaResult<Vec<Val>> {
    let Val::Table(reference) = value else {
        return Err(runtime_error("SetSpellFlyouts: expected array"));
    };
    let table = state
        .gc
        .tables
        .get(reference)
        .ok_or_else(|| runtime_error("SetSpellFlyouts: array unavailable"))?;
    let length = table.len(&state.gc.string_arena);
    let values: Vec<_> = (1..=length)
        .map(|index| table.get_int(index as i64))
        .collect();
    let extra_array = table
        .array_slice()
        .iter()
        .skip(length)
        .any(|value| !matches!(value, Val::Nil));
    let extra_hash = table.hash_entries().iter().any(|(key, value)| {
        !matches!(value, Val::Nil)
            && !matches!(key, Val::Num(i) if *i >= 1.0 && *i <= length as f64 && i.fract() == 0.0)
    });
    if extra_array || extra_hash || values.iter().any(|value| matches!(value, Val::Nil)) {
        return Err(runtime_error(
            "SetSpellFlyouts: expected dense one-based array",
        ));
    }
    Ok(values)
}

fn required_id(state: &mut LuaState, row: Val, field: &str) -> LuaResult<u32> {
    read_id(table_get(state, row, field), field)
}

fn read_id(value: Val, field: &str) -> LuaResult<u32> {
    match value {
        Val::Num(id) if (0.0..=f64::from(u32::MAX)).contains(&id) && id.fract() == 0.0 => {
            Ok(id as u32)
        }
        _ => Err(runtime_error(format!(
            "SetSpellFlyouts: {field} must be an unsigned integer"
        ))),
    }
}

fn required_text(state: &mut LuaState, row: Val, field: &str) -> LuaResult<String> {
    let value = table_get(state, row, field);
    if matches!(value, Val::Str(_)) {
        if let Some(text) = val_to_string(state, value) {
            return Ok(text);
        }
    }
    Err(runtime_error(format!(
        "SetSpellFlyouts: {field} must be a string"
    )))
}

fn required_bool(state: &mut LuaState, row: Val, field: &str) -> LuaResult<bool> {
    match table_get(state, row, field) {
        Val::Bool(flag) => Ok(flag),
        _ => Err(runtime_error(format!(
            "SetSpellFlyouts: {field} must be a boolean"
        ))),
    }
}
