//! Native action-bar namespace and helpers kept alive on the rilua path.

mod registration;
pub use registration::register_all;

use crate::Result;
use crate::c_api::c_action_bar::{get_action_cooldown_duration, read_action_cooldown};
use crate::lua_api::SimState;
use crate::lua_api::globals::lua_duration_object::new_duration_object_value;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, call_function_state, create_string, create_table,
    create_table_with_capacity, extract_frame_id, frame_ref, table_get, table_set,
};
use crate::lua_api::script_helpers::fire_named_event_state;
use crate::lua_bridge::stack_val;
use rilua::LuaApiMut;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
use std::cell::RefCell;
use std::rc::Rc;

const C_ACTION_BAR: &str = "C_ActionBar";
const NUM_ACTIONBAR_PAGES: i32 = 6;
const ACTION_COOLDOWN_HASH_FIELDS: usize = 4;
const ACTION_CHARGES_HASH_FIELDS: usize = 5;
const ACTION_LOC_COOLDOWN_HASH_FIELDS: usize = 5;

fn stack_slot(state: &LuaState) -> Option<u32> {
    stack_slot_at(state, 1)
}

fn stack_slot_at(state: &LuaState, index: i32) -> Option<u32> {
    match stack_val(state, index) {
        Val::Num(n) if n >= 0.0 => Some(n as u32),
        _ => None,
    }
}

fn action_texture_path(state: &SimState, slot: u32) -> Option<String> {
    let spell_id = state.action_bars.get(&slot)?;
    let spell = crate::spells::get_spell(*spell_id)?;
    crate::manifest_interface_data::get_texture_path(spell.icon_file_data_id).map(str::to_string)
}

fn current_bonus_bar_index(state: &mut LuaState) -> i32 {
    let namespace = table_get(state, Val::Table(state.global), C_ACTION_BAR);
    let get_bonus_bar_index = table_get(state, namespace, "GetBonusBarIndex");
    match call_function_state(state, get_bonus_bar_index, &[]) {
        Ok(Val::Num(n)) => n as i32,
        _ => 0,
    }
}

fn push_empty_table(state: &mut LuaState) -> LuaResult<u32> {
    let table = create_table(state);
    state.push(table);
    Ok(1)
}

fn push_bool(state: &mut LuaState, value: bool) -> LuaResult<u32> {
    state.push(Val::Bool(value));
    Ok(1)
}

fn push_i32(state: &mut LuaState, value: i32) -> LuaResult<u32> {
    state.push(Val::Num(value as f64));
    Ok(1)
}

fn push_nil(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Nil);
    Ok(1)
}

