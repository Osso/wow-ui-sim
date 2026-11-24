//! Supplied navigation target state for Retail and WoW Forever.
//!
//! API declarations: Retail and Forever `InGameNavigationDocumentation.lua`.
//! Behaviour is INFERRED, R88ff (native evidence pending).

use crate::c_api::c_super_track::NavigationTarget;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
#[cfg(feature = "retail-12-0-5")]
use crate::lua_api::methods::create_string;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, frame_ref, table_get};
use crate::lua_bridge::{TableBuilder, stack_val, table_set_rust_fn_static};
use crate::widget::AnchorPoint;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val, runtime_error};

const INVALID: i32 = 0;
const MIN_VALID_SCREEN_COORDINATE: f64 = 1.0 / 4_194_304.0;

/// The supplied-target model applies to Retail and Forever only; PTR keeps its
/// prior registrations (nearest-party token in Rust, the rest from the Lua shim).
fn models_navigation_targets() -> bool {
    matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever)
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_Navigation")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetNearestPartyMemberToken",
        get_nearest_party_member_token,
    )?;
    if models_navigation_targets() {
        register_target_queries(state, namespace)?;
    }
    Ok(())
}

fn register_target_queries(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetDistance", get_distance)?;
    table_set_rust_fn_static(state, namespace, "GetFrame", get_frame)?;
    table_set_rust_fn_static(state, namespace, "GetTargetState", get_target_state)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "HasValidScreenPosition",
        has_valid_screen_position,
    )?;
    table_set_rust_fn_static(state, namespace, "WasClampedToScreen", was_clamped)?;
    #[cfg(any(feature = "retail-12-1-0", feature = "client-wowforever"))]
    table_set_rust_fn_static(
        state,
        namespace,
        "GetNextWaypointForMap",
        get_next_waypoint_for_map,
    )?;
    Ok(())
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    if !models_navigation_targets() {
        return Ok(builder);
    }
    builder.set_function("SetNavigationTarget", set_navigation_target)
}

pub(crate) fn update_navigation_frame(state: &mut LuaState) -> LuaResult<()> {
    if !models_navigation_targets() {
        return Ok(());
    }
    let available = navigation_target(state)?.is_some();
    let changed = {
        let mut sim = borrow_state_mut(state)?;
        if sim.navigation_frame_active == available {
            false
        } else {
            sim.navigation_frame_active = available;
            true
        }
    };
    if changed {
        publish_navigation_lifecycle(state, available)?;
    }
    Ok(())
}

fn navigation_target(state: &LuaState) -> LuaResult<Option<NavigationTarget>> {
    let sim = borrow_state(state)?;
    let tracked = &sim.super_track;
    let is_tracking = tracked.quest_tracking
        || tracked.content_tracking
        || tracked.map_pin_tracking
        || if ACTIVE == ClientProfile::Retail {
            tracked.vignette_tracking
        } else {
            tracked.vignette.is_some()
        }
        || tracked.user_waypoint
        || tracked.corpse;
    Ok(is_tracking.then_some(tracked.navigation_target).flatten())
}

fn publish_navigation_lifecycle(state: &mut LuaState, created: bool) -> LuaResult<()> {
    if created {
        let id = borrow_state(state)?
            .widgets
            .get_id_by_name("NavigationFrame")
            .ok_or_else(|| runtime_error("C_Navigation requires the engine NavigationFrame"))?;
        let frame = frame_ref(state, id)?;
        dispatch_event_now(state, "NAVIGATION_FRAME_CREATED", &[frame])?;
    } else {
        dispatch_event_now(state, "NAVIGATION_FRAME_DESTROYED", &[])?;
    }
    Ok(())
}

fn get_distance(state: &mut LuaState) -> LuaResult<u32> {
    // R88ff VERIFIED: the native getter publishes one float; scenario values are inferred.
    let distance = navigation_target(state)?.map_or(0.0, |target| target.distance);
    state.push(Val::Num(distance));
    Ok(1)
}

fn get_target_state(state: &mut LuaState) -> LuaResult<u32> {
    // R88ff VERIFIED: the native getter publishes one integer; state selection is inferred.
    let value = navigation_target(state)?.map_or(INVALID, |target| target.state);
    state.push(Val::Num(f64::from(value)));
    Ok(1)
}

fn has_valid_screen_position(state: &mut LuaState) -> LuaResult<u32> {
    // R88ff VERIFIED: either stored coordinate may establish a valid position.
    let valid = navigation_target(state)?.is_some_and(|target| {
        target.screen_x.abs() >= MIN_VALID_SCREEN_COORDINATE
            || target.screen_y.abs() >= MIN_VALID_SCREEN_COORDINATE
    });
    state.push(Val::Bool(valid));
    Ok(1)
}

fn was_clamped(state: &mut LuaState) -> LuaResult<u32> {
    // R88ff VERIFIED: expose the stored clamped Boolean.
    let clamped = navigation_target(state)?.is_some_and(|target| target.clamped);
    state.push(Val::Bool(clamped));
    Ok(1)
}

