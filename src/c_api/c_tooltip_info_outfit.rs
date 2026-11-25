//! Outfit-name tooltip; no fabricated slot summaries.
use crate::lua_api::methods::borrow_state;
use crate::lua_api::methods::{create_string, create_table, table_set};
#[cfg(feature = "retail-12-0-5")]
use crate::lua_bridge::stack_val;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val};

// TooltipInfoSharedDocumentation: Enum.TooltipDataType.Outfit.
const OUTFIT_TOOLTIP_TYPE: f64 = 27.0;
#[cfg(feature = "client-wowforever")]
const USAGE: &str = "Usage: local data = C_TooltipInfo.GetOutfit(outfitID)";

pub(crate) fn register(state: &mut LuaState, ns: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, ns, "GetOutfit", outfit)
}
fn outfit(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_outfit_id(state)?;
    if id == 0.0 {
        return Ok(0);
    }
    let name = read_outfit_name(state, id)?;
    let Some(name) = name else {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    };
    publish_title(state, id, &name);
    Ok(1)
}

#[cfg(feature = "client-wowforever")]
fn read_outfit_id(state: &LuaState) -> LuaResult<f64> {
    let id = crate::c_api::tooltip_getter_args::round7_optional_unsigned(state, 1, USAGE)?
        .ok_or_else(|| rilua::runtime_error(USAGE.to_string()))?;
    Ok(f64::from(id))
}

#[cfg(feature = "retail-12-0-5")]
fn read_outfit_id(state: &LuaState) -> LuaResult<f64> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let Val::Num(id) = value else {
        return Err(rilua::runtime_error("outfitID must be a number"));
    };
    if !id.is_finite() || id.fract() != 0.0 || id <= 0.0 {
        return Err(rilua::runtime_error("outfitID must be a positive integer"));
    }
    Ok(id)
}

#[cfg(feature = "client-wowforever")]
fn read_outfit_name(state: &LuaState, id: f64) -> LuaResult<Option<String>> {
    let name = borrow_state(state)?
        .transmog_outfit_catalog
        .entries
        .iter()
        .find(|entry| entry.outfit_id as f64 == id)
        .map(|entry| entry.name.clone())
        .unwrap_or_default();
    Ok(Some(name))
}

#[cfg(feature = "retail-12-0-5")]
fn read_outfit_name(state: &LuaState, id: f64) -> LuaResult<Option<String>> {
    let name = borrow_state(state)?
        .transmog_outfit_catalog
        .entries
        .iter()
        .find(|entry| entry.outfit_id as f64 == id)
        .map(|entry| entry.name.clone());
    Ok(name)
}
fn publish_title(state: &mut LuaState, id: f64, name: &str) {
    // INFERRED bounded title-only payload. Native line layout/appearance descriptions
    // are not known; the catalog name/identity and declared TooltipData shape are modeled.
    let result = create_table(state);
    state.push(result);
    table_set(state, result, "type", Val::Num(OUTFIT_TOOLTIP_TYPE));
    table_set(state, result, "id", Val::Num(id));
    let lines = create_table(state);
    table_set(state, result, "lines", lines);
    let title = create_table(state);
    crate::c_api::helpers::set_table_array(state, lines, 1, title);
    table_set(state, title, "type", Val::Num(0.0));
    let text = create_string(state, name);
    table_set(state, title, "leftText", text);
}
