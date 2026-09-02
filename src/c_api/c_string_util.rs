//! C_StringUtil: string escaping helpers used by Blizzard diagnostics.

use crate::lua_api::methods::{create_string, val_to_string};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

use super::helpers::ensure_global_table;

/// Register `C_StringUtil`, keeping whatever the namespace already holds.
///
/// This runs a second time from `workarounds::temporary::environment_cleanup_restore`,
/// which is after the workaround layer has installed
/// `C_StringUtil.CreateSecondsFormatter`. Building a fresh table and assigning it
/// over the global discarded that factory, so
/// `Blizzard_AuraContainer/Blizzard_AuraContainerShared.lua:94` called
/// `SetRounding` on nil and aborted the file. `ensure_global_table` reuses the
/// existing table when there is one.
pub fn register_c_string_util(state: &mut LuaState) -> LuaResult<()> {
    let Val::Table(c_string_util_ref) = ensure_global_table(state, "C_StringUtil") else {
        unreachable!("ensure_global_table must return a table");
    };
    table_set_rust_fn_static(
        state,
        c_string_util_ref,
        "EscapeQuotedCodes",
        c_string_util_escape_quoted_codes,
    )?;
    Ok(())
}

pub fn c_string_util_escape_quoted_codes(state: &mut LuaState) -> LuaResult<u32> {
    let Some(input) = val_to_string(state, stack_val(state, 1)) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let escaped = input.replace('|', "||");
    let escaped_value = create_string(state, &escaped);
    state.push(escaped_value);
    Ok(1)
}
