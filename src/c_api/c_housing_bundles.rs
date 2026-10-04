//! One typed bundle store for getters, featured selection, and view actions.
//! Retains the pre-existing simulator seed and its two legacy output extensions.

use std::collections::HashMap;

use crate::c_api::helpers::{ensure_namespace, set_table_array};
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_table, table_set_static};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const SEEDED_BUNDLE_ID: i32 = 5001;

#[derive(Clone, Debug)]
pub struct HousingBundles {
    pub records: HashMap<i32, HousingBundleInfo>,
    /// Ordered references, not independent copies of bundle state.
    /// INFERRED: removed/missing records are omitted from featured results.
    pub featured_ids: Vec<i32>,
}

impl Default for HousingBundles {
    fn default() -> Self {
        Self {
            records: HashMap::from([(SEEDED_BUNDLE_ID, seeded_bundle())]),
            featured_ids: vec![SEEDED_BUNDLE_ID],
        }
    }
}

#[derive(Clone, Debug)]
pub struct HousingBundleInfo {
    pub product_id: f64,
    pub price: f64,
    pub original_price: Option<f64>,
    pub non_decor_products: Vec<f64>,
    pub decor_entries: Vec<HousingBundleDecorEntryInfo>,
    pub can_preview: bool,
    /// Simulator compatibility extension; not a declared HousingBundleInfo field.
    pub was_viewed: bool,
}

#[derive(Clone, Debug)]
pub struct HousingBundleDecorEntryInfo {
    pub decor_id: f64,
    pub quantity: f64,
}

fn seeded_bundle() -> HousingBundleInfo {
    HousingBundleInfo {
        product_id: f64::from(SEEDED_BUNDLE_ID),
        price: 500.0,
        original_price: None,
        non_decor_products: vec![],
        decor_entries: vec![
            HousingBundleDecorEntryInfo {
                decor_id: 1001.0,
                quantity: 1.0,
            },
            HousingBundleDecorEntryInfo {
                decor_id: 1002.0,
                quantity: 1.0,
            },
        ],
        can_preview: true,
        was_viewed: false,
    }
}

// Unconditional: replaces Lua publishers previously loaded on every profile.
pub(crate) fn register_c_housing_bundles(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_HousingCatalog")?;
    table_set_rust_fn_static(state, namespace, "GetBundleInfo", bundle_info)?;
    table_set_rust_fn_static(state, namespace, "GetFeaturedBundles", featured_bundles)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "HousingMarketActionViewBundle",
        view_bundle,
    )
}

fn read_bundle_id(state: &mut LuaState) -> LuaResult<i32> {
    // INFERRED: public exact i32 selectors; no native coercion/secret parity.
    let Val::Num(number) = Val::from_stack(state, 1)? else {
        return Err(rilua::runtime_error(
            "housing bundle ID must be a public integer",
        ));
    };
    let id = number as i32;
    if f64::from(id) != number {
        return Err(rilua::runtime_error(
            "housing bundle ID is outside the integer input range",
        ));
    }
    Ok(id)
}

fn bundle_info(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_bundle_id(state)?;
    let record = borrow_state(state)?
        .housing_bundles
        .records
        .get(&id)
        .cloned();
    match record {
        Some(record) => {
            push_bundle(state, &record);
        }
        // Missing record returns the declared nullable result.
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn featured_bundles(state: &mut LuaState) -> LuaResult<u32> {
    let records: Vec<_> = {
        let sim = borrow_state(state)?;
        let bundles = &sim.housing_bundles;
        bundles
            .featured_ids
            .iter()
            .filter_map(|id| bundles.records.get(id))
            .cloned()
            .collect()
    };
    let array = push_table(state);
    for (index, record) in records.iter().enumerate() {
        let row = push_bundle(state, record);
        set_table_array(state, array, (index + 1) as i64, row);
        state.top -= 1;
    }
    Ok(1)
}

fn view_bundle(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_bundle_id(state)?;
    let viewed = {
        let mut sim = borrow_state_mut(state)?;
        match sim.housing_bundles.records.get_mut(&id) {
            Some(record) => {
                record.was_viewed = true;
                true
            }
            None => false,
        }
    };
    // Retain the existing simulator boolean return, although the cached action
    // declaration lists no returns. Native restrictions/events are not modeled.
    state.push(Val::Bool(viewed));
    Ok(1)
}

fn push_table(state: &mut LuaState) -> Val {
    let table = create_table(state);
    state.push(table);
    table
}

fn attach_child(state: &mut LuaState, parent: Val, field: &'static str, child: Val) {
    table_set_static(state, parent, field, child);
    state.top -= 1;
}

// INFERRED: fresh deep snapshots, not native aliasing proof.
// Root each table on the VM stack before allocating descendants.
fn push_bundle(state: &mut LuaState, record: &HousingBundleInfo) -> Val {
    let row = push_table(state);
    table_set_static(state, row, "productID", Val::Num(record.product_id));
    table_set_static(state, row, "price", Val::Num(record.price));
    if let Some(price) = record.original_price {
        table_set_static(state, row, "originalPrice", Val::Num(price));
    }
    // Cached documentation: bundles containing non-decor items cannot preview.
    let can_preview = record.can_preview && record.non_decor_products.is_empty();
    table_set_static(state, row, "canPreview", Val::Bool(can_preview));
    table_set_static(state, row, "wasViewed", Val::Bool(record.was_viewed));
    let products = push_numbers(state, &record.non_decor_products);
    attach_child(state, row, "nonDecorProducts", products);
    let entries = push_decor_entries(state, &record.decor_entries);
    attach_child(state, row, "decorEntries", entries);
    let entry_ids = push_decor_ids(state, &record.decor_entries);
    attach_child(state, row, "entryIDs", entry_ids);
    row
}

fn push_numbers(state: &mut LuaState, numbers: &[f64]) -> Val {
    let array = push_table(state);
    for (index, number) in numbers.iter().enumerate() {
        set_table_array(state, array, (index + 1) as i64, Val::Num(*number));
    }
    array
}

fn push_decor_ids(state: &mut LuaState, entries: &[HousingBundleDecorEntryInfo]) -> Val {
    let array = push_table(state);
    for (index, entry) in entries.iter().enumerate() {
        set_table_array(state, array, (index + 1) as i64, Val::Num(entry.decor_id));
    }
    array
}

fn push_decor_entries(state: &mut LuaState, entries: &[HousingBundleDecorEntryInfo]) -> Val {
    let array = push_table(state);
    for (index, entry) in entries.iter().enumerate() {
        let row = push_table(state);
        table_set_static(state, row, "decorID", Val::Num(entry.decor_id));
        table_set_static(state, row, "quantity", Val::Num(entry.quantity));
        set_table_array(state, array, (index + 1) as i64, row);
        state.top -= 1;
    }
    array
}
