//! `C_EncounterJournal` namespace and `EJ_*` legacy globals — the data
//! surface behind the Adventure Guide. Backed by static tables in
//! `crates/sim-data/data/encounter_journal.rs` (regenerated via `wow-cli generate
//! encounter-journal`) and per-session state in `SimState.encounter_journal`.
//!
//! Schema parity:
//!
//! - `C_EncounterJournal.GetEncounterInfo` — 8-tuple matching the official
//!   retail return: `(name, description, encounterID, rootSectionID,
//!   linkSection, journalInstanceID, dungeonEncounterID, instanceID)`.
//! - `C_EncounterJournal.GetInstanceInfo` — 9-tuple: `(name, description,
//!   bgImage, buttonImage1, loreImage, buttonImage2, dungeonAreaMapID,
//!   linkRaidID, linkDungeonID)`. Image fields are returned as fileDataIDs
//!   (numbers) per the real DBC schema.
//! - `C_EncounterJournal.GetSectionInfo` — table per
//!   `vendor/wow-ui-source/.../EncounterJournalDocumentation` with at least
//!   `spellID`, `headerType`, `description`, `title`, `siblingSectionID`,
//!   `firstChildSectionID`.
//! - `C_EncounterJournal.GetLootInfo` / `GetLootInfoByIndex` — table with
//!   `name`, `icon`, `itemQuality`, `itemID`, `link`, `slot`, `armorType`,
//!   `encounterID`, `displayAsPerPlayerLoot` (always false here).
//!
//! `EJ_*` legacy globals route through the same data, with state held
//! in `SimState.encounter_journal` rather than Lua globals.

use super::ensure_namespace;
use crate::encounter_journal_data as data;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_set, table_set_num,
};
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use crate::spells;
use rilua::vm::closure::RustFn;
#[cfg(feature = "retail-12-1-0")]
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
#[cfg(feature = "retail-12-1-0")]
use rilua::vm::table::Table;
use rilua::{LuaApiMut, LuaResult, Val};

mod adventure;
mod loot;
mod search;

const ENCOUNTER_JOURNAL_ICON_FLAG_COUNT: u32 = 14;
const FALLBACK_ABILITY_ICON: u32 = 136243;

const ENCOUNTER_JOURNAL_FUNCTIONS: &[(&str, RustFn)] = &[
    ("GetEncounterInfo", get_encounter_info),
    ("GetInstanceInfo", get_instance_info),
    ("GetInstanceForGameMap", get_instance_for_game_map),
    ("GetSectionInfo", get_section_info),
    ("GetLootInfo", loot::get_loot_info),
    ("GetLootInfoByIndex", loot::get_loot_info_by_index),
    ("GetSectionIconFlags", get_section_icon_flags),
    ("InstanceHasLoot", loot::instance_has_loot),
    ("GetSlotFilter", get_slot_filter),
    ("SetSlotFilter", set_slot_filter),
    ("ResetSlotFilter", reset_slot_filter),
    ("SetTab", set_tab),
    ("OnOpen", noop),
    ("OnClose", noop),
    ("InitalizeSelectedTier", initialize_selected_tier),
    ("StartArathiRPE", noop),
];

