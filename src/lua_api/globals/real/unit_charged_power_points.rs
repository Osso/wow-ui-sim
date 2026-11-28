use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, table_set_num};
use crate::lua_bridge::{FromStack, stack_val};
use rilua::vm::{state::LuaState, table::Table};
use rilua::{Lua, LuaApiMut, LuaResult, Val, runtime_error};

pub(crate) fn register_all(lua: &mut Lua) -> LuaResult<()> {
    LuaApiMut::register_function(
        lua,
        "GetUnitChargedPowerPoints",
        get_unit_charged_power_points,
    )
}

fn get_unit_charged_power_points(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let Some(points) = borrow_state(state)?
        .unit_charged_power_points
        .get(&unit)
        .cloned()
    else {
        return Ok(0);
    };
    let table = state.gc.alloc_table(Table::with_sizes(points.len(), 0));
    state.push(Val::Table(table));
    for (index, point) in points.into_iter().enumerate() {
        table_set_num(state, table, (index + 1) as f64, Val::Num(f64::from(point)));
    }
    Ok(1)
}

pub(crate) fn admin_set_unit_charged_power_points(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let points = read_point_indices(state, stack_val(state, 2))?;
    borrow_state_mut(state)?
        .unit_charged_power_points
        .insert(unit.clone(), points);
    let unit = crate::lua_api::methods::create_string(state, &unit);
    dispatch_event_now(state, "UNIT_POWER_POINT_CHARGE", &[unit])?;
    Ok(0)
}

fn read_point_indices(state: &LuaState, value: Val) -> LuaResult<Vec<i32>> {
    let Val::Table(table) = value else {
        return Err(runtime_error("SetUnitChargedPowerPoints expects an array"));
    };
    let mut points = Vec::new();
    for index in 1.. {
        match state.gc.tables.get(table).map(|array| array.get_int(index)) {
            Some(Val::Num(point))
                if point.fract() == 0.0 && point > 0.0 && point <= i32::MAX as f64 =>
            {
                points.push(point as i32);
            }
            Some(Val::Nil) | None => break,
            Some(_) => {
                return Err(runtime_error(
                    "charged point indices must be positive integers",
                ));
            }
        }
    }
    Ok(points)
}
