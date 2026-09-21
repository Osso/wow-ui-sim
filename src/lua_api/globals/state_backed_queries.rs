//! State-backed globals and namespaces that still fell through to bootstrap
//! stubs.
//!
//! This module fixes the split where admin helpers mutate `SimState`, but the
//! public query globals still read static Lua bootstrap defaults.

use crate::lua_api::frame::methods::text_attribute_event::callbacks::dispatch_unit_event_callbacks;
use crate::lua_api::game_data::RACE_DATA;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, frame_ref, table_set,
};
use crate::lua_api::script_helpers::{get_event_listeners, get_script, protected_call_state};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use crate::{c_api::item_spell::item_link_for_id, items};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaApiMut, LuaResult, Val};

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "GetMoney", get_money)?;
    LuaApiMut::register_function(lua, "GetGuildInfo", get_guild_info)?;
    LuaApiMut::register_function(lua, "GetAverageItemLevel", get_average_item_level)?;
    LuaApiMut::register_function(lua, "FireEvent", fire_event)?;
    LuaApiMut::register_function(lua, "ReloadUI", reload_ui)?;
    LuaApiMut::register_function(lua, "GetLootRollItemInfo", get_loot_roll_item_info)?;
    LuaApiMut::register_function(lua, "GetLootRollItemLink", get_loot_roll_item_link)?;
    LuaApiMut::register_function(lua, "GetLootRollTimeLeft", get_loot_roll_time_left)?;
    LuaApiMut::register_function(lua, "GetActiveLootRollIDs", get_active_loot_roll_ids)?;
    register_loot_slot_functions(lua)?;
    LuaApiMut::register_function(lua, "UnitRace", unit_race)?;
    LuaApiMut::register_function(lua, "UnitSex", unit_sex)?;
    LuaApiMut::register_function(lua, "UnitHonorLevel", unit_honor_level)?;
    LuaApiMut::register_function(lua, "UnitPowerBarTimerInfo", unit_power_bar_timer_info)?;
    LuaApiMut::register_function(lua, "SetCursor", set_cursor)?;
    LuaApiMut::register_function(lua, "ResetCursor", reset_cursor)?;
    // Camelot uses C_SpecializationInfo; its deprecated alias addon is excluded.
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::WowForever {
        LuaApiMut::register_function(lua, "GetSpecialization", get_specialization)?;
    }
    LuaApiMut::register_function(lua, "GuildQuit", guild_quit)?;
    LuaApiMut::register_function(lua, "GetGuildInfo", c_guild_get_guild_info)?;
    LuaApiMut::register_function(lua, "GetGMStatus", get_gm_status)?;

    let state = lua.state_mut();
    register_c_guild(state)?;
    register_c_weekly_rewards(state)?;
    Ok(())
}

fn register_loot_slot_functions(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "GetLootSlotInfo", get_loot_slot_info)?;
    LuaApiMut::register_function(lua, "GetLootSlotLink", get_loot_slot_link)?;
    LuaApiMut::register_function(lua, "GetLootSlotType", get_loot_slot_type)?;
    LuaApiMut::register_function(lua, "LootSlotHasItem", loot_slot_has_item)?;
    LuaApiMut::register_function(lua, "LootSlot", loot_slot)?;
    LuaApiMut::register_function(lua, "IsFishingLoot", is_fishing_loot)?;
    LuaApiMut::register_function(lua, "RollOnLoot", roll_on_loot)?;
    LuaApiMut::register_function(lua, "ConfirmLootRoll", confirm_loot_roll)?;
    Ok(())
}

pub(crate) fn dispatch_event_now(
    state: &mut LuaState,
    event_name: &str,
    args: &[Val],
) -> LuaResult<()> {
    #[cfg(feature = "retail-12-0-0")]
    super::real::event_callbacks::dispatch_event_callbacks(state, event_name, args)?;
    let listeners = get_event_listeners(state, event_name);
    for widget_id in listeners {
        dispatch_unit_event_callbacks(state, widget_id, event_name, args);
        let Some(handler) = get_script(state, widget_id, "OnEvent") else {
            continue;
        };
        let mut call_args = Vec::with_capacity(args.len() + 2);
        call_args.push(frame_ref(state, widget_id)?);
        call_args.push(create_string(state, event_name));
        call_args.extend_from_slice(args);
        let _ = protected_call_state(state, handler, &call_args);
    }
    Ok(())
}

fn get_money(state: &mut LuaState) -> LuaResult<u32> {
    let money = borrow_state(state)?.player.money;
    state.push(Val::Num(money as f64));
    Ok(1)
}

fn get_guild_info(state: &mut LuaState) -> LuaResult<u32> {
    push_guild_info_values(state, GuildInfoShape::Legacy)
}

