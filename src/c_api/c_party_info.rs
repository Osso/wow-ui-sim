//! `C_PartyInfo` probe surface backed by group state.
//!
//! `GetActiveCategories`, `GetActiveGroupType`, `IsPartyFull`, and
//! `IsGUIDInGroup` read the existing party roster model. `LeaveParty` and
//! `UninviteUnit` mutate the same roster paths as their legacy globals. Static
//! loot availability remains seeded, while Retail 12.0.5 loot getter/setter
//! share the legacy model. Unrelated instance abandon defaults stay in
//! temporary workarounds.

use crate::c_api::helpers::{ensure_namespace, set_table_array};
use crate::lua_api::globals::group_queries::active_party_count;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_table};
use crate::lua_api::state::SEEDED_LOCAL_CHARACTER_GUID;
use crate::lua_bridge::FromStack;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

#[cfg(feature = "retail-12-0-5")]
pub(crate) mod countdown;
#[cfg(feature = "retail-12-0-5")]
mod loot_method;
#[cfg(feature = "retail-12-0-5")]
mod ping_restrictions;

const AVAILABLE_LOOT_METHODS: [i32; 5] = [0, 1, 2, 3, 4];

pub(crate) fn register_c_party_info_surface(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_PartyInfo")?;
    register_group_membership_probes(state, table_ref)?;
    register_loot_method_probes(state, table_ref)?;
    #[cfg(feature = "retail-12-0-5")]
    {
        ping_restrictions::register(state, table_ref)?;
        countdown::register(state, table_ref)?;
    }
    Ok(())
}

fn register_group_membership_probes(
    state: &mut LuaState,
    table_ref: GcRef<Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetActiveCategories",
        c_party_info_get_active_categories,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetActiveGroupType",
        c_party_info_get_active_group_type,
    )?;
    table_set_rust_fn_static(state, table_ref, "IsPartyFull", c_party_info_is_party_full)?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "IsGUIDInGroup",
        c_party_info_is_guid_in_group,
    )?;
    table_set_rust_fn_static(state, table_ref, "LeaveParty", c_party_info_leave_party)?;
    table_set_rust_fn_static(state, table_ref, "UninviteUnit", c_party_info_uninvite_unit)?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "DemoteAssistant",
        c_party_info_demote_assistant,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "PromoteToAssistant",
        c_party_info_promote_to_assistant,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "PromoteToLeader",
        c_party_info_promote_to_leader,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "SetEveryoneIsAssistant",
        c_party_info_set_everyone_is_assistant,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "DoReadyCheck",
        c_party_info_do_ready_check,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "ConfirmReadyCheck",
        c_party_info_confirm_ready_check,
    )?;
    Ok(())
}

fn register_loot_method_probes(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetAvailableLootMethods",
        c_party_info_get_available_loot_methods,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "IsLootMethodAvailable",
        c_party_info_is_loot_method_available,
    )?;
    #[cfg(not(feature = "retail-12-0-5"))]
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetLootMethod",
        c_party_info_get_loot_method,
    )?;
    #[cfg(feature = "retail-12-0-5")]
    loot_method::register(state, table_ref)?;
    Ok(())
}

fn c_party_info_get_active_categories(state: &mut LuaState) -> LuaResult<u32> {
    let member_count = active_party_count(state)?;
    let array = create_table(state);
    if member_count > 0 {
        set_table_array(state, array, 1, Val::Num(1.0));
    }
    state.push(array);
    Ok(1)
}

fn c_party_info_get_active_group_type(state: &mut LuaState) -> LuaResult<u32> {
    let member_count = active_party_count(state)?;
    if member_count == 0 {
        state.push(Val::Nil);
    } else if member_count >= 6 {
        state.push(Val::Num(1.0));
    } else {
        state.push(Val::Num(0.0));
    }
    Ok(1)
}

fn c_party_info_is_party_full(state: &mut LuaState) -> LuaResult<u32> {
    let member_count = active_party_count(state)?;
    let full = if member_count == 0 {
        false
    } else if member_count >= 6 {
        member_count >= 39
    } else {
        member_count >= 4
    };
    state.push(Val::Bool(full));
    Ok(1)
}

