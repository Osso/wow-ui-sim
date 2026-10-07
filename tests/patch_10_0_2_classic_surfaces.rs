//! Retail retirement must preserve classic namespace lookup.
#![cfg(feature = "client-mists")]

#[test]
fn patch_10_0_2_mists_preserves_legacy_namespace_lookup() {
    let env = wow_ui_sim::lua_api::WowLuaEnv::new().unwrap();
    env.exec(r#"
for _, name in ipairs({'SetKeystoneTooltip'}) do
    assert(type(C_ChallengeMode[name]) == "function", name)
end
for _, name in ipairs({'SetCorruptionReforgerItemTooltip', 'SetItemConversionOutputTooltip'}) do
    assert(type(C_ItemInteraction[name]) == "function", name)
end
for _, name in ipairs({'GetUnspentPointsForSkillLine'}) do
    assert(type(C_ProfSpecs[name]) == "function", name)
end
for _, name in ipairs({'CancelCraftingOrder', 'CompleteCraftingOrder', 'DeclineCraftingOrder', 'GetCraftingOrders', 'GetCurrentOrder', 'GetRecipeTools', 'HasCraftingOrderFavorites', 'HasMaxCraftingOrderFavorites', 'IsCraftingOrderFavorite', 'ListCraftingOrder', 'QueryCraftingOrdersFavorites', 'QueryCraftingOrders', 'RecipeCanBeRecrafted', 'SetCraftingOrderFavorite', 'SetTooltipRecipeResultItem', 'StartCraftingOrder'}) do
    assert(type(C_TradeSkillUI[name]) == "function", name)
end
"#).unwrap();
}
