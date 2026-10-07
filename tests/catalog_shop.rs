use std::path::PathBuf;

use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::c_api::c_catalog_shop_products::{
    CatalogShopProductDisplayInfo, CatalogShopProductInfo, CatalogShopSectionInfo,
};
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
        wow_ui_sim::loader::load_addon(&env.loader_env(), toc_path).unwrap_or_else(|err| {
            panic!("[load {name}] FAILED: {err}");
        });
    }

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

fn catalog_product_record() -> CatalogShopProductInfo {
    CatalogShopProductInfo {
        catalog_shop_product_id: 2003.0,
        name: "Apprentice Rider Bundle".into(),
        r#type: Some("Bundle".into()),
        description: "Explicit full UI storefront fixture".into(),
        icon_texture: "Interface\\Icons\\Ability_Mount_RidingHorse".into(),
        price: "10".into(),
        original_price: "10".into(),
        background_texture: "shop-bg-map-blue".into(),
        is_bundle: true,
        sub_items_loaded: true,
        ..Default::default()
    }
}

fn catalog_display_record() -> CatalogShopProductDisplayInfo {
    CatalogShopProductDisplayInfo {
        default_preview_model_scene_id: 1.0,
        default_card_model_scene_id: 1.0,
        default_wide_card_model_scene_id: 1.0,
        product_type: Some("Bundle".into()),
        ..Default::default()
    }
}

fn insert_catalog_product_fixture(env: &WowLuaEnv) {
    let category_ids: Vec<i32> = env.eval("return C_CatalogShop.GetAvailableCategoryIDs()").unwrap();
    assert!(!category_ids.is_empty(), "fixture requires catalog navigation categories");
    let mut state = env.state().borrow_mut();
    let catalog = &mut state.catalog_shop_products;
    catalog.products.insert(2003, catalog_product_record());
    catalog.displays.insert(2003, catalog_display_record());
    for category_id in category_ids {
        catalog.sections.insert((category_id, 1), CatalogShopSectionInfo {
            id: 1.0,
            display_name: "Full UI fixture products".into(),
            parent_catalog_shop_category_info_id: Some(f64::from(category_id)),
            ..Default::default()
        });
    }
}

prefork_full_ui_case! {
fn catalog_shop_loads_and_populates_navigation_and_products(env: &WowLuaEnv) {
    insert_catalog_product_fixture(env);

    let (loaded, reason): (bool, Option<String>) = env
        .eval("return C_AddOns.LoadAddOn('Blizzard_CatalogShop')")
        .expect("LoadAddOn('Blizzard_CatalogShop') should return");
    assert!(
        loaded,
        "Blizzard_CatalogShop should load successfully: reason={reason:?}"
    );

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    env.exec("CatalogShopFrame:Show()")
        .expect("CatalogShopFrame:Show() should run");

    let targeted: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .clone()
        .into_iter()
        .filter(|message| message.contains("expected number, got nil at argument 1"))
        .collect();

    assert!(
        targeted.is_empty(),
        "CatalogShop should not emit the numeric startup regression when shown:\n  {}",
        targeted.join("\n  ")
    );

    let result: String = env
        .eval(
            r#"
            local categoryIDs = C_CatalogShop.GetAvailableCategoryIDs()
            if not categoryIDs or #categoryIDs == 0 then
                return "no_categories"
            end

            if CatalogShopFrame.failedLoad then
                return "failed_load"
            end

            if CatalogShopFrame.CatalogShopUnavailableScreenFrame:IsShown() then
                return "unavailable_screen_shown"
            end

            local navProvider = CatalogShopFrame.HeaderFrame.CatalogShopNavBar.NavButtonScrollBox:GetDataProvider()
            if not navProvider then
                return "missing_nav_provider"
            end

            if navProvider:GetSize() == 0 then
                return "nav_provider_empty"
            end

            EventRegistry:TriggerEvent("CatalogShop.OnCategorySelected", categoryIDs[1])

            local productProvider = CatalogShopFrame.ProductContainerFrame.ProductsScrollBoxContainer.ScrollBox:GetDataProvider()
            if not productProvider then
                return "missing_product_provider"
            end

            if productProvider:GetSize() == 0 then
                return "product_provider_empty"
            end

            local firstProduct = productProvider:FindElementDataByPredicate(function(elementData)
                return elementData.elementType == CatalogShopConstants.ScrollViewElementType.Product
            end)
            if not firstProduct then
                return "missing_first_product"
            end

            return firstProduct.name or ""
            "#,
        )
        .expect("catalog shop UI should be queryable");

    assert_eq!(result, "Apprentice Rider Bundle");
}
}

