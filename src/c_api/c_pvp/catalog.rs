//! Explicit PvP catalog/eligibility snapshots; queue requests use the existing queue.
use crate::lua_api::methods::{
    borrow_state, create_string, create_table, table_set, val_to_string,
};
use crate::lua_api::state::BattlefieldStatus;
use crate::lua_bridge::{stack_val, table_set_rust_fn_static};
use rilua::vm::{gc::arena::GcRef, state::LuaState, table::Table};
use rilua::{LuaResult, Val};

#[derive(Debug, Clone, Default)]
pub struct BattlegroundInfo {
    pub name: String,
    pub icon: Option<u32>,
    pub game_type: String,
    pub short_description: String,
    pub long_description: String,
    pub map_description: String,
    pub max_players: u32,
    pub battleground_id: Option<u32>,
    pub lfg_dungeon_id: Option<u32>,
    pub map_id: Option<u32>,
    pub is_holiday: bool,
    pub is_random: bool,
    pub can_enter: bool,
    pub is_training_ground: bool,
}

/// INFERRED empty/disabled defaults. Acquisition, matchmaking and rewards are not fabricated.
#[derive(Debug, Default)]
pub struct PvpCatalog {
    pub battlegrounds: Vec<BattlegroundInfo>,
    pub training_enabled: bool,
    pub training_eligible: bool,
    pub training_failure_reason: String,
    pub rated_eligible: bool,
    pub rated_failure_reason: String,
    pub random_training_win_today: bool,
    pub match_completed: bool,
}

pub(super) fn register(state: &mut LuaState, ns: GcRef<Table>) -> LuaResult<()> {
    if matches!(
        crate::client_profile::ACTIVE,
        crate::client_profile::ClientProfile::Retail
            | crate::client_profile::ClientProfile::Ptr
            | crate::client_profile::ClientProfile::WowForever
    ) {
        table_set_rust_fn_static(state, ns, "CanPlayerUseRatedPVPUI", rated_eligible)?;
    }
    for (name, function) in [
        ("GetBattlegroundInfo", battleground_info as rilua::RustFn),
        ("GetTrainingGrounds", training_grounds),
        ("AreTrainingGroundsEnabled", training_enabled),
        ("CanPlayerUseTrainingGroundsUI", training_eligible),
        ("HasRandomTrainingGroundWinToday", random_win),
        ("HasMatchStarted", match_started),
        ("JoinTrainingGround", join),
    ] {
        table_set_rust_fn_static(state, ns, name, function)?;
    }
    Ok(())
}

fn read_id(state: &LuaState) -> LuaResult<u32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    match value {
        Val::Num(n) if n > 0.0 && n <= u32::MAX as f64 && n.fract() == 0.0 => Ok(n as u32),
        _ => Err(rilua::runtime_error(
            "battleground selector must be a positive integer",
        )),
    }
}
fn read_battleground_index(state: &LuaState) -> LuaResult<i32> {
    let value = rilua::table_security::unwrap_secret(state, stack_val(state, 1))?;
    let number = match value {
        Val::Num(number) => Some(number),
        Val::Str(_) => val_to_string(state, value).and_then(|text| text.trim().parse::<f64>().ok()),
        _ => None,
    }
    .ok_or_else(|| {
        rilua::runtime_error(
            "Usage: local battlegroundInfo = C_PvP.GetBattlegroundInfo(battlegroundIndex)",
        )
    })?;

    if !number.is_finite() || !(0.0..=f64::from(u32::MAX)).contains(&number) {
        return Err(rilua::runtime_error(
            "battleground index must be between 0 and 4294967295",
        ));
    }

    Ok((number - 1.0).trunc() as i32)
}

