//! Restored rilua API surface for item, spell, tooltip, and small legacy globals.

mod account_store;
mod achievement_info;
mod anima_diversion;
mod area_poi;
mod auction_house;
mod character_select;
mod club_finder;
mod club_info;
mod creature_info;
mod delves_ui;
mod encoding_util;
mod encounter_events;
mod encounter_journal;
mod encounter_warnings;
mod friend_list;
mod garrison;
mod gossip_info;
mod heirloom;
mod item_socket_info;
mod item_spell;
mod mythic_plus;
mod pet_battles;
mod player_info;
mod profession_crafting;
pub(crate) mod professions;
mod quest_choice;
mod quest_log;
mod recruit_a_friend;
mod scenario_info;
mod small_namespaces;
mod small_probes;
mod tooltip_info;
mod traits;
mod transmog;
mod transmog_collection;
mod tutorial;
#[path = "missing_surface/ui_widget_manager.rs"]
mod ui_widget_manager;
mod voice_chat;
mod warband_scene;
mod zone_ability;

use crate::c_api;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, table_get, val_to_string,
};
use crate::lua_bridge::{FromStack, stack_val};
use crate::spells;

use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaApiMut, LuaResult, Val};

const TOOLTIP_TYPE_ITEM: f64 = 0.0;
const TOOLTIP_TYPE_SPELL: f64 = 1.0;
const TOOLTIP_TYPE_UNIT: f64 = 2.0;
const TOOLTIP_TYPE_CURRENCY: f64 = 5.0;
const TOOLTIP_TYPE_UNIT_AURA: f64 = 7.0;
const TOOLTIP_TYPE_COMPANION_PET: f64 = 9.0;
const TOOLTIP_TYPE_MINIMAP_MOUSEOVER: f64 = 21.0;

const LINE_TYPE_UNIT_NAME: f64 = 2.0;
const LINE_TYPE_SPELL_NAME: f64 = 13.0;
const LINE_TYPE_ITEM_BINDING: f64 = 20.0;
const LINE_TYPE_EQUIP_SLOT: f64 = 21.0;
const LINE_TYPE_ITEM_NAME: f64 = 22.0;
const LINE_TYPE_ITEM_LEVEL: f64 = 31.0;
const LINE_TYPE_SPELL_DESCRIPTION: f64 = 34.0;
const WORLD_LOOT_TOOLTIP_SPELL_ID: u32 = 19750;
const WORLD_LOOT_TOOLTIP_INVENTORY_TYPE: f64 = 13.0;
const WORLD_CURSOR_GUID: &str = "WorldLootObject-0000-0000C0DE";

pub(crate) fn item_link_for_id(item_id: u32) -> Option<String> {
    item_spell::item_link_for_id(item_id)
}

pub fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    register_legacy_global_shims(lua)?;
    let state = lua.state_mut();
    register_item_trait_surfaces(state)?;
    register_world_namespace_surfaces(state)?;
    register_social_namespace_surfaces(state)?;
    register_group_activity_surfaces(state)?;
    Ok(())
}

fn register_legacy_global_shims(lua: &mut rilua::Lua) -> LuaResult<()> {
    register_audio_and_utility_globals(lua)?;
    register_item_spell_globals(lua)?;
    register_scene_and_ui_globals(lua)?;
    register_string_globals(lua)?;
    register_spellbook_and_totem_globals(lua)?;
    encounter_journal::register_ej_globals(lua)
}

fn register_audio_and_utility_globals(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "PlaySound", play_sound)?;
    LuaApiMut::register_function(lua, "PlaySoundFile", play_sound_file)?;
    LuaApiMut::register_function(lua, "StopSound", stop_sound)?;
    LuaApiMut::register_function(lua, "LaunchURL", launch_url)?;
    LuaApiMut::register_function(lua, "CopyToClipboard", copy_to_clipboard)?;
    install_date_alias(lua)?;
    Ok(())
}

fn register_item_spell_globals(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "GetSpellLink", get_spell_link_global)?;
    LuaApiMut::register_function(lua, "GetSpellIcon", get_spell_icon_global)?;
    LuaApiMut::register_function(lua, "GetItemInfo", get_item_info_global)?;
    LuaApiMut::register_function(lua, "GetItemClassInfo", get_item_class_info_global)?;
    Ok(())
}

