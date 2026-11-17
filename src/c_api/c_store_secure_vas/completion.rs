use super::lookups::{find_character_object, service_type_for_product};
use crate::c_api::c_store_glue;
use crate::c_api::store_guid::Outcome;
use crate::c_api::store_vas_model::{StoreVasCompletionInput, StoreVasRequest};
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const GET_COMPLETION: &str = "GetVASCompletionInfo";
const GET_FAILURE: &str = "GetFailureInfo";
const ACK_FAILURE: &str = "AckFailure";
const CLEAR_EXTERNAL_ID: &str = "ClearPreGeneratedExternalTransactionID";

pub(super) fn get_completion_info(state: &mut LuaState) -> LuaResult<u32> {
    if c_store_glue::restricted_call(state)? {
        return Ok(0);
    }
    let completion = completion_result(state)?;
    if let CompletionResult::Pending { product_id, guid } = &completion {
        request_completion_info(state, *product_id, guid.clone())?;
    }
    super::pending_error(state, GET_COMPLETION)?;
    match completion {
        CompletionResult::Data(data) => push_completion_info(state, &data),
        CompletionResult::Pending { .. } | CompletionResult::Absent => return Ok(0),
    }
    Ok(4)
}

pub(super) fn get_failure_info(state: &mut LuaState) -> LuaResult<u32> {
    if c_store_glue::restricted_call(state)? {
        return Ok(0);
    }
    let (marker, received_code) = {
        let sim = borrow_state(state)?;
        let vas = &sim.store_glue.store_secure.vas;
        (vas.failure_marker, vas.received_failure_code)
    };
    super::pending_error(state, GET_FAILURE)?;
    let error = if marker == 0 {
        Val::Nil
    } else {
        Val::Num(f64::from(map_failure_code(received_code)))
    };
    state.push(error);
    state.push(Val::Nil);
    Ok(2)
}

pub(super) fn ack_failure(state: &mut LuaState) -> LuaResult<u32> {
    if c_store_glue::restricted_call(state)? {
        return Ok(0);
    }
    let marker = borrow_state(state)?
        .store_glue
        .store_secure
        .vas
        .failure_marker;
    let accepted = acknowledge_failure(state, marker)?;
    super::pending_error(state, ACK_FAILURE)?;
    state.push(Val::Bool(accepted));
    Ok(1)
}

pub(super) fn clear_external_transaction_id(state: &mut LuaState) -> LuaResult<u32> {
    if c_store_glue::restricted_call(state)? {
        return Ok(0);
    }
    borrow_state_mut(state)?
        .store_glue
        .store_secure
        .vas
        .pre_generated_external_transaction_id = None;
    super::pending_error(state, CLEAR_EXTERNAL_ID)?;
    Ok(0)
}

#[derive(Clone, Debug)]
enum CompletionResult {
    Data(CompletionData),
    Pending { product_id: i32, guid: Outcome },
    Absent,
}

#[derive(Clone, Debug)]
struct CompletionData {
    product_id: i32,
    guid: String,
    realm_name: String,
    should_handle: bool,
}

fn completion_result(state: &LuaState) -> LuaResult<CompletionResult> {
    let Some(input) = borrow_state(state)?
        .store_glue
        .store_secure
        .vas
        .completion
        .clone()
    else {
        return Ok(CompletionResult::Absent);
    };
    if !product_exists(state, input.product_id)? {
        return Ok(CompletionResult::Absent);
    }
    let service = service_type_for_product(state, input.product_id)?;
    if service.is_some_and(|value| matches!(value, 4 | 5 | 8 | 9)) {
        return Ok(CompletionResult::Data(transfer_completion(input)));
    }
    character_completion(state, input)
}

fn product_exists(state: &LuaState, product_id: i32) -> LuaResult<bool> {
    let sim = borrow_state(state)?;
    Ok(sim
        .store_glue
        .store_secure
        .catalog
        .products
        .iter()
        .any(|product| product.product_id == product_id))
}

fn transfer_completion(input: StoreVasCompletionInput) -> CompletionData {
    CompletionData {
        product_id: input.product_id,
        guid: input.guid,
        realm_name: String::new(),
        should_handle: input.first_auxiliary == 0 && input.second_auxiliary == 0,
    }
}

fn character_completion(
    state: &LuaState,
    input: StoreVasCompletionInput,
) -> LuaResult<CompletionResult> {
    let guid = borrow_state(state)?
        .store_glue
        .resolve_guid(input.guid.as_bytes());
    let character = find_character_object(state, &guid)?;
    match character.and_then(|row| row.realm_name) {
        Some(realm_name) => Ok(CompletionResult::Data(CompletionData {
            product_id: input.product_id,
            guid: input.guid,
            realm_name,
            should_handle: true,
        })),
        None => Ok(CompletionResult::Pending {
            product_id: input.product_id,
            guid,
        }),
    }
}

fn request_completion_info(state: &mut LuaState, product_id: i32, guid: Outcome) -> LuaResult<()> {
    let sequence = super::requests::advance_request_sequence(state)?;
    borrow_state_mut(state)?
        .store_glue
        .store_secure
        .vas
        .completion_request_pending = true;
    super::requests::submit_if_connected(
        state,
        StoreVasRequest::CompletionInfo {
            sequence,
            product_id,
            guid,
        },
    )
}

fn push_completion_info(state: &mut LuaState, info: &CompletionData) {
    state.push(Val::Num(f64::from(info.product_id)));
    push_guid(state, &info.guid);
    let realm_name = create_string(state, &info.realm_name);
    state.push(realm_name);
    state.push(Val::Bool(info.should_handle));
}

fn push_guid(state: &mut LuaState, guid: &str) {
    let value = if guid.is_empty() {
        Val::Nil
    } else {
        create_string(state, guid)
    };
    state.push(value);
}

fn map_failure_code(code: u32) -> u32 {
    match code {
        0 => 0,
        1 => 9,
        2 => 2,
        12 => 3,
        13 | 14 => 4,
        25 => 1,
        28 | 29 => 5,
        34 => 8,
        46 => 10,
        47 => 11,
        48 => 12,
        66 => 13,
        _ => 6,
    }
}

fn acknowledge_failure(state: &mut LuaState, marker: u32) -> LuaResult<bool> {
    if marker == 0 {
        return Ok(false);
    }
    let mut sim = borrow_state_mut(state)?;
    let vas = &mut sim.store_glue.store_secure.vas;
    vas.failure_marker = 0;
    drop(sim);
    super::requests::submit_if_connected(state, StoreVasRequest::AcknowledgeFailure { marker })?;
    Ok(true)
}
