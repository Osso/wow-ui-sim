//! Retail C_StoreSecure catalog and list methods, backed by neutral host inputs.
use super::c_store_glue::{Member, publish_namespace, read_argument_at, restricted_call};
use super::helpers::{bad_argument, parse_numeric_string, set_table_array};
use super::store_catalog_model::{StoreCurrencyInfo, StoreProductSharedData};
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, call_function_state, create_string, create_table,
    table_set_static,
};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

const PRODUCT_LIST_LIMIT: u32 = 5;
const PRODUCT_LIST_INTERVAL_MS: u32 = 2000;
const NO_TEXTURE_ID: u32 = 0;
const RGB_BYTE_MAX: f32 = 255.0;
const GROUP_INFO_USAGE: &str = "Usage: local info = C_StoreSecure.GetProductGroupInfo(groupID)";
const PRODUCTS_USAGE: &str =
    "Usage: local entryIDs = C_StoreSecure.GetProducts(groupID [, includeHiddenProducts])";
const ENTRY_INFO_USAGE: &str = "Usage: local entryInfo = C_StoreSecure.GetEntryInfo(entryID)";
const PRODUCT_INFO_USAGE: &str =
    "Usage: local productInfo = C_StoreSecure.GetProductInfo(productID)";
const DYNAMIC_BUNDLE_USAGE: &str =
    "Usage: local isDynamicBundle = C_StoreSecure.IsDynamicBundle(productID)";
const DYNAMIC_PRICE_USAGE: &str =
    "Usage: local dynamicPriceDataAvailable = C_StoreSecure.HasDynamicPriceData(productID)";

const MEMBERS: &[Member] = &[
    ("GetProductGroups", get_product_groups),
    ("GetProductGroupInfo", get_product_group_info),
    ("GetProducts", get_products),
    ("GetEntryInfo", get_entry_info),
    ("GetProductInfo", get_product_info),
    ("IsAvailable", is_available),
    ("HasPurchaseList", has_purchase_list),
    ("HasProductList", has_product_list),
    ("HasDistributionList", has_distribution_list),
    ("HasPurchaseInProgress", has_purchase_in_progress),
    ("IsRegionLocked", is_region_locked),
    (
        "GetLastProductListResponseError",
        get_last_product_list_response_error,
    ),
    ("GetCurrencyID", get_currency_id),
    ("GetCurrencyInfo", get_currency_info),
    ("IsDynamicBundle", is_dynamic_bundle),
    ("HasDynamicPriceData", has_dynamic_price_data),
    ("RequestAllDynamicPriceInfo", request_all_dynamic_price_info),
    ("GetProductList", get_product_list),
    ("GetPurchaseList", get_purchase_list),
];

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if ACTIVE != ClientProfile::Retail {
        return Ok(());
    }
    publish_namespace(state, "C_StoreSecure", MEMBERS)
}

fn pending_error(state: &mut LuaState, method: &str) -> LuaResult<()> {
    let message = borrow_state_mut(state)?.store_glue.pending_error.take();
    if let Some(message) = message {
        return Err(runtime_error(format!("{method}(): {message}")));
    }
    Ok(())
}

fn product_group_rows(state: &LuaState) -> LuaResult<Vec<(u32, u32)>> {
    Ok(borrow_state(state)?
        .store_glue
        .store_secure
        .product_group_rows())
}

fn get_product_groups(state: &mut LuaState) -> LuaResult<u32> {
    if restricted_call(state)? {
        return Ok(0);
    }
    let groups = product_group_rows(state)?;
    pending_error(state, "GetProductGroups")?;
    let result = create_table(state);
    state.push(result);
    for (index, (group_id, parent_id)) in groups.into_iter().enumerate() {
        append_group_row(state, result, index, group_id, parent_id);
    }
    Ok(1)
}

fn append_group_row(
    state: &mut LuaState,
    result: Val,
    index: usize,
    group_id: u32,
    parent_id: u32,
) {
    let saved_top = state.top;
    let row = create_table(state);
    state.push(row);
    set_u32(state, row, "groupID", group_id);
    set_u32(state, row, "parentGroupID", parent_id);
    set_table_array(state, result, index as i64 + 1, row);
    state.top = saved_top;
}

