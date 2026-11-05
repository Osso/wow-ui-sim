//! WM97 DQ8 weekly reward tooltip. Behavior evidence is Retail 70688; the entry
//! data is host-supplied because the native population and invalidation paths are
//! UNKNOWN. Ready, occupancy and descriptor presence stay separate inputs.

use crate::c_api::item_descriptor::{ItemDescriptor, parse_item_link};
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::globals::missing_surface::tooltip_info::tooltip_for_item_context;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, val_to_string};
use crate::lua_bridge::{FromStack, TableBuilder, stack_val, table_set_rust_fn_static};
use rilua::table_security::unwrap_secret;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};
use std::collections::HashMap;

const API_NAME: &str = "C_TooltipInfo.GetWeeklyReward";
/// Largest key the lossless publisher emits as an ordinary Lua number.
const LOSSELESS_NUMBER_MAX: u64 = 9_007_199_254_740_991;

#[derive(Debug, Clone, Default)]
pub struct WeeklyRewards {
    /// Ready state is independent of entry occupancy.
    pub ready: bool,
    /// Occupied entries that carry a descriptor, keyed by the exact 64-bit key.
    pub entries: HashMap<u64, ItemDescriptor>,
}

pub(crate) fn register(
    state: &mut LuaState,
    namespace: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetWeeklyReward", get_weekly_reward)
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    if !matches!(ACTIVE, ClientProfile::Retail | ClientProfile::Ptr) {
        return Ok(builder);
    }
    builder
        .set_function("SetWeeklyRewardsReady", set_ready)?
        .set_function("SetWeeklyRewardEntry", set_entry)?
        .set_function("ClearWeeklyRewards", clear)
}

fn set_ready(state: &mut LuaState) -> LuaResult<u32> {
    let Val::Bool(ready) = unwrap_secret(state, stack_val(state, 1))? else {
        return Err(runtime_error(
            "A_Admin.SetWeeklyRewardsReady: expected boolean",
        ));
    };
    let mut sim = borrow_state_mut(state)?;
    sim.weekly_rewards.ready = ready;
    Ok(0)
}

fn set_entry(state: &mut LuaState) -> LuaResult<u32> {
    let value = unwrap_secret(state, stack_val(state, 1))?;
    let key = parse_lossless_key(state, value, "A_Admin.SetWeeklyRewardEntry")?;
    let link = String::from_stack(state, 2)?;
    let Some(descriptor) = parse_item_link(&link).map(|parsed| parsed.descriptor) else {
        return Err(runtime_error(
            "A_Admin.SetWeeklyRewardEntry: entry requires a descriptor-bearing item link",
        ));
    };
    let mut sim = borrow_state_mut(state)?;
    sim.weekly_rewards.entries.insert(key, descriptor);
    Ok(0)
}

fn clear(state: &mut LuaState) -> LuaResult<u32> {
    let mut sim = borrow_state_mut(state)?;
    sim.weekly_rewards = WeeklyRewards::default();
    Ok(0)
}

fn get_weekly_reward(state: &mut LuaState) -> LuaResult<u32> {
    let value = unwrap_secret(state, stack_val(state, 1))?;
    let key = parse_lossless_key(state, value, API_NAME)?;
    let descriptor = {
        let model = borrow_state(state)?;
        if model.weekly_rewards.ready {
            model.weekly_rewards.entries.get(&key).cloned()
        } else {
            None
        }
    };
    let Some(descriptor) = descriptor else {
        // EMULATED: WM82 leaves native miss arity UNKNOWN; matches the shared policy.
        return Ok(0);
    };
    let player_level = {
        let model = borrow_state(state)?;
        let player = &model.player;
        if player.effective_level != 0 {
            player.effective_level
        } else {
            player.level
        }
    };
    let item = descriptor
        .resolve(player_level)
        .map_err(|error| runtime_error(format!("{API_NAME}: {error}")))?;
    let tooltip = tooltip_for_item_context(state, descriptor.item_id, item.as_ref());
    state.push(tooltip);
    Ok(1)
}

/// WM97 DQ8: ordinary Lua numbers through 2^53-1 truncate toward zero; larger keys
/// use `0x` plus exactly sixteen hex digits and must exceed that same threshold.
fn parse_lossless_key(state: &mut LuaState, value: Val, api: &str) -> LuaResult<u64> {
    match value {
        Val::Num(number) => {
            if !number.is_finite() || !(0.0..=LOSSELESS_NUMBER_MAX as f64).contains(&number) {
                return Err(runtime_error(format!(
                    "{api}: number key outside the lossless range"
                )));
            }
            Ok(number.trunc() as u64)
        }
        Val::Str(_) => {
            let text = val_to_string(state, value)
                .ok_or_else(|| runtime_error(format!("{api}: unreadable text key")))?;
            parse_encoded_key(&text)
                .ok_or_else(|| runtime_error(format!("{api}: malformed 0x 16-digit key")))
        }
        _ => Err(runtime_error(format!(
            "{api}: key must be a number or 0x text"
        ))),
    }
}

fn parse_encoded_key(text: &str) -> Option<u64> {
    let digits = text
        .get(..2)
        .filter(|prefix| prefix.eq_ignore_ascii_case("0x"))
        .map(|_| &text[2..])?;
    if digits.len() != 16 || !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    let key = u64::from_str_radix(digits, 16).ok()?;
    (key > LOSSELESS_NUMBER_MAX).then_some(key)
}
