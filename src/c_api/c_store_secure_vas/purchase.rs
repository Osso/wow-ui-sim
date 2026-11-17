use super::lookups::{
    find_character_object, guid_matches, numeric_value, read_guid, read_realm_address,
    read_secure_argument, read_signed_product_id, read_truthy_argument, service_type_for_product,
    text_bytes,
};
use super::pending_error;
use crate::c_api::c_store_glue;
use crate::c_api::helpers;
use crate::c_api::store_guid::Outcome;
use crate::c_api::store_vas_model::{StoreVasPriceInput, StoreVasPurchaseInput, StoreVasRequest};
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const OPEN_NYDUS: &str = "OpenNydusLink";
const GET_CONFIRMATION: &str = "GetConfirmationInfo";
const GET_UNREVOKED_BOOST: &str = "GetUnrevokedBoostInfo";
const PURCHASE_PRODUCT: &str = "PurchaseProduct";
const PURCHASE_CONFIRM: &str = "PurchaseProductConfirm";
const PURCHASE_VAS: &str = "PurchaseVASProduct";

pub(super) fn open_nydus_link(state: &mut LuaState) -> LuaResult<u32> {
    if c_store_glue::restricted_call(state)? {
        return Ok(0);
    }
    let entry_id = read_signed_product_id(state, OPEN_NYDUS)?;
    let link = nydus_link(state, entry_id)?;
    if let Some(link) = link.filter(|link| !link.is_empty()) {
        borrow_state_mut(state)?
            .store_glue
            .store_secure
            .vas
            .opened_nydus_links
            .push(link);
    }
    pending_error(state, OPEN_NYDUS)?;
    Ok(0)
}

pub(super) fn get_confirmation_info(state: &mut LuaState) -> LuaResult<u32> {
    if c_store_glue::restricted_call(state)? {
        return Ok(0);
    }
    let confirmation = confirmation_info(state)?;
    pending_error(state, GET_CONFIRMATION)?;
    let Some(confirmation) = confirmation else {
        return Ok(0);
    };
    push_confirmation_info(state, &confirmation);
    Ok(6)
}

pub(super) fn get_unrevoked_boost_info(state: &mut LuaState) -> LuaResult<u32> {
    if c_store_glue::restricted_call(state)? {
        return Ok(0);
    }
    let boost = unrevoked_boost_info(state)?;
    pending_error(state, GET_UNREVOKED_BOOST)?;
    let Some((product, character, realm)) = boost else {
        return Ok(0);
    };
    push_text(state, &product);
    push_text(state, &character);
    push_text(state, &realm);
    Ok(3)
}

pub(super) fn purchase_product(state: &mut LuaState) -> LuaResult<u32> {
    if c_store_glue::restricted_call(state)? {
        return Ok(0);
    }
    let product_id = read_signed_product_id(state, PURCHASE_PRODUCT)?;
    let accepted = prepare_product_purchase(state, product_id)?;
    pending_error(state, PURCHASE_PRODUCT)?;
    state.push(Val::Bool(accepted));
    Ok(1)
}

pub(super) fn purchase_product_confirm(state: &mut LuaState) -> LuaResult<u32> {
    if c_store_glue::restricted_call(state)? {
        return Ok(0);
    }
    let confirmed = read_truthy_argument(state, 1, PURCHASE_CONFIRM)?;
    let dollars = read_optional_u32(state, 2, PURCHASE_CONFIRM)?;
    let cents = read_optional_u32(state, 3, PURCHASE_CONFIRM)?;
    let accepted = prepare_purchase_confirmation(state, confirmed, dollars, cents)?;
    pending_error(state, PURCHASE_CONFIRM)?;
    state.push(Val::Bool(accepted));
    Ok(1)
}

pub(super) fn purchase_vas_product(state: &mut LuaState) -> LuaResult<u32> {
    if c_store_glue::restricted_call(state)? {
        return Ok(0);
    }
    let input = read_vas_purchase_input(state)?;
    let accepted = prepare_vas_purchase(state, input)?;
    pending_error(state, PURCHASE_VAS)?;
    state.push(Val::Bool(accepted));
    Ok(1)
}

#[derive(Debug)]
struct ConfirmationInfo {
    product_id: i32,
    wallet_name: String,
    original_amount: u32,
    current_amount: u32,
}

