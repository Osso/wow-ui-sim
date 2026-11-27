//! Shared guild-tabard and bank storage globals.

use crate::c_api::ensure_namespace;
use crate::items;
use crate::lua_api::methods::{borrow_state, borrow_state_mut, create_string};
use crate::lua_api::state::BagItem;
use crate::lua_bridge::{FromStack, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, RustFn, Val};

const MAX_BANK_BAG_SLOTS: i32 = 7;

pub fn register_guild_tabard_files(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "GetGuildTabardFiles", get_guild_tabard_files)?;
    Ok(())
}

pub fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {
    register_bank_globals(lua)?;
    register_guild_bank_globals(lua)?;
    register_c_guild_bank(lua.state_mut())?;
    Ok(())
}

fn register_bank_globals(lua: &mut rilua::Lua) -> crate::Result<()> {
    LuaApiMut::register_function(lua, "GetNumBankSlots", get_num_bank_slots)?;
    LuaApiMut::register_function(lua, "GetBankSlotCost", get_bank_slot_cost)?;
    LuaApiMut::register_function(lua, "PurchaseSlot", purchase_slot)?;
    LuaApiMut::register_function(
        lua,
        "BankButtonIDToInvSlotID",
        bank_button_id_to_inv_slot_id,
    )?;
    Ok(())
}

/// Legacy guild-bank globals. Every name in `GUILD_BANK_GLOBALS` is a `_G`
/// row in `data/secure-publication/retail-12-1-5-70688.csv` and is called by
/// the original Retail and Forever `Blizzard_GuildBankUI/Mainline` consumer
/// (money: `Blizzard_GuildBankUI.lua:204`, `MoneyFrame.lua:164`). The pinned
/// `Blizzard_APIDocumentationGenerated` trees document none of them.
pub fn register_guild_bank_globals(lua: &mut rilua::Lua) -> crate::Result<()> {
    for (name, function) in GUILD_BANK_GLOBALS {
        LuaApiMut::register_function(lua, name, *function)?;
    }
    Ok(())
}

const GUILD_BANK_GLOBALS: &[(&str, RustFn)] = &[
    ("GetCurrentGuildBankTab", get_current_guild_bank_tab),
    ("SetCurrentGuildBankTab", set_current_guild_bank_tab),
    ("GetNumGuildBankTabs", get_num_guild_bank_tabs),
    ("GetGuildBankTabInfo", get_guild_bank_tab_info),
    ("GetGuildBankTabCost", get_guild_bank_tab_cost),
    ("GetGuildBankText", get_guild_bank_text),
    ("SetGuildBankText", set_guild_bank_text),
    ("QueryGuildBankTab", query_guild_bank_tab),
    ("GetGuildBankItemInfo", get_guild_bank_item_info),
    ("GetGuildBankItemLink", get_guild_bank_item_link),
    ("PickupGuildBankItem", pickup_guild_bank_item),
    ("GetGuildBankMoney", get_guild_bank_money),
    ("GetGuildBankWithdrawMoney", get_guild_bank_withdraw_money),
    (
        "GetGuildBankWithdrawGoldLimit",
        get_guild_bank_withdraw_gold_limit,
    ),
    ("CanGuildBankRepair", can_guild_bank_repair),
    ("CanEditGuildTabInfo", can_edit_guild_tab_info),
    (
        "SetGuildBankWithdrawGoldLimit",
        set_guild_bank_withdraw_gold_limit,
    ),
    ("DepositGuildBankMoney", deposit_guild_bank_money),
    ("WithdrawGuildBankMoney", withdraw_guild_bank_money),
    ("PickupGuildBankMoney", pickup_guild_bank_money),
    ("CanWithdrawGuildBankMoney", can_withdraw_guild_bank_money),
    ("CanEditGuildBankTabInfo", can_edit_guild_bank_tab_info),
];

fn register_c_guild_bank(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_GuildBank")?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "IsGuildBankEnabled",
        c_guild_bank_is_enabled,
    )
}

fn get_num_bank_slots(state: &mut LuaState) -> LuaResult<u32> {
    let slots = borrow_state(state)?.bank_purchased_slots;
    state.push(Val::Num(slots as f64));
    state.push(Val::Bool(slots >= MAX_BANK_BAG_SLOTS));
    Ok(2)
}

fn get_bank_slot_cost(state: &mut LuaState) -> LuaResult<u32> {
    let purchased_slots = i32::from_stack(state, 1).unwrap_or(0);
    let cost = 1_000 * i64::from(purchased_slots + 1);
    state.push(Val::Num(cost as f64));
    Ok(1)
}

fn purchase_slot(state: &mut LuaState) -> LuaResult<u32> {
    let mut sim = borrow_state_mut(state)?;
    sim.bank_purchased_slots = (sim.bank_purchased_slots + 1).min(MAX_BANK_BAG_SLOTS);
    Ok(0)
}

