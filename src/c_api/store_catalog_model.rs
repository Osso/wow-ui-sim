//! Host-supplied inputs for the Retail C_StoreSecure catalog and list surface.
use crate::lua_api::WowLuaEnv;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct StoreSecureState {
    pub catalog: StoreCatalogInput,
    pub available: bool,
    pub has_product_list: bool,
    pub has_purchase_list: bool,
    pub distribution_list_ready: Option<bool>,
    pub currency_id: i32,
    pub currencies: HashMap<i32, StoreCurrencyInfo>,
    pub purchase_list: Vec<StorePurchaseInput>,
    pub last_product_list_response_error: u32,
    pub region_locked: bool,
    pub dynamic_prices: HashMap<i32, Vec<u8>>,
    pub current_time: f64,
    pub shared_time_adjustment: f64,
    pub request_time_ms: u32,
    pub product_list_request_count: u32,
    pub product_list_counter_reset_ms: u32,
    pub purchase_list_notification_pending: bool,
}

#[derive(Clone, Debug, Default)]
pub struct StoreCatalogInput {
    pub groups: Vec<StoreProductGroupInput>,
    pub products: Vec<StoreProductInput>,
    pub entries: Vec<StoreEntryInput>,
}

#[derive(Clone, Debug)]
pub struct StoreProductGroupInput {
    pub group_id: u32,
    pub parent_group_id: u32,
    pub order: i32,
    pub displayable: bool,
    pub name: String,
    pub texture: Option<u32>,
    pub display_type: i32,
    pub flags: u32,
    pub disabled_tooltip: Option<String>,
}

#[derive(Clone, Debug)]
pub struct StoreProductInput {
    pub product_id: i32,
    pub supported: bool,
    pub session_eligible: bool,
    pub display_eligible: bool,
    pub dynamic_bundle: bool,
    pub shared_data: StoreProductSharedData,
}

#[derive(Clone, Debug)]
pub struct StoreEntryInput {
    pub entry_id: i32,
    pub product_id: i32,
    pub group_id: u32,
    pub order: i32,
    pub banner_type: i32,
    pub hidden: bool,
}

/// The default values describe an empty host DTO, not an inserted product row.
#[derive(Clone, Debug, Default)]
pub struct StoreProductSharedData {
    pub normal_dollars: u32,
    pub normal_cents: u32,
    pub current_dollars: u32,
    pub current_cents: u32,
    pub flags: u32,
    pub eligibility: u32,
    pub model_scene_id: u32,
    pub card_type: u32,
    pub banner_type: u32,
    pub item_quantity: u32,
    pub buyable_here: bool,
    pub can_change_account: bool,
    pub can_change_bnet_account: bool,
    pub name: Option<String>,
    pub description: Option<String>,
    pub tooltip: Option<String>,
    pub instructions: Option<String>,
    pub disclaimer: Option<String>,
    pub override_background: Option<String>,
    pub override_texture: Option<String>,
    pub texture: Option<u32>,
    pub product_decorator: Option<u32>,
    pub boost_type: Option<u32>,
    pub item_id: Option<u32>,
    pub vas_service_type: Option<u32>,
    pub override_text_color: Option<[u8; 3]>,
    pub cards: Vec<StoreProductCardInput>,
    pub deliverables: Vec<StoreProductDeliverableInput>,
}

#[derive(Clone, Debug)]
pub struct StoreProductCardInput {
    pub title: String,
    pub model_scene_id: u32,
    pub creature_display_info_id: u32,
    pub item_modified_appearance_ids: Vec<u32>,
    pub display_transmog_items_individually: bool,
}

#[derive(Clone, Debug)]
pub struct StoreProductDeliverableInput {
    pub name: String,
    pub owned: bool,
}

#[derive(Clone, Debug)]
pub struct StoreCurrencyInfo {
    pub region_id: u32,
    pub format_short: String,
    pub format_long: String,
    pub license_accept_text: String,
    pub require_license_accept: bool,
    pub browse_has_star: bool,
    pub hide_browse_notice: bool,
    pub hide_confirmation_browse_notice: bool,
}

#[derive(Clone, Debug)]
pub struct StorePurchaseInput {
    pub blocked: bool,
    pub status: i32,
    pub expires_at: f64,
    pub client_time_adjustment: f64,
}

#[derive(Debug)]
pub struct StoreProductListResponse {
    pub result_code: u32,
    pub currency_id: i32,
    pub catalog: StoreCatalogInput,
}

#[derive(Debug)]
pub struct StorePurchaseListResponse {
    pub result_code: u32,
    pub rows: Vec<StorePurchaseInput>,
}

pub fn receive_availability(_env: &WowLuaEnv, _available: bool) -> crate::Result<()> {
    Ok(())
}

pub fn receive_product_list(
    _env: &WowLuaEnv,
    _response: StoreProductListResponse,
) -> crate::Result<()> {
    Ok(())
}

pub fn receive_purchase_list(
    _env: &WowLuaEnv,
    _response: StorePurchaseListResponse,
) -> crate::Result<()> {
    Ok(())
}

pub fn receive_dynamic_price(
    _env: &WowLuaEnv,
    _product_id: i32,
    _data: Vec<u8>,
) -> crate::Result<()> {
    Ok(())
}
