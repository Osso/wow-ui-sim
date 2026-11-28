use super::super::real::unit_secret_predicates::{
    finish_identity_restricted, finish_possession_restricted,
};
use super::super::unit_api::{active_group_member_index, is_raid_group};
use super::{is_friendly_unit, visible_party_member};
use crate::lua_api::game_data::RACE_DATA;
use crate::lua_api::methods::{borrow_state, create_string, val_to_string};
use crate::lua_api::state::{SimState, SubgroupOwnerRoster};
use crate::lua_bridge::{FromStack, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// `UnitInParty(unit)` — true when the unit is a member of the player's
/// party (including the player).
pub(super) fn unit_in_party(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let in_party = {
        let st = borrow_state(state)?;
        let active = st.party_group_active;
        match unit.as_str() {
            "player" | "pet" | "vehicle" => active,
            other => visible_party_member(&st, other).is_some(),
        }
    };
    state.push(Val::Bool(in_party));
    Ok(1)
}

/// `UnitInRaid(unit)` — true when the unit belongs to the player's raid
/// group (sim treats party >= 6 as raid).
pub(super) fn unit_in_raid(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let in_raid = {
        let st = borrow_state(state)?;
        if !is_raid_group(&st) {
            false
        } else {
            matches!(unit.as_str(), "player" | "pet" | "vehicle")
                || visible_party_member(&st, &unit).is_some()
        }
    };
    state.push(Val::Bool(in_raid));
    finish_identity_restricted(state, &[&unit], 1)
}

/// VERIFIED R88bb Retail 70688 owner/slot behavior.
/// INFERRED (R88bb Retail 70688 semantics applied to Forever/Mists).
/// Applying these semantics to PTR/Era/Anniversary is also INFERRED from cache references.
pub(super) fn unit_in_subgroup(state: &mut LuaState) -> LuaResult<u32> {
    let unit = read_subgroup_unit(state)?;
    let owner_index = read_subgroup_index(state)?;
    let identity = {
        let sim = borrow_state(state)?;
        super::super::unit_misc::existing_guid_for_unit(&sim, &unit).unwrap_or_default()
    };
    let in_subgroup = {
        let sim = borrow_state(state)?;
        select_subgroup_owner(&sim, owner_index)
            .is_some_and(|roster| subgroup_contains(roster, &identity))
    };
    state.push(Val::Bool(in_subgroup));
    finish_identity_restricted(state, &[&unit], 1)
}

fn read_subgroup_unit(state: &LuaState) -> LuaResult<String> {
    match stack_val(state, 1) {
        Val::Nil => Ok(String::new()),
        Val::Str(_) => val_to_string(state, stack_val(state, 1)).ok_or_else(subgroup_usage_error),
        Val::Num(number) => Ok(number.to_string()),
        _ => Err(subgroup_usage_error()),
    }
}

fn read_subgroup_index(state: &LuaState) -> LuaResult<Option<i64>> {
    match stack_val(state, 2) {
        Val::Nil => Ok(None),
        Val::Num(number) if number.is_finite() && (0.0..=u32::MAX as f64).contains(&number) => {
            Ok(Some((number - 1.0).trunc() as i64))
        }
        _ => Err(subgroup_usage_error()),
    }
}

fn subgroup_usage_error() -> rilua::LuaError {
    rilua::runtime_error("Usage: UnitInSubgroup(unit [, partyIndex])")
}

fn select_subgroup_owner(sim: &SimState, index: Option<i64>) -> Option<&SubgroupOwnerRoster> {
    match index {
        None => sim
            .instance_subgroup_roster
            .as_ref()
            .or(sim.home_subgroup_roster.as_ref()),
        Some(0) => sim.home_subgroup_roster.as_ref(),
        Some(1) => sim.instance_subgroup_roster.as_ref(),
        _ => None,
    }
}

fn subgroup_contains(roster: &SubgroupOwnerRoster, identity: &str) -> bool {
    roster
        .members
        .iter()
        .filter(|member| member.subgroup == roster.player_subgroup)
        .take(4)
        .any(|member| member.identity.as_deref() == Some(identity))
}

pub(super) fn unit_player_or_pet_in_party(state: &mut LuaState) -> LuaResult<u32> {
    unit_in_party(state)
}

pub(super) fn unit_player_or_pet_in_raid(state: &mut LuaState) -> LuaResult<u32> {
    unit_in_raid(state)
}

pub(super) fn unit_targets_vehicle_in_raid_ui(state: &mut LuaState) -> LuaResult<u32> {
    let _ = Option::<String>::from_stack(state, 1)?;
    state.push(Val::Bool(false));
    Ok(1)
}

/// `UnitIsPossessed(unit)` / `UnitIsCharmed(unit)` — possession and charm are
/// not modeled in current SimState; only their secrecy is.
pub(super) fn unit_is_possessed(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    state.push(Val::Bool(false));
    finish_possession_restricted(state, &unit, 1)
}

/// `UnitIsOwnerOrControllerOfUnit(controllingUnit, controlledUnit)` — pet and
/// vehicle ownership is not modeled. INFERRED: secret when either unit's
/// identity is secret.
pub(super) fn unit_is_owner_or_controller_of_unit(state: &mut LuaState) -> LuaResult<u32> {
    let controlling = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let controlled = Option::<String>::from_stack(state, 2)?.unwrap_or_default();
    state.push(Val::Bool(false));
    finish_identity_restricted(state, &[&controlling, &controlled], 1)
}

/// `UnitIsPVP(unit)` reads supplied PvP state for Retail party and raid units.
pub(super) fn unit_is_pvp(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    #[cfg(feature = "client-retail")]
    let is_pvp = {
        let sim = borrow_state(state)?;
        visible_party_member(&sim, &unit).is_some_and(|member| member.pvp.unwrap_or(false))
    };
    #[cfg(not(feature = "client-retail"))]
    let is_pvp = false;
    state.push(Val::Bool(is_pvp));
    finish_identity_restricted(state, &[&unit], 1)
}

/// `UnitRealmRelationship(unit)` — all simulated unit tokens are same-realm.
pub(super) fn unit_realm_relationship(state: &mut LuaState) -> LuaResult<u32> {
    let _ = Option::<String>::from_stack(state, 1)?;
    state.push(Val::Num(0.0));
    Ok(1)
}

#[cfg(feature = "retail-12-0-0")]
pub(super) use super::super::real::party_ai::unit_in_party_is_ai;
#[cfg(not(feature = "retail-12-0-0"))]
pub(super) use crate::lua_api::workarounds::unit_in_party_is_ai;

/// `UnitIsPVPFreeForAll(unit)` — FFA PVP is not modeled.
pub(super) fn unit_is_pvp_free_for_all(state: &mut LuaState) -> LuaResult<u32> {
    let _ = Option::<String>::from_stack(state, 1)?;
    state.push(Val::Bool(false));
    Ok(1)
}

/// `UnitPhaseReason(unit)` — no phase/Chromie-time reason is modeled.
pub(super) fn unit_phase_reason(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    state.push(Val::Nil);
    finish_identity_restricted(state, &[&unit], 1)
}

/// `UnitInOtherParty(unit)` — sim does not model cross-party; always false.
pub(super) fn unit_in_other_party(state: &mut LuaState) -> LuaResult<u32> {
    let _ = Option::<String>::from_stack(state, 1)?;
    state.push(Val::Bool(false));
    Ok(1)
}

/// `UnitInRange(unit)` — true for `player`, `pet`, current `target` /
/// `focus`, and visible party tokens. Everything else defaults to false.
pub(super) fn unit_in_range(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let in_range = unit_is_in_range(state, &unit)?;
    state.push(Val::Bool(in_range));
    state.push(Val::Bool(!unit.is_empty()));
    Ok(2)
}

/// `CheckInteractDistance(unit, index)` — sim reuses the coarse in-range
/// model used by `UnitInRange`; distance buckets are not modeled separately.
pub(super) fn check_interact_distance(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let dist_index = i32::from_stack(state, 2)?;
    let valid_index = matches!(dist_index, 1..=4);
    let in_range = valid_index && unit_is_in_range(state, &unit)?;
    state.push(Val::Bool(in_range));
    Ok(1)
}

fn unit_is_in_range(state: &LuaState, unit: &str) -> LuaResult<bool> {
    let st = borrow_state(state)?;
    let in_range = match unit {
        "player" | "pet" | "vehicle" => true,
        "target" => st.current_target.is_some(),
        "focus" => st.current_focus.is_some(),
        other => visible_party_member(&st, other).is_some(),
    };
    Ok(in_range)
}

/// `UnitInBattleground(unit)` — true when the player is currently in an
/// arena / battleground instance. Sim routes this through the world
/// flags set by the battlefield verbs.
pub(super) fn unit_in_battleground(state: &mut LuaState) -> LuaResult<u32> {
    let _ = Option::<String>::from_stack(state, 1)?;
    let in_bg = {
        let st = borrow_state(state)?;
        st.world.battlefield_arena
            || matches!(st.world.instance_type.as_str(), "arena" | "pvp" | "bg")
    };
    state.push(Val::Bool(in_bg));
    Ok(1)
}

fn player_faction_name(st: &crate::lua_api::state::SimState) -> &'static str {
    RACE_DATA
        .get(st.player.race_index)
        .map(|(_, _, faction)| *faction)
        .filter(|faction| !faction.is_empty())
        .unwrap_or("Alliance")
}