fn get_average_item_level(state: &mut LuaState) -> LuaResult<u32> {
    let item_level = borrow_state(state)?.player.item_level as f64;
    state.push(Val::Num(item_level));
    state.push(Val::Num(item_level));
    state.push(Val::Num(item_level));
    Ok(3)
}

fn fire_event(state: &mut LuaState) -> LuaResult<u32> {
    let event_name = String::from_stack(state, 1)?;
    let nargs = state.top as i32 - state.base as i32;
    let mut args = Vec::new();
    for slot in 2..=nargs {
        args.push(crate::lua_bridge::stack_val(state, slot));
    }
    dispatch_event_now(state, &event_name, &args)?;
    Ok(0)
}

fn reload_ui(state: &mut LuaState) -> LuaResult<u32> {
    dispatch_event_now(
        state,
        "PLAYER_ENTERING_WORLD",
        &[Val::Bool(false), Val::Bool(true)],
    )?;
    Ok(0)
}

fn get_loot_roll_item_info(state: &mut LuaState) -> LuaResult<u32> {
    let roll_id = i32::from_stack(state, 1)?;
    let Some(info) = borrow_state(state)?.world.loot_rolls.get(&roll_id).cloned() else {
        return Ok(0);
    };
    let texture = create_string(state, &info.texture);
    let name = create_string(state, &info.name);
    state.push(texture);
    state.push(name);
    state.push(Val::Num(info.count as f64));
    state.push(Val::Num(info.quality as f64));
    state.push(Val::Bool(info.bind_on_pickup));
    state.push(Val::Bool(info.can_need));
    state.push(Val::Bool(info.can_greed));
    state.push(Val::Bool(info.can_disenchant));
    state.push(Val::Num(info.disenchant_level as f64));
    state.push(Val::Num(info.item_level as f64));
    Ok(10)
}