#[derive(Debug)]
struct VasPurchaseInput {
    product_id: i32,
    character_guid: Outcome,
    new_name: Option<Vec<u8>>,
    guild_name: Option<Vec<u8>>,
    guild_master_guid: Option<Outcome>,
    destination_realm: Option<u32>,
    wow_account_guid: Option<Outcome>,
    bnet_account_guid: Option<Outcome>,
    transfer_faction_bundle: bool,
    guild_follow: bool,
}

fn nydus_link(state: &LuaState, entry_id: i32) -> LuaResult<Option<String>> {
    let sim = borrow_state(state)?;
    Ok(sim
        .store_glue
        .store_secure
        .catalog
        .entries
        .iter()
        .find(|entry| entry.entry_id == entry_id)
        .and_then(|entry| entry.nydus_link.clone()))
}

fn confirmation_info(state: &LuaState) -> LuaResult<Option<ConfirmationInfo>> {
    let sim = borrow_state(state)?;
    let store = &sim.store_glue.store_secure;
    let vas = &store.vas;
    let Some(product_id) = vas.selected_purchase_product_id else {
        return Ok(None);
    };
    let Some(record) = vas
        .purchase_records
        .iter()
        .find(|record| record.product_id == product_id)
    else {
        return Ok(None);
    };
    if !store
        .catalog
        .products
        .iter()
        .any(|product| product.product_id == product_id)
    {
        return Ok(None);
    }
    let amounts = purchase_amounts(record, vas.product_prices.get(&product_id));
    let Some(amounts) = amounts else {
        return Ok(None);
    };
    Ok(Some(ConfirmationInfo {
        product_id,
        wallet_name: record.wallet_name.clone(),
        original_amount: amounts.original_amount,
        current_amount: amounts.current_amount,
    }))
}

fn purchase_amounts(
    record: &StoreVasPurchaseInput,
    product_prices: Option<&StoreVasPriceInput>,
) -> Option<StoreVasPriceInput> {
    if record.original_amount == 0 && record.current_amount == 0 {
        return product_prices.cloned();
    }
    Some(StoreVasPriceInput {
        original_amount: record.original_amount,
        current_amount: record.current_amount,
    })
}

fn push_confirmation_info(state: &mut LuaState, info: &ConfirmationInfo) {
    state.push(Val::Num(f64::from(info.product_id)));
    push_text(state, &info.wallet_name);
    push_price_parts(state, info.original_amount);
    push_price_parts(state, info.current_amount);
}

fn push_price_parts(state: &mut LuaState, amount: u32) {
    state.push(Val::Num(f64::from(amount / 10000)));
    state.push(Val::Num(f64::from(amount / 100 % 100)));
}

fn unrevoked_boost_info(state: &LuaState) -> LuaResult<Option<(String, String, String)>> {
    let boost = borrow_state(state)?
        .store_glue
        .store_secure
        .vas
        .boost
        .clone();
    let Some(boost) = boost else { return Ok(None) };
    let Some(product_name) = boost_product_name(state, boost.product_id)? else {
        return Ok(None);
    };
    let Some(character_name) = boost_character_name(state, &boost.character_guid)? else {
        return Ok(None);
    };
    let Some(realm_name) = boost_realm_name(state, boost.realm_address)? else {
        return Ok(None);
    };
    Ok(Some((product_name, character_name, realm_name)))
}

fn boost_product_name(state: &LuaState, product_id: i32) -> LuaResult<Option<String>> {
    let sim = borrow_state(state)?;
    let product = sim
        .store_glue
        .store_secure
        .catalog
        .products
        .iter()
        .find(|product| product.product_id == product_id);
    Ok(product.map(|product| product.shared_data.name.clone().unwrap_or_default()))
}

fn boost_character_name(state: &LuaState, guid_text: &str) -> LuaResult<Option<String>> {
    let guid = borrow_state(state)?
        .store_glue
        .resolve_guid(guid_text.as_bytes());
    Ok(find_character_object(state, &guid)?.and_then(|character| character.name))
}

fn boost_realm_name(state: &LuaState, realm_address: u32) -> LuaResult<Option<String>> {
    let sim = borrow_state(state)?;
    Ok(sim
        .store_glue
        .store_secure
        .vas
        .realms
        .iter()
        .find(|realm| realm.virtual_realm_address == realm_address)
        .map(|realm| realm.realm_name.clone()))
}

