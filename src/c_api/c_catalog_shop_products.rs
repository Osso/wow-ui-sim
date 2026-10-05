//! Typed catalog product DTO inputs and fresh Lua snapshots.
//! Cached number fields remain f64; texture kits/atlases are strings.
//! Restrictions and purchase behavior are outside this DTO surface.

use std::collections::HashMap;

use crate::c_api::helpers::{ensure_namespace, set_table_array};
use crate::lua_api::methods::{borrow_state, create_string, create_table, table_set_static};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// Explicit per-environment inputs; no built-in storefront products.
#[derive(Clone, Debug, Default)]
pub struct CatalogShopProducts {
    pub products: HashMap<i32, CatalogShopProductInfo>,
    pub displays: HashMap<i32, CatalogShopProductDisplayInfo>,
    /// INFERRED host "new" labels in display order, independent of ownership.
    pub new_product_ids: Vec<i32>,
}

#[derive(Clone, Debug, Default)]
pub struct CatalogShopProductInfo {
    pub catalog_shop_product_id: f64,
    pub name: String,
    pub r#type: Option<String>,
    pub description: String,
    pub icon_texture: String,
    pub is_fully_owned: bool,
    pub is_purchase_pending: bool,
    pub refundable: bool,
    pub price: String,
    pub original_price: String,
    pub discount_percentage: f64,
    pub item_id: f64,
    pub mount_id: f64,
    pub mount_type_name: String,
    pub species_id: f64,
    pub transmog_set_id: f64,
    pub item_modified_appearance_id: f64,
    pub sub_items: Vec<CatalogShopSubItemInfo>,
    pub sub_items_loaded: bool,
    pub background_texture: String,
    pub foreground_texture: Option<String>,
    pub small_card_bg_texture: Option<String>,
    pub small_card_fg_texture: Option<String>,
    pub wide_card_bg_texture: Option<String>,
    pub wide_card_fg_texture: Option<String>,
    pub preview_icon_texture: Option<String>,
    pub optional_wide_card_background_texture: Option<String>,
    pub is_bundle: bool,
    pub bundle_children_size: f64,
    pub license_term_type: f64,
    pub license_term_duration: f64,
    pub virtual_currencies: Vec<CatalogShopVirtualCurrency>,
    pub is_hidden: bool,
    pub is_mystery: bool,
    pub has_pending_orders: bool,
    pub num_bundle_detail_cards: f64,
    pub is_dynamically_discounted: bool,
    pub should_show_original_price: bool,
    pub wide_card_bg_override_product_url: Option<String>,
    pub preview_bg_override_product_url: Option<String>,
    pub preview_small_bg_override_product_url: Option<String>,
    pub decor_quantity: Option<DecorQuantity>,
    pub is_vc_product: bool,
    pub contains_housing_item: bool,
    pub deferred_game_time_days: f64,
}

#[derive(Clone, Debug, Default)]
pub struct CatalogShopProductDisplayInfo {
    pub default_preview_model_scene_id: f64,
    pub default_card_model_scene_id: f64,
    pub default_wide_card_model_scene_id: f64,
    pub item_id: f64,
    pub override_preview_model_scene_id: Option<f64>,
    pub override_card_model_scene_id: Option<f64>,
    pub override_wide_card_model_scene_id: Option<f64>,
    pub creature_display_info_ids: Vec<f64>,
    pub spell_visual_ids: Vec<f64>,
    pub main_hand_item_modified_appearance_id: Option<f64>,
    pub off_hand_item_modified_appearance_id: Option<f64>,
    pub item_modified_appearance_ids: Vec<f64>,
    pub icon_file_data_id: Option<f64>,
    pub icon_texture_kit: Option<String>,
    pub product_type: Option<String>,
    pub item_description: Option<String>,
    pub has_unknown_license: bool,
    pub product_pmt_url: Option<String>,
    pub additional_product_pmt_urls: Vec<String>,
    pub other_product_image_atlas_name: Option<String>,
    pub other_product_game_title_base_tag: Option<String>,
    pub other_product_game_type: Option<String>,
    pub custom_looping_sound_start: Option<f64>,
    pub custom_looping_sound_middle: Option<f64>,
    pub custom_looping_sound_end: Option<f64>,
    pub special_actor_id_1: Option<String>,
    pub special_actor_id_2: Option<String>,
    pub special_actor_id_3: Option<String>,
    pub special_actor_id_4: Option<String>,
    pub special_actor_id_5: Option<String>,
    pub game_flavor_id: Option<f64>,
    pub decor_file_data_id: Option<f64>,
    pub quantity: Option<f64>,
    pub house_texture_atlas: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct CatalogShopSubItemInfo {
    pub name: String,
    pub item_id: f64,
    pub item_appearance_id: f64,
    pub inv_type: String,
    pub quality: i32,
}

#[derive(Clone, Debug, Default)]
pub struct CatalogShopVirtualCurrency {
    pub amount: f64,
    pub currency_code: String,
}

#[derive(Clone, Debug, Default)]
pub struct DecorQuantity {
    pub placed_quantity: f64,
    pub stored_quantity: f64,
}

pub(crate) fn register_c_catalog_shop_products(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_CatalogShop")?;
    #[cfg(feature = "retail-12-0-0")]
    table_set_rust_fn_static(state, namespace, "GetNewProducts", |s| {
        let ids = borrow_state(s)?
            .catalog_shop_products
            .new_product_ids
            .clone();
        let result = create_table(s);
        s.push(result);
        for (index, id) in ids.iter().enumerate() {
            set_table_array(s, result, index as i64 + 1, Val::Num(f64::from(*id)));
        }
        Ok(1)
    })?;
    table_set_rust_fn_static(state, namespace, "GetProductInfo", product_info)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetCatalogShopProductDisplayInfo",
        product_display_info,
    )
}

