//! Model and ModelScene widget method stubs.

use crate::lua_api::frame::handle::{get_sim_state, lud_to_id};
use mlua::{IntoLuaMulti, LightUserData, Lua, Result, Value};

pub fn add_model_methods(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    add_model_transform_methods(lua, methods)?;
    add_model_appearance_methods(lua, methods)?;
    add_model_scene_id_methods(lua, methods)?;
    Ok(())
}

fn add_model_transform_methods(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    methods.set("SetModel", lua.create_function(|_, (_ud, _path): (LightUserData, String)| Ok(()))?)?;
    methods.set("GetModel", lua.create_function(|_, _ud: LightUserData| -> Result<Option<String>> {
        Ok(None)
    })?)?;
    methods.set("SetModelScale", lua.create_function(|_, (_ud, _scale): (LightUserData, f64)| Ok(()))?)?;
    methods.set("GetModelScale", lua.create_function(|_, _ud: LightUserData| Ok(1.0_f64))?)?;
    methods.set("SetPosition", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;

    methods.set("GetPosition", lua.create_function(|lua, ud: LightUserData| {
        let id = lud_to_id(ud);
        if let Some((func, frame_ud)) = super::methods_helpers::get_mixin_override(lua, id, "GetPosition") {
            return func.call::<mlua::MultiValue>(frame_ud);
        }
        (0.0_f64, 0.0_f64, 0.0_f64).into_lua_multi(lua)
    })?)?;

    methods.set("SetFacing", lua.create_function(|_, (_ud, _radians): (LightUserData, f64)| Ok(()))?)?;
    methods.set("GetFacing", lua.create_function(|_, _ud: LightUserData| Ok(0.0_f64))?)?;

    // Mixin override: ModelScenelRotateButtonMixin defines SetRotation(direction)
    // Falls through to Texture:SetRotation(radians) when no mixin override exists.
    methods.set("SetRotation", lua.create_function(|lua, (ud, radians): (LightUserData, Value)| {
        let id = lud_to_id(ud);
        if let Some((func, frame_ud)) = super::methods_helpers::get_mixin_override(lua, id, "SetRotation") {
            return func.call::<()>((frame_ud, radians));
        }
        let rad_f64 = match radians {
            Value::Number(n) => n,
            Value::Integer(n) => n as f64,
            _ => 0.0,
        };
        let state_rc = get_sim_state(lua);
        let mut state = state_rc.borrow_mut();
        if let Some(frame) = state.widgets.get_mut_visual(id) {
            frame.rotation = rad_f64 as f32;
        }
        Ok(())
    })?)?;

    Ok(())
}

fn add_model_appearance_methods(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    methods.set("SetUnit", lua.create_function(|lua, (ud, args): (LightUserData, mlua::MultiValue)| {
        let id = lud_to_id(ud);
        if let Some((func, frame_ud)) = super::methods_helpers::get_mixin_override(lua, id, "SetUnit") {
            let mut call_args = vec![frame_ud];
            call_args.extend(args);
            return func.call::<()>(mlua::MultiValue::from_iter(call_args));
        }
        Ok(())
    })?)?;

    add_model_appearance_stubs(lua, methods)?;
    Ok(())
}

fn add_model_appearance_stubs(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    methods.set("SetAutoDress", lua.create_function(|_, (_ud, _auto): (LightUserData, bool)| Ok(()))?)?;
    methods.set("SetDisplayInfo", lua.create_function(|_, (_ud, _id): (LightUserData, i32)| Ok(()))?)?;
    methods.set("SetCreature", lua.create_function(|_, (_ud, _id): (LightUserData, i32)| Ok(()))?)?;
    methods.set("SetAnimation", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;
    methods.set("SetCamDistanceScale", lua.create_function(|_, (_ud, _scale): (LightUserData, f64)| Ok(()))?)?;
    methods.set("GetCamDistanceScale", lua.create_function(|_, _ud: LightUserData| Ok(1.0_f64))?)?;
    methods.set("SetCamera", lua.create_function(|_, (_ud, _cam): (LightUserData, i32)| Ok(()))?)?;
    methods.set("SetPortraitZoom", lua.create_function(|_, (_ud, _zoom): (LightUserData, f64)| Ok(()))?)?;
    methods.set("SetDesaturation", lua.create_function(|_, (_ud, _desat): (LightUserData, f64)| Ok(()))?)?;
    methods.set("SetLight", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;
    methods.set("SetSequence", lua.create_function(|_, (_ud, _seq): (LightUserData, i32)| Ok(()))?)?;
    methods.set("SetSequenceTime", lua.create_function(|_, (_ud, _seq, _time): (LightUserData, i32, i32)| Ok(()))?)?;
    methods.set("ClearModel", lua.create_function(|_, _ud: LightUserData| Ok(()))?)?;
    methods.set("RefreshUnit", lua.create_function(|_, _ud: LightUserData| Ok(()))?)?;
    methods.set("RefreshCamera", lua.create_function(|_, _ud: LightUserData| Ok(()))?)?;
    Ok(())
}

fn add_model_scene_id_methods(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    methods.set("TransitionToModelSceneID", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;
    methods.set("SetFromModelSceneID", lua.create_function(|_, (_ud, _id): (LightUserData, i32)| Ok(()))?)?;
    methods.set("GetModelSceneID", lua.create_function(|_, _ud: LightUserData| Ok(0i32))?)?;
    methods.set("CycleVariation", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;
    methods.set("GetUpperEmblemTexture", lua.create_function(|_, _ud: LightUserData| -> Result<Option<String>> {
        Ok(None)
    })?)?;
    methods.set("GetLowerEmblemTexture", lua.create_function(|_, _ud: LightUserData| -> Result<Option<String>> {
        Ok(None)
    })?)?;
    Ok(())
}

/// Native ModelScene methods (C++ side in WoW, stubs here).
/// The Lua-side logic lives in ModelSceneMixin; these are the engine calls it invokes.
pub fn add_model_scene_methods(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    add_model_scene_rendering_stubs(lua, methods)?;
    add_model_scene_camera_stubs(lua, methods)?;
    add_model_scene_light_stubs(lua, methods)?;
    add_model_scene_fog_stubs(lua, methods)?;
    Ok(())
}

fn add_model_scene_rendering_stubs(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    methods.set("SetAllowOverlappedModels", lua.create_function(|_, (_ud, _allow): (LightUserData, bool)| Ok(()))?)?;
    methods.set("IsAllowOverlappedModels", lua.create_function(|_, _ud: LightUserData| Ok(false))?)?;
    methods.set("SetPaused", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;
    methods.set("Project3DPointTo2D", lua.create_function(
        |_, (_ud, _args): (LightUserData, mlua::MultiValue)| -> Result<(f64, f64, f64)> {
            Ok((0.0, 0.0, 1.0))
        },
    )?)?;
    methods.set("SetViewInsets", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;
    methods.set("GetViewInsets", lua.create_function(|_, _ud: LightUserData| Ok((0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64)))?)?;
    methods.set("GetViewTranslation", lua.create_function(|_, _ud: LightUserData| Ok((0.0_f64, 0.0_f64)))?)?;
    Ok(())
}

fn add_model_scene_camera_stubs(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    methods.set("SetCameraPosition", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;
    methods.set("GetCameraPosition", lua.create_function(|_, _ud: LightUserData| Ok((0.0_f64, 0.0_f64, 0.0_f64)))?)?;
    methods.set("SetCameraOrientationByYawPitchRoll", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;
    methods.set("SetCameraOrientationByAxisVectors", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;
    methods.set("GetCameraForward", lua.create_function(|_, _ud: LightUserData| Ok((0.0_f64, 0.0_f64, 1.0_f64)))?)?;
    methods.set("GetCameraRight", lua.create_function(|_, _ud: LightUserData| Ok((1.0_f64, 0.0_f64, 0.0_f64)))?)?;
    methods.set("GetCameraUp", lua.create_function(|_, _ud: LightUserData| Ok((0.0_f64, 1.0_f64, 0.0_f64)))?)?;
    methods.set("SetCameraFieldOfView", lua.create_function(|_, (_ud, _fov): (LightUserData, f64)| Ok(()))?)?;
    methods.set("GetCameraFieldOfView", lua.create_function(|_, _ud: LightUserData| Ok(0.785_f64))?)?;
    methods.set("SetCameraNearClip", lua.create_function(|_, (_ud, _clip): (LightUserData, f64)| Ok(()))?)?;
    methods.set("SetCameraFarClip", lua.create_function(|_, (_ud, _clip): (LightUserData, f64)| Ok(()))?)?;
    methods.set("GetCameraNearClip", lua.create_function(|_, _ud: LightUserData| Ok(0.1_f64))?)?;
    methods.set("GetCameraFarClip", lua.create_function(|_, _ud: LightUserData| Ok(100.0_f64))?)?;
    Ok(())
}

fn add_model_scene_light_stubs(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    methods.set("SetLightType", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;
    methods.set("SetLightPosition", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;
    methods.set("GetLightPosition", lua.create_function(|_, _ud: LightUserData| Ok((0.0_f64, 0.0_f64, 0.0_f64)))?)?;
    methods.set("SetLightDirection", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;
    methods.set("GetLightDirection", lua.create_function(|_, _ud: LightUserData| Ok((0.0_f64, -1.0_f64, 0.0_f64)))?)?;
    methods.set("SetLightAmbientColor", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;
    methods.set("SetLightDiffuseColor", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;
    methods.set("SetLightVisible", lua.create_function(|_, (_ud, _vis): (LightUserData, bool)| Ok(()))?)?;
    methods.set("IsLightVisible", lua.create_function(|_, _ud: LightUserData| Ok(true))?)?;
    Ok(())
}

fn add_model_scene_fog_stubs(lua: &Lua, methods: &mlua::Table) -> Result<()> {
    methods.set("SetFogNear", lua.create_function(|_, (_ud, _near): (LightUserData, f64)| Ok(()))?)?;
    methods.set("SetFogFar", lua.create_function(|_, (_ud, _far): (LightUserData, f64)| Ok(()))?)?;
    methods.set("SetFogColor", lua.create_function(|_, (_ud, _args): (LightUserData, mlua::MultiValue)| Ok(()))?)?;
    methods.set("ClearFog", lua.create_function(|_, _ud: LightUserData| Ok(()))?)?;
    Ok(())
}
