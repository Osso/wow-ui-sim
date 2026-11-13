use super::super::helpers::{bad_argument, parse_numeric_string, read_untainted_signed32_argument};
use super::{UserWaypoint, create_world_position_vector};
use crate::lua_api::methods::{borrow_state, borrow_state_mut, call_function_state, table_get};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaApiMut, LuaResult, Val};

const SET_FUNCTION: &str = "SetUserWaypoint";
const SET_USAGE: &str = "Usage: local wasSet = C_Map.SetUserWaypoint(point)";
const POSITION_FUNCTION: &str = "GetUserWaypointPositionForMap";
const POSITION_USAGE: &str =
    "Usage: local mapPosition = C_Map.GetUserWaypointPositionForMap(uiMapID)";
const CAN_SET_FUNCTION: &str = "CanSetUserWaypointOnMap";
const CAN_SET_USAGE: &str = "Usage: local canSet = C_Map.CanSetUserWaypointOnMap(uiMapID)";

const USER_WAYPOINT_METHODS: &[(&str, rilua::vm::closure::RustFn)] = &[
    ("SetUserWaypoint", set_user_waypoint),
    ("GetUserWaypoint", get_user_waypoint),
    (
        "GetUserWaypointPositionForMap",
        get_user_waypoint_position_for_map,
    ),
    ("HasUserWaypoint", has_user_waypoint),
    ("ClearUserWaypoint", clear_user_waypoint),
    ("CanSetUserWaypointOnMap", can_set_user_waypoint_on_map),
];

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    // VERIFIED: generated API docs publish this surface on Retail, PTR and Forever.
    // INFERRED: PTR/Forever share the Retail behavior below; no profile-specific
    // behavioral assertions are made for either profile.
    if !matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::Ptr
            | crate::client_profile::ClientProfile::WowForever
    ) {
        return Ok(());
    }

    for &(name, function) in USER_WAYPOINT_METHODS {
        table_set_rust_fn_static(state, namespace, name, function)?;
    }
    Ok(())
}

fn set_user_waypoint(state: &mut LuaState) -> LuaResult<u32> {
    let waypoint = read_waypoint(state)?;
    let was_set = user_waypoint_is_valid(&waypoint) && can_set_map(waypoint.ui_map_id);
    if was_set {
        borrow_state_mut(state)?.user_waypoint = Some(waypoint);
        call_optional_waypoint_change_callback(state)?;
    }

    // INFERRED: publish one `wasSet` Boolean; the native return bridge is unresolved.
    state.push(Val::Bool(was_set));
    Ok(1)
}

fn get_user_waypoint(state: &mut LuaState) -> LuaResult<u32> {
    let waypoint = borrow_state(state)?.user_waypoint;
    let Some(waypoint) = waypoint.filter(user_waypoint_is_valid) else {
        // The API permits no return; invalid stored points are not serialized.
        return Ok(0);
    };

    let point = create_ui_map_point(state, waypoint)?;
    state.push(point);
    Ok(1)
}

fn get_user_waypoint_position_for_map(state: &mut LuaState) -> LuaResult<u32> {
    let map_id = read_untainted_signed32_argument(state, 1, POSITION_FUNCTION, POSITION_USAGE)?;
    let waypoint = borrow_state(state)?.user_waypoint;
    let Some(waypoint) = waypoint.filter(user_waypoint_is_valid) else {
        return Ok(0);
    };
    if waypoint.ui_map_id != map_id {
        // INFERRED per task fallback: do not project across UNKNOWN map relations.
        return Ok(0);
    }

    let position = create_world_position_vector(state, waypoint.x, waypoint.y)?;
    state.push(position);
    Ok(1)
}

fn has_user_waypoint(state: &mut LuaState) -> LuaResult<u32> {
    let waypoint = borrow_state(state)?.user_waypoint;
    state.push(Val::Bool(
        waypoint.is_some_and(|waypoint| user_waypoint_is_valid(&waypoint)),
    ));
    Ok(1)
}

fn clear_user_waypoint(state: &mut LuaState) -> LuaResult<u32> {
    let had_valid_waypoint = {
        let mut sim = borrow_state_mut(state)?;
        sim.user_waypoint
            .take()
            .is_some_and(|waypoint| user_waypoint_is_valid(&waypoint))
    };
    if had_valid_waypoint {
        call_optional_waypoint_change_callback(state)?;
    }
    Ok(0)
}

