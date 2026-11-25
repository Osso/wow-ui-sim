//! Macro C API projections backed by the shared MacroInfo rows.

use crate::c_api::helpers::ensure_namespace;
use crate::lua_api::globals::real::macro_getters::{
    macro_at_one_based_id, macro_counts, selected_macro_icon,
};
use crate::lua_api::methods::{borrow_state, create_string};
use crate::lua_bridge::{FromStack, TableBuilder, table_set_rust_fn_static};
use rilua::table_security::unwrap_secret;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};
use serde::Deserialize;

const QUESTION_MARK_ICON: u32 = 134400;

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct MacroIconCatalog {
    loose_spells: Vec<Option<String>>,
    loose_items: Vec<Option<String>>,
    spells: Vec<i32>,
    items: Vec<i32>,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !crate::lua_api::globals::real::macro_getters::USE_MODELED_MACRO_GETTERS {
        return Ok(());
    }
    publish_macro_constants(state)?;
    let namespace = ensure_namespace(state, "C_Macro")?;
    table_set_rust_fn_static(state, namespace, "GetNumMacros", get_num_macros)?;
    table_set_rust_fn_static(state, namespace, "GetMacroName", get_macro_name)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetSelectedMacroIcon",
        get_selected_macro_icon,
    )
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    if crate::lua_api::globals::real::macro_getters::USE_MODELED_MACRO_GETTERS {
        builder
            .set_function("SetMacroRows", set_macro_rows)?
            .set_function("SetMacroIconCatalog", set_macro_icon_catalog)
    } else {
        Ok(builder)
    }
}

fn publish_macro_constants(state: &mut LuaState) -> LuaResult<()> {
    let constants = ensure_namespace(state, "Constants")?;
    let macro_constants = TableBuilder::new(state)
        .set("MAX_ACCOUNT_MACROS", 120_u32)?
        .set("MAX_CHARACTER_MACROS", 30_u32)?
        .build();
    let table = rilua::Table::from_gc_ref(constants);
    let key = create_string(state, "MacroConsts");
    table.raw_set(state, key, macro_constants)?;
    Ok(())
}

fn get_num_macros(state: &mut LuaState) -> LuaResult<u32> {
    let (account, character) = macro_counts(&borrow_state(state)?.macros);
    state.push(Val::Num(account as f64));
    state.push(Val::Num(character as f64));
    Ok(2)
}

fn macro_id(state: &mut LuaState) -> LuaResult<Option<u32>> {
    let value = unwrap_secret(state, crate::lua_bridge::stack_val(state, 1))
        .map_err(|_| runtime_error("C_Macro getter requires an unsigned macro ID"))?;
    let number = match value {
        Val::Num(number) => Some(number),
        Val::Str(handle) => crate::c_api::parse_numeric_string(
            state
                .gc
                .string_arena
                .get(handle)
                .map_or(&[], |text| text.data()),
        ),
        _ => None,
    };
    Ok(number.and_then(|number| {
        (number.is_finite() && number >= 1.0 && number <= f64::from(u32::MAX))
            .then_some(number as u32)
    }))
}

fn get_macro_name(state: &mut LuaState) -> LuaResult<u32> {
    let Some(id) = macro_id(state)? else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let name = {
        let sim = borrow_state(state)?;
        macro_at_one_based_id(&sim.macros, id).map(|row| row.name.clone())
    };
    let value = name.map_or(Val::Nil, |name| create_string(state, &name));
    state.push(value);
    Ok(1)
}

fn get_selected_macro_icon(state: &mut LuaState) -> LuaResult<u32> {
    let Some(id) = macro_id(state)? else {
        state.push(Val::Num(f64::from(QUESTION_MARK_ICON)));
        return Ok(1);
    };
    let icon = {
        let sim = borrow_state(state)?;
        macro_at_one_based_id(&sim.macros, id)
            .and_then(|row| selected_macro_icon(&row.icon))
            .unwrap_or(QUESTION_MARK_ICON)
    };
    state.push(Val::Num(f64::from(icon)));
    Ok(1)
}

fn set_macro_rows(state: &mut LuaState) -> LuaResult<u32> {
    let json = String::from_stack(state, 1)?;
    let rows: Vec<crate::lua_api::state::MacroInfo> = serde_json::from_str(&json)
        .map_err(|_| runtime_error("A_Admin.SetMacroRows requires a JSON array"))?;
    crate::lua_api::methods::borrow_state_mut(state)?.macros = rows;
    Ok(0)
}

fn set_macro_icon_catalog(state: &mut LuaState) -> LuaResult<u32> {
    let json = String::from_stack(state, 1)?;
    let catalog: MacroIconCatalog = serde_json::from_str(&json)
        .map_err(|_| runtime_error("A_Admin.SetMacroIconCatalog requires a JSON object"))?;
    let mut sim = crate::lua_api::methods::borrow_state_mut(state)?;
    sim.macro_icons.loose_spells = catalog.loose_spells;
    sim.macro_icons.loose_items = catalog.loose_items;
    sim.macro_icons.spells = catalog.spells;
    sim.macro_icons.items = catalog.items;
    Ok(0)
}
