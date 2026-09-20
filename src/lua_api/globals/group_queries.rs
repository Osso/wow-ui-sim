//! Group / unit-query globals backed by `SimState`.
//!
//! Mirrors `unit_api_extra.rs`'s mlua implementations so that
//! `A_Admin.SetPartySize(N)` actually makes `UnitExists("party1")`,
//! `IsInGroup()`, `GetNumGroupMembers()`, `UnitName("party1")`,
//! `UnitClass("party1")`, and `UnitLevel("party1")` report real
//! simulated state. Without these overrides the env_init.rs stubs keep
//! returning the player/default values, and PartyFrame never leaves its
//! XML placeholder path.
//!
//! Called once from `admin::register_all` after the A_Admin table
//! is installed, so the globals are in place before any addon script runs.
//!
//! Registering these here keeps the admin module focused on the
//! A_Admin RustFn table; group/unit queries have different semantics and
//! share no code with it.

#[path = "group_queries_relationships.rs"]
mod relationships;

use crate::lua_api::game_data::CLASS_LABELS;
use crate::lua_api::globals::security::mark_secret_value;
use crate::lua_api::methods::{
    borrow_state, create_string, create_string_static, create_table, table_get, table_set,
};
use crate::lua_api::state::PartyMember;
use crate::lua_bridge::FromStack;
use rilua::vm::closure::{Closure, RustClosure};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, RustFn, Val};

const QUEST_NPC_NAME: &str = "Quest Giver";
const LE_PARTY_CATEGORY_HOME: i32 = 1;
const LE_PARTY_CATEGORY_INSTANCE: i32 = 2;

pub fn register_all(state: &mut LuaState) {
    register_group_status(state);
    register_unit_queries(state);
    register_unit_relationships(state);
    register_unit_liveness(state);
    ensure_string_join(state);
}

fn register_group_status(state: &mut LuaState) {
    set_global(state, "GetNumGroupMembers", get_num_group_members);
    set_global(state, "GetNumSubgroupMembers", get_num_subgroup_members);
    set_global(state, "GetNumPartyMembers", get_num_subgroup_members);
    set_global(state, "GetNumRaidMembers", get_num_raid_members);
    set_global(state, "GetRaidRosterInfo", get_raid_roster_info);
    set_global(state, "IsInGroup", is_in_group);
    set_global(state, "IsInRaid", is_in_raid);
    set_global(state, "IsPartyLFG", is_party_lfg);
    set_global(state, "IsGroupLeader", is_group_leader);
    set_global(state, "IsEveryoneAssistant", is_everyone_assistant);
    set_global(state, "IsPartyWorldPVP", always_false);
    set_global(state, "GroupHasOfflineMember", always_false);
    set_global(state, "GetPartyAssignment", get_party_assignment);
}

fn register_unit_queries(state: &mut LuaState) {
    set_global(state, "UnitExists", unit_exists);
    set_global(state, "UnitName", unit_name);
    set_global(state, "UnitNameUnmodified", unit_name_unmodified);
    set_global(state, "UnitClass", unit_class);
    set_global(state, "UnitLevel", unit_level);
    set_global(state, "UnitEffectiveLevel", unit_level);
    set_global(state, "UnitClassification", unit_classification);
    set_global(state, "UnitCreatureType", unit_creature_type);
    set_global(
        state,
        "UnitTreatAsPlayerForDisplay",
        unit_treat_as_player_for_display,
    );
    set_global(state, "UnitSelectionColor", unit_selection_color);
    set_global(state, "UnitFactionGroup", relationships::unit_faction_group);
    set_global(state, "UnitInRange", relationships::unit_in_range);
    set_global(
        state,
        "CheckInteractDistance",
        relationships::check_interact_distance,
    );
    set_global(
        state,
        "UnitInBattleground",
        relationships::unit_in_battleground,
    );
}

