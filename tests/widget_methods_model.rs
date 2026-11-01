//! Tests for PlayerModel, DressUpModel, and ModelScene widget methods.

use wow_ui_sim::lua_api::WowLuaEnv;

fn frame_storage_estimate(env: &WowLuaEnv, name: &str) -> usize {
    let state = env.state().borrow();
    let id = state
        .widgets
        .get_id_by_name(name)
        .unwrap_or_else(|| panic!("missing frame {name}"));
    state.widgets.get(id).unwrap().storage_estimate_bytes()
}

#[test]
fn model_storage_defaults_do_not_allocate_until_non_default_mutation() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("CreateFrame('Frame', 'OrdinaryModelMethodFrame', UIParent)")
        .unwrap();
    let initial = frame_storage_estimate(&env, "OrdinaryModelMethodFrame");

    env.exec(
        r#"
        local frame = OrdinaryModelMethodFrame
        local model = frame:GetModel()
        local scale = frame:GetModelScale()
        local x, y, z = frame:GetPosition()
        local facing = frame:GetFacing()
        local displayInfo = frame:GetDisplayInfo()
        local fileID = frame:GetModelFileID()
        local alpha = frame:GetModelAlpha()
        local blend = frame:GetDoBlend()
        local keep = frame:GetKeepModelOnHide()
        local left, right, top, bottom = frame:GetViewInsets()
        assert(model == "")
        assert(scale == 1 and x == 0 and y == 0 and z == 0 and facing == 0)
        assert(displayInfo == 0 and fileID == 0 and alpha == 1)
        assert(blend == false and keep == false)
        assert(left == 0 and right == 0 and top == 0 and bottom == 0)
        "#,
    )
    .unwrap();

    let after_default_reads = frame_storage_estimate(&env, "OrdinaryModelMethodFrame");
    assert_eq!(
        after_default_reads, initial,
        "default model getters should not allocate persistent state"
    );
    env.exec(
        r#"
        OrdinaryModelMethodFrame:SetModelScale(1)
        OrdinaryModelMethodFrame:SetPosition(0, 0, 0)
        OrdinaryModelMethodFrame:SetFacing(0)
        OrdinaryModelMethodFrame:SetModelAlpha(1)
        OrdinaryModelMethodFrame:SetDoBlend(false)
        OrdinaryModelMethodFrame:SetKeepModelOnHide(false)
        OrdinaryModelMethodFrame:SetViewInsets(0, 0, 0, 0)
        OrdinaryModelMethodFrame:ClearModel()
        OrdinaryModelMethodFrame:ClearScene()
        "#,
    )
    .unwrap();
    assert_eq!(
        frame_storage_estimate(&env, "OrdinaryModelMethodFrame"),
        after_default_reads,
        "default-valued model operations should not allocate persistent state"
    );

    env.exec("OrdinaryModelMethodFrame:SetModelScale(2)")
        .unwrap();
    assert!(
        frame_storage_estimate(&env, "OrdinaryModelMethodFrame") > after_default_reads,
        "first non-default model mutation should allocate persistent state"
    );
}

#[test]
fn model_storage_counts_scene_actor_tags() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local untagged = CreateFrame("ModelScene", "UntaggedActorScene", UIParent)
        local tagged = CreateFrame("ModelScene", "TaggedActorScene", UIParent)
        untagged:CreateActor("")
        tagged:CreateActor("storage-accounting-actor-tag")
        "#,
    )
    .unwrap();

    assert!(
        frame_storage_estimate(&env, "TaggedActorScene")
            > frame_storage_estimate(&env, "UntaggedActorScene"),
        "actor tag vector and string storage should be counted"
    );
}

#[test]
fn model_storage_counts_player_model_owned_strings() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local model = CreateFrame("PlayerModel", "PlayerModelStorage", UIParent)
        model:SetModelScale(2)
        "#,
    )
    .unwrap();
    let payload_only = frame_storage_estimate(&env, "PlayerModelStorage");

    env.exec(
        r#"
        PlayerModelStorage:SetItem("item:19019:storage-accounting")
        PlayerModelStorage:SetItemAppearance("appearance:123456:storage-accounting")
        PlayerModelStorage:SetUnit("player-storage-accounting")
        "#,
    )
    .unwrap();

    assert!(
        frame_storage_estimate(&env, "PlayerModelStorage") > payload_only,
        "player-model owned strings should be counted"
    );
}

