//! R82 (70688) hyperlink service. Native UNKNOWN paths publish no fabricated rows.
//! PTR/Forever reuse of the Retail routing contract is INFERRED.
mod catalog;
pub mod inputs;
pub mod parse;

use crate::items;
use crate::lua_api::globals::missing_surface::tooltip_info::{
    append_line, compose_tooltip, tooltip_for_currency, tooltip_for_item_context_with_sell_price,
    tooltip_for_item_hyperlink, tooltip_for_link_spell,
};
use crate::lua_api::methods::borrow_state;
use crate::lua_bridge::FromStack;
use parse::{HyperlinkRequest, Marker};
use rilua::{LuaResult, Val, vm::state::LuaState};

enum HyperlinkOutput {
    Tooltip(Val),
    False,
    Unknown,
    AlreadyPublished(u32),
}

pub(crate) fn get_hyperlink(state: &mut LuaState) -> LuaResult<u32> {
    let Some(request) = parse_hyperlink_request(state)? else {
        return crate::c_api::tooltip_data::unknown_getter_failure();
    };
    match route_hyperlink(state, request)? {
        HyperlinkOutput::Tooltip(tooltip) => {
            state.push(tooltip);
            Ok(1)
        }
        HyperlinkOutput::False => {
            state.push(Val::Bool(false));
            Ok(1)
        }
        HyperlinkOutput::Unknown => crate::c_api::tooltip_data::unknown_getter_failure(),
        HyperlinkOutput::AlreadyPublished(results) => Ok(results),
    }
}

fn parse_hyperlink_request(state: &mut LuaState) -> LuaResult<Option<HyperlinkRequest>> {
    let text = String::from_stack(state, 1)?;
    let first = Option::<i32>::from_stack(state, 2)?;
    let second = Option::<i32>::from_stack(state, 3)?;
    let hide_price = Option::<bool>::from_stack(state, 4)?.unwrap_or(false);
    // The initializer's pre-existing second field is zero in our request model.
    Ok(parse::parse(&text, first, second, hide_price, 0))
}

fn route_hyperlink(state: &mut LuaState, request: HyperlinkRequest) -> LuaResult<HyperlinkOutput> {
    match request {
        HyperlinkRequest::Item(request) => Ok(HyperlinkOutput::Tooltip(
            tooltip_for_item_hyperlink(state, &request.link, request.hide_vendor_price)?,
        )),
        HyperlinkRequest::Spell(request) => Ok(HyperlinkOutput::Tooltip(tooltip_for_link_spell(
            state,
            request.spell_id,
        ))),
        HyperlinkRequest::Keystone { item_id } => Ok(keystone_hyperlink_tooltip(state, item_id)?
            .map_or(HyperlinkOutput::Unknown, HyperlinkOutput::Tooltip)),
        HyperlinkRequest::Currency { id, quantity } => {
            Ok(currency_hyperlink_tooltip(state, id, quantity)?
                .map_or(HyperlinkOutput::Unknown, HyperlinkOutput::Tooltip))
        }
        // R82 proves the service's false result, not a synchronous server response.
        HyperlinkRequest::Trade { .. } => Ok(HyperlinkOutput::False),
        other => route_special_hyperlink(state, other),
    }
}

fn route_special_hyperlink(
    state: &mut LuaState,
    request: HyperlinkRequest,
) -> LuaResult<HyperlinkOutput> {
    let output = match request {
        HyperlinkRequest::Single {
            marker: Marker::Quest,
            id,
        } => quest_hyperlink_tooltip(state, id)?
            .map_or(HyperlinkOutput::Unknown, HyperlinkOutput::Tooltip),
        HyperlinkRequest::Single {
            marker: Marker::PvpTalent,
            id,
        } => pvp_talent_hyperlink_tooltip(state, id)?
            .map_or(HyperlinkOutput::Unknown, HyperlinkOutput::Tooltip),
        HyperlinkRequest::AzeriteEssence { id, rank } => {
            essence_hyperlink_tooltip(state, id, rank)?
                .map_or(HyperlinkOutput::Unknown, HyperlinkOutput::Tooltip)
        }
        // Producer-specific descriptor rows are composed by the hyperlink route.
        other => HyperlinkOutput::AlreadyPublished(power_route(state, other)?),
    };
    Ok(output)
}

