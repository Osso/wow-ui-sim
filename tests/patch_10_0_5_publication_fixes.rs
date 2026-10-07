//! Repeated removed-member lookup and preserved tracked-recipe successors.
#![cfg(feature = "client-retail")]

pub(crate) const RETIREMENT_ASSERTIONS: &str = r#"
for _, name in ipairs({'ContinueRecast', 'GetRecipeRepeatCount', 'HasRecipesTracked'}) do
    assert(rawget(C_TradeSkillUI, name) == nil, name)
    assert(C_TradeSkillUI[name] == nil, name)
    assert(C_TradeSkillUI[name] == nil, name)
end
assert(type(C_TradeSkillUI.GetRecipesTracked) == 'function')
assert(type(C_TradeSkillUI.IsRecipeTracked) == 'function')
assert(type(C_TradeSkillUI.SetRecipeTracked) == 'function')
"#;

#[test]
fn patch_10_0_5_unused_members_stay_absent() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(RETIREMENT_ASSERTIONS).unwrap();
}
