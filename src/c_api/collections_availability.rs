//! Collections journal subsystem availability.
//!
//! PTR 12.1.5 and WowForever 1.60.1 declare all three getters in their pinned
//! `Blizzard_APIDocumentationGenerated` addon:
//!
//! - `HeirloomInfoDocumentation.lua:29` — `C_HeirloomInfo.HeirloomsAvailable`.
//! - `MountJournalDocumentation.lua:498` — `C_MountJournal.MountEquipmentAvailable`.
//! - `WarbandSceneInfoDocumentation.lua:92` — `C_WarbandScene.WarbandScenesAvailable`.
//!
//! None of these names is declared in the Retail 12.1.5 pin, so they stay
//! unpublished there. The original consumers are
//! `Blizzard_Collections/Shared/Blizzard_Collections.lua:25,29` (PTR) / `:21,25`
//! (WowForever), which decide whether the Heirlooms and Warband Scenes tabs
//! exist, and `Blizzard_Collections/Mainline/Blizzard_MountCollection.lua:161`,
//! which wires or hides the mount-equipment inset.
//!
//! Native behaviour is UNKNOWN: no decompiled implementation has been consulted.
//! Both getters therefore read supplied scenario state instead of a fixed
//! constant. The defaults are best-guess scenario defaults for the two clients
//! that ship the respective collection surface.

use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{IntoStack, TableBuilder};
use rilua::LuaResult;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionsAvailability {
    /// Whether the heirloom journal is available to the current character.
    pub heirlooms: bool,
    /// Whether the mount journal offers mount equipment.
    pub mount_equipment: bool,
    /// Whether the warband scene journal is available to the current character.
    pub warband_scenes: bool,
}

impl Default for CollectionsAvailability {
    fn default() -> Self {
        Self {
            heirlooms: true,
            mount_equipment: true,
            warband_scenes: true,
        }
    }
}

fn publishes_availability() -> bool {
    matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Ptr
            | crate::client_profile::ClientProfile::WowForever
    )
}

pub(crate) fn register_mount_equipment_available(tb: TableBuilder) -> LuaResult<TableBuilder> {
    if !publishes_availability() {
        return Ok(tb);
    }
    tb.set_function("MountEquipmentAvailable", mount_equipment_available)
}

fn mount_equipment_available(state: &mut LuaState) -> LuaResult<u32> {
    let available = borrow_state(state)?
        .world
        .collections_availability
        .mount_equipment;
    available.into_stack(state)
}

pub(crate) fn register_heirlooms_available(
    state: &mut LuaState,
    namespace: GcRef<Table>,
) -> LuaResult<()> {
    if !publishes_availability() {
        return Ok(());
    }
    crate::lua_bridge::table_set_rust_fn_static(
        state,
        namespace,
        "HeirloomsAvailable",
        heirlooms_available,
    )
}

fn heirlooms_available(state: &mut LuaState) -> LuaResult<u32> {
    let available = borrow_state(state)?
        .world
        .collections_availability
        .heirlooms;
    available.into_stack(state)
}

pub(crate) fn register_warband_scenes_available(
    state: &mut LuaState,
    namespace: GcRef<Table>,
) -> LuaResult<()> {
    if !publishes_availability() {
        return Ok(());
    }
    crate::lua_bridge::table_set_rust_fn_static(
        state,
        namespace,
        "WarbandScenesAvailable",
        warband_scenes_available,
    )
}

fn warband_scenes_available(state: &mut LuaState) -> LuaResult<u32> {
    let available = borrow_state(state)?
        .world
        .collections_availability
        .warband_scenes;
    available.into_stack(state)
}