fn get_product_group_info(state: &mut LuaState) -> LuaResult<u32> {
    if restricted_call(state)? {
        return Ok(0);
    }
    let group_id = read_signed_id(state, "GetProductGroupInfo", GROUP_INFO_USAGE)?;
    let info = borrow_state(state)?
        .store_glue
        .store_secure
        .group_info(group_id);
    pending_error(state, "GetProductGroupInfo")?;
    let Some(info) = info else { return Ok(0) };
    let result = create_group_info(state, &info);
    state.push(result);
    Ok(1)
}

fn create_group_info(
    state: &mut LuaState,
    info: &super::store_catalog_model::StoreProductGroupInput,
) -> Val {
    let result = create_table(state);
    state.push(result);
    set_text(state, result, "groupName", &info.name);
    if let Some(texture) = info.texture.filter(|texture| *texture != NO_TEXTURE_ID) {
        set_u32(state, result, "texture", texture);
    }
    set_i32(state, result, "displayType", info.display_type);
    set_u32(state, result, "flags", info.flags);
    set_u32(state, result, "parentProductGroupID", info.parent_group_id);
    if let Some(tooltip) = &info.disabled_tooltip {
        set_text(state, result, "disabledTooltip", tooltip);
    }
    state.top -= 1;
    result
}

fn get_products(state: &mut LuaState) -> LuaResult<u32> {
    if restricted_call(state)? {
        return Ok(0);
    }
    let group_id = read_unsigned_id(state, "GetProducts", PRODUCTS_USAGE)?;
    let include_hidden = read_optional_truthy(state, 2, "GetProducts", PRODUCTS_USAGE)?;
    let entries = borrow_state(state)?
        .store_glue
        .store_secure
        .product_entry_ids(group_id, include_hidden);
    pending_error(state, "GetProducts")?;
    let result = create_table(state);
    state.push(result);
    for (index, entry_id) in entries.into_iter().enumerate() {
        set_table_array(
            state,
            result,
            index as i64 + 1,
            Val::Num(f64::from(entry_id)),
        );
    }
    Ok(1)
}

fn get_entry_info(state: &mut LuaState) -> LuaResult<u32> {
    if restricted_call(state)? {
        return Ok(0);
    }
    let entry_id = read_signed_id(state, "GetEntryInfo", ENTRY_INFO_USAGE)?;
    let entry = borrow_state(state)?
        .store_glue
        .store_secure
        .entry_info(entry_id);
    if entry.is_some() {
        borrow_state_mut(state)?
            .store_glue
            .store_secure
            .cache_derived_region();
    }
    pending_error(state, "GetEntryInfo")?;
    let Some((entry, product)) = entry else {
        state.push(Val::Nil);
        return Ok(1);
    };
    push_entry_info(state, &entry, &product)?;
    Ok(1)
}

fn push_entry_info(
    state: &mut LuaState,
    entry: &super::store_catalog_model::StoreEntryInput,
    product: &super::store_catalog_model::StoreProductInput,
) -> LuaResult<()> {
    let result = create_table(state);
    state.push(result);
    set_i32(state, result, "productID", entry.product_id);
    set_u32(state, result, "groupID", entry.group_id);
    set_i32(state, result, "bannerType", entry.banner_type);
    table_set_static(state, result, "alreadyOwned", Val::Bool(false));
    let shared = create_shared_data(state, &product.shared_data)?;
    table_set_static(state, result, "sharedData", shared);
    Ok(())
}

fn get_product_info(state: &mut LuaState) -> LuaResult<u32> {
    if restricted_call(state)? {
        return Ok(0);
    }
    let product_id = read_signed_id(state, "GetProductInfo", PRODUCT_INFO_USAGE)?;
    let product = borrow_state(state)?
        .store_glue
        .store_secure
        .product_info(product_id);
    if product.is_some() {
        borrow_state_mut(state)?
            .store_glue
            .store_secure
            .cache_derived_region();
    }
    pending_error(state, "GetProductInfo")?;
    let Some(product) = product else {
        state.push(Val::Nil);
        return Ok(1);
    };
    push_product_info(state, &product)?;
    Ok(1)
}

