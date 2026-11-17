//! Verified Retail StoreSecure VAS lookups and operations.

mod lookups;
mod requests;

use super::c_store_glue::{Member, publish_namespace};
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::borrow_state_mut;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, runtime_error};

pub(super) const INVALID_ARGUMENT: &str = "invalid argument";

const MEMBERS: &[Member] = &[
    ("GetRealmList", lookups::get_realm_list),
    ("GetVASRealmList", lookups::get_vas_realm_list),
    ("GetCharactersForRealm", lookups::get_characters_for_realm),
    (
        "GetCharacterInfoByGUID",
        lookups::get_character_info_by_guid,
    ),
    (
        "GetWoWAccountGUIDFromName",
        lookups::get_wow_account_guid_from_name,
    ),
    (
        "GetEligibleRacesForVASService",
        lookups::get_eligible_races_for_vas_service,
    ),
    (
        "GetVASGuildMasterInfoForCharacterByGUID",
        lookups::get_guild_master_info_for_character,
    ),
    ("GetVasServiceType", lookups::get_vas_service_type),
    ("ValidateBnetTransfer", requests::validate_bnet_transfer),
    (
        "RequestRealmGuildMasterInfo",
        requests::request_realm_guild_master_info,
    ),
    (
        "RequestCharacterGuildFollowInfo",
        requests::request_character_guild_follow_info,
    ),
];

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if ACTIVE != ClientProfile::Retail {
        return Ok(());
    }
    publish_namespace(state, "C_StoreSecure", MEMBERS)
}

pub(super) fn pending_error(state: &mut LuaState, method: &str) -> LuaResult<()> {
    let message = borrow_state_mut(state)?.store_glue.pending_error.take();
    if let Some(message) = message {
        return Err(runtime_error(format!("{method}(): {message}")));
    }
    Ok(())
}
