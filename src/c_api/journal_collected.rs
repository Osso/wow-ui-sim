//! Collected-only Collections journal predicates.
//!
//! PTR 12.1.7.70493 and WowForever 1.60.1 declare these getters in their
//! pinned `Blizzard_APIDocumentationGenerated` addon:
//!
//! - `MountJournalDocumentation.lua:385` — `C_MountJournal.HasDisplayableMount`.
//! - `ToyBoxInfoDocumentation.lua:21` — `C_ToyBoxInfo.HasAnyToy`.
//! - `TransmogItemsDocumentation.lua:691` — `C_TransmogCollection.HasAnyAppearance`.
//!
//! The Retail 12.1.5 pin declares none of them, and its
//! `Blizzard_Collections/Mainline/Blizzard_Collections.lua:13`
//! `CollectionsJournal_ValidateTab` never reads
//! `onlyShowCollectedItemsInJournal`, so the three names stay unpublished
//! there. PTR and WowForever read them inside the collected-only branch of
//! `Blizzard_Collections/Shared/Blizzard_Collections.lua:34,42,46` (PTR) /
//! `:30,38,42` (WowForever) and in
//! `Blizzard_FrameXMLUtil/CollectionsUtil.lua:544`.
//!
//! Native semantics are UNKNOWN: no decompiled implementation has been
//! consulted. Each getter answers from the supplied collection journal state
//! instead of a fixed constant. `HasDisplayableMount` reports at least one
//! collected mount in the mount journal and does not test usability.
//! `HasAnyToy` reports at least one collected toy. `HasAnyAppearance` reports
//! at least one collected appearance in the appearance catalogue.

use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::{IntoStack, TableBuilder};
use rilua::LuaResult;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;

fn publishes_collected_predicates() -> bool {
    matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Ptr
            | crate::client_profile::ClientProfile::WowForever
    )
}

pub(crate) fn register_has_displayable_mount(tb: TableBuilder) -> LuaResult<TableBuilder> {
    if !publishes_collected_predicates() {
        return Ok(tb);
    }
    tb.set_function("HasDisplayableMount", has_displayable_mount)
}

fn has_displayable_mount(state: &mut LuaState) -> LuaResult<u32> {
    let has_collected_mount = borrow_state(state)?
        .world
        .mounts
        .iter()
        .any(|mount| mount.is_collected);
    has_collected_mount.into_stack(state)
}

pub(crate) fn register_has_any_toy(tb: TableBuilder) -> LuaResult<TableBuilder> {
    if !publishes_collected_predicates() {
        return Ok(tb);
    }
    tb.set_function("HasAnyToy", has_any_toy)
}

fn has_any_toy(state: &mut LuaState) -> LuaResult<u32> {
    let has_collected_toy = borrow_state(state)?
        .world
        .toys
        .iter()
        .any(|toy| toy.is_collected);
    has_collected_toy.into_stack(state)
}

pub(crate) fn register_has_any_appearance(
    state: &mut LuaState,
    namespace: GcRef<Table>,
) -> LuaResult<()> {
    if !publishes_collected_predicates() {
        return Ok(());
    }
    crate::lua_bridge::table_set_rust_fn_static(
        state,
        namespace,
        "HasAnyAppearance",
        has_any_appearance,
    )
}

fn has_any_appearance(state: &mut LuaState) -> LuaResult<u32> {
    let has_collected_appearance = borrow_state(state)?
        .world
        .transmog_appearances
        .iter()
        .any(|appearance| appearance.is_collected);
    has_collected_appearance.into_stack(state)
}
