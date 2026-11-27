//! Bag cleanup preferences behind the `C_Container` getter/setter pairs
//! (ContainerDocumentation.lua:97/:328/:364/:483/:493/:513, Retail and Forever).
//!
//! The pinned Blizzard consumers reach these only through the API; no CVar
//! backs them in the cached Lua. INFERRED: R88g9 (native evidence pending):
//! each setter stores the Lua truthiness of its argument for the session, and
//! every preference starts unset (false).

use crate::c_api::ensure_namespace;
use crate::lua_api::SimState;
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[derive(Debug, Default, Clone, Copy)]
pub struct ContainerPreferences {
    pub sort_bags_right_to_left: bool,
    pub insert_items_left_to_right: bool,
    pub bank_autosort_disabled: bool,
}

type Field = fn(&mut SimState) -> &mut bool;

fn sort_right_to_left(sim: &mut SimState) -> &mut bool {
    &mut sim.container_preferences.sort_bags_right_to_left
}

fn insert_left_to_right(sim: &mut SimState) -> &mut bool {
    &mut sim.container_preferences.insert_items_left_to_right
}

fn bank_autosort_disabled(sim: &mut SimState) -> &mut bool {
    &mut sim.container_preferences.bank_autosort_disabled
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let ns = ensure_namespace(state, "C_Container")?;
    for (name, function) in [
        ("GetSortBagsRightToLeft", get_sort as rilua::RustFn),
        ("SetSortBagsRightToLeft", set_sort),
        ("GetInsertItemsLeftToRight", get_insert),
        ("SetInsertItemsLeftToRight", set_insert),
        ("GetBankAutosortDisabled", get_bank),
        ("SetBankAutosortDisabled", set_bank),
    ] {
        table_set_rust_fn_static(state, ns, name, function)?;
    }
    Ok(())
}

/// Lua truthiness of the first argument (`bool` arguments in the docs).
pub(crate) fn truthy_argument(state: &LuaState) -> LuaResult<bool> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    Ok(!matches!(value, Val::Nil | Val::Bool(false)))
}

fn read(state: &mut LuaState, field: Field) -> LuaResult<u32> {
    let value = *field(&mut *borrow_state_mut(state)?);
    state.push(Val::Bool(value));
    Ok(1)
}

fn write(state: &mut LuaState, field: Field) -> LuaResult<u32> {
    let value = truthy_argument(state)?;
    *field(&mut *borrow_state_mut(state)?) = value;
    Ok(0)
}

fn get_sort(state: &mut LuaState) -> LuaResult<u32> {
    read(state, sort_right_to_left)
}

fn set_sort(state: &mut LuaState) -> LuaResult<u32> {
    write(state, sort_right_to_left)
}

fn get_insert(state: &mut LuaState) -> LuaResult<u32> {
    read(state, insert_left_to_right)
}

fn set_insert(state: &mut LuaState) -> LuaResult<u32> {
    write(state, insert_left_to_right)
}

fn get_bank(state: &mut LuaState) -> LuaResult<u32> {
    read(state, bank_autosort_disabled)
}

fn set_bank(state: &mut LuaState) -> LuaResult<u32> {
    write(state, bank_autosort_disabled)
}