fn opposing_faction_name(faction: &str) -> &'static str {
    if faction == "Horde" {
        "Alliance"
    } else {
        "Horde"
    }
}

fn target_faction<'a>(is_enemy: bool, player_faction: &'a str) -> &'a str {
    if is_enemy {
        opposing_faction_name(player_faction)
    } else {
        player_faction
    }
}

/// `UnitFactionGroup(unit)` returns `(english, localized)` faction tokens.
/// INFERRED (R88dl, native evidence pending): Retail party and raid units use
/// their supplied faction when present; otherwise they inherit the player's.
pub(super) fn unit_faction_group(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let faction = {
        let st = borrow_state(state)?;
        faction_group_for_unit(&st, &unit, player_faction_name(&st))
    };
    match faction {
        Some(name) => {
            let value = create_string(state, &name);
            state.push(value);
            state.push(value);
        }
        None => {
            state.push(Val::Nil);
            state.push(Val::Nil);
        }
    }
    Ok(2)
}

fn faction_group_for_unit(
    st: &crate::lua_api::state::SimState,
    unit: &str,
    player_faction: &str,
) -> Option<String> {
    match unit {
        "player" | "pet" | "vehicle" => Some(player_faction.to_owned()),
        "target" => st
            .current_target
            .as_ref()
            .map(|target| target_faction(target.is_enemy, player_faction).to_owned()),
        "focus" => st
            .current_focus
            .as_ref()
            .map(|target| target_faction(target.is_enemy, player_faction).to_owned()),
        #[cfg(feature = "client-retail")]
        other => visible_party_member(st, other).map(|member| {
            member
                .faction_group
                .clone()
                .unwrap_or_else(|| player_faction.to_owned())
        }),
        #[cfg(not(feature = "client-retail"))]
        other if visible_party_member(st, other).is_some() => Some(player_faction.to_owned()),
        #[cfg(not(feature = "client-retail"))]
        _ => None,
    }
}

