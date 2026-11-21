//! Supplied quest-dialog major-faction reputation, separate from legacy rewards.

use std::collections::HashMap;

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::SimState;
use crate::lua_api::methods::{
    borrow_state, create_string, create_table, create_table_with_fields, table_set_num,
};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// Environment-owned input. Native classification of legacy faction pairs is unknown.
#[derive(Debug, Clone, Default)]
pub struct QuestOfferState {
    /// Quest ID -> ordered (faction ID, reward amount) rows; absence is not an empty list.
    pub major_faction_rewards: HashMap<i32, Vec<(i32, i32)>>,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    // These profiles' cached QuestOfferDocumentation declares this query.
    if !matches!(
        ACTIVE,
        ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
    ) {
        return Ok(());
    }
    let namespace = super::ensure_namespace(state, "C_QuestOffer")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetQuestOfferMajorFactionReputationRewards",
        get_major_faction_reputation_rewards,
    )?;
    if matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever) {
        register_progress_requirement_queries(state)?;
    }
    Ok(())
}

fn register_progress_requirement_queries(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_QuestOffer")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetQuestRequiredCurrencyInfo",
        get_required_currency_info,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetHideRequiredItems",
        get_hide_required_items,
    )?;
    Ok(())
}

fn get_hide_required_items(state: &mut LuaState) -> LuaResult<u32> {
    // INFERRED (R88db pending): absent inputs leave required rows visible.
    let hidden = {
        let sim = borrow_state(state)?;
        crate::c_api::quest_objectives::progress_requirements_for_active_dialog(&sim)
            .map(|requirements| requirements.hide_required_items)
            .unwrap_or(false)
    };
    state.push(Val::Bool(hidden));
    Ok(1)
}

fn get_required_currency_info(state: &mut LuaState) -> LuaResult<u32> {
    let index = Option::<f64>::from_stack(state, 1)?.unwrap_or(0.0) as usize;
    let currency = {
        let sim = borrow_state(state)?;
        crate::c_api::quest_objectives::progress_requirements_for_active_dialog(&sim)
            .and_then(|requirements| requirements.currencies.get(index.saturating_sub(1)))
            .cloned()
    };
    let Some(currency) = currency.filter(|_| index > 0) else {
        return Ok(0);
    };
    let name = create_string(state, &currency.name);
    let info = create_table_with_fields(
        state,
        &[
            ("texture", Val::Num(f64::from(currency.texture))),
            ("name", name),
            ("currencyID", Val::Num(f64::from(currency.currency_id))),
            ("quality", Val::Num(f64::from(currency.quality))),
            (
                "requiredAmount",
                Val::Num(f64::from(currency.required_amount)),
            ),
        ],
    );
    state.push(info);
    Ok(1)
}

fn rewards_for_active_dialog(sim: &SimState) -> Option<&[(i32, i32)]> {
    let (quest_id, _) = sim.quest_log_entries.readiness.dialog?;
    let selected = sim
        .selected_quest_log_id
        .and_then(|id| i32::try_from(id).ok());
    if selected != Some(quest_id) || !super::quest_readiness::has_active_npc_quest(sim, quest_id) {
        return None;
    }
    sim.quest_log_entries
        .offer
        .major_faction_rewards
        .get(&quest_id)
        .map(Vec::as_slice)
}

fn get_major_faction_reputation_rewards(state: &mut LuaState) -> LuaResult<u32> {
    let rewards = {
        let sim = borrow_state(state)?;
        rewards_for_active_dialog(&sim).map(|rewards| rewards.to_vec())
    };
    // MayReturnNothing permits this; choosing it for absent input is simulator policy.
    let Some(rewards) = rewards else {
        return Ok(0);
    };
    let Val::Table(list) = create_table(state) else {
        unreachable!("create_table must return a table");
    };
    for (index, (faction_id, reward_amount)) in rewards.into_iter().enumerate() {
        let row = create_table_with_fields(
            state,
            &[
                ("factionID", Val::Num(f64::from(faction_id))),
                ("rewardAmount", Val::Num(f64::from(reward_amount))),
            ],
        );
        table_set_num(state, list, (index + 1) as f64, row);
    }
    state.push(Val::Table(list));
    Ok(1)
}
