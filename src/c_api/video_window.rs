//! INFERRED window-size queries backed by the simulator's physical display.
use crate::lua_api::methods::{borrow_state, create_table, table_set, table_set_num};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    use crate::client_profile::{ACTIVE, ClientProfile};
    if !matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever) {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_VideoOptions")?;
    table_set_rust_fn_static(state, namespace, "GetCurrentGameWindowSize", current_size)?;
    table_set_rust_fn_static(state, namespace, "GetDefaultGameWindowSize", current_size)?;
    table_set_rust_fn_static(state, namespace, "GetGameWindowSizes", available_sizes)?;
    table_set_rust_fn_static(state, namespace, "SetGameWindowSize", set_size)
}

fn current_dimensions(state: &LuaState) -> LuaResult<(f32, f32)> {
    let model = borrow_state(state)?;
    Ok((model.physical_screen_width, model.physical_screen_height))
}

fn make_size(state: &mut LuaState, width: f32, height: f32) -> Val {
    let size = create_table(state);
    table_set(state, size, "x", Val::Num(f64::from(width)));
    table_set(state, size, "y", Val::Num(f64::from(height)));
    size
}

fn current_size(state: &mut LuaState) -> LuaResult<u32> {
    let (width, height) = current_dimensions(state)?;
    let size = make_size(state, width, height);
    state.push(size);
    Ok(1)
}

fn available_sizes(state: &mut LuaState) -> LuaResult<u32> {
    let (width, height) = current_dimensions(state)?;
    let size = make_size(state, width, height);
    let sizes = create_table(state);
    let Val::Table(sizes_ref) = sizes else {
        unreachable!("create_table returns a table")
    };
    table_set_num(state, sizes_ref, 1.0, size);
    state.push(Val::Table(sizes_ref));
    Ok(1)
}

fn set_size(state: &mut LuaState) -> LuaResult<u32> {
    let width = f64::from_stack(state, 1)?;
    let height = f64::from_stack(state, 2)?;
    if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
        return Err(runtime_error(
            "window dimensions must be finite and positive",
        ));
    }
    crate::lua_api::set_display_size_from_lua(state, width as f32, height as f32)?;
    Ok(0)
}
