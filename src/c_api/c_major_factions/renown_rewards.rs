//! Pair-keyed renown reward snapshots; the owning SimState map starts empty.
//! Missing pairs return an empty sequence (inferred); no faction eligibility check.

use crate::lua_api::methods::{
    borrow_state, create_string, create_table, create_table_with_fields, table_set_num,
    table_set_static,
};
use crate::lua_bridge::stack_val;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

/// Input row matching cached `MajorFactionRenownRewardInfo` declarations.
/// Optional booleans preserve absent, false, and true as distinct values.
#[derive(Clone, Debug, PartialEq)]
pub struct RenownRewardInfo {
    pub renown_reward_id: i64,
    pub ui_order: i32,
    pub is_account_unlock: bool,
    pub item_id: Option<i64>,
    pub spell_id: Option<i64>,
    pub mount_id: Option<i64>,
    pub transmog_id: Option<i64>,
    pub transmog_set_id: Option<i64>,
    pub title_mask_id: Option<i64>,
    pub transmog_illusion_source_id: Option<i64>,
    pub icon: Option<i64>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub toast_description: Option<String>,
    /// Cached primary documentation declares `number`, not an enum type.
    pub reward_type: Option<i32>,
    pub is_collected: Option<bool>,
}

pub(super) fn get_renown_rewards_for_level(state: &mut LuaState) -> LuaResult<u32> {
    let faction_id = read_selector(state, 1)?;
    let level = read_selector(state, 2)?;
    let key = (faction_id as i64, level as i32);
    // Stored selectors are integers; fractional numbers must not select a nearby pair.
    let rewards = if faction_id == key.0 as f64 && level == key.1 as f64 {
        borrow_state(state)?
            .major_faction_renown_rewards
            .get(&key)
            .cloned()
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let sequence = create_table(state);
    let Val::Table(sequence_ref) = sequence else {
        unreachable!("create_table must return a table");
    };
    for (index, reward) in rewards.iter().enumerate() {
        let row = build_reward_table(state, reward);
        table_set_num(state, sequence_ref, (index + 1) as f64, row);
    }
    state.push(sequence);
    Ok(1)
}

fn read_selector(state: &LuaState, index: i32) -> LuaResult<f64> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, index))?;
    let Val::Num(number) = value else {
        return Err(rilua::runtime_error(format!(
            "GetRenownRewardsForLevel argument #{index} must be a number"
        )));
    };
    Ok(number)
}

fn build_reward_table(state: &mut LuaState, reward: &RenownRewardInfo) -> Val {
    let row = create_table_with_fields(
        state,
        &[
            ("renownRewardID", Val::Num(reward.renown_reward_id as f64)),
            ("uiOrder", Val::Num(reward.ui_order as f64)),
            ("isAccountUnlock", Val::Bool(reward.is_account_unlock)),
        ],
    );
    set_reward_numbers(state, row, reward);
    set_reward_strings(state, row, reward);
    if cfg!(all(
        feature = "retail-12-0-5",
        any(feature = "profile-retail", feature = "client-ptr")
    )) && let Some(collected) = reward.is_collected
    {
        table_set_static(state, row, "isCollected", Val::Bool(collected));
    }
    row
}

fn set_reward_numbers(state: &mut LuaState, row: Val, reward: &RenownRewardInfo) {
    for (name, value) in [
        ("itemID", reward.item_id),
        ("spellID", reward.spell_id),
        ("mountID", reward.mount_id),
        ("transmogID", reward.transmog_id),
        ("transmogSetID", reward.transmog_set_id),
        ("titleMaskID", reward.title_mask_id),
        (
            "transmogIllusionSourceID",
            reward.transmog_illusion_source_id,
        ),
        ("icon", reward.icon),
        ("rewardType", reward.reward_type.map(i64::from)),
    ] {
        if let Some(value) = value {
            table_set_static(state, row, name, Val::Num(value as f64));
        }
    }
}

fn set_reward_strings(state: &mut LuaState, row: Val, reward: &RenownRewardInfo) {
    for (name, value) in [
        ("name", reward.name.as_deref()),
        ("description", reward.description.as_deref()),
        ("toastDescription", reward.toast_description.as_deref()),
    ] {
        if let Some(value) = value {
            let text = create_string(state, value);
            table_set_static(state, row, name, text);
        }
    }
}