fn register_scene_and_ui_globals(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(
        lua,
        "MapSceneCharacterHighlightStart",
        map_scene_character_highlight_start,
    )?;
    LuaApiMut::register_function(
        lua,
        "MapSceneCharacterHighlightEnd",
        map_scene_character_highlight_end,
    )?;
    LuaApiMut::register_function(lua, "CreateAtlasMarkup", create_atlas_markup)?;
    LuaApiMut::register_function(lua, "InGlue", in_glue)?;
    LuaApiMut::register_function(
        lua,
        "CanHearthAndResurrectFromArea",
        can_hearth_and_resurrect_from_area,
    )?;
    LuaApiMut::register_function(
        lua,
        "GetMaxLevelForLatestExpansion",
        get_max_level_for_latest_expansion,
    )?;
    LuaApiMut::register_function(lua, "GetRepairAllCost", get_repair_all_cost)?;
    LuaApiMut::register_function(lua, "SetActionUIButton", set_action_ui_button)?;
    Ok(())
}

fn register_string_globals(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "strsub", strsub)?;
    LuaApiMut::register_function(lua, "strconcat", strconcat)?;
    LuaApiMut::register_function(lua, "strlenutf8", strlenutf8)?;
    LuaApiMut::register_function(lua, "strcmputf8i", strcmputf8i)?;
    Ok(())
}

fn register_spellbook_and_totem_globals(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(
        lua,
        "FindSpellBookSlotBySpellID",
        find_spell_book_slot_by_spell_id,
    )?;
    LuaApiMut::register_function(lua, "GetMultiCastTotemSpells", get_multi_cast_totem_spells)?;
    LuaApiMut::register_function(lua, "GetMouseButtonClicked", get_mouse_button_clicked)?;
    Ok(())
}

fn install_date_alias(lua: &mut rilua::Lua) -> LuaResult<()> {
    let existing = LuaApiMut::get_global_val(lua, "date");
    if matches!(existing, Val::Function(_)) {
        return Ok(());
    }

    let os_table = LuaApiMut::get_global_val(lua, "os");
    let date_fn = table_get(lua.state_mut(), os_table, "date");
    if matches!(date_fn, Val::Function(_)) {
        LuaApiMut::set_global_val(lua, "date", date_fn)?;
    }
    Ok(())
}

fn register_item_trait_surfaces(state: &mut LuaState) -> LuaResult<()> {
    register_item_profession_surfaces(state)?;
    register_collection_surfaces(state)?;
    register_artifact_surfaces(state)?;
    register_interaction_surfaces(state)
}

fn register_item_profession_surfaces(state: &mut LuaState) -> LuaResult<()> {
    item_spell::register_item_and_spell_surfaces(state)?;
    encoding_util::register_encoding_util_surface(state)?;
    item_socket_info::register_item_socket_info_surface(state)?;
    professions::register_profession_surface(state)?;
    traits::register_trait_surfaces(state)?;
    tooltip_info::register_tooltip_surface(state)?;
    Ok(())
}

fn register_collection_surfaces(state: &mut LuaState) -> LuaResult<()> {
    transmog_collection::register_transmog_collection_surface(state)?;
    transmog::register_transmog_surface(state)?;
    tutorial::register_tutorial_surface(state)?;
    heirloom::register_heirloom_surface(state)?;
    Ok(())
}

fn register_artifact_surfaces(state: &mut LuaState) -> LuaResult<()> {
    register_spell_and_widget_surfaces(state)?;
    register_item_power_surfaces(state)?;
    register_character_progression_surfaces(state)?;
    Ok(())
}

fn register_spell_and_widget_surfaces(state: &mut LuaState) -> LuaResult<()> {
    c_api::register_spell_and_widget_tables(state)
}

fn register_item_power_surfaces(state: &mut LuaState) -> LuaResult<()> {
    c_api::register_item_power_tables(state)
}

fn register_character_progression_surfaces(state: &mut LuaState) -> LuaResult<()> {
    c_api::register_character_progression_tables(state)
}

fn register_interaction_surfaces(state: &mut LuaState) -> LuaResult<()> {
    c_api::register_interaction_tables(state)
}

fn register_world_namespace_surfaces(state: &mut LuaState) -> LuaResult<()> {
    register_map_and_encounter_surfaces(state)?;
    register_world_activity_surfaces(state)
}

