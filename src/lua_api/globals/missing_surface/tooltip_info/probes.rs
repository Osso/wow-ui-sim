use super::super::{
    LINE_TYPE_SPELL_NAME, TOOLTIP_TYPE_COMPANION_PET, TOOLTIP_TYPE_ITEM, TOOLTIP_TYPE_SPELL,
    TOOLTIP_TYPE_UNIT_AURA,
};
use super::builders::{
    empty_tooltip, pet_action_spell_id, push_plain_line, push_tooltip_line, tooltip_for_item_id,
};
use super::sources::{
    inbox_attachment_item_id, merchant_item_id, parse_spell_source, recipe_output_item,
    send_mail_attachment_item_id, tooltip_for_bag_item, tooltip_for_currency,
    tooltip_for_inventory_slot, tooltip_for_item_source, trade_skill_item_id, trade_slot_item_id,
};
#[cfg(not(feature = "retail-12-0-5"))]
use super::spell::lookup_player_aura_by_instance_id;
use super::spell::{
    append_action_binding_line, lookup_player_aura, spell_id_for_talent_id, tooltip_for_spell_id,
    tooltip_for_toy_item_id, tooltip_for_unit_aura,
};
use super::unit::{tooltip_for_unit, tooltip_for_unit_guid, tooltip_for_world_loot};
use crate::lua_api::globals::currency_data;
use crate::lua_api::globals::missing_surface::item_spell::{
    current_item_upgrade_location, parse_item_guid, parse_prefixed_id,
};
use crate::lua_api::globals::spellbook_data;
use crate::lua_api::methods::{borrow_state, create_string, table_get, table_set};
use crate::lua_bridge::{FromStack, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn c_tooltip_get_trait_entry(state: &mut LuaState) -> LuaResult<u32> {
    let tooltip = tooltip_for_spell_id(state, 19750);
    state.push(tooltip);
    Ok(1)
}

pub(super) fn c_tooltip_get_action(state: &mut LuaState) -> LuaResult<u32> {
    let slot = u32::from_stack(state, 1)?;
    let spell_id = borrow_state(state)?.action_bars.get(&slot).copied();
    match spell_id {
        Some(spell_id) => {
            let tooltip = tooltip_for_spell_id(state, spell_id);
            append_action_binding_line(state, tooltip, slot);
            state.push(tooltip);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

pub(super) fn c_tooltip_get_bag_item(state: &mut LuaState) -> LuaResult<u32> {
    let bag = i32::from_stack(state, 1)?;
    let slot = i32::from_stack(state, 2)?;
    let tooltip = tooltip_for_bag_item(state, bag, slot);
    state.push(tooltip);
    Ok(1)
}

pub(super) fn c_tooltip_get_currency_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let currency_id = i32::from_stack(state, 1)?;
    let amount = Option::<i32>::from_stack(state, 2)?;
    let tooltip = currency_data::get_currency_by_id(currency_id)
        .map(|currency| tooltip_for_currency(state, currency, amount))
        .unwrap_or_else(|| {
            use super::super::TOOLTIP_TYPE_CURRENCY;
            empty_tooltip(state, TOOLTIP_TYPE_CURRENCY)
        });
    state.push(tooltip);
    Ok(1)
}

pub(super) fn c_tooltip_get_currency_token(state: &mut LuaState) -> LuaResult<u32> {
    use super::super::TOOLTIP_TYPE_CURRENCY;
    let token_index = i32::from_stack(state, 1)?;
    let tooltip = currency_data::backpack_currencies()
        .nth((token_index - 1).max(0) as usize)
        .map(|currency| tooltip_for_currency(state, currency, None))
        .unwrap_or_else(|| empty_tooltip(state, TOOLTIP_TYPE_CURRENCY));
    state.push(tooltip);
    Ok(1)
}

pub(super) fn c_tooltip_get_backpack_token(state: &mut LuaState) -> LuaResult<u32> {
    c_tooltip_get_currency_token(state)
}

pub(super) fn c_tooltip_get_item(state: &mut LuaState) -> LuaResult<u32> {
    let tooltip = tooltip_for_item_source(state, stack_val(state, 1));
    state.push(tooltip);
    Ok(1)
}

#[cfg(not(feature = "retail-12-0-5"))]
pub(super) fn c_tooltip_get_item_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = u32::from_stack(state, 1)?;
    let tooltip = tooltip_for_item_id(state, item_id);
    state.push(tooltip);
    Ok(1)
}

pub(super) fn c_tooltip_get_item_by_guid(state: &mut LuaState) -> LuaResult<u32> {
    let guid = String::from_stack(state, 1)?;
    let tooltip = if let Some((_, _, item_id)) = parse_item_guid(&guid) {
        tooltip_for_item_id(state, item_id)
    } else {
        empty_tooltip(state, TOOLTIP_TYPE_ITEM)
    };
    let guid = create_string(state, &guid);
    table_set(state, tooltip, "guid", guid);
    state.push(tooltip);
    Ok(1)
}

pub(super) fn c_tooltip_get_owned_item_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = u32::from_stack(state, 1)?;
    let owned = borrow_state(state)?
        .bag_items
        .values()
        .any(|item| item.item_id == item_id);
    let tooltip = if owned {
        tooltip_for_item_id(state, item_id)
    } else {
        empty_tooltip(state, TOOLTIP_TYPE_ITEM)
    };
    state.push(tooltip);
    Ok(1)
}

pub(super) fn c_tooltip_get_recipe_reagent_item(state: &mut LuaState) -> LuaResult<u32> {
    let recipe_id = i32::from_stack(state, 1)?;
    let reagent_index = i32::from_stack(state, 2)?;
    let tooltip = trade_skill_item_id(recipe_id, Some(reagent_index))
        .map(|item_id| tooltip_for_item_id(state, item_id))
        .unwrap_or_else(|| empty_tooltip(state, TOOLTIP_TYPE_ITEM));
    state.push(tooltip);
    Ok(1)
}

pub(super) fn c_tooltip_get_recipe_result_item(state: &mut LuaState) -> LuaResult<u32> {
    let recipe_id = i32::from_stack(state, 1)?;
    let tooltip = if let Some(item_id) = recipe_output_item(recipe_id) {
        tooltip_for_item_id(state, item_id)
    } else {
        empty_tooltip(state, TOOLTIP_TYPE_ITEM)
    };
    state.push(tooltip);
    Ok(1)
}

pub(super) fn c_tooltip_get_recipe_result_item_for_order(state: &mut LuaState) -> LuaResult<u32> {
    let recipe_id = i32::from_stack(state, 1)?;
    let tooltip = if let Some(item_id) = recipe_output_item(recipe_id) {
        tooltip_for_item_id(state, item_id)
    } else {
        empty_tooltip(state, TOOLTIP_TYPE_ITEM)
    };
    state.push(tooltip);
    Ok(1)
}

pub(super) fn c_tooltip_get_trade_skill_item(state: &mut LuaState) -> LuaResult<u32> {
    let recipe_id = i32::from_stack(state, 1)?;
    let reagent_index = Option::<i32>::from_stack(state, 2)?;
    let tooltip = trade_skill_item_id(recipe_id, reagent_index)
        .map(|item_id| tooltip_for_item_id(state, item_id))
        .unwrap_or_else(|| empty_tooltip(state, TOOLTIP_TYPE_ITEM));
    state.push(tooltip);
    Ok(1)
}

pub(super) fn c_tooltip_get_trade_player_item(state: &mut LuaState) -> LuaResult<u32> {
    let slot = i32::from_stack(state, 1)?;
    let tooltip = trade_slot_item_id(state, slot, true)
        .map(|item_id| tooltip_for_item_id(state, item_id))
        .unwrap_or_else(|| empty_tooltip(state, TOOLTIP_TYPE_ITEM));
    state.push(tooltip);
    Ok(1)
}

pub(super) fn c_tooltip_get_trade_target_item(state: &mut LuaState) -> LuaResult<u32> {
    let slot = i32::from_stack(state, 1)?;
    let tooltip = trade_slot_item_id(state, slot, false)
        .map(|item_id| tooltip_for_item_id(state, item_id))
        .unwrap_or_else(|| empty_tooltip(state, TOOLTIP_TYPE_ITEM));
    state.push(tooltip);
    Ok(1)
}

pub(super) fn c_tooltip_get_merchant_item(state: &mut LuaState) -> LuaResult<u32> {
    let slot = i32::from_stack(state, 1)?;
    let tooltip = merchant_item_id(state, slot)
        .map(|item_id| tooltip_for_item_id(state, item_id))
        .unwrap_or_else(|| empty_tooltip(state, TOOLTIP_TYPE_ITEM));
    state.push(tooltip);
    Ok(1)
}

pub(super) fn c_tooltip_get_talent(state: &mut LuaState) -> LuaResult<u32> {
    let talent_id = u32::from_stack(state, 1)?;
    let tooltip = spell_id_for_talent_id(state, talent_id)
        .map(|spell_id| tooltip_for_spell_id(state, spell_id))
        .unwrap_or_else(|| empty_tooltip(state, TOOLTIP_TYPE_SPELL));
    state.push(tooltip);
    Ok(1)
}

#[cfg(not(feature = "retail-12-0-5"))]
pub(super) fn c_tooltip_get_mount_by_spell_id(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = u32::from_stack(state, 1)?;
    let tooltip = super::spell::tooltip_for_mount_spell_id(state, spell_id);
    state.push(tooltip);
    Ok(1)
}

pub(super) fn c_tooltip_get_companion_pet(state: &mut LuaState) -> LuaResult<u32> {
    let pet_id = Option::<String>::from_stack(state, 1)?;
    let tooltip = pet_id
        .as_deref()
        .and_then(|pet_id| tooltip_for_companion_pet(state, pet_id))
        .unwrap_or_else(|| empty_tooltip(state, TOOLTIP_TYPE_COMPANION_PET));
    state.push(tooltip);
    Ok(1)
}

fn tooltip_for_companion_pet(state: &mut LuaState, pet_id: &str) -> Option<Val> {
    let pet = {
        let sim = borrow_state(state).ok()?;
        sim.world
            .pets
            .iter()
            .find(|pet| pet.pet_id == pet_id)
            .cloned()
    }?;

    let saved_top = state.top;
    let tooltip = empty_tooltip(state, TOOLTIP_TYPE_COMPANION_PET);
    state.push(tooltip);
    let lines = table_get(state, tooltip, "lines");
    push_plain_line(state, lines, 1, &pet.name);
    push_plain_line(
        state,
        lines,
        2,
        &format!("Level {} Battle Pet", pet.level.max(1)),
    );
    let pet_id = create_string(state, &pet.pet_id);
    table_set(state, tooltip, "id", pet_id);
    table_set(state, tooltip, "speciesID", Val::Num(pet.species_id as f64));
    state.top = saved_top;
    Some(tooltip)
}

pub(super) fn c_tooltip_get_toy_by_item_id(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = u32::from_stack(state, 1)?;
    let tooltip = tooltip_for_toy_item_id(state, item_id);
    state.push(tooltip);
    Ok(1)
}

pub(super) fn c_tooltip_get_heirloom_by_item_id(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = u32::from_stack(state, 1)?;
    let tooltip = tooltip_for_item_id(state, item_id);
    state.push(tooltip);
    Ok(1)
}

pub(super) fn c_tooltip_get_socketed_item(state: &mut LuaState) -> LuaResult<u32> {
    use crate::lua_api::globals::missing_surface::item_socket_info;
    let tooltip = item_socket_info::socketed_item_id(state)
        .map(|item_id| tooltip_for_item_id(state, item_id))
        .unwrap_or_else(|| empty_tooltip(state, TOOLTIP_TYPE_ITEM));
    state.push(tooltip);
    Ok(1)
}

pub(super) fn c_tooltip_get_socket_gem(state: &mut LuaState) -> LuaResult<u32> {
    use crate::lua_api::globals::missing_surface::item_socket_info;
    let index = i32::from_stack(state, 1)?;
    let tooltip = item_socket_info::new_socket_item_id(state, index)
        .map(|item_id| tooltip_for_item_id(state, item_id))
        .unwrap_or_else(|| empty_tooltip(state, TOOLTIP_TYPE_ITEM));
    state.push(tooltip);
    Ok(1)
}

pub(super) fn c_tooltip_get_existing_socket_gem(state: &mut LuaState) -> LuaResult<u32> {
    use crate::lua_api::globals::missing_surface::item_socket_info;
    let index = i32::from_stack(state, 1)?;
    let tooltip = item_socket_info::existing_socket_item_id(state, index)
        .map(|item_id| tooltip_for_item_id(state, item_id))
        .unwrap_or_else(|| empty_tooltip(state, TOOLTIP_TYPE_ITEM));
    state.push(tooltip);
    Ok(1)
}

pub(super) fn c_tooltip_get_minimap_mouseover(state: &mut LuaState) -> LuaResult<u32> {
    use super::super::TOOLTIP_TYPE_MINIMAP_MOUSEOVER;
    let tooltip = empty_tooltip(state, TOOLTIP_TYPE_MINIMAP_MOUSEOVER);
    let lines = table_get(state, tooltip, "lines");
    push_tooltip_line(
        state,
        lines,
        1,
        LINE_TYPE_SPELL_NAME,
        "Stormwind City",
        None,
        false,
    );
    push_tooltip_line(
        state,
        lines,
        2,
        LINE_TYPE_SPELL_NAME,
        "Trade District",
        None,
        false,
    );
    state.push(tooltip);
    Ok(1)
}

pub(super) fn c_tooltip_get_upgrade_item(state: &mut LuaState) -> LuaResult<u32> {
    let item_id = current_item_upgrade_location(state).and_then(|(bag, slot)| {
        borrow_state(state)
            .ok()?
            .get_bag_item(bag, slot)
            .map(|(item_id, _)| item_id)
    });
    let tooltip = if let Some(item_id) = item_id {
        tooltip_for_item_id(state, item_id)
    } else {
        empty_tooltip(state, TOOLTIP_TYPE_ITEM)
    };
    state.push(tooltip);
    Ok(1)
}

pub(super) fn c_tooltip_get_inventory_item(state: &mut LuaState) -> LuaResult<u32> {
    let _unit = String::from_stack(state, 1)?;
    let slot = i32::from_stack(state, 2)?;
    let tooltip = tooltip_for_inventory_slot(state, slot);
    state.push(tooltip);
    Ok(1)
}

pub(super) fn c_tooltip_get_tooltip_data_for_item(state: &mut LuaState) -> LuaResult<u32> {
    let tooltip = tooltip_for_item_source(state, stack_val(state, 1));
    state.push(tooltip);
    Ok(1)
}

pub(super) fn c_tooltip_get_spell_book_item(state: &mut LuaState) -> LuaResult<u32> {
    let slot = i32::from_stack(state, 1)?;
    match spellbook_data::get_spell_at_slot(slot) {
        Some((_, entry, _)) => {
            let tooltip = tooltip_for_spell_id(state, entry.spell_id);
            state.push(tooltip);
        }
        None => {
            let tooltip = empty_tooltip(state, TOOLTIP_TYPE_SPELL);
            state.push(tooltip);
        }
    }
    Ok(1)
}

#[cfg(not(feature = "retail-12-0-5"))]
pub(super) fn c_tooltip_get_spell_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let spell_id = u32::from_stack(state, 1)?;
    let tooltip = tooltip_for_spell_id(state, spell_id);
    state.push(tooltip);
    Ok(1)
}

