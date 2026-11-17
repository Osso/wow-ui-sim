use super::lookups::{read_guid, read_realm_address, read_text_bytes};
use super::pending_error;
use crate::c_api::c_store_glue;
use crate::c_api::store_guid::parse;
use crate::c_api::store_vas_model::StoreVasRequest;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use rilua::LuaResult;
use rilua::vm::state::LuaState;

const VALIDATE_BNET: &str = "ValidateBnetTransfer";
const REQUEST_REALM_GUILD: &str = "RequestRealmGuildMasterInfo";
const REQUEST_CHARACTER_FOLLOW: &str = "RequestCharacterGuildFollowInfo";

pub(super) fn validate_bnet_transfer(state: &mut LuaState) -> LuaResult<u32> {
    if c_store_glue::restricted_call(state)? {
        return Ok(0);
    }
    let account_name = read_text_bytes(state, 1, VALIDATE_BNET)?;
    let sequence = advance_request_sequence(state)?;
    let request = StoreVasRequest::BnetTransferValidation {
        sequence,
        account_name,
    };
    submit_if_connected(state, request)?;
    pending_error(state, VALIDATE_BNET)?;
    Ok(0)
}

pub(super) fn request_realm_guild_master_info(state: &mut LuaState) -> LuaResult<u32> {
    if c_store_glue::restricted_call(state)? {
        return Ok(0);
    }
    let realm_address = read_realm_address(state, 1, REQUEST_REALM_GUILD)?;
    let request = realm_guild_request(state, realm_address)?;
    submit_if_connected(state, request)?;
    pending_error(state, REQUEST_REALM_GUILD)?;
    Ok(0)
}

pub(super) fn request_character_guild_follow_info(state: &mut LuaState) -> LuaResult<u32> {
    if c_store_glue::restricted_call(state)? {
        return Ok(0);
    }
    let guid = read_guid(state, 1, REQUEST_CHARACTER_FOLLOW)?;
    let realm_address = read_realm_address(state, 2, REQUEST_CHARACTER_FOLLOW)?;
    let request = StoreVasRequest::CharacterGuildFollowInfo {
        guid,
        realm_address,
    };
    submit_if_connected(state, request)?;
    pending_error(state, REQUEST_CHARACTER_FOLLOW)?;
    Ok(0)
}

fn advance_request_sequence(state: &mut LuaState) -> LuaResult<u64> {
    let mut sim = borrow_state_mut(state)?;
    let sequence = &mut sim.store_glue.request_sequence;
    *sequence = sequence
        .checked_add(1)
        .expect("Store request observation counter exhausted");
    Ok(*sequence)
}

fn realm_guild_request(state: &LuaState, realm_address: u32) -> LuaResult<StoreVasRequest> {
    let sim = borrow_state(state)?;
    let characters = sim
        .store_glue
        .store_secure
        .vas
        .characters
        .iter()
        .filter(|character| character.realm_address == realm_address)
        .filter_map(|character| character.guid.as_deref())
        .map(|guid| parse(guid.as_bytes()))
        .collect();
    Ok(StoreVasRequest::RealmGuildMasterInfo {
        realm_address,
        character_guids: characters,
    })
}

fn submit_if_connected(state: &mut LuaState, request: StoreVasRequest) -> LuaResult<()> {
    let mut sim = borrow_state_mut(state)?;
    if sim.store_glue.connection_present {
        sim.store_glue
            .store_secure
            .vas
            .submitted_requests
            .push(request);
    }
    Ok(())
}
