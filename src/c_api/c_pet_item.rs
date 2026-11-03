//! Retail pet-item lookup from the pinned WM78 data chain.

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
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Retail
        && cfg!(feature = "retail-12-1-5")
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

fn select_display(entry: &PetItem) -> LuaResult<Option<u32>> {
    match entry.display_choices.as_slice() {
        [] => Ok(None),
        [display] => Ok(Some(*display)),
        _ => Err(runtime_error(format!(
            "pet item {} display selection is unresolved",
            entry.item_id
        ))),
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
    let display = select_display(entry)?;
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