fn push_product_info(
    state: &mut LuaState,
    product: &super::store_catalog_model::StoreProductInput,
) -> LuaResult<()> {
    let result = create_table(state);
    state.push(result);
    set_i32(state, result, "productID", product.product_id);
    let shared = create_shared_data(state, &product.shared_data)?;
    table_set_static(state, result, "sharedData", shared);
    Ok(())
}

fn create_shared_data(state: &mut LuaState, data: &StoreProductSharedData) -> LuaResult<Val> {
    let saved_top = state.top;
    let result = create_table(state);
    state.push(result);
    set_shared_data_numbers(state, result, data);
    set_shared_data_booleans(state, result, data);
    set_shared_data_strings(state, result, data);
    set_shared_data_optional_numbers(state, result, data);
    set_shared_data_color(state, result, data)?;
    let cards = create_cards(state, &data.cards)?;
    table_set_static(state, result, "cards", cards);
    let deliverables = create_deliverables(state, &data.deliverables);
    table_set_static(state, result, "deliverables", deliverables);
    state.top = saved_top;
    Ok(result)
}

fn set_shared_data_numbers(state: &mut LuaState, table: Val, data: &StoreProductSharedData) {
    for (key, value) in [
        ("normalDollars", data.normal_dollars),
        ("normalCents", data.normal_cents),
        ("currentDollars", data.current_dollars),
        ("currentCents", data.current_cents),
        ("flags", data.flags),
        ("eligibility", data.eligibility),
        ("modelSceneID", data.model_scene_id),
        ("cardType", data.card_type),
        ("bannerType", data.banner_type),
        ("itemQuantity", data.item_quantity),
    ] {
        set_u32(state, table, key, value);
    }
}

fn set_shared_data_booleans(state: &mut LuaState, table: Val, data: &StoreProductSharedData) {
    for (key, value) in [
        ("buyableHere", data.buyable_here),
        ("canChangeAccount", data.can_change_account),
        ("canChangeBNetAccount", data.can_change_bnet_account),
    ] {
        table_set_static(state, table, key, Val::Bool(value));
    }
}

fn set_shared_data_strings(state: &mut LuaState, table: Val, data: &StoreProductSharedData) {
    for (key, value) in [
        ("name", data.name.as_deref()),
        ("description", data.description.as_deref()),
        ("tooltip", data.tooltip.as_deref()),
        ("instructions", data.instructions.as_deref()),
        ("disclaimer", data.disclaimer.as_deref()),
        ("overrideBackground", data.override_background.as_deref()),
        ("overrideTexture", data.override_texture.as_deref()),
    ] {
        if let Some(value) = value {
            set_text(state, table, key, value);
        }
    }
}

fn set_shared_data_optional_numbers(
    state: &mut LuaState,
    table: Val,
    data: &StoreProductSharedData,
) {
    for (key, value) in [
        ("texture", data.texture),
        ("productDecorator", data.product_decorator),
        ("boostType", data.boost_type),
        ("itemID", data.item_id),
        ("vasServiceType", data.vas_service_type),
    ] {
        if let Some(value) = value {
            set_u32(state, table, key, value);
        }
    }
}

fn set_shared_data_color(
    state: &mut LuaState,
    table: Val,
    data: &StoreProductSharedData,
) -> LuaResult<()> {
    if let Some(rgb) = data.override_text_color {
        let color = create_color(state, rgb)?;
        table_set_static(state, table, "overrideTextColor", color);
    }
    Ok(())
}

fn create_color(state: &mut LuaState, rgb: [u8; 3]) -> LuaResult<Val> {
    let inverse = 1.0_f32 / RGB_BYTE_MAX;
    let channel = |value: u8| f64::from(f32::from(value) * inverse);
    let mixin = super::global_val(state, "ColorMixin");
    let factory = super::global_val(state, "CreateFromMixins");
    let color = call_function_state(state, factory, &[mixin])?;
    for (key, value) in [
        ("r", channel(rgb[0])),
        ("g", channel(rgb[1])),
        ("b", channel(rgb[2])),
    ] {
        table_set_static(state, color, key, Val::Num(value));
    }
    Ok(color)
}

