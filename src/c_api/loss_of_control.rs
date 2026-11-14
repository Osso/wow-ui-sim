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

#[cfg(test)]
mod tests {
    use super::push_loss_of_control_info;
    use crate::lua_api::methods::table_get_static;
    use crate::lua_api::{LossOfControlInfo, WowLuaEnv};
    use rilua::Val;

    #[test]
    fn verified_serializer_shape_has_five_fields_and_native_types() {
        let env = WowLuaEnv::new().expect("Lua environment should initialize");
        let mut lua = env.rilua_mut();
        let state = lua.state_mut();
        let saved_base = state.base;
        let saved_top = state.top;
        state.base = saved_top;
        let info = LossOfControlInfo {
            start_time: 3.0,
            duration: 5.0,
            mod_rate: 1.0,
            is_active: true,
            should_replace_normal_cooldown: false,
        };

        // Internal serializer contract only; this does not prove the C_SpellBook
        // handler reaches this publisher (UNKNOWN in R88j.2 addendum).
        assert_eq!(push_loss_of_control_info(state, &info, false).unwrap(), 1);
        let table = state.stack_get(state.top - 1);
        assert!(matches!(table, Val::Table(_)));
        assert!(matches!(
            table_get_static(state, table, "startTime"),
            Val::Num(_)
        ));
        assert!(matches!(
            table_get_static(state, table, "duration"),
            Val::Num(_)
        ));
        assert!(matches!(
            table_get_static(state, table, "modRate"),
            Val::Num(_)
        ));
        assert!(matches!(
            table_get_static(state, table, "isActive"),
            Val::Bool(_)
        ));
        assert!(matches!(
            table_get_static(state, table, "shouldReplaceNormalCooldown"),
            Val::Bool(_)
        ));

        state.top = saved_top;
        state.base = saved_base;
    }
}
