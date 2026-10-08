//! Rilua A_Admin handlers — Transmog & collections.
//!
//! Extracted from rilua_admin_extras.rs per the 750-line file cap and to keep
//! each sub-module focused on a single concern. The parent entry
//! point in admin.rs imports these as pub(super) and weaves
//! them into the A_Admin TableBuilder chain.

use crate::event::{Event, EventArg};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, call_function_state, create_string, frame_ref,
};
use crate::lua_api::script_helpers::get_scripts_for_dispatch;
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
use std::collections::HashSet;

// ── Transmog & collections ────────────────────────────────────────────────────

const DEFAULT_TRANSMOG_SOURCE_TYPE: i32 = 1;

pub(super) fn add_transmog(state: &mut LuaState) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)?;
    borrow_state_mut(state)?
        .world
        .collected_transmogs
        .insert(id);
    Ok(0)
}

pub(super) fn remove_transmog(state: &mut LuaState) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)?;
    borrow_state_mut(state)?
        .world
        .collected_transmogs
        .remove(&id);
    Ok(0)
}

pub(super) fn add_transmog_appearance(state: &mut LuaState) -> LuaResult<u32> {
    use crate::lua_api::state_types::TransmogAppearance;
    let source_id = i32::from_stack(state, 1)?;
    let category_id = i32::from_stack(state, 2)?;
    let item_id = i32::from_stack(state, 3)?;
    let mut st = borrow_state_mut(state)?;
    let visual_id = st
        .world
        .transmog_appearances
        .iter()
        .map(|a| a.visual_id)
        .max()
        .unwrap_or(0)
        + 1;
    st.world.transmog_appearances.push(TransmogAppearance {
        source_id,
        visual_id,
        category_id,
        item_id,
        is_collected: true,
        source_type: DEFAULT_TRANSMOG_SOURCE_TYPE,
        item_mod_id: 0,
    });
    Ok(0)
}

pub(super) fn set_transmog_for_slot(state: &mut LuaState) -> LuaResult<u32> {
    let slot_id = i32::from_stack(state, 1)?;
    let source_id = i32::from_stack(state, 2)?;
    borrow_state_mut(state)?
        .world
        .applied_transmog_slots
        .insert(slot_id, source_id);
    Ok(0)
}

pub(super) fn collect_heirloom(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = i32::from_stack(state, 1)?;
    borrow_state_mut(state)?
        .world
        .collected_heirlooms
        .insert(item_id as u32);
    Ok(0)
}

pub(super) fn uncollect_heirloom(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = i32::from_stack(state, 1)?;
    borrow_state_mut(state)?
        .world
        .collected_heirlooms
        .remove(&(item_id as u32));
    Ok(0)
}

pub(super) fn set_mount_collected(state: &mut LuaState) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)?;
    let collected = bool::from_stack(state, 2)?;
    let mut st = borrow_state_mut(state)?;
    if collected {
        st.world.collected_mounts.insert(id);
    } else {
        st.world.collected_mounts.remove(&id);
    }
    Ok(0)
}

pub(super) fn set_pet_collected(state: &mut LuaState) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)?;
    let collected = bool::from_stack(state, 2)?;
    let mut st = borrow_state_mut(state)?;
    if collected {
        st.world.collected_pets.insert(id);
    } else {
        st.world.collected_pets.remove(&id);
    }
    Ok(0)
}

pub(super) fn set_toy_collected(state: &mut LuaState) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)?;
    let collected = bool::from_stack(state, 2)?;
    let mut st = borrow_state_mut(state)?;
    if collected {
        st.world.collected_toys.insert(id);
    } else {
        st.world.collected_toys.remove(&id);
    }
    Ok(0)
}

pub(super) fn set_campsite_collected(state: &mut LuaState) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)? as u32;
    let collected = bool::from_stack(state, 2)?;
    let mut st = borrow_state_mut(state)?;
    let mut newly_collected = false;
    if let Some(scene) = st
        .world
        .warband_scenes
        .iter_mut()
        .find(|scene| scene.warband_scene_id == id)
    {
        newly_collected = collected && !scene.is_collected;
        scene.is_collected = collected;
        if !collected {
            scene.is_favorite = false;
        }
    }

    if newly_collected {
        st.events.push(Event {
            name: "NEW_WARBAND_SCENE_ADDED".to_string(),
            args: vec![EventArg::Number(id as f64)],
        });
    }

    Ok(0)
}

pub(super) fn collect_campsite(state: &mut LuaState) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)? as u32;
    let mut st = borrow_state_mut(state)?;
    let mut newly_collected = false;
    if let Some(scene) = st
        .world
        .warband_scenes
        .iter_mut()
        .find(|scene| scene.warband_scene_id == id)
    {
        newly_collected = !scene.is_collected;
        scene.is_collected = true;
    }

    if newly_collected {
        st.events.push(Event {
            name: "NEW_WARBAND_SCENE_ADDED".to_string(),
            args: vec![EventArg::Number(id as f64)],
        });
    }

    Ok(0)
}