const EJ_GLOBAL_FUNCTIONS: &[(&str, RustFn)] = &[
    ("EJ_GetNumTiers", ej_get_num_tiers),
    ("EJ_GetTierInfo", ej_get_tier_info),
    ("EJ_GetCurrentTier", ej_get_current_tier),
    ("EJ_SelectTier", ej_select_tier),
    ("EJ_GetInstanceInfo", ej_get_instance_info),
    ("EJ_GetInstanceByIndex", ej_get_instance_by_index),
    ("EJ_GetEncounterInfo", ej_get_encounter_info),
    ("EJ_GetEncounterInfoByIndex", ej_get_encounter_info_by_index),
    ("EJ_GetCreatureInfo", ej_get_creature_info),
    ("EJ_SelectInstance", ej_select_instance),
    ("EJ_SelectEncounter", ej_select_encounter),
    ("EJ_GetSelectedInstance", ej_get_selected_instance),
    ("EJ_GetSelectedEncounter", ej_get_selected_encounter),
    ("EJ_GetDifficulty", ej_get_difficulty),
    ("EJ_SetDifficulty", ej_set_difficulty),
    ("EJ_InstanceIsRaid", ej_instance_is_raid),
    (
        "EJ_IsValidInstanceDifficulty",
        ej_is_valid_instance_difficulty,
    ),
    ("EJ_GetNumLoot", loot::ej_get_num_loot),
    (
        "EJ_GetLootInfoByIndex",
        loot::ej_get_loot_info_by_index_global,
    ),
    ("EJ_GetLootFilter", loot::ej_get_loot_filter),
    ("EJ_SetLootFilter", loot::ej_set_loot_filter),
    ("EJ_ResetLootFilter", loot::ej_reset_loot_filter),
    ("EJ_GetInvTypeSortOrder", loot::ej_get_inv_type_sort_order),
    (
        "EJ_GetNumEncountersForLootByIndex",
        loot::ej_get_num_encounters_for_loot_by_index,
    ),
    ("EJ_IsLootListOutOfDate", loot::ej_is_loot_list_out_of_date),
    ("EJ_GetSectionPath", ej_get_section_path),
    ("EJ_GetContentTuningID", ej_get_content_tuning_id),
    ("EJ_SetSearch", search::ej_set_search),
    ("EJ_ClearSearch", search::ej_clear_search),
    ("EJ_EndSearch", search::ej_end_search),
    ("EJ_GetSearchSize", search::ej_get_search_size),
    ("EJ_GetSearchProgress", search::ej_get_search_progress),
    ("EJ_GetNumSearchResults", search::ej_get_num_search_results),
    ("EJ_GetSearchResult", search::ej_get_search_result),
    ("EJ_IsSearchFinished", search::ej_is_search_finished),
    ("EJ_HandleLinkPath", ej_handle_link_path),
    ("EJ_HideLootJournalPanel", noop_global),
    ("EJ_HideNonInstancePanels", noop_global),
    ("EJ_HideSuggestPanel", noop_global),
    ("EJ_HideTutorialsPanel", noop_global),
];

// ---------------------------------------------------------------------------
// Registration
// ---------------------------------------------------------------------------

pub(super) fn register_encounter_journal_surface(state: &mut LuaState) -> LuaResult<()> {
    register_encounter_journal_functions(state)?;
    adventure::register_adventure_journal_surface(state)?;
    Ok(())
}

fn register_encounter_journal_functions(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_EncounterJournal")?;
    for &(name, handler) in ENCOUNTER_JOURNAL_FUNCTIONS {
        table_set_rust_fn_static(state, table_ref, name, handler)?;
    }
    #[cfg(feature = "retail-12-1-0")]
    register_patch_12_1_encounter_journal_functions(state, table_ref)?;
    Ok(())
}

#[cfg(feature = "retail-12-1-0")]
fn register_patch_12_1_encounter_journal_functions(
    state: &mut LuaState,
    table_ref: GcRef<Table>,
) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetBaseDifficultyID",
        get_base_difficulty_id,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "InstanceHasDifficultyID",
        instance_has_difficulty_id,
    )
}