#[test]
fn test_player_model_methods_still_resolve() {
    let env = WowLuaEnv::new().unwrap();

    let result: (bool, bool, bool) = env
        .eval(
            r#"
            local pm = CreateFrame("PlayerModel", "TestPlayerModelMethods", UIParent)
            return type(pm.ApplySpellVisualKit) == "function",
                   type(pm.SetKeepModelOnHide) == "function",
                   type(pm.GetDisplayInfo) == "function"
            "#,
        )
        .unwrap();

    assert!(result.0, "PlayerModel should expose ApplySpellVisualKit");
    assert!(result.1, "PlayerModel should expose SetKeepModelOnHide");
    assert!(result.2, "PlayerModel should expose GetDisplayInfo");
}

#[test]
fn test_model_and_model_scene_clear_fog_methods_absorb_visual_reset() {
    let env = WowLuaEnv::new().unwrap();

    let (model_method, model_call, scene_method, scene_call): (bool, bool, bool, bool) = env
        .eval(
            r#"
            local model = CreateFrame("Model")
            local scene = CreateFrame("ModelScene")
            return type(model.ClearFog) == "function",
                   pcall(model.ClearFog, model),
                   type(scene.ClearFog) == "function",
                   pcall(scene.ClearFog, scene)
            "#,
        )
        .unwrap();

    assert!(model_method, "Model should expose ClearFog");
    assert!(model_call, "Model:ClearFog should absorb the visual reset");
    assert!(scene_method, "ModelScene should expose ClearFog");
    assert!(
        scene_call,
        "ModelScene:ClearFog should absorb the visual reset"
    );
}

#[test]
fn test_player_model_set_model_persists_path_and_clears_file_id() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local pm = CreateFrame("PlayerModel", "TestPlayerModelSetModel", UIParent)
        pm:SetModel("Creature/Dragon/Dragon.m2")
    "#,
    )
    .unwrap();

    let model_path: String = env
        .eval("return TestPlayerModelSetModel:GetModel()")
        .unwrap();
    assert_eq!(model_path, "Creature/Dragon/Dragon.m2");

    let model_id = env
        .state()
        .borrow()
        .widgets
        .get_id_by_name("TestPlayerModelSetModel")
        .unwrap();
    let state = env.state().borrow();
    let frame = state.widgets.get(model_id).unwrap();
    assert_eq!(
        frame.model_state().model_path.as_deref(),
        Some("Creature/Dragon/Dragon.m2")
    );
    assert_eq!(frame.model_state().model_file_id, None);
}

#[test]
fn test_player_model_transform_and_camera_methods_persist_state() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local pm = CreateFrame("PlayerModel", "TestPlayerModelTransformCamera", UIParent)
        pm:SetModelScale(1.75)
        pm:SetPosition(10.5, -2.25, 8.0)
        pm:SetFacing(1.125)
        pm:SetCameraDistance(23.5)
        pm:SetCameraFacing(0.875)
        pm:SetCameraTarget(4.0, 5.5, -6.25)
        pm:SetCameraRoll(0.375)
    "#,
    )
    .unwrap();

    let model_scale: f64 = env
        .eval("return TestPlayerModelTransformCamera:GetModelScale()")
        .unwrap();
    let position: (f64, f64, f64) = env
        .eval("return TestPlayerModelTransformCamera:GetPosition()")
        .unwrap();
    let facing: f64 = env
        .eval("return TestPlayerModelTransformCamera:GetFacing()")
        .unwrap();
    let camera_distance: f64 = env
        .eval("return TestPlayerModelTransformCamera:GetCameraDistance()")
        .unwrap();
    let camera_facing: f64 = env
        .eval("return TestPlayerModelTransformCamera:GetCameraFacing()")
        .unwrap();
    let camera_target: (f64, f64, f64) = env
        .eval("return TestPlayerModelTransformCamera:GetCameraTarget()")
        .unwrap();
    let camera_roll: f64 = env
        .eval("return TestPlayerModelTransformCamera:GetCameraRoll()")
        .unwrap();

    assert!((model_scale - 1.75).abs() < 0.001);
    assert!((position.0 - 10.5).abs() < 0.001);
    assert!((position.1 + 2.25).abs() < 0.001);
    assert!((position.2 - 8.0).abs() < 0.001);
    assert!((facing - 1.125).abs() < 0.001);
    assert!((camera_distance - 23.5).abs() < 0.001);
    assert!((camera_facing - 0.875).abs() < 0.001);
    assert!((camera_target.0 - 4.0).abs() < 0.001);
    assert!((camera_target.1 - 5.5).abs() < 0.001);
    assert!((camera_target.2 + 6.25).abs() < 0.001);
    assert!((camera_roll - 0.375).abs() < 0.001);

    let model_id = env
        .state()
        .borrow()
        .widgets
        .get_id_by_name("TestPlayerModelTransformCamera")
        .unwrap();
    let state = env.state().borrow();
    let frame = state.widgets.get(model_id).unwrap();
    assert!((frame.model_state().model_transform.scale - 1.75).abs() < 0.001);
    assert!((frame.model_state().model_transform.position.0 - 10.5).abs() < 0.001);
    assert!((frame.model_state().model_transform.position.1 + 2.25).abs() < 0.001);
    assert!((frame.model_state().model_transform.position.2 - 8.0).abs() < 0.001);
    assert!((frame.model_state().model_transform.facing - 1.125).abs() < 0.001);
    assert!((frame.model_state().model_transform.camera.distance - 23.5).abs() < 0.001);
    assert!((frame.model_state().model_transform.camera.facing - 0.875).abs() < 0.001);
    assert!((frame.model_state().model_transform.camera.target.0 - 4.0).abs() < 0.001);
    assert!((frame.model_state().model_transform.camera.target.1 - 5.5).abs() < 0.001);
    assert!((frame.model_state().model_transform.camera.target.2 + 6.25).abs() < 0.001);
    assert!((frame.model_state().model_transform.camera.roll - 0.375).abs() < 0.001);
}