fn register_map_and_encounter_surfaces(state: &mut LuaState) -> LuaResult<()> {
    c_api::register_map_prefix_tables(state)?;
    achievement_info::register_achievement_info_surface(state)?;
    area_poi::register_area_poi_surface(state)?;
    auction_house::register_auction_house_surface(state)?;
    encounter_events::register_encounter_events_surface(state)?;
    encounter_warnings::register_encounter_warnings_surface(state)?;
    creature_info::register_creature_info_surface(state)?;
    delves_ui::register_delves_ui_surface(state)?;
    encounter_journal::register_encounter_journal_surface(state)?;
    c_api::c_instance_encounter::register_c_instance_encounter_surface(state)?;
    c_api::c_quest_hub::register_c_quest_hub_surface(state)?;
    c_api::register_map_environment_tables(state)?;
    gossip_info::register_gossip_info_surface(state)?;
    Ok(())
}

fn register_world_activity_surfaces(state: &mut LuaState) -> LuaResult<()> {
    c_api::c_catalog_shop::register_c_catalog_shop_surface(state)?;
    c_api::c_chromie_time::register_c_chromie_time_surface(state)?;
    mythic_plus::register_mythic_plus_surface(state)?;
    if cfg!(feature = "client-mists") {
        quest_choice::register_quest_choice_surface(state)?;
    }
    scenario_info::register_scenario_info_surface(state)?;
    warband_scene::register_warband_scene_surface(state)?;
    c_api::c_player_choice::register_c_player_choice_surface(state)?;
    c_api::register_nameplate_tables(state)?;
    c_api::c_housing::register_c_housing_surface(state)?;
    c_api::c_settings_util::register_c_settings_util_surface(state)?;
    ui_widget_manager::register_ui_widget_manager_surface(state)?;
    anima_diversion::register_anima_diversion_surface(state)?;
    garrison::register_garrison_talent_surface(state)?;
    Ok(())
}

fn register_social_namespace_surfaces(state: &mut LuaState) -> LuaResult<()> {
    c_api::c_auto_complete::register_c_auto_complete_surface(state)?;
    c_api::c_battle_net::register_c_battle_net_surface(state)?;
    c_api::c_character_services::register_c_character_services_surface(state)?;
    c_api::c_chat_bubbles::register_c_chat_bubbles_surface(state)?;
    club_finder::register_club_finder_surface(state)?;
    club_info::register_club_info_surface(state)?;
    friend_list::register_friend_list_surface(state)?;
    recruit_a_friend::register_recruit_a_friend_surface(state)?;
    voice_chat::register_voice_chat_surface(state)?;
    c_api::c_social::register_c_social_surface(state)?;
    c_api::c_summon_info::register_c_summon_info_surface(state)?;
    character_select::register_character_select_surface(state)?;
    Ok(())
}

fn register_group_activity_surfaces(state: &mut LuaState) -> LuaResult<()> {
    c_api::c_death_recap::register_c_death_recap_surface(state)?;
    c_api::c_discord::register_c_discord_surface(state)?;
    c_api::c_party_info::register_c_party_info_surface(state)?;
    player_info::register_player_info_surface(state)?;
    c_api::c_lfg_info::register_c_lfg_info_surface(state)?;
    c_api::c_pet_battles::register_c_pet_battles_surface(state)?;
    pet_battles::register_pet_battles_surface(state)?;
    c_api::c_account_services::register_c_account_services_surface(state)?;
    c_api::c_merchant_frame::register_c_merchant_frame_surface(state)?;
    account_store::register_account_store_surface(state)?;
    c_api::c_report_system::register_c_report_system_surface(state)?;
    zone_ability::register_zone_ability_surface(state)?;
    c_api::c_stable_info::register_c_stable_info_surface(state)?;
    small_namespaces::register_small_namespaces(state)?;
    small_probes::register_small_probes_surface(state)?;
    Ok(())
}

/// Register the `C_QuestLog` SimState-backed surface.
///
/// Must be called **after** `quest_surface::register_all` because quest_surface
/// seeds the same C_QuestLog namespace first; calling this second lets our
/// SimState-backed handlers win.
pub fn register_quest_log_overrides(lua: &mut rilua::Lua) -> LuaResult<()> {
    let state = lua.state_mut();
    quest_log::register_quest_log_surface(state)
}

fn can_hearth_and_resurrect_from_area(state: &mut LuaState) -> LuaResult<u32> {
    let sim = borrow_state(state)?;
    let allowed = sim.pending_resurrect.is_some() && sim.can_teleport && sim.has_hearthstone;
    drop(sim);
    state.push(Val::Bool(allowed));
    Ok(1)
}

