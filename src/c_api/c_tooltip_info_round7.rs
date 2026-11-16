//! Round 7 `C_TooltipInfo` getters. Native-miss paths stay empty until the
//! simulator has the state needed by the decoded producer.

use crate::c_api::item_spell::item_guid_for_bag_slot;
use crate::c_api::tooltip_getter_args;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::items::{self, ItemInfo};
use crate::lua_api::globals::missing_surface::tooltip_info::{
    build_item_interaction_cleansing_tooltip, build_quest_party_progress_tooltip,
    build_unresolved_conversion_item_tooltip, tooltip_for_item_context, tooltip_for_spell,
};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_set,
};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};
use std::collections::HashMap;

const LFG_REWARD_USAGE: &str =
    "Usage: local data = C_TooltipInfo.GetLFGDungeonReward(dungeonID, lootIndex)";
const LFG_SHORTAGE_USAGE: &str = concat!(
    "Usage: local data = C_TooltipInfo.GetLFGDungeonShortageReward(",
    "dungeonID, shortageSeverity, lootIndex)",
);
const ITEM_INTERACTION_CLEANSE_KIND: i32 = 2;
const ITEM_INTERACTION_CONVERSION_KIND: i32 = 4;
const POSSESSION_USAGE: &str = "Usage: local data = C_TooltipInfo.GetPossession(slot)";
const QUEST_SPECIAL_USAGE: &str =
    "Usage: local data = C_TooltipInfo.GetQuestLogSpecialItem(questIndex)";
const QUEST_PARTY_USAGE: &str = concat!(
    "Usage: local data = C_TooltipInfo.GetQuestPartyProgress(",
    "questID [, omitTitle, ignoreActivePlayer])",
);
const RECIPE_RANK_USAGE: &str =
    "Usage: local data = C_TooltipInfo.GetRecipeRankInfo(recipeID, rank)";
const RUNEFORGE_USAGE: &str = concat!(
    "Usage: local data = C_TooltipInfo.GetRuneforgeResultItem(",
    "itemGUID, itemLevel [, powerID, modifiers])",
);

/// One empty-by-default host input for selected item-interaction tooltip state.
/// Native SetPendingItem lifecycle and cleansing/conversion calculations are not modeled.
#[derive(Clone, Debug, Default)]
pub struct ItemInteractionTooltipInput {
    /// Selected object kind for the R88n.1 result-shape path.
    pub object_kind: Option<i32>,
    pub selected_item_guid: Option<String>,
    pub conversion_output_item_id: Option<u32>,
    pub cleansing_rows: Vec<ItemInteractionTooltipLine>,
}

/// Final cleansing row data supplied by the host; no row values or styles are inferred.
#[derive(Clone, Debug, Default)]
pub struct ItemInteractionTooltipLine {
    pub line_type: f64,
    pub left_text: String,
    pub left_color: Option<(f64, f64, f64)>,
    pub wrap: bool,
}

/// UNKNOWN (R88z): native keystone detail records and player-condition results
/// are not loaded here. The host must supply them; no values are seeded.
#[derive(Clone, Debug, Default)]
pub struct SlottedKeystoneTooltipInput {
    pub item_id: Option<u32>,
    pub item_guid: Option<String>,
    pub item_class_id: Option<u32>,
    pub item_subclass_id: Option<u32>,
    pub associated_player_guid: Option<String>,
    pub active_player_guid: Option<String>,
    pub display_season_id: Option<u32>,
    pub key_floors: Vec<SlottedKeystoneKeyFloor>,
    pub player_condition_results: HashMap<u32, bool>,
    /// `Some(None)` means the host confirms that the map record is missing.
    pub map_names: HashMap<u32, Option<String>>,
    /// Missing records and unavailable names do not produce affix-name rows.
    pub affix_names: HashMap<u32, Option<String>>,
}

#[derive(Clone, Debug, Default)]
pub struct SlottedKeystoneKeyFloor {
    pub display_season_id: u32,
    pub key_floor: u32,
    pub player_condition_id: u32,
}

