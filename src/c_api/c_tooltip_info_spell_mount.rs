//! Retail 12.0.5 spell/mount identifier and NeverSecret query boundaries.
//! Alias and arg1 rejection policies are inferred, not native permission evidence.

use super::c_spell::read_public_spell_identifier_at;
use crate::lua_api::globals::missing_surface::tooltip_info::tooltip_for_spell_identifier;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::LuaResult;
use rilua::table_security::is_secret_value;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;

/// Publish only after the caller has installed the namespace in globals.
pub(crate) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetMountBySpellID", get_mount_by_spell_id)?;
    table_set_rust_fn_static(state, namespace, "GetSpellByID", get_spell_by_id)?;
    Ok(())
}

fn get_mount_by_spell_id(state: &mut LuaState) -> LuaResult<u32> {
    push_tooltip(state, "C_TooltipInfo.GetMountBySpellID", 2, true)
}

fn get_spell_by_id(state: &mut LuaState) -> LuaResult<u32> {
    push_tooltip(state, "C_TooltipInfo.GetSpellByID", 6, false)
}

fn push_tooltip(
    state: &mut LuaState,
    api_name: &str,
    last_optional: i32,
    is_mount: bool,
) -> LuaResult<u32> {
    reject_secret_arguments(state, api_name, last_optional)?;
    let spell_id = read_public_spell_identifier_at(state, 1, api_name)?;
    // Public optional values retain the existing ignored-provider behavior.
    // Producers return Val, not GcRef; publish immediately without another allocation.
    let tooltip = tooltip_for_spell_identifier(state, spell_id, is_mount);
    state.push(tooltip);
    Ok(1)
}

fn reject_secret_arguments(state: &LuaState, api_name: &str, last_optional: i32) -> LuaResult<()> {
    // Check every boundary before the shared helper can read aliases or state.
    // Never unwrap private payloads or alter taint, even for secure callers.
    for index in 1..=last_optional {
        if is_secret_value(state, stack_val(state, index)) {
            let policy = if index == 1 {
                "conservative public spell identifier policy (native AllowedWhenTainted unknown)"
            } else {
                "NeverSecret policy"
            };
            return Err(rilua::runtime_error(format!(
                "{api_name}: argument {index} rejects secret values under {policy}"
            )));
        }
    }
    Ok(())
}
