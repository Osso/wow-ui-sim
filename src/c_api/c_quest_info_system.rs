//! Host-seeded quest favor inputs for Retail 12.0.5 row 293 and per-quest short
//! expiration warning flags (Retail 12.0.7).
//! INFERRED context selection, false default clamp, strict u32 IDs, zero on a miss
//! and public outputs; cached declarations alone do not establish native parity.

use std::collections::HashMap;

use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::table_security::unwrap_secret;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

/// Exact host-declared amounts; no cap arithmetic or synthetic reward values.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QuestRewardFavor {
    pub amount: f64,
    pub cycle_capped_amount: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct QuestFavorState {
    pub rewards: HashMap<u32, QuestRewardFavor>,
    /// INFERRED explicit nil-query context; never consult selected_quest_log_id.
    pub context_quest_id: Option<u32>,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_QuestInfoSystem")?;
    #[cfg(feature = "retail-12-0-7")]
    table_set_rust_fn_static(
        state,
        namespace,
        "GetQuestHasShortExpirationWarning",
        get_quest_has_short_expiration_warning,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetQuestLogRewardFavor",
        get_quest_log_reward_favor,
    )
}

/// AllowedWhenUntainted; INFERRED nil or unflagged quests have no short warning.
#[cfg(feature = "retail-12-0-7")]
fn get_quest_has_short_expiration_warning(state: &mut LuaState) -> LuaResult<u32> {
    const NAME: &str = "C_QuestInfoSystem.GetQuestHasShortExpirationWarning";
    let quest = unwrap_secret(state, stack_val(state, 1))?;
    let quest_id = validate_quest_id(quest, NAME)?;
    let flagged = match quest_id {
        Some(quest_id) => borrow_state(state)?
            .quest_short_expiration_warnings
            .contains(&quest_id),
        None => false,
    };
    state.push(Val::Bool(flagged));
    Ok(1)
}

fn get_quest_log_reward_favor(state: &mut LuaState) -> LuaResult<u32> {
    let (quest_id, clamp) = read_favor_arguments(state)?;
    let amount = read_favor_amount(state, quest_id, clamp)?;
    state.push(Val::Num(amount));
    Ok(1)
}

/// AllowedWhenUntainted: authenticate every supplied argument before validation.
fn read_favor_arguments(state: &LuaState) -> LuaResult<(Option<u32>, bool)> {
    let quest = unwrap_secret(state, stack_val(state, 1))?;
    let clamp = unwrap_secret(state, stack_val(state, 2))?;
    // INFERRED public extras are ignored, but cannot bypass the secret policy.
    for value in state.stack.iter().take(state.top).skip(state.base + 2) {
        unwrap_secret(state, *value)?;
    }
    let quest_id = validate_quest_id(quest, "C_QuestInfoSystem.GetQuestLogRewardFavor")?;
    let clamp = match clamp {
        Val::Nil => false, // INFERRED omitted/nil clamp selects the raw host amount.
        Val::Bool(clamp) => clamp,
        _ => {
            return Err(runtime_error(
                "C_QuestInfoSystem.GetQuestLogRewardFavor: clampFavorToCycleCap must be boolean or nil",
            ));
        }
    };
    Ok((quest_id, clamp))
}

fn validate_quest_id(value: Val, function: &str) -> LuaResult<Option<u32>> {
    if matches!(value, Val::Nil) {
        return Ok(None);
    }
    if let Val::Num(number) = value {
        let integral = number.is_finite() && number.fract() == 0.0;
        let in_range = (0.0..=f64::from(u32::MAX)).contains(&number);
        if integral && in_range {
            return Ok(Some(number as u32));
        }
    }
    Err(runtime_error(format!(
        "{function}: questID must be a finite integral u32 number or nil"
    )))
}

fn read_favor_amount(state: &LuaState, quest_id: Option<u32>, clamp: bool) -> LuaResult<f64> {
    let sim = borrow_state(state)?;
    // INFERRED nil uses only this explicit host context, not selected quest state.
    // INFERRED miss: a quest with no host favor record rewards no favor.
    let reward = quest_id
        .or(sim.quest_favor.context_quest_id)
        .and_then(|quest_id| sim.quest_favor.rewards.get(&quest_id));
    Ok(match reward {
        Some(reward) if clamp => reward.cycle_capped_amount,
        Some(reward) => reward.amount,
        None => 0.0,
    })
}
