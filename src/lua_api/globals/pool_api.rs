//! Pool API - CreateTexturePool, CreateFramePool, CreateObjectPool

use crate::lua_api::SimState;
use mlua::{Lua, Result, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// Register pool creation functions (CreateTexturePool, CreateFramePool, CreateObjectPool)
pub fn register_pool_api(lua: &Lua, state: Rc<RefCell<SimState>>) -> Result<()> {
    let globals = lua.globals();

    // CreateTexturePool(parent, layer, subLayer, textureTemplate, resetterFunc)
    // Creates a pool for managing reusable textures
    let create_texture_pool_state = Rc::clone(&state);
    let create_texture_pool = lua.create_function(move |lua, args: mlua::MultiValue| {
        let _state = create_texture_pool_state.borrow();
        let _args: Vec<Value> = args.into_iter().collect();
        // Create a simple pool table with Acquire/Release methods
        let pool = lua.create_table()?;
        let pool_storage = lua.create_table()?;
        pool.set("__storage", pool_storage)?;
        pool.set("__active", lua.create_table()?)?;
        pool.set(
            "Acquire",
            lua.create_function(|lua, this: mlua::Table| {
                // Return a new texture-like table
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
                // Track in active list
                let active: mlua::Table = this.get("__active")?;
                active.set(active.raw_len() + 1, texture.clone())?;
                Ok(texture)
            })?,
        )?;
        pool.set(
            "Release",
            lua.create_function(|_, (_this, _texture): (mlua::Table, mlua::Table)| Ok(()))?,
        )?;
        pool.set(
            "ReleaseAll",
            lua.create_function(|_, _this: mlua::Table| Ok(()))?,
        )?;
        Ok(pool)
    })?;
    globals.set("CreateTexturePool", create_texture_pool)?;

    // CreateFramePool(frameType, parent, template, resetterFunc, forbidden)
    // Creates a pool for managing reusable frames
    let create_frame_pool = lua.create_function(|lua, args: mlua::MultiValue| {
        let args: Vec<Value> = args.into_iter().collect();
        let frame_type: String = args.get(0)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Frame".to_string());
        // Parent can be nil, a string (global name), or a frame object
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
        pool.set(
            "Acquire",
            lua.create_function(move |lua, this: mlua::Table| {
                // First check inactive pool for a frame to reuse
                let inactive: mlua::Table = this.get("__inactive")?;
                let inactive_len = inactive.raw_len();
                if inactive_len > 0 {
                    // Reuse an existing frame
                    let frame: Value = inactive.get(inactive_len)?;
                    inactive.set(inactive_len, Value::Nil)?;
                    let active: mlua::Table = this.get("__active")?;
                    active.set(active.raw_len() + 1, frame.clone())?;
                    return Ok((frame, false)); // false = not new
                }

                // Create a new frame via CreateFrame
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
                Ok((frame, true)) // true = new frame
            })?,
        )?;
        pool.set(
            "Release",
            lua.create_function(|_, (this, frame): (mlua::Table, Value)| {
                // Move from active to inactive
                let inactive: mlua::Table = this.get("__inactive")?;
                inactive.set(inactive.raw_len() + 1, frame)?;
                Ok(())
            })?,
        )?;
        pool.set(
            "ReleaseAll",
            lua.create_function(|_, this: mlua::Table| {
                let active: mlua::Table = this.get("__active")?;
                let inactive: mlua::Table = this.get("__inactive")?;
                // Move all active to inactive
                for pair in active.pairs::<i64, Value>() {
                    let (_, frame) = pair?;
                    inactive.set(inactive.raw_len() + 1, frame)?;
                }
                // Clear active
                for i in 1..=active.raw_len() {
                    active.set(i, Value::Nil)?;
                }
                Ok(())
            })?,
        )?;
        pool.set(
            "GetNumActive",
            lua.create_function(|_, this: mlua::Table| {
                let active: mlua::Table = this.get("__active")?;
                Ok(active.raw_len())
            })?,
        )?;
        pool.set(
            "EnumerateActive",
            lua.create_function(|lua, this: mlua::Table| {
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
            })?,
        )?;
        Ok(pool)
    })?;
    globals.set("CreateFramePool", create_frame_pool)?;

    // CreateObjectPool(creatorFunc, resetterFunc) - generic object pool
    let create_object_pool = lua.create_function(|lua, (creator_func, _resetter_func): (mlua::Function, Option<mlua::Function>)| {
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
                Ok((obj, true)) // Return object and isNew flag
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
        pool.set(
            "GetNumActive",
            lua.create_function(|_, this: mlua::Table| {
                let active: mlua::Table = this.get("__active")?;
                Ok(active.raw_len())
            })?,
        )?;
        pool.set(
            "EnumerateActive",
            lua.create_function(|lua, _this: mlua::Table| {
                // Return an iterator function (simple stub)
                lua.create_function(|_, ()| Ok(Value::Nil))
            })?,
        )?;
        Ok(pool)
    })?;
    globals.set("CreateObjectPool", create_object_pool)?;

    Ok(())
}
