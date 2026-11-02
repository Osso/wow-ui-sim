//! Supplied current-area identity and AreaTable flags.

use std::collections::HashMap;

use crate::lua_api::methods::borrow_state;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const PVP_POI_FLAG: u32 = 32768;

#[derive(Debug, Clone, Default)]
pub struct AreaPvpInputs {
    pub current_area_id: Option<u32>,
    pub area_flags: HashMap<u32, u32>,
}

pub(crate) fn is_subzone_pvp_poi(state: &mut LuaState) -> LuaResult<u32> {
    let flagged = {
        let sim = borrow_state(state)?;
        let inputs = &sim.pvp_area_inputs;
        inputs
            .current_area_id
            .and_then(|id| inputs.area_flags.get(&id))
            .is_some_and(|flags| flags & PVP_POI_FLAG != 0)
    };
    state.push(Val::Bool(flagged));
    Ok(1)
}