fn register_unit_relationships(state: &mut LuaState) {
    table_set(
        state,
        Val::Table(state.global),
        "LE_REALM_RELATION_SAME",
        Val::Num(0.0),
    );
    state.gc.barrier_back(state.global);
    register_unit_reaction_relationships(state);
    register_unit_membership_relationships(state);
    register_unit_group_roles(state);
    set_global(state, "UnitIsPossessed", relationships::unit_is_possessed);
    set_global(
        state,
        "UnitRealmRelationship",
        relationships::unit_realm_relationship,
    );
    set_global(state, "UnitInPartyIsAI", relationships::unit_in_party_is_ai);
    set_global(
        state,
        "UnitIsPVPFreeForAll",
        relationships::unit_is_pvp_free_for_all,
    );
    set_global(state, "UnitPhaseReason", relationships::unit_phase_reason);
    set_global(state, "UnitHasLFGDeserter", always_false);
    set_global(
        state,
        "UnitHasLFGRandomCooldown",
        unit_has_lfg_random_cooldown,
    );
}

fn register_unit_reaction_relationships(state: &mut LuaState) {
    set_global(state, "UnitIsFriend", unit_is_friend);
    set_global(state, "UnitIsEnemy", unit_can_attack);
    set_global(state, "UnitCanAttack", unit_can_attack);
    set_global(state, "UnitCanAssist", unit_can_assist);
    set_global(state, "UnitCanCooperate", relationships::unit_can_cooperate);
    set_global(
        state,
        "UnitLeadsAnyGroup",
        relationships::unit_leads_any_group,
    );
}

fn register_unit_membership_relationships(state: &mut LuaState) {
    set_global(state, "UnitInParty", relationships::unit_in_party);
    set_global(state, "UnitInRaid", relationships::unit_in_raid);
    set_global(
        state,
        "UnitPlayerOrPetInParty",
        relationships::unit_player_or_pet_in_party,
    );
    set_global(
        state,
        "UnitPlayerOrPetInRaid",
        relationships::unit_player_or_pet_in_raid,
    );
    set_global(
        state,
        "UnitTargetsVehicleInRaidUI",
        relationships::unit_targets_vehicle_in_raid_ui,
    );
    set_global(
        state,
        "UnitInOtherParty",
        relationships::unit_in_other_party,
    );
}

fn register_unit_group_roles(state: &mut LuaState) {
    set_global(
        state,
        "UnitIsGroupLeader",
        relationships::unit_is_group_leader,
    );
    set_global(
        state,
        "UnitIsGroupAssistant",
        relationships::unit_is_group_assistant,
    );
}

fn register_unit_liveness(state: &mut LuaState) {
    set_global(state, "UnitIsDead", relationships::unit_is_dead);
    set_global(state, "UnitIsGhost", relationships::unit_is_ghost);
    set_global(
        state,
        "UnitIsDeadOrGhost",
        relationships::unit_is_dead_or_ghost,
    );
    set_global(state, "UnitIsConnected", relationships::unit_is_connected);
    set_global(state, "UnitIsCorpse", relationships::unit_is_corpse);
    set_global(state, "UnitIsCivilian", relationships::unit_is_civilian);
    set_global(
        state,
        "UnitIsUnconscious",
        relationships::unit_is_unconscious,
    );
    set_global(
        state,
        "UnitHasIncomingResurrection",
        relationships::unit_has_incoming_resurrection,
    );
    set_global(state, "UnitIsVisible", relationships::unit_is_visible);
}

fn set_global(state: &mut LuaState, name: &'static str, func: RustFn) {
    let global = state.global;
    let function = make_function(state, name, func);
    table_set(state, Val::Table(global), name, function);
    state.gc.barrier_back(global);
}

fn make_function(state: &mut LuaState, name: &'static str, func: RustFn) -> Val {
    let closure = Closure::Rust(RustClosure::new(func, name));
    Val::Function(state.gc.alloc_closure(closure))
}

fn ensure_global_table(state: &mut LuaState, name: &'static str) -> Val {
    let global = Val::Table(state.global);
    match table_get(state, global, name) {
        table @ Val::Table(_) => table,
        _ => {
            let table = create_table(state);
            table_set(state, global, name, table);
            table
        }
    }
}