fn read_product_id(state: &mut LuaState) -> LuaResult<i32> {
    // INFERRED: exact public i32 selectors; native argument coercion and
    // AllowedWhenUntainted secret acceptance are not modeled in this slice.
    let Val::Num(number) = Val::from_stack(state, 1)? else {
        return Err(rilua::runtime_error(
            "catalog shop product ID must be a public integer; secret access is not modeled",
        ));
    };
    let id = number as i32;
    if f64::from(id) != number {
        return Err(rilua::runtime_error(
            "catalog shop product ID is outside the integer input range",
        ));
    }
    Ok(id)
}

fn product_info(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_product_id(state)?;
    let record = borrow_state(state)?
        .catalog_shop_products
        .products
        .get(&id)
        .cloned();
    match record {
        Some(record) => {
            push_product(state, &record);
        }
        // INFERRED: absent stored product returns the declared nullable result.
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn product_display_info(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_product_id(state)?;
    let record = borrow_state(state)?
        .catalog_shop_products
        .displays
        .get(&id)
        .cloned();
    match record {
        Some(record) => {
            push_display(state, &record);
        }
        // INFERRED: requested nil-on-miss policy differs from the cached
        // GetCatalogShopProductDisplayInfo nonnullable return declaration.
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn push_table(state: &mut LuaState) -> Val {
    let table = create_table(state);
    state.push(table);
    table
}

fn set_text(state: &mut LuaState, table: Val, field: &'static str, text: &str) {
    let value = create_string(state, text);
    table_set_static(state, table, field, value);
}

fn attach_child(state: &mut LuaState, parent: Val, field: &'static str, child: Val) {
    table_set_static(state, parent, field, child);
    state.top -= 1;
}

// INFERRED: snapshots and all nested arrays are fresh, not native aliasing proof.
// Every table is rooted while descendants are allocated, mirroring housing DTOs.
fn push_sequence<T>(
    state: &mut LuaState,
    records: &[T],
    push_row: fn(&mut LuaState, &T) -> Val,
) -> Val {
    let sequence = push_table(state);
    for (index, record) in records.iter().enumerate() {
        let row = push_row(state, record);
        set_table_array(state, sequence, (index + 1) as i64, row);
        state.top -= 1;
    }
    sequence
}

fn push_number(state: &mut LuaState, number: &f64) -> Val {
    let value = Val::Num(*number);
    state.push(value);
    value
}

fn push_string(state: &mut LuaState, text: &String) -> Val {
    let value = create_string(state, text);
    state.push(value);
    value
}

fn push_product(state: &mut LuaState, record: &CatalogShopProductInfo) -> Val {
    let row = push_table(state);
    push_product_numbers(state, row, record);
    push_product_strings(state, row, record);
    push_product_booleans(state, row, record);
    let child = push_sequence(state, &record.sub_items, push_sub_item);
    attach_child(state, row, "subItems", child);
    let child = push_sequence(state, &record.virtual_currencies, push_currency);
    attach_child(state, row, "virtualCurrencies", child);
    if let Some(quantity) = &record.decor_quantity {
        let child = push_decor_quantity(state, quantity);
        attach_child(state, row, "decorQuantity", child);
    }
    row
}

fn push_product_numbers(state: &mut LuaState, row: Val, record: &CatalogShopProductInfo) {
    for (field, value) in [
        ("catalogShopProductID", record.catalog_shop_product_id),
        ("discountPercentage", record.discount_percentage),
        ("itemID", record.item_id),
        ("mountID", record.mount_id),
        ("speciesID", record.species_id),
        ("transmogSetID", record.transmog_set_id),
        (
            "itemModifiedAppearanceID",
            record.item_modified_appearance_id,
        ),
        ("bundleChildrenSize", record.bundle_children_size),
        ("licenseTermType", record.license_term_type),
        ("licenseTermDuration", record.license_term_duration),
        ("numBundleDetailCards", record.num_bundle_detail_cards),
        ("deferredGameTimeDays", record.deferred_game_time_days),
    ] {
        table_set_static(state, row, field, Val::Num(value));
    }
}

fn push_product_strings(state: &mut LuaState, row: Val, record: &CatalogShopProductInfo) {
    for (field, value) in [
        ("name", &record.name),
        ("description", &record.description),
        ("iconTexture", &record.icon_texture),
        ("price", &record.price),
        ("originalPrice", &record.original_price),
        ("mountTypeName", &record.mount_type_name),
        ("backgroundTexture", &record.background_texture),
    ] {
        set_text(state, row, field, value);
    }
    for (field, value) in [
        ("type", record.r#type.as_deref()),
        ("foregroundTexture", record.foreground_texture.as_deref()),
        (
            "smallCardBGTexture",
            record.small_card_bg_texture.as_deref(),
        ),
        (
            "smallCardFGTexture",
            record.small_card_fg_texture.as_deref(),
        ),
        ("wideCardBGTexture", record.wide_card_bg_texture.as_deref()),
        ("wideCardFGTexture", record.wide_card_fg_texture.as_deref()),
        ("previewIconTexture", record.preview_icon_texture.as_deref()),
        (
            "optionalWideCardBackgroundTexture",
            record.optional_wide_card_background_texture.as_deref(),
        ),
        (
            "wideCardBGOverrideProductURL",
            record.wide_card_bg_override_product_url.as_deref(),
        ),
        (
            "previewBGOverrideProductURL",
            record.preview_bg_override_product_url.as_deref(),
        ),
        (
            "previewSmallBGOverrideProductURL",
            record.preview_small_bg_override_product_url.as_deref(),
        ),
    ] {
        if let Some(value) = value {
            set_text(state, row, field, value);
        }
    }
}

fn push_product_booleans(state: &mut LuaState, row: Val, record: &CatalogShopProductInfo) {
    for (field, value) in [
        ("isFullyOwned", record.is_fully_owned),
        ("isPurchasePending", record.is_purchase_pending),
        ("refundable", record.refundable),
        ("subItemsLoaded", record.sub_items_loaded),
        ("isBundle", record.is_bundle),
        ("isHidden", record.is_hidden),
        ("isMystery", record.is_mystery),
        ("hasPendingOrders", record.has_pending_orders),
        ("isDynamicallyDiscounted", record.is_dynamically_discounted),
        ("shouldShowOriginalPrice", record.should_show_original_price),
        ("isVCProduct", record.is_vc_product),
        ("containsHousingItem", record.contains_housing_item),
    ] {
        table_set_static(state, row, field, Val::Bool(value));
    }
}

fn push_display(state: &mut LuaState, record: &CatalogShopProductDisplayInfo) -> Val {
    let row = push_table(state);
    push_display_numbers(state, row, record);
    push_display_strings(state, row, record);
    table_set_static(
        state,
        row,
        "hasUnknownLicense",
        Val::Bool(record.has_unknown_license),
    );
    let child = push_sequence(state, &record.creature_display_info_ids, push_number);
    attach_child(state, row, "creatureDisplayInfoIDs", child);
    let child = push_sequence(state, &record.spell_visual_ids, push_number);
    attach_child(state, row, "spellVisualIDs", child);
    let child = push_sequence(state, &record.item_modified_appearance_ids, push_number);
    attach_child(state, row, "itemModifiedAppearanceIDs", child);
    let child = push_sequence(state, &record.additional_product_pmt_urls, push_string);
    attach_child(state, row, "additionalProductPMTURLs", child);
    row
}

fn push_display_numbers(state: &mut LuaState, row: Val, record: &CatalogShopProductDisplayInfo) {
    for (field, value) in [
        (
            "defaultPreviewModelSceneID",
            record.default_preview_model_scene_id,
        ),
        (
            "defaultCardModelSceneID",
            record.default_card_model_scene_id,
        ),
        (
            "defaultWideCardModelSceneID",
            record.default_wide_card_model_scene_id,
        ),
        ("itemID", record.item_id),
    ] {
        table_set_static(state, row, field, Val::Num(value));
    }
    for (field, value) in [
        (
            "overridePreviewModelSceneID",
            record.override_preview_model_scene_id,
        ),
        (
            "overrideCardModelSceneID",
            record.override_card_model_scene_id,
        ),
        (
            "overrideWideCardModelSceneID",
            record.override_wide_card_model_scene_id,
        ),
        (
            "mainHandItemModifiedAppearanceID",
            record.main_hand_item_modified_appearance_id,
        ),
        (
            "offHandItemModifiedAppearanceID",
            record.off_hand_item_modified_appearance_id,
        ),
        ("iconFileDataID", record.icon_file_data_id),
        ("customLoopingSoundStart", record.custom_looping_sound_start),
        (
            "customLoopingSoundMiddle",
            record.custom_looping_sound_middle,
        ),
        ("customLoopingSoundEnd", record.custom_looping_sound_end),
        ("gameFlavorID", record.game_flavor_id),
        ("decorFileDataID", record.decor_file_data_id),
        ("quantity", record.quantity),
    ] {
        if let Some(value) = value {
            table_set_static(state, row, field, Val::Num(value));
        }
    }
}

fn push_display_strings(state: &mut LuaState, row: Val, record: &CatalogShopProductDisplayInfo) {
    for (field, value) in [
        ("iconTextureKit", record.icon_texture_kit.as_deref()),
        ("productType", record.product_type.as_deref()),
        ("itemDescription", record.item_description.as_deref()),
        ("productPMTURL", record.product_pmt_url.as_deref()),
        (
            "otherProductImageAtlasName",
            record.other_product_image_atlas_name.as_deref(),
        ),
        (
            "otherProductGameTitleBaseTag",
            record.other_product_game_title_base_tag.as_deref(),
        ),
        (
            "otherProductGameType",
            record.other_product_game_type.as_deref(),
        ),
        ("specialActorID_1", record.special_actor_id_1.as_deref()),
        ("specialActorID_2", record.special_actor_id_2.as_deref()),
        ("specialActorID_3", record.special_actor_id_3.as_deref()),
        ("specialActorID_4", record.special_actor_id_4.as_deref()),
        ("specialActorID_5", record.special_actor_id_5.as_deref()),
        ("houseTextureAtlas", record.house_texture_atlas.as_deref()),
    ] {
        if let Some(value) = value {
            set_text(state, row, field, value);
        }
    }
}

fn push_sub_item(state: &mut LuaState, record: &CatalogShopSubItemInfo) -> Val {
    let row = push_table(state);
    for (field, value) in [
        ("itemID", record.item_id),
        ("itemAppearanceID", record.item_appearance_id),
        ("quality", f64::from(record.quality)),
    ] {
        table_set_static(state, row, field, Val::Num(value));
    }
    for (field, value) in [("name", &record.name), ("invType", &record.inv_type)] {
        set_text(state, row, field, value);
    }
    row
}

fn push_currency(state: &mut LuaState, record: &CatalogShopVirtualCurrency) -> Val {
    let row = push_table(state);
    table_set_static(state, row, "amount", Val::Num(record.amount));
    set_text(state, row, "currencyCode", &record.currency_code);
    row
}

fn push_decor_quantity(state: &mut LuaState, record: &DecorQuantity) -> Val {
    let row = push_table(state);
    for (field, value) in [
        ("placedQuantity", record.placed_quantity),
        ("storedQuantity", record.stored_quantity),
    ] {
        table_set_static(state, row, field, Val::Num(value));
    }
    row
}
