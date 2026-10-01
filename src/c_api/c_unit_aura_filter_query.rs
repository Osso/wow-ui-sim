//! Bounded Retail 12.0.5 instance filtering over the existing aura model.

use crate::lua_api::globals::auras::{aura_matches_filter_string, find_aura_by_instance_id};
use crate::lua_api::methods::val_to_string;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::table_security::unwrap_secret;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_UnitAuras")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "IsAuraFilteredOutByInstanceID",
        is_aura_filtered_out_by_instance_id,
    )
}

fn is_aura_filtered_out_by_instance_id(state: &mut LuaState) -> LuaResult<u32> {
    // All documented arguments are AllowedWhenUntainted; the VM retains
    // caller taint and original wrapper secrecy. UnitAuraAccess is unmodeled.
    let unit = read_authenticated_string(state, 1)?;
    let instance_id = read_authenticated_instance_id(state)?;
    let filter = read_authenticated_string(state, 3)?;
    // INFERRED: absent instances are filtered. Unfiltered lookup deliberately
    // keeps blocked records resolvable, unlike public aura enumeration.
    let is_filtered = find_aura_by_instance_id(state, &unit, instance_id)
        .is_none_or(|aura| !aura_matches_filter_string(&aura, &filter));
    state.push(Val::Bool(is_filtered));
    Ok(1)
}

fn read_authenticated_string(state: &LuaState, index: i32) -> LuaResult<String> {
    let value = unwrap_secret(state, stack_val(state, index))?;
    // INFERRED representation policy: no nil default or numeric coercion.
    val_to_string(state, value).ok_or_else(|| {
        runtime_error(format!(
            "C_UnitAuras.IsAuraFilteredOutByInstanceID: argument {index} must be a UTF-8 string"
        ))
    })
}

fn read_authenticated_instance_id(state: &LuaState) -> LuaResult<i32> {
    let value = unwrap_secret(state, stack_val(state, 2))?;
    // INFERRED representation policy: signed IDs are valid, lossy casts are not.
    match value {
        Val::Num(number)
            if number.is_finite()
                && number.fract() == 0.0
                && number >= i32::MIN as f64
                && number <= i32::MAX as f64 =>
        {
            Ok(number as i32)
        }
        _ => Err(runtime_error(
            "C_UnitAuras.IsAuraFilteredOutByInstanceID: argument 2 must be a finite integral i32 number",
        )),
    }
}
