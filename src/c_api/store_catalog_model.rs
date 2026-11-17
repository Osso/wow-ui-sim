//! Host-supplied inputs for the Retail C_StoreSecure catalog and list surface.
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::WowLuaEnv;
use rilua::Val;
use std::collections::{HashMap, HashSet};

const ROOT_PRODUCT_GROUP_ID: u32 = 0;
const EMPTY_STORE_CURRENCY_ID: i32 = 0;
const PRODUCT_LIST_SUCCESS_CODE: u32 = 0;
const REGION_LOCK_RESPONSE_CODE: u32 = 3;
const PURCHASE_STATUS_EXCLUDED_FROM_PROGRESS: i32 = 6;
const EXCLUDED_GROUP_PRODUCT_IDS: [i32; 2] = [239, 477];

#[derive(Debug, Default)]
pub struct StoreSecureState {
    pub catalog: StoreCatalogInput,
    pub vas: super::store_vas_model::StoreVasState,
    pub available: bool,
    pub has_product_list: bool,
    pub has_purchase_list: bool,
    /// Connection-owned readiness; `None` leaves the distribution writer unknown.
    pub distribution_list_ready: Option<bool>,
    pub currency_id: i32,
    pub currencies: HashMap<i32, StoreCurrencyInfo>,
    pub derived_region: Option<u32>,
    pub purchase_list: Vec<StorePurchaseInput>,
    /// Explicit clock and offsets for the purchase-in-progress comparison.
    pub current_time: f64,
    pub shared_time_adjustment: f64,
    pub last_product_list_response_error: u32,
    pub region_locked: bool,
    /// Opaque dynamic-price response rows, keyed by signed product ID.
    pub dynamic_prices: HashMap<i32, Vec<u8>>,
    /// Host-supplied unsigned clock sample for product-list request limiting.
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

/// Host-supplied outcomes for the native group display and metadata reads.
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

/// Host-supplied predicate results and detail data; these are not raw native flags.
#[derive(Clone, Debug)]
pub struct StoreProductInput {
    pub product_id: i32,
    pub supported: bool,
    pub session_eligible: bool,
    pub display_eligible: bool,
    pub dynamic_bundle: bool,
    /// Host-supplied native product type for the VAS service-type mapping.
    pub product_type: Option<u32>,
    pub shared_data: StoreProductSharedData,
}

/// `hidden` is the host-supplied result for this entry in its group.
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

/// Host-supplied purchase-row fields used by the verified progress predicate.
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

impl StoreSecureState {
    pub fn group_info(&self, group_id: i32) -> Option<StoreProductGroupInput> {
        self.catalog
            .groups
            .iter()
            .find(|group| group.group_id as i32 == group_id)
            .cloned()
    }

    pub fn product_info(&self, product_id: i32) -> Option<StoreProductInput> {
        self.catalog
            .products
            .iter()
            .find(|product| product.product_id == product_id && product.supported)
            .cloned()
    }

    pub fn entry_info(&self, entry_id: i32) -> Option<(StoreEntryInput, StoreProductInput)> {
        let entry = self
            .catalog
            .entries
            .iter()
            .find(|entry| entry.entry_id == entry_id)?;
        let product = self.product_info(entry.product_id)?;
        Some((entry.clone(), product))
    }

    pub fn product_group_rows(&self) -> Vec<(u32, u32)> {
        let eligible_products = self.display_eligible_products();
        let eligible_groups: HashSet<u32> = self
            .catalog
            .entries
            .iter()
            .filter(|entry| eligible_products.contains(&entry.product_id))
            .map(|entry| entry.group_id)
            .collect();
        let eligible_parents: HashSet<u32> = self
            .catalog
            .groups
            .iter()
            .filter(|group| {
                group.parent_group_id != ROOT_PRODUCT_GROUP_ID
                    && eligible_groups.contains(&group.group_id)
            })
            .map(|group| group.parent_group_id)
            .collect();
        self.catalog
            .groups
            .iter()
            .filter(|group| {
                group.displayable
                    && (eligible_groups.contains(&group.group_id)
                        || eligible_parents.contains(&group.group_id))
            })
            .map(|group| (group.group_id, group.parent_group_id))
            .collect()
    }

    pub fn product_entry_ids(&self, group_id: u32, include_hidden: bool) -> Vec<i32> {
        let products = self.list_supported_products();
        self.catalog
            .entries
            .iter()
            .filter(|entry| entry.group_id == group_id && (include_hidden || !entry.hidden))
            .filter(|entry| !EXCLUDED_GROUP_PRODUCT_IDS.contains(&entry.product_id))
            .filter(|entry| products.contains(&entry.product_id))
            .map(|entry| entry.entry_id)
            .collect()
    }

    pub fn current_currency_info(&mut self) -> Option<(i32, StoreCurrencyInfo)> {
        if self.currency_id == EMPTY_STORE_CURRENCY_ID {
            return None;
        }
        let currency = self.currencies.get(&self.currency_id)?.clone();
        self.cache_derived_region();
        Some((self.currency_id, currency))
    }

    pub fn cache_derived_region(&mut self) {
        if self.derived_region.is_none() {
            self.derived_region = self
                .currencies
                .get(&self.currency_id)
                .map(|currency| currency.region_id);
        }
    }

    pub fn has_purchase_in_progress(&self) -> bool {
        self.purchase_list.iter().any(|purchase| {
            !purchase.blocked
                && purchase.status != PURCHASE_STATUS_EXCLUDED_FROM_PROGRESS
                && purchase.expires_at + purchase.client_time_adjustment
                    > self.current_time + self.shared_time_adjustment
        })
    }