#[cfg(not(feature = "retail-12-0-5"))]
pub(super) fn c_tooltip_get_unit_buff(state: &mut LuaState) -> LuaResult<u32> {
    let _unit = String::from_stack(state, 1)?;
    let index = i32::from_stack(state, 2)?;
    let aura = lookup_player_aura(state, index);
    let tooltip = tooltip_for_unit_aura(state, aura);
    state.push(tooltip);
    Ok(1)
}

#[cfg(not(feature = "retail-12-0-5"))]
pub(super) fn c_tooltip_get_unit_buff_by_aura_instance_id(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let aura_instance_id = i32::from_stack(state, 2)?;
    let aura = if unit == "player" {
        lookup_player_aura_by_instance_id(state, aura_instance_id)
    } else {
        None
    };
    let tooltip = tooltip_for_unit_aura(state, aura);
    state.push(tooltip);
    Ok(1)
}

#[cfg(not(feature = "retail-12-0-5"))]
pub(super) fn c_tooltip_get_unit_debuff(state: &mut LuaState) -> LuaResult<u32> {
    let _unit = String::from_stack(state, 1)?;
    let _index = i32::from_stack(state, 2)?;
    let tooltip = empty_tooltip(state, TOOLTIP_TYPE_UNIT_AURA);
    state.push(tooltip);
    Ok(1)
}

