//! WM108 §B: `GetEnhancedConduit` reuses the conduit membership check, adds two
//! ranks up to the conduit's rank-row count, and keeps quality and item level on
//! the original clamped rank.
//!
//! Retail/PTR/Forever `TooltipInfoDocumentation.lua:254` declare the getter.
//! Publication on PTR and Forever is **INFERRED** shared-model reuse.
//!
//! Modelled from WM108 §B and WM94 §5:
//! - the membership key is the conduit entry id, the same lookup `GetConduit`
//!   performs, so the two-rank increase applies to every valid conduit;
//! - the cap is the number of `SoulbindConduitRank` rows for that conduit, which
//!   is 15 in the pinned 70391 export;
//! - the adjusted rank is `original + 2` clamped to the cap, and the exact rank
//!   row must exist; the original clamped rank selects quality and item level;
//! - row composition is the `GetConduit` block plus the `SOULBIND_CONDUIT_ENHANCED`
//!   label, and the description's highlight-colour tokens become enhanced-colour
//!   tokens.
//!
//! EMULATED seam: the enhanced colour resolves through the simulator's named
//! colour surface. The pinned evidence does not record the `GlobalColor` value,
//! so the token rewrite is a no-op whenever that surface publishes the
//! placeholder white. Retire the seam when the colour table is modelled.

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
const DEFAULT_RANK_QUALITY: u8 = 4;
/// WM108 §B2: the added ranks and the exact-rank row requirement.
const ENHANCED_RANK_BONUS: i64 = 2;
/// The highlight-colour token the description rewrites (WM108 §B3 context).
const HIGHLIGHT_TOKEN: &str = "|cffffffff";

pub(crate) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetEnhancedConduit", get_enhanced_conduit)
}

fn get_enhanced_conduit(state: &mut LuaState) -> LuaResult<u32> {
    let conduit_id = signed(state, 1)?;
    let original_rank = signed(state, 2)?.max(1);
    let Some(conduit_type) = conduit_type(conduit_id) else {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    };
    let Some(adjusted_rank) = adjusted_rank(conduit_id, original_rank) else {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    };
    let Some(spell_id) = rank_spell(conduit_id, adjusted_rank) else {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    };
    let (item_level, quality) = rank_properties(original_rank);
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
    let enhanced = enhanced_color(state);
    let enhanced_label =
        global_text(state, "SOULBIND_CONDUIT_ENHANCED").unwrap_or_else(|| "Enhanced".to_string());
    append_line(state, tooltip, LINE_TYPE_PLAIN, &enhanced_label, enhanced);
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
    let description = enhanced_description(state, spell_id, enhanced);
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

/// WM108 §B2: `original + 2` with wrap, floor to 1, then clamp to the keyed-range
/// row count. Returns `None` when the conduit has no rank rows.
fn adjusted_rank(conduit_id: i32, original_rank: i32) -> Option<i32> {
    let cap = i64::from(rank_row_count(conduit_id)?);
    let adjusted = i64::from(original_rank)
        .wrapping_add(ENHANCED_RANK_BONUS)
        .max(1)
        .min(cap);
    i32::try_from(adjusted).ok()
}

/// WM108 §B2: the cap is the number of `SoulbindConduitRank` rows for the conduit.
fn rank_row_count(conduit_id: i32) -> Option<u8> {
    let conduit_id = u32::try_from(conduit_id).ok()?;
    let start = CONDUIT_RANKS.partition_point(|&(id, _, _)| id < conduit_id);
    let end = CONDUIT_RANKS.partition_point(|&(id, _, _)| id <= conduit_id);
    u8::try_from(end - start).ok().filter(|&count| count > 0)
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

/// The enhanced colour, when the named-colour surface resolves it.
fn enhanced_color(state: &mut LuaState) -> Option<(f64, f64, f64)> {
    named_color(state, "SOULBIND_CONDUIT_ENHANCED_COLOR")
}

/// WM108 §B3: the description replaces highlight-colour tokens with
/// enhanced-colour tokens. Without a resolved enhanced colour the text is
/// published unchanged.
fn enhanced_description(
    state: &mut LuaState,
    spell_id: u32,
    enhanced: Option<(f64, f64, f64)>,
) -> String {
    let description = spell_description_text(state, spell_id);
    let Some((red, green, blue)) = enhanced else {
        return description;
    };
    let token = format!(
        "|cff{:02x}{:02x}{:02x}",
        color_channel(red),
        color_channel(green),
        color_channel(blue)
    );
    description.replace(HIGHLIGHT_TOKEN, &token)
}

fn color_channel(value: f64) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}
