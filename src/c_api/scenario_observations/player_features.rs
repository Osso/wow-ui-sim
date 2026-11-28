use super::project;
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
}

pub(super) fn supply(state: &mut LuaState) -> LuaResult<u32> {
    let observation = read_input(state)?;
    borrow_state_mut(state)?
        .scenario_observations
        .player_features = observation;
    Ok(0)
}

pub(super) fn register(state: &mut LuaState) -> LuaResult<()> {
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