fn ensure_string_join(state: &mut LuaState) {
    let string_table = ensure_global_table(state, "string");
    if matches!(table_get(state, string_table, "join"), Val::Function(_)) {
        return;
    }
    fn string_join(state: &mut LuaState) -> LuaResult<u32> {
        super::utility_system_spell::strjoin(state)
    }
    let join = make_function(state, "string.join", string_join);
    table_set(state, string_table, "join", join);
}

fn get_num_subgroup_members(state: &mut LuaState) -> LuaResult<u32> {
    let n = active_party_count_for_arg(state, 1)? as f64;
    state.push(Val::Num(n));
    Ok(1)
}

fn get_num_group_members(state: &mut LuaState) -> LuaResult<u32> {
    let party_count = active_party_count_for_arg(state, 1)?;
    // Match mlua master: party_count + 1 (counting the player) when in a
    // group, else 0.
    let n = if party_count > 0 { party_count + 1 } else { 0 };
    state.push(Val::Num(n as f64));
    Ok(1)
}

fn is_in_group(state: &mut LuaState) -> LuaResult<u32> {
    let in_group = active_party_count_for_arg(state, 1)? > 0;
    state.push(Val::Bool(in_group));
    Ok(1)
}

fn is_in_raid(state: &mut LuaState) -> LuaResult<u32> {
    // Sim currently models party only; treat a party ≥ 6 as a raid.
    let in_raid = active_party_count(state)? >= 6;
    state.push(Val::Bool(in_raid));
    Ok(1)
}

/// `GetNumRaidMembers()` — retail returns total raid size (player + others)
/// when in a raid, else 0. Follows the same ≥ 6-party raid threshold as
/// `IsInRaid`.
fn get_num_raid_members(state: &mut LuaState) -> LuaResult<u32> {
    let party_count = active_party_count(state)?;
    let n = if party_count >= 6 { party_count + 1 } else { 0 };
    state.push(Val::Num(n as f64));
    Ok(1)
}

fn get_raid_roster_info(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?.max(0) as usize;
    let Some(member) = raid_roster_member(state, index)? else {
        push_empty_raid_roster_info(state);
        return Ok(12);
    };

    push_raid_roster_info(state, index, &member);
    Ok(12)
}

fn raid_roster_member(state: &LuaState, index: usize) -> LuaResult<Option<PartyMember>> {
    let st = borrow_state(state)?;
    let roster_count = st.party_members.len() + 1;
    if !st.party_group_active || index == 0 || index > roster_count {
        return Ok(None);
    }
    let member = if index == 1 {
        Some(player_as_raid_member(&st))
    } else {
        st.party_members.get(index - 2).cloned()
    };
    Ok(member)
}

fn player_as_raid_member(st: &crate::lua_api::state::SimState) -> PartyMember {
    PartyMember {
        name: st.player.name.clone(),
        class_index: st.player.class_index,
        level: st.player.level,
        health: st.player.health,
        health_max: st.player.health_max,
        power: st.player.power,
        power_max: st.player.power_max,
        power_type: st.player.power_type,
        power_type_name: String::new(),
        is_leader: true,
        dead_since: None,
        buffs: Vec::new(),
        debuffs: Vec::new(),
    }
}

fn push_empty_raid_roster_info(state: &mut LuaState) {
    for _ in 0..12 {
        state.push(Val::Nil);
    }
}

fn push_raid_roster_info(state: &mut LuaState, index: usize, member: &PartyMember) {
    let name = create_string(state, &member.name);
    mark_secret_value(state, name);
    let rank = if member.is_leader { 2.0 } else { 0.0 };
    let subgroup = ((index - 1) / 5 + 1) as f64;
    let (class_name, class_file, _) = class_info(member.class_index);
    let class_name = create_string_static(state, class_name);
    let class_file = create_string_static(state, class_file);
    let zone = create_string_static(state, "");
    let assigned_role = create_string_static(state, "NONE");

    state.push(name);
    state.push(Val::Num(rank));
    state.push(Val::Num(subgroup));
    state.push(Val::Num(member.level as f64));
    state.push(class_name);
    state.push(class_file);
    state.push(zone);
    state.push(Val::Bool(true));
    state.push(Val::Bool(member.dead_since.is_some()));
    state.push(Val::Nil);
    state.push(Val::Nil);
    state.push(assigned_role);
}