/// `UnitCanCooperate(unit1, unit2)` — true when both units are friendly
/// (simple same-faction proxy for the sim).
pub(super) fn unit_can_cooperate(state: &mut LuaState) -> LuaResult<u32> {
    let unit1 = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let unit2 = Option::<String>::from_stack(state, 2)?.unwrap_or_default();
    let cooperate = is_friendly_unit(state, &unit1)? && is_friendly_unit(state, &unit2)?;
    state.push(Val::Bool(cooperate));
    Ok(1)
}

/// `UnitIsGroupLeader(unit)` — resolves unit to party index, then matches
/// against `SimState.party_leader_index`. Player is leader when the
/// index is `None`.
pub(super) fn unit_is_group_leader(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let leader = is_group_leader(&*borrow_state(state)?, &unit);
    state.push(Val::Bool(leader));
    finish_identity_restricted(state, &[&unit], 1)
}

pub(crate) fn is_group_leader(st: &crate::lua_api::state::SimState, unit: &str) -> bool {
    let active = st.party_group_active && !st.party_members.is_empty();
    if !active {
        false
    } else if matches!(unit, "player" | "pet" | "vehicle") {
        st.party_leader_index.is_none()
    } else if let Some(idx) = active_group_member_index(st, unit) {
        st.party_leader_index == Some(idx)
    } else {
        false
    }
}

