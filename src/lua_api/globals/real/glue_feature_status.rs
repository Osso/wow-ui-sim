//! R44-Q18: server-supplied Glue feature status, independent of realm/character.

use crate::lua_api::WowLuaEnv;
use crate::lua_api::methods::borrow_state;
use crate::lua_api::methods::table_set;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};

/// The modeled field of a host-delivered feature-system status record.
#[derive(Clone, Copy, Debug, Default)]
pub struct GlueFeatureStatus {
    pub name_reservation_only: bool,
}

pub(crate) fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    let glue = borrow_state(lua.state_mut())?.screen_kind.is_glue();
    if glue {
        lua.register_function("IsNameReservationOnly", is_name_reservation_only)?;
    } else {
        let state = lua.state_mut();
        table_set(
            state,
            Val::Table(state.global),
            "IsNameReservationOnly",
            Val::Nil,
        );
    }
    Ok(())
}

fn is_name_reservation_only(state: &mut LuaState) -> LuaResult<u32> {
    let value = borrow_state(state)?
        .glue_feature_status
        .name_reservation_only;
    state.push(Val::Bool(value));
    Ok(1)
}

/// Deliver the host record with the two no-payload notifications in R44 order.
/// This host action dispatches synchronously; native callback latency is not
/// asserted by the model or tests.
pub fn deliver_status(env: &WowLuaEnv, status: GlueFeatureStatus) -> crate::Result<()> {
    env.fire_event("TIMERUNNING_SEASON_UPDATE")?;
    env.state().borrow_mut().glue_feature_status = status;
    env.fire_event("AVAILABLE_GAME_MODES_UPDATED")
}
