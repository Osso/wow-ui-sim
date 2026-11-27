//! Retail 12.1.5 `C_ProfSpecs` path state and entry queries (R88ax.1, R88ax.3).

use crate::lua_api::methods::{borrow_state, create_table, table_set_static};
use crate::lua_bridge::table_set_rust_fn_static;
use crate::screen::ScreenKind;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

const STATE_USAGE: &str = "Usage: local state = C_ProfSpecs.GetStateForPath(pathID, configID)";
const SPEND_ENTRY_USAGE: &str = "Usage: local entryID = C_ProfSpecs.GetSpendEntryForPath(pathID)";
const UNLOCK_ENTRY_USAGE: &str = "Usage: local entryID = C_ProfSpecs.GetUnlockEntryForPath(pathID)";
const SPEND_CURRENCY_USAGE: &str =
    "Usage: local currencyID = C_ProfSpecs.GetSpendCurrencyForPath(pathID)";
const PERKS_USAGE: &str = "Usage: local perkInfos = C_ProfSpecs.GetPerksForPath(pathID)";
const PERK_STATE_USAGE: &str = "Usage: local state = C_ProfSpecs.GetStateForPerk(perkID, configID)";
const PERK_UNLOCK_RANK_USAGE: &str =
    "Usage: local unlockRank = C_ProfSpecs.GetUnlockRankForPerk(perkID)";
const PERK_ENTRY_USAGE: &str = "Usage: local entryID = C_ProfSpecs.GetEntryIDForPerk(perkID)";

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if borrow_state(state)?.screen_kind != ScreenKind::Game {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_ProfSpecs")?;
    table_set_rust_fn_static(state, namespace, "GetStateForPath", get_state_for_path)?;
    table_set_rust_fn_static(state, namespace, "GetSpendEntryForPath", get_spend_entry)?;
    table_set_rust_fn_static(state, namespace, "GetUnlockEntryForPath", get_unlock_entry)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetSpendCurrencyForPath",
        get_spend_currency,
    )?;
    table_set_rust_fn_static(state, namespace, "GetPerksForPath", get_perks_for_path)?;
    table_set_rust_fn_static(state, namespace, "GetStateForPerk", get_state_for_perk)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetUnlockRankForPerk",
        get_unlock_rank_for_perk,
    )?;
    table_set_rust_fn_static(state, namespace, "GetEntryIDForPerk", get_entry_id_for_perk)
}

fn read_path_id(state: &LuaState, api: &str, usage: &str) -> LuaResult<i32> {
    super::helpers::read_untainted_signed32_argument(state, 1, api, usage)
}

fn read_state_arguments(state: &LuaState) -> LuaResult<(i32, i32)> {
    let path_id =
        super::helpers::read_untainted_signed32_argument(state, 1, "GetStateForPath", STATE_USAGE)?;
    let config_id =
        super::helpers::read_untainted_signed32_argument(state, 2, "GetStateForPath", STATE_USAGE)?;
    Ok((path_id, config_id))
}

fn get_state_for_path(state: &mut LuaState) -> LuaResult<u32> {
    let (path_id, config_id) = read_state_arguments(state)?;
    let result = borrow_state(state)?
        .profession_spec_inputs
        .state_for_path(path_id, config_id);
    state.push(Val::Num(f64::from(result)));
    Ok(1)
}

fn get_spend_entry(state: &mut LuaState) -> LuaResult<u32> {
    push_entry_id(state, "GetSpendEntryForPath", SPEND_ENTRY_USAGE, true)
}

fn get_unlock_entry(state: &mut LuaState) -> LuaResult<u32> {
    push_entry_id(state, "GetUnlockEntryForPath", UNLOCK_ENTRY_USAGE, false)
}

fn push_entry_id(state: &mut LuaState, api: &str, usage: &str, spend: bool) -> LuaResult<u32> {
    let path_id = read_path_id(state, api, usage)?;
    let entry_id = {
        let state_ref = borrow_state(state)?;
        let inputs = &state_ref.profession_spec_inputs;
        let entries = if spend {
            &inputs.path_spend_entry_ids
        } else {
            &inputs.path_unlock_entry_ids
        };
        entries.get(&path_id).copied().unwrap_or_default()
    };
    state.push(Val::Num(f64::from(entry_id)));
    Ok(1)
}

fn get_spend_currency(state: &mut LuaState) -> LuaResult<u32> {
    let path_id = read_path_id(state, "GetSpendCurrencyForPath", SPEND_CURRENCY_USAGE)?;
    let currency_id = borrow_state(state)?
        .profession_spec_inputs
        .path_spend_currency_ids
        .get(&path_id)
        .copied();
    state.push(currency_id.map_or(Val::Nil, |id| Val::Num(f64::from(id))));
    Ok(1)
}

fn get_perks_for_path(state: &mut LuaState) -> LuaResult<u32> {
    let path_id = read_path_id(state, "GetPerksForPath", PERKS_USAGE)?;
    let perks = borrow_state(state)?
        .profession_spec_inputs
        .perks_for_path(path_id);
    let list = create_table(state);
    state.push(list);
    for (index, (perk_id, is_major)) in perks.into_iter().enumerate() {
        let perk = create_table(state);
        table_set_static(state, perk, "perkID", Val::Num(f64::from(perk_id)));
        table_set_static(state, perk, "isMajorPerk", Val::Bool(is_major));
        super::helpers::set_table_array(state, list, (index + 1) as i64, perk);
    }
    Ok(1)
}

fn get_state_for_perk(state: &mut LuaState) -> LuaResult<u32> {
    let (perk_id, config_id) = read_perk_state_arguments(state)?;
    let result = borrow_state(state)?
        .profession_spec_inputs
        .state_for_perk(perk_id, config_id);
    state.push(Val::Num(f64::from(result)));
    Ok(1)
}

fn read_perk_state_arguments(state: &LuaState) -> LuaResult<(i32, i32)> {
    let perk_id = super::helpers::read_untainted_signed32_argument(
        state,
        1,
        "GetStateForPerk",
        PERK_STATE_USAGE,
    )?;
    let config_id = super::helpers::read_untainted_signed32_argument(
        state,
        2,
        "GetStateForPerk",
        PERK_STATE_USAGE,
    )?;
    Ok((perk_id, config_id))
}

fn get_unlock_rank_for_perk(state: &mut LuaState) -> LuaResult<u32> {
    let perk_id = read_path_id(state, "GetUnlockRankForPerk", PERK_UNLOCK_RANK_USAGE)?;
    let rank = borrow_state(state)?
        .profession_spec_inputs
        .perk_unlock_ranks
        .get(&perk_id)
        .copied();
    state.push(rank.map_or(Val::Nil, |value| Val::Num(f64::from(value))));
    Ok(1)
}

fn get_entry_id_for_perk(state: &mut LuaState) -> LuaResult<u32> {
    let perk_id = read_path_id(state, "GetEntryIDForPerk", PERK_ENTRY_USAGE)?;
    let entry_id = borrow_state(state)?
        .profession_spec_inputs
        .perk_entry_ids
        .get(&perk_id)
        .copied()
        .unwrap_or_default();
    state.push(Val::Num(f64::from(entry_id)));
    Ok(1)
}
