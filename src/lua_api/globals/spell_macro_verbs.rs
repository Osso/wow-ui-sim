//! Spell / talent / macro pickup verbs. Routes through
//! `SimState.cursor_item` and the macro table.
//!
//! Migrates 8 entries off `GLOBAL_NIL_STUBS`:
//!
//! - `PickupSpell(spellId)`        — cursor = Spell(spellId)
//! - `PickupTalent(talentId)`      — cursor = Talent(talentId, pvp=false)
//! - `PickupPvpTalent(talentId)`   — cursor = Talent(talentId, pvp=true)
//! - `PickupPetAction(slot)`       — cursor = PetAction(slot, synthesized
//!                                     spell_id = `1_000_000 + slot`)
//! - `PickupMacro(index)`          — cursor = Macro(macro_index)
//! - `RunMacro(index_or_name)`     — set `running_macro` to the resolved
//!                                     slot index. Silent no-op for unknown.
//! - `StopMacro()`                 — clears `running_macro`.
//! - `EditMacro(index_or_name, name?, icon?, body?)` — update macro
//!                                     name / icon / body in-place. Auto-
//!                                     grows the macro table when passing
//!                                     an index beyond current length.
//!
//! Registered from `register_tail_globals` after `missing_surface`.

use crate::lua_api::methods::{borrow_state_mut, call_function_state, create_string, create_table};
use crate::lua_api::state::MacroInfo;
use crate::lua_api::state_types::CursorInfo;
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaApiMut, LuaResult, Val};

const PET_ACTION_SPELL_OFFSET: u32 = 1_000_000;

fn ensure_namespace_table(state: &mut LuaState, namespace: &'static str) -> GcRef<Table> {
    let key = state.gc.intern_string_static(namespace.as_bytes());
    let global = state.global;
    let existing = state
        .gc
        .tables
        .get(global)
        .map(|table| table.get_str(key, &state.gc.string_arena));
    if let Some(Val::Table(table_ref)) = existing {
        return table_ref;
    }

    let table = create_table(state);
    let Val::Table(table_ref) = table else {
        unreachable!("create_table must return a table");
    };
    if let Some(global_table) = state.gc.tables.get_mut(global) {
        let _ = global_table.raw_set(Val::Str(key), table, &state.gc.string_arena);
    }
    state.gc.barrier_back(global);
    table_ref
}

fn stack_u32(state: &mut LuaState, index: i32) -> Option<u32> {
    match stack_val(state, index) {
        Val::Num(n) if n >= 0.0 => Some(n as u32),
        _ => None,
    }
}

fn stack_string(state: &mut LuaState, index: i32) -> Option<String> {
    Option::<String>::from_stack(state, index).ok().flatten()
}

/// Resolve a macro reference — numeric = 1-based slot, string = name —
/// into a 0-based slot index within the macro table.
fn resolve_macro_slot(state: &mut LuaState, index: i32, macros: &[MacroInfo]) -> Option<usize> {
    match stack_val(state, index) {
        Val::Num(n) if n >= 1.0 => Some(n as usize - 1),
        Val::Str(_) => {
            let name = stack_string(state, index)?;
            macros.iter().position(|m| m.name == name)
        }
        _ => None,
    }
}

/// `PickupSpell(spellId)` — cursor = Spell.
fn pickup_spell(state: &mut LuaState) -> LuaResult<u32> {
    let Some(spell_id) = stack_u32(state, 1) else {
        return Ok(0);
    };
    borrow_state_mut(state)?.cursor_item = Some(CursorInfo::Spell { spell_id });
    Ok(0)
}

/// `PickupTalent(talentId)` — cursor = Talent (pvp=false).
fn pickup_talent(state: &mut LuaState) -> LuaResult<u32> {
    pickup_talent_with_pvp_flag(state, false)
}

/// `PickupPvpTalent(talentId)` — cursor = Talent (pvp=true).
fn pickup_pvp_talent(state: &mut LuaState) -> LuaResult<u32> {
    pickup_talent_with_pvp_flag(state, true)
}

