//! Retail legacy macro getter shape from 12.1.5.70688 R88l.18–23.
//! The simulator supplies records, not the native icon enumeration machinery.

use crate::c_api::parse_numeric_string;
use crate::lua_api::methods::{borrow_state, create_string, create_table};
use crate::lua_api::state::MacroInfo;
use crate::lua_bridge::stack_val;
use rilua::table_security::{TableAccess, check_table_access, unwrap_secret};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

mod inferred_selection;

use inferred_selection::{by_name, is_occupied};

// Both registration and the compatibility bootstrap use this publication decision.
pub(crate) const USE_MODELED_MACRO_GETTERS: bool = matches!(
    crate::client_profile::ACTIVE,
    crate::client_profile::ClientProfile::Retail | crate::client_profile::ClientProfile::WowForever
);

const MACRO_INFO_SLOTS: usize = 240;

// INFERRED R88l.18 ingress: account/character are the tab partitions observed
// in Retail MacroFrame:SetAccountMacros/SetCharacterMacros, not labels proved
// by the native getter. Use those documented UI sizes without changing writers.
const ACCOUNT_SLOTS: usize = 120;
const CHARACTER_SLOTS: usize = 30;

pub(crate) fn register(lua: &mut rilua::Lua) -> crate::Result<()> {
    if !USE_MODELED_MACRO_GETTERS {
        return Ok(());
    }
    LuaApiMut::register_function(lua, "GetNumMacros", get_num_macros)?;
    LuaApiMut::register_function(lua, "GetMacroInfo", get_macro_info)?;
    LuaApiMut::register_function(lua, "GetMacroBody", get_macro_body)?;
    for (name, handler) in [
        ("GetLooseMacroIcons", get_loose_spell_icons as rilua::RustFn),
        ("GetLooseMacroItemIcons", get_loose_item_icons),
        ("GetMacroIcons", get_spell_icons),
        ("GetMacroItemIcons", get_item_icons),
    ] {
        LuaApiMut::register_function(lua, name, handler)?;
    }
    Ok(())
}

pub(crate) fn macro_counts(rows: &[MacroInfo]) -> (usize, usize) {
    let account_end = rows.len().min(ACCOUNT_SLOTS);
    let character_end = rows.len().min(ACCOUNT_SLOTS + CHARACTER_SLOTS);
    let count = |range: &[MacroInfo]| range.iter().filter(|row| is_occupied(row)).count();
    (
        count(&rows[..account_end]),
        count(&rows[account_end..character_end]),
    )
}

pub(crate) fn macro_at_one_based_id(rows: &[MacroInfo], id: u32) -> Option<&MacroInfo> {
    let rows = &rows[..rows.len().min(MACRO_INFO_SLOTS)];
    id.checked_sub(1)
        .and_then(|slot| rows.get(slot as usize))
        .filter(|row| is_occupied(row))
}

pub(crate) fn selected_macro_icon(icon: &str) -> Option<u32> {
    resolved_macro_icon(icon)
}

fn get_num_macros(state: &mut LuaState) -> LuaResult<u32> {
    let (account, character) = macro_counts(&borrow_state(state)?.macros);
    state.push(Val::Num(account as f64));
    state.push(Val::Num(character as f64));
    Ok(2)
}

fn get_macro_info(state: &mut LuaState) -> LuaResult<u32> {
    let Ok(input) = unwrap_secret(state, stack_val(state, 1)) else {
        return Ok(0);
    };
    let row = {
        let sim = borrow_state(state)?;
        let rows = &sim.macros[..sim.macros.len().min(MACRO_INFO_SLOTS)];
        select_macro_record(state, rows, input).cloned()
    };
    let Some(row) = row else {
        return Ok(0);
    };

    let name = create_string(state, &row.name);
    state.push(name);
    state.push(resolved_macro_icon(&row.icon).map_or(Val::Nil, |icon| Val::Num(f64::from(icon))));
    let body = create_string(state, &row.body);
    state.push(body);
    Ok(3)
}

