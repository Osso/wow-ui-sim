#![cfg(feature = "retail-12-0-0")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn p1127_model_collision_preference_is_permanently_unsupported() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(r#"
        local scene = CreateFrame('ModelScene')
        local actor = scene:CreateActor()
        assert(actor:IsPreferringModelCollisionBounds() == false)
        actor:SetPreferModelCollisionBounds(true)
        assert(actor:IsPreferringModelCollisionBounds() == false)
        actor:SetPreferModelCollisionBounds(false)
        assert(actor:IsPreferringModelCollisionBounds() == false)
    "#).unwrap();
}