fn get_party_assignment(state: &mut LuaState) -> LuaResult<u32> {
    let _assignment = Option::<String>::from_stack(state, 1)?;
    let _unit = Option::<String>::from_stack(state, 2)?;
    state.push(Val::Bool(false));
    Ok(1)
}

fn is_party_lfg(state: &mut LuaState) -> LuaResult<u32> {
    let lfg = borrow_state(state)?.is_party_lfg;
    state.push(Val::Bool(lfg));
    Ok(1)
}

/// `IsGroupLeader()` — true when the player is in a group AND leads it.
/// `SimState.party_leader_index = None` means the player is the leader.
fn is_group_leader(state: &mut LuaState) -> LuaResult<u32> {
    let in_group = active_party_count(state)? > 0;
    let leader = in_group && borrow_state(state)?.party_leader_index.is_none();
    state.push(Val::Bool(leader));
    Ok(1)
}

fn is_everyone_assistant(state: &mut LuaState) -> LuaResult<u32> {
    let flag = borrow_state(state)?.everyone_assistant;
    state.push(Val::Bool(flag));
    Ok(1)
}

pub(crate) fn active_party_count(state: &mut LuaState) -> LuaResult<usize> {
    let st = borrow_state(state)?;
    Ok(if st.party_group_active {
        st.party_members.len()
    } else {
        0
    })
}

fn active_party_count_for_arg(state: &mut LuaState, index: i32) -> LuaResult<usize> {
    if party_category_arg(state, index)? == Some(LE_PARTY_CATEGORY_INSTANCE) {
        return Ok(0);
    }

    active_party_count(state)
}

fn party_category_arg(state: &mut LuaState, index: i32) -> LuaResult<Option<i32>> {
    let category = Option::<f64>::from_stack(state, index)?;
    Ok(category
        .map(|value| value as i32)
        .filter(|value| matches!(*value, LE_PARTY_CATEGORY_HOME | LE_PARTY_CATEGORY_INSTANCE)))
}

fn unit_has_lfg_random_cooldown(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let has_cooldown = borrow_state(state)?
        .lfg_random_cooldown_units
        .iter()
        .any(|cooldown_unit| cooldown_unit == unit.as_str());
    state.push(Val::Bool(has_cooldown));
    Ok(1)
}

fn unit_exists(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let exists = unit_exists_in_state(&*borrow_state(state)?, &unit);
    state.push(Val::Bool(exists));
    Ok(1)
}

pub(crate) fn unit_exists_in_state(st: &crate::lua_api::state::SimState, unit: &str) -> bool {
    match unit {
        "" => false,
        "player" | "vehicle" | "pet" => true,
        "questnpc" => st.gossip.active,
        "softinteract" => super::targeting_verbs::resolve_unit_snapshot(st, unit)
            .is_some_and(|unit| !unit.interaction.is_game_object),
        "target" => st.current_target.is_some(),
        "focus" => st.current_focus.is_some(),
        other => {
            if let Some(idx) = crate::lua_api::globals::unit_api::parse_party_index(other) {
                st.party_group_active && idx < st.party_members.len()
            } else if let Some(rest) = other.strip_prefix("raid") {
                st.party_group_active
                    && rest
                        .parse::<usize>()
                        .ok()
                        .is_some_and(|n| n > 0 && n - 1 < st.party_members.len())
            } else {
                false
            }
        }
    }
}

fn unit_name(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let name = unit_name_for(state, &unit)?;
    let value = create_string(state, &name);
    if unit_identity_is_secret(&unit, &name) {
        mark_secret_value(state, value);
    }
    state.push(value);
    Ok(1)
}