fn prepare_product_purchase(state: &mut LuaState, product_id: i32) -> LuaResult<bool> {
    if !purchase_is_allowed(state, product_id)? {
        return Ok(false);
    }
    let sequence = super::requests::advance_request_sequence(state)?;
    let external_id = borrow_state_mut(state)?
        .store_glue
        .store_secure
        .vas
        .pre_generated_external_transaction_id
        .take();
    let request = StoreVasRequest::ProductPurchase {
        sequence,
        product_id,
        external_transaction_id: external_id,
    };
    super::requests::submit_if_connected(state, request)?;
    Ok(true)
}

fn prepare_purchase_confirmation(
    state: &mut LuaState,
    confirmed: bool,
    dollars: Option<u32>,
    cents: Option<u32>,
) -> LuaResult<bool> {
    let Some(record) = selected_purchase(state)? else {
        return Ok(false);
    };
    if record.status != 9 || record.secondary_status != 0 {
        return Ok(false);
    }
    let amount = confirmation_amount(confirmed, dollars, cents);
    let Some((dollars, cents)) = amount else {
        return Ok(false);
    };
    let correlation = borrow_state(state)?.store_glue.request_sequence;
    let request = StoreVasRequest::PurchaseConfirmation {
        correlation,
        confirmed,
        dollars,
        cents,
    };
    super::requests::submit_if_connected(state, request)?;
    Ok(true)
}

fn selected_purchase(state: &LuaState) -> LuaResult<Option<StoreVasPurchaseInput>> {
    let sim = borrow_state(state)?;
    let vas = &sim.store_glue.store_secure.vas;
    let Some(product_id) = vas.selected_purchase_product_id else {
        return Ok(None);
    };
    Ok(vas
        .purchase_records
        .iter()
        .find(|record| record.product_id == product_id)
        .cloned())
}

fn confirmation_amount(
    confirmed: bool,
    dollars: Option<u32>,
    cents: Option<u32>,
) -> Option<(u32, u16)> {
    if !confirmed {
        return Some((0, 0));
    }
    let dollars = dollars?;
    let cents = cents? as u16;
    (cents < 100).then_some((dollars, cents))
}

fn purchase_is_allowed(state: &LuaState, product_id: i32) -> LuaResult<bool> {
    let sim = borrow_state(state)?;
    let product_exists = sim
        .store_glue
        .store_secure
        .catalog
        .products
        .iter()
        .any(|product| product.product_id == product_id);
    let purchase_allowed = sim
        .store_glue
        .store_secure
        .vas
        .purchase_eligible_product_ids
        .contains(&product_id);
    Ok(product_exists && purchase_allowed)
}

fn read_vas_purchase_input(state: &LuaState) -> LuaResult<VasPurchaseInput> {
    Ok(VasPurchaseInput {
        product_id: read_signed_product_id(state, PURCHASE_VAS)?,
        character_guid: read_guid(state, 2, PURCHASE_VAS)?,
        new_name: optional_text_bytes(state, 3, PURCHASE_VAS)?,
        guild_name: optional_text_bytes(state, 4, PURCHASE_VAS)?,
        guild_master_guid: optional_guid(state, 5, PURCHASE_VAS)?,
        destination_realm: optional_realm(state, 6, PURCHASE_VAS)?,
        wow_account_guid: optional_guid(state, 7, PURCHASE_VAS)?,
        bnet_account_guid: optional_guid(state, 8, PURCHASE_VAS)?,
        transfer_faction_bundle: read_truthy_argument(state, 9, PURCHASE_VAS)?,
        guild_follow: read_truthy_argument(state, 10, PURCHASE_VAS)?,
    })
}

fn prepare_vas_purchase(state: &mut LuaState, input: VasPurchaseInput) -> LuaResult<bool> {
    let product_id = remap_product(input.product_id, input.transfer_faction_bundle);
    if !vas_purchase_is_allowed(state, &input, product_id)? {
        return Ok(false);
    }
    let sequence = super::requests::advance_request_sequence(state)?;
    let request = vas_purchase_request(input, product_id, sequence);
    super::requests::submit_if_connected(state, request)?;
    Ok(true)
}

fn remap_product(product_id: i32, faction_bundle: bool) -> i32 {
    if !faction_bundle {
        return product_id;
    }
    match product_id {
        189 => 239,
        476 => 477,
        _ => product_id,
    }
}

fn vas_purchase_is_allowed(
    state: &LuaState,
    input: &VasPurchaseInput,
    product_id: i32,
) -> LuaResult<bool> {
    if !purchase_is_allowed(state, product_id)? {
        return Ok(false);
    }
    let Some(service) = service_type_for_product(state, product_id)? else {
        return Ok(false);
    };
    if !character_exists(state, &input.character_guid)? {
        return Ok(false);
    }
    Ok(service_fields_are_valid(input, service))
}

