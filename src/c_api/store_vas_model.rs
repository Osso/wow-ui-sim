//! Empty-default host inputs for the Retail C_StoreSecure VAS surface.

use std::collections::{HashMap, HashSet};

mod responses;
pub use responses::{
    receive_bnet_transfer_validation, receive_guild_follow_info, receive_guild_master_info,
};

#[derive(Debug, Default)]
pub struct StoreVasState {
    pub realms: Vec<StoreVasRealmInput>,
    pub vas_realms: Vec<StoreVasRealmInput>,
    pub characters: Vec<StoreVasCharacterInput>,
    pub character_objects: Vec<StoreVasCharacterInput>,
    pub guild_character_guids: HashSet<String>,
    pub local_wow_account_guid: Option<String>,
    /// Exact account-name bytes map to an explicitly supplied WowAccount GUID.
    pub wow_account_guids: HashMap<Vec<u8>, String>,
    /// Computed client-side eligibility snapshots; not a server-supplied allow-list.
    pub race_eligibility: Vec<StoreVasRaceEligibilityInput>,
    pub guild_master_info: Vec<StoreVasGuildMasterInfoInput>,
    pub guild_follow_info: Vec<StoreVasGuildFollowInfoInput>,
    pub bnet_transfer_guid: Option<String>,
    pub bnet_game_accounts: Vec<Vec<u8>>,
    pub submitted_requests: Vec<StoreVasRequest>,
    pub purchase_eligible_product_ids: HashSet<i32>,
    pub product_prices: HashMap<i32, StoreVasPriceInput>,
    pub purchase_records: Vec<StoreVasPurchaseInput>,
    pub selected_purchase_product_id: Option<i32>,
    pub boost: Option<StoreVasBoostInput>,
    pub completion: Option<StoreVasCompletionInput>,
    pub completion_request_pending: bool,
    pub failure_marker: u32,
    pub received_failure_code: u32,
    pub pre_generated_external_transaction_id: Option<Vec<u8>>,
    pub opened_nydus_links: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StoreVasRequest {
    BnetTransferValidation {
        sequence: u64,
        account_name: Vec<u8>,
    },
    RealmGuildMasterInfo {
        realm_address: u32,
        character_guids: Vec<crate::c_api::store_guid::Outcome>,
    },
    CharacterGuildFollowInfo {
        guid: crate::c_api::store_guid::Outcome,
        realm_address: u32,
    },
    ProductPurchase {
        sequence: u64,
        product_id: i32,
        external_transaction_id: Option<Vec<u8>>,
    },
    PurchaseConfirmation {
        correlation: u64,
        confirmed: bool,
        dollars: u32,
        cents: u16,
    },
    VasPurchase {
        sequence: u64,
        product_id: i32,
        character_guid: crate::c_api::store_guid::Outcome,
        new_name: Option<Vec<u8>>,
        guild_name: Option<Vec<u8>>,
        guild_master_guid: Option<crate::c_api::store_guid::Outcome>,
        destination_realm: Option<u32>,
        wow_account_guid: Option<crate::c_api::store_guid::Outcome>,
        bnet_account_guid: Option<crate::c_api::store_guid::Outcome>,
        transfer_faction_bundle: bool,
        guild_follow: bool,
    },
    CompletionInfo {
        sequence: u64,
        product_id: i32,
        guid: crate::c_api::store_guid::Outcome,
    },
    AcknowledgeFailure {
        marker: u32,
    },
}

#[derive(Clone, Debug, Default)]
pub struct StoreVasRealmInput {
    pub virtual_realm_address: u32,
    pub realm_name: String,
    pub character_count: u32,
    pub pvp: Option<bool>,
    pub rp: Option<bool>,
}

#[derive(Clone, Debug, Default)]
pub struct StoreVasCharacterInput {
    pub realm_address: u32,
    pub realm_name: Option<String>,
    pub guid: Option<String>,
    /// Host rank in the native full-GUID comparison order; absent means unknown.
    pub guid_order: Option<u64>,
    pub name: Option<String>,
    pub class_name: Option<String>,
    pub race_name: Option<String>,
    pub level: u32,
    pub class_file_name: Option<String>,
    pub race_file_name: Option<String>,
    pub wow_account: u32,
    pub current_server: u32,
    pub faction: u32,
    pub sex: u32,
    pub create_screen_icon_atlas: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct StoreVasRaceEligibilityInput {
    pub guid: String,
    pub service_type: u8,
    /// None means the required client-side race data is absent; Some(empty) is a result.
    pub results: Option<Vec<StoreVasEligibleRaceInput>>,
}

/// One copied result row from a host-computed client eligibility query.
#[derive(Clone, Debug, Default)]
pub struct StoreVasEligibleRaceInput {
    pub race_name: String,
    pub is_allied_race: bool,
    pub is_heritage_armor_unlocked: bool,
}

#[derive(Clone, Debug, Default)]
pub struct StoreVasPriceInput {
    pub original_amount: u32,
    pub current_amount: u32,
}

#[derive(Clone, Debug, Default)]
pub struct StoreVasPurchaseInput {
    pub product_id: i32,
    pub status: i32,
    pub secondary_status: i32,
    pub wallet_name: String,
    pub original_amount: u32,
    pub current_amount: u32,
}

#[derive(Clone, Debug, Default)]
pub struct StoreVasBoostInput {
    pub product_id: i32,
    pub character_guid: String,
    pub realm_address: u32,
}

#[derive(Clone, Debug, Default)]
pub struct StoreVasCompletionInput {
    pub product_id: i32,
    pub guid: String,
    pub first_auxiliary: u32,
    pub second_auxiliary: u32,
}

#[derive(Clone, Debug, Default)]
pub struct StoreVasGuildFollowInfoInput {
    pub guid: String,
    pub transferred_realm: Option<String>,
    pub faction_changed: bool,
}

/// A copied cached guild-master result; no record is inserted by default.
#[derive(Clone, Debug, Default)]
pub struct StoreVasGuildMasterInfoInput {
    pub guid: String,
    pub guild_name: String,
    pub members: Vec<StoreVasGuildMemberInput>,
}

#[derive(Clone, Debug, Default)]
pub struct StoreVasGuildMemberInput {
    pub guid: Option<String>,
    pub member_name: String,
}