fn unit_name_unmodified(state: &mut LuaState) -> LuaResult<u32> {
    unit_name(state)
}

fn unit_identity_is_secret(unit: &str, name: &str) -> bool {
    name != "Unknown"
        && (crate::lua_api::globals::unit_api::parse_party_index(unit).is_some()
            || unit.strip_prefix("raid").is_some())
}

fn unit_name_for(state: &mut LuaState, unit: &str) -> LuaResult<String> {
    let name = {
        let st = borrow_state(state)?;
        match unit {
            "player" | "pet" | "vehicle" => st.player.name.clone(),
            "questnpc" => {
                if st.gossip.active {
                    QUEST_NPC_NAME.to_string()
                } else {
                    "Unknown".to_string()
                }
            }
            "target" => st
                .current_target
                .as_ref()
                .map(|target| target.name.clone())
                .unwrap_or_else(|| "Unknown".to_string()),
            "focus" => st
                .current_focus
                .as_ref()
                .map(|target| target.name.clone())
                .unwrap_or_else(|| "Unknown".to_string()),
            other => visible_party_member(&st, other)
                .map(|member| member.name.clone())
                .unwrap_or_else(|| "Unknown".to_string()),
        }
    };
    Ok(name)
}

fn unit_level(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let level = {
        let st = borrow_state(state)?;
        match unit.as_str() {
            "player" | "pet" | "vehicle" => st.player.level,
            "target" => st
                .current_target
                .as_ref()
                .map(|target| target.level)
                .unwrap_or(1),
            "focus" => st
                .current_focus
                .as_ref()
                .map(|target| target.level)
                .unwrap_or(1),
            other => visible_party_member(&st, other)
                .map(|member| member.level)
                .unwrap_or(1),
        }
    };
    state.push(Val::Num(level as f64));
    Ok(1)
}

fn unit_class(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let class_index = {
        let st = borrow_state(state)?;
        match unit.as_str() {
            "player" | "pet" | "vehicle" => st.player.class_index,
            "target" => st
                .current_target
                .as_ref()
                .map(|target| target.class_index)
                .unwrap_or(2),
            "focus" => st
                .current_focus
                .as_ref()
                .map(|target| target.class_index)
                .unwrap_or(2),
            other => visible_party_member(&st, other)
                .map(|member| member.class_index)
                .unwrap_or(2),
        }
    };
    let (class_name, class_file, class_id) = class_info(class_index);
    let class_name_val = create_string_static(state, class_name);
    let class_file_val = create_string_static(state, class_file);
    state.push(class_name_val);
    state.push(class_file_val);
    state.push(Val::Num(class_id as f64));
    Ok(3)
}

fn visible_party_member<'a>(
    st: &'a crate::lua_api::state::SimState,
    unit: &str,
) -> Option<&'a crate::lua_api::state::PartyMember> {
    if !st.party_group_active {
        return None;
    }
    if let Some(idx) = crate::lua_api::globals::unit_api::parse_party_index(unit) {
        if idx < st.party_members.len() {
            st.party_members.get(idx)
        } else {
            None
        }
    } else if let Some(rest) = unit.strip_prefix("raid") {
        rest.parse::<usize>()
            .ok()
            .and_then(|n| n.checked_sub(1))
            .and_then(|idx| st.party_members.get(idx))
    } else {
        None
    }
}

fn unit_classification(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let classification = {
        let st = borrow_state(state)?;
        match unit.as_str() {
            "target" => st
                .current_target
                .as_ref()
                .map(|target| target.classification.clone())
                .unwrap_or_else(|| "normal".to_string()),
            "focus" => st
                .current_focus
                .as_ref()
                .map(|target| target.classification.clone())
                .unwrap_or_else(|| "normal".to_string()),
            _ => "normal".to_string(),
        }
    };
    let value = create_string(state, &classification);
    state.push(value);
    Ok(1)
}

