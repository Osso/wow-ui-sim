//! Last challenge completion snapshot, independent of historical weekly bests.

use crate::c_api::ensure_namespace;
use crate::client_profile::{ACTIVE, ClientProfile};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, create_table_with_capacity,
    table_set_static,
};
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum KeystoneLocation {
    Bag(i32, i32),
    Equipment(i32),
}

#[derive(Debug, Default)]
pub struct KeystoneState {
    pub receptacle_guid: Option<String>,
    pub selected_guid: Option<String>,
    pub selected_location: Option<KeystoneLocation>,
    pub selected_mode: Option<i32>,
    pub item_modifiers: BTreeMap<KeystoneLocation, (u32, Vec<(u32, u32)>)>,
    pub current_map_selector: Option<u32>,
    pub reason_one_locks: BTreeSet<KeystoneLocation>,
    pub other_locks: BTreeSet<KeystoneLocation>,
    pub native_locks: BTreeSet<KeystoneLocation>,
}

impl KeystoneState {
    pub fn is_locked(&self, location: KeystoneLocation) -> bool {
        self.reason_one_locks.contains(&location)
            || self.other_locks.contains(&location)
            || self.native_locks.contains(&location)
    }
}

/// Host-supplied result of the most recently completed challenge.
#[derive(Debug, Clone, Default)]
pub struct ChallengeCompletionInfo {
    pub map_challenge_mode_id: i32,
    pub level: i32,
    /// Completion time in milliseconds, as consumed by ChallengesUI.
    pub time: f64,
    pub on_time: bool,
    pub keystone_upgrade_levels: i32,
    pub practice_run: bool,
    pub old_overall_dungeon_score: Option<f64>,
    pub new_overall_dungeon_score: Option<f64>,
    pub is_map_record: bool,
    pub is_affix_record: bool,
    pub is_eligible_for_score: bool,
    pub members: Vec<ChallengeCompletionMember>,
}

#[derive(Debug, Clone)]
pub struct ChallengeCompletionMember {
    pub member_guid: String,
    pub name: String,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_ChallengeMode")?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetChallengeCompletionInfo",
        get_challenge_completion_info,
    )?;
    if ACTIVE == ClientProfile::Retail {
        table_set_rust_fn_static(state, namespace, "CloseKeystoneFrame", close_keystone_frame)?;
        table_set_rust_fn_static(state, namespace, "ClearKeystone", clear_keystone)?;
        table_set_rust_fn_static(
            state,
            namespace,
            "CanUseKeystoneInCurrentMap",
            can_use_keystone_in_current_map,
        )?;
    }
    Ok(())
}

fn close_keystone_frame(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.keystone.receptacle_guid = None;
    Ok(0)
}

fn can_use_keystone_in_current_map(state: &mut LuaState) -> LuaResult<u32> {
    use crate::c_api::item_upgrade_eligibility::{ItemSlot, read_item_slot};

    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let location = match read_item_slot(state, value) {
        Some(ItemSlot::Bag(bag, slot)) => KeystoneLocation::Bag(bag, slot),
        Some(ItemSlot::Equipment(slot)) => KeystoneLocation::Equipment(slot),
        None => {
            return Err(rilua::runtime_error(
                "CanUseKeystoneInCurrentMap requires an item location",
            ));
        }
    };
    let item_id = crate::c_api::item_spell::location_item(state, value)
        .map(|(id, _)| id)
        .ok_or_else(|| {
            rilua::runtime_error("CanUseKeystoneInCurrentMap requires an existing item")
        })?;
    let sim = borrow_state(state)?;
    let challenge_id = sim
        .keystone
        .item_modifiers
        .get(&location)
        .filter(|(observed_id, _)| *observed_id == item_id)
        .and_then(|(_, modifiers)| modifiers.iter().find(|(kind, _)| *kind == 17))
        .map_or(0, |(_, value)| *value);
    let maps = sim_data::map_challenge_modes::MAPS;
    let result = maps
        .binary_search_by_key(&challenge_id, |row| row.0)
        .ok()
        .is_some_and(|index| {
            u32::from(maps[index].1) == sim.keystone.current_map_selector.unwrap_or(0)
        });
    drop(sim);
    state.push(Val::Bool(result));
    Ok(1)
}