fn create_cards(
    state: &mut LuaState,
    cards: &[super::store_catalog_model::StoreProductCardInput],
) -> LuaResult<Val> {
    let saved_top = state.top;
    let result = create_table(state);
    state.push(result);
    for (index, card) in cards.iter().enumerate() {
        let row = create_card(state, card)?;
        set_table_array(state, result, index as i64 + 1, row);
    }
    state.top = saved_top;
    Ok(result)
}

fn create_card(
    state: &mut LuaState,
    card: &super::store_catalog_model::StoreProductCardInput,
) -> LuaResult<Val> {
    let saved_top = state.top;
    let result = create_table(state);
    state.push(result);
    set_text(state, result, "title", &card.title);
    set_u32(state, result, "modelSceneID", card.model_scene_id);
    set_u32(
        state,
        result,
        "creatureDisplayInfoID",
        card.creature_display_info_id,
    );
    table_set_static(
        state,
        result,
        "displayTransmogItemsIndividually",
        Val::Bool(card.display_transmog_items_individually),
    );
    let appearances = create_number_array(state, &card.item_modified_appearance_ids);
    table_set_static(state, result, "itemModifiedAppearanceIDs", appearances);
    state.top = saved_top;
    Ok(result)
}

fn create_number_array(state: &mut LuaState, values: &[u32]) -> Val {
    let saved_top = state.top;
    let result = create_table(state);
    state.push(result);
    for (index, value) in values.iter().enumerate() {
        set_table_array(state, result, index as i64 + 1, Val::Num(f64::from(*value)));
    }
    state.top = saved_top;
    result
}

fn create_deliverables(
    state: &mut LuaState,
    rows: &[super::store_catalog_model::StoreProductDeliverableInput],
) -> Val {
    let saved_top = state.top;
    let result = create_table(state);
    state.push(result);
    for (index, row) in rows.iter().enumerate() {
        let item = create_table(state);
        state.push(item);
        set_text(state, item, "name", &row.name);
        table_set_static(state, item, "owned", Val::Bool(row.owned));
        set_table_array(state, result, index as i64 + 1, item);
        state.top -= 1;
    }
    state.top = saved_top;
    result
}

fn get_boolean(state: &mut LuaState, method: &str, value: bool) -> LuaResult<u32> {
    pending_error(state, method)?;
    state.push(Val::Bool(value));
    Ok(1)
}

fn is_available(state: &mut LuaState) -> LuaResult<u32> {
    if restricted_call(state)? {
        return Ok(0);
    }
    let value = borrow_state(state)?.store_glue.store_secure.available;
    get_boolean(state, "IsAvailable", value)
}

fn has_purchase_list(state: &mut LuaState) -> LuaResult<u32> {
    if restricted_call(state)? {
        return Ok(0);
    }
    let value = borrow_state(state)?
        .store_glue
        .store_secure
        .has_purchase_list;
    get_boolean(state, "HasPurchaseList", value)
}

fn has_product_list(state: &mut LuaState) -> LuaResult<u32> {
    if restricted_call(state)? {
        return Ok(0);
    }
    let value = borrow_state(state)?
        .store_glue
        .store_secure
        .has_product_list;
    get_boolean(state, "HasProductList", value)
}

fn has_distribution_list(state: &mut LuaState) -> LuaResult<u32> {
    if restricted_call(state)? {
        return Ok(0);
    }
    let sim = borrow_state(state)?;
    let value = sim.store_glue.connection_present
        && sim.store_glue.store_secure.distribution_list_ready == Some(true);
    drop(sim);
    get_boolean(state, "HasDistributionList", value)
}

fn has_purchase_in_progress(state: &mut LuaState) -> LuaResult<u32> {
    if restricted_call(state)? {
        return Ok(0);
    }
    let value = borrow_state(state)?
        .store_glue
        .store_secure
        .has_purchase_in_progress();
    get_boolean(state, "HasPurchaseInProgress", value)
}

fn is_region_locked(state: &mut LuaState) -> LuaResult<u32> {
    if restricted_call(state)? {
        return Ok(0);
    }
    let value = borrow_state(state)?.store_glue.store_secure.region_locked;
    get_boolean(state, "IsRegionLocked", value)
}