fn unit_creature_type(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let creature_type = {
        let st = borrow_state(state)?;
        match unit.as_str() {
            "target" => st
                .current_target
                .as_ref()
                .map(|target| target.creature_type.clone())
                .unwrap_or_else(|| "Humanoid".to_string()),
            "focus" => st
                .current_focus
                .as_ref()
                .map(|target| target.creature_type.clone())
                .unwrap_or_else(|| "Humanoid".to_string()),
            _ => "Humanoid".to_string(),
        }
    };
    let value = create_string(state, &creature_type);
    state.push(value);
    Ok(1)
}

fn unit_treat_as_player_for_display(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let treat_as_player = {
        let st = borrow_state(state)?;
        match unit.as_str() {
            "player" | "pet" | "vehicle" => true,
            "target" => st
                .current_target
                .as_ref()
                .map(|target| target.is_player)
                .unwrap_or(false),
            "focus" => st
                .current_focus
                .as_ref()
                .map(|target| target.is_player)
                .unwrap_or(false),
            other => visible_party_member(&st, other).is_some(),
        }
    };
    state.push(Val::Bool(treat_as_player));
    Ok(1)
}

fn unit_is_friend(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 2)?.unwrap_or_default();
    let is_friend = is_friendly_unit(state, &unit)?;
    state.push(Val::Bool(is_friend));
    Ok(1)
}

fn unit_can_attack(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 2)?.unwrap_or_default();
    let can_attack = is_attackable_unit(state, &unit)?;
    state.push(Val::Bool(can_attack));
    Ok(1)
}

fn unit_can_assist(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 2)?.unwrap_or_default();
    let can_assist = is_friendly_unit(state, &unit)?;
    state.push(Val::Bool(can_assist));
    Ok(1)
}

fn unit_selection_color(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let (r, g, b) = if is_attackable_unit(state, &unit)? {
        (1.0, 0.0, 0.0)
    } else if is_friendly_unit(state, &unit)? {
        (0.0, 1.0, 0.0)
    } else {
        (1.0, 1.0, 0.0)
    };
    state.push(Val::Num(r));
    state.push(Val::Num(g));
    state.push(Val::Num(b));
    Ok(3)
}

fn is_friendly_unit(state: &mut LuaState, unit: &str) -> LuaResult<bool> {
    let st = borrow_state(state)?;
    Ok(match unit {
        "player" | "pet" | "vehicle" => true,
        "target" => st
            .current_target
            .as_ref()
            .map(|target| !target.is_enemy || target.reaction >= 4)
            .unwrap_or(true),
        "focus" => st
            .current_focus
            .as_ref()
            .map(|target| !target.is_enemy || target.reaction >= 4)
            .unwrap_or(true),
        other => visible_party_member(&st, other).is_some(),
    })
}

fn is_attackable_unit(state: &mut LuaState, unit: &str) -> LuaResult<bool> {
    let st = borrow_state(state)?;
    Ok(match unit {
        "target" => st
            .current_target
            .as_ref()
            .map(|target| target.is_enemy || target.reaction < 4)
            .unwrap_or(false),
        "focus" => st
            .current_focus
            .as_ref()
            .map(|target| target.is_enemy || target.reaction < 4)
            .unwrap_or(false),
        _ => false,
    })
}

fn class_info(class_index: i32) -> (&'static str, &'static str, i32) {
    const CLASS_FILES: &[&str] = &[
        "WARRIOR",
        "PALADIN",
        "HUNTER",
        "ROGUE",
        "PRIEST",
        "DEATHKNIGHT",
        "SHAMAN",
        "MAGE",
        "WARLOCK",
        "MONK",
        "DRUID",
        "DEMONHUNTER",
        "EVOKER",
    ];

    let idx = class_index
        .max(1)
        .min(CLASS_LABELS.len() as i32)
        .saturating_sub(1) as usize;
    (CLASS_LABELS[idx], CLASS_FILES[idx], class_index.max(1))
}

fn always_false(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}
