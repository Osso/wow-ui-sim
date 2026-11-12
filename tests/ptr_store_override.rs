//! R75: PTR rewards override and catalog session validation.

use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(feature = "client-ptr")]
#[test]
fn override_queries_are_registered_in_game_and_glue_with_neutral_defaults() {
    use wow_ui_sim::screen::ScreenKind;

    for screen in [ScreenKind::Game, ScreenKind::Login] {
        let env = WowLuaEnv::new().unwrap();
        env.set_screen_mode(screen);
        let result: (bool, i64, bool, i64, bool, bool) = env
            .eval(
                r#"
            return C_StorePublic.IsRewardsShopOverrideEnabled(),
                select('#', C_StorePublic.IsRewardsShopOverrideEnabled()),
                C_CatalogShop.ValidateOverrideShopInteraction(),
                select('#', C_CatalogShop.ValidateOverrideShopInteraction()),
                C_CatalogShop.ValidateOverrideShopInteraction(nil),
                C_CatalogShop.ValidateOverrideShopInteraction('')
        "#,
            )
            .unwrap();
        assert_eq!(result, (false, 1, false, 1, false, false));
    }
}

#[cfg(not(feature = "client-ptr"))]
#[test]
fn ptr_override_queries_are_not_registered_on_other_profiles() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local store = rawget(_G, 'C_StorePublic')
        local catalog = rawget(_G, 'C_CatalogShop')
        assert(store == nil or rawget(store, 'IsRewardsShopOverrideEnabled') == nil)
        assert(catalog == nil or rawget(catalog, 'ValidateOverrideShopInteraction') == nil)
    "#,
    )
    .unwrap();
}