pub(crate) fn register_ej_globals(lua: &mut rilua::Lua) -> LuaResult<()> {
    for &(name, handler) in EJ_GLOBAL_FUNCTIONS {
        LuaApiMut::register_function(lua, name, handler)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// C_EncounterJournal handlers
// ---------------------------------------------------------------------------

fn get_encounter_info(state: &mut LuaState) -> LuaResult<u32> {
    let encounter_id = u32::from_stack(state, 1)?;
    let Some(row) = data::encounter_by_id(encounter_id) else {
        return Ok(0);
    };
    push_encounter_tuple(state, row);
    Ok(8)
}

fn get_instance_info(state: &mut LuaState) -> LuaResult<u32> {
    let instance_id = u32::from_stack(state, 1)?;
    let Some(row) = data::instance_by_id(instance_id) else {
        return Ok(0);
    };
    push_instance_tuple(state, row);
    Ok(9)
}

#[cfg(feature = "retail-12-1-0")]
fn get_base_difficulty_id(state: &mut LuaState) -> LuaResult<u32> {
    let instance_id = u32::from_stack(state, 1)?;
    let Some(row) = data::instance_by_id(instance_id) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let difficulty_id = base_difficulty_id(row);
    state.push(Val::Num(difficulty_id as f64));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn instance_has_difficulty_id(state: &mut LuaState) -> LuaResult<u32> {
    let instance_id = u32::from_stack(state, 1)?;
    let difficulty_id = u32::from_stack(state, 2).unwrap_or(0);
    let has_difficulty = data::instance_by_id(instance_id)
        .map(|row| instance_difficulties(row).contains(&difficulty_id))
        .unwrap_or(false);
    state.push(Val::Bool(has_difficulty));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn base_difficulty_id(row: &data::Instance) -> u32 {
    if row.is_raid { 14 } else { 1 }
}

#[cfg(feature = "retail-12-1-0")]
fn instance_difficulties(row: &data::Instance) -> &'static [u32] {
    if row.is_raid {
        &[14, 15, 16, 17]
    } else {
        &[1, 2, 8, 23]
    }
}

fn get_section_info(state: &mut LuaState) -> LuaResult<u32> {
    let section_id = u32::from_stack(state, 1)?;
    let Some(row) = data::section_by_id(section_id) else {
        return Ok(0);
    };
    let table = build_section_info_table(state, row);
    state.push(table);
    Ok(1)
}

fn build_section_info_table(state: &mut LuaState, row: &data::Section) -> Val {
    let table = create_table(state);
    set_section_text_fields(state, table, row);
    set_section_relationship_fields(state, table, row);
    set_section_visual_fields(state, table, row);
    table_set(state, table, "filteredByDifficulty", Val::Bool(false));
    table_set(state, table, "startsOpen", Val::Bool(false));
    table
}

fn set_section_text_fields(state: &mut LuaState, table: Val, row: &data::Section) {
    let title = create_string(state, row.title);
    let body = create_string(state, row.body);
    let empty_link = create_string(state, "");

    table_set(state, table, "spellID", Val::Num(row.spell_id as f64));
    table_set(state, table, "headerType", Val::Num(row.kind as f64));
    table_set(state, table, "title", title);
    table_set(state, table, "description", body);
    table_set(state, table, "link", empty_link);
}

fn set_section_relationship_fields(state: &mut LuaState, table: Val, row: &data::Section) {
    table_set(
        state,
        table,
        "siblingSectionID",
        Val::Num(row.next_sibling_id as f64),
    );
    table_set(
        state,
        table,
        "firstChildSectionID",
        Val::Num(row.first_child_id as f64),
    );
    table_set(
        state,
        table,
        "parentSectionID",
        Val::Num(row.parent_id as f64),
    );
}

fn set_section_visual_fields(state: &mut LuaState, table: Val, row: &data::Section) {
    table_set(
        state,
        table,
        "creatureDisplayID",
        Val::Num(row.icon_creature_display_id as f64),
    );
    table_set(
        state,
        table,
        "uiModelSceneID",
        Val::Num(row.model_scene_id as f64),
    );
    if let Some(ability_icon) = ability_icon_for_section(row) {
        table_set(state, table, "abilityIcon", Val::Num(ability_icon as f64));
    }
}

fn ability_icon_for_section(row: &data::Section) -> Option<u32> {
    if row.icon_file_id != 0 {
        return Some(row.icon_file_id);
    }

    if row.spell_id == 0 {
        return None;
    }

    Some(
        spells::get_spell(row.spell_id)
            .map(|spell| spell.icon_file_data_id)
            .unwrap_or(FALLBACK_ABILITY_ICON),
    )
}

fn get_section_icon_flags(state: &mut LuaState) -> LuaResult<u32> {
    let section_id = u32::from_stack(state, 1)?;
    let Some(section) = data::section_by_id(section_id) else {
        return Ok(0);
    };
    let Some(icon_indices) = section_icon_indices_table(state, section.icon_flags) else {
        return Ok(0);
    };
    state.push(icon_indices);
    Ok(1)
}

fn section_icon_indices_table(state: &mut LuaState, icon_flags: u32) -> Option<Val> {
    if icon_flags == 0 {
        return None;
    }

    let table = create_table(state);
    let Val::Table(table_ref) = table else {
        return None;
    };

    let mut lua_index = 1.0;
    for icon_index in 0..ENCOUNTER_JOURNAL_ICON_FLAG_COUNT {
        let flag = 1_u32 << icon_index;
        if icon_flags & flag != 0 {
            table_set_num(state, table_ref, lua_index, Val::Num(icon_index as f64));
            lua_index += 1.0;
        }
    }

    Some(table)
}

fn get_slot_filter(state: &mut LuaState) -> LuaResult<u32> {
    let slot = borrow_state(state)?.encounter_journal.slot_filter;
    state.push(Val::Num(slot as f64));
    Ok(1)
}

fn set_slot_filter(state: &mut LuaState) -> LuaResult<u32> {
    let slot = i32::from_stack(state, 1).unwrap_or(-1);
    borrow_state_mut(state)?.encounter_journal.slot_filter = slot;
    Ok(0)
}

fn reset_slot_filter(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.encounter_journal.slot_filter = -1;
    Ok(0)
}

/// Best guess (native UNKNOWN): several journal instances can share one game
/// map (Dire Maul wings, Stratholme gates); the lowest journal instance ID
/// wins. Rows with `map_id` 0 carry no game map and never match.
fn get_instance_for_game_map(state: &mut LuaState) -> LuaResult<u32> {
    let map_id = u32::from_stack(state, 1)?;
    let instance_id = data::INSTANCES
        .iter()
        .filter(|row| map_id != 0 && row.map_id == map_id)
        .map(|row| row.id)
        .min();
    state.push(instance_id.map_or(Val::Nil, |id| Val::Num(id as f64)));
    Ok(1)
}

fn set_tab(state: &mut LuaState) -> LuaResult<u32> {
    let tab = u32::from_stack(state, 1).unwrap_or(0);
    borrow_state_mut(state)?.encounter_journal.current_tab = tab;
    Ok(0)
}

fn initialize_selected_tier(state: &mut LuaState) -> LuaResult<u32> {
    let mut sim = borrow_state_mut(state)?;
    // InitalizeSelectedTier resets to the Dragonflight tier -- the first expansion
    // with Journeys content (EJ_JOURNEYS_MIN_TIER in the Lua layer).  Tier 10 is
    // Dragonflight (expansion 1000).  If absent, fall back to the smallest tier.
    let df_order = data::TIERS
        .iter()
        .find(|t| t.expansion == 1000)
        .map(|t| t.order)
        .unwrap_or_else(|| data::TIERS.iter().map(|t| t.order).min().unwrap_or(1));
    sim.encounter_journal.current_tier = df_order;
    Ok(0)
}

fn noop(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn noop_global(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

// ---------------------------------------------------------------------------
// EJ_* handlers
// ---------------------------------------------------------------------------

fn ej_get_num_tiers(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(data::TIERS.len() as f64));
    Ok(1)
}

fn ej_get_tier_info(state: &mut LuaState) -> LuaResult<u32> {
    let tier_order = u32::from_stack(state, 1).unwrap_or(0);
    let Some(tier) = data::tier_by_order(tier_order) else {
        let empty = create_string(state, "");
        state.push(empty);
        return Ok(1);
    };
    let name = create_string(state, tier.name);
    state.push(name);
    Ok(1)
}

fn ej_get_current_tier(state: &mut LuaState) -> LuaResult<u32> {
    let tier = borrow_state(state)?.encounter_journal.current_tier;
    state.push(Val::Num(tier as f64));
    Ok(1)
}

fn ej_select_tier(state: &mut LuaState) -> LuaResult<u32> {
    let tier = u32::from_stack(state, 1).unwrap_or(0);
    if data::tier_by_order(tier).is_some() {
        borrow_state_mut(state)?.encounter_journal.current_tier = tier;
    }
    Ok(0)
}

fn ej_get_instance_info(state: &mut LuaState) -> LuaResult<u32> {
    let arg_present = !matches!(stack_val(state, 1), Val::Nil);
    let instance_id = if arg_present {
        u32::from_stack(state, 1).unwrap_or(0)
    } else {
        borrow_state(state)?.encounter_journal.current_instance
    };
    let Some(row) = data::instance_by_id(instance_id) else {
        return Ok(0);
    };
    push_instance_legacy_tuple(state, row);
    Ok(12)
}

fn ej_get_instance_by_index(state: &mut LuaState) -> LuaResult<u32> {
    let index = u32::from_stack(state, 1).unwrap_or(0) as usize;
    let is_raid = bool::from_stack(state, 2).unwrap_or(true);
    let tier_order = borrow_state(state)?.encounter_journal.current_tier;
    let instances = data::instances_for_tier(tier_order, is_raid);
    if index == 0 || index > instances.len() {
        return Ok(0);
    }
    let inst = instances[index - 1];
    state.push(Val::Num(inst.id as f64));
    push_instance_legacy_tuple(state, inst);
    Ok(13)
}

fn ej_get_encounter_info(state: &mut LuaState) -> LuaResult<u32> {
    let encounter_id = u32::from_stack(state, 1)?;
    let Some(row) = data::encounter_by_id(encounter_id) else {
        return Ok(0);
    };
    push_encounter_tuple(state, row);
    Ok(8)
}

fn ej_get_encounter_info_by_index(state: &mut LuaState) -> LuaResult<u32> {
    let index = u32::from_stack(state, 1).unwrap_or(0) as usize;
    let arg2_present = !matches!(stack_val(state, 2), Val::Nil);
    let instance_id = if arg2_present {
        u32::from_stack(state, 2).unwrap_or(0)
    } else {
        borrow_state(state)?.encounter_journal.current_instance
    };
    let encounters = data::encounters_for_instance(instance_id);
    if index == 0 || index > encounters.len() {
        return Ok(0);
    }
    push_encounter_tuple(state, encounters[index - 1]);
    Ok(8)
}

fn ej_get_creature_info(state: &mut LuaState) -> LuaResult<u32> {
    let index = u32::from_stack(state, 1).unwrap_or(0) as usize;
    let arg2_present = !matches!(stack_val(state, 2), Val::Nil);
    let encounter_id = if arg2_present {
        u32::from_stack(state, 2).unwrap_or(0)
    } else {
        borrow_state(state)?.encounter_journal.current_encounter
    };
    let creatures = data::creatures_for_encounter(encounter_id);
    if index == 0 || index > creatures.len() {
        return Ok(0);
    }
    let c = creatures[index - 1];
    let name = create_string(state, c.name);
    let description = create_string(state, c.description);
    state.push(Val::Num(c.id as f64));
    state.push(name);
    state.push(description);
    state.push(Val::Num(c.display_id as f64));
    state.push(optional_file_data_id(c.icon_file_id));
    state.push(Val::Num(c.model_scene_id as f64));
    Ok(6)
}

fn optional_file_data_id(file_data_id: u32) -> Val {
    if file_data_id == 0 {
        Val::Nil
    } else {
        Val::Num(file_data_id as f64)
    }
}

fn ej_select_instance(state: &mut LuaState) -> LuaResult<u32> {
    let instance_id = u32::from_stack(state, 1).unwrap_or(0);
    let mut sim = borrow_state_mut(state)?;
    sim.encounter_journal.current_instance = instance_id;
    sim.encounter_journal.current_encounter = 0;
    if let Some(inst) = data::instance_by_id(instance_id) {
        sim.encounter_journal.is_raid = inst.is_raid;
    }
    Ok(0)
}

fn ej_select_encounter(state: &mut LuaState) -> LuaResult<u32> {
    let encounter_id = u32::from_stack(state, 1).unwrap_or(0);
    borrow_state_mut(state)?.encounter_journal.current_encounter = encounter_id;
    Ok(0)
}

fn ej_get_selected_instance(state: &mut LuaState) -> LuaResult<u32> {
    let id = borrow_state(state)?.encounter_journal.current_instance;
    if id == 0 {
        return Ok(0);
    }
    state.push(Val::Num(id as f64));
    Ok(1)
}

fn ej_get_selected_encounter(state: &mut LuaState) -> LuaResult<u32> {
    let id = borrow_state(state)?.encounter_journal.current_encounter;
    if id == 0 {
        return Ok(0);
    }
    state.push(Val::Num(id as f64));
    Ok(1)
}

fn ej_get_difficulty(state: &mut LuaState) -> LuaResult<u32> {
    let difficulty = borrow_state(state)?.encounter_journal.difficulty;
    state.push(Val::Num(difficulty as f64));
    Ok(1)
}

fn ej_set_difficulty(state: &mut LuaState) -> LuaResult<u32> {
    let difficulty = u32::from_stack(state, 1).unwrap_or(0);
    borrow_state_mut(state)?.encounter_journal.difficulty = difficulty;
    Ok(0)
}

fn ej_instance_is_raid(state: &mut LuaState) -> LuaResult<u32> {
    let is_raid = {
        let sim = borrow_state(state)?;
        let id = sim.encounter_journal.current_instance;
        data::instance_by_id(id)
            .map(|i| i.is_raid)
            .unwrap_or(sim.encounter_journal.is_raid)
    };
    state.push(Val::Bool(is_raid));
    Ok(1)
}

fn ej_is_valid_instance_difficulty(state: &mut LuaState) -> LuaResult<u32> {
    let difficulty = u32::from_stack(state, 1).unwrap_or(0);
    state.push(Val::Bool(difficulty > 0));
    Ok(1)
}

fn ej_get_section_path(state: &mut LuaState) -> LuaResult<u32> {
    let section_id = u32::from_stack(state, 1).unwrap_or(0);
    let mut path: Vec<&'static str> = Vec::new();
    let mut cursor = section_id;
    while cursor != 0 {
        let Some(row) = data::section_by_id(cursor) else {
            break;
        };
        path.push(row.title);
        cursor = row.parent_id;
        if path.len() >= 16 {
            break;
        }
    }
    path.reverse();
    let joined = path.join(" > ");
    let s = create_string(state, &joined);
    state.push(s);
    Ok(1)
}

fn ej_get_content_tuning_id(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    Ok(1)
}

fn ej_handle_link_path(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn push_encounter_tuple(state: &mut LuaState, row: &data::Encounter) {
    let name = create_string(state, row.name);
    let description = create_string(state, row.description);
    let link = create_string(state, "");
    state.push(name);
    state.push(description);
    state.push(Val::Num(row.id as f64));
    state.push(Val::Num(row.first_section_id as f64));
    state.push(link);
    state.push(Val::Num(row.instance_id as f64));
    state.push(Val::Num(row.dungeon_encounter_id as f64));
    state.push(Val::Num(row.ui_map_id as f64));
}

fn push_instance_tuple(state: &mut LuaState, row: &data::Instance) {
    push_instance_base_tuple(state, row);
}

fn push_instance_legacy_tuple(state: &mut LuaState, row: &data::Instance) {
    let name = create_string(state, row.name);
    let description = create_string(state, row.description);
    state.push(name);
    state.push(description);
    state.push(Val::Num(row.bg_file_id as f64));
    state.push(Val::Num(row.button_file_id as f64));
    state.push(Val::Num(row.lore_file_id as f64));
    state.push(Val::Num(row.button_small_file_id as f64));
    state.push(Val::Num(row.area_id as f64));
    state.push(Val::Num(if row.is_raid { row.id as f64 } else { 0.0 }));
    state.push(Val::Bool(row.is_raid));
    state.push(Val::Num(row.map_id as f64));
    state.push(Val::Num(0.0));
    state.push(Val::Bool(row.is_raid));
}

fn push_instance_base_tuple(state: &mut LuaState, row: &data::Instance) {
    let name = create_string(state, row.name);
    let description = create_string(state, row.description);
    let link_dungeon_id = linked_lfd_dungeon_id(row);
    state.push(name);
    state.push(description);
    state.push(Val::Num(row.bg_file_id as f64));
    state.push(Val::Num(row.button_file_id as f64));
    state.push(Val::Num(row.lore_file_id as f64));
    state.push(Val::Num(row.button_small_file_id as f64));
    state.push(Val::Num(row.area_id as f64));
    state.push(Val::Num(if row.is_raid { row.id as f64 } else { 0.0 }));
    state.push(Val::Num(link_dungeon_id as f64));
}

fn linked_lfd_dungeon_id(row: &data::Instance) -> u32 {
    if row.is_raid {
        return 0;
    }
    match row.name {
        "Ara-Kara, City of Echoes" => 1201,
        "City of Threads" => 1202,
        "Mists of Tirna Scithe" => 1203,
        "The Stonevault" => 1204,
        "Grim Batol" => 1205,
        "The Dawnbreaker" => 1206,
        "Darkflame Cleft" => 1207,
        "The Rookery" => 1208,
        _ => row.id,
    }
}
