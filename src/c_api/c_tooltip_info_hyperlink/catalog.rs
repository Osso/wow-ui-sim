//! R82/R82b prefix composition. Catalog bindings are INFERRED from
//! the existing 70391 export; this module is isolated so that binding is droppable.
use crate::c_api::tooltip_power_catalog_data::{
    ARTIFACT_POWER_RANKS, ARTIFACT_POWERS, CONDUIT_RANK_PROPERTIES, CONDUIT_RANKS, CONDUIT_TYPES,
};
use crate::lua_api::globals::missing_surface::tooltip_info::{
    append_line, compose_tooltip, format_global_numeric, format_global_pair, global_text,
    item_quality_color, named_color, restore_plain_line_type, spell_description_text,
};
use crate::lua_api::methods::{borrow_state, table_set};
use rilua::{LuaResult, Val, vm::state::LuaState};

pub(super) fn artifact(
    state: &mut LuaState,
    id: u32,
    base: u8,
    total: u8,
) -> LuaResult<Option<Val>> {
    let Some(&(_, _, _, flags, _)) = ARTIFACT_POWERS.iter().find(|row| row.0 == id) else {
        return Ok(None);
    };
    let Some(display) = borrow_state(state)?
        .hyperlink_tooltip
        .artifact_display
        .get(&id)
        .cloned()
    else {
        return Ok(None);
    };
    let ranks = super::parse::artifact_ranks(flags, base, total, &display);
    let Some(&(_, _, spell_id)) = ARTIFACT_POWER_RANKS
        .iter()
        .find(|row| row.0 == id && row.1 == ranks.lookup_rank - 1)
    else {
        return Ok(None);
    };
    let saved_top = state.top;
    let tooltip = compose_tooltip(state, 1.0, None);
    state.push(tooltip);
    let highlight = named_color(state, "HIGHLIGHT_FONT_COLOR");
    if let Some(spell) = crate::spells::get_spell(spell_id) {
        let line = append_line(state, tooltip, 0.0, spell.name, highlight);
        restore_plain_line_type(state, line);
    }
    if flags & (64 | 4) == 0 {
        let text = format_global_pair(
            state,
            "TOOLTIP_TALENT_RANK",
            f64::from(ranks.displayed_total),
            f64::from(ranks.displayed_maximum),
        )?;
        let colour = if ranks.adjusted_bonus != 0 {
            named_color(state, "INCREASE_STAT_COLOR")
        } else {
            highlight
        };
        let line = append_line(state, tooltip, 0.0, &text, colour);
        restore_plain_line_type(state, line);
        append_line(state, tooltip, 1.0, " ", None);
    }
    append_description(state, tooltip, spell_id);
    state.top = saved_top;
    Ok(Some(tooltip))
}

pub(super) fn conduit(state: &mut LuaState, id: u32, rank: i32) -> LuaResult<Option<Val>> {
    let Some(&(_, conduit_type)) = CONDUIT_TYPES.iter().find(|row| row.0 == id) else {
        return Ok(None);
    };
    let Ok(rank_index) = u8::try_from(rank - 1) else {
        return Ok(None);
    };
    let Some(&(_, _, spell_id)) = CONDUIT_RANKS
        .iter()
        .find(|row| row.0 == id && row.1 == rank_index)
    else {
        return Ok(None);
    };
    let saved_top = state.top;
    let tooltip = compose_tooltip(state, 13.0, None);
    state.push(tooltip);
    // R82 normal conduit path calls no identity setter.
    table_set(state, tooltip, "type", Val::Nil);
    if let Some(spell) = crate::spells::get_spell(spell_id) {
        let Some(&(_, level, quality)) = CONDUIT_RANK_PROPERTIES
            .iter()
            .find(|row| i32::from(row.0) == rank)
        else {
            return Ok(None);
        };
        let line = append_line(
            state,
            tooltip,
            0.0,
            spell.name,
            Some(item_quality_color(quality)),
        );
        restore_plain_line_type(state, line);
        let key = match conduit_type {
            0 => "CONDUIT_TYPE_FINESSE",
            1 => "CONDUIT_TYPE_POTENCY",
            2 => "CONDUIT_TYPE_ENDURANCE",
            _ => return Ok(None),
        };
        if let Some(label) = global_text(state, key) {
            let line = append_line(state, tooltip, 0.0, &label, None);
            restore_plain_line_type(state, line);
        }
        let text = format_global_numeric(state, "ITEM_LEVEL", f64::from(level))?;
        let line = append_line(state, tooltip, 31.0, &text, None);
        table_set(state, line, "itemLevel", Val::Num(f64::from(level)));
    }
    append_description(state, tooltip, spell_id);
    state.top = saved_top;
    Ok(Some(tooltip))
}

fn append_description(state: &mut LuaState, tooltip: Val, spell_id: u32) {
    // Shared producer is changing in R81. Keep the existing bounded composition,
    // not guessed cost/learn/footer rows. Full common spell rows await integration.
    let description = spell_description_text(state, spell_id);
    if !description.is_empty() {
        let colour = named_color(state, "GREEN_FONT_COLOR");
        append_line(state, tooltip, 34.0, &description, colour);
    }
}
