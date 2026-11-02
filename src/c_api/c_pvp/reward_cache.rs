//! Supplied reward caches; their native refresh lifecycle is not modeled.

use crate::lua_api::methods::{
    borrow_state, create_string, create_table, table_set, table_set_num,
};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val};

#[derive(Debug, Clone, Default)]
pub struct PvpRewardCache {
    pub honor: i64,
    pub experience: i64,
    pub items: Vec<PvpRewardItem>,
    pub currencies: Vec<PvpRewardCurrency>,
}

#[derive(Debug, Clone)]
pub struct PvpRewardItem {
    pub id: i64,
    pub name: String,
    pub texture: i64,
    pub quantity: i64,
}

#[derive(Debug, Clone)]
pub struct PvpRewardCurrency {
    pub id: i64,
    pub quantity: i64,
}

#[derive(Debug, Clone, Default)]
pub struct PvpRewardCaches {
    pub random_bg: PvpRewardCache,
    pub arena_skirmish: PvpRewardCache,
    pub random_epic_bg: PvpRewardCache,
}

pub(super) fn register_random_bg(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetRandomBGRewards", random_bg_rewards)
}

pub(super) fn register_arena(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetArenaSkirmishRewards", arena_rewards)
}

fn random_bg_rewards(state: &mut LuaState) -> LuaResult<u32> {
    let cache = borrow_state(state)?.pvp_reward_caches.random_bg.clone();
    push_rewards(state, &cache)
}

fn arena_rewards(state: &mut LuaState) -> LuaResult<u32> {
    let cache = borrow_state(state)?
        .pvp_reward_caches
        .arena_skirmish
        .clone();
    push_rewards(state, &cache)
}

fn push_rewards(state: &mut LuaState, cache: &PvpRewardCache) -> LuaResult<u32> {
    state.push(Val::Num(cache.honor as f64));
    state.push(Val::Num(cache.experience as f64));
    push_items(state, &cache.items);
    push_currencies(state, &cache.currencies);
    state.push(Val::Nil);
    Ok(5)
}

fn push_items(state: &mut LuaState, items: &[PvpRewardItem]) {
    if items.is_empty() {
        state.push(Val::Nil);
        return;
    }
    let array = create_table(state);
    state.push(array);
    let Val::Table(table) = array else {
        unreachable!()
    };
    for (index, item) in items.iter().enumerate() {
        let row = create_table(state);
        table_set_num(state, table, (index + 1) as f64, row);
        table_set(state, row, "id", Val::Num(item.id as f64));
        let name = create_string(state, &item.name);
        table_set(state, row, "name", name);
        table_set(state, row, "texture", Val::Num(item.texture as f64));
        table_set(state, row, "quantity", Val::Num(item.quantity as f64));
    }
}

fn push_currencies(state: &mut LuaState, currencies: &[PvpRewardCurrency]) {
    if currencies.is_empty() {
        state.push(Val::Nil);
        return;
    }
    let array = create_table(state);
    state.push(array);
    let Val::Table(table) = array else {
        unreachable!()
    };
    for (index, currency) in currencies.iter().enumerate() {
        let row = create_table(state);
        table_set_num(state, table, (index + 1) as f64, row);
        table_set(state, row, "id", Val::Num(currency.id as f64));
        table_set(state, row, "quantity", Val::Num(currency.quantity as f64));
    }
}
