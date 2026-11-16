#![cfg(feature = "client-retail")]

use wow_ui_sim::c_api::store_catalog_model::{
    StoreCatalogInput, StoreCurrencyInfo, StoreEntryInput, StoreProductCardInput,
    StoreProductDeliverableInput, StoreProductGroupInput, StoreProductInput,
    StoreProductListResponse, StoreProductSharedData, StorePurchaseInput, StorePurchaseListResponse,
};
use wow_ui_sim::c_api::store_glue_model::StoreRequest;
use wow_ui_sim::lua_api::WowLuaEnv;

const CATALOG_CONSUMER: &str = r##"
    local filtered = StoreFrame_FilterEntries({8101, 8102})
    assert(#filtered == 2 and filtered[1] == 8101 and filtered[2] == 8102)
    local scrollBox = {}
    function scrollBox:SetDataProvider(provider) self.provider = provider end
    CategoryTreeScrollContainerMixin.UpdateCategories({ScrollBox = scrollBox})
    assert(scrollBox.provider ~= nil)
    local groups = C_StoreSecure.GetProductGroups()
    assert(#groups == 2 and groups[1].groupID == 4102 and groups[2].groupID == 4101)
    assert(not StoreFrame_IsProductGroupDisabled(4101))
    assert(not StoreFrame_IsProductGroupDisabled(4102))
    assert(StoreFrame_IsProductGroupDisabled(4999))
    local group = C_StoreSecure.GetProductGroupInfo(4101)
    assert(group.groupName == "Host Alpha" and group.displayType == 1 and group.flags == 0)
    assert(select("#", C_StoreSecure.GetProductGroupInfo(4999)) == 0)
    local entries = C_StoreSecure.GetProducts(4101)
    assert(#entries == 2 and entries[1] == 8102 and entries[2] == 8101)
    local hidden = C_StoreSecure.GetProducts(4102, true)
    assert(#hidden == 2 and hidden[1] == 8103 and hidden[2] == 8105)
    assert(#C_StoreSecure.GetProducts(4999) == 0)
    assert(C_StoreSecure.GetProducts(4102)[2] == nil)
    local entry = C_StoreSecure.GetEntryInfo(8101)
    assert(entry.productID == 8201 and entry.groupID == 4101 and entry.bannerType == 1)
    assert(entry.alreadyOwned == false and entry.sharedData.name == "Host One")
    assert(C_StoreSecure.GetEntryInfo(4999) == nil)
    local first = C_StoreSecure.GetProductInfo(8201)
    local second = C_StoreSecure.GetProductInfo(8202)
    assert(first.productID == 8201 and second.productID == 8202)
    assert(first.sharedData.cards[1].itemModifiedAppearanceIDs[2] == 10)
    assert(first.sharedData.deliverables[1].owned == false)
    assert(C_StoreSecure.GetProductInfo(4999) == nil)
    return true
"##;

const CURRENCY_CONSUMER: &str = r#"
    local storeInfo = StoreFrame_CurrencyInfo()
    assert(FormatCurrencyStringShort == "short-%s", "StoreFrame must consume host currency formatting")
    assert(FormatCurrencyStringLong == "long-%s")
    assert(storeInfo.hideBrowseNotice == true)
    local currencyInfo = C_StoreSecure.GetCurrencyInfo()
    assert(type(currencyInfo.sharedData) == "table")
    assert(currencyInfo.currencyFields == nil)
    assert(currencyInfo.sharedData.licenseAcceptText == "terms-61")
    return true
"#;

const STORE_EVENT_CONSUMER: &str = r##"
    STORE_CATALOG_EVENTS = {}
    local listener = CreateFrame("Frame")
    for _, event in ipairs({"STORE_STATUS_CHANGED", "STORE_PRODUCTS_UPDATED",
        "STORE_PURCHASE_LIST_UPDATED", "DYNAMIC_BUNDLE_PRICE_UPDATED"}) do
        listener:RegisterEvent(event)
    end
    listener:SetScript("OnEvent", function(_, event, ...)
        local record = {event = event, count = select("#", ...)}
        if event == "STORE_STATUS_CHANGED" then
            record.available = C_StoreSecure.IsAvailable()
        elseif event == "STORE_PRODUCTS_UPDATED" then
            record.ready = C_StoreSecure.HasProductList()
            record.currency = C_StoreSecure.GetCurrencyID()
            record.groups = #C_StoreSecure.GetProductGroups()
        elseif event == "STORE_PURCHASE_LIST_UPDATED" then
            record.ready = C_StoreSecure.HasPurchaseList()
            record.progress = C_StoreSecure.HasPurchaseInProgress()
        else
            record.productID = ...
            record.hasData = C_StoreSecure.HasDynamicPriceData(record.productID)
        end
        table.insert(STORE_CATALOG_EVENTS, record)
    end)
"##;

fn load_store_ui() -> WowLuaEnv {
    let ui_dir = crate::common::blizzard_addon_harness::blizzard_ui_dir();
    let (env, loaded) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &ui_dir,
        &["Blizzard_StoreUI"],
        &[],
    );
    assert!(loaded.iter().any(|name| name == "Blizzard_StoreUI"));
    assert!(loaded.iter().any(|name| name == "Blizzard_CatalogShop"));
    env
}

fn shared_data(name: &str) -> StoreProductSharedData {
    StoreProductSharedData {
        normal_dollars: 10,
        current_dollars: 10,
        item_quantity: 1,
        buyable_here: true,
        name: Some(name.into()),
        cards: vec![card(name)],
        deliverables: vec![deliverable(name)],
        ..Default::default()
    }
}

fn card(name: &str) -> StoreProductCardInput {
    StoreProductCardInput {
        title: format!("{name} card"),
        model_scene_id: 7,
        creature_display_info_id: 8,
        item_modified_appearance_ids: vec![9, 10],
        display_transmog_items_individually: false,
    }
}

fn deliverable(name: &str) -> StoreProductDeliverableInput {
    StoreProductDeliverableInput {
        name: format!("{name} mount"),
        owned: false,
    }
}

fn group(id: u32, name: &str, order: i32, displayable: bool) -> StoreProductGroupInput {
    StoreProductGroupInput {
        group_id: id,
        parent_group_id: 0,
        order,
        displayable,
        name: name.into(),
        texture: Some(17),
        display_type: 1,
        flags: 0,
        disabled_tooltip: None,
    }
}

fn product(id: i32, name: &str, dynamic: bool, visible: bool) -> StoreProductInput {
    StoreProductInput {
        product_id: id,
        supported: true,
        session_eligible: true,
        display_eligible: visible,
        dynamic_bundle: dynamic,
        shared_data: shared_data(name),
    }
}

fn entry(id: i32, product_id: i32, group_id: u32, order: i32, hidden: bool) -> StoreEntryInput {
    StoreEntryInput {
        entry_id: id,
        product_id,
        group_id,
        order,
        banner_type: order,
        hidden,
    }
}

fn currency(region: u32, short: &str, long: &str, license: &str) -> StoreCurrencyInfo {
    StoreCurrencyInfo {
        region_id: region,
        format_short: short.into(),
        format_long: long.into(),
        license_accept_text: license.into(),
        require_license_accept: region == 1,
        browse_has_star: region == 2,
        hide_browse_notice: region == 1,
        hide_confirmation_browse_notice: region == 2,
    }
}

fn seed_groups(store: &mut wow_ui_sim::c_api::store_catalog_model::StoreSecureState) {
    store.catalog.groups = vec![
        group(4102, "Host Beta", 10, true),
        group(4101, "Host Alpha", 20, true),
        group(4103, "Filtered Group", 30, false),
    ];
}

fn seed_products(store: &mut wow_ui_sim::c_api::store_catalog_model::StoreSecureState) {
    store.catalog.products = vec![
        product(8201, "Host One", true, true),
        product(8202, "Host Two", true, true),
        product(8203, "Host Three", false, true),
        product(8204, "Hidden", false, true),
        product(239, "Excluded", false, true),
    ];
}

fn seed_entries(store: &mut wow_ui_sim::c_api::store_catalog_model::StoreSecureState) {
    store.catalog.entries = vec![
        entry(8103, 8203, 4102, 1, false),
        entry(8104, 239, 4102, 2, false),
        entry(8105, 8204, 4102, 3, true),
        entry(8102, 8202, 4101, 1, false),
        entry(8101, 8201, 4101, 2, false),
    ];
}

fn seed_currencies(store: &mut wow_ui_sim::c_api::store_catalog_model::StoreSecureState) {
    store.currency_id = 61;
    store.currencies.insert(61, currency(1, "short-%s", "long-%s", "terms-61"));
    store.currencies.insert(62, currency(2, "short2-%s", "long2-%s", "terms-62"));
}

fn seed_catalog(env: &WowLuaEnv) {
    let mut sim = env.state().borrow_mut();
    let store = &mut sim.store_glue.store_secure;
    seed_groups(store);
    seed_products(store);
    seed_entries(store);
    seed_currencies(store);
}

fn seed_purchase_progress(env: &WowLuaEnv) {
    let mut sim = env.state().borrow_mut();
    let store = &mut sim.store_glue.store_secure;
    store.current_time = 1000.0;
    store.shared_time_adjustment = 100.0;
    store.purchase_list = vec![
        StorePurchaseInput { blocked: false, status: 1, expires_at: 1200.0, client_time_adjustment: 50.0 },
        StorePurchaseInput { blocked: true, status: 1, expires_at: 5000.0, client_time_adjustment: 0.0 },
    ];
}

fn enable_store_callbacks(env: &WowLuaEnv) {
    let mut sim = env.state().borrow_mut();
    let callbacks = &mut sim.store_glue.callbacks;
    callbacks.store_products_updated = true;
    callbacks.store_status_changed = true;
    callbacks.store_purchase_list_updated = true;
    callbacks.dynamic_bundle_price_updated = true;
}

fn receive_success_responses(env: &WowLuaEnv) {
    wow_ui_sim::c_api::store_catalog_model::receive_availability(env, true).unwrap();
    let catalog = env.state().borrow().store_glue.store_secure.catalog.clone();
    let response = StoreProductListResponse { result_code: 0, currency_id: 62, catalog };
    wow_ui_sim::c_api::store_catalog_model::receive_product_list(env, response).unwrap();
    wow_ui_sim::c_api::store_catalog_model::receive_dynamic_price(env, 8201, vec![1, 2]).unwrap();
    let purchase = StorePurchaseListResponse { result_code: 0, rows: Vec::new() };
    wow_ui_sim::c_api::store_catalog_model::receive_purchase_list(env, purchase).unwrap();
}

fn receive_failure_responses(env: &WowLuaEnv) {
    let product = StoreProductListResponse {
        result_code: 3,
        currency_id: 77,
        catalog: StoreCatalogInput::default(),
    };
    wow_ui_sim::c_api::store_catalog_model::receive_product_list(env, product).unwrap();
    let rows = vec![purchase_input(false), purchase_input(true)];
    let purchase = StorePurchaseListResponse { result_code: 7, rows };
    wow_ui_sim::c_api::store_catalog_model::receive_purchase_list(env, purchase).unwrap();
}

fn purchase_input(blocked: bool) -> StorePurchaseInput {
    StorePurchaseInput { blocked, status: 1, expires_at: 5000.0, client_time_adjustment: 0.0 }
}

fn assert_success_state(env: &WowLuaEnv) {
    env.exec_rilua_secure(
        "assert(C_StoreSecure.IsAvailable() and C_StoreSecure.HasProductList() and C_StoreSecure.HasPurchaseList() and C_StoreSecure.GetCurrencyID() == 62 and not C_StoreSecure.HasPurchaseInProgress() and C_StoreSecure.GetProductInfo(8201).sharedData.name == 'Host One' and C_StoreSecure.HasDynamicPriceData(8201))",
    )
    .unwrap();
}

fn assert_failure_state(env: &WowLuaEnv) {
    env.exec_rilua_secure(
        "assert(not C_StoreSecure.HasProductList() and not C_StoreSecure.HasPurchaseList() and C_StoreSecure.IsRegionLocked() and C_StoreSecure.GetLastProductListResponseError() == 3 and C_StoreSecure.GetCurrencyID() == 62 and #C_StoreSecure.GetProductGroups() == 0 and not C_StoreSecure.HasPurchaseInProgress())",
    )
    .unwrap();
}

fn assert_response_events(env: &WowLuaEnv) {
    env.exec_rilua_secure(
        "local e=STORE_CATALOG_EVENTS; assert(#e==6 and e[1].event=='STORE_STATUS_CHANGED' and e[1].count==0 and e[1].available and e[2].event=='STORE_PRODUCTS_UPDATED' and e[2].count==0 and e[2].ready and e[2].currency==62 and e[2].groups==2 and e[3].event=='DYNAMIC_BUNDLE_PRICE_UPDATED' and e[3].count==1 and e[3].productID==8201 and e[3].hasData and e[4].event=='STORE_PURCHASE_LIST_UPDATED' and e[4].count==0 and e[4].ready and e[4].progress==false and e[5].event=='STORE_PRODUCTS_UPDATED' and not e[5].ready and e[6].event=='STORE_PURCHASE_LIST_UPDATED' and not e[6].ready and not e[6].progress)",
    )
    .unwrap();
}

#[test]
fn retail_store_consumers_resolve_host_catalog_records() {
    let env = load_store_ui();
    seed_catalog(&env);
    env.exec_rilua_secure(CATALOG_CONSUMER)
        .expect("secure StoreUI consumer must resolve host records");
}

#[test]
fn retail_store_currency_consumer_uses_shared_data() {
    let env = load_store_ui();
    seed_catalog(&env);
    env.exec_rilua_secure(CURRENCY_CONSUMER)
        .expect("secure StoreUI consumer must read sharedData");
    env.state().borrow_mut().store_glue.store_secure.currency_id = 62;
    env.exec_rilua_secure("local info=StoreFrame_CurrencyInfo(); assert(FormatCurrencyStringShort == 'short2-%s' and info.browseHasStar)")
        .expect("StoreFrame must consume the second host currency row");
    env.state().borrow_mut().store_glue.store_secure.currency_id = 63;
    env.exec_rilua_secure("assert(C_StoreSecure.GetCurrencyInfo() == nil)")
        .expect("a currency metadata miss must return nil");
}

#[test]
fn retail_store_empty_defaults_reach_original_catalog_consumer() {
    let env = load_store_ui();
    env.exec_rilua_secure(
        "assert(CatalogShopMixin:IsLoading(), 'CatalogShop must remain loading with empty host lists'); local a,b,c,d,e,f,g=C_StoreSecure.IsAvailable(), C_StoreSecure.HasProductList(), C_StoreSecure.HasPurchaseList(), C_StoreSecure.HasDistributionList(), C_StoreSecure.HasPurchaseInProgress(), C_StoreSecure.GetCurrencyID(), C_StoreSecure.GetLastProductListResponseError(); assert(not a and not b and not c and not d and not e and f == 0 and g == 0); assert(C_StoreSecure.GetCurrencyInfo() == nil and #C_StoreSecure.GetProductGroups() == 0)",
    )
    .expect("original CatalogShop consumer must observe empty-default Store inputs");
}

#[test]
fn retail_store_response_events_publish_updated_state() {
    let env = load_store_ui();
    seed_catalog(&env);
    enable_store_callbacks(&env);
    env.exec_rilua_secure(STORE_EVENT_CONSUMER).unwrap();
    receive_success_responses(&env);
    assert_success_state(&env);
    receive_failure_responses(&env);
    assert_failure_state(&env);
    assert_response_events(&env);
}

fn prepare_connected_requests(env: &WowLuaEnv) {
    let mut sim = env.state().borrow_mut();
    sim.store_glue.connection_present = true;
    sim.store_glue.store_secure.request_time_ms = 0;
}

fn assert_request_results(env: &WowLuaEnv) {
    let sim = env.state().borrow();
    let state = &sim.store_glue.store_secure;
    assert_eq!(state.product_list_request_count, 6);
    assert_eq!(sim.store_glue.submissions.len(), 7);
    assert_eq!(sim.store_glue.submissions.iter().filter(|r| matches!(r, StoreRequest::ProductList)).count(), 4);
    assert_eq!(sim.store_glue.submissions.iter().filter(|r| matches!(r, StoreRequest::PurchaseList)).count(), 1);
    assert_eq!(sim.store_glue.submissions.iter().filter(|r| matches!(r, StoreRequest::DynamicPriceInfo(_))).count(), 2);
}

#[test]
fn retail_store_requests_use_connection_and_host_catalog_inputs() {
    let env = load_store_ui();
    seed_catalog(&env);
    env.exec_rilua_secure("assert(select('#', C_StoreSecure.GetProductList()) == 0 and select('#', C_StoreSecure.GetPurchaseList()) == 0)").unwrap();
    let disconnected = env.state().borrow();
    assert_eq!(disconnected.store_glue.store_secure.product_list_request_count, 1);
    assert!(disconnected.store_glue.submissions.is_empty());
    drop(disconnected);
    prepare_connected_requests(&env);
    env.exec_rilua_secure("assert(select('#', C_StoreSecure.RequestAllDynamicPriceInfo()) == 0 and C_StoreSecure.IsDynamicBundle(8201) and C_StoreSecure.IsDynamicBundle(8202) and not C_StoreSecure.IsDynamicBundle(4999) and not C_StoreSecure.HasDynamicPriceData(8201) and not C_StoreSecure.HasDynamicPriceData(4999))").unwrap();
    for _ in 0..5 { env.exec_rilua_secure("C_StoreSecure.GetProductList()").unwrap(); }
    env.exec_rilua_secure("C_StoreSecure.GetPurchaseList()").unwrap();
    assert_request_results(&env);
    env.state().borrow_mut().store_glue.store_secure.request_time_ms = 2000;
    env.exec_rilua_secure("C_StoreSecure.GetProductList()").unwrap();
    let sim = env.state().borrow();
    assert_eq!(sim.store_glue.store_secure.product_list_request_count, 1);
    assert_eq!(sim.store_glue.submissions.len(), 8);
    assert_eq!(sim.store_glue.submissions.iter().filter(|r| matches!(r, StoreRequest::ProductList)).count(), 5);
}

#[test]
fn retail_store_list_queries_use_two_rows_and_absence() {
    let env = load_store_ui();
    seed_purchase_progress(&env);
    env.exec_rilua_secure("assert(C_StoreSecure.HasPurchaseInProgress())").unwrap();
    env.state().borrow_mut().store_glue.store_secure.purchase_list.clear();
    env.exec_rilua_secure("assert(not C_StoreSecure.HasPurchaseInProgress())").unwrap();
    {
        let mut sim = env.state().borrow_mut();
        sim.store_glue.store_secure.distribution_list_ready = Some(true);
        sim.store_glue.connection_present = false;
    }
    env.exec_rilua_secure("assert(not C_StoreSecure.HasDistributionList())").unwrap();
    env.state().borrow_mut().store_glue.connection_present = true;
    env.exec_rilua_secure("assert(C_StoreSecure.HasDistributionList())").unwrap();
}