#[cfg(not(feature = "retail-12-0-5"))]
pub(super) fn c_tooltip_get_unit_debuff_by_aura_instance_id(
    state: &mut LuaState,
) -> LuaResult<u32> {
    let _unit = String::from_stack(state, 1)?;
    let _aura_instance_id = i32::from_stack(state, 2)?;
    let tooltip = empty_tooltip(state, TOOLTIP_TYPE_UNIT_AURA);
    state.push(tooltip);
    Ok(1)
}

pub(super) fn c_tooltip_get_unit_aura(state: &mut LuaState) -> LuaResult<u32> {
    crate::c_api::unit_aura_access::require_unit_aura_access(state, "C_TooltipInfo.GetUnitAura")?;
    let _unit = String::from_stack(state, 1)?;
    let index = i32::from_stack(state, 2)?;
    let filter = String::from_stack(state, 3).unwrap_or_default();
    let tooltip = if filter.eq_ignore_ascii_case("HARMFUL") {
        empty_tooltip(state, TOOLTIP_TYPE_UNIT_AURA)
    } else {
        let aura = lookup_player_aura(state, index);
        tooltip_for_unit_aura(state, aura)
    };
    state.push(tooltip);
    Ok(1)
}

#[cfg(not(feature = "retail-12-0-5"))]
pub(super) fn c_tooltip_get_unit_aura_by_aura_instance_id(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let aura_instance_id = i32::from_stack(state, 2)?;
    let aura = if unit == "player" {
        lookup_player_aura_by_instance_id(state, aura_instance_id)
    } else {
        None
    };
    let tooltip = tooltip_for_unit_aura(state, aura);
    state.push(tooltip);
    Ok(1)
}

