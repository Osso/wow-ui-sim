//! Round 7 `C_TooltipInfo` getters. Native-miss paths stay empty until the
//! simulator has the state needed by the decoded producer.

use crate::c_api::tooltip_getter_args;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::items::{self, ItemInfo};
use crate::lua_api::globals::missing_surface::tooltip_info::tooltip_for_item_context;
use crate::lua_api::methods::{borrow_state, create_string, table_set};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};

const ESSENCE_USAGE: &str =
    "Usage: local data = C_TooltipInfo.GetAzeriteEssence(essenceID [, rank])";
const ESSENCE_SLOT_USAGE: &str = "Usage: local data = C_TooltipInfo.GetAzeriteEssenceSlot(slot)";
const LFG_REWARD_USAGE: &str =
    "Usage: local data = C_TooltipInfo.GetLFGDungeonReward(dungeonID, lootIndex)";
const LFG_SHORTAGE_USAGE: &str = "Usage: local data = C_TooltipInfo.GetLFGDungeonShortageReward(dungeonID, shortageSeverity, lootIndex)";
const POSSESSION_USAGE: &str = "Usage: local data = C_TooltipInfo.GetPossession(slot)";
const QUEST_SPECIAL_USAGE: &str =
    "Usage: local data = C_TooltipInfo.GetQuestLogSpecialItem(questIndex)";
const QUEST_PARTY_USAGE: &str = "Usage: local data = C_TooltipInfo.GetQuestPartyProgress(questID [, omitTitle, ignoreActivePlayer])";
const RECIPE_RANK_USAGE: &str =
    "Usage: local data = C_TooltipInfo.GetRecipeRankInfo(recipeID, rank)";
const RUNEFORGE_USAGE: &str = "Usage: local data = C_TooltipInfo.GetRuneforgeResultItem(itemGUID, itemLevel [, powerID, modifiers])";

