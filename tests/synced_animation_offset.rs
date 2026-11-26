use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn synced_animation_consumer_starts_at_distinct_offsets() {
    let env = WowLuaEnv::new().unwrap();
    let root = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let source = std::fs::read_to_string(root.join("Blizzard_SharedXML/AnimationTemplates.lua")).unwrap();
    env.exec(&source).unwrap();
    env.exec(r#"
        group = UIParent:CreateAnimationGroup()
        group:CreateAnimation("Alpha"):SetDuration(2)
        group:SetLooping("REPEAT")
        Mixin(group, SyncedAnimGroupMixin)
        group:PlaySynced(false, "R88g7-offset-proof")
        firstElapsed = group:GetElapsed()
        group:Stop()
    "#).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(250));
    env.exec(r#"
        group:PlaySynced(false, "R88g7-offset-proof")
        local second = group:GetElapsed()
        assert(firstElapsed < 0.01 and second > 0.20 and second < 0.30)
        assert(second > firstElapsed + 0.20, "constant offset producer must fail")
    "#).unwrap();
}
