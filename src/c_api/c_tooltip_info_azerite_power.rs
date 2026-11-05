//! WM94 §5: `GetAzeritePower` selects the power record's spell and returns the
//! common spell producer's result.
//!
//! Retail/PTR/Forever `TooltipInfoDocumentation.lua:102` declare the getter.
//! Publication on PTR and Forever is **INFERRED** shared-model reuse, not
//! separate native proof.
//!
//! Modelled from WM94:
//! - the power record supplies one spell ID and the producer delegates to the
//!   common spell composition, with no separate extra-line invocation;
//! - a missing power record rejects construction;
//! - the supplied item ID, item level and power ID enter the spell request, and
//!   an optional owning-item link is parsed only to attach item fields.
//!
//! UNKNOWN and therefore not represented: what those item-context inputs change
//! inside the native common producer, and the exact common spell line set. The
//! simulator's common spell producer publishes the same lines with or without
//! item context, so the three catalog inputs are authenticated and unused. A
//! covered power whose spell has no `SpellName` row in the pinned export still
//! publishes the identified-but-empty spell tooltip, matching `GetSpellByID`.

use crate::c_api::tooltip_getter_args::{optional_link, signed};
use crate::c_api::tooltip_power_catalog_data::AZERITE_POWER_SPELLS;
use crate::lua_api::globals::missing_surface::tooltip_info::tooltip_for_spell;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::LuaResult;
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};

pub(crate) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetAzeritePower", get_azerite_power)
}

fn get_azerite_power(state: &mut LuaState) -> LuaResult<u32> {
    // WM94 §5: three required signed integers; all present arguments are
    // secret-checked. Only the power ID selects data.
    let _item_id = signed(state, 1)?;
    let _item_level = signed(state, 2)?;
    let power_id = signed(state, 3)?;
    let _owning_link = optional_link(state, 4)?;
    let Some(spell_id) = spell_for_power(power_id) else {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    };
    let tooltip = tooltip_for_spell(state, spell_id);
    state.push(tooltip);
    Ok(1)
}

fn spell_for_power(power_id: i32) -> Option<u32> {
    let power_id = u32::try_from(power_id).ok()?;
    AZERITE_POWER_SPELLS
        .binary_search_by_key(&power_id, |&(power, _)| power)
        .ok()
        .map(|index| AZERITE_POWER_SPELLS[index].1)
}
