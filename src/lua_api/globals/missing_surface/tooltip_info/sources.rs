use super::super::{
    LINE_TYPE_SPELL_DESCRIPTION, LINE_TYPE_SPELL_NAME, TOOLTIP_TYPE_CURRENCY, TOOLTIP_TYPE_ITEM,
};
use super::builders::{empty_tooltip, item_quality_color, push_tooltip_line, tooltip_for_item_id};
use crate::lua_api::globals::missing_surface::item_spell::parse_prefixed_id;
use crate::lua_api::globals::{currency_data, profession_data};
use crate::lua_api::methods::{borrow_state, table_get, table_set, val_to_string};
use rilua::Val;
use rilua::vm::state::LuaState;

pub(super) fn tooltip_for_item_source(state: &mut LuaState, value: Val) -> Val {
    match value {
        Val::Table(location) => tooltip_for_item_location(state, Val::Table(location)),
        other => {
            let item_id = parse_item_source_id(state, other);
            tooltip_for_optional_item_id(state, item_id)
        }
    }
}

fn tooltip_for_item_location(state: &mut LuaState, location: Val) -> Val {
    if let Some(slot) = table_i32_field(state, location, "equipmentSlotIndex") {
        return tooltip_for_inventory_slot(state, slot);
    }

    let bag = table_i32_field(state, location, "bagID");
    let slot = table_i32_field(state, location, "slotIndex");
    match (bag, slot) {
        (Some(bag), Some(slot)) => tooltip_for_bag_item(state, bag, slot),
        _ => empty_tooltip(state, TOOLTIP_TYPE_ITEM),
    }
}

fn table_i32_field(state: &mut LuaState, table: Val, field: &str) -> Option<i32> {
    match table_get(state, table, field) {
        Val::Num(value) => Some(value as i32),
        _ => None,
    }
}

fn tooltip_for_optional_item_id(state: &mut LuaState, item_id: Option<u32>) -> Val {
    item_id
        .map(|item_id| tooltip_for_item_id(state, item_id))
        .unwrap_or_else(|| empty_tooltip(state, TOOLTIP_TYPE_ITEM))
}

fn parse_item_source_id(state: &mut LuaState, value: Val) -> Option<u32> {
    match value {
        Val::Num(value) if value > 0.0 => Some(value as u32),
        Val::Str(_) => val_to_string(state, value).and_then(|text| parse_item_id_text(&text)),
        _ => None,
    }
}

fn parse_item_id_text(text: &str) -> Option<u32> {
    parse_prefixed_id(text, "item")
        .or_else(|| text.strip_prefix("item:")?.split(':').next()?.parse().ok())
        .or_else(|| text.parse().ok())
}

pub(super) fn parse_spell_source(state: &mut LuaState, value: Val) -> Option<u32> {
    match value {
        Val::Num(value) if value > 0.0 => Some(value as u32),
        Val::Str(_) => val_to_string(state, value).and_then(|text| {
            parse_prefixed_id(&text, "spell")
                .or_else(|| text.strip_prefix("spell:")?.split(':').next()?.parse().ok())
                .or_else(|| text.parse().ok())
        }),
        _ => None,
    }
}

pub(super) fn tooltip_for_bag_item(state: &mut LuaState, bag: i32, slot: i32) -> Val {
    let item_id = borrow_state(state)
        .ok()
        .and_then(|st| st.get_bag_item(bag, slot).map(|(item_id, _)| item_id))
        .unwrap_or(0);
    tooltip_for_item_id(state, item_id)
}

pub(super) fn tooltip_for_inventory_slot(state: &mut LuaState, slot: i32) -> Val {
    let item_id = borrow_state(state)
        .ok()
        .and_then(|st| st.player.equipped_items.get(&slot).map(|item| item.item_id))
        .unwrap_or(0);
    tooltip_for_item_id(state, item_id)
}