fn push_no_results(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn get_bonus_bar_index_for_slot(state: &mut LuaState) -> LuaResult<u32> {
    let _ = stack_val(state, 1);
    push_i32(state, 0)
}

fn is_on_bar_or_special_bar(state: &mut LuaState) -> LuaResult<u32> {
    let _ = stack_val(state, 1);
    push_bool(state, false)
}

fn find_spell_action_buttons(state: &mut LuaState) -> LuaResult<u32> {
    let _ = stack_val(state, 1);
    push_empty_table(state)
}

fn get_current_action_bar_by_class(state: &mut LuaState) -> LuaResult<u32> {
    push_i32(state, 1)
}

fn has_flyout_action_buttons(state: &mut LuaState) -> LuaResult<u32> {
    let _ = stack_val(state, 1);
    push_bool(state, false)
}

fn enable_action_range_check(state: &mut LuaState) -> LuaResult<u32> {
    let _ = stack_val(state, 1);
    let _ = stack_val(state, 2);
    push_no_results(state)
}

fn is_assisted_combat_action(state: &mut LuaState) -> LuaResult<u32> {
    let _ = stack_val(state, 1);
    push_bool(state, false)
}

fn has_assisted_combat_action_buttons(state: &mut LuaState) -> LuaResult<u32> {
    push_bool(state, false)
}

fn get_action_bar_page(state: &mut LuaState) -> LuaResult<u32> {
    let page = borrow_state(state)?.action_bar_page as i32;
    push_i32(state, page)
}

fn set_action_bar_page(state: &mut LuaState) -> LuaResult<u32> {
    let Some(page) = stack_slot(state) else {
        return push_no_results(state);
    };
    if page < 1 || page as i32 > NUM_ACTIONBAR_PAGES {
        return push_no_results(state);
    }
    let changed = {
        let mut sim = borrow_state_mut(state)?;
        let was = sim.action_bar_page;
        sim.action_bar_page = page;
        was != page
    };
    if changed {
        fire_named_event_state(state, "ACTIONBAR_PAGE_CHANGED", &[]);
    }
    push_no_results(state)
}

fn get_extra_bar_index(state: &mut LuaState) -> LuaResult<u32> {
    push_i32(state, 13)
}

fn get_multicast_bar_index(state: &mut LuaState) -> LuaResult<u32> {
    push_i32(state, 7)
}

fn get_vehicle_bar_index(state: &mut LuaState) -> LuaResult<u32> {
    let index = borrow_state(state)?.vehicle_bar_index;
    push_i32(state, index)
}

fn get_override_bar_index(state: &mut LuaState) -> LuaResult<u32> {
    let index = borrow_state(state)?.override_bar_index;
    push_i32(state, index)
}

fn get_temp_shapeshift_bar_index(state: &mut LuaState) -> LuaResult<u32> {
    let index = borrow_state(state)?.temp_shapeshift_bar_index;
    push_i32(state, index)
}

fn get_bonus_bar_index(state: &mut LuaState) -> LuaResult<u32> {
    let bonus_bar_index = borrow_state(state)?.bonus_bar_index;
    push_i32(state, bonus_bar_index)
}

fn get_bonus_bar_offset(state: &mut LuaState) -> LuaResult<u32> {
    let bonus_bar_index = current_bonus_bar_index(state);
    push_i32(state, (bonus_bar_index - NUM_ACTIONBAR_PAGES).max(0))
}

fn get_override_bar_skin(state: &mut LuaState) -> LuaResult<u32> {
    let override_bar_skin = borrow_state(state)?.override_bar_skin;
    match override_bar_skin {
        Some(skin) => push_i32(state, skin),
        None => push_nil(state),
    }
}

fn has_vehicle_action_bar(state: &mut LuaState) -> LuaResult<u32> {
    let has_vehicle_action_bar = borrow_state(state)?.has_vehicle_action_bar;
    push_bool(state, has_vehicle_action_bar)
}

fn has_override_action_bar(state: &mut LuaState) -> LuaResult<u32> {
    let has_override_action_bar = borrow_state(state)?.has_override_action_bar;
    push_bool(state, has_override_action_bar)
}

fn has_bonus_action_bar(state: &mut LuaState) -> LuaResult<u32> {
    let has_bonus_action_bar = borrow_state(state)?.has_bonus_action_bar;
    push_bool(state, has_bonus_action_bar)
}

fn has_temp_shapeshift_action_bar(state: &mut LuaState) -> LuaResult<u32> {
    let has_temp_shapeshift_action_bar = borrow_state(state)?.has_temp_shapeshift_action_bar;
    push_bool(state, has_temp_shapeshift_action_bar)
}

fn has_extra_action_bar(state: &mut LuaState) -> LuaResult<u32> {
    let granted = borrow_state(state)?.extra_action_button.spell_id.is_some();
    push_bool(state, granted)
}

fn is_possess_bar_visible(state: &mut LuaState) -> LuaResult<u32> {
    let visible = borrow_state(state)?.action_bar_state.possess_bar_visible;
    push_bool(state, visible)
}

fn get_action_text(state: &mut LuaState) -> LuaResult<u32> {
    let _ = stack_val(state, 1);
    push_nil(state)
}

fn get_action_count(state: &mut LuaState) -> LuaResult<u32> {
    let _ = stack_val(state, 1);
    push_i32(state, 0)
}

fn get_action_display_count(state: &mut LuaState) -> LuaResult<u32> {
    let _ = stack_val(state, 1);
    let _ = stack_val(state, 2);
    push_nil(state)
}

fn get_action_use_count(state: &mut LuaState) -> LuaResult<u32> {
    let _ = stack_val(state, 1);
    push_i32(state, 0)
}

fn is_consumable_action(state: &mut LuaState) -> LuaResult<u32> {
    let _ = stack_val(state, 1);
    push_bool(state, false)
}

fn is_stackable_action(state: &mut LuaState) -> LuaResult<u32> {
    let _ = stack_val(state, 1);
    push_bool(state, false)
}

fn is_item_action(state: &mut LuaState) -> LuaResult<u32> {
    let _ = stack_val(state, 1);
    push_bool(state, false)
}

fn is_attack_action(state: &mut LuaState) -> LuaResult<u32> {
    let _ = stack_val(state, 1);
    push_bool(state, false)
}

fn is_auto_repeat_action(state: &mut LuaState) -> LuaResult<u32> {
    let _ = stack_val(state, 1);
    push_bool(state, false)
}

fn is_equipped_action(state: &mut LuaState) -> LuaResult<u32> {
    let _ = stack_val(state, 1);
    push_bool(state, false)
}

fn is_equipped_gear_outfit_action(state: &mut LuaState) -> LuaResult<u32> {
    let slot = stack_slot(state);
    let is_equipped_gear_outfit = {
        let sim = borrow_state(state)?;
        slot.is_some_and(|slot| sim.equipped_gear_outfit_action_slots.contains(&slot))
    };
    push_bool(state, is_equipped_gear_outfit)
}

fn is_helpful_action(state: &mut LuaState) -> LuaResult<u32> {
    let _ = stack_val(state, 1);
    let _ = stack_val(state, 2);
    push_bool(state, false)
}

fn is_harmful_action(state: &mut LuaState) -> LuaResult<u32> {
    let _ = stack_val(state, 1);
    let _ = stack_val(state, 2);
    push_bool(state, false)
}

fn is_press_hold_release_spell(state: &mut LuaState) -> LuaResult<u32> {
    let _ = stack_val(state, 1);
    push_bool(state, false)
}

fn get_action_loss_of_control_cooldown(state: &mut LuaState) -> LuaResult<u32> {
    let _ = stack_val(state, 1);
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    Ok(2)
}

fn get_action_loss_of_control_cooldown_info(state: &mut LuaState) -> LuaResult<u32> {
    let _ = stack_val(state, 1);
    let info = create_table_with_capacity(state, ACTION_LOC_COOLDOWN_HASH_FIELDS);
    table_set(state, info, "isActive", Val::Bool(false));
    table_set(state, info, "startTime", Val::Num(0.0));
    table_set(state, info, "duration", Val::Num(0.0));
    table_set(state, info, "modRate", Val::Num(1.0));
    table_set(state, info, "shouldReplaceNormalCooldown", Val::Bool(false));
    state.push(info);
    Ok(1)
}

fn uses_action_text(state: &mut LuaState) -> LuaResult<u32> {
    let _ = stack_val(state, 1);
    push_bool(state, false)
}

fn get_action_charge_duration(state: &mut LuaState) -> LuaResult<u32> {
    let _ = stack_val(state, 1);
    let duration = new_duration_object_value(state);
    state.push(duration);
    Ok(1)
}

fn get_action_loss_of_control_cooldown_duration(state: &mut LuaState) -> LuaResult<u32> {
    let _ = stack_val(state, 1);
    let duration = new_duration_object_value(state);
    state.push(duration);
    Ok(1)
}

fn get_spell(state: &mut LuaState) -> LuaResult<u32> {
    let _ = stack_val(state, 1);
    push_nil(state)
}

fn get_item_action_on_equip_spell_id(state: &mut LuaState) -> LuaResult<u32> {
    let _ = stack_val(state, 1);
    push_nil(state)
}

fn find_flyout_action_buttons(state: &mut LuaState) -> LuaResult<u32> {
    let _ = stack_val(state, 1);
    push_empty_table(state)
}

fn find_pet_action_buttons(state: &mut LuaState) -> LuaResult<u32> {
    let _ = stack_val(state, 1);
    push_empty_table(state)
}

fn get_pet_action_pet_bar_indices(state: &mut LuaState) -> LuaResult<u32> {
    let _ = stack_val(state, 1);
    push_empty_table(state)
}

fn register_action_ui_button(state: &mut LuaState) -> LuaResult<u32> {
    let button = stack_val(state, 1);
    let Some(button_id) = extract_frame_id(state, button) else {
        return push_no_results(state);
    };
    let Some(action) = stack_slot_at(state, 2) else {
        return push_no_results(state);
    };
    let mut sim = borrow_state_mut(state)?;
    sim.action_ui_buttons.retain(|(id, _)| *id != button_id);
    sim.action_ui_buttons.push((button_id, action));
    drop(sim);
    push_no_results(state)
}

fn is_auto_cast_pet_action(state: &mut LuaState) -> LuaResult<u32> {
    let _ = stack_val(state, 1);
    push_bool(state, false)
}

fn is_enabled_auto_cast_pet_action(state: &mut LuaState) -> LuaResult<u32> {
    let _ = stack_val(state, 1);
    push_bool(state, false)
}

fn toggle_auto_cast_pet_action(state: &mut LuaState) -> LuaResult<u32> {
    let _ = stack_val(state, 1);
    push_no_results(state)
}

fn has_action(state: &mut LuaState) -> LuaResult<u32> {
    let slot = stack_slot(state);
    let has_action = {
        let sim = borrow_state(state)?;
        slot.is_some_and(|slot| {
            sim.action_bars.contains_key(&slot)
                || sim.action_outfits.contains_key(&slot)
                || sim.action_macros.contains_key(&slot)
        })
    };
    state.push(Val::Bool(has_action));
    Ok(1)
}

fn get_action_texture(state: &mut LuaState) -> LuaResult<u32> {
    let slot = stack_slot(state);
    let texture = {
        let sim = borrow_state(state)?;
        slot.and_then(|slot| action_texture_path(&sim, slot))
    };

    match texture {
        Some(path) => {
            let texture = create_string(state, &path);
            state.push(texture);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn is_usable_action(state: &mut LuaState) -> LuaResult<u32> {
    let slot = stack_slot(state);
    let usable = {
        let sim = borrow_state(state)?;
        slot.is_some_and(|slot| {
            sim.action_bars.contains_key(&slot)
                || sim.action_outfits.contains_key(&slot)
                || sim.action_macros.contains_key(&slot)
        })
    };
    state.push(Val::Bool(usable));
    state.push(Val::Bool(false));
    Ok(2)
}

fn is_current_action(state: &mut LuaState) -> LuaResult<u32> {
    let slot = stack_slot(state).unwrap_or(0);
    let is_current = {
        let sim = borrow_state(state)?;
        if let Some(casting) = sim.casting.as_ref().map(|cast| cast.spell_id) {
            sim.action_bars.get(&slot).copied() == Some(casting)
        } else {
            false
        }
    };
    state.push(Val::Bool(is_current));
    Ok(1)
}

fn get_action_cooldown(state: &mut LuaState) -> LuaResult<u32> {
    let (start, duration) = read_action_cooldown(state)?;
    let info = create_table_with_capacity(state, ACTION_COOLDOWN_HASH_FIELDS);
    table_set(state, info, "startTime", Val::Num(start));
    table_set(state, info, "duration", Val::Num(duration));
    table_set(state, info, "isEnabled", Val::Bool(true));
    table_set(state, info, "modRate", Val::Num(1.0));
    state.push(info);
    Ok(1)
}

fn get_action_charges(state: &mut LuaState) -> LuaResult<u32> {
    let _ = stack_val(state, 1);
    let info = create_table_with_capacity(state, ACTION_CHARGES_HASH_FIELDS);
    table_set(state, info, "currentCharges", Val::Num(0.0));
    table_set(state, info, "maxCharges", Val::Num(0.0));
    table_set(state, info, "cooldownStartTime", Val::Num(0.0));
    table_set(state, info, "cooldownDuration", Val::Num(0.0));
    table_set(state, info, "chargeModRate", Val::Num(1.0));
    state.push(info);
    Ok(1)
}

/// `C_ActionBar.PutActionInSlot(slot, targetSlot)` → `bool`.
///
/// Moves the spell or outfit action stored at `slot` into `targetSlot`,
/// firing `ACTIONBAR_SLOT_CHANGED` for both slots so the action button
/// mixins refresh their textures. Returns `true` when there was an action
/// to move; `false` for empty slots, missing target, or non-numeric input.
fn put_action_in_slot(state: &mut LuaState) -> LuaResult<u32> {
    let Some(source) = stack_slot(state) else {
        return push_bool(state, false);
    };
    let Some(target) = stack_slot_at(state, 2) else {
        return push_bool(state, false);
    };
    if !transfer_action_between_slots(state, source, target)? {
        return push_bool(state, false);
    }
    fire_named_event_state(state, "ACTIONBAR_SLOT_CHANGED", &[Val::Num(source as f64)]);
    fire_named_event_state(state, "ACTIONBAR_SLOT_CHANGED", &[Val::Num(target as f64)]);
    push_bool(state, true)
}

fn transfer_action_between_slots(
    state: &mut LuaState,
    source: u32,
    target: u32,
) -> LuaResult<bool> {
    let moved_spell = {
        let mut sim = borrow_state_mut(state)?;
        sim.action_bars.remove(&source)
    };
    let moved_outfit = {
        let mut sim = borrow_state_mut(state)?;
        sim.action_outfits.remove(&source)
    };
    let moved_macro = borrow_state_mut(state)?.action_macros.remove(&source);
    if let Some(spell_id) = moved_spell {
        let mut sim = borrow_state_mut(state)?;
        crate::c_api::action_macros::clear_slot(&mut sim, target);
        sim.action_bars.insert(target, spell_id);
        sim.equipped_gear_outfit_action_slots.remove(&source);
        sim.equipped_gear_outfit_action_slots.remove(&target);
    } else if let Some(outfit_id) = moved_outfit {
        let mut sim = borrow_state_mut(state)?;
        transfer_outfit_action(&mut sim, source, target, outfit_id);
    } else if let Some(id) = moved_macro {
        let mut sim = borrow_state_mut(state)?;
        crate::c_api::action_macros::assign_macro(&mut sim, target, id)?;
        sim.equipped_gear_outfit_action_slots.remove(&source);
    } else {
        return Ok(false);
    }
    Ok(true)
}

fn transfer_outfit_action(sim: &mut SimState, source: u32, target: u32, outfit_id: i64) {
    let is_equipped_gear = sim.equipped_gear_outfit_action_slots.remove(&source);
    crate::c_api::action_macros::clear_slot(sim, target);
    sim.action_outfits.insert(target, outfit_id);
    if is_equipped_gear {
        sim.equipped_gear_outfit_action_slots.insert(target);
    } else {
        sim.equipped_gear_outfit_action_slots.remove(&target);
    }
}

/// `C_ActionBar.ForceUpdateAction(slot)` — fires `ACTIONBAR_SLOT_CHANGED`
/// for the given slot so the rotation manager / action button mixins
/// re-pull texture and cooldown state. No-op for non-numeric input.
fn force_update_action(state: &mut LuaState) -> LuaResult<u32> {
    let Some(slot) = stack_slot(state) else {
        return push_no_results(state);
    };
    fire_named_event_state(state, "ACTIONBAR_SLOT_CHANGED", &[Val::Num(slot as f64)]);
    push_no_results(state)
}

/// `C_ActionBar.GetProfessionQualityInfo(slot)` → ProfessionQualityInfo or nil.
///
/// Reads `state.action_profession_quality`. Missing entries return nil so
/// `ActionBarActionButtonMixin:UpdateProfessionQuality` clears the overlay
/// frame instead of populating it with a stale atlas.
fn get_profession_quality_info(state: &mut LuaState) -> LuaResult<u32> {
    let Some(slot) = stack_slot(state) else {
        return push_nil(state);
    };
    let info = borrow_state(state)?
        .action_profession_quality
        .get(&(slot as i32))
        .cloned();
    let Some(info) = info else {
        return push_nil(state);
    };
    let table = create_table(state);
    table_set(
        state,
        table,
        "inventoryQuality",
        Val::Num(info.inventory_quality as f64),
    );
    let icon_inventory = create_string(state, &info.icon_inventory);
    table_set(state, table, "iconInventory", icon_inventory);
    let icon_quality_container = create_string(state, &info.icon_quality_container);
    table_set(state, table, "iconQualityContainer", icon_quality_container);
    state.push(table);
    Ok(1)
}

pub fn push_action_button_state_update(
    state: &Rc<RefCell<SimState>>,
    lua: &mut rilua::Lua,
) -> Result<()> {
    let button_ids = {
        let sim = state.borrow();
        sim.action_ui_buttons
            .iter()
            .map(|(button_id, _)| *button_id)
            .collect::<Vec<_>>()
    };
    if button_ids.is_empty() {
        return Ok(());
    }

    let state = lua.state_mut();
    for button_id in button_ids {
        let button = frame_ref(state, button_id)?;
        let update_state = table_get(state, button, "UpdateState");
        if matches!(update_state, rilua::Val::Function(_)) {
            call_function_state(state, update_state, &[button])?;
        }
    }
    Ok(())
}

pub fn spell_cooldown_times(state: &SimState, spell_id: u32, now: f64) -> (f64, f64) {
    let mut best_start = 0.0_f64;
    let mut best_end = 0.0_f64;

    if let Some((gcd_start, gcd_duration)) = state.gcd {
        let gcd_end = gcd_start + gcd_duration;
        if gcd_end > now {
            best_start = gcd_start;
            best_end = gcd_end;
        }
    }

    if let Some(cooldown) = state.spell_cooldowns.get(&spell_id) {
        let cooldown_end = cooldown.start + cooldown.duration;
        if cooldown_end > now && cooldown_end > best_end {
            best_start = cooldown.start;
            best_end = cooldown_end;
        }
    }

    if best_end > now {
        (best_start, best_end - best_start)
    } else {
        (0.0, 0.0)
    }
}

pub fn start_cooldowns<T>(_state: &Rc<RefCell<SimState>>, _lua: T, _spell_id: u32) -> Result<()> {
    Ok(())
}

pub fn start_cast<T>(
    _state: &Rc<RefCell<SimState>>,
    _lua: T,
    _spell_id: u32,
    _cast_time_ms: i32,
) -> Result<()> {
    Ok(())
}

pub fn apply_instant_spell<T>(
    _state: &Rc<RefCell<SimState>>,
    _lua: T,
    _spell_id: u32,
) -> Result<()> {
    Ok(())
}