    pub fn is_dynamic_bundle(&self, product_id: i32) -> bool {
        self.catalog
            .products
            .iter()
            .find(|product| product.product_id == product_id)
            .is_some_and(|product| product.dynamic_bundle)
    }

    fn display_eligible_products(&self) -> HashSet<i32> {
        self.catalog
            .products
            .iter()
            .filter(|product| {
                product.supported && product.session_eligible && product.display_eligible
            })
            .map(|product| product.product_id)
            .collect()
    }

    fn list_supported_products(&self) -> HashSet<i32> {
        self.catalog
            .products
            .iter()
            .filter(|product| product.supported && product.session_eligible)
            .map(|product| product.product_id)
            .collect()
    }
}

pub fn receive_disconnect(env: &WowLuaEnv) {
    if ACTIVE != ClientProfile::Retail {
        return;
    }
    let mut sim = env.state().borrow_mut();
    let store = &mut sim.store_glue.store_secure;
    store.available = false;
    store.has_product_list = false;
    store.has_purchase_list = false;
    store.region_locked = false;
    store.last_product_list_response_error = 0;
    store.currency_id = 0;
    store.dynamic_prices.clear();
    sim.store_glue.connection_present = false;
}

pub fn receive_availability(env: &WowLuaEnv, available: bool) -> crate::Result<()> {
    if ACTIVE != ClientProfile::Retail {
        return Ok(());
    }
    let notify = {
        let mut sim = env.state().borrow_mut();
        sim.store_glue.store_secure.available = available;
        sim.store_glue.callbacks.store_status_changed
    };
    if notify {
        env.fire_event_with_args("STORE_STATUS_CHANGED", &[])?;
    }
    Ok(())
}

pub fn receive_product_list(
    env: &WowLuaEnv,
    response: StoreProductListResponse,
) -> crate::Result<()> {
    if ACTIVE != ClientProfile::Retail {
        return Ok(());
    }
    let notify = {
        let mut sim = env.state().borrow_mut();
        apply_product_list_response(&mut sim.store_glue.store_secure, response);
        sim.store_glue.callbacks.store_products_updated
    };
    if notify {
        env.fire_event_with_args("STORE_PRODUCTS_UPDATED", &[])?;
    }
    Ok(())
}

fn apply_product_list_response(
    store: &mut StoreSecureState,
    mut response: StoreProductListResponse,
) {
    store.last_product_list_response_error = response.result_code;
    store.region_locked = response.result_code == REGION_LOCK_RESPONSE_CODE;
    if response.result_code != PRODUCT_LIST_SUCCESS_CODE {
        store.catalog.groups.clear();
        store.catalog.products.clear();
        store.has_product_list = false;
        return;
    }
    response.catalog.groups.sort_by_key(|group| group.order);
    response
        .catalog
        .entries
        .sort_by_key(|entry| (entry.group_id, entry.order));
    store.catalog = response.catalog;
    store.currency_id = response.currency_id;
    store.derived_region = None;
    store.last_product_list_response_error = 0;
    store.region_locked = false;
    store.has_product_list = true;
}

pub fn receive_purchase_list(
    env: &WowLuaEnv,
    response: StorePurchaseListResponse,
) -> crate::Result<()> {
    if ACTIVE != ClientProfile::Retail {
        return Ok(());
    }
    {
        let mut sim = env.state().borrow_mut();
        apply_purchase_list_response(&mut sim.store_glue.store_secure, response);
        queue_purchase_list_notification(&mut sim.store_glue);
    }
    Ok(())
}

pub fn flush_purchase_list_notification(env: &WowLuaEnv) -> crate::Result<()> {
    if ACTIVE != ClientProfile::Retail {
        return Ok(());
    }
    let pending = {
        let mut sim = env.state().borrow_mut();
        std::mem::take(
            &mut sim
                .store_glue
                .store_secure
                .purchase_list_notification_pending,
        )
    };
    if pending {
        env.fire_event_with_args("STORE_PURCHASE_LIST_UPDATED", &[])?;
    }
    Ok(())
}

fn apply_purchase_list_response(store: &mut StoreSecureState, response: StorePurchaseListResponse) {
    store.has_purchase_list = response.result_code == PRODUCT_LIST_SUCCESS_CODE;
    store.purchase_list = if store.has_purchase_list {
        response.rows
    } else {
        Vec::new()
    };
}

fn queue_purchase_list_notification(store: &mut super::store_glue_model::StoreGlueState) {
    if !store.callbacks.store_purchase_list_updated {
        return;
    }
    if store.store_secure.purchase_list_notification_pending {
        return;
    }
    store.store_secure.purchase_list_notification_pending = true;
}

pub fn receive_dynamic_price(env: &WowLuaEnv, product_id: i32, data: Vec<u8>) -> crate::Result<()> {
    if ACTIVE != ClientProfile::Retail {
        return Ok(());
    }
    let notify = {
        let mut sim = env.state().borrow_mut();
        sim.store_glue
            .store_secure
            .dynamic_prices
            .insert(product_id, data);
        sim.store_glue.callbacks.dynamic_bundle_price_updated
    };
    if notify {
        env.fire_event_with_args(
            "DYNAMIC_BUNDLE_PRICE_UPDATED",
            &[Val::Num(f64::from(product_id))],
        )?;
    }
    Ok(())
}