fn can_set_user_waypoint_on_map(state: &mut LuaState) -> LuaResult<u32> {
    let map_id = read_untainted_signed32_argument(state, 1, CAN_SET_FUNCTION, CAN_SET_USAGE)?;
    state.push(Val::Bool(can_set_map(map_id)));
    Ok(1)
}

/// INFERRED: map eligibility uses only the nonzero signed map ID because native
/// category and allowed-map inputs remain UNKNOWN in R88h.6.
fn can_set_map(map_id: i32) -> bool {
    map_id != 0
}

fn read_waypoint(state: &mut LuaState) -> LuaResult<UserWaypoint> {
    // Retail, PTR and Forever generated API docs declare AllowedWhenUntainted;
    // R88h.1 supplies the denial template used by this gate.
    let point = read_allowed_when_untainted(state, stack_val(state, 1))?;
    let Val::Table(point) = point else {
        return Err(point_conversion_error());
    };
    let point = Val::Table(point);

    let raw_map_id = read_point_field(state, point, "uiMapID")?;
    let raw_position = read_point_field(state, point, "position")?;
    let Val::Table(position) = raw_position else {
        return Err(point_conversion_error());
    };
    let position = Val::Table(position);
    let raw_x = read_point_field(state, position, "x")?;
    let raw_y = read_point_field(state, position, "y")?;
    let raw_z = read_point_field(state, point, "z")?;

    Ok(UserWaypoint {
        ui_map_id: parse_signed_map_id(state, raw_map_id)?,
        x: numeric_value(state, raw_x).ok_or_else(point_conversion_error)?,
        y: numeric_value(state, raw_y).ok_or_else(point_conversion_error)?,
        // INFERRED: finite numeric z is accepted; rejected z values are omitted.
        z: numeric_value(state, raw_z).filter(|number| number.is_finite()),
    })
}

fn read_point_field(state: &mut LuaState, table: Val, key: &str) -> LuaResult<Val> {
    let value = table_get(state, table, key);
    read_allowed_when_untainted(state, value)
}

fn read_allowed_when_untainted(state: &LuaState, value: Val) -> LuaResult<Val> {
    if rilua::table_security::is_secret_value(state, value)
        && !rilua::table_security::can_access_secrets(state)
    {
        return Err(secret_access_error());
    }
    rilua::table_security::unwrap_secret(state, value).map_err(|_| secret_access_error())
}

fn secret_access_error() -> rilua::LuaError {
    bad_argument(
        1,
        SET_FUNCTION,
        &format!(
            "{SET_USAGE}. Secret values are only allowed during untainted execution for this argument."
        ),
    )
}

fn parse_signed_map_id(state: &LuaState, value: Val) -> LuaResult<i32> {
    let number = numeric_value(state, value).ok_or_else(point_conversion_error)?;
    if !number.is_finite() || number < f64::from(i32::MIN) || number > f64::from(i32::MAX) {
        return Err(point_conversion_error());
    }
    Ok(number.trunc() as i32)
}

fn numeric_value(state: &LuaState, value: Val) -> Option<f64> {
    match value {
        Val::Num(number) => Some(number),
        Val::Str(reference) => state
            .gc
            .string_arena
            .get(reference)
            .and_then(|string| parse_numeric_string(string.data())),
        _ => None,
    }
}

fn point_conversion_error() -> rilua::LuaError {
    // INFERRED suffix: R88h.1 establishes the `%s - %s` composition and Usage,
    // but not the second operand's exact text.
    bad_argument(1, SET_FUNCTION, &format!("{SET_USAGE} - invalid point"))
}

fn user_waypoint_is_valid(waypoint: &UserWaypoint) -> bool {
    waypoint.ui_map_id != 0 && is_normalized(waypoint.x) && is_normalized(waypoint.y)
}

fn is_normalized(value: f64) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}

fn create_ui_map_point(state: &mut LuaState, waypoint: UserWaypoint) -> LuaResult<Val> {
    let ui_map_point = LuaApiMut::get_global_val(state, "UiMapPoint");
    let constructor = table_get(state, ui_map_point, "CreateFromCoordinates");
    let mut arguments = vec![
        Val::Num(f64::from(waypoint.ui_map_id)),
        Val::Num(waypoint.x),
        Val::Num(waypoint.y),
    ];
    if let Some(z) = waypoint.z {
        arguments.push(Val::Num(z));
    }
    call_function_state(state, constructor, &arguments)
}

/// The callback is present in the native path, but its identity and event are
/// UNKNOWN. Keep one hook and dispatch nothing until that evidence is known.
fn call_optional_waypoint_change_callback(_state: &mut LuaState) -> LuaResult<()> {
    Ok(())
}