pub(super) fn c_tooltip_get_hyperlink(state: &mut LuaState) -> LuaResult<u32> {
    let link = String::from_stack(state, 1)?;
    let tooltip = if let Some(item_id) = parse_prefixed_id(&link, "item") {
        tooltip_for_item_id(state, item_id)
    } else if let Some(spell_id) = parse_prefixed_id(&link, "spell") {
        tooltip_for_spell_id(state, spell_id)
    } else if let Some(guid) = link.strip_prefix("unit:") {
        tooltip_for_unit_guid(state, guid).unwrap_or(Val::Nil)
    } else {
        Val::Nil
    };
    state.push(tooltip);
    Ok(1)
}

pub(super) fn c_tooltip_get_inbox_item(state: &mut LuaState) -> LuaResult<u32> {
    let message_index = i32::from_stack(state, 1)?;
    let attachment_index = Option::<i32>::from_stack(state, 2)?;
    let tooltip = inbox_attachment_item_id(state, message_index, attachment_index)
        .map(|item_id| tooltip_for_item_id(state, item_id))
        .unwrap_or_else(|| empty_tooltip(state, TOOLTIP_TYPE_ITEM));
    state.push(tooltip);
    Ok(1)
}

pub(super) fn c_tooltip_get_send_mail_item(state: &mut LuaState) -> LuaResult<u32> {
    let attachment_index = Option::<i32>::from_stack(state, 1)?;
    let tooltip = send_mail_attachment_item_id(state, attachment_index)
        .map(|item_id| tooltip_for_item_id(state, item_id))
        .unwrap_or_else(|| empty_tooltip(state, TOOLTIP_TYPE_ITEM));
    state.push(tooltip);
    Ok(1)
}

