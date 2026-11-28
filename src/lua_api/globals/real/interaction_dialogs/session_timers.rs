//! Server countdowns read by death and instance dialogs.
//!
//! Declarations (byte-identical Retail/Forever): `InstanceDocumentation.lua`
//! (`GetInstanceBootTimeRemaining -> number`, `GetInstanceLockTimeRemaining ->
//! timeLeft, extending, encountersTotal, encountersCompleted`,
//! `GetInstanceLockTimeRemainingEncounter(index) -> encounterName, texture,
//! isKilled, ineligible`) and `PlayerScriptDocumentation.lua`
//! (`GetCorpseRecoveryDelay -> number`, `GetResSicknessDuration -> string?`).
//! Consumers: `Blizzard_StaticPopup_Game/GameDialogDefs.lua` `INSTANCE_BOOT`,
//! `INSTANCE_LOCK` (opened by `INSTANCE_BOOT_START`, `INSTANCE_LOCK_START`,
//! `INSTANCE_LOCK_WARNING`), `RESURRECT` (`timeleft = GetCorpseRecoveryDelay() + 60`)
//! and `Blizzard_Game` `CONFIRM_XP_LOSS` (`GetResSicknessDuration`).
//! Deadlines live on the simulator clock. INFERRED (R-ag10-10): an expired or
//! absent countdown reads 0, and an encounter index outside the supplied list
//! returns nothing.

use super::{fire, push_optional_string};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, table_get, val_to_string,
};
use crate::lua_bridge::{FromStack, TableBuilder, stack_val};
use rilua::vm::state::LuaState;
use rilua::{LuaApiMut, LuaResult, Val, runtime_error};