pub(crate) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    let is_ptr = ACTIVE == ClientProfile::Ptr;
    table_set_rust_fn_static(state, namespace, "GetAzeriteEssence", get_azerite_essence)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetAzeriteEssenceSlot",
        get_azerite_essence_slot,
    )?;
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
            get_quest_log_special_item,
        )?;
        table_set_rust_fn_static(
            state,
            namespace,
            "GetQuestPartyProgress",
            get_quest_party_progress,
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

fn get_azerite_essence(state: &mut LuaState) -> LuaResult<u32> {
    let essence_id = tooltip_getter_args::round7_signed(state, 1, ESSENCE_USAGE)?;
    let requested_rank = tooltip_getter_args::round7_optional_unsigned(state, 2, ESSENCE_USAGE)?;
    let selected_rank = requested_rank.or_else(|| current_essence_rank(state, essence_id));
    if selected_rank.is_none() {
        return no_values();
    }
    // The state supplies the selected rank, but the rank-to-spell tooltip
    // catalogue and spell-block producer are not modelled.
    no_values()
}

fn get_azerite_essence_slot(state: &mut LuaState) -> LuaResult<u32> {
    let slot = tooltip_getter_args::round7_signed(state, 1, ESSENCE_SLOT_USAGE)? as u8;
    if slot > 3 {
        return Err(rilua::runtime_error(format!(
            "outside of expected range 0 to 3 - {ESSENCE_SLOT_USAGE}"
        )));
    }
    let Some(essence_id) = active_essence_for_slot(state, i32::from(slot)) else {
        return no_values();
    };
    if current_essence_rank(state, essence_id).is_none() {
        return no_values();
    }
    // The state supplies the selected slot and rank, but not the native
    // rank-to-spell tooltip catalogue or producer.
    no_values()
}

fn get_item_interaction_item(_state: &mut LuaState) -> LuaResult<u32> {
    // Current interaction state contains dialog text and cost, not the selected
    // object descriptor or the kind-2/kind-4 payload inputs.
    no_values()
}

fn get_lfg_dungeon_reward(state: &mut LuaState) -> LuaResult<u32> {
    let _dungeon_id = tooltip_getter_args::round7_signed(state, 1, LFG_REWARD_USAGE)?;
    let _reward_index = tooltip_getter_args::round7_signed(state, 2, LFG_REWARD_USAGE)?;
    no_values()
}

fn get_lfg_dungeon_shortage_reward(state: &mut LuaState) -> LuaResult<u32> {
    let _dungeon_id = tooltip_getter_args::round7_signed(state, 1, LFG_SHORTAGE_USAGE)?;
    let _severity = tooltip_getter_args::round7_signed(state, 2, LFG_SHORTAGE_USAGE)? as u8;
    let _reward_index = tooltip_getter_args::round7_signed(state, 3, LFG_SHORTAGE_USAGE)?;
    no_values()
}

fn get_possession(state: &mut LuaState) -> LuaResult<u32> {
    let slot = tooltip_getter_args::round7_signed(state, 1, POSSESSION_USAGE)?;
    if slot != 0 {
        return no_values();
    }
    // The simulator has no possession-spell/context state.
    no_values()
}

fn get_quest_log_special_item(state: &mut LuaState) -> LuaResult<u32> {
    let _quest_index = tooltip_getter_args::round7_signed(state, 1, QUEST_SPECIAL_USAGE)?;
    // Quest-log special items require the native player's item-object relation;
    // quest_log_entries alone is not an equivalent selector.
    no_values()
}

fn get_quest_party_progress(state: &mut LuaState) -> LuaResult<u32> {
    let _quest_id = tooltip_getter_args::round7_signed(state, 1, QUEST_PARTY_USAGE)?;
    let _omit_title = tooltip_getter_args::round7_optional_bool(state, 2, QUEST_PARTY_USAGE)?;
    let _ignore_active_player =
        tooltip_getter_args::round7_optional_bool(state, 3, QUEST_PARTY_USAGE)?;
    // There is no quest-progress service or party objective snapshot model.
    no_values()
}

fn get_recipe_rank_info(state: &mut LuaState) -> LuaResult<u32> {
    let _recipe_id = tooltip_getter_args::round7_signed(state, 1, RECIPE_RANK_USAGE)?;
    let _rank = tooltip_getter_args::round7_signed(state, 2, RECIPE_RANK_USAGE)?;
    // RecipeEntry is a crafting-panel output record, not the native parent / rank chain.
    no_values()
}

fn get_runeforge_result_item(state: &mut LuaState) -> LuaResult<u32> {
    let guid = tooltip_getter_args::round7_guid(state, 1, RUNEFORGE_USAGE)?;
    let level = tooltip_getter_args::round7_signed(state, 2, RUNEFORGE_USAGE)?;
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
    // The selected owned instance is modelled. Power and modifier catalogues
    // are not, so do not turn their identifiers into guessed bonus data.
    if power_id.is_some() || modifiers {
        return no_values();
    }
    let Some((item_id, item)) = item_for_owned_guid(state, &guid) else {
        return no_values();
    };
    let mut displayed_item = item.clone();
    if level != 0 {
        // The shared builder stores a u16 item level. Do not invent conversion
        // behavior for native int32 values that this representation cannot hold.
        let Ok(item_level) = u16::try_from(level) else {
            return no_values();
        };
        displayed_item.item_level = item_level;
    }
    let data = tooltip_for_item_context(state, item_id, Some(&displayed_item));
    let guid = create_string(state, &guid);
    table_set(state, data, "guid", guid);
    if level != 0 {
        table_set(state, data, "overrideItemLevel", Val::Num(f64::from(level)));
    }
    state.push(data);
    Ok(1)
}

fn get_slotted_keystone(_state: &mut LuaState) -> LuaResult<u32> {
    // The slotted GUID is not represented in simulator state.
    no_values()
}

fn current_essence_rank(state: &LuaState, essence_id: i32) -> Option<u32> {
    borrow_state(state)
        .ok()?
        .azerite_essence
        .essences
        .get(&essence_id)
        .and_then(|essence| u32::try_from(essence.rank).ok())
}

fn active_essence_for_slot(state: &LuaState, slot: i32) -> Option<i32> {
    borrow_state(state)
        .ok()?
        .azerite_essence
        .milestones
        .iter()
        .find(|milestone| milestone.slot == Some(slot))
        .and_then(|milestone| milestone.active_essence_id)
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