fn c_party_info_is_guid_in_group(state: &mut LuaState) -> LuaResult<u32> {
    let guid = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let is_member = {
        let sim = borrow_state(state)?;
        sim.party_group_active
            && (guid == SEEDED_LOCAL_CHARACTER_GUID
                || sim
                    .party_members
                    .iter()
                    .enumerate()
                    .any(|(index, _)| party_member_guid(index) == guid))
    };
    state.push(Val::Bool(is_member));
    Ok(1)
}

fn party_member_guid(index: usize) -> String {
    format!("Player-0000-000000{:02}", index + 2)
}

fn party_member_index_from_unit(state: &mut LuaState, unit: &str) -> LuaResult<Option<usize>> {
    if unit == "player" {
        return Ok(None);
    }
    if let Some(index) = crate::lua_api::globals::unit_api::parse_party_index(unit) {
        return Ok(Some(index));
    }
    let sim = borrow_state(state)?;
    Ok(sim
        .party_members
        .iter()
        .position(|member| member.name == unit))
}

fn c_party_info_leave_party(state: &mut LuaState) -> LuaResult<u32> {
    crate::lua_api::globals::group_verbs::clear_party_roster(state)?;
    crate::lua_api::globals::group_verbs::push_event(state, "GROUP_ROSTER_UPDATE")?;
    Ok(0)
}

fn c_party_info_uninvite_unit(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let removed = {
        let mut sim = borrow_state_mut(state)?;
        if let Some(index) = crate::lua_api::globals::unit_api::parse_party_index(&unit)
            && index < sim.party_members.len()
        {
            sim.party_members.remove(index);
            true
        } else {
            let before = sim.party_members.len();
            sim.party_members.retain(|member| member.name != unit);
            before != sim.party_members.len()
        }
    };
    if removed {
        crate::lua_api::globals::group_verbs::push_event(state, "GROUP_ROSTER_UPDATE")?;
    }
    Ok(0)
}

fn c_party_info_demote_assistant(state: &mut LuaState) -> LuaResult<u32> {
    let _unit = Option::<String>::from_stack(state, 1)?;
    borrow_state_mut(state)?.everyone_assistant = false;
    Ok(0)
}

fn c_party_info_promote_to_assistant(state: &mut LuaState) -> LuaResult<u32> {
    let _unit = Option::<String>::from_stack(state, 1)?;
    borrow_state_mut(state)?.everyone_assistant = true;
    Ok(0)
}

fn c_party_info_promote_to_leader(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let leader_index = party_member_index_from_unit(state, &unit)?;
    borrow_state_mut(state)?.party_leader_index = leader_index;
    Ok(0)
}

fn c_party_info_set_everyone_is_assistant(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = Option::<bool>::from_stack(state, 1)?.unwrap_or(false);
    borrow_state_mut(state)?.everyone_assistant = enabled;
    Ok(0)
}

fn c_party_info_do_ready_check(state: &mut LuaState) -> LuaResult<u32> {
    crate::lua_api::globals::group_verbs::start_ready_check(state)?;
    Ok(0)
}

fn c_party_info_confirm_ready_check(state: &mut LuaState) -> LuaResult<u32> {
    crate::lua_api::globals::group_verbs::confirm_ready_check(state)?;
    Ok(0)
}

fn c_party_info_get_available_loot_methods(state: &mut LuaState) -> LuaResult<u32> {
    let array = create_table(state);
    for (index, method) in AVAILABLE_LOOT_METHODS.iter().enumerate() {
        set_table_array(state, array, index as i64 + 1, Val::Num(*method as f64));
    }
    state.push(array);
    Ok(1)
}

fn c_party_info_is_loot_method_available(state: &mut LuaState) -> LuaResult<u32> {
    let method = i32::from_stack(state, 1)?;
    let available = matches!(method, 0..=4);
    state.push(Val::Bool(available));
    Ok(1)
}

#[cfg(not(feature = "retail-12-0-5"))]
fn c_party_info_get_loot_method(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(3.0));
    state.push(Val::Nil);
    state.push(Val::Nil);
    Ok(3)
}