pub(crate) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    let is_ptr = ACTIVE == ClientProfile::Ptr;
    crate::c_api::c_tooltip_info_azerite_essence::register_essence(state, namespace)?;
    crate::c_api::c_tooltip_info_azerite_essence::register_slot(state, namespace)?;
    if !is_ptr {
        table_set_rust_fn_static(
            state,
            namespace,
            "GetItemInteractionItem",
            get_item_interaction_item,
        )?;
        table_set_rust_fn_static(
            state,
            namespace,
            "GetLFGDungeonReward",
            get_lfg_dungeon_reward,
        )?;
        table_set_rust_fn_static(
            state,
            namespace,
            "GetLFGDungeonShortageReward",
            get_lfg_dungeon_shortage_reward,
        )?;
        table_set_rust_fn_static(state, namespace, "GetPossession", get_possession)?;
        table_set_rust_fn_static(
            state,
            namespace,
            "GetQuestLogSpecialItem",
            publish_quest_log_special_item,
        )?;
        table_set_rust_fn_static(
            state,
            namespace,
            "GetQuestPartyProgress",
            publish_quest_party_progress,
        )?;
        table_set_rust_fn_static(state, namespace, "GetRecipeRankInfo", get_recipe_rank_info)?;
        table_set_rust_fn_static(
            state,
            namespace,
            "GetRuneforgeResultItem",
            get_runeforge_result_item,
        )?;
        table_set_rust_fn_static(state, namespace, "GetSlottedKeystone", get_slotted_keystone)?;
    }
    Ok(())
}

fn get_item_interaction_item(state: &mut LuaState) -> LuaResult<u32> {
    // Only Retail's result paths are verified. Keep the other profiles' behavior.
    if ACTIVE != ClientProfile::Retail {
        return no_values();
    }
    let Some(input) = borrow_state(state)?.item_interaction_tooltip_input.clone() else {
        return no_values();
    };
    let Some(kind) = input.object_kind else {
        return no_values();
    };
    let Some(tooltip) = item_interaction_tooltip_for_kind(state, kind, &input)? else {
        return no_values();
    };
    state.push(tooltip);
    Ok(1)
}

fn item_interaction_tooltip_for_kind(
    state: &mut LuaState,
    kind: i32,
    input: &ItemInteractionTooltipInput,
) -> LuaResult<Option<Val>> {
    match kind {
        ITEM_INTERACTION_CLEANSE_KIND => cleansing_item_interaction_tooltip(state, input),
        ITEM_INTERACTION_CONVERSION_KIND => conversion_item_interaction_tooltip(state, input),
        _ => Ok(Some(empty_item_interaction_tooltip(state))),
    }
}

fn cleansing_item_interaction_tooltip(
    state: &mut LuaState,
    input: &ItemInteractionTooltipInput,
) -> LuaResult<Option<Val>> {
    if input.cleansing_rows.is_empty() || !item_interaction_source_is_owned(state, input) {
        return Ok(None);
    }
    Ok(Some(build_item_interaction_cleansing_tooltip(
        state,
        &input.cleansing_rows,
    )))
}

fn conversion_item_interaction_tooltip(
    state: &mut LuaState,
    input: &ItemInteractionTooltipInput,
) -> LuaResult<Option<Val>> {
    if !item_interaction_source_is_owned(state, input) {
        return Ok(None);
    }
    conversion_output_tooltip(state, input)
}

fn item_interaction_source_is_owned(state: &LuaState, input: &ItemInteractionTooltipInput) -> bool {
    input
        .selected_item_guid
        .as_deref()
        .is_some_and(|guid| item_for_owned_guid(state, guid).is_some())
}

fn conversion_output_tooltip(
    state: &mut LuaState,
    input: &ItemInteractionTooltipInput,
) -> LuaResult<Option<Val>> {
    let Some(output_item_id) = input.conversion_output_item_id else {
        return Ok(None);
    };
    if output_item_id == 0 {
        return Ok(Some(build_unresolved_conversion_item_tooltip(state)));
    }
    let Some(output_item) = borrow_state(state)?.item_info(output_item_id).cloned() else {
        return Ok(None);
    };
    Ok(Some(tooltip_for_item_context(
        state,
        output_item_id,
        Some(&output_item),
    )))
}

fn empty_item_interaction_tooltip(state: &mut LuaState) -> Val {
    let saved_top = state.top;
    let tooltip = create_table(state);
    state.push(tooltip);
    let lines = create_table(state);
    table_set(state, tooltip, "lines", lines);
    state.top = saved_top;
    tooltip
}

fn get_lfg_dungeon_reward(state: &mut LuaState) -> LuaResult<u32> {
    let dungeon_id = tooltip_getter_args::round7_signed(state, 1, LFG_REWARD_USAGE)?;
    let reward_index = tooltip_getter_args::round7_signed(state, 2, LFG_REWARD_USAGE)?;
    if crate::client_profile::ACTIVE != ClientProfile::Retail {
        return no_values();
    }
    let Some(reward) = lfg_tooltip_reward(state, dungeon_id, reward_index, None) else {
        return no_values();
    };
    publish_lfg_tooltip_reward(state, reward)
}

