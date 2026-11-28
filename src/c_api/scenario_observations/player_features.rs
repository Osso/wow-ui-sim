// C_ChatInfo.GetMentorChannelID: ChatInfoDocumentation.lua:257.
// C_Bank.FetchViewableBankTypes: Retail BankDocumentation.lua:240; Forever BankDocumentation.lua:271.
use super::{borrow_state, numbers, project};
use super::{borrow_state_mut, read_input, register_methods};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[derive(Debug, Default, serde::Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct PlayerFeatures {
    area_loot_allowed: bool,
    account_inventory_locked: bool,
    account_bank_enabled: bool,
    character_bank_enabled: bool,
    heroic_world_tier: bool,
    self_found_active: bool,
    viewable_bank_types: Vec<u32>,
    mentor_channel_id: u32,
}

pub(super) fn supply(state: &mut LuaState) -> LuaResult<u32> {
    let observation: PlayerFeatures = read_input(state)?;
    // ItemConstantsDocumentation.lua BankType: Character 0, Guild 1, Account 2.
    if observation
        .viewable_bank_types
        .iter()
        .any(|&bank_type| bank_type > 2)
    {
        return Err(rilua::runtime_error(
            "unknown BankType in viewable bank types",
        ));
    }
    borrow_state_mut(state)?
        .scenario_observations
        .player_features = observation;
    Ok(0)
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
    register_methods(
        state,
        "C_Bank",
        &[("FetchViewableBankTypes", viewable_bank_types)],
    )?;
    register_methods(
        state,
        "C_ChatInfo",
        &[("GetMentorChannelID", mentor_channel_id)],
    )?;
    register_playerinfo(state)
}

fn register_playerinfo(state: &mut LuaState) -> LuaResult<()> {
    register_methods(
        state,
        "C_PlayerInfo",
        &[
            // PlayerInfoDocumentation.lua:20
            ("CanPlayerUseAreaLoot", area_loot_allowed),
            // PlayerInfoDocumentation.lua:182
            ("HasAccountInventoryLock", account_inventory_locked),
            // PlayerInfoDocumentation.lua:206
            ("IsAccountBankEnabled", account_bank_enabled),
            // PlayerInfoDocumentation.lua:215
            ("IsCharacterBankEnabled", character_bank_enabled),
            // PlayerInfoDocumentation.lua:295
            ("IsPlayerInTimerunningHeroicWorldTier", heroic_world_tier),
            // PlayerInfoDocumentation.lua:322
            ("IsSelfFoundActive", self_found_active),
        ],
    )
}

fn area_loot_allowed(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| Val::Bool(o.player_features.area_loot_allowed))
}

fn account_inventory_locked(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| {
        Val::Bool(o.player_features.account_inventory_locked)
    })
}

fn account_bank_enabled(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| Val::Bool(o.player_features.account_bank_enabled))
}

fn character_bank_enabled(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| {
        Val::Bool(o.player_features.character_bank_enabled)
    })
}

fn heroic_world_tier(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| Val::Bool(o.player_features.heroic_world_tier))
}

fn self_found_active(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| Val::Bool(o.player_features.self_found_active))
}

fn viewable_bank_types(state: &mut LuaState) -> LuaResult<u32> {
    let banks = borrow_state(state)?
        .scenario_observations
        .player_features
        .viewable_bank_types
        .clone();
    numbers(state, banks)
}

fn mentor_channel_id(state: &mut LuaState) -> LuaResult<u32> {
    project(state, |o| {
        Val::Num(f64::from(o.player_features.mentor_channel_id))
    })
}
