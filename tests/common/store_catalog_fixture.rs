//! Explicit host records for Store consumer tests; these are not runtime defaults.

use wow_ui_sim::c_api::store_catalog_model::{
    StoreCatalogInput, StoreCurrencyInfo, StoreEntryInput, StoreProductGroupInput,
    StoreProductInput, StoreProductListResponse, StoreProductSharedData,
    StorePurchaseListResponse,
};
use wow_ui_sim::lua_api::WowLuaEnv;

const FEATURED_GROUP_ID: u32 = 501;
const FEATURED_ENTRY_ID: i32 = 1003;
const FEATURED_PRODUCT_ID: i32 = 2003;
const CURRENCY_ID: i32 = 61;

pub fn seed_vas_catalog(env: &WowLuaEnv) {
    let vas_service_type = enum_value(env, "Enum.VasServiceType.NameChange");
    let product = featured_product(0, vas_service_type);
    let entry = featured_entry();
    let mut sim = env.state().borrow_mut();
    sim.store_glue.store_secure.catalog.products.push(product);
    sim.store_glue.store_secure.catalog.entries.push(entry);
}

pub fn publish_featured_catalog(env: &WowLuaEnv) {
    let card_type = enum_value(env, "Enum.BattlepayCardType.MediumCardWithBuyButton");
    let vas_service_type = enum_value(env, "Enum.VasServiceType.NameChange");
    let catalog = featured_catalog(card_type, vas_service_type);
    store_host_readiness(env, &catalog);
    receive_featured_responses(env, catalog);
}

fn store_host_readiness(env: &WowLuaEnv, catalog: &StoreCatalogInput) {
    let mut sim = env.state().borrow_mut();
    let store = &mut sim.store_glue.store_secure;
    store.catalog = catalog.clone();
    store.currency_id = CURRENCY_ID;
    store.currencies.insert(CURRENCY_ID, currency_info());
    store.distribution_list_ready = Some(true);
    sim.store_glue.connection_present = true;
    let callbacks = &mut sim.store_glue.callbacks;
    callbacks.store_status_changed = true;
    callbacks.store_products_updated = true;
    callbacks.store_purchase_list_updated = true;
}

fn receive_featured_responses(env: &WowLuaEnv, catalog: StoreCatalogInput) {
    wow_ui_sim::c_api::store_catalog_model::receive_availability(env, true)
        .expect("store availability response should dispatch");
    wow_ui_sim::c_api::store_catalog_model::receive_product_list(
        env,
        StoreProductListResponse {
            result_code: 0,
            currency_id: CURRENCY_ID,
            catalog,
        },
    )
    .expect("store product-list response should dispatch");
    wow_ui_sim::c_api::store_catalog_model::receive_purchase_list(
        env,
        StorePurchaseListResponse {
            result_code: 0,
            rows: Vec::new(),
        },
    )
    .expect("store purchase-list response should dispatch");
}

fn featured_catalog(card_type: u32, vas_service_type: u32) -> StoreCatalogInput {
    StoreCatalogInput {
        groups: vec![StoreProductGroupInput {
            group_id: FEATURED_GROUP_ID,
            parent_group_id: 0,
            order: 1,
            displayable: true,
            name: "Featured".into(),
            texture: None,
            display_type: 1,
            flags: 0,
            disabled_tooltip: None,
        }],
        products: vec![featured_product(card_type, vas_service_type)],
        entries: vec![featured_entry()],
    }
}

fn featured_product(card_type: u32, vas_service_type: u32) -> StoreProductInput {
    StoreProductInput {
        product_id: FEATURED_PRODUCT_ID,
        supported: true,
        session_eligible: true,
        display_eligible: true,
        dynamic_bundle: false,
        product_type: Some(7),
        shared_data: StoreProductSharedData {
            normal_dollars: 10,
            current_dollars: 10,
            item_quantity: 1,
            buyable_here: true,
            can_change_account: true,
            can_change_bnet_account: true,
            card_type,
            vas_service_type: Some(vas_service_type),
            name: Some("Apprentice Rider Bundle".into()),
            description: Some("Store UI test input".into()),
            tooltip: Some("Store UI test input".into()),
            ..Default::default()
        },
    }
}

fn featured_entry() -> StoreEntryInput {
    StoreEntryInput {
        entry_id: FEATURED_ENTRY_ID,
        product_id: FEATURED_PRODUCT_ID,
        group_id: FEATURED_GROUP_ID,
        order: 1,
        banner_type: 0,
        hidden: false,
    }
}

fn currency_info() -> StoreCurrencyInfo {
    StoreCurrencyInfo {
        region_id: 1,
        format_short: "short-%s".into(),
        format_long: "long-%s".into(),
        license_accept_text: "test terms".into(),
        require_license_accept: false,
        browse_has_star: false,
        hide_browse_notice: true,
        hide_confirmation_browse_notice: false,
    }
}

fn enum_value(env: &WowLuaEnv, path: &str) -> u32 {
    let value: i64 = env
        .eval(&format!("return {path}"))
        .unwrap_or_else(|error| panic!("{path} should be available: {error}"));
    u32::try_from(value).unwrap_or_else(|_| panic!("{path} should be a non-negative enum value"))
}
