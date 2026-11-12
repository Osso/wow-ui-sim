//! R82 (70688) hyperlink service. Native UNKNOWN paths publish no fabricated rows.
//! PTR/Forever reuse of the Retail routing contract is INFERRED.
pub mod inputs;
pub mod parse;

use crate::lua_api::globals::missing_surface::tooltip_info::{
    append_line, compose_tooltip, restore_plain_line_type, tooltip_for_currency,
    tooltip_for_item_hyperlink, tooltip_for_link_spell,
};
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::FromStack;
use parse::{HyperlinkRequest, Marker};
use rilua::{LuaResult, Val, vm::state::LuaState};

pub(crate) fn get_hyperlink(state: &mut LuaState) -> LuaResult<u32> {
    let text = String::from_stack(state, 1)?;
    let first = Option::<i32>::from_stack(state, 2)?;
    let second = Option::<i32>::from_stack(state, 3)?;
    let hide_price = Option::<bool>::from_stack(state, 4)?.unwrap_or(false);
    // The initializer's pre-existing second field is zero in our request model.
    let Some(request) = parse::parse(&text, first, second, hide_price, 0) else {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    };
    let tooltip = match request {
        HyperlinkRequest::Item(request) => tooltip_for_item_hyperlink(state, &request.link)?,
        HyperlinkRequest::Spell(request) => tooltip_for_link_spell(state, request.spell_id),
        HyperlinkRequest::Currency { id, quantity } => {
            let Some(currency) = i32::try_from(id)
                .ok()
                .and_then(crate::lua_api::globals::currency_data::get_currency_by_id)
            else {
                return crate::c_api::tooltip_data::unknown_getter_failure();
            };
            let tooltip = tooltip_for_currency(state, currency, Some(quantity));
            // R82 Q3 currency title is type 0, not a spell-name line.
            let lines = crate::lua_api::methods::table_get(state, tooltip, "lines");
            if let Val::Table(lines) = lines {
                let line = state
                    .gc
                    .tables
                    .get(lines)
                    .map(|lines| lines.get_int(1))
                    .unwrap_or(Val::Nil);
                restore_plain_line_type(state, line);
            }
            tooltip
        }
        // R82 proves the service's false result, not a synchronous server response.
        HyperlinkRequest::Trade { .. } => {
            state.push(Val::Bool(false));
            return Ok(1);
        }
        HyperlinkRequest::Single {
            marker: Marker::Quest,
            id,
        } => {
            let title = borrow_state(state)?
                .quest_log_entries
                .entries
                .iter()
                .find(|entry| entry.quest_id as u32 == id)
                .map(|entry| entry.title.clone());
            let Some(title) = title else {
                return crate::c_api::tooltip_data::unknown_getter_failure();
            };
            let tooltip = compose_tooltip(state, 23.0, (id != 0).then_some(f64::from(id)));
            let line = append_line(state, tooltip, 0.0, &title, None);
            restore_plain_line_type(state, line);
            tooltip
        }
        HyperlinkRequest::Single {
            marker: Marker::PvpTalent,
            id,
        } => {
            let Some(spell_id) = pvp_talent_spell(state, id)? else {
                return crate::c_api::tooltip_data::unknown_getter_failure();
            };
            // R82 secondaryFlag=true suppresses all grant/inspect annotations.
            tooltip_for_link_spell(state, spell_id)
        }
        HyperlinkRequest::AzeriteEssence { id, rank } => {
            let Some(input) = borrow_state(state)?
                .hyperlink_tooltip
                .essence_spells
                .get(&(id, rank))
                .cloned()
            else {
                return crate::c_api::tooltip_data::unknown_getter_failure();
            };
            let tooltip = compose_tooltip(state, 8.0, Some(f64::from(id)));
            let line = append_line(state, tooltip, 0.0, &input.main_heading, None);
            restore_plain_line_type(state, line);
            append_spell_lines(state, tooltip, input.main_spell_id);
            let line = append_line(state, tooltip, 0.0, &input.passive_heading, None);
            restore_plain_line_type(state, line);
            append_spell_lines(state, tooltip, input.passive_spell_id);
            tooltip
        }
        // Producer-specific descriptor rows are composed by the hyperlink route.
        other => return power_route(state, other),
    };
    state.push(tooltip);
    Ok(1)
}

fn pvp_talent_spell(state: &LuaState, id: u32) -> LuaResult<Option<u32>> {
    #[cfg(feature = "client-retail")]
    {
        return Ok(borrow_state(state)?
            .pvp_talent_tooltip
            .catalog
            .get(&(id as i32))
            .map(|row| row.spell_id));
    }
    #[cfg(not(feature = "client-retail"))]
    {
        let _ = (state, id);
        Ok(None)
    }
}

fn power_route(_state: &mut LuaState, _request: HyperlinkRequest) -> LuaResult<u32> {
    // No fallback: unknown row providers and no-producer routes stay unmodelled.
    crate::c_api::tooltip_data::unknown_getter_failure()
}

fn append_spell_lines(state: &mut LuaState, tooltip: Val, spell_id: u32) {
    use crate::lua_api::methods::{table_get, table_set_num};
    let spell = tooltip_for_link_spell(state, spell_id);
    let source = table_get(state, spell, "lines");
    let destination = table_get(state, tooltip, "lines");
    let (Val::Table(source), Val::Table(destination)) = (source, destination) else {
        return;
    };
    let rows: Vec<_> = (1..)
        .map_while(|index| {
            let row = state.gc.tables.get(source)?.get_int(index);
            (row != Val::Nil).then_some(row)
        })
        .collect();
    let mut index = 1;
    while state
        .gc
        .tables
        .get(destination)
        .is_some_and(|rows| rows.get_int(index) != Val::Nil)
    {
        index += 1;
    }
    for row in rows {
        table_set_num(state, destination, index as f64, row);
        index += 1;
    }
}