pub(super) fn c_tooltip_get_spell(state: &mut LuaState) -> LuaResult<u32> {
    let tooltip = parse_spell_source(state, stack_val(state, 1))
        .map(|spell_id| tooltip_for_spell_id(state, spell_id))
        .unwrap_or_else(|| empty_tooltip(state, TOOLTIP_TYPE_SPELL));
    state.push(tooltip);
    Ok(1)
}

pub(super) fn c_tooltip_get_world_cursor(state: &mut LuaState) -> LuaResult<u32> {
    let tooltip = tooltip_for_world_loot(state);
    state.push(tooltip);
    Ok(1)
}

pub(super) fn c_tooltip_get_world_loot_object(state: &mut LuaState) -> LuaResult<u32> {
    let _unit = String::from_stack(state, 1)?;
    let tooltip = tooltip_for_world_loot(state);
    state.push(tooltip);
    Ok(1)
}

pub(super) fn c_tooltip_get_unit(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1)?;
    let tooltip = tooltip_for_unit(state, &unit);
    state.push(tooltip);
    Ok(1)
}

pub(super) fn c_tooltip_get_achievement_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let achievement_id = i32::from_stack(state, 1)?;
    let name = borrow_state(state)?
        .achievements
        .get(&achievement_id)
        .map(|a| a.name.clone());
    let Some(name) = name else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let tooltip = empty_tooltip(state, TOOLTIP_TYPE_SPELL);
    state.push(tooltip);
    let lines = table_get(state, tooltip, "lines");
    push_plain_line(state, lines, 1, &name);
    Ok(1)
}