fn get_spell_link_global(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = u32::from_stack(state, 1)?;
    match item_spell::spell_link_for_id(spell_id) {
        Some(link) => {
            let link = create_string(state, &link);
            state.push(link);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn get_spell_icon_global(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = u32::from_stack(state, 1)?;
    let icon = spells::get_spell(spell_id)
        .map(|spell| {
            if spell.icon_file_data_id == 0 {
                136243
            } else {
                spell.icon_file_data_id
            }
        })
        .unwrap_or(136243);
    state.push(Val::Num(icon as f64));
    Ok(1)
}

fn get_item_class_info_global(state: &mut LuaState) -> LuaResult<u32> {
    let class_id = i32::from_stack(state, 1)?;
    let name = create_string(state, item_spell::item_class_name(class_id));
    state.push(name);
    Ok(1)
}

fn get_item_info_global(state: &mut LuaState) -> LuaResult<u32> {
    let Some(item_id) = item_spell::parse_item_id_from_val(state, stack_val(state, 1)) else {
        return Ok(0);
    };
    item_spell::push_item_info(state, item_id)
}

fn get_repair_all_cost(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    state.push(Val::Bool(false));
    Ok(2)
}

fn play_sound(state: &mut LuaState) -> LuaResult<u32> {
    let sound_kit_id = u32::from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    sim.last_sound_kit_requested = Some(sound_kit_id);
    if let Some(manager) = sim.sound_manager.as_mut() {
        let _ = manager.play_sound(sound_kit_id);
    }
    Ok(0)
}

fn play_sound_file(state: &mut LuaState) -> LuaResult<u32> {
    let path = val_to_string(state, stack_val(state, 1)).unwrap_or_default();
    let mut sim = borrow_state_mut(state)?;
    sim.last_sound_file_requested = Some(path.clone());
    if let Some(manager) = sim.sound_manager.as_mut() {
        let _ = manager.play_sound_file(&path);
    }
    Ok(0)
}

fn stop_sound(state: &mut LuaState) -> LuaResult<u32> {
    let handle = u32::from_stack(state, 1)?;
    let mut sim = borrow_state_mut(state)?;
    sim.last_stopped_sound_handle = Some(handle);
    if let Some(manager) = sim.sound_manager.as_mut() {
        manager.stop_sound(handle);
    }
    Ok(0)
}

fn launch_url(state: &mut LuaState) -> LuaResult<u32> {
    let url = val_to_string(state, stack_val(state, 1)).unwrap_or_default();
    borrow_state_mut(state)?.last_launched_url = Some(url);
    Ok(0)
}

fn copy_to_clipboard(state: &mut LuaState) -> LuaResult<u32> {
    let raw = val_to_string(state, stack_val(state, 1)).unwrap_or_default();
    let remove_markup = bool::from_stack(state, 2).unwrap_or(false);
    let stored = if remove_markup {
        crate::render::strip_wow_markup(&raw)
    } else {
        raw
    };
    let mut sim = borrow_state_mut(state)?;
    sim.clipboard.last_text = Some(stored);
    sim.clipboard.last_remove_markup = remove_markup;
    drop(sim);
    state.push(Val::Bool(true));
    Ok(1)
}

fn get_max_level_for_latest_expansion(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(80.0));
    Ok(1)
}

fn set_action_ui_button(state: &mut LuaState) -> LuaResult<u32> {
    let button = stack_val(state, 1);
    let action = u32::from_stack(state, 2)?;
    let Some(button_id) = crate::lua_api::methods::extract_frame_id(state, button) else {
        return Ok(0);
    };
    let mut sim = borrow_state_mut(state)?;
    sim.action_ui_buttons.retain(|(id, _)| *id != button_id);
    sim.action_ui_buttons.push((button_id, action));
    Ok(0)
}

fn map_scene_character_highlight_start(state: &mut LuaState) -> LuaResult<u32> {
    let guid = val_to_string(state, stack_val(state, 1)).unwrap_or_default();
    borrow_state_mut(state)?.highlighted_map_scene_character_guid = Some(guid);
    Ok(0)
}

fn map_scene_character_highlight_end(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.highlighted_map_scene_character_guid = None;
    Ok(0)
}

fn create_atlas_markup(state: &mut LuaState) -> LuaResult<u32> {
    let atlas_name = match stack_val(state, 1) {
        Val::Str(_) => val_to_string(state, stack_val(state, 1)).unwrap_or_default(),
        _ => String::new(),
    };
    let text = if atlas_name.is_empty() {
        String::new()
    } else {
        format!("|A:{atlas_name}:0:0|a")
    };
    let value = create_string(state, &text);
    state.push(value);
    Ok(1)
}

fn in_glue(state: &mut LuaState) -> LuaResult<u32> {
    let is_glue = crate::lua_api::methods::borrow_state(state)
        .map(|sim| sim.screen_kind.is_glue())
        .unwrap_or(false);
    state.push(Val::Bool(is_glue));
    Ok(1)
}

fn strsub(state: &mut LuaState) -> LuaResult<u32> {
    let Some(text) = val_to_string(state, stack_val(state, 1)) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let start = i32::from_stack(state, 2).unwrap_or(1);
    let end = i32::from_stack(state, 3).unwrap_or(-1);
    let len = text.chars().count() as i32;
    let normalize = |index: i32| {
        if index < 0 {
            (len + index + 1).max(1)
        } else {
            index.max(1)
        }
    };
    let start = normalize(start);
    let end = normalize(end).min(len);
    let result = if start > end || start > len {
        String::new()
    } else {
        text.chars()
            .skip((start - 1) as usize)
            .take((end - start + 1) as usize)
            .collect::<String>()
    };
    let value = create_string(state, &result);
    state.push(value);
    Ok(1)
}

fn strconcat(state: &mut LuaState) -> LuaResult<u32> {
    let nargs = state.top.saturating_sub(state.base);
    let mut result = String::new();
    for index in 0..nargs {
        if let Some(text) = val_to_string(state, state.stack_get(state.base + index)) {
            result.push_str(&text);
        }
    }
    let value = create_string(state, &result);
    state.push(value);
    Ok(1)
}

fn strlenutf8(state: &mut LuaState) -> LuaResult<u32> {
    let text = val_to_string(state, stack_val(state, 1)).unwrap_or_default();
    state.push(Val::Num(text.chars().count() as f64));
    Ok(1)
}

fn strcmputf8i(state: &mut LuaState) -> LuaResult<u32> {
    let left = val_to_string(state, stack_val(state, 1)).unwrap_or_default();
    let right = val_to_string(state, stack_val(state, 2)).unwrap_or_default();
    let ordering = left.to_lowercase().cmp(&right.to_lowercase());
    let result = match ordering {
        std::cmp::Ordering::Less => -1.0,
        std::cmp::Ordering::Equal => 0.0,
        std::cmp::Ordering::Greater => 1.0,
    };
    state.push(Val::Num(result));
    Ok(1)
}

fn find_spell_book_slot_by_spell_id(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = u32::from_stack(state, 1)?;
    match crate::lua_api::globals::spellbook_data::find_spell_slot(spell_id) {
        Some((slot, _)) => state.push(Val::Num(slot as f64)),
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn get_multi_cast_totem_spells(state: &mut LuaState) -> LuaResult<u32> {
    let _ = stack_val(state, 1);
    state.push(Val::Nil);
    Ok(1)
}

fn get_mouse_button_clicked(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Nil);
    Ok(1)
}

/// `name` must be a `&'static str` (typically a literal like `"C_Container"`)
/// so the pointer-keyed static intern cache short-circuits on repeat calls.
/// Every current caller passes a compile-time literal; `resolve_global_path`
/// exists as a separate entry point for the parse-time / addon-author case.
pub(super) fn ensure_namespace(
    state: &mut LuaState,
    name: &'static str,
) -> LuaResult<GcRef<Table>> {
    let key_ref = state.gc.intern_string_static(name.as_bytes());
    let current = state
        .gc
        .tables
        .get(state.global)
        .map(|globals| globals.get_str(key_ref, &state.gc.string_arena))
        .unwrap_or(Val::Nil);
    let table_ref = match current {
        Val::Table(table_ref) => table_ref,
        _ => {
            let table_ref = state.gc.alloc_table(Table::new());
            let global = state.global;
            if let Some(globals) = state.gc.tables.get_mut(global) {
                let _ = globals.raw_set(
                    Val::Str(key_ref),
                    Val::Table(table_ref),
                    &state.gc.string_arena,
                );
            }
            state.gc.barrier_back(global);
            table_ref
        }
    };
    Ok(table_ref)
}

pub(super) fn set_table_array(state: &mut LuaState, table: Val, index: i64, value: Val) {
    let Val::Table(table_ref) = table else { return };
    if let Some(table) = state.gc.tables.get_mut(table_ref) {
        let _ = table.raw_set(Val::Num(index as f64), value, &state.gc.string_arena);
    }
    state.gc.barrier_back(table_ref);
}
