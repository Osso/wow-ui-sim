//! Per-member search result rows: `C_LFGList.GetSearchResultLeaderInfo` and
//! `GetSearchResultPlayerInfo` (Forever 70707 `LFGListInfoDocumentation.lua:411-443`)
//! publish `LfgSearchResultPlayerInfo` (`:1017-1031`) from rows supplied with
//! `A_Admin.SetPremadeListingMembers(searchResultID, members)`.
//! A listing without supplied rows returns nothing, which both documented
//! functions allow (`MayReturnNothing`).

use crate::c_api::c_lfg_list_roles::LfgRoles;
use crate::lua_api::globals::lfg_list::premade_listing;
use crate::lua_api::methods::{
    borrow_state_mut, create_string, create_table, table_get, table_set,
};
use crate::lua_bridge::{FromStack, TableBuilder, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const ADMIN: &str = "A_Admin.SetPremadeListingMembers";

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LfgSearchMember {
    pub name: Option<String>,
    pub level: Option<f64>,
    pub area_name: Option<String>,
    pub class_name: String,
    pub class_filename: String,
    pub spec_name: Option<String>,
    pub assigned_role: String,
    pub lfg_roles: LfgRoles,
    pub is_leader: bool,
    pub is_leaver: bool,
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::WowForever {
        return Ok(builder);
    }
    builder.set_function("SetPremadeListingMembers", admin_set_members)
}

pub(crate) fn get_search_result_leader_info(state: &mut LuaState) -> LuaResult<u32> {
    let result_id = u32::from_stack(state, 1)?;
    let leader = premade_listing(state, result_id)
        .and_then(|listing| listing.members.into_iter().find(|member| member.is_leader));
    push_member(state, leader)
}

pub(crate) fn get_search_result_player_info(state: &mut LuaState) -> LuaResult<u32> {
    let result_id = u32::from_stack(state, 1)?;
    let member_index = i64::from_stack(state, 2)?;
    let member = premade_listing(state, result_id).and_then(|listing| {
        usize::try_from(member_index - 1)
            .ok()
            .and_then(|index| listing.members.get(index).cloned())
    });
    push_member(state, member)
}

fn push_member(state: &mut LuaState, member: Option<LfgSearchMember>) -> LuaResult<u32> {
    let Some(member) = member else {
        return Ok(0);
    };
    let info = create_table(state);
    for (key, value) in [
        ("name", member.name.as_deref()),
        ("areaName", member.area_name.as_deref()),
        ("className", Some(member.class_name.as_str())),
        ("classFilename", Some(member.class_filename.as_str())),
        ("specName", member.spec_name.as_deref()),
        ("assignedRole", Some(member.assigned_role.as_str())),
    ] {
        if let Some(value) = value {
            let value = create_string(state, value);
            table_set(state, info, key, value);
        }
    }
    if let Some(level) = member.level {
        table_set(state, info, "level", Val::Num(level));
    }
    table_set(state, info, "isLeader", Val::Bool(member.is_leader));
    table_set(state, info, "isLeaver", Val::Bool(member.is_leaver));
    let roles = create_table(state);
    table_set(state, roles, "tank", Val::Bool(member.lfg_roles.tank));
    table_set(state, roles, "healer", Val::Bool(member.lfg_roles.healer));
    table_set(state, roles, "dps", Val::Bool(member.lfg_roles.dps));
    table_set(state, info, "lfgRoles", roles);
    state.push(info);
    Ok(1)
}

fn admin_set_members(state: &mut LuaState) -> LuaResult<u32> {
    let result_id = u32::from_stack(state, 1)?;
    let rows = read_array(state, stack_val(state, 2))?;
    let members = rows
        .into_iter()
        .map(|row| read_member(state, row))
        .collect::<LuaResult<Vec<_>>>()?;
    let mut sim = borrow_state_mut(state)?;
    let listing = sim
        .world
        .premade_listings
        .iter_mut()
        .find(|listing| listing.search_result_id == result_id)
        .ok_or_else(|| runtime_error(format!("{ADMIN}: unknown search result {result_id}")))?;
    listing.members = members;
    Ok(0)
}

fn read_array(state: &LuaState, value: Val) -> LuaResult<Vec<Val>> {
    let error = || runtime_error(format!("{ADMIN}: members must be an array of tables"));
    let Val::Table(array) = value else {
        return Err(error());
    };
    let table = state.gc.tables.get(array).ok_or_else(error)?;
    let length = table.len(&state.gc.string_arena);
    Ok((1..=length)
        .map(|index| table.get_int(index as i64))
        .collect())
}

fn read_member(state: &mut LuaState, row: Val) -> LuaResult<LfgSearchMember> {
    if !matches!(row, Val::Table(_)) {
        return Err(runtime_error(format!("{ADMIN}: member must be a table")));
    }
    Ok(LfgSearchMember {
        name: optional_string(state, row, "name")?,
        level: optional_number(state, row, "level")?,
        area_name: optional_string(state, row, "areaName")?,
        class_name: required_string(state, row, "className")?,
        class_filename: required_string(state, row, "classFilename")?,
        spec_name: optional_string(state, row, "specName")?,
        assigned_role: required_string(state, row, "assignedRole")?,
        lfg_roles: read_roles(state, row)?,
        is_leader: required_bool(state, row, "isLeader")?,
        is_leaver: optional_bool(state, row, "isLeaver")?.unwrap_or(false),
    })
}

fn read_roles(state: &mut LuaState, row: Val) -> LuaResult<LfgRoles> {
    let roles = table_get(state, row, "lfgRoles");
    if !matches!(roles, Val::Table(_)) {
        return Err(runtime_error(format!("{ADMIN}: lfgRoles must be a table")));
    }
    Ok(LfgRoles {
        tank: required_bool(state, roles, "tank")?,
        healer: required_bool(state, roles, "healer")?,
        dps: required_bool(state, roles, "dps")?,
    })
}

fn invalid(key: &str) -> rilua::LuaError {
    runtime_error(format!("{ADMIN}: invalid {key}"))
}

fn optional_string(state: &mut LuaState, row: Val, key: &str) -> LuaResult<Option<String>> {
    match table_get(state, row, key) {
        Val::Nil => Ok(None),
        value @ Val::Str(_) => Ok(Some(
            crate::lua_api::methods::val_to_string(state, value).ok_or_else(|| invalid(key))?,
        )),
        _ => Err(invalid(key)),
    }
}

fn required_string(state: &mut LuaState, row: Val, key: &str) -> LuaResult<String> {
    optional_string(state, row, key)?.ok_or_else(|| invalid(key))
}

fn optional_number(state: &mut LuaState, row: Val, key: &str) -> LuaResult<Option<f64>> {
    match table_get(state, row, key) {
        Val::Nil => Ok(None),
        Val::Num(value) if value.is_finite() => Ok(Some(value)),
        _ => Err(invalid(key)),
    }
}

fn optional_bool(state: &mut LuaState, row: Val, key: &str) -> LuaResult<Option<bool>> {
    match table_get(state, row, key) {
        Val::Nil => Ok(None),
        Val::Bool(value) => Ok(Some(value)),
        _ => Err(invalid(key)),
    }
}

fn required_bool(state: &mut LuaState, row: Val, key: &str) -> LuaResult<bool> {
    optional_bool(state, row, key)?.ok_or_else(|| invalid(key))
}