fn bank_button_id_to_inv_slot_id(state: &mut LuaState) -> LuaResult<u32> {
    let button_id = i32::from_stack(state, 1).unwrap_or(0);
    let is_bag = bool::from_stack(state, 2).unwrap_or(false);
    let inv_slot_id = if is_bag {
        bank_bag_inventory_slot_id(button_id)
    } else {
        button_id
    };
    state.push(Val::Num(inv_slot_id as f64));
    Ok(1)
}

fn bank_bag_inventory_slot_id(button_id: i32) -> i32 {
    68 + button_id
}

fn c_guild_bank_is_enabled(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(true));
    Ok(1)
}

fn get_current_guild_bank_tab(state: &mut LuaState) -> LuaResult<u32> {
    let tab = borrow_state(state)?.current_guild_bank_tab.max(1);
    state.push(Val::Num(tab as f64));
    Ok(1)
}

fn set_current_guild_bank_tab(state: &mut LuaState) -> LuaResult<u32> {
    let tab = i32::from_stack(state, 1).unwrap_or(1).max(1);
    borrow_state_mut(state)?.current_guild_bank_tab = tab;
    Ok(0)
}

fn get_num_guild_bank_tabs(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.guild_bank_tabs.len();
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn get_guild_bank_tab_info(state: &mut LuaState) -> LuaResult<u32> {
    let tab = i32::from_stack(state, 1)?;
    let Some(index) = usize::try_from(tab).ok().and_then(|tab| tab.checked_sub(1)) else {
        return Ok(0);
    };
    let entry = borrow_state(state)?.guild_bank_tabs.get(index).cloned();
    let Some(entry) = entry else { return Ok(0) };
    let name = create_string(state, &entry.name);
    let icon = create_string(state, &entry.icon);
    state.push(name);
    state.push(icon);
    state.push(Val::Bool(entry.is_viewable));
    state.push(Val::Bool(entry.can_deposit));
    state.push(Val::Num(entry.withdrawal_limit as f64));
    state.push(Val::Num(entry.remaining_withdrawals as f64));
    state.push(Val::Bool(entry.is_filtered));
    Ok(7)
}

fn get_guild_bank_tab_cost(state: &mut LuaState) -> LuaResult<u32> {
    let cost = borrow_state(state)?.guild_bank_tab_cost;
    state.push(Val::Num(cost as f64));
    Ok(1)
}

fn get_guild_bank_text(state: &mut LuaState) -> LuaResult<u32> {
    let tab = i32::from_stack(state, 1).unwrap_or(1);
    let text = borrow_state(state)?
        .guild_bank_text
        .get(&tab)
        .cloned()
        .unwrap_or_default();
    let text = create_string(state, &text);
    state.push(text);
    Ok(1)
}

fn set_guild_bank_text(state: &mut LuaState) -> LuaResult<u32> {
    #[cfg(feature = "client-mists")]
    {
        let _ = state;
        return Ok(0);
    }
    #[cfg(not(feature = "client-mists"))]
    {
        let tab = i32::from_stack(state, 1)?;
        let text = String::from_stack(state, 2)?;
        borrow_state_mut(state)?.guild_bank_text.insert(tab, text);
        Ok(0)
    }
}

fn query_guild_bank_tab(state: &mut LuaState) -> LuaResult<u32> {
    set_current_guild_bank_tab(state)
}

fn get_guild_bank_item_info(state: &mut LuaState) -> LuaResult<u32> {
    let tab = i32::from_stack(state, 1)?;
    let slot = i32::from_stack(state, 2)?;
    let Some(item) = guild_bank_item(state, tab, slot)? else {
        return Ok(0);
    };
    let item_info = items::get_item(item.item_id);
    let texture = item_info
        .map(|info| info.icon_file_data_id)
        .filter(|texture| *texture != 0)
        .unwrap_or(134400);
    let quality = item_info.map(|info| info.quality).unwrap_or(0);
    state.push(Val::Num(texture as f64));
    state.push(Val::Num(item.stack_count as f64));
    state.push(Val::Bool(false));
    state.push(Val::Bool(false));
    state.push(Val::Num(quality as f64));
    Ok(5)
}

fn guild_bank_item(state: &LuaState, tab: i32, slot: i32) -> LuaResult<Option<BagItem>> {
    Ok(borrow_state(state)?
        .guild_bank_items
        .get(&(tab, slot))
        .cloned())
}

fn get_guild_bank_item_link(state: &mut LuaState) -> LuaResult<u32> {
    let tab = i32::from_stack(state, 1)?;
    let slot = i32::from_stack(state, 2)?;
    let link = guild_bank_item(state, tab, slot)?
        .and_then(|item| crate::c_api::item_spell::item_link_for_id(item.item_id));
    match link {
        Some(link) => {
            let link = create_string(state, &link);
            state.push(link);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn pickup_guild_bank_item(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn get_guild_bank_money(state: &mut LuaState) -> LuaResult<u32> {
    let money = borrow_state(state)?.guild_bank_money;
    state.push(Val::Num(money as f64));
    Ok(1)
}

#[cfg(not(feature = "client-mists"))]
fn guild_bank_money_limit(state: &LuaState) -> LuaResult<Option<i64>> {
    let sim = borrow_state(state)?;
    let selected = sim.world.guild_selected_rank.checked_sub(1);
    let index = selected.and_then(|rank| usize::try_from(rank).ok());
    Ok(index
        .and_then(|index| sim.world.guild_bank_rank_permissions.get(&index))
        .map(|permissions| permissions.money_per_day))
}

fn get_guild_bank_withdraw_money(state: &mut LuaState) -> LuaResult<u32> {
    #[cfg(feature = "client-mists")]
    let _ = state;
    #[cfg(feature = "client-mists")]
    let limit = -1;
    #[cfg(not(feature = "client-mists"))]
    let limit = guild_bank_money_limit(state)?.unwrap_or(0);
    state.push(Val::Num(limit as f64));
    Ok(1)
}

fn get_guild_bank_withdraw_gold_limit(state: &mut LuaState) -> LuaResult<u32> {
    #[cfg(feature = "client-mists")]
    let _ = state;
    #[cfg(feature = "client-mists")]
    let gold = 0;
    #[cfg(not(feature = "client-mists"))]
    let gold = guild_bank_money_limit(state)?.unwrap_or(0);
    let gold = if gold < 0 { gold } else { gold / 10_000 };
    state.push(Val::Num(gold as f64));
    Ok(1)
}

fn set_guild_bank_withdraw_gold_limit(state: &mut LuaState) -> LuaResult<u32> {
    #[cfg(feature = "client-mists")]
    {
        let _ = state;
        return Ok(0);
    }
    #[cfg(not(feature = "client-mists"))]
    {
        let gold = i64::from_stack(state, 1)?;
        let selected = borrow_state(state)?
            .world
            .guild_selected_rank
            .checked_sub(1);
        if let Some(index) = selected.and_then(|rank| usize::try_from(rank).ok()) {
            if let Some(permissions) = borrow_state_mut(state)?
                .world
                .guild_bank_rank_permissions
                .get_mut(&index)
            {
                permissions.money_per_day = gold.saturating_mul(10_000);
            }
        }
        Ok(0)
    }
}

fn deposit_guild_bank_money(state: &mut LuaState) -> LuaResult<u32> {
    let amount = i64::from_stack(state, 1).unwrap_or(0).max(0);
    let mut sim = borrow_state_mut(state)?;
    sim.guild_bank_money += amount;
    sim.player.money -= amount;
    Ok(0)
}

fn withdraw_guild_bank_money(state: &mut LuaState) -> LuaResult<u32> {
    let amount = i64::from_stack(state, 1).unwrap_or(0).max(0);
    let mut sim = borrow_state_mut(state)?;
    let withdrawn = sim.guild_bank_money.min(amount);
    sim.guild_bank_money -= withdrawn;
    sim.player.money += withdrawn;
    Ok(0)
}

fn pickup_guild_bank_money(state: &mut LuaState) -> LuaResult<u32> {
    withdraw_guild_bank_money(state)
}

fn can_withdraw_guild_bank_money(state: &mut LuaState) -> LuaResult<u32> {
    #[cfg(feature = "client-mists")]
    let _ = state;
    #[cfg(feature = "client-mists")]
    let allowed = true;
    #[cfg(not(feature = "client-mists"))]
    let allowed = guild_bank_money_limit(state)?.is_some_and(|limit| limit != 0);
    state.push(Val::Bool(allowed));
    Ok(1)
}

fn can_guild_bank_repair(state: &mut LuaState) -> LuaResult<u32> {
    let allowed = borrow_state(state)?.guild_bank_can_repair;
    state.push(Val::Bool(allowed));
    Ok(1)
}

fn can_edit_guild_bank_tab_info(state: &mut LuaState) -> LuaResult<u32> {
    let tab = borrow_state(state)?.current_guild_bank_tab;
    can_edit_guild_tab(state, tab)
}

fn can_edit_guild_tab_info(state: &mut LuaState) -> LuaResult<u32> {
    let tab = i32::from_stack(state, 1).unwrap_or(1);
    can_edit_guild_tab(state, tab)
}

fn can_edit_guild_tab(state: &mut LuaState, tab: i32) -> LuaResult<u32> {
    #[cfg(feature = "client-mists")]
    let allowed = true;
    #[cfg(not(feature = "client-mists"))]
    let allowed = {
        let sim = borrow_state(state)?;
        let selected = sim.world.guild_selected_rank.checked_sub(1);
        let index = selected.and_then(|rank| usize::try_from(rank).ok());
        let tab = usize::try_from(tab.saturating_sub(1)).ok();
        index
            .and_then(|index| sim.world.guild_bank_rank_permissions.get(&index))
            .zip(tab)
            .is_some_and(|(permissions, tab)| {
                permissions
                    .tab_rights
                    .get(tab)
                    .is_some_and(|rights| rights & 0x4 != 0)
            })
    };
    state.push(Val::Bool(allowed));
    Ok(1)
}

fn get_guild_tabard_files(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(180159.0));
    state.push(Val::Num(180158.0));
    state.push(Val::Nil);
    state.push(Val::Nil);
    state.push(Val::Nil);
    state.push(Val::Nil);
    Ok(6)
}
