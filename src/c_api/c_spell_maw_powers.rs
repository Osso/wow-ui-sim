//! Host-declared Maw power strings; no native catalog or acquisition.
//! INFERRED nullable atlas, zero-result link misses, public outputs, strict public
//! identifier representations, and conservative secret rejection; not native parity.

use std::collections::HashMap;

use crate::lua_api::methods::{borrow_state, create_string};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::LuaResult;
#[cfg(not(feature = "retail-12-0-7"))]
use rilua::Val;
use rilua::vm::state::LuaState;

#[derive(Default)]
pub struct MawPowers {
    /// Exact host-declared atlas strings keyed by resolved spell ID; empty by default.
    pub border_atlases: HashMap<u32, String>,
    /// Exact host-declared links keyed by resolved spell ID; empty by default.
    pub links: HashMap<u32, String>,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = super::ensure_namespace(state, "C_Spell")?;
    #[cfg(not(feature = "retail-12-0-7"))]
    table_set_rust_fn_static(
        state,
        namespace,
        "GetMawPowerBorderAtlasBySpellID",
        get_maw_power_border_atlas_by_spell_id,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetMawPowerLinkBySpellID",
        get_maw_power_link_by_spell_id,
    )
}

#[cfg(not(feature = "retail-12-0-7"))]
fn get_maw_power_border_atlas_by_spell_id(state: &mut LuaState) -> LuaResult<u32> {
    let atlas = match super::c_spell::read_public_spell_identifier_at(
        state,
        1,
        "C_Spell.GetMawPowerBorderAtlasBySpellID",
    )? {
        Some(spell_id) => borrow_state(state)?
            .maw_powers
            .border_atlases
            .get(&spell_id)
            .cloned(),
        None => None,
    };
    // INFERRED from the retired nil shim and cached deprecated rarity-atlas wrapper.
    let result = atlas.map_or(Val::Nil, |atlas| create_string(state, &atlas));
    state.push(result);
    Ok(1)
}

fn get_maw_power_link_by_spell_id(state: &mut LuaState) -> LuaResult<u32> {
    let link = match super::c_spell::read_public_spell_identifier_at(
        state,
        1,
        "C_Spell.GetMawPowerLinkBySpellID",
    )? {
        Some(spell_id) => borrow_state(state)?
            .maw_powers
            .links
            .get(&spell_id)
            .cloned(),
        None => None,
    };
    // INFERRED zero-result miss from MayReturnNothing with a nonnil cstring return.
    let Some(link) = link else {
        return Ok(0);
    };
    let result = create_string(state, &link);
    state.push(result);
    Ok(1)
}