#[test]
fn test_player_model_appearance_and_state_methods_persist_and_clear_state() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local pm = CreateFrame("PlayerModel", "TestPlayerModelAppearanceState", UIParent)
        pm:SetModel("Creature/Dragon/Dragon.m2")
        pm:SetDisplayInfo(1234)
    "#,
    )
    .unwrap();

    let display_info: i64 = env
        .eval("return TestPlayerModelAppearanceState:GetDisplayInfo()")
        .unwrap();
    assert_eq!(display_info, 1234);

    let model_id = env
        .state()
        .borrow()
        .widgets
        .get_id_by_name("TestPlayerModelAppearanceState")
        .unwrap();

    {
        let state = env.state().borrow();
        let frame = state.widgets.get(model_id).unwrap();
        assert_eq!(frame.model_state().model_path, None);
        assert_eq!(frame.model_state().model_file_id, None);
        assert_eq!(frame.model_state().model_appearance.display_info, Some(1234));
        assert_eq!(frame.model_state().model_appearance.creature_id, None);
    }

    env.exec(
        r#"
        TestPlayerModelAppearanceState:SetCreature(5678)
        TestPlayerModelAppearanceState:SetAnimation(42)
        TestPlayerModelAppearanceState:SetSequence(7)
        TestPlayerModelAppearanceState:RefreshUnit()
        TestPlayerModelAppearanceState:RefreshCamera()
    "#,
    )
    .unwrap();

    let has_animation: bool = env
        .eval("return TestPlayerModelAppearanceState:HasAnimation()")
        .unwrap();
    assert!(
        has_animation,
        "SetAnimation should make HasAnimation return true"
    );

    {
        let state = env.state().borrow();
        let frame = state.widgets.get(model_id).unwrap();
        assert_eq!(frame.model_state().model_appearance.display_info, None);
        assert_eq!(frame.model_state().model_appearance.creature_id, Some(5678));
        assert_eq!(frame.model_state().model_appearance.animation_id, Some(42));
        assert_eq!(frame.model_state().model_appearance.sequence_id, Some(7));
        assert_eq!(frame.model_state().model_appearance.sequence_time_ms, None);
        assert_eq!(frame.model_state().model_appearance.refresh_unit_count, 1);
        assert_eq!(frame.model_state().model_appearance.refresh_camera_count, 1);
    }

    env.exec("TestPlayerModelAppearanceState:SetSequenceTime(7, 250)")
        .unwrap();

    {
        let state = env.state().borrow();
        let frame = state.widgets.get(model_id).unwrap();
        assert_eq!(frame.model_state().model_appearance.sequence_id, Some(7));
        assert_eq!(frame.model_state().model_appearance.sequence_time_ms, Some(250));
    }

    env.exec("TestPlayerModelAppearanceState:ClearModel()")
        .unwrap();

    let cleared: (i64, String, bool) = env
        .eval(
            r#"
            return TestPlayerModelAppearanceState:GetDisplayInfo(),
                   TestPlayerModelAppearanceState:GetModel(),
                   TestPlayerModelAppearanceState:HasAnimation()
        "#,
        )
        .unwrap();
    assert_eq!(cleared.0, 0);
    assert_eq!(cleared.1, "");
    assert!(
        !cleared.2,
        "ClearModel should clear the active animation state"
    );

    {
        let state = env.state().borrow();
        let frame = state.widgets.get(model_id).unwrap();
        assert_eq!(frame.model_state().model_path, None);
        assert_eq!(frame.model_state().model_file_id, None);
        assert_eq!(frame.model_state().model_appearance.display_info, None);
        assert_eq!(frame.model_state().model_appearance.creature_id, None);
        assert_eq!(frame.model_state().model_appearance.animation_id, None);
        assert_eq!(frame.model_state().model_appearance.sequence_id, None);
        assert_eq!(frame.model_state().model_appearance.sequence_time_ms, None);
        assert_eq!(frame.model_state().model_appearance.refresh_unit_count, 1);
        assert_eq!(frame.model_state().model_appearance.refresh_camera_count, 1);
    }
}

