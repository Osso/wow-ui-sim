//! State-backed ClubFinder queries used by Communities.

use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

#[derive(Clone, Debug, Default)]
pub(crate) struct ClubFinderState {
    /// INFERRED R88fq (native evidence pending): no supplied locale filter means zero flags.
    pub player_applicant_locale_flags: u32,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    if !matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever) {
        return Ok(());
    }

    let namespace = super::helpers::ensure_namespace(state, "C_ClubFinder")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetRecruitingClubInfoFromClubID",
        get_recruiting_club_info,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetPlayerApplicantLocaleFlags",
        get_player_applicant_locale_flags,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "SetPlayerApplicantLocaleFlags",
        set_player_applicant_locale_flags,
    )?;
    Ok(())
}

// VERIFIED (Retail/Forever ClubFinderDocumentation.lua:181-193): the posting result is nilable.
fn get_recruiting_club_info(state: &mut LuaState) -> LuaResult<u32> {
    let _club_id = String::from_stack(state, 1)?;
    // Recruiting postings are not modeled, so no club has a posting record.
    state.push(Val::Nil);
    Ok(1)
}

// VERIFIED (Retail/Forever ClubFinderDocumentation.lua:124-130): this query returns one number.
fn get_player_applicant_locale_flags(state: &mut LuaState) -> LuaResult<u32> {
    let flags = borrow_state(state)?
        .club_finder
        .player_applicant_locale_flags;
    state.push(Val::Num(flags as f64));
    Ok(1)
}

// VERIFIED (Retail/Forever ClubFinderDocumentation.lua:578-586): this setter takes one number.
fn set_player_applicant_locale_flags(state: &mut LuaState) -> LuaResult<u32> {
    let flags = u32::from_stack(state, 1)?;
    borrow_state_mut(state)?
        .club_finder
        .player_applicant_locale_flags = flags;
    Ok(0)
}