fn get_loot_roll_item_link(state: &mut LuaState) -> LuaResult<u32> {
    let roll_id = i32::from_stack(state, 1)?;
    let link = borrow_state(state)?
        .world
        .loot_rolls
        .get(&roll_id)
        .map(|info| info.item_link.clone())
        .filter(|link| !link.is_empty());
    match link {
        Some(link) => {
            let link = create_string(state, &link);
            state.push(link);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn get_loot_roll_time_left(state: &mut LuaState) -> LuaResult<u32> {
    let roll_id = i32::from_stack(state, 1)?;
    let Some(time_left) = borrow_state(state)?
        .world
        .loot_rolls
        .get(&roll_id)
        .map(|info| info.roll_time)
    else {
        state.push(Val::Nil);
        return Ok(1);
    };
    state.push(Val::Num(time_left));
    Ok(1)
}

fn get_active_loot_roll_ids(state: &mut LuaState) -> LuaResult<u32> {
    let mut ids: Vec<i32> = borrow_state(state)?
        .world
        .loot_rolls
        .keys()
        .copied()
        .collect();
    ids.sort_unstable();
    let array = create_table(state);
    for (index, roll_id) in ids.into_iter().enumerate() {
        set_array_entry(state, array, index + 1, Val::Num(roll_id as f64));
    }
    state.push(array);
    Ok(1)
}

fn loot_slot_index(state: &mut LuaState) -> LuaResult<Option<usize>> {
    let slot = i32::from_stack(state, 1)?;
    Ok((slot > 0).then_some((slot - 1) as usize))
}

fn get_loot_slot_info(state: &mut LuaState) -> LuaResult<u32> {
    let Some(index) = loot_slot_index(state)? else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let Some(slot) = borrow_state(state)?.loot_slots.get(index).cloned() else {
        state.push(Val::Nil);
        return Ok(1);
    };

    let item = items::get_item(slot.item_id);
    let texture = item
        .map(|item| item.icon_file_data_id)
        .filter(|texture| *texture != 0)
        .unwrap_or(134400);
    let name = item.map(|item| item.name).unwrap_or("Unknown Item");
    let quality = item.map(|item| item.quality).unwrap_or(1);

    let name = create_string(state, name);
    state.push(Val::Num(texture as f64));
    state.push(name);
    state.push(Val::Num(slot.stack_count.max(1) as f64));
    state.push(Val::Nil);
    state.push(Val::Num(quality as f64));
    state.push(Val::Bool(false));
    Ok(6)
}

fn get_loot_slot_link(state: &mut LuaState) -> LuaResult<u32> {
    let Some(index) = loot_slot_index(state)? else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let link = borrow_state(state)?.loot_slots.get(index).and_then(|slot| {
        slot.hyperlink
            .clone()
            .or_else(|| item_link_for_id(slot.item_id))
    });
    match link {
        Some(link) => {
            let link = create_string(state, &link);
            state.push(link);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn get_loot_slot_type(state: &mut LuaState) -> LuaResult<u32> {
    let Some(index) = loot_slot_index(state)? else {
        state.push(Val::Num(0.0));
        return Ok(1);
    };
    let slot_type = if borrow_state(state)?.loot_slots.get(index).is_some() {
        1.0
    } else {
        0.0
    };
    state.push(Val::Num(slot_type));
    Ok(1)
}

fn loot_slot_has_item(state: &mut LuaState) -> LuaResult<u32> {
    let has_item = loot_slot_index(state)?.is_some_and(|index| {
        borrow_state(state).is_ok_and(|sim| sim.loot_slots.get(index).is_some())
    });
    state.push(Val::Bool(has_item));
    Ok(1)
}

fn loot_slot(state: &mut LuaState) -> LuaResult<u32> {
    let Some(index) = loot_slot_index(state)? else {
        return Ok(0);
    };
    let removed = {
        let mut sim = borrow_state_mut(state)?;
        if index < sim.loot_slots.len() {
            sim.loot_slots.remove(index);
            sim.loot_frame_open = !sim.loot_slots.is_empty();
            true
        } else {
            false
        }
    };
    if removed {
        dispatch_event_now(state, "LOOT_SLOT_CLEARED", &[Val::Num((index + 1) as f64)])?;
    }
    Ok(0)
}

fn is_fishing_loot(state: &mut LuaState) -> LuaResult<u32> {
    let is_fishing = borrow_state(state)?
        .loot_slots
        .iter()
        .any(|slot| slot.item_id == 6358);
    state.push(Val::Bool(is_fishing));
    Ok(1)
}

fn roll_on_loot(state: &mut LuaState) -> LuaResult<u32> {
    let roll_id = i32::from_stack(state, 1)?;
    let roll_type = i32::from_stack(state, 2)?;
    {
        let mut sim = borrow_state_mut(state)?;
        sim.world.loot_rolls.remove(&roll_id);
        sim.last_loot_roll_choice = Some(roll_type);
    }
    dispatch_event_now(state, "CANCEL_LOOT_ROLL", &[Val::Num(roll_id as f64)])?;
    Ok(0)
}

fn confirm_loot_roll(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn unit_race(state: &mut LuaState) -> LuaResult<u32> {
    let _ = Option::<String>::from_stack(state, 1)?;
    let race_index = borrow_state(state)?
        .player
        .race_index
        .min(RACE_DATA.len().saturating_sub(1));
    let (localized, english, _) = RACE_DATA[race_index];
    let localized = create_string(state, localized);
    let english = create_string(state, english);
    state.push(localized);
    state.push(english);
    state.push(Val::Num((race_index + 1) as f64));
    Ok(3)
}

fn unit_sex(state: &mut LuaState) -> LuaResult<u32> {
    let _ = Option::<String>::from_stack(state, 1)?;
    let sex = borrow_state(state)?.player.sex;
    state.push(Val::Num(sex as f64));
    Ok(1)
}

fn unit_honor_level(state: &mut LuaState) -> LuaResult<u32> {
    let _ = Option::<String>::from_stack(state, 1)?;
    let honor_level = borrow_state(state)?.player.honor_level;
    state.push(Val::Num(honor_level as f64));
    Ok(1)
}

fn unit_power_bar_timer_info(state: &mut LuaState) -> LuaResult<u32> {
    let _ = Option::<String>::from_stack(state, 1)?;
    let _ = Option::<f64>::from_stack(state, 2)?;
    state.push(Val::Nil);
    Ok(1)
}

fn set_cursor(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
    Ok(1)
}

fn reset_cursor(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn get_specialization(state: &mut LuaState) -> LuaResult<u32> {
    let spec_index = borrow_state(state)?.player.active_spec_index;
    state.push(Val::Num(spec_index as f64));
    Ok(1)
}

fn guild_quit(state: &mut LuaState) -> LuaResult<u32> {
    {
        let mut sim = borrow_state_mut(state)?;
        sim.world.guild_name = None;
        sim.world.guild_rank = None;
        sim.world.guild_num_members = 0;
    }
    dispatch_event_now(state, "PLAYER_GUILD_UPDATE", &[])?;
    Ok(0)
}

fn get_gm_status(state: &mut LuaState) -> LuaResult<u32> {
    dispatch_event_now(state, "UPDATE_GM_STATUS", &[Val::Num(0.0)])?;
    Ok(0)
}

fn register_c_guild(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_Guild");
    table_set_rust_fn_static(state, table_ref, "GetGuildInfo", c_guild_get_guild_info)?;
    table_set_rust_fn_static(state, table_ref, "IsInGuild", c_guild_is_in_guild)?;
    Ok(())
}

fn c_guild_get_guild_info(state: &mut LuaState) -> LuaResult<u32> {
    let _ = Option::<String>::from_stack(state, 1)?;
    push_guild_info_values(state, GuildInfoShape::Namespace)
}

enum GuildInfoShape {
    Legacy,
    Namespace,
}

fn push_guild_info_values(state: &mut LuaState, shape: GuildInfoShape) -> LuaResult<u32> {
    let (name, rank, num_members) = {
        let world = &borrow_state(state)?.world;
        (
            world.guild_name.clone(),
            world.guild_rank.clone(),
            world.guild_num_members,
        )
    };
    let Some(name) = name else {
        return Ok(0);
    };
    let name = create_string(state, &name);
    state.push(name);
    match rank {
        Some(rank) => {
            let rank = create_string(state, &rank);
            state.push(rank);
        }
        None => state.push(Val::Nil),
    }
    let third_value = match shape {
        GuildInfoShape::Legacy => 1.0,
        GuildInfoShape::Namespace => num_members as f64,
    };
    state.push(Val::Num(third_value));
    Ok(3)
}

fn c_guild_is_in_guild(state: &mut LuaState) -> LuaResult<u32> {
    let in_guild = borrow_state(state)?.world.guild_name.is_some();
    state.push(Val::Bool(in_guild));
    Ok(1)
}

fn register_c_weekly_rewards(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_WeeklyRewards");
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetActivities",
        c_weekly_rewards_get_activities,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "HasAvailableRewards",
        c_weekly_rewards_has_available_rewards,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "CanClaimRewards",
        c_weekly_rewards_can_claim_rewards,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetConquestWeeklyProgress",
        c_weekly_rewards_get_conquest_weekly_progress,
    )?;
    Ok(())
}

fn c_weekly_rewards_get_conquest_weekly_progress(state: &mut LuaState) -> LuaResult<u32> {
    let progress = create_table(state);
    table_set(state, progress, "progress", Val::Num(0.0));
    table_set(state, progress, "maxProgress", Val::Num(0.0));
    table_set(state, progress, "displayType", Val::Num(0.0));
    table_set(state, progress, "unlocksCompleted", Val::Num(0.0));
    table_set(state, progress, "maxUnlocks", Val::Num(0.0));
    table_set(state, progress, "sampleItemHyperlink", Val::Nil);
    state.push(progress);
    Ok(1)
}

fn c_weekly_rewards_get_activities(state: &mut LuaState) -> LuaResult<u32> {
    let filter_type = Option::<f64>::from_stack(state, 1)?.map(|value| value as i32);
    let activities = borrow_state(state)?
        .world
        .great_vault_activities
        .iter()
        .filter(|activity| filter_type.is_none_or(|expected| activity.activity_type == expected))
        .cloned()
        .collect::<Vec<_>>();
    let array = create_table(state);
    for (index, activity) in activities.into_iter().enumerate() {
        let row = create_table(state);
        table_set(state, row, "type", Val::Num(activity.activity_type as f64));
        table_set(state, row, "index", Val::Num(activity.index as f64));
        table_set(state, row, "threshold", Val::Num(activity.threshold as f64));
        table_set(state, row, "progress", Val::Num(activity.progress as f64));
        table_set(state, row, "level", Val::Num(activity.level as f64));
        set_array_entry(state, array, index + 1, row);
    }
    state.push(array);
    Ok(1)
}

fn c_weekly_rewards_has_available_rewards(state: &mut LuaState) -> LuaResult<u32> {
    let has_rewards = borrow_state(state)?.world.great_vault_has_rewards;
    state.push(Val::Bool(has_rewards));
    Ok(1)
}

fn c_weekly_rewards_can_claim_rewards(state: &mut LuaState) -> LuaResult<u32> {
    let can_claim = borrow_state(state)?.world.great_vault_can_claim;
    state.push(Val::Bool(can_claim));
    Ok(1)
}

fn ensure_namespace(state: &mut LuaState, name: &'static str) -> GcRef<Table> {
    let key = state.gc.intern_string(name.as_bytes());
    let global = state.global;
    let existing = state
        .gc
        .tables
        .get(global)
        .map(|table| table.get_str(key, &state.gc.string_arena));
    if let Some(Val::Table(table_ref)) = existing {
        return table_ref;
    }

    let namespace = create_table(state);
    let Val::Table(table_ref) = namespace else {
        unreachable!("create_table must return a table");
    };
    if let Some(global_table) = state.gc.tables.get_mut(global) {
        let _ = global_table.raw_set(Val::Str(key), namespace, &state.gc.string_arena);
    }
    state.gc.barrier_back(global);
    table_ref
}

fn set_array_entry(state: &mut LuaState, table: Val, index: usize, value: Val) {
    let Val::Table(table_ref) = table else {
        return;
    };
    if let Some(array) = state.gc.tables.get_mut(table_ref) {
        let _ = array.raw_set(Val::Num(index as f64), value, &state.gc.string_arena);
    }
    state.gc.barrier_back(table_ref);
}