/// `UnitIsGroupAssistant(unit)` — true only when raid-wide
/// `everyone_assistant` is set and the unit resolves to a party member
/// or the player.
pub(super) fn unit_is_group_assistant(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let assistant = is_group_assistant(&*borrow_state(state)?, &unit);
    state.push(Val::Bool(assistant));
    finish_identity_restricted(state, &[&unit], 1)
}

#[cfg(feature = "retail-12-0-7")]
pub(crate) fn is_group_assistant(st: &crate::lua_api::state::SimState, unit: &str) -> bool {
    crate::c_api::c_party_info::roles_1207::is_assistant(st, unit)
}

#[cfg(not(feature = "retail-12-0-7"))]
pub(crate) fn is_group_assistant(st: &crate::lua_api::state::SimState, unit: &str) -> bool {
    st.everyone_assistant
        && (matches!(unit, "player" | "pet" | "vehicle")
            || visible_party_member(st, unit).is_some())
}

/// `UnitGetAvailableRoles(unit)` — the player reads supplied role eligibility.
/// Other unsupplied queries preserve the existing all-eligible simulator model.
pub(super) fn unit_get_available_roles(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    #[cfg(feature = "player-requests")]
    let available = if unit == "player" {
        borrow_state(state)?
            .player_requests
            .roles
            .as_ref()
            .map_or([true; 3], |r| r.available)
    } else {
        [true; 3]
    };
    #[cfg(not(feature = "player-requests"))]
    let available = [true; 3];
    for role in available {
        state.push(Val::Bool(role));
    }
    finish_identity_restricted(state, &[&unit], 3)
}

/// `UnitIsRaidOfficer(unit)` — INFERRED: the raid leader or an assistant while
/// the group is a raid (party >= 6, matching `UnitInRaid`).
pub(super) fn unit_is_raid_officer(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let officer = {
        let st = borrow_state(state)?;
        is_raid_group(&st) && (is_group_leader(&st, &unit) || is_group_assistant(&st, &unit))
    };
    state.push(Val::Bool(officer));
    finish_identity_restricted(state, &[&unit], 1)
}

pub(super) fn unit_leads_any_group(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let leads = {
        let st = borrow_state(state)?;
        #[cfg(feature = "retail-12-0-7")]
        let assistant = crate::c_api::c_party_info::roles_1207::is_assistant(&st, &unit);
        #[cfg(not(feature = "retail-12-0-7"))]
        let assistant = st.everyone_assistant;
        if !st.party_group_active || st.party_members.is_empty() {
            false
        } else if matches!(unit.as_str(), "player" | "pet" | "vehicle") {
            st.party_leader_index.is_none() || assistant
        } else if let Some(idx) = active_group_member_index(&st, &unit) {
            st.party_leader_index == Some(idx) || assistant
        } else {
            false
        }
    };
    state.push(Val::Bool(leads));
    finish_identity_restricted(state, &[&unit], 1)
}

fn is_unit_dead(st: &crate::lua_api::state::SimState, unit: &str) -> bool {
    match unit {
        "player" | "pet" | "vehicle" => st.player.health <= 0,
        "target" => st.current_target.as_ref().is_some_and(|t| t.health <= 0),
        "focus" => st.current_focus.as_ref().is_some_and(|t| t.health <= 0),
        other => visible_party_member(st, other).is_some_and(|m| m.dead_since.is_some()),
    }
}

