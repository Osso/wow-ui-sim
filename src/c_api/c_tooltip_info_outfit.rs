//! Outfit-name tooltip from the existing outfit catalog; no fabricated slot summaries.
use crate::lua_api::methods::{borrow_state, create_string, create_table, table_set};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val};

// TooltipInfoSharedDocumentation: Enum.TooltipDataType.Outfit.
const OUTFIT_TOOLTIP_TYPE: f64 = 27.0;

pub(crate) fn register(state: &mut LuaState, ns: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, ns, "GetOutfit", outfit)
}
fn outfit(state: &mut LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let Val::Num(id) = value else {
        return Err(rilua::runtime_error("outfitID must be a number"));
    };
    if !id.is_finite() || id.fract() != 0.0 || id <= 0.0 {
        return Err(rilua::runtime_error("outfitID must be a positive integer"));
    }
    let name = borrow_state(state)?
        .transmog_outfit_catalog
        .entries
        .iter()
        .find(|entry| entry.outfit_id as f64 == id)
        .map(|entry| entry.name.clone());
    let Some(name) = name else {
        return Ok(0);
    };
    publish_title(state, id, &name);
    Ok(1)
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
    super::helpers::set_table_array(state, lines, 1, title);
    table_set(state, title, "type", Val::Num(0.0));
    let text = create_string(state, name);
    table_set(state, title, "leftText", text);
}
