//! Integration tests for `ModelScene:SetResetCallback(callback)`.
//!
//! Surfaced by `AlliedRacesFrameMixin:OnShow`
//! (`Blizzard_AlliedRacesFrameUI.lua:98`), which calls
//! `self.ModelScene:SetResetCallback(GenerateClosure(self.OnModelSceneReset, self))`.
//! The Blizzard `ModelSceneMixin:Reset()` handler reads `self.resetCallback`
//! and invokes it with the scene, so the widget method must persist the
//! callback in a place `self.resetCallback` can later read.

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn set_reset_callback_method_exists_on_model_scene() {
    let env = WowLuaEnv::new().expect("env");
    let kind: String = env
        .eval(
            r#"
            local scene = CreateFrame("ModelScene", "TestSceneMethodExists")
            return type(scene.SetResetCallback)
            "#,
        )
        .unwrap();
    assert_eq!(kind, "function");
}

#[test]
fn set_reset_callback_persists_callback_for_blizzard_reset() {
    let env = WowLuaEnv::new().expect("env");
    env.exec(
        r#"
            local scene = CreateFrame("ModelScene", "TestSceneStoresCallback")
            local function cb(self) end
            scene:SetResetCallback(cb)
            STORED_TYPE = type(scene.resetCallback)
            STORED_MATCHES = (scene.resetCallback == cb)
        "#,
    )
    .unwrap();
    let stored_type: String = env.eval("return STORED_TYPE").unwrap();
    let stored_matches: bool = env.eval("return STORED_MATCHES").unwrap();
    assert_eq!(stored_type, "function");
    assert!(
        stored_matches,
        "scene.resetCallback should be the exact closure handed to SetResetCallback"
    );
}

#[test]
fn set_reset_callback_accepts_nil_to_clear() {
    let env = WowLuaEnv::new().expect("env");
    env.exec(
        r#"
            local scene = CreateFrame("ModelScene", "TestSceneClearCallback")
            scene:SetResetCallback(function() end)
            scene:SetResetCallback(nil)
            CLEARED = (scene.resetCallback == nil)
        "#,
    )
    .unwrap();
    let cleared: bool = env.eval("return CLEARED").unwrap();
    assert!(
        cleared,
        "passing nil to SetResetCallback should clear the stored callback"
    );
}

#[test]
fn set_reset_callback_ignores_non_function_arguments() {
    let env = WowLuaEnv::new().expect("env");
    env.exec(
        r#"
            local scene = CreateFrame("ModelScene", "TestSceneIgnoresGarbage")
            local function cb() end
            scene:SetResetCallback(cb)
            scene:SetResetCallback(42)
            scene:SetResetCallback("not a function")
            scene:SetResetCallback({})
            STILL_CALLBACK = (scene.resetCallback == cb)
        "#,
    )
    .unwrap();
    let still_callback: bool = env.eval("return STILL_CALLBACK").unwrap();
    assert!(
        still_callback,
        "non-function/non-nil arguments must be ignored, leaving the previous callback intact"
    );
}

#[test]
fn set_reset_callback_is_per_frame() {
    let env = WowLuaEnv::new().expect("env");
    env.exec(
        r#"
            local a = CreateFrame("ModelScene", "TestSceneA")
            local b = CreateFrame("ModelScene", "TestSceneB")
            local function cb_a() return "a" end
            local function cb_b() return "b" end
            a:SetResetCallback(cb_a)
            b:SetResetCallback(cb_b)
            A_MATCH = (a.resetCallback == cb_a)
            B_MATCH = (b.resetCallback == cb_b)
            CROSS_TALK = (a.resetCallback == b.resetCallback)
        "#,
    )
    .unwrap();
    let a_match: bool = env.eval("return A_MATCH").unwrap();
    let b_match: bool = env.eval("return B_MATCH").unwrap();
    let cross_talk: bool = env.eval("return CROSS_TALK").unwrap();
    assert!(a_match, "frame A should retain its own callback");
    assert!(b_match, "frame B should retain its own callback");
    assert!(
        !cross_talk,
        "callbacks on separate frames must not share storage"
    );
}

#[test]
fn allied_races_on_show_pattern_executes_without_error() {
    // Mirrors AlliedRacesFrameMixin:OnShow line 98 — the closure is
    // generated on the fly and immediately handed to SetResetCallback. If
    // the method is missing or rejects valid input, this script throws.
    let env = shared_xml_base_env();
    env.exec(
        r#"
            local scene = CreateFrame("ModelScene", "TestSceneOnShow")
            local owner = { reset_count = 0 }
            function owner:OnModelSceneReset() self.reset_count = self.reset_count + 1 end
            scene:SetResetCallback(GenerateClosure(owner.OnModelSceneReset, owner))
            scene.resetCallback(scene)
            FIRED = owner.reset_count
        "#,
    )
    .unwrap();
    let fired: f64 = env.eval("return FIRED").unwrap();
    assert_eq!(
        fired, 1.0,
        "stored closure must be callable and bound to the original owner"
    );
}

/// GenerateClosure is owned by Blizzard_SharedXMLBase FunctionUtil.lua since
/// 096d8a1f8c retired the bootstrap copy.
fn shared_xml_base_env() -> WowLuaEnv {
    use crate::common::blizzard_addon_harness::{
        blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
    };
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_SharedXMLBase"], &[]);
    env
}