fn get_frame(state: &mut LuaState) -> LuaResult<u32> {
    let active = borrow_state(state)?.navigation_frame_active;
    if !active {
        state.push(Val::Nil);
        return Ok(1);
    }
    let id = borrow_state(state)?
        .widgets
        .get_id_by_name("NavigationFrame")
        .ok_or_else(|| runtime_error("C_Navigation requires the engine NavigationFrame"))?;
    let frame = frame_ref(state, id)?;
    state.push(frame);
    Ok(1)
}

fn get_next_waypoint_for_map(_state: &mut LuaState) -> LuaResult<u32> {
    // R88ff VERIFIED: no-waypoint failure returns zero values, not a nil tuple.
    Ok(0)
}

fn get_nearest_party_member_token(state: &mut LuaState) -> LuaResult<u32> {
    #[cfg(feature = "retail-12-0-5")]
    {
        authorize_navigation_caller(state)?;
        let token = borrow_state(state)?
            .nearest_party_member_token
            .clone()
            .ok_or_else(|| {
                runtime_error(
                    "C_Navigation.GetNearestPartyMemberToken has no host-selected party token",
                )
            })?;
        let value = create_string(state, &token);
        state.push(value);
        return Ok(1);
    }
    #[cfg(not(feature = "retail-12-0-5"))]
    {
        state.push(Val::Nil);
        Ok(1)
    }
}

fn set_navigation_target(state: &mut LuaState) -> LuaResult<u32> {
    let target = parse_navigation_target(state)?;
    borrow_state_mut(state)?.super_track.navigation_target = target;
    if let Some(target) = target {
        place_navigation_frame(state, target)?;
    }
    update_navigation_frame(state)?;
    Ok(0)
}

fn parse_navigation_target(state: &mut LuaState) -> LuaResult<Option<NavigationTarget>> {
    let value = stack_val(state, 1);
    if value == Val::Nil {
        return Ok(None);
    }
    let Val::Table(_) = value else {
        return Err(runtime_error(
            "A_Admin.SetNavigationTarget expects a table or nil",
        ));
    };
    read_navigation_target(state, value).map(Some)
}

fn read_navigation_target(state: &mut LuaState, value: Val) -> LuaResult<NavigationTarget> {
    let state_value = required_number(state, value, "state")?;
    let distance = required_number(state, value, "distance")?;
    let clamped = required_bool(state, value, "clamped")?;
    let screen_x = required_number(state, value, "screenX")?;
    let screen_y = required_number(state, value, "screenY")?;
    if !(0.0..=3.0).contains(&state_value) || state_value.fract() != 0.0 {
        return Err(runtime_error(
            "A_Admin.SetNavigationTarget state must be 0..3",
        ));
    }
    if distance < 0.0 {
        return Err(runtime_error(
            "A_Admin.SetNavigationTarget distance must be nonnegative",
        ));
    }
    Ok(NavigationTarget {
        state: state_value as i32,
        distance,
        clamped,
        screen_x,
        screen_y,
    })
}

fn required_number(state: &mut LuaState, record: Val, key: &str) -> LuaResult<f64> {
    match table_get(state, record, key) {
        Val::Num(value) if value.is_finite() => Ok(value),
        _ => Err(runtime_error(format!(
            "A_Admin.SetNavigationTarget {key} must be finite number"
        ))),
    }
}

fn required_bool(state: &mut LuaState, record: Val, key: &str) -> LuaResult<bool> {
    match table_get(state, record, key) {
        Val::Bool(value) => Ok(value),
        _ => Err(runtime_error(format!(
            "A_Admin.SetNavigationTarget {key} must be boolean"
        ))),
    }
}

fn place_navigation_frame(state: &mut LuaState, target: NavigationTarget) -> LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    let frame_id = sim
        .widgets
        .get_id_by_name("NavigationFrame")
        .ok_or_else(|| {
            runtime_error("A_Admin.SetNavigationTarget requires the engine NavigationFrame")
        })?;
    let world_id = sim
        .widgets
        .get_id_by_name("WorldFrame")
        .ok_or_else(|| runtime_error("A_Admin.SetNavigationTarget requires WorldFrame"))?;
    if let Some(frame) = sim.widgets.get_mut_for_anchor_edit(frame_id) {
        frame.clear_all_points();
        frame.set_point(
            AnchorPoint::BottomLeft,
            Some(world_id as usize),
            AnchorPoint::BottomLeft,
            target.screen_x as f32,
            target.screen_y as f32,
        );
    }
    sim.widgets.add_anchor_dependent(world_id, frame_id);
    sim.widgets.mark_anchor_rect_dirty(frame_id);
    Ok(())
}

/// Preserve the simulator's untainted-caller restriction for this native query.
#[cfg(feature = "retail-12-0-5")]
fn authorize_navigation_caller(state: &LuaState) -> LuaResult<()> {
    let active_depth = state.ci.saturating_add(1);
    let addon_caller = state
        .call_stack
        .iter()
        .take(active_depth)
        .any(|info| info.taint.is_some());
    if addon_caller {
        return Err(runtime_error(
            "C_Navigation.GetNearestPartyMemberToken cannot be called by addons",
        ));
    }
    Ok(())
}
