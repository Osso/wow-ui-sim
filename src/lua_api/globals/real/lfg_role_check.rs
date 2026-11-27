//! LFG role check in progress (`GetLFGRoleUpdate`, Retail secure publication
//! `data/secure-publication/retail-12-1-5-70688.csv`).
//!
//! Returns `inProgress, slots, members, category, lfgID, isBattleground`, the
//! order read by `LFGUtilShared.lua` `GetLFGMode` and `QueueStatusFrame.lua`.
//! `A_Admin.SetLFGRoleCheck(info)` starts a role check and
//! `A_Admin.ClearLFGRoleCheck()` ends it. INFERRED R88g9 (native evidence
//! pending): without a role check the result is `false, 0, 0, 0, 0, false`,
//! the tuple the simulator reported before this model.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{FromStack, TableBuilder};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LfgRoleCheck {
    pub slots: i32,
    pub members: i32,
    pub category: i32,
    pub lfg_id: i32,
    pub is_battleground: bool,
}

pub(crate) fn register(lua: &mut rilua::Lua) -> LuaResult<()> {
    if matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr) {
        LuaApiMut::register_function(lua, "GetLFGRoleUpdate", get_lfg_role_update)?;
    }
    Ok(())
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder
        .set_function("SetLFGRoleCheck", set_lfg_role_check)?
        .set_function("ClearLFGRoleCheck", |state| {
            borrow_state_mut(state)?.lfg_role_check = None;
            Ok(0)
        })
}

fn get_lfg_role_update(state: &mut LuaState) -> LuaResult<u32> {
    let check = borrow_state(state)?.lfg_role_check.clone();
    let in_progress = check.is_some();
    let check = check.unwrap_or_default();
    state.push(Val::Bool(in_progress));
    for value in [check.slots, check.members, check.category, check.lfg_id] {
        state.push(Val::Num(f64::from(value)));
    }
    state.push(Val::Bool(check.is_battleground));
    Ok(6)
}

fn set_lfg_role_check(state: &mut LuaState) -> LuaResult<u32> {
    let category = i32::from_stack(state, 1)?;
    let lfg_id = i32::from_stack(state, 2)?;
    let slots = i32::from_stack(state, 3)?;
    let members = i32::from_stack(state, 4)?;
    let is_battleground = bool::from_stack(state, 5)?;
    if category < 1 || lfg_id < 0 || slots < 0 || members < 0 {
        return Err(runtime_error(
            "A_Admin.SetLFGRoleCheck: expected category >= 1 and non-negative lfgID, slots, members",
        ));
    }
    borrow_state_mut(state)?.lfg_role_check = Some(LfgRoleCheck {
        slots,
        members,
        category,
        lfg_id,
        is_battleground,
    });
    Ok(0)
}