fn character_exists(state: &LuaState, guid: &Outcome) -> LuaResult<bool> {
    let sim = borrow_state(state)?;
    Ok(sim
        .store_glue
        .store_secure
        .vas
        .characters
        .iter()
        .any(|character| {
            character
                .guid
                .as_deref()
                .is_some_and(|text| guid_matches(guid, text))
        }))
}

fn service_fields_are_valid(input: &VasPurchaseInput, service: u32) -> bool {
    if input.guild_follow && !matches!(service, 1 | 4 | 5) {
        return false;
    }
    if matches!(service, 0 | 6) && input.new_name.is_none() {
        return false;
    }
    if service == 7 && (input.guild_master_guid.is_none() || !has_name(input)) {
        return false;
    }
    if matches!(service, 8 | 9) && input.guild_master_guid.is_none() {
        return false;
    }
    destination_is_valid(input, service)
}

fn has_name(input: &VasPurchaseInput) -> bool {
    input.new_name.is_some() || input.guild_name.is_some()
}

fn destination_is_valid(input: &VasPurchaseInput, service: u32) -> bool {
    if !matches!(service, 4 | 5 | 8 | 9) {
        return true;
    }
    let wow_account = has_guid_kind(input.wow_account_guid.as_ref(), "WowAccount");
    let bnet_account = has_guid_kind(input.bnet_account_guid.as_ref(), "BNetAccount");
    if wow_account && bnet_account {
        return true;
    }
    if wow_account || bnet_account {
        return false;
    }
    input
        .destination_realm
        .is_some_and(|realm| realm != 0 && realm != u32::MAX)
}

fn has_guid_kind(guid: Option<&Outcome>, kind: &str) -> bool {
    matches!(guid, Some(Outcome::Known(identity)) if identity.kind == kind)
}

fn vas_purchase_request(
    input: VasPurchaseInput,
    product_id: i32,
    sequence: u64,
) -> StoreVasRequest {
    StoreVasRequest::VasPurchase {
        sequence,
        product_id,
        character_guid: input.character_guid,
        new_name: input.new_name,
        guild_name: input.guild_name,
        guild_master_guid: input.guild_master_guid,
        destination_realm: input.destination_realm,
        wow_account_guid: input.wow_account_guid,
        bnet_account_guid: input.bnet_account_guid,
        transfer_faction_bundle: input.transfer_faction_bundle,
        guild_follow: input.guild_follow,
    }
}

fn optional_text_bytes(state: &LuaState, index: i32, method: &str) -> LuaResult<Option<Vec<u8>>> {
    let Some(value) = optional_argument(state, index, method)? else {
        return Ok(None);
    };
    text_bytes(state, value, index, method).map(Some)
}

fn optional_guid(state: &LuaState, index: i32, method: &str) -> LuaResult<Option<Outcome>> {
    let Some(bytes) = optional_text_bytes(state, index, method)? else {
        return Ok(None);
    };
    Ok(Some(borrow_state(state)?.store_glue.resolve_guid(&bytes)))
}

fn optional_realm(state: &LuaState, index: i32, method: &str) -> LuaResult<Option<u32>> {
    let Some(value) = optional_argument(state, index, method)? else {
        return Ok(None);
    };
    let number = numeric_value(state, value)
        .filter(|number| number.is_finite())
        .ok_or_else(|| helpers::bad_argument(index, method, super::INVALID_ARGUMENT))?;
    Ok(Some(number.trunc() as u32))
}

fn read_optional_u32(state: &LuaState, index: i32, method: &str) -> LuaResult<Option<u32>> {
    let Some(value) = optional_argument(state, index, method)? else {
        return Ok(None);
    };
    let number = numeric_value(state, value)
        .filter(|number| number.is_finite())
        .ok_or_else(|| helpers::bad_argument(index, method, super::INVALID_ARGUMENT))?;
    Ok(Some(number.trunc() as u32))
}

fn optional_argument(state: &LuaState, index: i32, method: &str) -> LuaResult<Option<Val>> {
    if stack_val(state, index) == Val::Nil {
        return Ok(None);
    }
    read_secure_argument(state, index, method).map(Some)
}

fn push_text(state: &mut LuaState, text: &str) {
    let value = create_string(state, text);
    state.push(value);
}
