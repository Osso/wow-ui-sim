//! Guild/arena charter petitions shown by `PETITION_SHOW`.
//!
//! Consumer: cached Retail `Blizzard_UIPanels_Game/Mainline/PetitionFrame.lua`
//! `PetitionFrame_Update` reads `CanSignPetition`, the seven `GetPetitionInfo`
//! returns (type, title, body, maxSignatures, originator, isOriginator,
//! minSignatures), `GetNumPetitionNames` and `GetPetitionNameInfo(i)`; the
//! frame's `OnHide` calls `ClosePetition` (PetitionFrame.xml).
//! INFERRED (R-ag10-3): `ClosePetition` ends the server session and fires
//! `PETITION_CLOSED` once.

use super::{Arg, fire, push_optional_string};
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string, val_to_string};
use crate::lua_bridge::{FromStack, TableBuilder, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

const USAGE: &str = "Usage: A_Admin.OpenPetition(petitionType, title, bodyText, \
                     minSignatures, maxSignatures, originator, isOriginator, canSign, signatures)";

/// The petition the server is showing and its current signatures.
#[derive(Clone, Debug, PartialEq)]
pub struct Petition {
    pub petition_type: String,
    pub title: String,
    pub body_text: String,
    pub min_signatures: u32,
    pub max_signatures: u32,
    pub originator: String,
    pub is_originator: bool,
    pub can_sign: bool,
    pub signatures: Vec<String>,
}

fn read_signatures(state: &LuaState, index: i32) -> LuaResult<Vec<String>> {
    let Val::Table(table) = stack_val(state, index) else {
        return Err(runtime_error(USAGE));
    };
    let values: Vec<Val> = state
        .gc
        .tables
        .get(table)
        .map(|table| table.array_slice().to_vec())
        .unwrap_or_default();
    values
        .into_iter()
        .map(|value| val_to_string(state, value).ok_or_else(|| runtime_error(USAGE)))
        .collect()
}

fn read_petition(state: &LuaState) -> LuaResult<Petition> {
    let petition = Petition {
        petition_type: String::from_stack(state, 1)?,
        title: String::from_stack(state, 2)?,
        body_text: String::from_stack(state, 3)?,
        min_signatures: u32::from_stack(state, 4)?,
        max_signatures: u32::from_stack(state, 5)?,
        originator: String::from_stack(state, 6)?,
        is_originator: bool::from_stack(state, 7)?,
        can_sign: bool::from_stack(state, 8)?,
        signatures: read_signatures(state, 9)?,
    };
    let valid = matches!(petition.petition_type.as_str(), "guild" | "arena")
        && !petition.title.is_empty()
        && petition.min_signatures <= petition.max_signatures
        && petition.signatures.len() <= petition.max_signatures as usize;
    valid
        .then_some(petition)
        .ok_or_else(|| runtime_error(USAGE))
}

fn open_petition(state: &mut LuaState) -> LuaResult<u32> {
    let petition = read_petition(state)?;
    borrow_state_mut(state)?.interaction_dialogs.petition = Some(petition);
    fire(state, "PETITION_SHOW", Vec::new())?;
    Ok(0)
}

fn current(state: &LuaState) -> LuaResult<Option<Petition>> {
    Ok(borrow_state(state)?.interaction_dialogs.petition.clone())
}

fn get_petition_info(state: &mut LuaState) -> LuaResult<u32> {
    let Some(petition) = current(state)? else {
        return Ok(0);
    };
    for text in [
        &petition.petition_type,
        &petition.title,
        &petition.body_text,
    ] {
        let value = create_string(state, text);
        state.push(value);
    }
    state.push(Val::Num(f64::from(petition.max_signatures)));
    let originator = create_string(state, &petition.originator);
    state.push(originator);
    state.push(Val::Bool(petition.is_originator));
    state.push(Val::Num(f64::from(petition.min_signatures)));
    Ok(7)
}

fn get_num_petition_names(state: &mut LuaState) -> LuaResult<u32> {
    let count = current(state)?.map_or(0, |petition| petition.signatures.len());
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn get_petition_name_info(state: &mut LuaState) -> LuaResult<u32> {
    let index = i64::from_stack(state, 1)?;
    let name = current(state)?.and_then(|petition| {
        let slot = usize::try_from(index).ok()?.checked_sub(1)?;
        petition.signatures.get(slot).cloned()
    });
    push_optional_string(state, name.as_deref());
    Ok(1)
}

fn can_sign_petition(state: &mut LuaState) -> LuaResult<u32> {
    let can_sign = current(state)?.is_some_and(|petition| petition.can_sign);
    state.push(Val::Bool(can_sign));
    Ok(1)
}

fn close_petition(state: &mut LuaState) -> LuaResult<u32> {
    let closed = borrow_state_mut(state)?
        .interaction_dialogs
        .petition
        .take()
        .is_some();
    if closed {
        fire(state, "PETITION_CLOSED", Vec::<Arg>::new())?;
    }
    Ok(0)
}

pub(super) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    LuaApiMut::register_function(lua, "GetPetitionInfo", get_petition_info)?;
    LuaApiMut::register_function(lua, "GetNumPetitionNames", get_num_petition_names)?;
    LuaApiMut::register_function(lua, "GetPetitionNameInfo", get_petition_name_info)?;
    LuaApiMut::register_function(lua, "CanSignPetition", can_sign_petition)?;
    LuaApiMut::register_function(lua, "ClosePetition", close_petition)
}

pub(super) fn register_admin(b: TableBuilder) -> LuaResult<TableBuilder> {
    b.set_function("OpenPetition", open_petition)
}
