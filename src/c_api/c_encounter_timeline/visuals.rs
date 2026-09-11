//! Script colors and icon slots derive from event data; no secret-aspect enforcement.
use super::{layout, read_id};
use crate::lua_api::methods::{borrow_state, call_function_state};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val, runtime_error};

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetEventColor", color)?;
    table_set_rust_fn_static(state, namespace, "SetEventIconTextures", icons)
}

fn color(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_id(state)?;
    let trigger = match stack_val(state, 2) {
        Val::Nil => None,
        Val::Num(value) if (0.0..=2.0).contains(&value) && value.fract() == 0.0 => {
            Some(value as u8)
        }
        _ => return Err(runtime_error("unknown encounter color trigger")),
    };
    let color = borrow_state(state)?
        .encounter_timeline
        .events
        .get(&id)
        .map(|event| {
            let highlight = trigger == Some(2)
                || (trigger.is_none() && layout::remaining(event) <= layout::HIGHLIGHT_TIME);
            if highlight {
                [1.0, 0.3, 0.2, 1.0]
            } else {
                match event.info.severity {
                    0 => [1.0, 1.0, 1.0, 1.0],
                    1 => [1.0, 0.82, 0.0, 1.0],
                    _ => [1.0, 0.2, 0.2, 1.0],
                }
            }
        });
    let Some(color) = color else { return Ok(0) };
    let function = crate::c_api::global_val(state, "CreateColor");
    let value = call_function_state(state, function, &color.map(Val::Num))?;
    state.push(value);
    Ok(1)
}

fn call_method(state: &mut LuaState, object: Val, method: &str, argument: Val) -> LuaResult<()> {
    let key = state.gc.intern_string(method.as_bytes());
    let function = state.gettable(object, Val::Str(key))?;
    call_function_state(state, function, &[object, argument])?;
    Ok(())
}

fn mask(state: &LuaState) -> LuaResult<u32> {
    match stack_val(state, 2) {
        Val::Num(value) if (0.0..=f64::from(u32::MAX)).contains(&value) && value.fract() == 0.0 => {
            Ok(value as u32)
        }
        _ => Err(runtime_error("icon mask must be a nonnegative integer")),
    }
}

fn icons(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_id(state)?;
    let selected = mask(state)?;
    let table = stack_val(state, 3);
    let Val::Table(reference) = table else {
        return Err(runtime_error("icon textures must be an array"));
    };
    let info = borrow_state(state)?
        .encounter_timeline
        .events
        .get(&id)
        .map(|event| event.info.clone());
    let Some(info) = info else { return Ok(0) };
    let bits: Vec<_> = (0..10)
        .map(|shift| 1u32 << shift)
        .filter(|bit| info.icons & selected & bit != 0)
        .collect();
    let count = state
        .gc
        .tables
        .get(reference)
        .map_or(0, |table| table.len(&state.gc.string_arena));
    for index in 1..=count {
        let texture = state.gettable(table, Val::Num(index as f64))?;
        assign_icon(state, texture, bits.get(index - 1).copied(), info.icon)?;
    }
    Ok(0)
}

fn assign_icon(state: &mut LuaState, texture: Val, bit: Option<u32>, icon: u32) -> LuaResult<()> {
    let atlas = match bit {
        Some(128) => Some("roleicon-tiny-tank"),
        Some(256) => Some("roleicon-tiny-healer"),
        Some(512) => Some("roleicon-tiny-dps"),
        _ => None,
    };
    if let Some(atlas) = atlas {
        let value = state.gc.intern_string(atlas.as_bytes());
        call_method(state, texture, "SetAtlas", Val::Str(value))?;
    } else {
        let value = bit.map_or(Val::Nil, |_| Val::Num(f64::from(icon)));
        call_method(state, texture, "SetTexture", value)?;
    }
    call_method(
        state,
        texture,
        "SetAlpha",
        Val::Num(if bit.is_some() { 1.0 } else { 0.0 }),
    )
}