pub(super) fn uncollect_campsite(state: &mut LuaState) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)? as u32;
    let mut st = borrow_state_mut(state)?;
    if let Some(scene) = st
        .world
        .warband_scenes
        .iter_mut()
        .find(|scene| scene.warband_scene_id == id)
    {
        scene.is_collected = false;
        scene.is_favorite = false;
    }
    Ok(0)
}

pub(super) fn set_achievement_earned(state: &mut LuaState) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)?;
    let collected = bool::from_stack(state, 2)?;
    let mut st = borrow_state_mut(state)?;
    if collected {
        st.world.earned_achievements.insert(id);
    } else {
        st.world.earned_achievements.remove(&id);
    }
    Ok(0)
}

pub(super) fn has_achievement(state: &mut LuaState) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)?;
    let result = is_achievement_earned(&borrow_state(state)?.world.earned_achievements, id);
    state.push(Val::Bool(result));
    Ok(1)
}

fn is_achievement_earned(earned_achievements: &HashSet<i32>, id: i32) -> bool {
    earned_achievements.contains(&id)
}

pub(super) fn earn_achievement(state: &mut LuaState) -> LuaResult<u32> {
    let id = i32::from_stack(state, 1)?;
    let newly_earned = borrow_state_mut(state)?
        .world
        .earned_achievements
        .insert(id);
    if newly_earned {
        fire_achievement_earned(state, id);
    }
    Ok(0)
}

fn fire_achievement_earned(state: &mut LuaState, achievement_id: i32) {
    if let Ok(mut sim) = borrow_state_mut(state) {
        sim.events.push(Event {
            name: "ACHIEVEMENT_EARNED".to_string(),
            args: vec![EventArg::Number(achievement_id as f64)],
        });
    }
    let listeners = borrow_state(state)
        .map(|sim| sim.widgets.get_event_listeners("ACHIEVEMENT_EARNED"))
        .unwrap_or_default();
    for widget_id in listeners {
        let Ok(frame) = frame_ref(state, widget_id) else {
            continue;
        };
        let event_name = create_string(state, "ACHIEVEMENT_EARNED");
        let call_args = [frame, event_name, Val::Num(achievement_id as f64)];
        for handler in get_scripts_for_dispatch(state, widget_id, "OnEvent") {
            let _ = call_function_state(state, handler, &call_args);
        }
    }
}

pub(super) fn collect_mount(state: &mut LuaState) -> LuaResult<u32> {
    let mount_id = u32::from_stack(state, 1)?;
    let mut st = borrow_state_mut(state)?;
    st.world.collected_mounts.insert(mount_id as i32);
    if let Some(m) = st.world.mounts.iter_mut().find(|m| m.mount_id == mount_id) {
        m.is_collected = true;
        m.is_usable = true;
    }
    Ok(0)
}

pub(super) fn uncollect_mount(state: &mut LuaState) -> LuaResult<u32> {
    let mount_id = u32::from_stack(state, 1)?;
    let mut st = borrow_state_mut(state)?;
    st.world.collected_mounts.remove(&(mount_id as i32));
    if let Some(m) = st.world.mounts.iter_mut().find(|m| m.mount_id == mount_id) {
        m.is_collected = false;
        m.is_usable = false;
    }
    Ok(0)
}

pub(super) fn collect_pet(state: &mut LuaState) -> LuaResult<u32> {
    let species_id = u32::from_stack(state, 1)?;
    let mut st = borrow_state_mut(state)?;
    st.world.collected_pets.insert(species_id as i32);
    if let Some(p) = st
        .world
        .pets
        .iter_mut()
        .find(|p| p.species_id == species_id)
    {
        p.is_collected = true;
    }
    Ok(0)
}

pub(super) fn uncollect_pet(state: &mut LuaState) -> LuaResult<u32> {
    let species_id = u32::from_stack(state, 1)?;
    let mut st = borrow_state_mut(state)?;
    st.world.collected_pets.remove(&(species_id as i32));
    if let Some(p) = st
        .world
        .pets
        .iter_mut()
        .find(|p| p.species_id == species_id)
    {
        p.is_collected = false;
    }
    Ok(0)
}

pub(super) fn collect_toy(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = u32::from_stack(state, 1)?;
    let mut st = borrow_state_mut(state)?;
    st.world.collected_toys.insert(item_id as i32);
    let newly_acquired = if let Some(toy) = st.world.toys.iter_mut().find(|t| t.item_id == item_id)
    {
        let newly_acquired = !toy.is_collected;
        toy.is_collected = true;
        toy.is_usable = true;
        newly_acquired
    } else {
        false
    };
    if newly_acquired {
        st.world.toy_fanfare.mark_acquired(item_id);
    }
    Ok(0)
}

pub(super) fn uncollect_toy(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = u32::from_stack(state, 1)?;
    let mut st = borrow_state_mut(state)?;
    st.world.collected_toys.remove(&(item_id as i32));
    st.world.toy_fanfare.clear(item_id);
    if let Some(toy) = st.world.toys.iter_mut().find(|t| t.item_id == item_id) {
        toy.is_collected = false;
        toy.is_usable = false;
    }
    Ok(0)
}