fn get_last_product_list_response_error(state: &mut LuaState) -> LuaResult<u32> {
    if restricted_call(state)? {
        return Ok(0);
    }
    let value = borrow_state(state)?
        .store_glue
        .store_secure
        .last_product_list_response_error;
    pending_error(state, "GetLastProductListResponseError")?;
    state.push(Val::Num(f64::from(value)));
    Ok(1)
}

fn get_currency_id(state: &mut LuaState) -> LuaResult<u32> {
    if restricted_call(state)? {
        return Ok(0);
    }
    let value = borrow_state(state)?.store_glue.store_secure.currency_id;
    pending_error(state, "GetCurrencyID")?;
    state.push(Val::Num(f64::from(value)));
    Ok(1)
}

fn get_currency_info(state: &mut LuaState) -> LuaResult<u32> {
    if restricted_call(state)? {
        return Ok(0);
    }
    let info = borrow_state_mut(state)?
        .store_glue
        .store_secure
        .current_currency_info();
    pending_error(state, "GetCurrencyInfo")?;
    let Some((currency_id, info)) = info else {
        state.push(Val::Nil);
        return Ok(1);
    };
    push_currency_info(state, currency_id, &info);
    Ok(1)
}

fn push_currency_info(state: &mut LuaState, currency_id: i32, info: &StoreCurrencyInfo) {
    let result = create_table(state);
    state.push(result);
    set_i32(state, result, "currencyID", currency_id);
    let shared = create_currency_shared_data(state, info);
    table_set_static(state, result, "sharedData", shared);
}

fn create_currency_shared_data(state: &mut LuaState, info: &StoreCurrencyInfo) -> Val {
    let saved_top = state.top;
    let shared = create_table(state);
    state.push(shared);
    set_u32(state, shared, "regionID", info.region_id);
    set_text(state, shared, "formatShort", &info.format_short);
    set_text(state, shared, "formatLong", &info.format_long);
    set_text(
        state,
        shared,
        "licenseAcceptText",
        &info.license_accept_text,
    );
    set_currency_flags(state, shared, info);
    state.top = saved_top;
    shared
}

fn set_currency_flags(state: &mut LuaState, table: Val, info: &StoreCurrencyInfo) {
    for (key, value) in [
        ("requireLicenseAccept", info.require_license_accept),
        ("browseHasStar", info.browse_has_star),
        ("hideBrowseNotice", info.hide_browse_notice),
        (
            "hideConfirmationBrowseNotice",
            info.hide_confirmation_browse_notice,
        ),
    ] {
        table_set_static(state, table, key, Val::Bool(value));
    }
}

fn is_dynamic_bundle(state: &mut LuaState) -> LuaResult<u32> {
    if restricted_call(state)? {
        return Ok(0);
    }
    let product_id = read_signed_id(state, "IsDynamicBundle", DYNAMIC_BUNDLE_USAGE)?;
    let value = borrow_state(state)?
        .store_glue
        .store_secure
        .is_dynamic_bundle(product_id);
    pending_error(state, "IsDynamicBundle")?;
    state.push(Val::Bool(value));
    Ok(1)
}

fn has_dynamic_price_data(state: &mut LuaState) -> LuaResult<u32> {
    if restricted_call(state)? {
        return Ok(0);
    }
    let product_id = read_signed_id(state, "HasDynamicPriceData", DYNAMIC_PRICE_USAGE)?;
    let value = borrow_state(state)?
        .store_glue
        .store_secure
        .dynamic_prices
        .contains_key(&product_id);
    pending_error(state, "HasDynamicPriceData")?;
    state.push(Val::Bool(value));
    Ok(1)
}

fn request_all_dynamic_price_info(state: &mut LuaState) -> LuaResult<u32> {
    if restricted_call(state)? {
        return Ok(0);
    }
    let product_ids: Vec<i32> = borrow_state(state)?
        .store_glue
        .store_secure
        .catalog
        .products
        .iter()
        .filter(|product| product.dynamic_bundle)
        .map(|product| product.product_id)
        .collect();
    let mut sim = borrow_state_mut(state)?;
    sim.store_glue.submissions.extend(
        product_ids
            .into_iter()
            .map(super::store_glue_model::StoreRequest::DynamicPriceInfo),
    );
    drop(sim);
    pending_error(state, "RequestAllDynamicPriceInfo")?;
    Ok(0)
}