fn keystone_hyperlink_tooltip(state: &mut LuaState, item_id: u32) -> LuaResult<Option<Val>> {
    if crate::client_profile::ACTIVE != crate::client_profile::ClientProfile::Retail {
        return Ok(None);
    }
    // R88aw: the keystone route passes mode zero and leaves preview clear.
    let item = items::get_item(item_id);
    Ok(Some(tooltip_for_item_context_with_sell_price(
        state, item_id, item,
    )))
}

fn currency_hyperlink_tooltip(
    state: &mut LuaState,
    id: u32,
    quantity: i32,
) -> LuaResult<Option<Val>> {
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::WowForever
    ) {
        let Ok(currency_id) = i32::try_from(id) else {
            return Ok(None);
        };
        return crate::c_api::c_tooltip_info_currency::build_currency_tooltip(
            state,
            currency_id,
            Some(quantity),
        );
    }
    non_retail_currency_hyperlink_tooltip(state, id, quantity)
}

fn non_retail_currency_hyperlink_tooltip(
    state: &mut LuaState,
    id: u32,
    quantity: i32,
) -> LuaResult<Option<Val>> {
    let Some(currency) = i32::try_from(id)
        .ok()
        .and_then(crate::lua_api::globals::currency_data::get_currency_by_id)
    else {
        return Ok(None);
    };
    let tooltip = tooltip_for_currency(state, currency, Some(quantity));
    Ok(Some(tooltip))
}

fn quest_hyperlink_tooltip(state: &mut LuaState, id: u32) -> LuaResult<Option<Val>> {
    let title = borrow_state(state)?
        .quest_log_entries
        .entries
        .iter()
        .find(|entry| entry.quest_id as u32 == id)
        .map(|entry| entry.title.clone());
    let Some(title) = title else {
        return Ok(None);
    };
    let tooltip = compose_tooltip(state, 23.0, (id != 0).then_some(f64::from(id)));
    append_line(state, tooltip, 0.0, &title, None);
    Ok(Some(tooltip))
}

fn pvp_talent_hyperlink_tooltip(state: &mut LuaState, id: u32) -> LuaResult<Option<Val>> {
    let Some(spell_id) = pvp_talent_spell(state, id)? else {
        return Ok(None);
    };
    // R82 secondaryFlag=true suppresses all grant/inspect annotations.
    Ok(Some(tooltip_for_link_spell(state, spell_id)))
}

fn essence_hyperlink_tooltip(state: &mut LuaState, id: u32, rank: u32) -> LuaResult<Option<Val>> {
    let input = borrow_state(state)?
        .hyperlink_tooltip
        .essence_spells
        .get(&(id, rank))
        .cloned();
    let Some(input) = input else {
        return Ok(None);
    };
    let tooltip = compose_tooltip(state, 8.0, Some(f64::from(id)));
    append_line(state, tooltip, 0.0, &input.main_heading, None);
    append_spell_lines(state, tooltip, input.main_spell_id);
    append_line(state, tooltip, 0.0, &input.passive_heading, None);
    append_spell_lines(state, tooltip, input.passive_spell_id);
    Ok(Some(tooltip))
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

fn power_route(state: &mut LuaState, request: HyperlinkRequest) -> LuaResult<u32> {
    let tooltip = match request {
        HyperlinkRequest::ArtifactPower { id, base, total } => {
            catalog::artifact(state, id, base, total)?
        }
        HyperlinkRequest::Conduit { id, rank } => catalog::conduit(state, id, rank)?,
        _ => None,
    };
    match tooltip {
        Some(tooltip) => {
            state.push(tooltip);
            Ok(1)
        }
        None => crate::c_api::tooltip_data::unknown_getter_failure(),
    }
}

fn append_spell_lines(state: &mut LuaState, tooltip: Val, spell_id: u32) {
    use crate::lua_api::methods::{table_get, table_set_num};
    let saved_top = state.top;
    state.push(tooltip);
    let spell = tooltip_for_link_spell(state, spell_id);
    state.push(spell);
    let source = table_get(state, spell, "lines");
    let destination = table_get(state, tooltip, "lines");
    let (Val::Table(source), Val::Table(destination)) = (source, destination) else {
        state.top = saved_top;
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
    state.top = saved_top;
}
