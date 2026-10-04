use super::{is_friendly_unit, visible_party_member};
use crate::lua_api::game_data::RACE_DATA;
use crate::lua_api::methods::{borrow_state, create_string};
use crate::lua_bridge::FromStack;
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
        if !st.party_group_active || st.party_members.len() < 6 {
            false
        } else {
            matches!(unit.as_str(), "player" | "pet" | "vehicle")
                || visible_party_member(&st, &unit).is_some()
        }
    };
    state.push(Val::Bool(in_raid));
    Ok(1)
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

/// `UnitIsPossessed(unit)` — possession is not modeled in current SimState.
pub(super) fn unit_is_possessed(state: &mut LuaState) -> LuaResult<u32> {
    let _ = Option::<String>::from_stack(state, 1)?;
    state.push(Val::Bool(false));
    Ok(1)
}

/// `UnitRealmRelationship(unit)` — all simulated unit tokens are same-realm.
pub(super) fn unit_realm_relationship(state: &mut LuaState) -> LuaResult<u32> {
    let _ = Option::<String>::from_stack(state, 1)?;
    state.push(Val::Num(0.0));
    Ok(1)
}

/// `UnitInPartyIsAI(unit)` — follower/AI party members are not modeled.
pub(super) fn unit_in_party_is_ai(state: &mut LuaState) -> LuaResult<u32> {
    let _ = Option::<String>::from_stack(state, 1)?;
    state.push(Val::Bool(false));
    Ok(1)
}

/// `UnitIsPVPFreeForAll(unit)` — FFA PVP is not modeled.
pub(super) fn unit_is_pvp_free_for_all(state: &mut LuaState) -> LuaResult<u32> {
    let _ = Option::<String>::from_stack(state, 1)?;
    state.push(Val::Bool(false));
    Ok(1)
}

/// `UnitPhaseReason(unit)` — no phase/Chromie-time reason is modeled.
pub(super) fn unit_phase_reason(state: &mut LuaState) -> LuaResult<u32> {
    let _ = Option::<String>::from_stack(state, 1)?;
    state.push(Val::Nil);
    Ok(1)
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

/// `UnitFactionGroup(unit)` — returns `(english, localized)` faction tokens
/// for simple UI gating. Player / party / friendly target inherit the player's
/// faction; hostile targets return the opposing faction.
pub(super) fn unit_faction_group(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let faction = {
        let st = borrow_state(state)?;
        let pf = player_faction_name(&st);
        match unit.as_str() {
            "player" | "pet" | "vehicle" => Some(pf),
            "target" => st
                .current_target
                .as_ref()
                .map(|t| target_faction(t.is_enemy, pf)),
            "focus" => st
                .current_focus
                .as_ref()
                .map(|t| target_faction(t.is_enemy, pf)),
            other if visible_party_member(&st, other).is_some() => Some(pf),
            _ => None,
        }
    };
    match faction {
        Some(name) => {
            let s = create_string(state, name);
            state.push(s);
            state.push(s);
        }
        None => {
            state.push(Val::Nil);
            state.push(Val::Nil);
        }
    }
    Ok(2)
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
    let leader = {
        let st = borrow_state(state)?;
        let player_tokens = matches!(unit.as_str(), "player" | "pet" | "vehicle");
        let active = st.party_group_active && !st.party_members.is_empty();
        if !active {
            false
        } else if player_tokens {
            st.party_leader_index.is_none()
        } else if let Some(idx) = resolve_unit_party_index(&st, &unit) {
            st.party_leader_index == Some(idx)
        } else {
            false
        }
    };
    state.push(Val::Bool(leader));
    Ok(1)
}

/// `UnitIsGroupAssistant(unit)` — true only when raid-wide
/// `everyone_assistant` is set and the unit resolves to a party member
/// or the player.
pub(super) fn unit_is_group_assistant(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    #[cfg(feature = "retail-12-0-7")]
    let assistant = {
        let sim = borrow_state(state)?;
        crate::c_api::c_party_info::roles_1207::is_assistant(&sim, &unit)
    };
    #[cfg(not(feature = "retail-12-0-7"))]
    let assistant = {
        let st = borrow_state(state)?;
        if !st.everyone_assistant {
            false
        } else {
            matches!(unit.as_str(), "player" | "pet" | "vehicle")
                || visible_party_member(&st, &unit).is_some()
        }
    };
    state.push(Val::Bool(assistant));
    Ok(1)
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
        } else if let Some(idx) = resolve_unit_party_index(&st, &unit) {
            st.party_leader_index == Some(idx) || assistant
        } else {
            false
        }
    };
    state.push(Val::Bool(leads));
    Ok(1)
}

fn resolve_unit_party_index(st: &crate::lua_api::state::SimState, unit: &str) -> Option<usize> {
    let party_len = st.party_members.len();
    if let Some(idx) = crate::lua_api::globals::unit_api::parse_party_index(unit)
        && idx < party_len
    {
        return Some(idx);
    }
    unit.strip_prefix("raid")
        .and_then(|s| s.parse::<usize>().ok())
        .and_then(|n| n.checked_sub(1))
        .filter(|&n| n < party_len)
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

/// `UnitIsDead(unit)` — true when the unit is dead.
pub(super) fn unit_is_dead(state: &mut LuaState) -> LuaResult<u32> {
    unit_is_dead_or_ghost(state)
}

/// `UnitIsGhost(unit)` — the sim has no separate ghost state today.
pub(super) fn unit_is_ghost(state: &mut LuaState) -> LuaResult<u32> {
    let _ = Option::<String>::from_stack(state, 1)?;
    state.push(Val::Bool(false));
    Ok(1)
}

/// `UnitIsConnected(unit)` — party tokens read modeled connection state;
/// other known unit tokens retain their existing presence-based behavior.
pub(super) fn unit_is_connected(state: &mut LuaState) -> LuaResult<u32> {
    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();
    let connected = {
        let st = borrow_state(state)?;
        match unit.as_str() {
            "player" | "pet" | "vehicle" => true,
            "target" => st.current_target.is_some(),
            "focus" => st.current_focus.is_some(),
            other if crate::lua_api::globals::unit_api::parse_party_index(other).is_some() => {
                visible_party_member(&st, other).is_some_and(|member| member.connected)
            }
            other => visible_party_member(&st, other).is_some(),
        }
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