fn battleground_info(state: &mut LuaState) -> LuaResult<u32> {
    let index = read_battleground_index(state)?;
    let record = if index >= 0 {
        borrow_state(state)?
            .pvp_catalog
            .battlegrounds
            .get(index as usize)
            .cloned()
    } else {
        None
    };
    if let Some(record) = record {
        let result = publish(state, &record);
        if record.icon == Some(0) {
            table_set(state, result, "icon", Val::Nil);
        }
    } else {
        state.push(Val::Nil);
    }
    Ok(1)
}
fn training_grounds(state: &mut LuaState) -> LuaResult<u32> {
    let records: Vec<_> = borrow_state(state)?
        .pvp_catalog
        .battlegrounds
        .iter()
        .filter(|row| row.is_training_ground)
        .cloned()
        .collect();
    let result = create_table(state);
    state.push(result);
    for (index, record) in records.iter().enumerate() {
        let row = publish(state, record);
        super::super::helpers::set_table_array(state, result, index as i64 + 1, row);
        state.pop();
    }
    Ok(1)
}
fn publish(state: &mut LuaState, record: &BattlegroundInfo) -> Val {
    let result = create_table(state);
    state.push(result);
    for (key, text) in [
        ("name", &record.name),
        ("gameType", &record.game_type),
        ("shortDescription", &record.short_description),
        ("longDescription", &record.long_description),
        ("mapDescription", &record.map_description),
    ] {
        let value = create_string(state, text);
        table_set(state, result, key, value);
    }
    for (key, value) in [
        ("icon", record.icon),
        ("battlegroundID", record.battleground_id),
        ("lfgDungeonID", record.lfg_dungeon_id),
        ("mapID", record.map_id),
    ] {
        table_set(
            state,
            result,
            key,
            value.map_or(Val::Nil, |v| Val::Num(f64::from(v))),
        );
    }
    table_set(
        state,
        result,
        "maxPlayers",
        Val::Num(f64::from(record.max_players)),
    );
    for (key, value) in [
        ("isHoliday", record.is_holiday),
        ("isRandom", record.is_random),
        ("canEnter", record.can_enter),
        ("isTrainingGround", record.is_training_ground),
    ] {
        table_set(state, result, key, Val::Bool(value));
    }
    result
}
fn training_enabled(state: &mut LuaState) -> LuaResult<u32> {
    let value = borrow_state(state)?.pvp_catalog.training_enabled;
    state.push(Val::Bool(value));
    Ok(1)
}
fn training_eligible(state: &mut LuaState) -> LuaResult<u32> {
    let (allowed, reason) = {
        let sim = borrow_state(state)?;
        let catalog = &sim.pvp_catalog;
        (
            catalog.training_enabled && catalog.training_eligible,
            catalog.training_failure_reason.clone(),
        )
    };
    state.push(Val::Bool(allowed));
    let value = create_string(state, if allowed { "" } else { &reason });
    state.push(value);
    Ok(2)
}
fn rated_eligible(state: &mut LuaState) -> LuaResult<u32> {
    let (allowed, reason) = {
        let sim = borrow_state(state)?;
        (
            sim.pvp_catalog.rated_eligible,
            sim.pvp_catalog.rated_failure_reason.clone(),
        )
    };
    state.push(Val::Bool(allowed));
    let value = create_string(state, if allowed { "" } else { &reason });
    state.push(value);
    Ok(2)
}
fn random_win(state: &mut LuaState) -> LuaResult<u32> {
    let value = borrow_state(state)?.pvp_catalog.random_training_win_today;
    state.push(Val::Bool(value));
    Ok(1)
}
fn match_started(state: &mut LuaState) -> LuaResult<u32> {
    let value = {
        let sim = borrow_state(state)?;
        sim.battlefield_queue.status == BattlefieldStatus::Active || sim.pvp_catalog.match_completed
    };
    state.push(Val::Bool(value));
    Ok(1)
}
fn join(state: &mut LuaState) -> LuaResult<u32> {
    let id = read_id(state)?;
    let name = {
        let sim = borrow_state(state)?;
        if !sim.pvp_catalog.training_enabled || !sim.pvp_catalog.training_eligible {
            return Err(rilua::runtime_error(
                "training grounds are unavailable for this player",
            ));
        }
        sim.pvp_catalog
            .battlegrounds
            .iter()
            .find(|row| row.lfg_dungeon_id == Some(id) && row.is_training_ground && row.can_enter)
            .map(|row| row.name.clone())
            .ok_or_else(|| rilua::runtime_error("training ground is unknown or not enterable"))?
    };
    // INFERRED one queue slot; use the same event/state transition as JoinBattlefield.
    crate::lua_api::globals::battlefield_verbs::queue_battlefield(state, 1, name)?;
    Ok(0)
}
