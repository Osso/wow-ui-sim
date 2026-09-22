//! Console command records for the simulator's supported CVar catalog.

use crate::c_api::helpers::set_table_array;
use crate::lua_api::methods::{borrow_state, create_string, create_table, table_set};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const CONSOLE_CATEGORY_NONE: f64 = 10.0;
const CONSOLE_COMMAND_TYPE_CVAR: f64 = 0.0;

pub(crate) fn get_all_commands(state: &mut LuaState) -> LuaResult<u32> {
    let names = borrow_state(state)?.cvars.all_keys();
    let commands = create_table(state);
    for (index, name) in names.iter().enumerate() {
        let entry = create_cvar_record(state, name);
        set_table_array(state, commands, index as i64 + 1, entry);
    }
    state.push(commands);
    Ok(1)
}

fn create_cvar_record(state: &mut LuaState, name: &str) -> Val {
    let entry = create_table(state);
    let command = create_string(state, name);
    table_set(state, entry, "command", command);
    table_set(state, entry, "category", Val::Num(CONSOLE_CATEGORY_NONE));
    table_set(
        state,
        entry,
        "commandType",
        Val::Num(CONSOLE_COMMAND_TYPE_CVAR),
    );
    // CVar storage has no native help/category/script metadata. These fields
    // match the Wowless CVar-only catalog policy, not a complete client catalog.
    let empty = create_string(state, "");
    for field in ["help", "scriptContents", "scriptParameters"] {
        table_set(state, entry, field, empty);
    }
    entry
}
