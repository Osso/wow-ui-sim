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
#[cfg(any(not(feature = "retail-12-0-7"), feature = "client-retail"))]
use crate::lua_api::methods::borrow_state;
#[cfg(not(feature = "retail-12-0-7"))]
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_api::methods::create_table;
#[cfg(not(feature = "retail-12-0-7"))]
use crate::lua_api::state::SEEDED_LOCAL_CHARACTER_GUID;
use crate::lua_bridge::FromStack;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use crate::lua_bridge::TableBuilder;
#[cfg(feature = "retail-12-0-7")]
use crate::lua_bridge::stack_val;
use crate::lua_bridge::table_set_rust_fn_static;
#[cfg(any(feature = "retail-12-0-7", feature = "client-wowforever"))]
use rilua::runtime_error;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

#[cfg(feature = "retail-12-0-5")]
pub(crate) mod countdown;
#[cfg(feature = "retail-12-0-5")]
mod loot_method;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) mod pending_invites;
#[cfg(feature = "retail-12-0-5")]
mod ping_restrictions;
#[cfg(feature = "retail-12-0-7")]
pub(crate) mod roles_1207;
#[cfg(feature = "retail-12-0-7")]
pub(crate) mod solo_1207;

const AVAILABLE_LOOT_METHODS: [i32; 5] = [0, 1, 2, 3, 4];

/// Retail/Forever PartyInfoDocumentation.lua:196-201, 219-227, 230-245,
/// 338-344, 378-384, 550-562.
/// INFERRED (R88dt (native evidence pending)): supplied instance-abandon vote inputs.
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
#[derive(Default)]
pub(crate) struct InstanceAbandonVoteState {
    vote_duration: f64,
    vote_time_left: f64,
    shutdown_duration: f64,
    shutdown_time_left: f64,
    votes_required: f64,
    keystone_owner_vote_weight: f64,
    is_keystone_owner: bool,
    response: Option<bool>,
    group_responses: Vec<bool>,
}