fn get_lfg_dungeon_shortage_reward(state: &mut LuaState) -> LuaResult<u32> {
    let dungeon_id = tooltip_getter_args::round7_signed(state, 1, LFG_SHORTAGE_USAGE)?;
    let severity = tooltip_getter_args::round7_signed(state, 2, LFG_SHORTAGE_USAGE)?;
    let reward_index = tooltip_getter_args::round7_signed(state, 3, LFG_SHORTAGE_USAGE)?;
    if crate::client_profile::ACTIVE != ClientProfile::Retail {
        return no_values();
    }
    let tier = severity.wrapping_sub(1) as u8;
    if tier > 2 {
        return no_values();
    }
    let Some(reward) = lfg_tooltip_reward(state, dungeon_id, reward_index, Some(tier)) else {
        return no_values();
    };
    publish_lfg_tooltip_reward(state, reward)
}

fn lfg_tooltip_reward(
    state: &LuaState,
    dungeon_id: i32,
    reward_index: i32,
    shortage_tier: Option<u8>,
) -> Option<crate::lua_api::state::LfgTooltipReward> {
    let index = usize::try_from(reward_index.wrapping_sub(1)).ok()?;
    let sim = crate::lua_api::methods::borrow_state(state).ok()?;
    if !sim.lfg_tooltip_rewards_ready {
        return None;
    }
    let dungeon = sim
        .lfd_dungeons
        .iter()
        .find(|dungeon| dungeon.dungeon_id == dungeon_id)?;
    let group = sim.lfg_tooltip_reward_groups.iter().find(|group| {
        group.dungeon_id == dungeon.dungeon_id && group.dungeon_type_id == dungeon.type_id
    })?;
    let rewards = match shortage_tier {
        Some(tier) => group.shortage_rewards.get(usize::from(tier))?,
        None => &group.rewards,
    };
    rewards.get(index).cloned()
}

fn publish_lfg_tooltip_reward(
    state: &mut LuaState,
    reward: crate::lua_api::state::LfgTooltipReward,
) -> LuaResult<u32> {
    let tooltip = if reward.is_currency {
        let Ok(currency_id) = i32::try_from(reward.reward_id) else {
            return no_values();
        };
        let Some(currency) =
            crate::lua_api::globals::currency_data::get_currency_by_id(currency_id)
        else {
            return no_values();
        };
        crate::lua_api::globals::missing_surface::tooltip_info::tooltip_for_currency(
            state,
            currency,
            Some(reward.quantity),
        )
    } else {
        // Native item-quantity effects are UNKNOWN; the shared sim item builder has no
        // quantity input.
        let Some(item) = crate::lua_api::methods::borrow_state(state)
            .ok()
            .and_then(|sim| sim.item_info(reward.reward_id).cloned())
        else {
            return no_values();
        };
        tooltip_for_item_context(state, reward.reward_id, Some(&item))
    };
    state.push(tooltip);
    Ok(1)
}

fn get_possession(state: &mut LuaState) -> LuaResult<u32> {
    let slot = tooltip_getter_args::round7_signed(state, 1, POSSESSION_USAGE)?;
    let Some(slot_index) = slot.checked_sub(1) else {
        return no_values();
    };
    if slot_index != 0 || ACTIVE != ClientProfile::Retail {
        return no_values();
    }
    let Some(spell_id) = borrow_state(state)?.possession_spell_id else {
        return no_values();
    };
    if crate::spells::get_spell(spell_id).is_none() {
        return no_values();
    }
    let tooltip = tooltip_for_spell(state, spell_id);
    state.push(tooltip);
    Ok(1)
}

fn publish_quest_log_special_item(state: &mut LuaState) -> LuaResult<u32> {
    // Retail TooltipInfoDocumentation.lua:817; host inputs separate quest-ID
    // context, cache IDs and the selected object because native writers are unknown.
    let quest_index = tooltip_getter_args::round7_signed(state, 1, QUEST_SPECIAL_USAGE)?;
    if ACTIVE != ClientProfile::Retail {
        return no_values();
    }
    let Some((quest_id, is_not_ready)) = quest_cache_lookup_for_index(state, quest_index) else {
        return no_values();
    };
    if is_not_ready {
        borrow_state_mut(state)?
            .quest_log_entries
            .quest_special_item_inputs
            .save_quest_record_callback(quest_id);
        // INFERRED (R88u.u3/u4): no TooltipData is returned before record readiness.
        return no_values();
    }
    publish_ready_quest_special_item(state, quest_id)
}

