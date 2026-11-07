//! Retail and PTR pet-item lookup from the pinned WM78 data chain.
//!
//! `GetPetInfoByItemID` is absent from both pinned `Blizzard_APIDocumentationGenerated`
//! caches (`PetJournalInfoDocumentation.lua` jumps from `GetPetInfoTableBySpeciesID` to
//! `GetPetLoadOutInfo`), so the contract comes from the pinned consumers:
//! `Blizzard_PlayerChoice/Blizzard_PlayerChoiceOptionBase.lua:691` calls it unguarded,
//! `Blizzard_UIPanels_Game/Mainline/DressUpFrames.lua:178` reads creatureID (4), displayID (12)
//! and speciesID (13), and the PTR `Blizzard_SharedXML/LinkUtil.lua:127` reads the displayID.
//!
//! Known data gap: `data/pet-item-catalog.json` is derived from build 12.1.5.70391 while the
//! pinned PTR UI is 12.1.7.70493. Regenerating this generator over the 70493 tables adds 9 items
//! and changes 3 records (one renamed species), so PTR answers for those ids are stale.
//! See `docs/specs/pet-info-by-item.md`.

use std::collections::HashMap;
use std::sync::OnceLock;

use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};
use serde::Deserialize;

use crate::lua_api::methods::{create_string, val_to_string};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};

#[derive(Deserialize)]
struct PetItem {
    item_id: u32,
    name: Option<String>,
    icon: Option<u32>,
    pet_type: i32,
    creature_id: u32,
    source: String,
    description: String,
    wild: bool,
    can_battle: bool,
    tradeable: bool,
    unique: bool,
    obtainable: bool,
    display_choices: Vec<u32>,
    species_id: u32,
}

fn catalog() -> &'static HashMap<u32, PetItem> {
    static CATALOG: OnceLock<HashMap<u32, PetItem>> = OnceLock::new();
    CATALOG.get_or_init(|| {
        let entries: Vec<PetItem> =
            serde_json::from_str(include_str!("../../data/pet-item-catalog.json"))
                .expect("generated pet-item catalog must be valid");
        entries
            .into_iter()
            .map(|entry| (entry.item_id, entry))
            .collect()
    })
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail | crate::client_profile::ClientProfile::Ptr
    ) && cfg!(feature = "retail-12-1-5")
    {
        let namespace = super::ensure_namespace(state, "C_PetJournal")?;
        table_set_rust_fn_static(state, namespace, "GetPetInfoByItemID", get_pet_info_by_item)?;
    }
    Ok(())
}

fn item_id(state: &LuaState) -> LuaResult<i32> {
    let value = stack_val(state, 1);
    let number = match value {
        Val::Num(number) => Some(number),
        Val::Str(_) => val_to_string(state, value).and_then(|text| text.trim().parse().ok()),
        _ => None,
    }
    .ok_or_else(|| runtime_error("Usage: Script_GetPetInfoByItemID(itemID)"))?;
    if !number.is_finite() || number < i32::MIN as f64 || number > i32::MAX as f64 {
        return Err(runtime_error(format!(
            "integer overflow attempting to store {number:.6}"
        )));
    }
    Ok(number.trunc() as i32)
}

fn select_display(entry: &PetItem) -> Option<Option<u32>> {
    match entry.display_choices.as_slice() {
        [] => Some(None),
        [display] => Some(Some(*display)),
        _ => None,
    }
}

fn push_optional_number(state: &mut LuaState, value: Option<u32>) {
    state.push(value.map_or(Val::Nil, |value| Val::Num(value as f64)));
}

fn get_pet_info_by_item(state: &mut LuaState) -> LuaResult<u32> {
    let id = item_id(state)?;
    let Some(entry) = u32::try_from(id).ok().and_then(|id| catalog().get(&id)) else {
        return Ok(0);
    };
    let Some(display) = select_display(entry) else {
        return Ok(0);
    };
    let name = entry
        .name
        .as_ref()
        .map_or(Val::Nil, |name| create_string(state, name));
    state.push(name);
    push_optional_number(state, entry.icon);
    state.push(Val::Num(entry.pet_type as f64));
    state.push(Val::Num(entry.creature_id as f64));
    let source = create_string(state, &entry.source);
    state.push(source);
    let description = create_string(state, &entry.description);
    state.push(description);
    for value in [
        entry.wild,
        entry.can_battle,
        entry.tradeable,
        entry.unique,
        entry.obtainable,
    ] {
        state.push(Val::Bool(value));
    }
    push_optional_number(state, display);
    state.push(Val::Num(entry.species_id as f64));
    Ok(13)
}