#[test]
fn test_player_model_rendering_flag_methods_persist_state() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local pm = CreateFrame("PlayerModel", "TestPlayerModelRenderingFlags", UIParent)
        pm:SetModelAlpha(0.35)
        pm:SetShadowEffect(0.8)
        pm:SetParticlesEnabled(true)
        pm:SetUseGBuffer(true)
    "#,
    )
    .unwrap();

    let render_state: (f64, f64) = env
        .eval(
            r#"
            return TestPlayerModelRenderingFlags:GetModelAlpha(),
                   TestPlayerModelRenderingFlags:GetShadowEffect()
        "#,
        )
        .unwrap();
    assert!((render_state.0 - 0.35).abs() < 0.001);
    assert!((render_state.1 - 0.8).abs() < 0.001);

    let model_id = env
        .state()
        .borrow()
        .widgets
        .get_id_by_name("TestPlayerModelRenderingFlags")
        .unwrap();
    let state = env.state().borrow();
    let frame = state.widgets.get(model_id).unwrap();
    assert!((frame.model_state().model_rendering.alpha - 0.35).abs() < 0.001);
    assert!((frame.model_state().model_rendering.shadow_effect - 0.8).abs() < 0.001);
    assert!(frame.model_state().model_rendering.particles_enabled);
    assert!(frame.model_state().model_rendering.use_gbuffer);
}

#[test]
fn test_player_model_specific_methods_persist_state() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local pm = CreateFrame("PlayerModel", "TestPlayerModelSpecificState", UIParent)
        pm:SetDoBlend(true)
        pm:SetKeepModelOnHide(true)
        pm:SetItem(19019)
        pm:SetItemAppearance(4242)
        pm:PlayAnimKit(777)
    "#,
    )
    .unwrap();

    let lua_state: (bool, bool, bool) = env
        .eval(
            r#"
            return TestPlayerModelSpecificState:CanSetUnit(),
                   TestPlayerModelSpecificState:GetDoBlend(),
                   TestPlayerModelSpecificState:GetKeepModelOnHide()
        "#,
        )
        .unwrap();
    assert!(
        lua_state.0,
        "PlayerModel should report unit assignment support"
    );
    assert!(
        lua_state.1,
        "SetDoBlend should round-trip through GetDoBlend"
    );
    assert!(
        lua_state.2,
        "SetKeepModelOnHide should round-trip through GetKeepModelOnHide"
    );

    let model_id = env
        .state()
        .borrow()
        .widgets
        .get_id_by_name("TestPlayerModelSpecificState")
        .unwrap();

    {
        let state = env.state().borrow();
        let frame = state.widgets.get(model_id).unwrap();
        assert!(frame.model_state().player_model_state.do_blend);
        assert!(frame.model_state().player_model_state.keep_model_on_hide);
        assert_eq!(frame.model_state().player_model_state.last_item.as_deref(), Some("19019"));
        assert_eq!(
            frame.model_state().player_model_state.last_item_appearance.as_deref(),
            Some("4242")
        );
        assert_eq!(frame.model_state().player_model_state.active_anim_kit, Some(777));
    }

    env.exec("TestPlayerModelSpecificState:StopAnimKit()")
        .unwrap();

    let state = env.state().borrow();
    let frame = state.widgets.get(model_id).unwrap();
    assert_eq!(frame.model_state().player_model_state.active_anim_kit, None);
}

