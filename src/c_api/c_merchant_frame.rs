//! `C_MerchantFrame` 12.0.7 probe surface.
//!
//! Retail 12.0.7 currency IDs read the explicit ordered host snapshot.

#[cfg(feature = "client-wowforever")]
pub mod junk;

use crate::c_api::helpers::ensure_namespace;
#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
use crate::lua_api::methods::create_table;
use crate::lua_api::methods::{
    borrow_state, create_string, create_table_with_capacity, table_set_static,
};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::LuaResult;
#[cfg(feature = "client-wowforever")]
use rilua::Val;
use rilua::vm::state::LuaState;

pub(crate) fn register_c_merchant_frame_surface(state: &mut LuaState) -> LuaResult<()> {
    let merchant = ensure_namespace(state, "C_MerchantFrame")?;
    // Forever 70707 MerchantFrameDocumentation.lua:27-39 shares this model.
    if cfg!(feature = "retail-12-0-0")
        || crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::WowForever
    {
        table_set_rust_fn_static(state, merchant, "GetItemInfo", get_item_info)?;
        table_set_rust_fn_static(
            state,
            merchant,
            "IsMerchantItemRefundable",
            is_merchant_item_refundable,
        )?;
    }
    register_patch_12_0_7_merchant_frame_surface(state, merchant)?;
    // Forever MerchantFrameDocumentation.lua:77-83 declares this member.
    #[cfg(feature = "client-wowforever")]
    {
        table_set_rust_fn_static(state, merchant, "GetNumJunkItems", junk::get_num_junk_items)?;
        table_set_rust_fn_static(
            state,
            merchant,
            "IsSellAllJunkEnabled",
            is_sell_all_junk_enabled,
        )?;
    }
    Ok(())
}

#[cfg(feature = "client-wowforever")]
fn is_sell_all_junk_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = borrow_state(state)?.merchant_sell_all_junk_feature_enabled;
    state.push(Val::Bool(enabled));
    Ok(1)
}

/// Host merchant IDs and explicit cost records supply visible offers and prices.
/// Missing stock and eligibility data retain the existing scenario defaults.
fn get_item_info(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let (item_id, costs) = {
        let sim = borrow_state(state)?;
        let Some((slot, item_id)) = index
            .checked_sub(1)
            .and_then(|slot| usize::try_from(slot).ok())
            .and_then(|index| {
                crate::lua_api::globals::real::merchant_filter::visible_merchant_item(&sim, index)
            })
        else {
            return Ok(0);
        };
        let costs = u32::try_from(slot)
            .ok()
            .and_then(|slot| sim.merchant_offer_costs.get(&slot).copied());
        (item_id, costs)
    };
    let Some(item) = crate::items::get_item(item_id) else {
        return Ok(0);
    };
    let info = create_table_with_capacity(state, 11);
    state.push(info);
    let name = create_string(state, item.name);
    table_set_static(state, info, "name", name);
    set_offer_fields(state, info, item.icon_file_data_id, costs);
    Ok(1)
}

fn is_merchant_item_refundable(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let refundable = {
        let sim = borrow_state(state)?;
        index
            .checked_sub(1)
            .and_then(|index| usize::try_from(index).ok())
            .and_then(|index| {
                crate::lua_api::globals::real::merchant_filter::visible_merchant_item(&sim, index)
            })
            .and_then(|(slot, _)| u32::try_from(slot).ok())
            .and_then(|slot| sim.merchant_item_refundable.get(&slot).copied())
            .unwrap_or(false)
    };
    state.push(rilua::Val::Bool(refundable));
    Ok(1)
}

fn set_offer_fields(
    state: &mut LuaState,
    info: rilua::Val,
    icon_id: u32,
    costs: Option<crate::lua_api::state::MerchantCostRecord>,
) {
    let price = costs.map_or(0.0, |record| record.price as f64);
    let has_extended_cost = costs.is_some_and(|record| offer_has_extended_cost(&record));
    for (key, value) in [
        ("texture", f64::from(icon_id)),
        ("price", price),
        ("stackCount", 1.0),
        ("numAvailable", -1.0),
    ] {
        table_set_static(state, info, key, rilua::Val::Num(value));
    }
    for (key, value) in [
        ("isPurchasable", true),
        ("isUsable", true),
        ("hasExtendedCost", has_extended_cost),
        ("isQuestStartItem", false),
    ] {
        table_set_static(state, info, key, rilua::Val::Bool(value));
    }
}

fn offer_has_extended_cost(record: &crate::lua_api::state::MerchantCostRecord) -> bool {
    record.item_costs.iter().any(|&item_id| item_id != 0)
        || record
            .currency_costs
            .iter()
            .any(|&currency_id| currency_id != 0)
}

#[cfg(any(feature = "retail-12-0-7", feature = "retail-12-1-0"))]
fn register_patch_12_0_7_merchant_frame_surface(
    state: &mut LuaState,
    merchant: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        merchant,
        "GetMerchantCurrencies",
        get_merchant_currencies,
    )
}

#[cfg(not(any(feature = "retail-12-0-7", feature = "retail-12-1-0")))]
fn register_patch_12_0_7_merchant_frame_surface(
    _state: &mut LuaState,
    _merchant: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    Ok(())
}

#[cfg(feature = "retail-12-0-7")]
fn get_merchant_currencies(state: &mut LuaState) -> LuaResult<u32> {
    // INFERRED: no SecretArguments declaration; reject secret extras for every caller.
    for value in &state.stack[state.base..state.top] {
        if rilua::table_security::is_secret_value(state, *value) {
            return Err(rilua::runtime_error(
                "GetMerchantCurrencies rejects secret arguments",
            ));
        }
    }
    // INFERRED: preserve host order; absent merchant input returns one empty table.
    let ids = crate::lua_api::methods::borrow_state(state)?
        .merchant_currencies
        .clone();
    let currencies = create_table(state);
    state.push(currencies);
    for (index, id) in ids.into_iter().enumerate() {
        crate::c_api::helpers::set_table_array(
            state,
            currencies,
            index as i64 + 1,
            rilua::Val::Num(f64::from(id)),
        );
    }
    Ok(1)
}

#[cfg(all(feature = "retail-12-1-0", not(feature = "retail-12-0-7")))]
fn get_merchant_currencies(state: &mut LuaState) -> LuaResult<u32> {
    let currencies = create_table(state);
    state.push(currencies);
    Ok(1)
}