prefork_full_ui_case! {
fn catalog_shop_bundle_card_uses_default_model_scene_id(env: &WowLuaEnv) {

    let (loaded, reason): (bool, Option<String>) = env
        .eval("return C_AddOns.LoadAddOn('Blizzard_CatalogShop')")
        .expect("LoadAddOn('Blizzard_CatalogShop') should return");
    assert!(
        loaded,
        "Blizzard_CatalogShop should load successfully: reason={reason:?}"
    );

    env.exec(
        r#"
        local seen_model_scene_id = nil
        CatalogShopUtil.SetupModelSceneForBundle = function(modelScene, modelSceneID, displayData, ...)
            seen_model_scene_id = modelSceneID
        end

        local card = CreateFrame("Button", "CatalogShopCardProbe", UIParent, "DefaultCatalogShopCardTemplate")
        card.useWideCardSettings = false
        card.productInfo = {
            catalogShopProductID = 1,
            cardDisplayData = {
                selectedModelSceneID = 4242,
                defaultModelSceneID = 31337,
                overrideModelSceneID = 4242,
            },
            isBundleChild = false,
            categoryID = 1,
        }

        local displayInfo = {
            defaultCardModelSceneID = 31337,
            overrideCardModelSceneID = 4242,
            selectedModelSceneID = 4242,
            hasUnknownLicense = false,
            productType = CatalogShopConstants.ProductType.Bundle,
        }

        card:SetModelScene(card.productInfo, true, displayInfo, CatalogShopConstants.ProductType.Bundle)
        assert(seen_model_scene_id == 31337, "bundle cards should pass defaultCardModelSceneID to SetupModelSceneForBundle")
        "#,
    )
    .expect("bundle card default model scene routing should run");
}
}

prefork_full_ui_case! {
fn catalog_shop_product_display_translation_uses_override_scene_when_default_is_missing(env: &WowLuaEnv) {

    env.exec(
        r#"
        local original_get_model_scene_info_by_id = C_ModelInfo.GetModelSceneInfoByID
        local seen_model_scene_id = nil
        C_ModelInfo.GetModelSceneInfoByID = function(modelSceneID)
            seen_model_scene_id = modelSceneID
            return "mock", {}, {}, 0
        end

        local displayInfo = CatalogShopUtil.TranslateProductInfoToProductDisplayData(
            { productType = CatalogShopConstants.ProductType.Bundle },
            nil,
            4242
        )

        C_ModelInfo.GetModelSceneInfoByID = original_get_model_scene_info_by_id

        assert(seen_model_scene_id == 4242, "translation should fall back to the override scene id")
        assert(displayInfo.selectedModelSceneID == 4242, "translation should preserve the selected scene id")
        "#,
    )
    .expect("catalog shop product display translation should run");
}
}

prefork_full_ui_case! {
fn catalog_shop_load_does_not_request_nil_model_scene_ids(env: &WowLuaEnv) {

    env.exec(
        r#"
        local original = C_ModelInfo.GetModelSceneInfoByID
        __catalog_shop_nil_model_scene_calls = {}
        C_ModelInfo.GetModelSceneInfoByID = function(modelSceneID)
            if modelSceneID == nil then
                __catalog_shop_nil_model_scene_calls[#__catalog_shop_nil_model_scene_calls + 1] = debug.traceback("nil model scene id")
                return nil
            end
            return original(modelSceneID)
        end
        "#,
    )
    .expect("should install model scene probe");

    let (loaded, reason): (bool, Option<String>) = env
        .eval("return C_AddOns.LoadAddOn('Blizzard_CatalogShop')")
        .expect("LoadAddOn('Blizzard_CatalogShop') should return");
    assert!(
        loaded,
        "Blizzard_CatalogShop should load successfully: reason={reason:?}"
    );

    let calls: String = env
        .eval("return table.concat(__catalog_shop_nil_model_scene_calls or {}, '\\n---\\n')")
        .expect("nil model scene call log should stringify");
    assert!(
        calls.is_empty(),
        "CatalogShop should not request nil model scene ids:\n{calls}"
    );
}
}