#[test]
fn test_model_scene_camera_light_and_fog_methods_persist_state() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local scene = CreateFrame("ModelScene", "TestModelSceneState", UIParent)
        scene:SetCameraPosition(1.5, -2.25, 3.75)
        scene:SetCameraOrientationByAxisVectors(0, 0, 1, 1, 0, 0, 0, 1, 0)
        scene:SetCameraFieldOfView(1.125)
        scene:SetCameraNearClip(0.25)
        scene:SetCameraFarClip(250.0)
        scene:SetLightType(2)
        scene:SetLightPosition(4.5, 5.5, -6.5)
        scene:SetLightDirection(0.1, -0.2, 0.3)
        scene:SetLightAmbientColor(0.11, 0.22, 0.33)
        scene:SetLightDiffuseColor(0.44, 0.55, 0.66)
        scene:SetLightVisible(false)
        scene:SetFogNear(7.5)
        scene:SetFogFar(8.5)
        scene:SetFogColor(0.7, 0.8, 0.9)
        scene:SetPaused(true, false)
        scene:SetViewInsets(10, 20, 30, 40)
    "#,
    )
    .unwrap();

    let camera_position: (f64, f64, f64) = env
        .eval("return TestModelSceneState:GetCameraPosition()")
        .unwrap();
    let camera_forward: (f64, f64, f64) = env
        .eval("return TestModelSceneState:GetCameraForward()")
        .unwrap();
    let camera_right: (f64, f64, f64) = env
        .eval("return TestModelSceneState:GetCameraRight()")
        .unwrap();
    let camera_up: (f64, f64, f64) = env
        .eval("return TestModelSceneState:GetCameraUp()")
        .unwrap();
    let field_of_view: f64 = env
        .eval("return TestModelSceneState:GetCameraFieldOfView()")
        .unwrap();
    let near_clip: f64 = env
        .eval("return TestModelSceneState:GetCameraNearClip()")
        .unwrap();
    let far_clip: f64 = env
        .eval("return TestModelSceneState:GetCameraFarClip()")
        .unwrap();
    let light_type: i64 = env
        .eval("return TestModelSceneState:GetLightType()")
        .unwrap();
    let light_position: (f64, f64, f64) = env
        .eval("return TestModelSceneState:GetLightPosition()")
        .unwrap();
    let light_direction: (f64, f64, f64) = env
        .eval("return TestModelSceneState:GetLightDirection()")
        .unwrap();
    let ambient_color: (f64, f64, f64) = env
        .eval("return TestModelSceneState:GetLightAmbientColor()")
        .unwrap();
    let diffuse_color: (f64, f64, f64) = env
        .eval("return TestModelSceneState:GetLightDiffuseColor()")
        .unwrap();
    let light_visible: bool = env
        .eval("return TestModelSceneState:IsLightVisible()")
        .unwrap();
    let fog_near: f64 = env.eval("return TestModelSceneState:GetFogNear()").unwrap();
    let fog_far: f64 = env.eval("return TestModelSceneState:GetFogFar()").unwrap();
    let fog_color: (f64, f64, f64) = env
        .eval("return TestModelSceneState:GetFogColor()")
        .unwrap();
    let paused: bool = env.eval("return TestModelSceneState:GetPaused()").unwrap();
    let view_insets: (f64, f64, f64, f64) = env
        .eval("return TestModelSceneState:GetViewInsets()")
        .unwrap();

    assert!((camera_position.0 - 1.5).abs() < 0.001);
    assert!((camera_position.1 + 2.25).abs() < 0.001);
    assert!((camera_position.2 - 3.75).abs() < 0.001);
    assert_eq!(camera_forward, (0.0, 0.0, 1.0));
    assert_eq!(camera_right, (1.0, 0.0, 0.0));
    assert_eq!(camera_up, (0.0, 1.0, 0.0));
    assert!((field_of_view - 1.125).abs() < 0.001);
    assert!((near_clip - 0.25).abs() < 0.001);
    assert!((far_clip - 250.0).abs() < 0.001);
    assert_eq!(light_type, 2);
    assert!((light_position.0 - 4.5).abs() < 0.001);
    assert!((light_position.1 - 5.5).abs() < 0.001);
    assert!((light_position.2 + 6.5).abs() < 0.001);
    assert!((light_direction.0 - 0.1).abs() < 0.001);
    assert!((light_direction.1 + 0.2).abs() < 0.001);
    assert!((light_direction.2 - 0.3).abs() < 0.001);
    assert!((ambient_color.0 - 0.11).abs() < 0.001);
    assert!((ambient_color.1 - 0.22).abs() < 0.001);
    assert!((ambient_color.2 - 0.33).abs() < 0.001);
    assert!((diffuse_color.0 - 0.44).abs() < 0.001);
    assert!((diffuse_color.1 - 0.55).abs() < 0.001);
    assert!((diffuse_color.2 - 0.66).abs() < 0.001);
    assert!(!light_visible);
    assert!((fog_near - 7.5).abs() < 0.001);
    assert!((fog_far - 8.5).abs() < 0.001);
    assert!((fog_color.0 - 0.7).abs() < 0.001);
    assert!((fog_color.1 - 0.8).abs() < 0.001);
    assert!((fog_color.2 - 0.9).abs() < 0.001);
    assert!(paused);
    assert_eq!(view_insets, (10.0, 20.0, 30.0, 40.0));

    let scene_id = env
        .state()
        .borrow()
        .widgets
        .get_id_by_name("TestModelSceneState")
        .unwrap();
    let state = env.state().borrow();
    let frame = state.widgets.get(scene_id).unwrap();
    assert!((frame.model_state().model_scene_state.camera.position.0 - 1.5).abs() < 0.001);
    assert!((frame.model_state().model_scene_state.camera.position.1 + 2.25).abs() < 0.001);
    assert!((frame.model_state().model_scene_state.camera.position.2 - 3.75).abs() < 0.001);
    assert_eq!(frame.model_state().model_scene_state.camera.forward, (0.0, 0.0, 1.0));
    assert_eq!(frame.model_state().model_scene_state.camera.right, (1.0, 0.0, 0.0));
    assert_eq!(frame.model_state().model_scene_state.camera.up, (0.0, 1.0, 0.0));
    assert!((frame.model_state().model_scene_state.camera.field_of_view - 1.125).abs() < 0.001);
    assert!((frame.model_state().model_scene_state.camera.near_clip - 0.25).abs() < 0.001);
    assert!((frame.model_state().model_scene_state.camera.far_clip - 250.0).abs() < 0.001);
    assert_eq!(frame.model_state().model_scene_state.light.light_type, 2);
    assert!((frame.model_state().model_scene_state.light.position.0 - 4.5).abs() < 0.001);
    assert!((frame.model_state().model_scene_state.light.position.1 - 5.5).abs() < 0.001);
    assert!((frame.model_state().model_scene_state.light.position.2 + 6.5).abs() < 0.001);
    assert!((frame.model_state().model_scene_state.light.direction.0 - 0.1).abs() < 0.001);
    assert!((frame.model_state().model_scene_state.light.direction.1 + 0.2).abs() < 0.001);
    assert!((frame.model_state().model_scene_state.light.direction.2 - 0.3).abs() < 0.001);
    assert!((frame.model_state().model_scene_state.light.ambient_color.r - 0.11).abs() < 0.001);
    assert!((frame.model_state().model_scene_state.light.ambient_color.g - 0.22).abs() < 0.001);
    assert!((frame.model_state().model_scene_state.light.ambient_color.b - 0.33).abs() < 0.001);
    assert!((frame.model_state().model_scene_state.light.diffuse_color.r - 0.44).abs() < 0.001);
    assert!((frame.model_state().model_scene_state.light.diffuse_color.g - 0.55).abs() < 0.001);
    assert!((frame.model_state().model_scene_state.light.diffuse_color.b - 0.66).abs() < 0.001);
    assert!(!frame.model_state().model_scene_state.light.visible);
    assert!((frame.model_state().model_scene_state.fog.near - 7.5).abs() < 0.001);
    assert!((frame.model_state().model_scene_state.fog.far - 8.5).abs() < 0.001);
    assert!((frame.model_state().model_scene_state.fog.color.r - 0.7).abs() < 0.001);
    assert!((frame.model_state().model_scene_state.fog.color.g - 0.8).abs() < 0.001);
    assert!((frame.model_state().model_scene_state.fog.color.b - 0.9).abs() < 0.001);
    assert!(frame.model_state().model_scene_state.paused);
    assert_eq!(
        frame.model_state().model_scene_state.view_insets,
        (10.0, 20.0, 30.0, 40.0)
    );
}