pub(super) fn c_tooltip_get_aura(state: &mut LuaState) -> LuaResult<u32> {
    let _unit = String::from_stack(state, 1)?;
    let index = i32::from_stack(state, 2)?;
    let filter = Option::<String>::from_stack(state, 3)?.unwrap_or_default();
    let tooltip = if filter.eq_ignore_ascii_case("HARMFUL") {
        empty_tooltip(state, TOOLTIP_TYPE_UNIT_AURA)
    } else {
        let aura = lookup_player_aura(state, index);
        tooltip_for_unit_aura(state, aura)
    };
    state.push(tooltip);
    Ok(1)
}

pub(super) fn c_tooltip_get_guild_bank_item(state: &mut LuaState) -> LuaResult<u32> {
    let _tab = i32::from_stack(state, 1)?;
    let _slot = i32::from_stack(state, 2)?;
    state.push(Val::Nil);
    Ok(1)
}

pub(super) fn c_tooltip_get_instance_lock_encounters_complete(
    state: &mut LuaState,
) -> LuaResult<u32> {
    let _difficulty_id = Option::<i32>::from_stack(state, 1)?;
    let _lock_id = Option::<i32>::from_stack(state, 2)?;
    state.push(Val::Nil);
    Ok(1)
}

pub(super) fn c_tooltip_get_lfg_dungeon(state: &mut LuaState) -> LuaResult<u32> {
    let _dungeon_id = Option::<i32>::from_stack(state, 1)?;
    state.push(Val::Nil);
    Ok(1)
}

pub(super) fn c_tooltip_get_pet_action(state: &mut LuaState) -> LuaResult<u32> {
    let slot = i32::from_stack(state, 1)?;
    match pet_action_spell_id(state, slot) {
        Some(spell_id) => {
            let tooltip = tooltip_for_spell_id(state, spell_id);
            state.push(tooltip);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

pub(super) fn c_tooltip_get_quest_currency(state: &mut LuaState) -> LuaResult<u32> {
    let _quest_id = Option::<i32>::from_stack(state, 1)?;
    let _currency_type = Option::<i32>::from_stack(state, 2)?;
    let _index = Option::<i32>::from_stack(state, 3)?;
    state.push(Val::Nil);
    Ok(1)
}

pub(super) fn c_tooltip_get_quest_item(state: &mut LuaState) -> LuaResult<u32> {
    let _quest_type = Option::<String>::from_stack(state, 1)?;
    let _slot = Option::<i32>::from_stack(state, 2)?;
    state.push(Val::Nil);
    Ok(1)
}

pub(super) fn c_tooltip_get_quest_log_currency(state: &mut LuaState) -> LuaResult<u32> {
    let _currency_type = Option::<i32>::from_stack(state, 1)?;
    let _index = Option::<i32>::from_stack(state, 2)?;
    state.push(Val::Nil);
    Ok(1)
}

pub(super) fn c_tooltip_get_quest_log_item(state: &mut LuaState) -> LuaResult<u32> {
    let _quest_type = Option::<String>::from_stack(state, 1)?;
    let _slot = Option::<i32>::from_stack(state, 2)?;
    state.push(Val::Nil);
    Ok(1)
}

pub(super) fn c_tooltip_get_shapeshift(state: &mut LuaState) -> LuaResult<u32> {
    let slot = i32::from_stack(state, 1)?;
    let form = {
        let sim = borrow_state(state)?;
        let zero_based = usize::try_from(slot.saturating_sub(1)).unwrap_or(usize::MAX);
        sim.shapeshift_forms.get(zero_based).cloned()
    };
    let Some(form) = form else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let tooltip = empty_tooltip(state, TOOLTIP_TYPE_SPELL);
    state.push(tooltip);
    let lines = table_get(state, tooltip, "lines");
    push_plain_line(state, lines, 1, &form.name);
    table_set(state, tooltip, "id", Val::Num(form.spell_id as f64));
    Ok(1)
}

pub(super) fn c_tooltip_get_trainer_service(state: &mut LuaState) -> LuaResult<u32> {
    let _service_index = Option::<i32>::from_stack(state, 1)?;
    state.push(Val::Nil);
    Ok(1)
}
