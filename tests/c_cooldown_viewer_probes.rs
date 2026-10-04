use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn cooldown_viewer_empty_state_has_no_cooldowns() {
    let env = env();
    let (category_count, cooldown_is_nil, cooldown_id_is_nil): (i32, bool, bool) = env
        .eval(
            r#"
            return
                #C_CooldownViewer.GetCooldownViewerCategorySet(Enum.CooldownViewerCategory.Essential, true),
                C_CooldownViewer.GetCooldownViewerCooldownInfo(1) == nil,
                C_CooldownViewer.GetCooldownID() == nil
            "#,
        )
        .unwrap();
    assert_eq!(category_count, 0);
    assert!(cooldown_is_nil);
    assert!(cooldown_id_is_nil);
}