#[test]
fn test_model_scene_overlap_flag_persists_state() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local scene = CreateFrame("ModelScene", "TestModelSceneOverlap", UIParent)
        scene:SetAllowOverlappedModels(true)
    "#,
    )
    .unwrap();

    let allow: bool = env
        .eval("return TestModelSceneOverlap:IsAllowOverlappedModels()")
        .unwrap();
    assert!(allow);

    let scene_id = env
        .state()
        .borrow()
        .widgets
        .get_id_by_name("TestModelSceneOverlap")
        .unwrap();
    let state = env.state().borrow();
    let frame = state.widgets.get(scene_id).unwrap();
    assert!(frame.model_state().model_scene_state.allow_overlapped_models);
}

#[test]
fn test_model_scene_project_3d_point_uses_camera_projection() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local scene = CreateFrame("ModelScene", "TestModelSceneProjection", UIParent)
        scene:SetSize(400, 200)
        scene:SetCameraPosition(1.0, 2.0, 3.0)
        scene:SetCameraFieldOfView(1.0)
        scene:SetViewInsets(10, 20, 30, 40)
        scene:SetViewTranslation(12, -6)
    "#,
    )
    .unwrap();

    let center: (f64, f64, f64) = env
        .eval("return TestModelSceneProjection:Project3DPointTo2D(1.0, 2.0, 13.0)")
        .unwrap();
    let offset: (f64, f64, f64) = env
        .eval("return TestModelSceneProjection:Project3DPointTo2D(3.0, 4.0, 13.0)")
        .unwrap();
    let behind: rilua::Val = env
        .eval("return TestModelSceneProjection:Project3DPointTo2D(1.0, 2.0, 2.0)")
        .unwrap();

    assert!((center.0 - 197.0).abs() < 0.001);
    assert!((center.1 - 59.0).abs() < 0.001);
    assert!((center.2 - 0.9009009).abs() < 0.001);
    assert!((offset.0 - 220.796340).abs() < 0.001);
    assert!((offset.1 - 82.796340).abs() < 0.001);
    assert!((offset.2 - 0.9009009).abs() < 0.001);
    assert!(matches!(behind, rilua::Val::Nil));
}

