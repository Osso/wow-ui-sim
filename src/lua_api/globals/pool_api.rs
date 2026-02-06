//! Pool API - CreateTexturePool, CreateFramePool, CreateObjectPool

use crate::lua_api::SimState;
use mlua::{Lua, Result, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// Register pool creation functions (CreateTexturePool, CreateFramePool, CreateObjectPool)
pub fn register_pool_api(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let globals = lua.globals();

    globals.set("CreateTexturePool", create_texture_pool_fn(lua, state)?)?;
    globals.set("CreateFramePool", create_frame_pool_fn(lua)?)?;
    globals.set("CreateObjectPool", create_object_pool_fn(lua)?)?;

    Ok(())
}

/// Create the `CreateTexturePool` function.
fn create_texture_pool_fn(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<mlua::Function> {
    lua.create_function(move |lua, args: mlua::MultiValue| {
        let _state = state.borrow();
        let _args: Vec<Value> = args.into_iter().collect();
        let pool = lua.create_table()?;
        pool.set("__storage", lua.create_table()?)?;
        pool.set("__active", lua.create_table()?)?;
        pool.set("Acquire", lua.create_function(acquire_texture)?)?;
        pool.set(
            "Release",
            lua.create_function(|_, (_this, _texture): (mlua::Table, mlua::Table)| Ok(()))?,
        )?;
        pool.set(
            "ReleaseAll",
            lua.create_function(|_, _this: mlua::Table| Ok(()))?,
        )?;
        Ok(pool)
    })
}

/// Acquire a texture from the pool - creates a stub texture table.
fn acquire_texture(lua: &Lua, this: mlua::Table) -> Result<mlua::Table> {
    let texture = lua.create_table()?;
    texture.set("SetTexture", lua.create_function(|_, _: mlua::MultiValue| Ok(()))?)?;
    texture.set("SetTexCoord", lua.create_function(|_, _: mlua::MultiValue| Ok(()))?)?;
    texture.set("SetVertexColor", lua.create_function(|_, _: mlua::MultiValue| Ok(()))?)?;
    texture.set("SetBlendMode", lua.create_function(|_, _: String| Ok(()))?)?;
    texture.set("SetDrawLayer", lua.create_function(|_, _: mlua::MultiValue| Ok(()))?)?;
    texture.set("SetAllPoints", lua.create_function(|_, _: mlua::MultiValue| Ok(()))?)?;
    texture.set("SetPoint", lua.create_function(|_, _: mlua::MultiValue| Ok(()))?)?;
    texture.set("ClearAllPoints", lua.create_function(|_, ()| Ok(()))?)?;
    texture.set("SetAlpha", lua.create_function(|_, _: f64| Ok(()))?)?;
    texture.set("SetSize", lua.create_function(|_, _: mlua::MultiValue| Ok(()))?)?;
    texture.set("Show", lua.create_function(|_, ()| Ok(()))?)?;
    texture.set("Hide", lua.create_function(|_, ()| Ok(()))?)?;
    texture.set("SetParent", lua.create_function(|_, _: mlua::MultiValue| Ok(()))?)?;
    let active: mlua::Table = this.get("__active")?;
    active.set(active.raw_len() + 1, texture.clone())?;
    Ok(texture)
}

/// Create the `CreateFramePool` function.
fn create_frame_pool_fn(lua: &Lua) -> Result<mlua::Function> {
    lua.create_function(|lua, args: mlua::MultiValue| {
        let args: Vec<Value> = args.into_iter().collect();
        let frame_type: String = args.get(0)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Frame".to_string());
        let parent_val = args.get(1).cloned().unwrap_or(Value::Nil);
        let template: Option<String> = args.get(2)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let pool = lua.create_table()?;
        pool.set("__active", lua.create_table()?)?;
        pool.set("__inactive", lua.create_table()?)?;
        pool.set("__frame_type", frame_type.clone())?;
        pool.set("__parent", parent_val.clone())?;
        if let Some(ref tmpl) = template {
            pool.set("__template", tmpl.clone())?;
        }
        pool.set("Acquire", lua.create_function(acquire_frame)?)?;
        pool.set("Release", lua.create_function(release_frame)?)?;
        pool.set("ReleaseAll", lua.create_function(release_all_frames)?)?;
        pool.set("GetNumActive", lua.create_function(get_num_active)?)?;
        pool.set("EnumerateActive", lua.create_function(enumerate_active)?)?;
        Ok(pool)
    })
}

/// Acquire a frame from the pool, reusing inactive ones or creating new via CreateFrame.
fn acquire_frame(lua: &Lua, this: mlua::Table) -> Result<(Value, bool)> {
    let inactive: mlua::Table = this.get("__inactive")?;
    let inactive_len = inactive.raw_len();
    if inactive_len > 0 {
        let frame: Value = inactive.get(inactive_len)?;
        inactive.set(inactive_len, Value::Nil)?;
        let active: mlua::Table = this.get("__active")?;
        active.set(active.raw_len() + 1, frame.clone())?;
        return Ok((frame, false));
    }

    let create_frame: mlua::Function = lua.globals().get("CreateFrame")?;
    let frame_type: String = this.get("__frame_type")?;
    let parent: Value = this.get("__parent")?;
    let template: Option<String> = this.get("__template").ok();

    let frame = if let Some(tmpl) = template {
        create_frame.call::<Value>((frame_type, Value::Nil, parent, tmpl))?
    } else {
        create_frame.call::<Value>((frame_type, Value::Nil, parent))?
    };

    let active: mlua::Table = this.get("__active")?;
    active.set(active.raw_len() + 1, frame.clone())?;
    Ok((frame, true))
}

/// Release a frame back to the inactive pool.
fn release_frame(_: &Lua, (this, frame): (mlua::Table, Value)) -> Result<()> {
    let inactive: mlua::Table = this.get("__inactive")?;
    inactive.set(inactive.raw_len() + 1, frame)?;
    Ok(())
}

/// Release all active frames back to the inactive pool.
fn release_all_frames(_: &Lua, this: mlua::Table) -> Result<()> {
    let active: mlua::Table = this.get("__active")?;
    let inactive: mlua::Table = this.get("__inactive")?;
    for pair in active.pairs::<i64, Value>() {
        let (_, frame) = pair?;
        inactive.set(inactive.raw_len() + 1, frame)?;
    }
    for i in 1..=active.raw_len() {
        active.set(i, Value::Nil)?;
    }
    Ok(())
}

/// Get the number of active frames in the pool.
fn get_num_active(_: &Lua, this: mlua::Table) -> Result<i64> {
    let active: mlua::Table = this.get("__active")?;
    Ok(active.raw_len() as i64)
}

/// Return an iterator over active frames.
fn enumerate_active(lua: &Lua, this: mlua::Table) -> Result<(mlua::Function, mlua::Table, Value)> {
    let active: mlua::Table = this.get("__active")?;
    let iter_state = lua.create_table()?;
    iter_state.set("tbl", active)?;
    iter_state.set("idx", 0i64)?;
    let iter_fn = lua.create_function(|_, state: mlua::Table| {
        let tbl: mlua::Table = state.get("tbl")?;
        let idx: i64 = state.get("idx")?;
        let next_idx = idx + 1;
        if next_idx <= tbl.raw_len() as i64 {
            state.set("idx", next_idx)?;
            let val: Value = tbl.get(next_idx)?;
            Ok((Some(val), Value::Nil))
        } else {
            Ok((None, Value::Nil))
        }
    })?;
    Ok((iter_fn, iter_state, Value::Nil))
}

/// Create the `CreateObjectPool` function.
fn create_object_pool_fn(lua: &Lua) -> Result<mlua::Function> {
    lua.create_function(|lua, (creator_func, _resetter_func): (mlua::Function, Option<mlua::Function>)| {
        let pool = lua.create_table()?;
        pool.set("__creator", creator_func.clone())?;
        pool.set("__active", lua.create_table()?)?;
        pool.set("__inactive", lua.create_table()?)?;
        pool.set(
            "Acquire",
            lua.create_function(|_lua, this: mlua::Table| {
                let creator: mlua::Function = this.get("__creator")?;
                let obj = creator.call::<Value>(())?;
                let active: mlua::Table = this.get("__active")?;
                active.set(active.raw_len() + 1, obj.clone())?;
                Ok((obj, true))
            })?,
        )?;
        pool.set(
            "Release",
            lua.create_function(|_, (_this, _obj): (mlua::Table, Value)| Ok(()))?,
        )?;
        pool.set(
            "ReleaseAll",
            lua.create_function(|_, _this: mlua::Table| Ok(()))?,
        )?;
        pool.set("GetNumActive", lua.create_function(get_num_active)?)?;
        pool.set(
            "EnumerateActive",
            lua.create_function(|lua, _this: mlua::Table| {
                lua.create_function(|_, ()| Ok(Value::Nil))
            })?,
        )?;
        Ok(pool)
    })
}
