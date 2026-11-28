//! Host inputs for plain globals added in Retail 12.0.0 and declared by Forever.
//! INFERRED: no native producer/default evidence; cost zero, marker system on,
//! clothing shown, empty role/threat/class inputs. No persistence or 3D rendering.

use crate::lua_api::methods::borrow_state;
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

pub(crate) use retail::register_all;

/// World markers placed in the world, cleared by `ClearRaidMarker`.
///
/// Declarations: Retail and Forever `RaidMarkersDocumentation.lua`
/// (byte-identical; `ClearRaidMarker(raidMarkerIndex = MAX_RAID_MARKERS)`,
/// `IsRaidMarkerActive(index) -> bool`). Consumers: `/clearworldmarker`
/// (`SlashCommands.lua`: a number clears one marker, `ALL` passes nil to clear
/// every marker) and `Blizzard_CompactRaidFrameManager.lua` marker buttons.
/// Placement needs a ground target, so the host supplies placed markers.
/// INFERRED (R-ag10-7): nil clears all markers; an index outside 1..8 changes
/// nothing; the documentation declares no event for either.
#[cfg(feature = "world-marker-state")]
pub(crate) mod world_markers {
    use super::is_raid_marker_active;
    use crate::lua_api::methods::borrow_state_mut;
    use crate::lua_bridge::{FromStack, TableBuilder};
    use rilua::vm::state::LuaState;
    use rilua::{LuaApiMut, LuaResult, runtime_error};

    /// `Constants.RaidMarkerConsts` world marker count (`MAX_RAID_MARKERS`).
    const WORLD_MARKERS: i32 = 8;

    fn clear_raid_marker(state: &mut LuaState) -> LuaResult<u32> {
        let index = Option::<i32>::from_stack(state, 1)?;
        let markers = &mut borrow_state_mut(state)?
            .plain_global_inputs
            .active_raid_markers;
        match index {
            None => markers.clear(),
            Some(index) => {
                markers.remove(&index);
            }
        }
        Ok(0)
    }

    fn set_world_marker(state: &mut LuaState) -> LuaResult<u32> {
        let index = i32::from_stack(state, 1)?;
        let placed = bool::from_stack(state, 2)?;
        if !(1..=WORLD_MARKERS).contains(&index) {
            return Err(runtime_error(
                "Usage: A_Admin.SetWorldMarker(index 1..8, placed)",
            ));
        }
        let markers = &mut borrow_state_mut(state)?
            .plain_global_inputs
            .active_raid_markers;
        if placed {
            markers.insert(index);
        } else {
            markers.remove(&index);
        }
        Ok(0)
    }

    pub(crate) fn register(lua: &mut rilua::Lua) -> LuaResult<()> {
        LuaApiMut::register_function(lua, "ClearRaidMarker", clear_raid_marker)?;
        LuaApiMut::register_function(lua, "IsRaidMarkerActive", is_raid_marker_active)
    }

    pub(crate) fn register_admin(b: TableBuilder) -> LuaResult<TableBuilder> {
        b.set_function("SetWorldMarker", set_world_marker)
    }
}

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
