//! `ModelSceneActor:SetModelByFileID` stores the file ID that
//! `GetModelFileID` reports. Blizzard's `ScriptAnimatedModelSceneActor:SetEffect`
//! (`ScriptAnimatedModelSceneActor.lua:67`) calls it when CatalogShop shows its
//! loading effect.

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn model_scene_actor_reports_supplied_model_file_ids() {
    let env = WowLuaEnv::new().expect("env");
    let ids: (f64, f64) = env
        .eval(
            r#"
            local scene = CreateFrame("ModelScene", "ActorFileIdScene")
            local actor = scene:CreateActor()
            actor:SetModelByFileID(165767)
            local first = actor:GetModelFileID()
            actor:SetModelByFileID(4207652)
            return first, actor:GetModelFileID()
            "#,
        )
        .unwrap();
    assert_eq!(ids, (165767.0, 4207652.0));
}
