//! State supplied by the simulator for addon restriction queries.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, table_get, table_set};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const INACTIVE: u8 = 0;
const ACTIVATING: u8 = 1;
const ACTIVE_STATE: u8 = 2;
const CHALLENGE_MODE: usize = 2;
const FORCED_CVARS: [&str; 6] = [
    "addonCombatRestrictionsForced",
    "addonEncounterRestrictionsForced",
    "addonChallengeModeRestrictionsForced",
    "addonPvPMatchRestrictionsForced",
    "addonMapRestrictionsForced",
    "addonChatRestrictionsForced",
];

pub(crate) fn supported() -> bool {
    matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr)
}

pub(crate) fn initial_states(cvars: &crate::cvars::CVarStorage) -> [u8; 6] {
    std::array::from_fn(|kind| {
        if supported() && cvars.get_bool(FORCED_CVARS[kind]) {
            ACTIVE_STATE
        } else {
            INACTIVE
        }
    })
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !supported() {
        return Ok(());
    }
    let enums = super::ensure_global_table(state, "Enum");
    let types = table_get(state, enums, "AddOnRestrictionType");
    table_set(state, types, "Chat", Val::Num(5.0));
    let metadata = table_get(state, enums, "AddOnRestrictionTypeMeta");
    table_set(state, metadata, "MaxValue", Val::Num(5.0));
    table_set(state, metadata, "NumValues", Val::Num(6.0));
    let namespace = super::ensure_namespace(state, "C_RestrictedActions")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetAddOnRestrictionState",
        restriction_state,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "IsAddOnRestrictionActive",
        restriction_active,
    )
}

fn restriction_type(state: &mut LuaState) -> LuaResult<usize> {
    let value = f64::from_stack(state, 1)?;
    if value.fract() != 0.0 || !(0.0..=5.0).contains(&value) {
        return Err(rilua::runtime_error(
            "expected an addon restriction type from 0 to 5",
        ));
    }
    Ok(value as usize)
}

fn restriction_state(state: &mut LuaState) -> LuaResult<u32> {
    let kind = restriction_type(state)?;
    let value = borrow_state(state)?.restricted_actions[kind];
    state.push(Val::Num(f64::from(value)));
    Ok(1)
}

fn restriction_active(state: &mut LuaState) -> LuaResult<u32> {
    let kind = restriction_type(state)?;
    let active = {
        let sim = borrow_state(state)?;
        // Documented: always false during ADDON_RESTRICTION_STATE_CHANGED dispatch.
        !sim.restricted_actions_dispatching && sim.restricted_actions[kind] == ACTIVE_STATE
    };
    state.push(Val::Bool(active));
    Ok(1)
}

pub(crate) fn set_supplied_restriction(state: &mut LuaState) -> LuaResult<u32> {
    let kind = restriction_type(state)?;
    if kind == 0 || kind == 5 {
        return Err(rilua::runtime_error(
            "restriction follows its supplied context",
        ));
    }
    let active = bool::from_stack(state, 2)?;
    set_restriction(state, kind, active)?;
    Ok(0)
}

/// Supply the native inputs whose server-side producers are not modeled.
pub(crate) fn set_supplied_context(state: &mut LuaState) -> LuaResult<u32> {
    let encounter_status = bool::from_stack(state, 1)?;
    let encounter_count = u32::from_stack(state, 2)?;
    let pvp_state = u32::from_stack(state, 3)?;
    let map_classification = u32::from_stack(state, 4)?;
    let map_flags = u32::from_stack(state, 5)?;
    let chat_client_flag = bool::from_stack(state, 6)?;
    let map_matches = map_classification != 0 && map_flags & 512 == 0;
    {
        let mut sim = borrow_state_mut(state)?;
        sim.restriction_chat_client_flag = chat_client_flag;
        sim.restriction_map_matches = map_matches;
    }
    set_restriction(state, 1, encounter_status || encounter_count > 0)?;
    set_restriction(state, 3, (1..=4).contains(&pvp_state))?;
    set_restriction(state, 4, map_matches)?;
    recompute_chat(state)?;
    Ok(0)
}

fn recompute_chat(state: &mut LuaState) -> LuaResult<()> {
    let chat = {
        let sim = borrow_state(state)?;
        sim.cvars.get_bool("addonChatRestrictionsEnabled")
            && sim.restriction_chat_client_flag
            && sim.restriction_map_matches
            && [1, 2, 3]
                .iter()
                .any(|&kind| sim.restricted_actions[kind] == ACTIVE_STATE)
    };
    borrow_state_mut(state)?.restricted_actions_natural[5] = chat;
    recompute_restriction(state, 5)
}

pub(crate) fn apply_host_event(state: &mut LuaState, event: &str) -> LuaResult<()> {
    let transition = match event {
        "ENCOUNTER_START" => Some((1, true)),
        "ENCOUNTER_END" => Some((1, false)),
        "PVP_MATCH_ACTIVE" => Some((3, true)),
        "PVP_MATCH_COMPLETE" | "PVP_MATCH_INACTIVE" => Some((3, false)),
        _ => None,
    };
    if let Some((kind, active)) = transition {
        set_restriction(state, kind, active)?;
    }
    Ok(())
}

pub(crate) fn set_challenge_mode(state: &mut LuaState, active: bool) -> LuaResult<()> {
    set_restriction(state, CHALLENGE_MODE, active)
}

pub(crate) fn set_restriction(state: &mut LuaState, kind: usize, active: bool) -> LuaResult<()> {
    if !supported() {
        return Ok(());
    }
    borrow_state_mut(state)?.restricted_actions_natural[kind] = active;
    recompute_restriction(state, kind)?;
    if (1..=3).contains(&kind) {
        recompute_chat(state)?;
    }
    Ok(())
}

pub(crate) fn update_forced_cvar(state: &mut LuaState, name: &str) -> LuaResult<()> {
    if !supported() {
        return Ok(());
    }
    if name.eq_ignore_ascii_case("addonChatRestrictionsEnabled") {
        return recompute_chat(state);
    }
    if let Some(kind) = FORCED_CVARS
        .iter()
        .position(|cvar| cvar.eq_ignore_ascii_case(name))
    {
        recompute_restriction(state, kind)?;
        if (1..=3).contains(&kind) {
            recompute_chat(state)?;
        }
    }
    Ok(())
}

fn recompute_restriction(state: &mut LuaState, kind: usize) -> LuaResult<()> {
    let (old, active) = {
        let sim = borrow_state(state)?;
        (
            sim.restricted_actions[kind],
            sim.restricted_actions_natural[kind] || sim.cvars.get_bool(FORCED_CVARS[kind]),
        )
    };
    if old == ACTIVATING || (old == ACTIVE_STATE) == active {
        return Ok(());
    }
    let next = if active { ACTIVATING } else { INACTIVE };
    let outer_dispatch = {
        let mut sim = borrow_state_mut(state)?;
        sim.restricted_actions[kind] = next;
        std::mem::replace(&mut sim.restricted_actions_dispatching, true)
    };
    let dispatched = dispatch_event_now(
        state,
        "ADDON_RESTRICTION_STATE_CHANGED",
        &[Val::Num(kind as f64), Val::Num(f64::from(next))],
    );
    borrow_state_mut(state)?.restricted_actions_dispatching = outer_dispatch;
    dispatched?;
    if active && borrow_state(state)?.restricted_actions[kind] == ACTIVATING {
        borrow_state_mut(state)?.restricted_actions[kind] = ACTIVE_STATE;
    }
    Ok(())
}