#[test]
fn test_model_scene_get_player_actor_returns_reusable_stub_actor() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local scene = CreateFrame("ModelScene", "TestModelScenePlayerActor", UIParent)
        local actor1 = scene:GetPlayerActor()
        local actor2 = scene:GetPlayerActor()

        _G.player_actor_state = {
            actor_exists = actor1 ~= nil,
            actor_reused = actor1 == actor2,
            actor_type = actor1 and actor1:GetObjectType(),
            set_model_by_unit = actor1 and actor1:SetModelByUnit("player"),
            actor_count = scene:GetNumActors(),
            actor_is_index1 = scene:GetActorAtIndex(1) == actor1,
        }
    "#,
    )
    .unwrap();

    let actor_state: (bool, bool, String, bool, i64, bool) = env
        .eval(
            r#"
            local s = _G.player_actor_state
            return s.actor_exists,
                   s.actor_reused,
                   s.actor_type,
                   s.set_model_by_unit,
                   s.actor_count,
                   s.actor_is_index1
        "#,
        )
        .unwrap();

    assert!(actor_state.0);
    assert!(actor_state.1);
    assert_eq!(actor_state.2, "ModelSceneActor");
    assert!(actor_state.3);
    assert_eq!(actor_state.4, 1);
    assert!(actor_state.5);
}

#[test]
fn test_model_scene_mixin_transition_resolves_collectionator_player_actor() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local scene = CreateFrame("ModelScene", "TestCollectionatorPlayerModelScene", UIParent, "ModelSceneMixinTemplate")
        scene:TransitionToModelSceneID(596, CAMERA_TRANSITION_TYPE_IMMEDIATE, CAMERA_MODIFICATION_TYPE_DISCARD, true)
        local actor = scene:GetPlayerActor()

        _G.collectionator_player_actor_state = {
            lifecycle_methods_visible = type(scene.OnReleased) == "function"
                and type(scene.ApplyFromModelSceneActorInfo) == "function",
            actor_exists = actor ~= nil,
            actor_type = actor and actor:GetObjectType(),
            set_model_by_unit = actor and actor:SetModelByUnit("player"),
            human_male_actor = scene:GetActorByTag("human-male") ~= nil,
            human_actor = scene:GetActorByTag("human") ~= nil,
            player_actor = scene:GetActorByTag("player") ~= nil,
            tauren_actor = scene:GetActorByTag("tauren-male") ~= nil,
        }
    "#,
    )
    .unwrap();

    let actor_state: (bool, bool, String, bool, bool, bool, bool, bool) = env
        .eval(
            r#"
            local s = _G.collectionator_player_actor_state
            return s.lifecycle_methods_visible,
                   s.actor_exists,
                   s.actor_type,
                   s.set_model_by_unit,
                   s.human_male_actor,
                   s.human_actor,
                   s.player_actor,
                   s.tauren_actor
        "#,
        )
        .unwrap();

    assert!(actor_state.0);
    assert!(actor_state.1);
    assert_eq!(actor_state.2, "ModelSceneActor");
    assert!(actor_state.3);
    assert!(!actor_state.4);
    assert!(!actor_state.5);
    assert!(actor_state.6);
    assert!(actor_state.7);
}

