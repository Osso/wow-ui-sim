#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn load_full_game_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);

    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }

    wow_ui_sim::xml::register_intrinsic_templates();

    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    for (name, toc_path) in &addons {
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

prefork_full_ui_case! {
fn blizzard_catalog_shop_shared_util_constants_table_is_populated(env: &WowLuaEnv) {

    let constants_present: bool = env
        .eval(
            "return type(CatalogShopConstants) == 'table' \
                and type(CatalogShopConstants.ScrollViewType) == 'table' \
                and CatalogShopConstants.ScrollViewType.List == 1 \
                and CatalogShopConstants.ScrollViewType.Grid == 2 \
                and type(CatalogShopConstants.CardTemplate) == 'table' \
                and CatalogShopConstants.CardTemplate.Wide == 'WideCatalogShopProductCardTemplate' \
                and CatalogShopConstants.CardTemplate.Small == 'SmallCatalogShopProductCardTemplate'",
        )
        .expect("constants query should succeed");
    assert!(
        constants_present,
        "CatalogShopConstants and its nested tables should be populated after load"
    );
}
}

prefork_full_ui_case! {
fn blizzard_catalog_shop_shared_util_secure_dependency_loads_helpers(env: &WowLuaEnv) {

    let helpers_loaded_from_secure_dependency: bool = env
        .eval(
            "local secureUtil = rawget(__secureenv, 'CatalogShopUtil'); \
             return type(_G.CatalogShopUtil) == 'table' \
                and type(secureUtil) == 'table' \
                and type(secureUtil.GetPlayerActorLabelTag) == 'function' \
                and type(secureUtil.CreateDefaultProductDisplayData) == 'function'",
        )
        .expect("CatalogShopUtil secure dependency query should succeed");
    assert!(
        helpers_loaded_from_secure_dependency,
        "Blizzard_CatalogShopSharedUtil should publish CatalogShopUtil helpers in both _G and __secureenv"
    );
}
}

prefork_full_ui_case! {
fn blizzard_catalog_shop_shared_util_exposes_helper_functions(env: &WowLuaEnv) {

    let helpers_present: bool = env
        .eval(
            "return type(CatalogShopUtil) == 'table' \
                and type(CatalogShopUtil.GetPlayerActorLabelTag) == 'function' \
                and type(CatalogShopUtil.CreateDefaultProductDisplayData) == 'function' \
                and type(CatalogShopUtil.ExtractProductInfoForDisplayData) == 'function' \
                and type(CatalogShopUtil.TranslateProductInfoToProductDisplayData) == 'function' \
                and type(CatalogShopUtil.HasSpecialActors) == 'function'",
        )
        .expect("helpers query should succeed");
    assert!(
        helpers_present,
        "CatalogShopUtil should expose its core helper functions after load"
    );

    let default_display_data_shape_ok: bool = env
        .eval(
            "local data = CatalogShopUtil.CreateDefaultProductDisplayData(); \
             return type(data) == 'table'",
        )
        .expect("default display data query should succeed");
    assert!(
        default_display_data_shape_ok,
        "CatalogShopUtil.CreateDefaultProductDisplayData should return a table"
    );
}
}