/// `UnitIsDeadOrGhost(unit)` — true when the unit is dead or in ghost form.
/// Sim treats health <= 0 as dead and has no separate ghost state.
pub(super) fn unit_is_dead_or_ghost(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let dead = {
        let st = borrow_state(state)?;
        is_unit_dead(&st, &unit)
    };
    state.push(Val::Bool(dead));
    Ok(1)
}

/// `UnitIsDead(unit)` — true when the unit is dead and not a ghost.
pub(super) fn unit_is_dead(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let dead = {
        let st = borrow_state(state)?;
        is_unit_dead(&st, &unit) && !(unit == "player" && player_is_ghost(&st))
    };
    state.push(Val::Bool(dead));
    Ok(1)
}

/// `UnitIsGhost(unit)` — only the player's released-spirit form is modeled.
pub(super) fn unit_is_ghost(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let ghost = unit == "player" && player_is_ghost(&*borrow_state(state)?);
    state.push(Val::Bool(ghost));
    Ok(1)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn player_is_ghost(st: &SimState) -> bool {
    st.death_release.is_ghost
}

#[cfg(not(any(feature = "client-retail", feature = "client-wowforever")))]
fn player_is_ghost(_st: &SimState) -> bool {
    false
}

/// `UnitIsConnected(unit)` — party tokens read modeled connection state;
/// other known unit tokens retain their existing presence-based behavior.
pub(crate) fn unit_is_connected_in_state(st: &SimState, unit: &str) -> bool {
    match unit {
        "player" | "pet" | "vehicle" => true,
        "target" => st.current_target.is_some(),
        "focus" => st.current_focus.is_some(),
        other if crate::lua_api::globals::unit_api::parse_party_index(other).is_some() => {
            visible_party_member(st, other).is_some_and(|member| member.connected)
        }
        other => visible_party_member(st, other).is_some(),
    }
}

pub(super) fn unit_is_connected(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let connected = {
        let sim = borrow_state(state)?;
        unit_is_connected_in_state(&sim, &unit)
    };
    state.push(Val::Bool(connected));
    Ok(1)
}

/// `UnitIsCorpse(unit)` — true when the unit is dead (same as DeadOrGhost
/// for the sim, which doesn't distinguish ghost runs from corpse runs).
pub(super) fn unit_is_corpse(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let corpse = {
        let st = borrow_state(state)?;
        is_unit_dead(&st, &unit)
    };
    state.push(Val::Bool(corpse));
    Ok(1)
}

/// `UnitIsCivilian(unit)` — civilian NPC flags are not modeled today.
pub(super) fn unit_is_civilian(state: &mut LuaState) -> LuaResult<u32> {
    let _ = Option::<String>::from_stack(state, 1)?;
    state.push(Val::Bool(false));
    Ok(1)
}

/// `UnitIsUnconscious(unit)` — unconscious state is specific to DK
/// start-zone / Monk Transcendence flavour retail mechanics that the
/// sim doesn't model; always false.
pub(super) fn unit_is_unconscious(state: &mut LuaState) -> LuaResult<u32> {
    let _ = Option::<String>::from_stack(state, 1)?;
    state.push(Val::Bool(false));
    Ok(1)
}

/// `UnitHasIncomingResurrection(unit)` — true for the player when
/// `pending_resurrect.is_some()`. Party members do not carry per-unit
/// incoming-resurrect state today; returns false.
pub(super) fn unit_has_incoming_resurrection(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let incoming = {
        let st = borrow_state(state)?;
        matches!(unit.as_str(), "player" | "pet" | "vehicle") && st.pending_resurrect.is_some()
    };
    state.push(Val::Bool(incoming));
    Ok(1)
}

/// `UnitIsVisible(unit)` — the sim has no fog-of-war / range-based
/// visibility, so any known unit is visible.
pub(super) fn unit_is_visible(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let visible = {
        let st = borrow_state(state)?;
        match unit.as_str() {
            "" => false,
            "player" | "pet" | "vehicle" => true,
            "target" => st.current_target.is_some(),
            "focus" => st.current_focus.is_some(),
            other => visible_party_member(&st, other).is_some(),
        }
    };
    state.push(Val::Bool(visible));
    Ok(1)
}
