//! Shared loot state boundary; validation and master resolution are inferred policies.

use crate::c_api::c_chat_info::reject_chat_messaging_lockdown;
use crate::event::Event;
use crate::lua_api::methods::{borrow_state, borrow_state_mut};
use crate::lua_api::state::SimState;
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use rilua::table_security::is_secret_value;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val, runtime_error};

// Numeric Enum.LootMethod assignments from missing_enums.lua, not availability order.
const TOKENS: [&str; 6] = [
    "freeforall",
    "roundrobin",
    "master",
    "group",
    "needbeforegreed",
    "personalloot",
];

pub(super) fn register(state: &mut LuaState, namespace: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, namespace, "GetLootMethod", get_loot_method)?;
    table_set_rust_fn_static(state, namespace, "SetLootMethod", set_loot_method)
}

fn get_loot_method(state: &mut LuaState) -> LuaResult<u32> {
    let (method, party, raid) = {
        let sim = borrow_state(state)?;
        let loot = &sim.loot_method;
        (
            encode_method(&loot.method)?,
            nullable_index(loot.party_master_index),
            nullable_index(loot.raid_master_index),
        )
    };
    state.push(Val::Num(method as f64));
    state.push(party);
    state.push(raid);
    Ok(3)
}

fn encode_method(token: &str) -> LuaResult<usize> {
    TOKENS
        .iter()
        .position(|candidate| *candidate == token)
        .ok_or_else(|| runtime_error(format!("GetLootMethod: unknown shared token {token:?}")))
}

fn nullable_index(index: i32) -> Val {
    if index > 0 {
        Val::Num(f64::from(index))
    } else {
        Val::Nil
    }
}

fn set_loot_method(state: &mut LuaState) -> LuaResult<u32> {
    reject_chat_messaging_lockdown(state, "SetLootMethod")?;
    let method = read_method(state)?;
    let master = read_master(state)?;
    let success = {
        let mut sim = borrow_state_mut(state)?;
        match resolve_indices(&sim, method, master.as_deref()) {
            Some((party, raid)) => {
                apply_selection(&mut sim, TOKENS[method], party, raid);
                true
            }
            None => false,
        }
    };
    state.push(Val::Bool(success));
    Ok(1)
}

fn reject_secret(state: &LuaState, value: Val) -> LuaResult<()> {
    // Conservative rejection, not cached AllowedWhenUntainted parity.
    if is_secret_value(state, value) {
        return Err(runtime_error(
            "SetLootMethod: secret argument access is not modeled",
        ));
    }
    Ok(())
}

fn read_method(state: &LuaState) -> LuaResult<usize> {
    let value = stack_val(state, 1);
    reject_secret(state, value)?;
    match value {
        Val::Num(number) if matches!(number, 0.0 | 1.0 | 2.0 | 3.0 | 4.0 | 5.0) => {
            Ok(number as usize)
        }
        _ => Err(runtime_error(
            "SetLootMethod: method must be a public enum number from 0 to 5",
        )),
    }
}

fn read_master(state: &LuaState) -> LuaResult<Option<String>> {
    let value = stack_val(state, 2);
    reject_secret(state, value)?;
    match value {
        Val::Nil => Ok(None),
        Val::Str(_) => String::from_stack(state, 2).map(Some),
        _ => Err(runtime_error(
            "SetLootMethod: lootMaster must be a public string or nil",
        )),
    }
}

fn resolve_indices(sim: &SimState, method: usize, master: Option<&str>) -> Option<(i32, i32)> {
    if method != 2 {
        return Some((0, 0));
    }
    let name = master.filter(|name| !name.is_empty())?;
    if !sim.party_group_active || sim.party_members.is_empty() {
        return None;
    }
    if sim.party_members.len() >= 6 {
        // Existing raid roster places the player first. General raid aliases
        // disagree with that roster; do not invent a member-to-index mapping.
        return (name == sim.player.name).then_some((0, 1));
    }
    let index = sim
        .party_members
        .iter()
        .position(|member| member.name == name)?;
    let party = i32::try_from(index + 1).ok()?;
    Some((party, 0))
}

fn apply_selection(sim: &mut SimState, token: &str, party: i32, raid: i32) {
    let loot = &mut sim.loot_method;
    if loot.method == token && loot.party_master_index == party && loot.raid_master_index == raid {
        return;
    }
    loot.method = token.into();
    loot.party_master_index = party;
    loot.raid_master_index = raid;
    sim.events.push(Event {
        name: "PARTY_LOOT_METHOD_CHANGED".into(),
        args: Vec::new(),
    });
}
