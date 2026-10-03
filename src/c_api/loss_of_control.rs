//! Shared five-field loss-of-control snapshot serialization.

use crate::lua_api::LossOfControlInfo;
use crate::lua_api::methods::{create_table_with_capacity, table_set_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn push_loss_of_control_info(
    state: &mut LuaState,
    info: &LossOfControlInfo,
    restricted: bool,
) -> LuaResult<u32> {
    let table = create_table_with_capacity(state, 5);
    // Root the ordinary result before keys or host-secret wrappers allocate.
    state.push(table);
    for (key, number) in [
        ("startTime", info.start_time),
        ("duration", info.duration),
        ("modRate", f64::from(info.mod_rate)),
    ] {
        let value = if restricted {
            rilua::table_security::wrap_host_secret_number(state, number)
        } else {
            Val::Num(number)
        };
        table_set_static(state, table, key, value);
    }
    table_set_static(state, table, "isActive", Val::Bool(info.is_active));
    table_set_static(
        state,
        table,
        "shouldReplaceNormalCooldown",
        Val::Bool(info.should_replace_normal_cooldown),
    );
    Ok(1)
}