fn pickup_talent_with_pvp_flag(state: &mut LuaState, pvp: bool) -> LuaResult<u32> {
    let Some(talent_id) = stack_u32(state, 1) else {
        return Ok(0);
    };
    borrow_state_mut(state)?.cursor_item = Some(CursorInfo::Talent { talent_id, pvp });
    Ok(0)
}

/// `PickupPetAction(slot)` — cursor = PetAction with synthesized spell_id.
fn pickup_pet_action(state: &mut LuaState) -> LuaResult<u32> {
    let Some(slot) = stack_u32(state, 1) else {
        return Ok(0);
    };
    borrow_state_mut(state)?.cursor_item = Some(CursorInfo::PetAction {
        slot,
        spell_id: PET_ACTION_SPELL_OFFSET.saturating_add(slot),
    });
    Ok(0)
}

/// `PickupMacro(index)` — cursor = Macro(index).
fn pickup_macro(state: &mut LuaState) -> LuaResult<u32> {
    let Some(macro_index) = stack_u32(state, 1) else {
        return Ok(0);
    };
    borrow_state_mut(state)?.cursor_item = Some(CursorInfo::Macro { macro_index });
    Ok(0)
}

/// `RunMacro(index_or_name)` — flip `running_macro` to the resolved slot.
fn run_macro(state: &mut LuaState) -> LuaResult<u32> {
    let macros = borrow_state_mut(state)?.macros.clone();
    let Some(zero_based) = resolve_macro_slot(state, 1, &macros) else {
        return Ok(0);
    };
    if zero_based >= macros.len() {
        return Ok(0);
    }
    borrow_state_mut(state)?.running_macro = Some((zero_based + 1) as u32);
    Ok(0)
}

/// `StopMacro()` — clear `running_macro`.
fn stop_macro(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.running_macro = None;
    Ok(0)
}

/// `C_Macro.RunMacroText(text [, button])` — execute the supported secure macro
/// slash commands through the same globals SecureTemplates would have called.
fn run_macro_text(state: &mut LuaState) -> LuaResult<u32> {
    let Some(text) = stack_string(state, 1) else {
        return Ok(0);
    };

    for line in text.lines() {
        run_macro_text_line(state, line)?;
    }
    Ok(0)
}

fn run_macro_text_line(state: &mut LuaState, line: &str) -> LuaResult<()> {
    let trimmed = line.trim();
    if trimmed.is_empty() || !trimmed.starts_with('/') {
        return Ok(());
    }

    let Some((command, argument)) = split_macro_command(trimmed) else {
        return Ok(());
    };
    match command.as_str() {
        "/target" | "/tar" => call_named_global(state, "TargetUnit", argument),
        "/focus" => call_named_global(state, "FocusUnit", argument),
        "/cast" | "/spell" => call_named_global(state, "CastSpellByName", argument),
        _ => Ok(()),
    }
}

fn split_macro_command(line: &str) -> Option<(String, &str)> {
    let mut parts = line.splitn(2, char::is_whitespace);
    let command = parts.next()?.to_ascii_lowercase();
    let argument = parts.next().unwrap_or_default().trim();
    if argument.is_empty() {
        return None;
    }
    Some((command, argument))
}

fn call_named_global(state: &mut LuaState, name: &str, argument: &str) -> LuaResult<()> {
    let function = LuaApiMut::get_global_val(state, name);
    if !matches!(function, Val::Function(_)) {
        return Ok(());
    }
    let argument = create_string(state, argument);
    call_function_state(state, function, &[argument])?;
    Ok(())
}

