use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn recipe_source_text_reads_current_host_records() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("assert(C_TradeSkillUI.GetRecipeSourceText(4101) == nil)")
        .unwrap();
    {
        let mut state = env.state().borrow_mut();
        state
            .recipe_source_text
            .insert(4101, "Found in an old journal".into());
        state
            .recipe_source_text
            .insert(4102, "Taught by a master".into());
    }
    env.exec(
        r#"
        assert(C_TradeSkillUI.GetRecipeSourceText(4101) == "Found in an old journal")
        assert(C_TradeSkillUI.GetRecipeSourceText(4102) == "Taught by a master")
        assert(C_TradeSkillUI.GetRecipeSourceText(4103) == nil)
        "#,
    )
    .unwrap();
    env.state().borrow_mut().recipe_source_text.remove(&4101);
    env.exec("assert(C_TradeSkillUI.GetRecipeSourceText(4101) == nil)")
        .unwrap();
}