#[test]
fn test_model_scene_actor_management_tracks_created_indexed_and_taken_actors() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local scene = CreateFrame("ModelScene", "TestModelSceneActors", UIParent)
        local actor1 = scene:CreateActor("FirstActor", "ModelSceneActorTemplate")
        local actor2 = scene:CreateActor("SecondActor", "ModelSceneActorTemplate")
        local count_after_create = scene:GetNumActors()
        local actor1_is_index1 = scene:GetActorAtIndex(1) == actor1
        local actor2_is_index2 = scene:GetActorAtIndex(2) == actor2
        local missing = scene:GetActorAtIndex(3)
        local taken = scene:TakeActor()

        _G.actor_scene_state = {
            actor1_ok = actor1 ~= nil,
            actor2_ok = actor2 ~= nil,
            count_after_create = count_after_create,
            actor1_is_index1 = actor1_is_index1,
            actor2_is_index2 = actor2_is_index2,
            missing_is_nil = missing == nil,
            taken_is_actor2 = taken == actor2,
            count_after_take = scene:GetNumActors(),
            actor1_still_index1 = scene:GetActorAtIndex(1) == actor1,
            actor2_removed = scene:GetActorAtIndex(2) == nil,
        }
    "#,
    )
    .unwrap();

    let actor_state: (bool, bool, i64, bool, bool, bool, bool, i64, bool, bool) = env
        .eval(
            r#"
            local s = _G.actor_scene_state
            return s.actor1_ok,
                   s.actor2_ok,
                   s.count_after_create,
                   s.actor1_is_index1,
                   s.actor2_is_index2,
                   s.missing_is_nil,
                   s.taken_is_actor2,
                   s.count_after_take,
                   s.actor1_still_index1,
                   s.actor2_removed
        "#,
        )
        .unwrap();

    assert!(actor_state.0);
    assert!(actor_state.1);
    assert_eq!(actor_state.2, 2);
    assert!(actor_state.3);
    assert!(actor_state.4);
    assert!(actor_state.5);
    assert!(actor_state.6);
    assert_eq!(actor_state.7, 1);
    assert!(actor_state.8);
    assert!(actor_state.9);

    let scene_id = env
        .state()
        .borrow()
        .widgets
        .get_id_by_name("TestModelSceneActors")
        .unwrap();
    let first_actor_id = env
        .state()
        .borrow()
        .widgets
        .get_id_by_name("FirstActor")
        .unwrap();
    let second_actor_id = env
        .state()
        .borrow()
        .widgets
        .get_id_by_name("SecondActor")
        .unwrap();

    let state = env.state().borrow();
    let scene = state.widgets.get(scene_id).unwrap();
    assert_eq!(scene.model_state().model_scene_actor_ids, vec![first_actor_id]);

    let first_actor = state.widgets.get(first_actor_id).unwrap();
    assert_eq!(first_actor.parent_id, Some(scene_id));
    assert_eq!(
        first_actor.object_type_name.as_deref(),
        Some("ModelSceneActor")
    );

    let second_actor = state.widgets.get(second_actor_id).unwrap();
    assert_eq!(second_actor.parent_id, None);
    assert_eq!(
        second_actor.object_type_name.as_deref(),
        Some("ModelSceneActor")
    );
}

#[test]
fn test_model_scene_actor_accepts_model_loaded_script() {
    let env = WowLuaEnv::new().unwrap();

    env.exec(
        r#"
        local scene = CreateFrame("ModelScene", "TestModelSceneActorScripts", UIParent)
        local actor = scene:CreateActor("ScriptedActor", "ModelSceneActorTemplate")
        actor:SetScript("OnModelLoaded", function() end)
        _G.actor_model_loaded_script_registered = actor:HasScript("OnModelLoaded")
    "#,
    )
    .unwrap();

    let registered: bool = env
        .eval("return _G.actor_model_loaded_script_registered")
        .unwrap();

    assert!(registered);
}
