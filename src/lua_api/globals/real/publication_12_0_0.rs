//! Host inputs for plain globals added in Retail 12.0.0 and declared by Forever.
//! INFERRED: no native producer/default evidence; cost zero, marker system on,
//! clothing shown, empty role/threat/class inputs. No persistence or 3D rendering.

#[cfg(feature = "world-marker-placement")]
use crate::event::Event;
#[cfg(feature = "world-marker-placement")]
use crate::lua_api::globals::state_backed_queries::dispatch_event_now;
use crate::lua_api::methods::borrow_state;
#[cfg(feature = "world-marker-placement")]
use crate::lua_api::methods::borrow_state_mut;
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct PlainGlobalInputs {
    pub collapsing_star_cost: f64,
    pub raid_marker_system_enabled: bool,
    /// Supplied marker indexes; empty default is simulator policy, not native state.
    pub active_raid_markers: HashSet<i32>,
    pub showing_cloak: bool,
    pub showing_helm: bool,
    /// Known-unit GUIDs, not selected-unit token preferences.
    pub lieutenant_guids: HashSet<String>,
    pub minion_guids: HashSet<String>,
    pub npc_as_player_guids: HashSet<String>,
    pub cast_target_classes: HashMap<String, String>,
    /// (unit GUID, mob GUID) -> explicit lead snapshot. No fabricated thresholds.
    pub threat_lead: HashMap<(String, String), ThreatLeadSnapshot>,
    pub threat_state_restricted: bool,
    /// Explicit one-use gamepad hardware grant; no hardware source is fabricated.
    pub gamepad_cursor_input_available: bool,
}

