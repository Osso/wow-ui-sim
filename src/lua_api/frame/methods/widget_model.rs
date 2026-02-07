//! Model and ModelScene widget method stubs.

use super::FrameHandle;
use mlua::{Result, UserDataMethods, Value};

pub fn add_model_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetModel", |_, _this, _path: String| Ok(()));
    methods.add_method("GetModel", |_, _this, ()| -> Result<Option<String>> {
        Ok(None)
    });
    methods.add_method("SetModelScale", |_, _this, _scale: f64| Ok(()));
    methods.add_method("GetModelScale", |_, _this, ()| Ok(1.0_f64));
    methods.add_method("SetPosition", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("GetPosition", |_, _this, ()| {
        Ok((0.0_f64, 0.0_f64, 0.0_f64))
    });
    methods.add_method("SetFacing", |_, _this, _radians: f64| Ok(()));
    methods.add_method("GetFacing", |_, _this, ()| Ok(0.0_f64));
    methods.add_method("SetUnit", |_, _this, _unit: Option<String>| Ok(()));
    methods.add_method("SetAutoDress", |_, _this, _auto_dress: bool| Ok(()));
    methods.add_method("SetDisplayInfo", |_, _this, _display_id: i32| Ok(()));
    methods.add_method("SetCreature", |_, _this, _creature_id: i32| Ok(()));
    methods.add_method("SetAnimation", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("SetCamDistanceScale", |_, _this, _scale: f64| Ok(()));
    methods.add_method("GetCamDistanceScale", |_, _this, ()| Ok(1.0_f64));
    methods.add_method("SetCamera", |_, _this, _camera_id: i32| Ok(()));
    methods.add_method("SetPortraitZoom", |_, _this, _zoom: f64| Ok(()));
    methods.add_method("SetDesaturation", |_, _this, _desat: f64| Ok(()));
    methods.add_method("SetRotation", |_, _this, _radians: Value| Ok(()));
    methods.add_method("SetLight", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("SetSequence", |_, _this, _sequence: i32| Ok(()));
    methods.add_method("SetSequenceTime", |_, _this, (_seq, _time): (i32, i32)| {
        Ok(())
    });
    methods.add_method("ClearModel", |_, _this, ()| Ok(()));
    methods.add_method(
        "TransitionToModelSceneID",
        |_, _this, _args: mlua::MultiValue| Ok(()),
    );
    methods.add_method("SetFromModelSceneID", |_, _this, _scene_id: i32| Ok(()));
    methods.add_method("GetModelSceneID", |_, _this, ()| Ok(0i32));
    methods.add_method("CycleVariation", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("GetUpperEmblemTexture", |_, _this, ()| -> Result<Option<String>> {
        Ok(None)
    });
    methods.add_method("GetLowerEmblemTexture", |_, _this, ()| -> Result<Option<String>> {
        Ok(None)
    });
    methods.add_method("RefreshUnit", |_, _this, ()| Ok(()));
    methods.add_method("RefreshCamera", |_, _this, ()| Ok(()));
}

/// Native ModelScene methods (C++ side in WoW, stubs here).
/// The Lua-side logic lives in ModelSceneMixin; these are the engine calls it invokes.
pub fn add_model_scene_methods<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    add_model_scene_rendering_stubs(methods);
    add_model_scene_camera_stubs(methods);
    add_model_scene_light_stubs(methods);
    add_model_scene_fog_stubs(methods);
}

fn add_model_scene_rendering_stubs<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetAllowOverlappedModels", |_, _this, _allow: bool| Ok(()));
    methods.add_method("IsAllowOverlappedModels", |_, _this, ()| Ok(false));
    methods.add_method("SetPaused", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("SetDrawLayer", |_, _this, _args: mlua::MultiValue| Ok(()));
    // Project3DPointTo2D(x, y, z) -> screenX, screenY, depthScale
    methods.add_method(
        "Project3DPointTo2D",
        |_, _this, _args: mlua::MultiValue| -> Result<(f64, f64, f64)> {
            Ok((0.0, 0.0, 1.0))
        },
    );
    methods.add_method("SetViewInsets", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("GetViewInsets", |_, _this, ()| {
        Ok((0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64))
    });
    methods.add_method("GetViewTranslation", |_, _this, ()| {
        Ok((0.0_f64, 0.0_f64))
    });
}

fn add_model_scene_camera_stubs<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetCameraPosition", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("GetCameraPosition", |_, _this, ()| {
        Ok((0.0_f64, 0.0_f64, 0.0_f64))
    });
    methods.add_method("SetCameraOrientationByYawPitchRoll", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("SetCameraOrientationByAxisVectors", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("GetCameraForward", |_, _this, ()| Ok((0.0_f64, 0.0_f64, 1.0_f64)));
    methods.add_method("GetCameraRight", |_, _this, ()| Ok((1.0_f64, 0.0_f64, 0.0_f64)));
    methods.add_method("GetCameraUp", |_, _this, ()| Ok((0.0_f64, 1.0_f64, 0.0_f64)));
    methods.add_method("SetCameraFieldOfView", |_, _this, _fov: f64| Ok(()));
    methods.add_method("GetCameraFieldOfView", |_, _this, ()| Ok(0.785_f64));
    methods.add_method("SetCameraNearClip", |_, _this, _clip: f64| Ok(()));
    methods.add_method("SetCameraFarClip", |_, _this, _clip: f64| Ok(()));
    methods.add_method("GetCameraNearClip", |_, _this, ()| Ok(0.1_f64));
    methods.add_method("GetCameraFarClip", |_, _this, ()| Ok(100.0_f64));
}

fn add_model_scene_light_stubs<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetLightType", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("SetLightPosition", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("GetLightPosition", |_, _this, ()| Ok((0.0_f64, 0.0_f64, 0.0_f64)));
    methods.add_method("SetLightDirection", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("GetLightDirection", |_, _this, ()| Ok((0.0_f64, -1.0_f64, 0.0_f64)));
    methods.add_method("SetLightAmbientColor", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("SetLightDiffuseColor", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("SetLightVisible", |_, _this, _visible: bool| Ok(()));
    methods.add_method("IsLightVisible", |_, _this, ()| Ok(true));
}

fn add_model_scene_fog_stubs<M: UserDataMethods<FrameHandle>>(methods: &mut M) {
    methods.add_method("SetFogNear", |_, _this, _near: f64| Ok(()));
    methods.add_method("SetFogFar", |_, _this, _far: f64| Ok(()));
    methods.add_method("SetFogColor", |_, _this, _args: mlua::MultiValue| Ok(()));
    methods.add_method("ClearFog", |_, _this, ()| Ok(()));
}