fn publish_ready_quest_special_item(state: &mut LuaState, quest_id: i32) -> LuaResult<u32> {
    let Some(selected_object) = find_quest_special_item_object(state, quest_id) else {
        return no_values();
    };
    let guid = item_guid_for_bag_slot(
        selected_object.bag,
        selected_object.slot,
        selected_object.item_id,
    );
    let Some((item_id, item)) = item_for_owned_guid(state, &guid) else {
        return no_values();
    };
    let data = tooltip_for_item_context(state, item_id, Some(item));
    let guid = create_string(state, &guid);
    table_set(state, data, "guid", guid);
    state.push(data);
    Ok(1)
}

fn publish_quest_party_progress(state: &mut LuaState) -> LuaResult<u32> {
    // Retail TooltipInfoDocumentation.lua:833; only the Retail 70688
    // populated output is evidenced for this implementation.
    let quest_id = tooltip_getter_args::round7_signed(state, 1, QUEST_PARTY_USAGE)?;
    let omit_title = tooltip_getter_args::round7_optional_bool(state, 2, QUEST_PARTY_USAGE)?;
    let ignore_active_player =
        tooltip_getter_args::round7_optional_bool(state, 3, QUEST_PARTY_USAGE)?;
    if ACTIVE != ClientProfile::Retail {
        return no_values();
    }
    let snapshot = borrow_state(state)?
        .quest_log_entries
        .party_progress
        .get(&quest_id)
        .cloned();
    let Some(snapshot) = snapshot else {
        return no_values();
    };
    let Some(data) = build_quest_party_progress_tooltip(
        state,
        quest_id,
        omit_title,
        ignore_active_player,
        &snapshot,
    )?
    else {
        return no_values();
    };
    state.push(data);
    Ok(1)
}

fn get_recipe_rank_info(state: &mut LuaState) -> LuaResult<u32> {
    let recipe_id = tooltip_getter_args::round7_signed(state, 1, RECIPE_RANK_USAGE)?;
    let rank = tooltip_getter_args::round7_signed(state, 2, RECIPE_RANK_USAGE)?;
    if crate::client_profile::ACTIVE != ClientProfile::Retail {
        return no_values();
    }
    let Some(data) =
        crate::lua_api::globals::missing_surface::tooltip_info::build_recipe_rank_info(
            state, recipe_id, rank,
        )?
    else {
        return no_values();
    };
    state.push(data);
    Ok(1)
}

fn get_runeforge_result_item(state: &mut LuaState) -> LuaResult<u32> {
    let guid = tooltip_getter_args::round7_guid(state, 1, RUNEFORGE_USAGE)?;
    let level = tooltip_getter_args::round7_signed(state, 2, RUNEFORGE_USAGE)?;
    if has_unsupported_runeforge_customization(state)? {
        return no_values();
    }
    let Some((item_id, item)) = item_for_owned_guid(state, &guid) else {
        return no_values();
    };
    let Some(data) = build_runeforge_result_data(state, &guid, item_id, item, level) else {
        return no_values();
    };
    state.push(data);
    Ok(1)
}

fn has_unsupported_runeforge_customization(state: &mut LuaState) -> LuaResult<bool> {
    let power_id = match stack_val(state, 3) {
        Val::Nil => None,
        _ => Some(tooltip_getter_args::round7_signed(
            state,
            3,
            RUNEFORGE_USAGE,
        )?),
    };
    let modifiers = match tooltip_getter_args::allowed_when_untainted(
        state,
        4,
        "C_TooltipInfo.GetRuneforgeResultItem",
    )? {
        Val::Nil => false,
        Val::Table(_) => true,
        _ => return Err(rilua::runtime_error(RUNEFORGE_USAGE)),
    };
    Ok(power_id.is_some() || modifiers)
}

fn build_runeforge_result_data(
    state: &mut LuaState,
    guid: &str,
    item_id: u32,
    item: &ItemInfo,
    level: i32,
) -> Option<Val> {
    let mut displayed_item = item.clone();
    if level != 0 {
        // The shared builder stores a u16 item level. Do not invent conversion
        // behavior for native int32 values that this representation cannot hold.
        displayed_item.item_level = u16::try_from(level).ok()?;
    }
    let data = tooltip_for_item_context(state, item_id, Some(&displayed_item));
    let guid = create_string(state, guid);
    table_set(state, data, "guid", guid);
    if level != 0 {
        table_set(state, data, "overrideItemLevel", Val::Num(f64::from(level)));
    }
    Some(data)
}