/// One boss of the instance lock: name, texture, killed, ineligible.
#[derive(Clone, Debug, PartialEq)]
pub struct LockEncounter {
    pub name: String,
    pub texture: String,
    pub killed: bool,
    pub ineligible: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InstanceLock {
    pub expires_at: f64,
    pub extending: bool,
    pub encounters: Vec<LockEncounter>,
}

#[derive(Clone, Debug, Default)]
pub struct SessionTimers {
    pub instance_boot_at: Option<f64>,
    pub instance_lock: Option<InstanceLock>,
    pub corpse_recovery_at: f64,
    pub res_sickness: Option<String>,
}

const LOCK_USAGE: &str =
    "Usage: A_Admin.SetInstanceLock(seconds|nil, extending, enforce, encounters)";

fn now(state: &LuaState) -> LuaResult<f64> {
    Ok(borrow_state(state)?.clock_elapsed().as_secs_f64())
}

fn remaining(state: &LuaState, at: Option<f64>) -> LuaResult<f64> {
    let now = now(state)?;
    Ok(at.map_or(0.0, |at| (at - now).max(0.0)))
}

fn read_seconds(state: &LuaState, index: i32, usage: &str) -> LuaResult<Option<f64>> {
    let seconds = Option::<f64>::from_stack(state, index)?;
    if seconds.is_some_and(|s| !s.is_finite() || s < 0.0) {
        return Err(runtime_error(usage));
    }
    Ok(seconds)
}

fn set_instance_boot(state: &mut LuaState) -> LuaResult<u32> {
    let seconds = read_seconds(state, 1, "Usage: A_Admin.SetInstanceBoot(seconds|nil)")?;
    let at = match seconds {
        Some(seconds) => Some(now(state)? + seconds),
        None => None,
    };
    borrow_state_mut(state)?
        .interaction_dialogs
        .timers
        .instance_boot_at = at;
    let event = if at.is_some() {
        "INSTANCE_BOOT_START"
    } else {
        "INSTANCE_BOOT_STOP"
    };
    fire(state, event, Vec::new())?;
    Ok(0)
}

fn read_string_field(state: &mut LuaState, row: Val, key: &str) -> Option<String> {
    let value = table_get(state, row, key);
    matches!(value, Val::Str(_))
        .then(|| val_to_string(state, value))
        .flatten()
}

fn read_flag(state: &mut LuaState, row: Val, key: &str) -> bool {
    matches!(table_get(state, row, key), Val::Bool(true))
}

fn read_encounter(state: &mut LuaState, row: Val) -> LuaResult<LockEncounter> {
    if !matches!(row, Val::Table(_)) {
        return Err(runtime_error(LOCK_USAGE));
    }
    let name = read_string_field(state, row, "name").filter(|name| !name.is_empty());
    Ok(LockEncounter {
        name: name.ok_or_else(|| runtime_error(LOCK_USAGE))?,
        texture: read_string_field(state, row, "texture").unwrap_or_default(),
        killed: read_flag(state, row, "killed"),
        ineligible: read_flag(state, row, "ineligible"),
    })
}

fn read_encounters(state: &mut LuaState, index: i32) -> LuaResult<Vec<LockEncounter>> {
    let Val::Table(table) = stack_val(state, index) else {
        return Err(runtime_error(LOCK_USAGE));
    };
    let rows: Vec<Val> = state
        .gc
        .tables
        .get(table)
        .map(|table| table.array_slice().to_vec())
        .unwrap_or_default();
    rows.into_iter()
        .map(|row| read_encounter(state, row))
        .collect()
}

fn set_instance_lock(state: &mut LuaState) -> LuaResult<u32> {
    let Some(seconds) = read_seconds(state, 1, LOCK_USAGE)? else {
        borrow_state_mut(state)?
            .interaction_dialogs
            .timers
            .instance_lock = None;
        fire(state, "INSTANCE_LOCK_STOP", Vec::new())?;
        return Ok(0);
    };
    let lock = InstanceLock {
        expires_at: now(state)? + seconds,
        extending: bool::from_stack(state, 2)?,
        encounters: read_encounters(state, 4)?,
    };
    let enforce = bool::from_stack(state, 3)?;
    borrow_state_mut(state)?
        .interaction_dialogs
        .timers
        .instance_lock = Some(lock);
    let event = if enforce {
        "INSTANCE_LOCK_START"
    } else {
        "INSTANCE_LOCK_WARNING"
    };
    fire(state, event, Vec::new())?;
    Ok(0)
}

fn get_instance_boot_time_remaining(state: &mut LuaState) -> LuaResult<u32> {
    let at = borrow_state(state)?
        .interaction_dialogs
        .timers
        .instance_boot_at;
    let left = remaining(state, at)?;
    state.push(Val::Num(left));
    Ok(1)
}

fn get_instance_lock_time_remaining(state: &mut LuaState) -> LuaResult<u32> {
    let lock = borrow_state(state)?
        .interaction_dialogs
        .timers
        .instance_lock
        .clone();
    let left = remaining(state, lock.as_ref().map(|lock| lock.expires_at))?;
    let encounters = lock.as_ref().map_or(&[][..], |lock| &lock.encounters[..]);
    let killed = encounters
        .iter()
        .filter(|encounter| encounter.killed)
        .count();
    state.push(Val::Num(left));
    state.push(Val::Bool(lock.as_ref().is_some_and(|lock| lock.extending)));
    state.push(Val::Num(encounters.len() as f64));
    state.push(Val::Num(killed as f64));
    Ok(4)
}

fn get_instance_lock_time_remaining_encounter(state: &mut LuaState) -> LuaResult<u32> {
    let index = i64::from_stack(state, 1)?;
    let encounter = {
        let sim = borrow_state(state)?;
        let lock = sim.interaction_dialogs.timers.instance_lock.as_ref();
        let slot = usize::try_from(index - 1).ok();
        lock.zip(slot)
            .and_then(|(lock, slot)| lock.encounters.get(slot).cloned())
    };
    let Some(encounter) = encounter else {
        return Ok(0);
    };
    let name = create_string(state, &encounter.name);
    state.push(name);
    let texture = create_string(state, &encounter.texture);
    state.push(texture);
    state.push(Val::Bool(encounter.killed));
    state.push(Val::Bool(encounter.ineligible));
    Ok(4)
}

fn set_corpse_recovery_delay(state: &mut LuaState) -> LuaResult<u32> {
    let usage = "Usage: A_Admin.SetCorpseRecoveryDelay(seconds)";
    let seconds = read_seconds(state, 1, usage)?.ok_or_else(|| runtime_error(usage))?;
    let at = now(state)? + seconds;
    borrow_state_mut(state)?
        .interaction_dialogs
        .timers
        .corpse_recovery_at = at;
    Ok(0)
}

fn get_corpse_recovery_delay(state: &mut LuaState) -> LuaResult<u32> {
    let at = borrow_state(state)?
        .interaction_dialogs
        .timers
        .corpse_recovery_at;
    let left = remaining(state, Some(at))?;
    state.push(Val::Num(left));
    Ok(1)
}

fn set_res_sickness_duration(state: &mut LuaState) -> LuaResult<u32> {
    let text = Option::<String>::from_stack(state, 1)?;
    borrow_state_mut(state)?
        .interaction_dialogs
        .timers
        .res_sickness = text;
    Ok(0)
}

fn get_res_sickness_duration(state: &mut LuaState) -> LuaResult<u32> {
    let text = borrow_state(state)?
        .interaction_dialogs
        .timers
        .res_sickness
        .clone();
    push_optional_string(state, text.as_deref());
    Ok(1)
}

type Global = (&'static str, fn(&mut LuaState) -> LuaResult<u32>);

pub(crate) fn register_all(lua: &mut rilua::Lua) -> LuaResult<()> {
    let functions: &[Global] = &[
        (
            "GetInstanceBootTimeRemaining",
            get_instance_boot_time_remaining,
        ),
        (
            "GetInstanceLockTimeRemaining",
            get_instance_lock_time_remaining,
        ),
        (
            "GetInstanceLockTimeRemainingEncounter",
            get_instance_lock_time_remaining_encounter,
        ),
        ("GetCorpseRecoveryDelay", get_corpse_recovery_delay),
        ("GetResSicknessDuration", get_res_sickness_duration),
    ];
    for (name, function) in functions {
        LuaApiMut::register_function(lua, name, *function)?;
    }
    Ok(())
}

pub(crate) fn register_admin(b: TableBuilder) -> LuaResult<TableBuilder> {
    b.set_function("SetInstanceBoot", set_instance_boot)?
        .set_function("SetInstanceLock", set_instance_lock)?
        .set_function("SetCorpseRecoveryDelay", set_corpse_recovery_delay)?
        .set_function("SetResSicknessDuration", set_res_sickness_duration)
}
