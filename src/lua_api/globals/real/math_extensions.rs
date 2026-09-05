//! Native `math` extensions published by retail 12.1.5.
//!
//! `Blizzard_SharedXMLBase/MathUtil.lua` aliases these functions directly, so
//! they must be registered before Blizzard source loads.

use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

pub fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    let state = lua.state_mut();
    let math = math_table(state)?;
    table_set_rust_fn_static(state, math, "clamp", clamp)?;
    table_set_rust_fn_static(state, math, "isfinite", is_finite)?;
    table_set_rust_fn_static(state, math, "isinf", is_infinite)?;
    table_set_rust_fn_static(state, math, "isnan", is_nan)?;
    table_set_rust_fn_static(state, math, "lerp", lerp)?;
    table_set_rust_fn_static(state, math, "normalize", normalize)?;
    table_set_rust_fn_static(state, math, "remap", remap)?;
    table_set_rust_fn_static(state, math, "round", round)?;
    table_set_rust_fn_static(state, math, "saturate", saturate)?;
    table_set_rust_fn_static(state, math, "sign", sign)?;
    table_set_rust_fn_static(state, math, "wrap", wrap)?;
    Ok(())
}

fn math_table(
    state: &mut LuaState,
) -> LuaResult<rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>> {
    let math_key = state.gc.intern_string_static(b"math");
    let Some(global) = state.gc.tables.get(state.global) else {
        return Err(runtime_error("global table has been collected"));
    };
    let Val::Table(math) = global.get_str(math_key, &state.gc.string_arena) else {
        return Err(runtime_error("math table is unavailable"));
    };
    Ok(math)
}

fn number(state: &LuaState, index: i32) -> LuaResult<f64> {
    f64::from_stack(state, index)
}

fn optional_number(state: &LuaState, index: i32, default: f64) -> LuaResult<f64> {
    if matches!(stack_val(state, index), Val::Nil) {
        Ok(default)
    } else {
        number(state, index)
    }
}

fn checked_range(minimum: f64, maximum: f64) -> LuaResult<()> {
    if minimum > maximum {
        return Err(runtime_error("minimum must not exceed maximum"));
    }
    Ok(())
}

fn clamp(state: &mut LuaState) -> LuaResult<u32> {
    let value = number(state, 1)?;
    let minimum = number(state, 2)?;
    let maximum = number(state, 3)?;
    checked_range(minimum, maximum)?;
    state.push(Val::Num(value.max(minimum).min(maximum)));
    Ok(1)
}

fn is_finite(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(number(state, 1)?.is_finite()));
    Ok(1)
}

fn is_infinite(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(number(state, 1)?.is_infinite()));
    Ok(1)
}

fn is_nan(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(number(state, 1)?.is_nan()));
    Ok(1)
}

fn lerp(state: &mut LuaState) -> LuaResult<u32> {
    let start = number(state, 1)?;
    let end = number(state, 2)?;
    let amount = number(state, 3)?;
    state.push(Val::Num(start + (end - start) * amount));
    Ok(1)
}

fn normalize(state: &mut LuaState) -> LuaResult<u32> {
    let value = number(state, 1)?;
    let minimum = number(state, 2)?;
    let maximum = number(state, 3)?;
    state.push(Val::Num((value - minimum) / (maximum - minimum)));
    Ok(1)
}

fn remap(state: &mut LuaState) -> LuaResult<u32> {
    let value = number(state, 1)?;
    let source_minimum = number(state, 2)?;
    let source_maximum = number(state, 3)?;
    let destination_minimum = number(state, 4)?;
    let destination_maximum = number(state, 5)?;
    let amount = (value - source_minimum) / (source_maximum - source_minimum);
    state.push(Val::Num(
        destination_minimum + (destination_maximum - destination_minimum) * amount,
    ));
    Ok(1)
}

fn round(state: &mut LuaState) -> LuaResult<u32> {
    let value = number(state, 1)?;
    let decimal_places = optional_number(state, 2, 0.0)?;
    let scale = 10.0_f64.powf(decimal_places);
    let scaled = value * scale;
    let rounded = if scaled.is_sign_negative() {
        (scaled - 0.5).ceil()
    } else {
        (scaled + 0.5).floor()
    };
    state.push(Val::Num(rounded / scale));
    Ok(1)
}

fn saturate(state: &mut LuaState) -> LuaResult<u32> {
    let value = number(state, 1)?;
    state.push(Val::Num(value.max(0.0).min(1.0)));
    Ok(1)
}

fn sign(state: &mut LuaState) -> LuaResult<u32> {
    let value = number(state, 1)?;
    let sign = if value > 0.0 {
        1.0
    } else if value < 0.0 {
        -1.0
    } else {
        0.0
    };
    state.push(Val::Num(sign));
    Ok(1)
}

fn wrap(state: &mut LuaState) -> LuaResult<u32> {
    let value = number(state, 1)?;
    let minimum = number(state, 2)?;
    let maximum = number(state, 3)?;
    checked_range(minimum, maximum)?;
    if minimum == maximum {
        state.push(Val::Num(minimum));
        return Ok(1);
    }

    let range = maximum - minimum;
    state.push(Val::Num((value - minimum).rem_euclid(range) + minimum));
    Ok(1)
}