fn get_product_list(state: &mut LuaState) -> LuaResult<u32> {
    if restricted_call(state)? {
        return Ok(0);
    }
    submit_product_list(state)?;
    pending_error(state, "GetProductList")?;
    Ok(0)
}

fn submit_product_list(state: &mut LuaState) -> LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    let connected = sim.store_glue.connection_present;
    let send = {
        let store = &mut sim.store_glue.store_secure;
        if store.product_list_request_count >= PRODUCT_LIST_LIMIT
            && store
                .request_time_ms
                .wrapping_sub(store.product_list_counter_reset_ms)
                >= PRODUCT_LIST_INTERVAL_MS
        {
            store.product_list_request_count = 0;
            store.product_list_counter_reset_ms = store.request_time_ms;
        }
        store.product_list_request_count = store.product_list_request_count.wrapping_add(1);
        store.product_list_request_count <= PRODUCT_LIST_LIMIT
    };
    if connected && send {
        sim.store_glue
            .submissions
            .push(super::store_glue_model::StoreRequest::ProductList);
    }
    Ok(())
}

fn get_purchase_list(state: &mut LuaState) -> LuaResult<u32> {
    if restricted_call(state)? {
        return Ok(0);
    }
    if borrow_state(state)?.store_glue.connection_present {
        borrow_state_mut(state)?
            .store_glue
            .submissions
            .push(super::store_glue_model::StoreRequest::PurchaseList);
    }
    pending_error(state, "GetPurchaseList")?;
    Ok(0)
}

fn read_signed_id(state: &LuaState, method: &str, usage: &str) -> LuaResult<i32> {
    let value = read_numeric_argument(state, 1, method, usage)?;
    let Some(value) =
        value.filter(|value| (f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(value))
    else {
        let reason = format!(
            "outside of expected range {} to {} - {usage}",
            i32::MIN,
            i32::MAX
        );
        return Err(bad_argument(1, method, &reason));
    };
    Ok(value.trunc() as i32)
}

fn read_unsigned_id(state: &LuaState, method: &str, usage: &str) -> LuaResult<u32> {
    let value = read_numeric_argument(state, 1, method, usage)?;
    let Some(value) = value.filter(|value| (0.0..=f64::from(u32::MAX)).contains(value)) else {
        let reason = format!("outside of expected range 0 to {} - {usage}", u32::MAX);
        return Err(bad_argument(1, method, &reason));
    };
    Ok(value.trunc() as u32)
}

fn read_numeric_argument(
    state: &LuaState,
    index: i32,
    method: &str,
    usage: &str,
) -> LuaResult<Option<f64>> {
    let value = read_argument_at(state, index, method, usage, "C_StoreSecure")?;
    Ok(match value {
        Val::Num(number) => Some(number),
        Val::Str(reference) => state
            .gc
            .string_arena
            .get(reference)
            .and_then(|value| parse_numeric_string(value.data())),
        _ => None,
    })
}

fn read_optional_truthy(
    state: &LuaState,
    index: i32,
    method: &str,
    usage: &str,
) -> LuaResult<bool> {
    if stack_val(state, index) == Val::Nil {
        return Ok(false);
    }
    let value = read_argument_at(state, index, method, usage, "C_StoreSecure")?;
    Ok(value != Val::Bool(false))
}

fn set_text(state: &mut LuaState, table: Val, key: &'static str, value: &str) {
    let value = create_string(state, value);
    table_set_static(state, table, key, value);
}

fn set_i32(state: &mut LuaState, table: Val, key: &'static str, value: i32) {
    table_set_static(state, table, key, Val::Num(f64::from(value)));
}

fn set_u32(state: &mut LuaState, table: Val, key: &'static str, value: u32) {
    table_set_static(state, table, key, Val::Num(f64::from(value)));
}
