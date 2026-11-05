//! WM94 §5: `GetConduit` composes the conduit heading block, the item-level and
//! soulbound rows, and the conduit spell's description text.
//!
//! Retail/PTR/Forever `TooltipInfoDocumentation.lua:204` declare the getter.
//! Publication on PTR and Forever is **INFERRED** shared-model reuse.
//!
//! Modelled from WM94:
//! - a signed rank below 1 becomes 1; the conduit record and the exact rank row
//!   (`RankIndex == rank - 1`) are required, and the nearest lower rank is not
//!   substituted;
//! - the row supplies the spell ID; a missing spell-name record skips the custom
//!   heading block but the spell text still runs;
//! - the name is quality-coloured, then the Finesse/Potency/Endurance label, an
//!   item-level row (numeric type 31 with integer `itemLevel`), `ITEM_SOULBOUND`,
//!   and the description text in green;
//! - missing rank-properties data yields default quality 4 and item level zero.
//!
//! INFERRED layout: the only conduit-typed value in the pinned
//! `TooltipDataType` enum is `EnhancedConduit` (13), so both conduit getters use
//! it; the label, item-level and description rows use the simulator's established
//! plain (0), item-level (31) and spell-description (34) types. The native
//! heading/label line types are not recorded in the available evidence.
//!
//! UNKNOWN and therefore not published: the native item-level row text and the
//! exact localized description rendering.

use crate::c_api::tooltip_getter_args::signed;
use crate::c_api::tooltip_power_catalog_data::{
    CONDUIT_RANK_PROPERTIES, CONDUIT_RANKS, CONDUIT_TYPES,
};
use crate::lua_api::globals::missing_surface::tooltip_info::{
    append_line, compose_tooltip, format_global_numeric, global_text, item_quality_color,
    named_color, spell_description_text,
};
use crate::lua_api::methods::table_set;
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::LuaResult;
use rilua::Val;
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};

/// `Enum.TooltipDataType.EnhancedConduit`; the only conduit-typed enum value.
const TOOLTIP_TYPE_CONDUIT: f64 = 13.0;
const LINE_TYPE_SPELL_NAME: f64 = 13.0;
const LINE_TYPE_ITEM_LEVEL: f64 = 31.0;
const LINE_TYPE_ITEM_BINDING: f64 = 20.0;
const LINE_TYPE_SPELL_DESCRIPTION: f64 = 34.0;
const LINE_TYPE_PLAIN: f64 = 0.0;
/// WM94 §5: a missing rank-properties row uses default quality 4.
const DEFAULT_RANK_QUALITY: u8 = 4;

pub(crate) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetConduit", get_conduit)
}

fn get_conduit(state: &mut LuaState) -> LuaResult<u32> {
    let conduit_id = signed(state, 1)?;
    let rank = signed(state, 2)?.max(1);
    let Some(conduit_type) = conduit_type(conduit_id) else {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    };
    let Some(spell_id) = rank_spell(conduit_id, rank) else {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    };
    let (item_level, quality) = rank_properties(rank);
    let tooltip = compose_tooltip(state, TOOLTIP_TYPE_CONDUIT, None);
    if let Some(spell) = crate::spells::get_spell(spell_id) {
        append_line(
            state,
            tooltip,
            LINE_TYPE_SPELL_NAME,
            spell.name,
            Some(item_quality_color(quality)),
        );
    }
    if let Some(label) = conduit_type_label(state, conduit_type)? {
        append_line(state, tooltip, LINE_TYPE_PLAIN, &label, None);
    }
    let level_text = format_global_numeric(state, "ITEM_LEVEL", f64::from(item_level))?;
    let level_line = append_line(state, tooltip, LINE_TYPE_ITEM_LEVEL, &level_text, None);
    table_set(
        state,
        level_line,
        "itemLevel",
        Val::Num(f64::from(item_level)),
    );
    let soulbound = global_text(state, "ITEM_SOULBOUND").unwrap_or_else(|| "Soulbound".to_string());
    append_line(state, tooltip, LINE_TYPE_ITEM_BINDING, &soulbound, None);
    let description = spell_description_text(state, spell_id);
    if !description.is_empty() {
        let green = named_color(state, "GREEN_FONT_COLOR");
        append_line(
            state,
            tooltip,
            LINE_TYPE_SPELL_DESCRIPTION,
            &description,
            green,
        );
    }
    state.push(tooltip);
    Ok(1)
}

fn conduit_type(conduit_id: i32) -> Option<u8> {
    let conduit_id = u32::try_from(conduit_id).ok()?;
    CONDUIT_TYPES
        .binary_search_by_key(&conduit_id, |&(id, _)| id)
        .ok()
        .map(|index| CONDUIT_TYPES[index].1)
}

fn rank_spell(conduit_id: i32, rank: i32) -> Option<u32> {
    let conduit_id = u32::try_from(conduit_id).ok()?;
    let rank_index = u8::try_from(rank.checked_sub(1)?).ok()?;
    let key = (conduit_id, rank_index);
    CONDUIT_RANKS
        .binary_search_by_key(&key, |&(id, index, _)| (id, index))
        .ok()
        .map(|index| CONDUIT_RANKS[index].2)
}

fn rank_properties(rank: i32) -> (u32, u8) {
    let Ok(rank) = u8::try_from(rank) else {
        return (0, DEFAULT_RANK_QUALITY);
    };
    CONDUIT_RANK_PROPERTIES
        .binary_search_by_key(&rank, |&(value, _, _)| value)
        .map(|index| {
            let (_, item_level, quality) = CONDUIT_RANK_PROPERTIES[index];
            (item_level, quality)
        })
        .unwrap_or((0, DEFAULT_RANK_QUALITY))
}

fn conduit_type_label(state: &mut LuaState, conduit_type: u8) -> LuaResult<Option<String>> {
    let key = match conduit_type {
        0 => "CONDUIT_TYPE_FINESSE",
        1 => "CONDUIT_TYPE_POTENCY",
        2 => "CONDUIT_TYPE_ENDURANCE",
        _ => return Ok(None),
    };
    Ok(global_text(state, key))
}
