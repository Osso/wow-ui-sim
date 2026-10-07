//! Repeated lookup must not fabricate removed namespace members.
#![cfg(feature = "client-retail")]

pub(crate) const RETIREMENT_ASSERTIONS: &str = r#"
for _, name in ipairs({'SetKeystoneTooltip'}) do
    assert(rawget(C_ChallengeMode, name) == nil, name)
    assert(C_ChallengeMode[name] == nil, name)
    assert(C_ChallengeMode[name] == nil, name)
end
for _, name in ipairs({'SetCorruptionReforgerItemTooltip', 'SetItemConversionOutputTooltip'}) do
    assert(rawget(C_ItemInteraction, name) == nil, name)
    assert(C_ItemInteraction[name] == nil, name)
    assert(C_ItemInteraction[name] == nil, name)
end
for _, name in ipairs({'GetUnspentPointsForSkillLine'}) do
    assert(rawget(C_ProfSpecs, name) == nil, name)
    assert(C_ProfSpecs[name] == nil, name)
    assert(C_ProfSpecs[name] == nil, name)
end
for _, name in ipairs({'CancelCraftingOrder', 'CompleteCraftingOrder', 'DeclineCraftingOrder', 'GetCraftingOrders', 'GetCurrentOrder', 'GetRecipeTools', 'HasCraftingOrderFavorites', 'HasMaxCraftingOrderFavorites', 'IsCraftingOrderFavorite', 'ListCraftingOrder', 'QueryCraftingOrdersFavorites', 'QueryCraftingOrders', 'RecipeCanBeRecrafted', 'SetCraftingOrderFavorite', 'SetTooltipRecipeResultItem', 'StartCraftingOrder'}) do
    assert(rawget(C_TradeSkillUI, name) == nil, name)
    assert(C_TradeSkillUI[name] == nil, name)
    assert(C_TradeSkillUI[name] == nil, name)
end
assert(type(C_CraftingOrders.GetMyOrders) == "function")
assert(type(C_TooltipInfo.GetItemByID) == "function")
"#;

#[test]
fn patch_10_0_2_unused_members_stay_absent() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(RETIREMENT_ASSERTIONS).unwrap();
}