/// Supplied party service policy observations, false until the host supplies them.
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
#[derive(Default)]
pub(crate) struct PartyPolicyFlags {
    to_raid: bool,
    to_party: bool,
    in_jailers_tower: bool,
    walk_in: bool,
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn register_party_policy(state: &mut LuaState, ns: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, ns, "AllowedToDoPartyConversion", allowed_to_convert)?;
    table_set_rust_fn_static(state, ns, "IsPartyInJailersTower", in_jailers_tower)?;
    table_set_rust_fn_static(state, ns, "IsPartyWalkIn", is_party_walk_in)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn allowed_to_convert(state: &mut LuaState) -> LuaResult<u32> {
    let to_raid = bool::from_stack(state, 1)?;
    let allowed = {
        let sim = borrow_state(state)?;
        if to_raid {
            sim.party_policy_flags.to_raid
        } else {
            sim.party_policy_flags.to_party
        }
    };
    state.push(Val::Bool(allowed));
    Ok(1)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn in_jailers_tower(state: &mut LuaState) -> LuaResult<u32> {
    let flag = borrow_state(state)?.party_policy_flags.in_jailers_tower;
    state.push(Val::Bool(flag));
    Ok(1)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn is_party_walk_in(state: &mut LuaState) -> LuaResult<u32> {
    let flag = borrow_state(state)?.party_policy_flags.walk_in;
    state.push(Val::Bool(flag));
    Ok(1)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn set_party_policy_flags(state: &mut LuaState) -> LuaResult<u32> {
    use crate::lua_bridge::stack_val;
    let [
        Val::Bool(to_raid),
        Val::Bool(to_party),
        Val::Bool(in_jailers_tower),
        Val::Bool(walk_in),
    ] = [1, 2, 3, 4].map(|index| stack_val(state, index))
    else {
        return Err(rilua::runtime_error(
            "A_Admin.SetPartyPolicyFlags: expected four booleans",
        ));
    };
    crate::lua_api::methods::borrow_state_mut(state)?.party_policy_flags = PartyPolicyFlags {
        to_raid,
        to_party,
        in_jailers_tower,
        walk_in,
    };
    Ok(0)
}

/// INFERRED (simulator mapping, R88bk pending): mark the supplied active group as a raid.
/// PartySync confirmation and event emission are UNKNOWN and remain unmodeled.
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn convert_to_raid(state: &mut LuaState) -> LuaResult<u32> {
    let mut sim = crate::lua_api::methods::borrow_state_mut(state)?;
    if sim.party_group_active {
        sim.party_is_raid = Some(true);
    }
    Ok(0)
}

/// INFERRED (simulator mapping): the invite is server-mediated, so local membership changes only on acceptance.
/// Events, confirmation and taint behavior are UNKNOWN pending R88bl.
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn invite_unit(state: &mut LuaState) -> LuaResult<u32> {
    let _target_name = String::from_stack(state, 1)?;
    Ok(0)
}

pub(crate) fn register_c_party_info_surface(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_PartyInfo")?;
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    pending_invites::register(state)?;
    register_group_membership_probes(state, table_ref)?;
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    register_instance_abandon_vote_surface(state, table_ref)?;
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    register_party_policy(state, table_ref)?;
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    table_set_rust_fn_static(state, table_ref, "ConvertToRaid", convert_to_raid)?;
    #[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
    table_set_rust_fn_static(state, table_ref, "InviteUnit", invite_unit)?;
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::Ptr
            | crate::client_profile::ClientProfile::WowForever
    ) {
        // PartyInfoDocumentation.lua:322-335 is byte-identical on Retail and Forever.
        table_set_rust_fn_static(state, table_ref, "GetMinLevel", c_party_info_get_min_level)?;
    } else if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Mists {
        // Mists 5.5.4.70268 PartyInfoDocumentation.lua:125–135; omitted category at130.
        table_set_rust_fn_static(state, table_ref, "GetMinLevel", mists_supplied_min_level)?;
    }
    register_loot_method_probes(state, table_ref)?;
    #[cfg(feature = "retail-12-0-5")]
    {
        ping_restrictions::register(state, table_ref)?;
        countdown::register(state, table_ref)?;
    }
    Ok(())
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn register_instance_abandon_vote_surface(
    state: &mut LuaState,
    table_ref: GcRef<Table>,
) -> LuaResult<()> {
    register_vote_information(state, table_ref)?;
    register_vote_responses(state, table_ref)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn register_vote_information(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetInstanceAbandonVoteRequirements",
        vote_requirements,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "IsChallengeModeKeystoneOwner",
        is_keystone_owner,
    )?;
    table_set_rust_fn_static(state, table_ref, "GetInstanceAbandonVoteTime", vote_time)?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetInstanceAbandonShutdownTime",
        shutdown_time,
    )?;
    Ok(())
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn register_vote_responses(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetInstanceAbandonVoteResponse",
        vote_response,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "SetInstanceAbandonVoteResponse",
        set_vote_response,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetNumInstanceAbandonGroupVoteResponses",
        group_vote_response_count,
    )
}

/// A_Admin.SetInstanceAbandonVoteState(voteDuration, voteTimeLeft,
/// shutdownDuration, shutdownTimeLeft, votesRequired, ownerVoteWeight,
/// isOwner, response, groupResponses).
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    builder
        .set_function(
            "SetInstanceAbandonVoteState",
            set_instance_abandon_vote_state,
        )?
        .set_function("SetPartyPolicyFlags", set_party_policy_flags)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn vote_requirements(state: &mut LuaState) -> LuaResult<u32> {
    let (votes_required, owner_vote_weight) = {
        let sim = borrow_state(state)?;
        (
            sim.instance_abandon_vote.votes_required,
            sim.instance_abandon_vote.keystone_owner_vote_weight,
        )
    };
    state.push(Val::Num(votes_required));
    state.push(Val::Num(owner_vote_weight));
    Ok(2)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn is_keystone_owner(state: &mut LuaState) -> LuaResult<u32> {
    let is_owner = borrow_state(state)?.instance_abandon_vote.is_keystone_owner;
    state.push(Val::Bool(is_owner));
    Ok(1)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn vote_time(state: &mut LuaState) -> LuaResult<u32> {
    let (duration, time_left) = {
        let sim = borrow_state(state)?;
        (
            sim.instance_abandon_vote.vote_duration,
            sim.instance_abandon_vote.vote_time_left,
        )
    };
    state.push(Val::Num(duration));
    state.push(Val::Num(time_left));
    Ok(2)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn shutdown_time(state: &mut LuaState) -> LuaResult<u32> {
    let (duration, time_left) = {
        let sim = borrow_state(state)?;
        (
            sim.instance_abandon_vote.shutdown_duration,
            sim.instance_abandon_vote.shutdown_time_left,
        )
    };
    state.push(Val::Num(duration));
    state.push(Val::Num(time_left));
    Ok(2)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn vote_response(state: &mut LuaState) -> LuaResult<u32> {
    let response = borrow_state(state)?.instance_abandon_vote.response;
    state.push(response.map_or(Val::Nil, Val::Bool));
    Ok(1)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn set_vote_response(state: &mut LuaState) -> LuaResult<u32> {
    let response = bool::from_stack(state, 1)?;
    crate::lua_api::methods::borrow_state_mut(state)?
        .instance_abandon_vote
        .response = Some(response);
    Ok(0)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn group_vote_response_count(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?
        .instance_abandon_vote
        .group_responses
        .len();
    state.push(Val::Num(count as f64));
    Ok(1)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn set_instance_abandon_vote_state(state: &mut LuaState) -> LuaResult<u32> {
    let vote_duration = f64::from_stack(state, 1)?;
    let vote_time_left = f64::from_stack(state, 2)?;
    let shutdown_duration = f64::from_stack(state, 3)?;
    let shutdown_time_left = f64::from_stack(state, 4)?;
    let votes_required = f64::from_stack(state, 5)?;
    let keystone_owner_vote_weight = f64::from_stack(state, 6)?;
    let is_keystone_owner = bool::from_stack(state, 7)?;
    let response = Option::<bool>::from_stack(state, 8)?;
    let group_responses = read_group_vote_responses(state)?;
    let mut sim = crate::lua_api::methods::borrow_state_mut(state)?;
    sim.instance_abandon_vote = InstanceAbandonVoteState {
        vote_duration,
        vote_time_left,
        shutdown_duration,
        shutdown_time_left,
        votes_required,
        keystone_owner_vote_weight,
        is_keystone_owner,
        response,
        group_responses,
    };
    Ok(0)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn read_group_vote_responses(state: &mut LuaState) -> LuaResult<Vec<bool>> {
    let Val::Table(table_ref) = Val::from_stack(state, 9)? else {
        return Err(runtime_error("group vote responses must be an array"));
    };
    let Some(table) = state.gc.tables.get(table_ref) else {
        return Err(runtime_error("group vote response table is unavailable"));
    };
    table
        .array_slice()
        .iter()
        .enumerate()
        .map(|(index, value)| match value {
            Val::Bool(response) => Ok(*response),
            _ => Err(runtime_error(format!(
                "group vote response {} must be a boolean",
                index + 1
            ))),
        })
        .collect()
}

fn mists_supplied_min_level(state: &mut LuaState) -> LuaResult<u32> {
    if state.top > state.base {
        return Err(rilua::runtime_error(
            "unsupported: C_PartyInfo.GetMinLevel explicit category",
        ));
    }
    let minimum = {
        let sim = crate::lua_api::methods::borrow_state(state)?;
        match sim.mists_party_min_level {
            Some(minimum) => minimum,
            None if sim.party_group_active => {
                // Mists grouped minimum: user-decided, not native evidence.
                sim.party_members
                    .iter()
                    .map(|member| member.level)
                    .fold(sim.player.level, i32::min) as f64
            }
            None => sim.player.level as f64,
        }
    };
    state.push(Val::Num(minimum));
    Ok(1)
}

/// Minimum of the supplied player and active roster; no separate party snapshot.
fn c_party_info_get_min_level(state: &mut LuaState) -> LuaResult<u32> {
    let minimum = {
        let sim = crate::lua_api::methods::borrow_state(state)?;
        if sim.party_group_active {
            sim.party_members
                .iter()
                .map(|member| member.level)
                .fold(sim.player.level, i32::min)
        } else {
            sim.player.level
        }
    };
    state.push(Val::Num(minimum as f64));
    Ok(1)
}

fn register_group_membership_probes(
    state: &mut LuaState,
    table_ref: GcRef<Table>,
) -> LuaResult<()> {
    #[cfg(feature = "client-retail")]
    table_set_rust_fn_static(
        state,
        table_ref,
        "IsCrossFactionParty",
        c_party_info_is_cross_faction_party,
    )?;
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

/// INFERRED (R88dc pending native evidence): unsupplied member factions match the player.
#[cfg(feature = "client-retail")]
fn c_party_info_is_cross_faction_party(state: &mut LuaState) -> LuaResult<u32> {
    let category = Option::<f64>::from_stack(state, 1)?.map(|value| value as i32);
    let cross_faction = if category == Some(2) {
        false
    } else {
        let sim = borrow_state(state)?;
        let player_faction = crate::lua_api::game_data::RACE_DATA
            .get(sim.player.race_index)
            .map(|(_, _, faction)| *faction)
            .filter(|faction| !faction.is_empty())
            .unwrap_or("Alliance");
        sim.party_group_active
            && sim.party_members.iter().any(|member| {
                member
                    .faction_group
                    .as_deref()
                    .is_some_and(|faction| faction != player_faction)
            })
    };
    state.push(Val::Bool(cross_faction));
    Ok(1)
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
    #[cfg(feature = "retail-12-0-7")]
    {
        roles_1207::is_guid_in_group(state)
    }
    #[cfg(not(feature = "retail-12-0-7"))]
    {
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
}

fn party_member_guid(index: usize) -> String {
    format!("Player-0000-000000{:02}", index + 2)
}

#[cfg(not(feature = "retail-12-0-7"))]
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
    #[cfg(feature = "retail-12-0-7")]
    {
        roles_1207::uninvite_unit(state)
    }
    #[cfg(not(feature = "retail-12-0-7"))]
    {
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
}

/// March 25 leadership restrictions were not loosened by March 31's
/// countdown/ready-check/ping/loot policy. INFERRED: the migrated C_PartyInfo
/// successors retain the legacy leadership restriction; denial raises an error.
/// This reads VM caller taint and live combat state, never a Lua permission flag.
fn reject_addon_leadership_combat(state: &LuaState) -> LuaResult<()> {
    let restricted_caller = cfg!(feature = "retail-12-0-5") && !rilua::api::state_is_secure(state);
    if restricted_caller
        && crate::lua_api::methods::borrow_state(state)?
            .player
            .in_combat
    {
        return Err(rilua::runtime_error(
            "C_PartyInfo: addon leadership changes are blocked during combat",
        ));
    }
    Ok(())
}

fn c_party_info_demote_assistant(state: &mut LuaState) -> LuaResult<u32> {
    reject_addon_leadership_combat(state)?;
    #[cfg(feature = "retail-12-0-7")]
    {
        roles_1207::demote_assistant(state)
    }
    #[cfg(not(feature = "retail-12-0-7"))]
    {
        let _unit = Option::<String>::from_stack(state, 1)?;
        borrow_state_mut(state)?.everyone_assistant = false;
        Ok(0)
    }
}

fn c_party_info_promote_to_assistant(state: &mut LuaState) -> LuaResult<u32> {
    reject_addon_leadership_combat(state)?;
    #[cfg(feature = "retail-12-0-7")]
    {
        roles_1207::promote_to_assistant(state)
    }
    #[cfg(not(feature = "retail-12-0-7"))]
    {
        let _unit = Option::<String>::from_stack(state, 1)?;
        borrow_state_mut(state)?.everyone_assistant = true;
        Ok(0)
    }
}

fn c_party_info_promote_to_leader(state: &mut LuaState) -> LuaResult<u32> {
    reject_addon_leadership_combat(state)?;
    #[cfg(feature = "retail-12-0-7")]
    {
        roles_1207::promote_to_leader(state)
    }
    #[cfg(not(feature = "retail-12-0-7"))]
    {
        let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
        let leader_index = party_member_index_from_unit(state, &unit)?;
        borrow_state_mut(state)?.party_leader_index = leader_index;
        Ok(0)
    }
}

fn c_party_info_set_everyone_is_assistant(state: &mut LuaState) -> LuaResult<u32> {
    reject_addon_leadership_combat(state)?;
    #[cfg(feature = "retail-12-0-7")]
    {
        roles_1207::set_everyone_is_assistant(state)
    }
    #[cfg(not(feature = "retail-12-0-7"))]
    {
        let enabled = Option::<bool>::from_stack(state, 1)?.unwrap_or(false);
        borrow_state_mut(state)?.everyone_assistant = enabled;
        Ok(0)
    }
}

fn c_party_info_do_ready_check(state: &mut LuaState) -> LuaResult<u32> {
    crate::lua_api::globals::group_verbs::start_ready_check(state)?;
    Ok(0)
}

fn c_party_info_confirm_ready_check(state: &mut LuaState) -> LuaResult<u32> {
    let is_ready = read_ready_check_response(state)?;
    crate::lua_api::globals::group_verbs::confirm_ready_check(state, is_ready)?;
    Ok(0)
}

/// Retail 12.0.7 `SecretArguments = AllowedWhenUntainted`, non-nilable `isReady`.
/// INFERRED: every supplied argument (extras included) is authenticated before
/// the boolean check and the chat-lockdown restriction.
#[cfg(feature = "retail-12-0-7")]
fn read_ready_check_response(state: &LuaState) -> LuaResult<bool> {
    let is_ready = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    for value in state.stack.iter().take(state.top).skip(state.base + 1) {
        rilua::table_security::unwrap_secret(state, *value)?;
    }
    match is_ready {
        Val::Bool(is_ready) => Ok(is_ready),
        _ => Err(runtime_error(
            "C_PartyInfo.ConfirmReadyCheck: isReady must be a boolean",
        )),
    }
}

#[cfg(not(feature = "retail-12-0-7"))]
fn read_ready_check_response(state: &LuaState) -> LuaResult<bool> {
    Ok(Option::<bool>::from_stack(state, 1)?.unwrap_or(false))
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
