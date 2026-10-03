//! Catalog-backed active outfit selection shared by secure actions and /outfit.

use crate::lua_api::globals::security::resolve_cmd_option;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val};

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    for (name, handler) in [
        (
            "GetActiveOutfitID",
            get_active_outfit_id as fn(&mut LuaState) -> LuaResult<u32>,
        ),
        ("ChangeToOutfit", change_to_outfit),
        ("ClearOutfit", clear_outfit),
    ] {
        table_set_rust_fn_static(state, namespace, name, handler)?;
    }
    Ok(())
}

fn get_active_outfit_id(state: &mut LuaState) -> LuaResult<u32> {
    // INFERRED: zero represents no active outfit in the nonnil numeric API.
    let id = borrow_state(state)?.active_transmog_outfit_id.unwrap_or(0);
    state.push(Val::Num(id as f64));
    Ok(1)
}

fn change_to_outfit(state: &mut LuaState) -> LuaResult<u32> {
    let index = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let toggle = rilua::table_security::unwrap_secret(state, stack_val(state, 2))?;
    let Val::Num(index) = index else {
        return Err(rilua::runtime_error(
            "ChangeToOutfit requires a numeric player-facing index",
        ));
    };
    let Val::Bool(allow_remove) = toggle else {
        return Err(rilua::runtime_error(
            "ChangeToOutfit requires a boolean allowRemoveOutfit",
        ));
    };
    apply_outfit_index(state, index, allow_remove)?;
    Ok(0)
}

fn apply_outfit_index(state: &mut LuaState, index: f64, allow_remove: bool) -> LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    let id = sim
        .transmog_outfit_catalog
        .entries
        .iter()
        .find(|entry| entry.player_facing_outfit_index as f64 == index)
        .map(|entry| entry.outfit_id);
    // INFERRED: missing indices preserve selection, without fabricating an outfit.
    // Duplicate indices choose first catalog entry, like existing catalog queries.
    let Some(id) = id else { return Ok(()) };
    let remove = allow_remove && sim.active_transmog_outfit_id == Some(id);
    sim.active_transmog_outfit_id = if remove { None } else { Some(id) };
    Ok(())
}

fn clear_outfit(state: &mut LuaState) -> LuaResult<u32> {
    clear_active_outfit(state)?;
    Ok(0)
}

fn clear_active_outfit(state: &mut LuaState) -> LuaResult<()> {
    borrow_state_mut(state)?.active_transmog_outfit_id = None;
    Ok(())
}

pub(crate) fn run_outfit_command(state: &mut LuaState, argument: &str) -> LuaResult<()> {
    // Vendor SlashCommands clears an empty command. The existing condition
    // selector has no empty clause, so handle that documented command directly.
    if argument.trim().is_empty() {
        return clear_active_outfit(state);
    }
    let selected = {
        let sim = borrow_state(state)?;
        resolve_cmd_option(argument, &sim).map(str::to_owned)
    };
    let Some(selected) = selected else {
        return Ok(());
    };
    let (index, allow_remove) = match selected.strip_prefix('!') {
        Some(index) => (index, false),
        None => (selected.as_str(), true),
    };
    // INFERRED: Rust decimal-number parsing, not the entire Lua tonumber grammar.
    let Ok(index) = index.trim().parse::<f64>() else {
        return Ok(());
    };
    apply_outfit_index(state, index, allow_remove)
}
