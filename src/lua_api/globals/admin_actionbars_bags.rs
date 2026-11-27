//! Rilua A_Admin handlers — Action bars, Bags.
//!
//! Extracted from rilua_admin_world.rs per the 750-line file cap and to keep
//! each sub-module focused on a single concern. The parent entry
//! point in admin.rs imports these as pub(super) and weaves
//! them into the A_Admin TableBuilder chain.

use crate::lua_api::methods::borrow_state_mut;
use crate::lua_api::script_helpers::fire_named_event_state;
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use crate::lua_api::state::PossessActionSlot;
use crate::lua_api::state::{GuildBankRankPermissions, PetActionSlot};
use crate::lua_bridge::{FromStack, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val, runtime_error};

// ── Action bars ───────────────────────────────────────────────────────────────

pub(super) fn set_possess_bar_visible(state: &mut LuaState) -> LuaResult<u32> {
    let visible = bool::from_stack(state, 1)?;
    borrow_state_mut(state)?
        .action_bar_state
        .possess_bar_visible = visible;
    Ok(0)
}

pub(super) fn set_action_slot(state: &mut LuaState) -> LuaResult<u32> {
    let slot = u32::from_stack(state, 1)?;
    let spell_id = u32::from_stack(state, 2)?;
    let mut state = borrow_state_mut(state)?;
    crate::c_api::action_macros::clear_slot(&mut state, slot);
    state.action_bars.insert(slot, spell_id);
    Ok(0)
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
pub(super) fn set_possess_action_slot(state: &mut LuaState) -> LuaResult<u32> {
    let slot = u32::from_stack(state, 1)?;
    let texture = String::from_stack(state, 2)?;
    let spell_id = u32::from_stack(state, 3)?;
    let enabled = bool::from_stack(state, 4)?;
    let Some(index) = slot
        .checked_sub(1)
        .and_then(|slot| usize::try_from(slot).ok())
    else {
        return Ok(0);
    };
    let mut sim = borrow_state_mut(state)?;
    if let Some(entry) = sim.action_bar_state.possess_slots.get_mut(index) {
        *entry = Some(PossessActionSlot {
            texture,
            spell_id,
            enabled,
        });
    }
    Ok(0)
}

pub(super) fn set_pet_action_slot(state: &mut LuaState) -> LuaResult<u32> {
    let slot = u32::from_stack(state, 1)?;
    let name = String::from_stack(state, 2)?;
    let texture = String::from_stack(state, 3)?;
    let spell_id = Option::<u32>::from_stack(state, 4)?;
    let passive = Option::<bool>::from_stack(state, 5)?.unwrap_or(false);
    let Some(index) = slot
        .checked_sub(1)
        .and_then(|slot| usize::try_from(slot).ok())
    else {
        return Ok(0);
    };

    let mut sim = borrow_state_mut(state)?;
    if let Some(action) = sim.pet_actions.get_mut(index) {
        *action = PetActionSlot {
            has_action: true,
            name: Some(name),
            texture: Some(texture),
            is_token: false,
            is_active: false,
            auto_cast_allowed: !passive,
            auto_cast_enabled: false,
            spell_id,
            passive,
            cooldown: None,
            packed_action: 0,
        };
    }
    drop(sim);

    fire_named_event_state(state, "PET_UI_UPDATE", &[]);
    fire_named_event_state(state, "PET_BAR_UPDATE", &[]);
    Ok(0)
}

pub(super) fn clear_action_slot(state: &mut LuaState) -> LuaResult<u32> {
    let slot = u32::from_stack(state, 1)?;
    let mut state = borrow_state_mut(state)?;
    crate::c_api::action_macros::clear_slot(&mut state, slot);
    Ok(0)
}

pub(super) fn clear_action_bars(state: &mut LuaState) -> LuaResult<u32> {
    let mut state = borrow_state_mut(state)?;
    state.action_macros.clear();
    state.action_bars.clear();
    state.action_outfits.clear();
    state.equipped_gear_outfit_action_slots.clear();
    Ok(0)
}

// ── Bags ──────────────────────────────────────────────────────────────────────

pub(super) fn add_bag_item(state: &mut LuaState) -> LuaResult<u32> {
    use crate::lua_api::state::BagItem;
    let bag = i32::from_stack(state, 1)?;
    let slot = i32::from_stack(state, 2)?;
    let item_id = u32::from_stack(state, 3)?;
    let stack = Option::<i32>::from_stack(state, 4)?;
    let mut sim = borrow_state_mut(state)?;
    crate::c_api::item_upgrade_eligibility::discard_location(
        &mut sim,
        crate::c_api::item_location::ItemSlot::Bag(bag, slot),
    );
    sim.item_tooltip_socket_projections.remove(&(bag, slot));
    sim.bag_items.insert(
        (bag, slot),
        BagItem {
            item_id,
            stack_count: stack.unwrap_or(1),
            hyperlink: None,
            durability: None,
            applied_transmog: None,
            owned_identity: None,
        },
    );
    Ok(0)
}

pub(super) fn remove_bag_item(state: &mut LuaState) -> LuaResult<u32> {
    let bag = i32::from_stack(state, 1)?;
    let slot = i32::from_stack(state, 2)?;
    let mut sim = borrow_state_mut(state)?;
    crate::c_api::item_upgrade_eligibility::discard_location(
        &mut sim,
        crate::c_api::item_location::ItemSlot::Bag(bag, slot),
    );
    sim.bag_items.remove(&(bag, slot));
    sim.item_tooltip_socket_projections.remove(&(bag, slot));
    Ok(0)
}

pub(super) fn clear_bags(state: &mut LuaState) -> LuaResult<u32> {
    let mut sim = borrow_state_mut(state)?;
    crate::c_api::item_upgrade_eligibility::discard_bags(&mut sim);
    sim.bag_items.clear();
    sim.item_tooltip_socket_projections.clear();
    Ok(0)
}

pub(super) fn set_merchant_items(state: &mut LuaState) -> LuaResult<u32> {
    let items = match stack_val(state, 1) {
        Val::Table(items_ref) => state
            .gc
            .tables
            .get(items_ref)
            .map(|table| {
                table
                    .array_slice()
                    .iter()
                    .filter_map(|item| match item {
                        Val::Num(value) if *value > 0.0 => Some(*value as u32),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default(),
        Val::Num(value) if value > 0.0 => vec![value as u32],
        _ => Vec::new(),
    };

    borrow_state_mut(state)?.merchant_items = items;
    Ok(0)
}

pub(super) fn add_guild_bank_item(state: &mut LuaState) -> LuaResult<u32> {
    use crate::lua_api::state::BagItem;
    let tab = i32::from_stack(state, 1)?;
    let slot = i32::from_stack(state, 2)?;
    let item_id = u32::from_stack(state, 3)?;
    let stack = Option::<i32>::from_stack(state, 4)?;
    borrow_state_mut(state)?.guild_bank_items.insert(
        (tab, slot),
        BagItem {
            item_id,
            stack_count: stack.unwrap_or(1),
            hyperlink: None,
            durability: None,
            applied_transmog: None,
            owned_identity: None,
        },
    );
    Ok(0)
}

pub(super) fn add_guild_bank_tab(state: &mut LuaState) -> LuaResult<u32> {
    use crate::lua_api::state::GuildBankTab;
    let tab = GuildBankTab {
        name: String::from_stack(state, 1)?,
        icon: String::from_stack(state, 2)?,
        is_viewable: bool::from_stack(state, 3)?,
        can_deposit: bool::from_stack(state, 4)?,
        withdrawal_limit: i32::from_stack(state, 5)?,
        remaining_withdrawals: i32::from_stack(state, 6)?,
        is_filtered: bool::from_stack(state, 7)?,
    };
    borrow_state_mut(state)?.guild_bank_tabs.push(tab);
    Ok(0)
}

pub(super) fn clear_guild_bank_tabs(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.guild_bank_tabs.clear();
    Ok(0)
}

pub(super) fn set_guild_bank_tab_cost(state: &mut LuaState) -> LuaResult<u32> {
    let cost = i64::from_stack(state, 1)?;
    if cost < 0 {
        return Err(runtime_error("Guild bank tab cost must be nonnegative"));
    }
    borrow_state_mut(state)?.guild_bank_tab_cost = cost;
    fire_named_event_state(state, "GUILDBANK_UPDATE_TABS", &[]);
    Ok(0)
}

fn guild_bank_permission_values(state: &LuaState, argument: i32) -> LuaResult<Vec<i32>> {
    let Val::Table(reference) = stack_val(state, argument) else {
        return Err(runtime_error("Guild bank permissions must be tables"));
    };
    let values = state
        .gc
        .tables
        .get(reference)
        .map(|table| {
            table
                .array_slice()
                .iter()
                .filter_map(|value| match value {
                    Val::Num(number)
                        if number.is_finite()
                            && number.fract() == 0.0
                            && *number >= i32::MIN as f64
                            && *number <= i32::MAX as f64 =>
                    {
                        Some(*number as i32)
                    }
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(values)
}

pub(super) fn set_guild_bank_rank_permissions(state: &mut LuaState) -> LuaResult<u32> {
    let rank = i32::from_stack(state, 1)?;
    let money_per_day = i64::from_stack(state, 2)?;
    let can_repair = bool::from_stack(state, 5)?;
    let rights = guild_bank_permission_values(state, 3)?;
    let stacks = guild_bank_permission_values(state, 4)?;
    if !(1..=10).contains(&rank) || rights.len() != 6 || stacks.len() != 6 {
        return Err(runtime_error("Invalid guild bank rank permission input"));
    }
    if rights.iter().any(|value| !(0..=7).contains(value)) || stacks.iter().any(|value| *value < 0)
    {
        return Err(runtime_error("Invalid guild bank rank permission values"));
    }
    let mut sim = borrow_state_mut(state)?;
    let rank = usize::try_from(rank).expect("validated positive rank");
    sim.world.guild_selected_rank = rank as i32;
    sim.guild_bank_can_repair = can_repair;
    sim.world.guild_bank_rank_permissions.insert(
        rank - 1,
        GuildBankRankPermissions {
            money_per_day,
            tab_rights: rights.try_into().expect("validated six rights"),
            tab_stacks_per_day: stacks.try_into().expect("validated six stacks"),
        },
    );
    drop(sim);
    fire_named_event_state(state, "GUILDBANK_UPDATE_WITHDRAWMONEY", &[]);
    fire_named_event_state(state, "GUILDBANK_UPDATE_TABS", &[]);
    Ok(0)
}

/// Supplies the copper amount returned by `GetGuildBankMoney()`.
/// INFERRED R88g9 (native evidence pending): a guild-money change reaches the
/// UI as `GUILDBANK_UPDATE_MONEY`, the event the original `MoneyFrame.lua:158-165`
/// GUILDBANK money type and `Blizzard_GuildBankUI.lua:21` register for.
pub(super) fn set_guild_bank_money(state: &mut LuaState) -> LuaResult<u32> {
    let copper = i64::from_stack(state, 1)?;
    borrow_state_mut(state)?.guild_bank_money = copper;
    fire_named_event_state(state, "GUILDBANK_UPDATE_MONEY", &[]);
    Ok(0)
}

pub(super) fn clear_guild_bank(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.guild_bank_items.clear();
    Ok(0)
}
