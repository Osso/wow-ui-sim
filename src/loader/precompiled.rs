//! Precompiled Lua helper functions to eliminate repeated source compilation.
//!
//! The XML loader generates thousands of unique Lua code strings via `env.exec()` —
//! each compiled from source every load. This module precompiles parameterized helper
//! functions once at startup and stores them for reuse, eliminating ~12,000+ redundant
//! Lua compilation calls.

use mlua::{Function, Lua};

/// Precompiled Lua functions for the XML loader, stored in Lua app_data.
///
/// Each function is compiled once at startup and called with arguments instead of
/// generating and compiling unique Lua source strings for each frame.
pub struct PrecompiledFns {
    /// Fire OnLoad lifecycle script on a frame (by global name).
    pub fire_onload: Function,
    /// Fire OnShow lifecycle script on a frame (by global name).
    pub fire_onshow: Function,
    /// Increment the `__suppress_create_frame_onload` counter.
    pub suppress_push: Function,
    /// Decrement the `__suppress_create_frame_onload` counter.
    pub suppress_pop: Function,
    /// Assign `_G[parent_name][key] = _G[child_name]`.
    pub assign_parent_key: Function,
    /// Set `_G[frame_name].intrinsic = base_name`.
    pub set_intrinsic: Function,
}

impl PrecompiledFns {
    /// Compile all helper functions once and return the struct.
    pub fn new(lua: &Lua) -> mlua::Result<Self> {
        Ok(Self {
            fire_onload: compile_fire_onload(lua)?,
            fire_onshow: compile_fire_onshow(lua)?,
            suppress_push: compile_suppress_push(lua)?,
            suppress_pop: compile_suppress_pop(lua)?,
            assign_parent_key: compile_assign_parent_key(lua)?,
            set_intrinsic: compile_set_intrinsic(lua)?,
        })
    }
}

fn compile_fire_onload(lua: &Lua) -> mlua::Result<Function> {
    lua.load(r#"
        local frame = _G[...]
        if not frame then return end
        if type(frame.OnLoad_Intrinsic) == "function" then
            local ok, err = pcall(frame.OnLoad_Intrinsic, frame)
            if not ok then
                __report_script_error("[OnLoad_Intrinsic] " .. tostring(err))
            end
        end
        local handler = frame:GetScript("OnLoad")
        if handler then
            local ok, err = pcall(handler, frame)
            if not ok then
                local name = frame.GetName and frame:GetName() or "?"
                __report_script_error("[OnLoad] " .. name .. ": " .. tostring(err))
            end
        end
    "#).into_function()
}

fn compile_fire_onshow(lua: &Lua) -> mlua::Result<Function> {
    lua.load(r#"
        local frame = _G[...]
        if not frame then return end
        if frame:IsVisible() then
            local handler = frame:GetScript("OnShow")
            if handler then
                local ok, err = pcall(handler, frame)
                if not ok then
                    local name = frame.GetName and frame:GetName() or "?"
                    __report_script_error("[OnShow] " .. name .. ": " .. tostring(err))
                end
            end
            if type(frame.OnShow_Intrinsic) == "function" then
                local ok, err = pcall(frame.OnShow_Intrinsic, frame)
                if not ok then
                    __report_script_error("[OnShow_Intrinsic] " .. tostring(err))
                end
            end
        end
    "#).into_function()
}

fn compile_suppress_push(lua: &Lua) -> mlua::Result<Function> {
    lua.load(
        "__suppress_create_frame_onload = (__suppress_create_frame_onload or 0) + 1"
    ).into_function()
}

fn compile_suppress_pop(lua: &Lua) -> mlua::Result<Function> {
    lua.load(
        "__suppress_create_frame_onload = __suppress_create_frame_onload - 1"
    ).into_function()
}

fn compile_assign_parent_key(lua: &Lua) -> mlua::Result<Function> {
    lua.load(r#"
        local parent_name, key, child_name = ...
        local parent = _G[parent_name]
        local child = _G[child_name]
        if parent and child then
            parent[key] = child
        end
    "#).into_function()
}

fn compile_set_intrinsic(lua: &Lua) -> mlua::Result<Function> {
    lua.load(r#"
        local frame_name, base = ...
        local frame = _G[frame_name]
        if frame then
            frame.intrinsic = base
        end
    "#).into_function()
}

/// Initialize precompiled functions and store them in Lua app_data.
///
/// Must be called once during `WowLuaEnv::new()` after globals are registered
/// (since the functions reference `__report_script_error` etc.).
pub fn init(lua: &Lua) -> mlua::Result<()> {
    let fns = PrecompiledFns::new(lua)?;
    lua.set_app_data(fns);
    Ok(())
}

/// Retrieve the precompiled functions from Lua app_data.
///
/// Returns cloned `Function` handles (cheap Rc clone) to avoid holding
/// a `Ref<PrecompiledFns>` borrow across Lua calls.
pub fn get(lua: &Lua) -> PrecompiledFnsRef {
    let fns = lua.app_data_ref::<PrecompiledFns>()
        .expect("PrecompiledFns not initialized — call precompiled::init() first");
    PrecompiledFnsRef {
        fire_onload: fns.fire_onload.clone(),
        fire_onshow: fns.fire_onshow.clone(),
        suppress_push: fns.suppress_push.clone(),
        suppress_pop: fns.suppress_pop.clone(),
        assign_parent_key: fns.assign_parent_key.clone(),
        set_intrinsic: fns.set_intrinsic.clone(),
    }
}

/// Owned copy of precompiled function handles (cheap Rc clones).
///
/// This avoids holding a `Ref<PrecompiledFns>` borrow across Lua calls.
pub struct PrecompiledFnsRef {
    pub fire_onload: Function,
    pub fire_onshow: Function,
    pub suppress_push: Function,
    pub suppress_pop: Function,
    pub assign_parent_key: Function,
    pub set_intrinsic: Function,
}