pub(super) fn tooltip_for_currency(
    state: &mut LuaState,
    currency: &currency_data::CurrencyEntry,
    amount_override: Option<i32>,
) -> Val {
    let saved_top = state.top;
    let tooltip = empty_tooltip(state, TOOLTIP_TYPE_CURRENCY);
    state.push(tooltip);
    let lines = table_get(state, tooltip, "lines");
    let amount = amount_override.unwrap_or(currency.quantity);
    let max_display = if currency.max_quantity > 0 {
        format!("{amount} / {}", currency.max_quantity)
    } else {
        amount.to_string()
    };
    push_tooltip_line(
        state,
        lines,
        1,
        LINE_TYPE_SPELL_NAME,
        currency.name,
        Some(item_quality_color(currency.quality as u8)),
        false,
    );
    push_tooltip_line(
        state,
        lines,
        2,
        LINE_TYPE_SPELL_DESCRIPTION,
        &format!("Amount: {max_display}"),
        None,
        false,
    );
    table_set(state, tooltip, "id", Val::Num(currency.currency_id as f64));
    table_set(state, tooltip, "quantity", Val::Num(amount as f64));
    state.top = saved_top;
    tooltip
}

pub(super) fn inbox_attachment_item_id(
    state: &mut LuaState,
    message_index: i32,
    attachment_index: Option<i32>,
) -> Option<u32> {
    let zero_based_message = usize::try_from(message_index.saturating_sub(1)).ok()?;
    let zero_based_attachment =
        usize::try_from(attachment_index.unwrap_or(1).saturating_sub(1)).ok()?;
    let st = borrow_state(state).ok()?;
    st.player
        .inbox
        .get(zero_based_message)?
        .items
        .get(zero_based_attachment)
        .map(|item| item.item_id)
}

pub(super) fn send_mail_attachment_item_id(
    state: &mut LuaState,
    attachment_index: Option<i32>,
) -> Option<u32> {
    let zero_based = usize::try_from(attachment_index.unwrap_or(1).saturating_sub(1)).ok()?;
    let st = borrow_state(state).ok()?;
    st.player
        .send_mail_items
        .get(zero_based)
        .and_then(|item| item.as_ref())
        .map(|item| item.item_id)
}

pub(super) fn merchant_item_id(state: &LuaState, slot: i32) -> Option<u32> {
    let zero_based = usize::try_from(slot.saturating_sub(1)).ok()?;
    borrow_state(state)
        .ok()?
        .merchant_items
        .get(zero_based)
        .copied()
        .filter(|item_id| *item_id != 0)
}

pub(super) fn trade_slot_item_id(
    state: &mut LuaState,
    slot: i32,
    player_side: bool,
) -> Option<u32> {
    let zero_based = usize::try_from(slot.saturating_sub(1)).ok()?;
    let st = borrow_state(state).ok()?;
    let trade = st.active_trade.as_ref()?;
    let item_id = if player_side {
        *trade.player_slots.get(zero_based)?
    } else {
        *trade.target_slots.get(zero_based)?
    };
    (item_id != 0).then_some(item_id)
}

pub(super) fn recipe_output_item(recipe_id: i32) -> Option<u32> {
    profession_data::get_recipe(recipe_id)
        .and_then(|recipe| (recipe.output_item_id != 0).then_some(recipe.output_item_id))
}

pub(super) fn trade_skill_item_id(recipe_id: i32, reagent_index: Option<i32>) -> Option<u32> {
    let recipe = profession_data::get_recipe(recipe_id)?;
    let reagent_index = reagent_index.unwrap_or(0);
    if reagent_index <= 0 {
        return (recipe.output_item_id != 0).then_some(recipe.output_item_id);
    }
    let zero_based = usize::try_from(reagent_index.saturating_sub(1)).ok()?;
    recipe
        .reagents
        .get(zero_based)
        .map(|reagent| reagent.item_id)
}
