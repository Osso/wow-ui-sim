use super::super::{LINE_TYPE_SPELL_NAME, TOOLTIP_TYPE_CURRENCY, TOOLTIP_TYPE_ITEM};
use super::builders::{
    RetailSellPriceRequest, empty_tooltip, item_quality_color, push_tooltip_line,
    stamp_data_instance, tooltip_for_item_id, tooltip_for_item_id_with_request,
    tooltip_for_item_info_with_request,
};
use crate::lua_api::globals::missing_surface::item_spell::parse_prefixed_id;
use crate::lua_api::globals::{currency_data, profession_data};
use crate::lua_api::methods::{borrow_state, table_get, table_set, val_to_string};
use rilua::LuaResult;
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

pub(super) fn tooltip_for_item_link(
    state: &mut LuaState,
    parsed: &crate::c_api::item_descriptor::ParsedItemLink,
    hide_vendor_price: bool,
) -> LuaResult<Val> {
    if !item_link_level_resolution_enabled() {
        return Ok(tooltip_for_item_id(state, parsed.descriptor.item_id));
    }
    let descriptor = &parsed.descriptor;
    let Some(item) = resolve_item_link_info(state, descriptor)? else {
        return Ok(empty_tooltip(state, TOOLTIP_TYPE_ITEM));
    };
    Ok(tooltip_for_item_info_with_request(
        state,
        descriptor.item_id,
        &item,
        RetailSellPriceRequest::without_instance(hide_vendor_price),
    ))
}

fn item_link_level_resolution_enabled() -> bool {
    matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail | crate::client_profile::ClientProfile::Ptr
    )
}

fn resolve_item_link_info(
    state: &mut LuaState,
    descriptor: &crate::c_api::item_descriptor::ItemDescriptor,
) -> LuaResult<Option<crate::items::ItemInfo>> {
    let player_level = item_link_player_level(state)?;
    descriptor
        .resolve(player_level)
        .map_err(|error| rilua::runtime_error(format!("C_TooltipInfo.GetHyperlink: {error}")))
}

fn item_link_player_level(state: &mut LuaState) -> LuaResult<i32> {
    let model = borrow_state(state)?;
    let player = &model.player;
    Ok(if player.effective_level != 0 {
        player.effective_level
    } else {
        player.level
    })
}

fn tooltip_for_item_location(state: &mut LuaState, location: Val) -> Val {
    if let Some(slot) = table_i32_field(state, location, "equipmentSlotIndex") {
        return tooltip_for_inventory_slot(state, slot);
    }

    let bag = table_i32_field(state, location, "bagID");
    let slot = table_i32_field(state, location, "slotIndex");
    match (bag, slot) {
        (Some(bag), Some(slot)) => compose_bag_item(state, bag, slot),
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
    #[cfg(feature = "client-retail")]
    if let Some(tooltip) =
        crate::c_api::c_tooltip_info_battle_pet::tooltip_for_bag_item(state, bag, slot)
    {
        return tooltip;
    }
    let tooltip = compose_bag_item_with_request(
        state,
        bag,
        slot,
        Some(RetailSellPriceRequest::bag_item(bag, slot)),
    );
    // VERIFIED R88n.4b (Retail 12.1.5.70688): ordinary bag tooltips append
    // a top-level integer repairCost only for a positive queried cost.
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Retail {
        let cost = supplied_bag_repair_cost(state, bag, slot);
        if let Some(cost) = cost.filter(|cost| *cost > 0) {
            let saved_top = state.top;
            state.push(tooltip);
            table_set(state, tooltip, "repairCost", Val::Num(f64::from(cost)));
            state.top = saved_top;
        }
    }
    stamp_bag_item_guid(state, tooltip, bag, slot);
    tooltip
}

/// INFERRED R88g9 (native evidence pending): the bag tooltip carries the
/// item's GUID as top-level `guid`, the field that `TooltipUtil.GetDisplayedItem`
/// (Blizzard_SharedXMLGame/Tooltip/TooltipUtil.lua:9-19) resolves through
/// `C_Item.GetItemLinkByGUID`. Same GUID as `GetInventoryItemByID` tooltips.
fn stamp_bag_item_guid(state: &mut LuaState, tooltip: Val, bag: i32, slot: i32) {
    use crate::client_profile::{ACTIVE, ClientProfile};
    if !matches!(ACTIVE, ClientProfile::Retail | ClientProfile::WowForever) {
        return;
    }
    let item_id = borrow_state(state)
        .ok()
        .and_then(|sim| sim.get_bag_item(bag, slot).map(|(item_id, _)| item_id));
    let Some(item_id) = item_id.filter(|item_id| *item_id > 0) else {
        return;
    };
    let guid = crate::c_api::item_spell::item_guid_for_bag_slot(bag, slot, item_id);
    let saved_top = state.top;
    state.push(tooltip);
    let guid = crate::lua_api::methods::create_string(state, &guid);
    table_set(state, tooltip, "guid", guid);
    state.top = saved_top;
}

fn supplied_bag_repair_cost(state: &LuaState, bag: i32, slot: i32) -> Option<i32> {
    let sim = borrow_state(state).ok()?;
    sim.get_bag_item(bag, slot)?;
    sim.bag_item_repair_costs.get(&(bag, slot)).copied()
}

fn compose_bag_item(state: &mut LuaState, bag: i32, slot: i32) -> Val {
    compose_bag_item_with_request(state, bag, slot, None)
}

fn compose_bag_item_with_request(
    state: &mut LuaState,
    bag: i32,
    slot: i32,
    request: Option<RetailSellPriceRequest>,
) -> Val {
    let item_id = borrow_state(state)
        .ok()
        .and_then(|st| st.get_bag_item(bag, slot).map(|(item_id, _)| item_id))
        .unwrap_or(0);
    match request {
        Some(request) => tooltip_for_item_id_with_request(state, item_id, request),
        None => tooltip_for_item_id(state, item_id),
    }
}

pub(super) fn tooltip_for_inventory_slot(state: &mut LuaState, slot: i32) -> Val {
    let item_id = borrow_state(state)
        .ok()
        .and_then(|st| st.player.equipped_items.get(&slot).map(|item| item.item_id))
        .unwrap_or(0);
    tooltip_for_item_id(state, item_id)
}

pub(crate) fn tooltip_for_currency(
    state: &mut LuaState,
    currency: &currency_data::CurrencyEntry,
    amount_override: Option<i32>,
) -> Val {
    let saved_top = state.top;
    let tooltip = empty_tooltip(state, TOOLTIP_TYPE_CURRENCY);
    state.push(tooltip);
    stamp_data_instance(state, tooltip);
    let lines = table_get(state, tooltip, "lines");
    push_tooltip_line(
        state,
        lines,
        1,
        LINE_TYPE_SPELL_NAME,
        currency.name,
        Some(item_quality_color(currency.quality as u8)),
        false,
    );
    // WM97 DQ6: quantity is composition data (zero for offer costs), not a
    // separate wrapper amount text line. Flag-derived lines still need WM96 evidence.
    let quantity = amount_override.unwrap_or(currency.quantity);
    table_set(state, tooltip, "id", Val::Num(currency.currency_id as f64));
    table_set(state, tooltip, "quantity", Val::Num(quantity as f64));
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
