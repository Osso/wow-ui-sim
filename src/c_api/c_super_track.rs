//! Supplied SuperTrack targets. No priority arbitration is inferred.

#[derive(Debug, Clone, Default)]
pub struct SuperTrackState {
    /// Optional public projection of the saved quest ID; its tracking flag is separate.
    pub quest_id: Option<i32>,
    pub quest_tracking: bool,
    /// Selection presence is separate from saved type/ID values and the tracking flag.
    pub content: Option<(i32, i32)>,
    /// Saved values remain unchanged when the content selection clears.
    pub content_saved: (i32, i32),
    pub content_tracking: bool,
    /// Selection presence is separate from saved type/ID values and the tracking flag.
    pub map_pin: Option<(i32, i32)>,
    /// Saved values remain unchanged when the map-pin selection clears.
    pub map_pin_saved: (i32, i32),
    pub map_pin_tracking: bool,
    pub vignette: Option<String>,
    /// User-waypoint tracking flag; the two native getterless flags stay unmodeled.
    pub user_waypoint: bool,
    /// INFERRED simulator default false; VERIFIED preserved when waypoint tracking starts.
    pub corpse: bool,
}

use super::helpers::{bad_argument, ensure_namespace, validate_supplied_relation_id};
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::closure::RustFn;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const METHODS: &[(&str, RustFn)] = &[
    ("GetSuperTrackedQuestID", quest_id),
    ("GetSuperTrackedContent", content),
    ("GetSuperTrackedMapPin", map_pin),
    ("GetSuperTrackedVignette", vignette),
    ("IsSuperTrackingAnything", anything),
    ("IsSuperTrackingQuest", is_quest),
    ("IsSuperTrackingContent", is_content),
    ("IsSuperTrackingMapPin", is_map_pin),
    ("IsSuperTrackingCorpse", corpse),
    ("SetSuperTrackedQuestID", set_quest),
    ("SetSuperTrackedContent", set_content),
    ("ClearSuperTrackedContent", clear_content),
    ("ClearSuperTrackedMapPin", clear_map_pin),
    ("ClearAllSuperTracked", clear_all),
    ("SetSuperTrackedUserWaypoint", set_user_waypoint),
    ("IsSuperTrackingUserWaypoint", is_user_waypoint),
];

const USER_WAYPOINT_SET_FUNCTION: &str = "C_SuperTrack.SetSuperTrackedUserWaypoint";
const USER_WAYPOINT_SET_USAGE: &str =
    "Usage: C_SuperTrack.SetSuperTrackedUserWaypoint(superTracked)";

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_SuperTrack")?;
    for &(name, function) in METHODS {
        table_set_rust_fn_static(state, namespace, name, function)?;
    }
    Ok(())
}

fn supplied_id(state: &LuaState, index: i32) -> LuaResult<i32> {
    validate_supplied_relation_id(stack_val(state, index), "SuperTrack ID must be a number")?
        .ok_or_else(|| rilua::runtime_error("SuperTrack ID must be a finite i32 integer"))
}

fn quest_id(state: &mut LuaState) -> LuaResult<u32> {
    let quest = borrow_state(state)?.super_track.quest_id;
    // Documented-optional representation, not established native sentinel parity.
    state.push(quest.map_or(Val::Nil, |id| Val::Num(f64::from(id))));
    Ok(1)
}

fn push_pair(state: &mut LuaState, pair: Option<(i32, i32)>) -> LuaResult<u32> {
    let Some((kind, id)) = pair else { return Ok(0) };
    state.push(Val::Num(f64::from(kind)));
    state.push(Val::Num(f64::from(id)));
    Ok(2)
}

fn content(state: &mut LuaState) -> LuaResult<u32> {
    let pair = {
        let sim = borrow_state(state)?;
        let tracked = &sim.super_track;
        if tracked.content_tracking {
            tracked.content
        } else {
            None
        }
    };
    push_pair(state, pair)
}

fn map_pin(state: &mut LuaState) -> LuaResult<u32> {
    let pair = {
        let sim = borrow_state(state)?;
        let tracked = &sim.super_track;
        if tracked.map_pin_tracking {
            tracked.map_pin
        } else {
            None
        }
    };
    push_pair(state, pair)
}

fn vignette(state: &mut LuaState) -> LuaResult<u32> {
    let guid = borrow_state(state)?.super_track.vignette.clone();
    let value = guid.map_or(Val::Nil, |guid| create_string(state, &guid));
    state.push(value);
    Ok(1)
}

fn is_quest(state: &mut LuaState) -> LuaResult<u32> {
    let is_tracking = borrow_state(state)?.super_track.quest_tracking;
    state.push(Val::Bool(is_tracking));
    Ok(1)
}

fn is_content(state: &mut LuaState) -> LuaResult<u32> {
    let is_tracking = borrow_state(state)?.super_track.content_tracking;
    state.push(Val::Bool(is_tracking));
    Ok(1)
}

fn is_map_pin(state: &mut LuaState) -> LuaResult<u32> {
    let is_tracking = borrow_state(state)?.super_track.map_pin_tracking;
    state.push(Val::Bool(is_tracking));
    Ok(1)
}

fn corpse(state: &mut LuaState) -> LuaResult<u32> {
    let is_tracking = borrow_state(state)?.super_track.corpse;
    state.push(Val::Bool(is_tracking));
    Ok(1)
}