pub(crate) fn get_macro_body(state: &mut LuaState) -> LuaResult<u32> {
    let Ok(input) = unwrap_secret(state, stack_val(state, 1)) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let body = {
        let sim = borrow_state(state)?;
        let rows = &sim.macros[..sim.macros.len().min(MACRO_INFO_SLOTS)];
        select_macro_record(state, rows, input).map(|row| row.body.clone())
    };
    let value = body.map_or(Val::Nil, |body| create_string(state, &body));
    state.push(value);
    Ok(1)
}

fn select_macro_record<'a>(
    state: &LuaState,
    rows: &'a [MacroInfo],
    input: Val,
) -> Option<&'a MacroInfo> {
    match input {
        Val::Num(number) => by_index(rows, number),
        Val::Str(handle) => select_string(rows, state.gc.string_arena.get(handle)?.data()),
        _ => None,
    }
}

fn select_string<'a>(rows: &'a [MacroInfo], bytes: &[u8]) -> Option<&'a MacroInfo> {
    match parse_numeric_string(bytes) {
        Some(number) => by_index(rows, number),
        None => by_name(rows, bytes),
    }
}

fn by_index(rows: &[MacroInfo], number: f64) -> Option<&MacroInfo> {
    rows.get(inferred_selection::index(number)?)
        .filter(|row| is_occupied(row))
}

// INFERRED simulator representation adapter, not a claim about the native icon helper.
// Existing host macros can store either a numeric ID as text or a texture path.
fn resolved_macro_icon(icon: &str) -> Option<u32> {
    icon.parse()
        .ok()
        .or_else(|| crate::limited_listfile::lookup_texture_path(icon))
        .or_else(|| {
            (!icon.is_empty() && !icon.contains(['/', '\x5c']))
                .then(|| {
                    crate::limited_listfile::lookup_texture_path(&format!("Interface/Icons/{icon}"))
                })
                .flatten()
        })
        .filter(|id| *id != 0)
}

enum IconEntries {
    Loose(Vec<Option<String>>),
    Catalog(Vec<i32>),
}

fn get_loose_spell_icons(state: &mut LuaState) -> LuaResult<u32> {
    append_icons(state, "GetLooseMacroIcons", |sim| {
        IconEntries::Loose(sim.macro_icons.loose_spells.clone())
    })
}

fn get_loose_item_icons(state: &mut LuaState) -> LuaResult<u32> {
    append_icons(state, "GetLooseMacroItemIcons", |sim| {
        IconEntries::Loose(sim.macro_icons.loose_items.clone())
    })
}

fn get_spell_icons(state: &mut LuaState) -> LuaResult<u32> {
    append_icons(state, "GetMacroIcons", |sim| {
        IconEntries::Catalog(sim.macro_icons.spells.clone())
    })
}

fn get_item_icons(state: &mut LuaState) -> LuaResult<u32> {
    append_icons(state, "GetMacroItemIcons", |sim| {
        IconEntries::Catalog(sim.macro_icons.items.clone())
    })
}

fn append_icons(
    state: &mut LuaState,
    name: &str,
    supply: impl FnOnce(&crate::lua_api::SimState) -> IconEntries,
) -> LuaResult<u32> {
    let table = match unwrap_secret(state, stack_val(state, 1))? {
        Val::Nil => create_table(state),
        value @ Val::Table(_) => value,
        _ => return Err(rilua::runtime_error(format!("Usage: {name}([table])"))),
    };
    let Val::Table(table_ref) = table else {
        unreachable!("table creation must return a table");
    };
    check_table_access(state, table_ref, None, TableAccess::GetLength)?;
    state.push(table);
    let output = rilua::Table::from_gc_ref(table_ref);
    let offset = output.raw_len(state) + 1;
    let entries = {
        let sim = borrow_state(state)?;
        supply(&sim)
    };
    match entries {
        IconEntries::Loose(entries) => {
            for (index, text) in (offset..).zip(entries) {
                let value = text
                    .as_deref()
                    .map_or(Val::Nil, |s| create_string(state, s));
                output.raw_set(state, Val::Num(index as f64), value)?;
            }
        }
        IconEntries::Catalog(entries) => {
            for (index, value) in (offset..).zip(entries) {
                output.raw_set(state, Val::Num(index as f64), Val::Num(f64::from(value)))?;
            }
        }
    }
    Ok(1)
}