fn clear_keystone(state: &mut LuaState) -> LuaResult<u32> {
    let unlocked = {
        let mut sim = borrow_state_mut(state)?;
        if sim.keystone.selected_guid.is_none() {
            return Ok(0);
        }
        let location = sim.keystone.selected_location;
        location.filter(|location| {
            let before = sim.keystone.is_locked(*location);
            sim.keystone.reason_one_locks.remove(location);
            let after = sim.keystone.is_locked(*location);
            before && !after && location_exists(&sim, *location)
        })
    };
    if let Some(location) = unlocked {
        let args = match location {
            KeystoneLocation::Bag(bag, slot) => {
                vec![Val::Num(f64::from(bag)), Val::Num(f64::from(slot))]
            }
            KeystoneLocation::Equipment(slot) => vec![Val::Num(f64::from(slot))],
        };
        crate::lua_api::globals::state_backed_queries::dispatch_event_now(
            state,
            "ITEM_LOCK_CHANGED",
            &args,
        )?;
        crate::lua_api::globals::state_backed_queries::dispatch_event_now(
            state,
            "ITEM_UNLOCKED",
            &args,
        )?;
    }
    let mut sim = borrow_state_mut(state)?;
    sim.keystone.selected_guid = None;
    sim.keystone.selected_location = None;
    sim.keystone.selected_mode = None;
    Ok(0)
}

fn location_exists(sim: &crate::lua_api::SimState, location: KeystoneLocation) -> bool {
    match location {
        KeystoneLocation::Bag(bag, slot) => sim.get_bag_item(bag, slot).is_some(),
        KeystoneLocation::Equipment(slot) => sim.player.equipped_items.contains_key(&slot),
    }
}

fn get_challenge_completion_info(state: &mut LuaState) -> LuaResult<u32> {
    let completion = borrow_state(state)?.challenge_completion.clone();
    let info = create_table_with_capacity(state, 12);
    state.push(info);
    set_completion_fields(state, info, &completion);
    let members = create_table(state);
    table_set_static(state, info, "members", members);
    for (index, member) in completion.members.iter().enumerate() {
        let row = create_table_with_capacity(state, 2);
        crate::c_api::helpers::set_table_array(state, members, index as i64 + 1, row);
        let guid = create_string(state, &member.member_guid);
        table_set_static(state, row, "memberGUID", guid);
        let name = create_string(state, &member.name);
        table_set_static(state, row, "name", name);
    }
    Ok(1)
}

fn set_completion_fields(state: &mut LuaState, info: Val, completion: &ChallengeCompletionInfo) {
    set_completion_numbers(state, info, completion);
    set_completion_flags(state, info, completion);
    table_set_static(
        state,
        info,
        "oldOverallDungeonScore",
        completion
            .old_overall_dungeon_score
            .map_or(Val::Nil, Val::Num),
    );
    table_set_static(
        state,
        info,
        "newOverallDungeonScore",
        completion
            .new_overall_dungeon_score
            .map_or(Val::Nil, Val::Num),
    );
}

fn set_completion_numbers(state: &mut LuaState, info: Val, completion: &ChallengeCompletionInfo) {
    for (name, value) in [
        (
            "mapChallengeModeID",
            f64::from(completion.map_challenge_mode_id),
        ),
        ("level", f64::from(completion.level)),
        ("time", completion.time),
        (
            "keystoneUpgradeLevels",
            f64::from(completion.keystone_upgrade_levels),
        ),
    ] {
        table_set_static(state, info, name, Val::Num(value));
    }
}

fn set_completion_flags(state: &mut LuaState, info: Val, completion: &ChallengeCompletionInfo) {
    for (name, value) in [
        ("onTime", completion.on_time),
        ("practiceRun", completion.practice_run),
        ("isMapRecord", completion.is_map_record),
        ("isAffixRecord", completion.is_affix_record),
        ("isEligibleForScore", completion.is_eligible_for_score),
    ] {
        table_set_static(state, info, name, Val::Bool(value));
    }
}