impl Default for PlainGlobalInputs {
    fn default() -> Self {
        Self {
            collapsing_star_cost: 0.0,
            raid_marker_system_enabled: true,
            active_raid_markers: HashSet::new(),
            showing_cloak: true,
            showing_helm: true,
            lieutenant_guids: HashSet::new(),
            minion_guids: HashSet::new(),
            npc_as_player_guids: HashSet::new(),
            cast_target_classes: HashMap::new(),
            threat_lead: HashMap::new(),
            threat_state_restricted: false,
            gamepad_cursor_input_available: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ThreatLeadSnapshot {
    pub is_first: bool,
    /// Explicit host category 0..3; native category thresholds remain unverified.
    pub lead_state: u8,
}

fn is_raid_marker_system_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let enabled = borrow_state(state)?
        .plain_global_inputs
        .raid_marker_system_enabled;
    state.push(Val::Bool(enabled));
    Ok(1)
}

pub(crate) fn register_raid_marker_capability(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(
        lua,
        "IsRaidMarkerSystemEnabled",
        is_raid_marker_system_enabled,
    )?;
    Ok(())
}

fn is_raid_marker_active(state: &mut LuaState) -> LuaResult<u32> {
    // Era/Anniversary RaidMarkersDocumentation.lua:48-59: luaIndex -> bool.
    let index = i32::from_stack(state, 1)?;
    let active = borrow_state(state)?
        .plain_global_inputs
        .active_raid_markers
        .contains(&index);
    state.push(Val::Bool(active));
    Ok(1)
}

pub(crate) fn register_classic_marker_state(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "IsRaidMarkerActive", is_raid_marker_active)
}

/// Retail/Forever RaidMarkersDocumentation blocks are byte-identical. VERIFIED consumer
/// contract: the CompactRaidFrameManager uses these functions to update world-marker state.
/// Restricted-action authorization and optional token position are not modeled.
pub(crate) fn register_world_marker_state(lua: &mut rilua::Lua) -> LuaResult<()> {
    register_classic_marker_state(lua)?;
    LuaApiMut::register_function(lua, "ClearRaidMarker", clear_raid_marker)?;
    #[cfg(feature = "world-marker-placement")]
    LuaApiMut::register_function(lua, "PlaceRaidMarker", place_raid_marker)?;
    Ok(())
}

#[cfg(feature = "world-marker-placement")]
fn place_raid_marker(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    Option::<String>::from_stack(state, 2)?;
    if !(1..=8).contains(&index) {
        return Err(rilua::runtime_error(
            "PlaceRaidMarker: index must be in 1..8",
        ));
    }
    {
        let mut sim = borrow_state_mut(state)?;
        sim.plain_global_inputs.active_raid_markers.insert(index);
        sim.events.push(Event {
            name: "RAID_TARGET_UPDATE".to_string(),
            args: Vec::new(),
        });
    }
    dispatch_event_now(state, "RAID_TARGET_UPDATE", &[])?;
    Ok(0)
}

fn clear_raid_marker(state: &mut LuaState) -> LuaResult<u32> {
    let index = Option::<i32>::from_stack(state, 1)?;
    let mut sim = crate::lua_api::methods::borrow_state_mut(state)?;
    let markers = &mut sim.plain_global_inputs.active_raid_markers;
    match index {
        Some(index) => {
            markers.remove(&index);
        }
        None => markers.clear(),
    }
    Ok(0)
}

/// `A_Admin.SetActiveRaidMarkers(indexes)`: replaces the placed world-marker set.
pub(crate) fn admin_set_active_raid_markers(state: &mut LuaState) -> LuaResult<u32> {
    let rilua::Val::Table(table) = crate::lua_bridge::stack_val(state, 1) else {
        return Err(rilua::runtime_error(
            "A_Admin.SetActiveRaidMarkers expects an array of marker indexes",
        ));
    };
    let mut markers = HashSet::new();
    for position in 1.. {
        match state.gc.tables.get(table).map(|t| t.get_int(position)) {
            Some(Val::Num(index)) if index.fract() == 0.0 && (1.0..=8.0).contains(&index) => {
                markers.insert(index as i32);
            }
            Some(Val::Nil) | None => break,
            Some(_) => {
                return Err(rilua::runtime_error(
                    "A_Admin.SetActiveRaidMarkers indexes must be integers in 1..8",
                ));
            }
        }
    }
    crate::lua_api::methods::borrow_state_mut(state)?
        .plain_global_inputs
        .active_raid_markers = markers;
    Ok(0)
}

pub(crate) use retail::register_all;

mod retail {
    use super::*;
    use crate::lua_api::methods::borrow_state_mut;
    use crate::lua_bridge::stack_val;
    use rilua::table_security::unwrap_secret;
    fn get_collapsing_star_cost(state: &mut LuaState) -> LuaResult<u32> {
        let cost = borrow_state(state)?
            .plain_global_inputs
            .collapsing_star_cost;
        state.push(Val::Num(cost));
        Ok(1)
    }

    fn showing_cloak(state: &mut LuaState) -> LuaResult<u32> {
        let shown = borrow_state(state)?.plain_global_inputs.showing_cloak;
        state.push(Val::Bool(shown));
        Ok(1)
    }

    fn showing_helm(state: &mut LuaState) -> LuaResult<u32> {
        let shown = borrow_state(state)?.plain_global_inputs.showing_helm;
        state.push(Val::Bool(shown));
        Ok(1)
    }

    fn show_cloak(state: &mut LuaState) -> LuaResult<u32> {
        let shown = unwrap_secret(state, stack_val(state, 1))?.is_truthy();
        borrow_state_mut(state)?.plain_global_inputs.showing_cloak = shown;
        Ok(0)
    }

    fn show_helm(state: &mut LuaState) -> LuaResult<u32> {
        let shown = unwrap_secret(state, stack_val(state, 1))?.is_truthy();
        borrow_state_mut(state)?.plain_global_inputs.showing_helm = shown;
        Ok(0)
    }

    pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
        LuaApiMut::register_function(lua, "GetCollapsingStarCost", get_collapsing_star_cost)?;
        LuaApiMut::register_function(lua, "ShowCloak", show_cloak)?;
        LuaApiMut::register_function(lua, "ShowHelm", show_helm)?;
        LuaApiMut::register_function(lua, "ShowingCloak", showing_cloak)?;
        LuaApiMut::register_function(lua, "ShowingHelm", showing_helm)?;
        Ok(())
    }
}