/// `EditMacro(index_or_name, name?, icon?, body?)` — update a macro slot
/// in-place. Passing an index beyond current length grows the macro table
/// with empty entries until the slot exists.
fn edit_macro(state: &mut LuaState) -> LuaResult<u32> {
    let macros = borrow_state_mut(state)?.macros.clone();
    let slot = match stack_val(state, 1) {
        Val::Num(n) if n >= 1.0 => Some(n as usize - 1),
        Val::Str(_) => {
            let Some(name) = stack_string(state, 1) else {
                return Ok(0);
            };
            macros.iter().position(|m| m.name == name)
        }
        _ => None,
    };
    let Some(slot) = slot else {
        return Ok(0);
    };

    let new_name = stack_string(state, 2);
    let new_icon = stack_string(state, 3);
    let new_body = stack_string(state, 4);

    let mut st = borrow_state_mut(state)?;
    while st.macros.len() <= slot {
        st.macros.push(MacroInfo::default());
    }
    let entry = &mut st.macros[slot];
    if let Some(name) = new_name {
        entry.name = name;
    }
    if let Some(icon) = new_icon {
        entry.icon = icon;
    }
    if let Some(body) = new_body {
        entry.body = body;
    }
    Ok(0)
}

/// Allocates the first unused slot in the simulator's account/character range.
fn create_macro(state: &mut LuaState) -> LuaResult<u32> {
    let name = String::from_stack(state, 1)?;
    let icon = String::from_stack(state, 2)?;
    let body = String::from_stack(state, 3)?;
    let per_character = Option::<bool>::from_stack(state, 4)?.unwrap_or(false);
    if name.is_empty() {
        return Err(rilua::runtime_error("macro name must not be empty"));
    }
    let range = if per_character { 120..138 } else { 0..120 };
    let id = {
        let mut sim = borrow_state_mut(state)?;
        let slot = range
            .into_iter()
            .find(|&slot| {
                sim.macros
                    .get(slot)
                    .is_none_or(|entry| entry.name.is_empty())
            })
            .ok_or_else(|| rilua::runtime_error("macro storage is full"))?;
        let length = sim.macros.len().max(slot + 1);
        sim.macros.resize_with(length, MacroInfo::default);
        sim.macros[slot] = MacroInfo { name, icon, body };
        slot + 1
    };
    state.push(Val::Num(id as f64));
    Ok(1)
}

/// Keep IDs stable; deletion clears associations rather than renumbering slots.
fn delete_macro(state: &mut LuaState) -> LuaResult<u32> {
    let macros = borrow_state_mut(state)?.macros.clone();
    let Some(slot) = resolve_macro_slot(state, 1, &macros).filter(|&slot| slot < macros.len())
    else {
        return Ok(0);
    };
    let id = (slot + 1) as u32;
    let mut sim = borrow_state_mut(state)?;
    sim.macros[slot] = MacroInfo::default();
    sim.action_macros.retain(|_, value| *value != id);
    if sim.running_macro == Some(id) {
        sim.running_macro = None;
    }
    if matches!(sim.cursor_item, Some(CursorInfo::Macro { macro_index }) if macro_index == id) {
        sim.cursor_item = None;
    }
    Ok(0)
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "CreateMacro", create_macro)?;
    LuaApiMut::register_function(lua, "DeleteMacro", delete_macro)?;
    LuaApiMut::register_function(lua, "PickupSpell", pickup_spell)?;
    LuaApiMut::register_function(lua, "PickupTalent", pickup_talent)?;
    LuaApiMut::register_function(lua, "PickupPvpTalent", pickup_pvp_talent)?;
    LuaApiMut::register_function(lua, "PickupPetAction", pickup_pet_action)?;
    LuaApiMut::register_function(lua, "PickupMacro", pickup_macro)?;
    LuaApiMut::register_function(lua, "RunMacro", run_macro)?;
    LuaApiMut::register_function(lua, "StopMacro", stop_macro)?;
    LuaApiMut::register_function(lua, "EditMacro", edit_macro)?;
    let c_macro = ensure_namespace_table(lua.state_mut(), "C_Macro");
    table_set_rust_fn_static(lua.state_mut(), c_macro, "RunMacroText", run_macro_text)?;
    Ok(())
}
