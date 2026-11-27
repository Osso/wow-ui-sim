//! Current account region (`ExpansionDocumentation.lua` `GetCurrentRegion`,
//! `GetCurrentRegionName`; Retail :46, Forever :55).
//!
//! `A_Admin.SetCurrentRegion(regionID, regionName)` supplies both values.
//! INFERRED R88g9 (native evidence pending): the default is region 1 named
//! `"US"`, the pair the simulator already reported before this model.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_bridge::{FromStack, TableBuilder};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

#[derive(Clone, Debug, PartialEq)]
pub struct RegionInfo {
    pub id: i32,
    pub name: String,
}

impl Default for RegionInfo {
    fn default() -> Self {
        Self {
            id: 1,
            name: "US".to_string(),
        }
    }
}

pub(crate) fn register(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "GetCurrentRegion", get_current_region)?;
    if matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    ) {
        LuaApiMut::register_function(lua, "GetCurrentRegionName", get_current_region_name)?;
    }
    Ok(())
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder.set_function("SetCurrentRegion", set_current_region)
}

fn get_current_region(state: &mut LuaState) -> LuaResult<u32> {
    let id = borrow_state(state)?.current_region.id;
    state.push(Val::Num(f64::from(id)));
    Ok(1)
}

fn get_current_region_name(state: &mut LuaState) -> LuaResult<u32> {
    let name = borrow_state(state)?.current_region.name.clone();
    let value = create_string(state, &name);
    state.push(value);
    Ok(1)
}

fn set_current_region(state: &mut LuaState) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)?;
    let name = String::from_stack(state, 2)?;
    if id < 1 || name.is_empty() {
        return Err(runtime_error(
            "A_Admin.SetCurrentRegion: expected a positive regionID and a non-empty regionName",
        ));
    }
    borrow_state_mut(state)?.current_region = RegionInfo { id, name };
    Ok(0)
}
