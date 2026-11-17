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
    assert(#filtered == 2 and filtered[1] == 8101 and filtered[2] == 8102, "StoreFrame_FilterEntries must keep both host entries")
    local scrollBox = {}
    function scrollBox:SetDataProvider(provider) self.provider = provider end
    CategoryTreeScrollContainerMixin.UpdateCategories({ScrollBox = scrollBox})
    assert(scrollBox.provider ~= nil, "the category consumer must publish its tree provider")
    local groups = C_StoreSecure.GetProductGroups()
    assert(#groups == 2 and groups[1].groupID == 4102 and groups[2].groupID == 4101, "group rows must keep host order")
    assert(not StoreFrame_IsProductGroupDisabled(4101), "known host group 4101 must be enabled")
    assert(not StoreFrame_IsProductGroupDisabled(4102), "known host group 4102 must be enabled")
    assert(StoreFrame_IsProductGroupDisabled(4999), "an absent group must be disabled")
    local group = C_StoreSecure.GetProductGroupInfo(4101)
    assert(group.groupName == "Host Alpha" and group.displayType == 1 and group.flags == 0, "group info must map host fields")
    assert(select("#", C_StoreSecure.GetProductGroupInfo(4999)) == 0, "an absent group info must return no values")
    local entries = C_StoreSecure.GetProducts(4101)
    assert(#entries == 2 and entries[1] == 8102 and entries[2] == 8101, "product entries must retain host order")
    local hidden = C_StoreSecure.GetProducts(4102, true)
    assert(#hidden == 2 and hidden[1] == 8103 and hidden[2] == 8105, "includeHidden must expose the hidden host entry")
    assert(#C_StoreSecure.GetProducts(4999) == 0, "an absent group must have no entries")
    assert(C_StoreSecure.GetProducts(4102)[2] == nil, "hidden entries must stay excluded by default")
    local entry = C_StoreSecure.GetEntryInfo(8101)
    local secondEntry = C_StoreSecure.GetEntryInfo(8102)
    assert(entry.productID == 8201 and entry.groupID == 4101 and entry.bannerType == 1, "entry info must map host product, group, and banner")
    assert(entry.alreadyOwned == false and entry.sharedData.name == "Host One", "entry info must include shared product data")
    assert(secondEntry.productID == 8202 and secondEntry.sharedData.name == "Host Two", "second backed entry must map its own product data")
    assert(C_StoreSecure.GetEntryInfo(4999) == nil, "an absent entry must return nil")
    local first = C_StoreSecure.GetProductInfo(8201)
    local second = C_StoreSecure.GetProductInfo(8202)
    assert(first.productID == 8201 and second.productID == 8202, "product info must map both host products")
    assert(first.sharedData.cards[1].itemModifiedAppearanceIDs[2] == 10, "product cards must preserve appearance IDs")
    assert(first.sharedData.deliverables[1].owned == false, "product deliverables must preserve host ownership")
    local firstColor = first.sharedData.overrideTextColor
    local firstR, firstG, firstB = firstColor:GetRGB()
    assert(firstColor.a == nil and math.abs(firstR - 128 / 255) < 0.000001
        and math.abs(firstG - 64 / 255) < 0.000001
        and math.abs(firstB - 1) < 0.000001, "first override color must use the ColorMixin RGB values only")
    local secondColor = second.sharedData.overrideTextColor
    local secondR, secondG, secondB = secondColor:GetRGB()
    assert(secondColor.a == nil and math.abs(secondR - 17 / 255) < 0.000001
        and math.abs(secondG - 128 / 255) < 0.000001
        and math.abs(secondB) < 0.000001, "second override color must use its own ColorMixin RGB values")
    assert(C_StoreSecure.GetProductInfo(8203).sharedData.overrideTextColor == nil, "an absent override color must stay nil")
    assert(C_StoreSecure.GetProductInfo(4999) == nil, "an absent product must return nil")
    return true
"##;

const CURRENCY_CONSUMER: &str = r#"
    local storeInfo = StoreFrame_CurrencyInfo()
    assert(StoreFrame_CurrencyFormatShort(10, 0) == "short-10", "StoreFrame must format with the host short pattern")
    assert(storeInfo.hideBrowseNotice == true)
    local currencyInfo = C_StoreSecure.GetCurrencyInfo()
    assert(type(currencyInfo.sharedData) == "table")
    assert(currencyInfo.currencyFields == nil)
    assert(currencyInfo.sharedData.formatLong == "long-%s")
    assert(currencyInfo.sharedData.licenseAcceptText == "terms-61")
    return true
"#;

const PURCHASE_EVENT_OBSERVER: &str = r##"
    STORE_PURCHASE_EVENT_PAYLOAD_COUNTS = {}
    local listener = CreateFrame("Frame")
    listener:RegisterEvent("STORE_PURCHASE_LIST_UPDATED")
    listener:SetScript("OnEvent", function(_, _, ...)
        table.insert(STORE_PURCHASE_EVENT_PAYLOAD_COUNTS, select("#", ...))
    end)
"##;

const STORE_EVENT_OBSERVER: &str = r##"
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

const IMMEDIATE_SUCCESS_EVENTS: &str = "local e=STORE_CATALOG_EVENTS; assert(#e==3 and e[1].event=='STORE_STATUS_CHANGED' and e[1].count==0 and e[1].available and e[2].event=='STORE_PRODUCTS_UPDATED' and e[2].count==0 and e[2].ready and e[2].currency==62 and e[2].groups==2 and e[3].event=='DYNAMIC_BUNDLE_PRICE_UPDATED' and e[3].count==1 and e[3].productID==8201 and e[3].hasData)";

const QUEUED_SUCCESS_EVENT: &str = "local e=STORE_CATALOG_EVENTS; assert(#e==4 and e[4].event=='STORE_PURCHASE_LIST_UPDATED' and e[4].count==0 and e[4].ready and e[4].progress==false)";

const IMMEDIATE_FAILURE_EVENT: &str = "local e=STORE_CATALOG_EVENTS; assert(#e==5 and e[5].event=='STORE_PRODUCTS_UPDATED' and e[5].count==0 and not e[5].ready)";

const QUEUED_FAILURE_EVENT: &str = "local e=STORE_CATALOG_EVENTS; assert(#e==6 and e[6].event=='STORE_PURCHASE_LIST_UPDATED' and e[6].count==0 and not e[6].ready and not e[6].progress)";

const SUCCESS_STATE: &str = "assert(C_StoreSecure.IsAvailable() and C_StoreSecure.HasProductList() and C_StoreSecure.HasPurchaseList() and C_StoreSecure.GetCurrencyID() == 62 and not C_StoreSecure.HasPurchaseInProgress() and C_StoreSecure.GetProductInfo(8201).sharedData.name == 'Host One' and C_StoreSecure.HasDynamicPriceData(8201))";

const FAILURE_STATE: &str = "assert(not C_StoreSecure.HasProductList() and not C_StoreSecure.HasPurchaseList() and C_StoreSecure.IsRegionLocked() and C_StoreSecure.GetLastProductListResponseError() == 3 and C_StoreSecure.GetCurrencyID() == 62 and #C_StoreSecure.GetProductGroups() == 0 and not C_StoreSecure.HasPurchaseInProgress())";

const ZERO_VALUE_LIST_REQUESTS: &str = "assert(select('#', C_StoreSecure.GetProductList()) == 0 and select('#', C_StoreSecure.GetPurchaseList()) == 0)";

const DYNAMIC_BUNDLE_QUERIES: &str = "assert(select('#', C_StoreSecure.RequestAllDynamicPriceInfo()) == 0 and C_StoreSecure.IsDynamicBundle(8201) and C_StoreSecure.IsDynamicBundle(8202) and not C_StoreSecure.IsDynamicBundle(8203) and not C_StoreSecure.IsDynamicBundle(4999) and not C_StoreSecure.HasDynamicPriceData(8201) and not C_StoreSecure.HasDynamicPriceData(4999))";

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

fn load_store_ui_connected() -> WowLuaEnv {
    let ui_dir = crate::common::blizzard_addon_harness::blizzard_ui_dir();
    let env = crate::common::blizzard_addon_harness::new_blizzard_addon_env(&ui_dir);
    env.state().borrow_mut().store_glue.connection_present = true;
    let loaded = crate::common::blizzard_addon_harness::load_blizzard_addon_closure_into_env(
        &env,
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

const HOST_ONE_COLOR: [u8; 3] = [128, 64, 255];
const HOST_TWO_COLOR: [u8; 3] = [17, 128, 0];

fn product(
    id: i32,
    name: &str,
    dynamic: bool,
    visible: bool,
    card_type: u32,
) -> StoreProductInput {
    let mut shared_data = shared_data(name);
    shared_data.card_type = card_type;
    shared_data.override_text_color = match id {
        8201 => Some(HOST_ONE_COLOR),
        8202 => Some(HOST_TWO_COLOR),
        _ => None,
    };
    StoreProductInput {
        product_id: id,
        supported: true,
        session_eligible: true,
        display_eligible: visible,
        dynamic_bundle: dynamic,
        shared_data,
    }
}

fn entry(id: i32, product_id: i32, group_id: u32, order: i32, hidden: bool) -> StoreEntryInput {
    StoreEntryInput {
        entry_id: id,
        product_id,
        group_id,
        order,
        banner_type: 1,
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

fn seed_products(
    store: &mut wow_ui_sim::c_api::store_catalog_model::StoreSecureState,
    card_type: u32,
) {
    store.catalog.products = vec![
        product(8201, "Host One", true, true, card_type),
        product(8202, "Host Two", true, true, card_type),
        product(8203, "Host Three", false, true, card_type),
        product(8204, "Hidden", false, true, card_type),
        product(239, "Excluded", false, true, card_type),
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
    let card_type: i64 = env
        .eval("return Enum.BattlepayCardType.MediumCardWithBuyButton")
        .expect("StoreUI card type must be available");
    let card_type = u32::try_from(card_type).expect("StoreUI card type must be non-negative");
    let mut sim = env.state().borrow_mut();
    let store = &mut sim.store_glue.store_secure;
    seed_groups(store);
    seed_products(store, card_type);
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
    let purchase = StorePurchaseListResponse {
        result_code: 0,
        rows: vec![purchase_input(false), purchase_input(true)],
    };
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

fn active_purchase_input(expires_at: f64) -> StorePurchaseInput {
    StorePurchaseInput {
        blocked: false,
        status: 1,
        expires_at,
        client_time_adjustment: 0.0,
    }
}

fn assert_catalog_shop_loading(env: &WowLuaEnv, expected: bool) {
    let expected = if expected { "true" } else { "false" };
    env.exec_rilua_secure(&format!(
        "assert(CatalogShopMixin:IsLoading() == {expected})"
    ))
    .expect("original CatalogShop loading consumer must match the supplied list state");
}

fn assert_store_frame_loading(env: &WowLuaEnv, expected: bool) {
    let expected = if expected { "true" } else { "false" };
    env.exec_rilua_secure(&format!(
        "assert(StoreFrame_IsLoading(StoreFrame) == {expected})"
    ))
    .expect("original StoreFrame loading consumer must match the supplied list state");
}

fn assert_transaction_notice_state(env: &WowLuaEnv, expected: bool) {
    let assertion = if expected {
        "StoreFrame.Notice:IsShown() and StoreFrame.Notice.Title:GetText() == BLIZZARD_STORE_TRANSACTION_IN_PROGRESS"
    } else {
        "not StoreFrame.Notice:IsShown() or StoreFrame.Notice.Title:GetText() ~= BLIZZARD_STORE_TRANSACTION_IN_PROGRESS"
    };
    env.exec_rilua_secure(&format!("assert({assertion})"))
        .expect("original StoreFrame event consumer must reflect the purchase-list state");
}

fn assert_transaction_notice(env: &WowLuaEnv, expected: bool) {
    let assertion = if expected {
        "StoreFrame.Notice:IsShown() and StoreFrame.Notice.Title:GetText() == BLIZZARD_STORE_TRANSACTION_IN_PROGRESS"
    } else {
        "not StoreFrame.Notice:IsShown() or StoreFrame.Notice.Title:GetText() ~= BLIZZARD_STORE_TRANSACTION_IN_PROGRESS"
    };
    env.exec_rilua_secure(&format!(
        "StoreFrame.ErrorFrame:Hide(); StoreFrame_UpdateActivePanel(StoreFrame); assert({assertion})"
    ))
    .expect("original StoreFrame panel consumer must reflect purchase progress");
}

fn assert_store_frame_products(env: &WowLuaEnv, category_id: u32, expected: [(i32, &str); 2]) {
    let [(first_id, first_name), (second_id, second_name)] = expected;
    env.exec_rilua_secure(&format!(
        r#"
        local selected = StoreFrame_GetSelectedCategoryID()
        if selected ~= {category_id} then
            error("selected category " .. tostring(selected))
        end
        local count = 0
        local namesByEntry = {{}}
        for card in StoreFrame.productCardPoolCollection:EnumerateActive() do
            count = count + 1
            namesByEntry[card:GetID()] = card.ProductName:GetText()
        end
        if count ~= 2 then
            error("active product cards " .. count)
        end
        if namesByEntry[{first_id}] ~= "{first_name}" or namesByEntry[{second_id}] ~= "{second_name}" then
            error("card names " .. tostring(namesByEntry[{first_id}]) .. ", " .. tostring(namesByEntry[{second_id}]))
        end
        "#,
    ))
    .expect("original StoreFrame consumer must publish both selected host products");
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
    env.exec_rilua_secure("local info=StoreFrame_CurrencyInfo(); assert(StoreFrame_CurrencyFormatShort(10, 0) == 'short2-10' and info.browseHasStar)")
        .expect("StoreFrame must consume the second host currency row");
    env.state().borrow_mut().store_glue.store_secure.currency_id = 63;
    env.exec_rilua_secure("local info=StoreFrame_CurrencyInfo(); assert(next(info) == nil)")
        .expect("StoreFrame must expose its empty state when currency metadata is absent");
}

fn connect_with_distribution_ready(env: &WowLuaEnv) {
    let mut sim = env.state().borrow_mut();
    sim.store_glue.connection_present = true;
    sim.store_glue.store_secure.distribution_list_ready = Some(true);
}

fn receive_catalog_response(env: &WowLuaEnv, currency_id: u32) {
    let catalog = env.state().borrow().store_glue.store_secure.catalog.clone();
    wow_ui_sim::c_api::store_catalog_model::receive_product_list(
        env,
        StoreProductListResponse {
            result_code: 0,
            currency_id,
            catalog,
        },
    )
    .unwrap();
}

#[test]
fn retail_store_empty_defaults_reach_original_catalog_consumer() {
    let env = load_store_ui();
    connect_with_distribution_ready(&env);
    enable_store_callbacks(&env);
    wow_ui_sim::c_api::store_catalog_model::receive_purchase_list(
        &env,
        StorePurchaseListResponse {
            result_code: 0,
            rows: vec![purchase_input(false), purchase_input(true)],
        },
    )
    .unwrap();
    assert_catalog_shop_loading(&env, true);

    seed_catalog(&env);
    for currency_id in [61, 62] {
        receive_catalog_response(&env, currency_id);
        assert_catalog_shop_loading(&env, false);
    }

    receive_failure_responses(&env);
    assert_catalog_shop_loading(&env, true);
}

#[test]
fn retail_store_response_events_publish_updated_state() {
    let env = load_store_ui();
    seed_catalog(&env);
    enable_store_callbacks(&env);
    env.exec_rilua_secure(
        "StoreFrame:Show(); StoreFrame_SetSelectedCategoryID(4101); assert(StoreFrame:IsShown())",
    )
    .expect("original StoreFrame OnShow consumer must run");

    receive_success_responses(&env);
    env.exec_rilua_secure(
        "assert(StoreFrame_DoesProductGroupHavePurchasableItems(4101)); assert(StoreFrame_DoesProductGroupHavePurchasableItems(4102)); assert(not StoreFrame_DoesProductGroupHavePurchasableItems(4999))",
    )
    .expect("original StoreFrame consumer must map two groups and an absent group to their entries");
    env.exec_rilua_secure("StoreFrame_SetCategory()")
        .expect("original StoreFrame consumer must publish the selected product entries");
    assert_store_frame_products(&env, 4101, [(8102, "Host Two"), (8101, "Host One")]);
    env.exec_rilua_secure(
        "StoreFrame_SetSelectedCategoryID(4999); StoreFrame_UpdateSelectedCategory(); assert(StoreFrame_GetSelectedCategoryID() ~= nil and StoreFrame_GetSelectedCategoryID() ~= 4999); StoreFrame_SetCategory()",
    )
    .expect("original StoreFrame consumer must recover from an absent category");

    receive_failure_responses(&env);
    env.fire_on_update(1.0 / 60.0)
        .expect("queued purchase-list failure must reach StoreFrame on update");
    env.exec_rilua_secure(
        "assert(StoreFrame.Notice:IsShown() and StoreFrame.Notice.Title:GetText() == BLIZZARD_STORE_REGION_LOCKED)",
    )
    .expect("original StoreFrame event consumer must render the supplied region-lock outcome");
}

fn request_count(env: &WowLuaEnv, matches_kind: fn(&StoreRequest) -> bool) -> usize {
    env.state()
        .borrow()
        .store_glue
        .submissions
        .iter()
        .filter(|request| matches_kind(request))
        .count()
}

fn dynamic_price_requests(env: &WowLuaEnv) -> Vec<i32> {
    env.state()
        .borrow()
        .store_glue
        .submissions
        .iter()
        .filter_map(|request| match request {
            StoreRequest::DynamicPriceInfo(product_id) => Some(*product_id),
            _ => None,
        })
        .collect()
}

fn product_list_request_count(env: &WowLuaEnv) -> u32 {
    env.state().borrow().store_glue.store_secure.product_list_request_count
}

#[test]
fn retail_store_requests_use_connection_and_host_catalog_inputs() {
    let env = load_store_ui_connected();
    seed_catalog(&env);
    enable_store_callbacks(&env);
    env.exec_rilua_secure("StoreFrame:Show(); assert(StoreFrame:IsShown())")
        .expect("original StoreFrame OnShow consumer must request its lists");

    wow_ui_sim::c_api::store_catalog_model::receive_availability(&env, true).unwrap();
    receive_catalog_response(&env, 61);

    assert_eq!(product_list_request_count(&env), 1, "StoreFrame OnShow must request one product list");
    assert_eq!(request_count(&env, |request| matches!(request, StoreRequest::ProductList)), 1);
    assert_eq!(
        request_count(&env, |request| matches!(request, StoreRequest::PurchaseList)),
        1,
        "StoreFrame OnLoad must request the purchase list while connected"
    );
    let dynamic_product_ids = dynamic_price_requests(&env);
    assert_eq!(dynamic_product_ids, vec![8201, 8202]);
    assert!(!dynamic_product_ids.contains(&4999), "an absent product must not be requested");
}

#[test]
fn retail_store_list_queries_use_two_rows_and_absence() {
    let env = load_store_ui();
    seed_catalog(&env);
    {
        let mut sim = env.state().borrow_mut();
        sim.store_glue.connection_present = true;
        let store = &mut sim.store_glue.store_secure;
        store.available = true;
        store.has_product_list = true;
        store.has_purchase_list = true;
    }
    seed_purchase_progress(&env);

    assert_store_frame_loading(&env, true);
    env.state().borrow_mut().store_glue.store_secure.distribution_list_ready = Some(false);
    assert_store_frame_loading(&env, true);
    env.state().borrow_mut().store_glue.store_secure.distribution_list_ready = Some(true);
    assert_store_frame_loading(&env, false);

    for expires_at in [1200.0, 1300.0] {
        env.state().borrow_mut().store_glue.store_secure.purchase_list =
            vec![active_purchase_input(expires_at)];
        assert_transaction_notice(&env, true);
    }

    env.state().borrow_mut().store_glue.store_secure.purchase_list = vec![purchase_input(true)];
    assert_transaction_notice(&env, false);
    env.state().borrow_mut().store_glue.store_secure.purchase_list.clear();
    assert_transaction_notice(&env, false);
}

fn prepare_purchase_consumer(env: &WowLuaEnv) {
    seed_catalog(env);
    enable_store_callbacks(env);
    {
        let mut sim = env.state().borrow_mut();
        sim.store_glue.connection_present = true;
        let store = &mut sim.store_glue.store_secure;
        store.available = true;
        store.has_product_list = true;
        store.has_purchase_list = true;
        store.distribution_list_ready = Some(true);
    }
    seed_purchase_progress(env);
    assert_transaction_notice(env, true);
    env.exec_rilua_secure(PURCHASE_EVENT_OBSERVER)
        .expect("purchase-event observer must register in the consumer environment");
}

fn receive_two_purchase_responses(env: &WowLuaEnv) {
    for rows in [vec![purchase_input(false), purchase_input(true)], Vec::new()] {
        wow_ui_sim::c_api::store_catalog_model::receive_purchase_list(
            env,
            StorePurchaseListResponse { result_code: 0, rows },
        )
        .unwrap();
    }
}

fn assert_coalesced_pending_state(env: &WowLuaEnv) {
    env.exec_rilua_secure("assert(#STORE_PURCHASE_EVENT_PAYLOAD_COUNTS == 0)")
        .expect("queued purchase responses must not dispatch before the tick");
    let sim = env.state().borrow();
    let store = &sim.store_glue.store_secure;
    assert!(store.purchase_list_notification_pending, "the pending notification must coalesce both responses");
    assert_eq!(store.purchase_list.len(), 0, "the latest response state must be visible before dispatch");
}

fn assert_one_zero_payload_purchase_event(env: &WowLuaEnv, context: &str) {
    env.exec_rilua_secure(
        "assert(#STORE_PURCHASE_EVENT_PAYLOAD_COUNTS == 1 and STORE_PURCHASE_EVENT_PAYLOAD_COUNTS[1] == 0)",
    )
    .expect(context);
}

#[test]
fn retail_store_purchase_list_updates_coalesce_after_state_write() {
    let env = load_store_ui();
    prepare_purchase_consumer(&env);
    receive_two_purchase_responses(&env);
    assert_coalesced_pending_state(&env);

    env.fire_on_update(1.0 / 60.0)
        .expect("pending purchase notification must dispatch on update");
    assert_one_zero_payload_purchase_event(&env, "two queued updates must deliver one zero-payload event");
    assert!(
        !env.state().borrow().store_glue.store_secure.purchase_list_notification_pending,
        "dispatch must clear the pending marker"
    );
    assert_transaction_notice_state(&env, false);

    env.fire_on_update(1.0 / 60.0)
        .expect("a later update must not replay a delivered purchase event");
    assert_one_zero_payload_purchase_event(
        &env,
        "a later update must not replay the coalesced zero-payload event",
    );
}

fn receive_event_contract_success(env: &WowLuaEnv) {
    wow_ui_sim::c_api::store_catalog_model::receive_availability(env, true).unwrap();
    receive_catalog_response(env, 62);
    wow_ui_sim::c_api::store_catalog_model::receive_dynamic_price(env, 8201, vec![1, 2]).unwrap();
    let purchase = StorePurchaseListResponse { result_code: 0, rows: Vec::new() };
    wow_ui_sim::c_api::store_catalog_model::receive_purchase_list(env, purchase).unwrap();
}

fn assert_store_lua(env: &WowLuaEnv, code: &str, context: &str) {
    env.exec_rilua_secure(code).expect(context);
}

#[test]
fn retail_store_response_events_carry_payloads_after_state_writes() {
    let env = load_store_ui();
    seed_catalog(&env);
    enable_store_callbacks(&env);
    assert_store_lua(&env, STORE_EVENT_OBSERVER, "Store event observer must register");

    receive_event_contract_success(&env);
    assert_store_lua(&env, IMMEDIATE_SUCCESS_EVENTS, "status, product and dynamic-price events must follow their state writes");
    env.fire_on_update(1.0 / 60.0).expect("queued success purchase event must dispatch");
    assert_store_lua(&env, QUEUED_SUCCESS_EVENT, "the queued purchase event must follow its state write");
    assert_store_lua(&env, SUCCESS_STATE, "success responses must publish their state");

    receive_failure_responses(&env);
    assert_store_lua(&env, IMMEDIATE_FAILURE_EVENT, "a failed product list must publish cleared readiness");
    env.fire_on_update(1.0 / 60.0).expect("queued failure purchase event must dispatch");
    assert_store_lua(&env, QUEUED_FAILURE_EVENT, "the failed purchase list must publish cleared readiness");
    assert_store_lua(&env, FAILURE_STATE, "error 3 must keep currency, clear groups and set region lock");
}

fn assert_disconnected_product_request(env: &WowLuaEnv) {
    assert_store_lua(env, ZERO_VALUE_LIST_REQUESTS, "list requests must return zero values");
    assert_eq!(product_list_request_count(env), 1, "a disconnected request still counts");
    assert!(env.state().borrow().store_glue.submissions.is_empty(), "nothing is submitted while disconnected");
}

fn assert_limited_requests(env: &WowLuaEnv) {
    assert_eq!(product_list_request_count(env), 6);
    assert_eq!(env.state().borrow().store_glue.submissions.len(), 7);
    assert_eq!(request_count(env, |request| matches!(request, StoreRequest::ProductList)), 4);
    assert_eq!(request_count(env, |request| matches!(request, StoreRequest::PurchaseList)), 1);
    assert_eq!(dynamic_price_requests(env), vec![8201, 8202]);
}

#[test]
fn retail_store_product_list_rate_limit_and_dynamic_bundles() {
    let env = load_store_ui();
    seed_catalog(&env);
    assert_disconnected_product_request(&env);
    prepare_connected_requests(&env);
    assert_store_lua(&env, DYNAMIC_BUNDLE_QUERIES, "dynamic bundle markers and price data must follow host input");
    for _ in 0..5 {
        assert_store_lua(&env, "C_StoreSecure.GetProductList()", "product-list request must run");
    }
    assert_store_lua(&env, "C_StoreSecure.GetPurchaseList()", "purchase-list request must run");
    assert_limited_requests(&env);

    env.state().borrow_mut().store_glue.store_secure.request_time_ms = 2000;
    assert_store_lua(&env, "C_StoreSecure.GetProductList()", "product-list request must run after reset");
    assert_eq!(product_list_request_count(&env), 1, "the counter resets after 2000 ms");
    assert_eq!(env.state().borrow().store_glue.submissions.len(), 8);
    assert_eq!(request_count(&env, |request| matches!(request, StoreRequest::ProductList)), 5);
}

fn prepare_connected_requests(env: &WowLuaEnv) {
    let mut sim = env.state().borrow_mut();
    sim.store_glue.connection_present = true;
    sim.store_glue.store_secure.request_time_ms = 0;
}

#[test]
fn retail_store_distribution_list_requires_connection() {
    let env = load_store_ui();
    {
        let mut sim = env.state().borrow_mut();
        sim.store_glue.store_secure.distribution_list_ready = Some(true);
        sim.store_glue.connection_present = false;
    }
    assert_store_lua(&env, "assert(not C_StoreSecure.HasDistributionList())", "no connection means no distribution list");
    env.state().borrow_mut().store_glue.connection_present = true;
    assert_store_lua(&env, "assert(C_StoreSecure.HasDistributionList())", "supplied distribution state must be visible when connected");
}
