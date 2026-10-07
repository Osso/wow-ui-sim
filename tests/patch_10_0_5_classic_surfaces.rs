//! Retail retirement leaves the existing classic namespace lookup unchanged.
#![cfg(feature = "client-mists")]

#[test]
fn patch_10_0_5_mists_preserves_legacy_namespace_lookup() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _, name in ipairs({'ContinueRecast', 'GetRecipeRepeatCount', 'HasRecipesTracked'}) do
            assert(type(C_TradeSkillUI[name]) == 'function', name)
        end
        "#,
    )
    .unwrap();
}