fn get_slotted_keystone(state: &mut LuaState) -> LuaResult<u32> {
    if ACTIVE != ClientProfile::Retail {
        return no_values();
    }
    let (guid, selected_item_id) = {
        let sim = borrow_state(state)?;
        let Some(guid) = sim.keystone.selected_guid.clone() else {
            return no_values();
        };
        (guid, sim.keystone.selected_item_id)
    };
    let Some((item_id, item)) = selected_keystone_item(state, &guid, selected_item_id) else {
        return no_values();
    };
    let tooltip = tooltip_for_item_context(state, item_id, Some(item));
    let guid = create_string(state, &guid);
    table_set(state, tooltip, "guid", guid);
    state.push(tooltip);
    Ok(1)
}

fn selected_keystone_item(
    state: &LuaState,
    guid: &str,
    selected_item_id: Option<u32>,
) -> Option<(u32, &'static ItemInfo)> {
    let Some(item_id) = selected_item_id else {
        return item_for_owned_guid(state, guid);
    };
    let (_, _, guid_item_id) = crate::c_api::item_spell::parse_item_guid(guid)?;
    if guid_item_id != item_id {
        return None;
    }
    items::get_item(item_id).map(|item| (item_id, item))
}

fn quest_cache_lookup_for_index(state: &LuaState, quest_index: i32) -> Option<(i32, bool)> {
    let sim = borrow_state(state).ok()?;
    let inputs = &sim.quest_log_entries.quest_special_item_inputs;
    let quest_id = *inputs.quest_ids_by_log_index.get(&quest_index)?;
    let record_exists = inputs.cached_quest_records.contains_key(&quest_id);
    let not_ready = record_exists && inputs.not_ready_cache_records.contains(&quest_id);
    Some((quest_id, not_ready))
}

fn find_quest_special_item_object(
    state: &LuaState,
    quest_id: i32,
) -> Option<crate::c_api::quest_special_item::QuestSpecialItemObject> {
    let sim = borrow_state(state).ok()?;
    let inputs = &sim.quest_log_entries.quest_special_item_inputs;
    let cache_record = inputs.cached_quest_records.get(&quest_id)?;
    let selected_object = *inputs.selected_item_objects.get(&quest_id)?;
    let primary_id_matches = cache_record.special_item_id == Some(selected_object.item_id)
        && selected_object.item_id != 0;
    let candidate_id_matches = cache_record.has_candidate_items
        && cache_record
            .candidate_item_ids
            .contains(&Some(selected_object.item_id));
    (primary_id_matches || candidate_id_matches).then_some(selected_object)
}

fn item_for_owned_guid(state: &LuaState, guid: &str) -> Option<(u32, &'static ItemInfo)> {
    let (bag, slot, item_id) = crate::c_api::item_spell::parse_item_guid(guid)?;
    let sim = borrow_state(state).ok()?;
    let owned = sim.bag_items.get(&(bag, slot))?;
    if owned.item_id != item_id
        || crate::c_api::item_spell::item_guid_for_bag_slot(bag, slot, item_id) != guid
    {
        return None;
    }
    Some((item_id, items::get_item(item_id)?))
}

fn no_values() -> LuaResult<u32> {
    crate::c_api::tooltip_data::unknown_getter_failure()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lua_api::state_types::{AzeriteEssenceInfo, AzeriteEssenceMilestoneInfo};

    #[test]
    fn populated_azerite_selection_misses_without_rank_spell_data() {
        if !matches!(
            ACTIVE,
            ClientProfile::Retail | ClientProfile::Ptr | ClientProfile::WowForever
        ) {
            return;
        }
        let env = crate::lua_api::WowLuaEnv::new().unwrap();
        {
            let mut sim = env.state().borrow_mut();
            sim.azerite_essence.essences.insert(
                241,
                AzeriteEssenceInfo {
                    id: 241,
                    name: "Round Seven Essence".to_string(),
                    rank: 3,
                    icon: 1,
                    unlocked: true,
                    valid: true,
                    access_rank: 3,
                    has_never_activated: false,
                },
            );
            sim.azerite_essence
                .milestones
                .push(AzeriteEssenceMilestoneInfo {
                    id: 1,
                    required_level: 1,
                    slot: Some(0),
                    unlocked: true,
                    can_unlock: false,
                    is_major_slot: true,
                    swirl_scale: 1.0,
                    requires_only_aura: false,
                    spell_id: 1,
                    rank: None,
                    active_essence_id: Some(241),
                });
        }
        env.exec(
            r#"
                assert(C_AzeriteEssence.GetEssenceInfo(241).rank == 3)
                assert(C_AzeriteEssence.GetMilestoneEssence(1) == 241)
                assert(select('#', C_TooltipInfo.GetAzeriteEssence(241)) == 0)
                assert(select('#', C_TooltipInfo.GetAzeriteEssenceSlot(0)) == 0)
            "#,
        )
        .unwrap();
    }
}