/// Aggregate the six modeled flags. The two native getterless flags remain unmodeled
/// (R88j.5); agreement with selections from a consistent start is INFERRED (R88j.6).
fn anything(state: &mut LuaState) -> LuaResult<u32> {
    let active = {
        let sim = borrow_state(state)?;
        let tracked = &sim.super_track;
        [
            tracked.quest_tracking,
            tracked.content_tracking,
            tracked.map_pin_tracking,
            tracked.vignette.is_some(),
            tracked.user_waypoint,
            tracked.corpse,
        ]
        .into_iter()
        .any(|active| active)
    };
    state.push(Val::Bool(active));
    Ok(1)
}

/// Pre-existing simulator dispatch of SUPER_TRACKING_CHANGED; the native named-event
/// effect is UNKNOWN (R88j.5).
fn publish_change(state: &mut LuaState) -> LuaResult<u32> {
    dispatch_event_now(state, "SUPER_TRACKING_CHANGED", &[])?;
    Ok(0)
}

fn set_quest(state: &mut LuaState) -> LuaResult<u32> {
    // Pre-existing argument acceptance; native validation is UNKNOWN pending R88j.8.
    let id = supplied_id(state, 1)?;
    {
        let mut sim = borrow_state_mut(state)?;
        let tracked = &mut sim.super_track;
        let saved_id = tracked.quest_id.unwrap_or(0);
        if saved_id != id {
            tracked.quest_id = (id != 0).then_some(id);
            if id == 0 {
                tracked.quest_tracking = false;
            } else {
                tracked.quest_tracking = true;
                tracked.content_tracking = false;
                tracked.map_pin_tracking = false;
                tracked.content = None;
                tracked.map_pin = None;
                tracked.vignette = None;
            }
        }
    }
    publish_change(state)
}

fn set_content(state: &mut LuaState) -> LuaResult<u32> {
    let pair = (supplied_id(state, 1)?, supplied_id(state, 2)?);
    {
        let mut sim = borrow_state_mut(state)?;
        let tracked = &mut sim.super_track;
        tracked.content = Some(pair);
        tracked.content_saved = pair;
        tracked.content_tracking = true;
        tracked.quest_id = None;
        tracked.quest_tracking = false;
        tracked.map_pin_tracking = false;
        tracked.map_pin = None;
        tracked.vignette = None;
    }
    publish_change(state)
}

fn clear_content(state: &mut LuaState) -> LuaResult<u32> {
    {
        let mut sim = borrow_state_mut(state)?;
        sim.super_track.content_tracking = false;
        sim.super_track.content = None;
    }
    publish_change(state)
}

fn clear_map_pin(state: &mut LuaState) -> LuaResult<u32> {
    {
        let mut sim = borrow_state_mut(state)?;
        if sim.super_track.map_pin_tracking {
            sim.super_track.map_pin_tracking = false;
            sim.super_track.map_pin = None;
        }
    }
    publish_change(state)
}

fn clear_all(state: &mut LuaState) -> LuaResult<u32> {
    {
        let mut sim = borrow_state_mut(state)?;
        let content_saved = sim.super_track.content_saved;
        let map_pin_saved = sim.super_track.map_pin_saved;
        sim.super_track = SuperTrackState {
            content_saved,
            map_pin_saved,
            ..SuperTrackState::default()
        };
    }
    publish_change(state)
}

fn set_user_waypoint(state: &mut LuaState) -> LuaResult<u32> {
    let requested = read_user_waypoint_flag(state)?;
    let mut sim = borrow_state_mut(state)?;
    if sim.super_track.user_waypoint == requested {
        // UNKNOWN (R88i.3): the implementation uses the neutral zero-value seam.
        return Ok(0);
    }

    let waypoint_is_valid = sim
        .user_waypoint
        .is_some_and(|waypoint| waypoint.is_valid());
    let should_track = requested && waypoint_is_valid;
    if should_track {
        // VERIFIED: clear the named flags and selections; corpse tracking is preserved.
        // The additional getterless native flag remains unmodeled.
        sim.super_track.quest_id = None;
        sim.super_track.quest_tracking = false;
        sim.super_track.content_tracking = false;
        sim.super_track.map_pin_tracking = false;
        sim.super_track.content = None;
        sim.super_track.map_pin = None;
        sim.super_track.vignette = None;
    }
    sim.super_track.user_waypoint = should_track;
    // UNKNOWN (R88i.3): event dispatch is not proved; do not fire SUPER_TRACKING_CHANGED.
    Ok(0)
}

fn is_user_waypoint(state: &mut LuaState) -> LuaResult<u32> {
    let is_tracking = borrow_state(state)?.super_track.user_waypoint;
    state.push(Val::Bool(is_tracking));
    // UNKNOWN (R88i.3): the implementation uses one boolean as the neutral return seam.
    Ok(1)
}

fn read_user_waypoint_flag(state: &LuaState) -> LuaResult<bool> {
    let value = stack_val(state, 1);
    if rilua::table_security::is_secret_value(state, value)
        && !rilua::table_security::can_access_secrets(state)
    {
        return Err(user_waypoint_secret_argument_error());
    }
    let value = rilua::table_security::unwrap_secret(state, value)
        .map_err(|_| user_waypoint_secret_argument_error())?;
    if matches!(value, Val::Nil) {
        return Err(bad_argument(
            1,
            USER_WAYPOINT_SET_FUNCTION,
            USER_WAYPOINT_SET_USAGE,
        ));
    }
    Ok(!matches!(value, Val::Bool(false)))
}

fn user_waypoint_secret_argument_error() -> rilua::LuaError {
    bad_argument(
        1,
        USER_WAYPOINT_SET_FUNCTION,
        &format!(
            "{USER_WAYPOINT_SET_USAGE}. Secret values are only allowed during untainted execution for this argument."
        ),
    )
}
