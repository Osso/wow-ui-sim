//! INFERRED (R88dg pending): host-supplied mission records for Retail/Forever consumers.

use std::collections::HashMap;

use crate::c_api::helpers::{ensure_namespace, set_table_array};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table_with_capacity, table_get,
    table_set_static, val_to_string,
};
use crate::lua_bridge::{TableBuilder, stack_val, table_set_rust_fn_static};
use rilua::vm::gc::arena::GcRef;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val, runtime_error};

const INPUT_ERROR: &str = "A_Admin.SetGarrisonMissions: expected an array of mission tables";

#[derive(Clone, Debug, Default)]
pub struct GarrisonMissionLists {
    pub missions: Vec<GarrisonMission>,
    pub follower_quality_tables: HashMap<u32, Vec<(u32, f64)>>,
    /// INFERRED (R88do, native evidence pending): supplied NPC interaction state.
    pub at_garrison_mission_npc: bool,
    /// INFERRED (R88dr, native evidence pending): explicit map-tab visibility state.
    pub should_show_map_tab: Option<bool>,
}

#[derive(Clone, Debug)]
pub struct GarrisonMission {
    pub mission_id: u32,
    pub name: String,
    pub follower_type_id: u32,
    pub level: u32,
    pub duration_seconds: f64,
    pub duration: String,
    pub time_left: String,
    pub xp: f64,
    pub can_be_completed: bool,
    pub rewards: Vec<GarrisonMissionReward>,
    pub in_progress: bool,
    pub completed: bool,
    pub encounter_icon_info: Option<GarrisonMissionEncounterIconInfo>,
}

#[derive(Clone, Debug, Default)]
pub struct GarrisonMissionEncounterIconInfo {
    pub portrait_file_data_id: u32,
    pub mission_scalar: f64,
    pub is_elite: bool,
    pub is_rare: bool,
}

#[derive(Clone, Debug)]
pub struct GarrisonMissionReward {
    pub item_id: Option<u32>,
    pub currency_id: Option<u32>,
    pub quantity: Option<u32>,
}

#[derive(Clone, Copy)]
enum MissionListKind {
    Available,
    InProgress,
    Complete,
}

pub(crate) fn register(state: &mut LuaState) -> LuaResult<()> {
    let namespace = ensure_namespace(state, "C_Garrison")?;
    table_set_rust_fn_static(state, namespace, "GetAvailableMissions", get_available)?;
    table_set_rust_fn_static(state, namespace, "GetInProgressMissions", get_in_progress)?;
    table_set_rust_fn_static(state, namespace, "GetCompleteMissions", get_complete)?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetMissionEncounterIconInfo",
        get_mission_encounter_icon_info,
    )?;
    table_set_rust_fn_static(
        state,
        namespace,
        "GetFollowerQualityTable",
        get_follower_quality_table,
    )?;
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Retail {
        table_set_rust_fn_static(
            state,
            namespace,
            "IsAtGarrisonMissionNPC",
            is_at_garrison_mission_npc,
        )?;
        table_set_rust_fn_static(state, namespace, "CloseMissionNPC", close_mission_npc)?;
        table_set_rust_fn_static(state, namespace, "ShouldShowMapTab", should_show_map_tab)
    } else {
        Ok(())
    }
}

pub(crate) fn register_admin(builder: TableBuilder) -> LuaResult<TableBuilder> {
    let builder = builder
        .set_function("SetGarrisonMissions", set_missions)?
        .set_function(
            "SetGarrisonFollowerQualityTable",
            set_follower_quality_table,
        )?;
    if crate::client_profile::ACTIVE == crate::client_profile::ClientProfile::Retail {
        builder
            .set_function("SetAtGarrisonMissionNPC", set_at_garrison_mission_npc)?
            .set_function(
                "SetShouldShowGarrisonMissionMapTab",
                set_should_show_map_tab,
            )
    } else {
        Ok(builder)
    }
}

fn set_missions(state: &mut LuaState) -> LuaResult<u32> {
    let missions = parse_missions(state, stack_val(state, 1))?;
    borrow_state_mut(state)?.garrison_mission_lists.missions = missions;
    Ok(0)
}

fn is_at_garrison_mission_npc(state: &mut LuaState) -> LuaResult<u32> {
    let at_npc = borrow_state(state)?
        .garrison_mission_lists
        .at_garrison_mission_npc;
    state.push(Val::Bool(at_npc));
    Ok(1)
}

fn set_at_garrison_mission_npc(state: &mut LuaState) -> LuaResult<u32> {
    let Val::Bool(at_npc) = stack_val(state, 1) else {
        return Err(runtime_error("bad argument #1 (boolean expected)"));
    };
    borrow_state_mut(state)?
        .garrison_mission_lists
        .at_garrison_mission_npc = at_npc;
    Ok(0)
}

fn close_mission_npc(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?
        .garrison_mission_lists
        .at_garrison_mission_npc = false;
    crate::lua_api::globals::state_backed_queries::dispatch_event_now(
        state,
        "GARRISON_MISSION_NPC_CLOSED",
        &[],
    )?;
    Ok(0)
}

fn should_show_map_tab(state: &mut LuaState) -> LuaResult<u32> {
    read_u32(stack_val(state, 1))
        .ok_or_else(|| runtime_error("bad argument #1 (garrisonType must be a number)"))?;
    let should_show = borrow_state(state)?
        .garrison_mission_lists
        .should_show_map_tab
        .unwrap_or(true);
    state.push(Val::Bool(should_show));
    Ok(1)
}

fn set_should_show_map_tab(state: &mut LuaState) -> LuaResult<u32> {
    let Val::Bool(should_show) = stack_val(state, 1) else {
        return Err(runtime_error("bad argument #1 (boolean expected)"));
    };
    borrow_state_mut(state)?
        .garrison_mission_lists
        .should_show_map_tab = Some(should_show);
    Ok(0)
}

fn get_available(state: &mut LuaState) -> LuaResult<u32> {
    get_missions(state, MissionListKind::Available)
}

fn get_in_progress(state: &mut LuaState) -> LuaResult<u32> {
    get_missions(state, MissionListKind::InProgress)
}

fn get_complete(state: &mut LuaState) -> LuaResult<u32> {
    let follower_type_id = read_follower_type(stack_val(state, 1), 1)?;
    let missions = select_missions(state, follower_type_id, MissionListKind::Complete)?;
    let result = publish_missions(state, &missions)?;
    state.push(result);
    Ok(1)
}

fn get_mission_encounter_icon_info(state: &mut LuaState) -> LuaResult<u32> {
    let mission_id = read_u32(stack_val(state, 1))
        .ok_or_else(|| runtime_error("bad argument #1 (missionID must be a number)"))?;
    let info = borrow_state(state)?
        .garrison_mission_lists
        .missions
        .iter()
        .find(|mission| mission.mission_id == mission_id)
        .and_then(|mission| mission.encounter_icon_info.clone())
        .unwrap_or_default();
    let result = publish_encounter_icon_info(state, &info);
    state.push(result);
    Ok(1)
}

fn get_follower_quality_table(state: &mut LuaState) -> LuaResult<u32> {
    let follower_type_id = read_follower_type(stack_val(state, 1), 1)?;
    let entries = borrow_state(state)?
        .garrison_mission_lists
        .follower_quality_tables
        .get(&follower_type_id)
        .cloned()
        .unwrap_or_default();
    let result = create_table_with_capacity(state, entries.len());
    let Val::Table(table_ref) = result else {
        unreachable!("create_table_with_capacity returns a table")
    };
    for (quality, xp) in entries {
        let table = state
            .gc
            .tables
            .get_mut(table_ref)
            .expect("newly created quality table exists");
        table.raw_set(
            Val::Num(f64::from(quality)),
            Val::Num(xp),
            &state.gc.string_arena,
        )?;
    }
    state.push(result);
    Ok(1)
}

fn set_follower_quality_table(state: &mut LuaState) -> LuaResult<u32> {
    let follower_type_id = read_follower_type(stack_val(state, 1), 1)?;
    let entries = parse_follower_quality_table(state, stack_val(state, 2))?;
    borrow_state_mut(state)?
        .garrison_mission_lists
        .follower_quality_tables
        .insert(follower_type_id, entries);
    Ok(0)
}

fn get_missions(state: &mut LuaState, kind: MissionListKind) -> LuaResult<u32> {
    let (output, follower_type_id) = read_list_arguments(state)?;
    let missions = select_missions(state, follower_type_id, kind)?;
    let result = output.unwrap_or_else(|| create_table_with_capacity(state, missions.len()));
    publish_missions_into(state, result, &missions)?;
    state.push(result);
    Ok(1)
}

fn read_list_arguments(state: &LuaState) -> LuaResult<(Option<Val>, u32)> {
    let first = stack_val(state, 1);
    match first {
        Val::Table(_) => Ok((Some(first), read_follower_type(stack_val(state, 2), 2)?)),
        Val::Num(_) => Ok((None, read_follower_type(first, 1)?)),
        Val::Nil => Ok((None, read_follower_type(stack_val(state, 2), 2)?)),
        _ => Err(runtime_error(
            "bad argument #1 (table, number or nil expected)",
        )),
    }
}

fn read_follower_type(value: Val, index: i32) -> LuaResult<u32> {
    read_u32(value).ok_or_else(|| {
        runtime_error(format!(
            "bad argument #{index} (followerTypeID must be a number)"
        ))
    })
}

fn select_missions(
    state: &LuaState,
    follower_type_id: u32,
    kind: MissionListKind,
) -> LuaResult<Vec<GarrisonMission>> {
    let sim = borrow_state(state)?;
    Ok(sim
        .garrison_mission_lists
        .missions
        .iter()
        .filter(|mission| {
            mission.follower_type_id == follower_type_id && matches_kind(mission, kind)
        })
        .cloned()
        .collect())
}

fn matches_kind(mission: &GarrisonMission, kind: MissionListKind) -> bool {
    match kind {
        MissionListKind::Available => !mission.in_progress && !mission.completed,
        MissionListKind::InProgress => mission.in_progress,
        MissionListKind::Complete => mission.completed,
    }
}

fn publish_missions(state: &mut LuaState, missions: &[GarrisonMission]) -> LuaResult<Val> {
    let result = create_table_with_capacity(state, missions.len());
    publish_missions_into(state, result, missions)?;
    Ok(result)
}

fn publish_encounter_icon_info(
    state: &mut LuaState,
    info: &GarrisonMissionEncounterIconInfo,
) -> Val {
    let result = create_table_with_capacity(state, 4);
    table_set_static(
        state,
        result,
        "portraitFileDataID",
        Val::Num(f64::from(info.portrait_file_data_id)),
    );
    table_set_static(
        state,
        result,
        "missionScalar",
        Val::Num(info.mission_scalar),
    );
    table_set_static(state, result, "isElite", Val::Bool(info.is_elite));
    table_set_static(state, result, "isRare", Val::Bool(info.is_rare));
    result
}

fn publish_missions_into(
    state: &mut LuaState,
    result: Val,
    missions: &[GarrisonMission],
) -> LuaResult<()> {
    let Val::Table(table) = result else {
        return Err(runtime_error("expected mission result table"));
    };
    clear_table(state, table)?;
    for (index, mission) in missions.iter().enumerate() {
        let value = publish_mission(state, mission)?;
        set_table_array(state, result, (index + 1) as i64, value);
    }
    Ok(())
}

fn publish_mission(state: &mut LuaState, mission: &GarrisonMission) -> LuaResult<Val> {
    let result = create_table_with_capacity(state, 12);
    let name = create_string(state, &mission.name);
    let duration = create_string(state, &mission.duration);
    let time_left = create_string(state, &mission.time_left);
    table_set_static(
        state,
        result,
        "missionID",
        Val::Num(f64::from(mission.mission_id)),
    );
    table_set_static(state, result, "name", name);
    table_set_static(
        state,
        result,
        "followerTypeID",
        Val::Num(f64::from(mission.follower_type_id)),
    );
    table_set_static(state, result, "level", Val::Num(f64::from(mission.level)));
    table_set_static(
        state,
        result,
        "durationSeconds",
        Val::Num(mission.duration_seconds),
    );
    table_set_static(state, result, "duration", duration);
    table_set_static(state, result, "timeLeft", time_left);
    table_set_static(state, result, "xp", Val::Num(mission.xp));
    table_set_static(
        state,
        result,
        "canBeCompleted",
        Val::Bool(mission.can_be_completed),
    );
    table_set_static(state, result, "inProgress", Val::Bool(mission.in_progress));
    table_set_static(state, result, "completed", Val::Bool(mission.completed));
    let rewards = publish_rewards(state, &mission.rewards)?;
    table_set_static(state, result, "rewards", rewards);
    Ok(result)
}

fn publish_rewards(state: &mut LuaState, rewards: &[GarrisonMissionReward]) -> LuaResult<Val> {
    let result = create_table_with_capacity(state, rewards.len());
    for (index, reward) in rewards.iter().enumerate() {
        let value = publish_reward(state, reward);
        set_table_array(state, result, (index + 1) as i64, value);
    }
    Ok(result)
}

fn publish_reward(state: &mut LuaState, reward: &GarrisonMissionReward) -> Val {
    let result = create_table_with_capacity(state, 3);
    set_optional_number(state, result, "itemID", reward.item_id);
    set_optional_number(state, result, "currencyID", reward.currency_id);
    set_optional_number(state, result, "quantity", reward.quantity);
    result
}

fn set_optional_number(state: &mut LuaState, table: Val, key: &'static str, value: Option<u32>) {
    if let Some(value) = value {
        table_set_static(state, table, key, Val::Num(f64::from(value)));
    }
}

fn clear_table(state: &mut LuaState, table_ref: GcRef<Table>) -> LuaResult<()> {
    let keys = table_keys(state, table_ref)?;
    let tables = &mut state.gc.tables;
    let strings = &state.gc.string_arena;
    if let Some(table) = tables.get_mut(table_ref) {
        for key in keys {
            table.raw_set(key, Val::Nil, strings)?;
        }
    }
    Ok(())
}

fn table_keys(state: &LuaState, table_ref: GcRef<Table>) -> LuaResult<Vec<Val>> {
    let Some(table) = state.gc.tables.get(table_ref) else {
        return Err(runtime_error("mission result table is unavailable"));
    };
    let mut keys = Vec::new();
    let mut key = Val::Nil;
    while let Some((next_key, _)) = table.next(key, &state.gc.string_arena)? {
        keys.push(next_key);
        key = next_key;
    }
    Ok(keys)
}

fn parse_missions(state: &mut LuaState, value: Val) -> LuaResult<Vec<GarrisonMission>> {
    let Val::Table(array) = value else {
        return Err(runtime_error(INPUT_ERROR));
    };
    let length = state
        .gc
        .tables
        .get(array)
        .ok_or_else(|| runtime_error(INPUT_ERROR))?
        .len(&state.gc.string_arena);
    (1..=length)
        .map(|index| {
            let entry = state
                .gc
                .tables
                .get(array)
                .map(|table| table.get_int(index as i64))
                .unwrap_or(Val::Nil);
            parse_mission(state, entry)
        })
        .collect()
}

fn parse_mission(state: &mut LuaState, value: Val) -> LuaResult<GarrisonMission> {
    let Val::Table(table) = value else {
        return Err(runtime_error(INPUT_ERROR));
    };
    let reward_values = field(state, table, "rewards");
    let rewards = parse_rewards(state, reward_values)?;
    let encounter_icon_info = parse_optional_encounter_icon_info(state, table)?;
    Ok(GarrisonMission {
        mission_id: read_field_u32(state, table, "missionID")?,
        name: read_field_string(state, table, "name")?,
        follower_type_id: read_field_u32(state, table, "followerTypeID")?,
        level: read_field_u32(state, table, "level")?,
        duration_seconds: read_field_number(state, table, "durationSeconds")?,
        duration: read_field_string(state, table, "duration")?,
        time_left: read_field_string(state, table, "timeLeft")?,
        xp: read_field_number(state, table, "xp")?,
        can_be_completed: read_field_bool(state, table, "canBeCompleted")?,
        rewards,
        in_progress: read_field_bool(state, table, "inProgress")?,
        completed: read_field_bool(state, table, "completed")?,
        encounter_icon_info,
    })
}

fn parse_optional_encounter_icon_info(
    state: &mut LuaState,
    mission: GcRef<Table>,
) -> LuaResult<Option<GarrisonMissionEncounterIconInfo>> {
    let value = field(state, mission, "encounterIconInfo");
    let Val::Table(info) = value else {
        return if matches!(value, Val::Nil) {
            Ok(None)
        } else {
            Err(invalid_field("encounterIconInfo"))
        };
    };
    Ok(Some(GarrisonMissionEncounterIconInfo {
        portrait_file_data_id: read_field_u32(state, info, "portraitFileDataID")?,
        mission_scalar: read_field_number(state, info, "missionScalar")?,
        is_elite: read_field_bool(state, info, "isElite")?,
        is_rare: read_field_bool(state, info, "isRare")?,
    }))
}

fn parse_follower_quality_table(state: &LuaState, value: Val) -> LuaResult<Vec<(u32, f64)>> {
    let Val::Table(table_ref) = value else {
        return Err(invalid_field("followerQualityTable"));
    };
    let table = state
        .gc
        .tables
        .get(table_ref)
        .ok_or_else(|| invalid_field("followerQualityTable"))?;
    let mut entries = Vec::new();
    let mut key = Val::Nil;
    while let Some((next_key, xp)) = table.next(key, &state.gc.string_arena)? {
        let quality = read_u32(next_key).ok_or_else(|| invalid_field("followerQuality"))?;
        let xp = match xp {
            Val::Num(value) if value.is_finite() && value >= 0.0 => value,
            _ => return Err(invalid_field("followerQualityXP")),
        };
        entries.push((quality, xp));
        key = next_key;
    }
    entries.sort_by_key(|(quality, _)| *quality);
    Ok(entries)
}

fn parse_rewards(state: &mut LuaState, value: Val) -> LuaResult<Vec<GarrisonMissionReward>> {
    let Val::Table(array) = value else {
        return Err(invalid_field("rewards"));
    };
    let length = state
        .gc
        .tables
        .get(array)
        .ok_or_else(|| invalid_field("rewards"))?
        .len(&state.gc.string_arena);
    (1..=length)
        .map(|index| {
            let entry = state
                .gc
                .tables
                .get(array)
                .map(|table| table.get_int(index as i64))
                .unwrap_or(Val::Nil);
            parse_reward(state, entry)
        })
        .collect()
}

fn parse_reward(state: &mut LuaState, value: Val) -> LuaResult<GarrisonMissionReward> {
    let Val::Table(table) = value else {
        return Err(invalid_field("reward"));
    };
    Ok(GarrisonMissionReward {
        item_id: read_optional_u32(state, table, "itemID")?,
        currency_id: read_optional_u32(state, table, "currencyID")?,
        quantity: read_optional_u32(state, table, "quantity")?,
    })
}

fn read_field_u32(state: &mut LuaState, table: GcRef<Table>, key: &'static str) -> LuaResult<u32> {
    read_u32(field(state, table, key)).ok_or_else(|| invalid_field(key))
}

fn read_optional_u32(
    state: &mut LuaState,
    table: GcRef<Table>,
    key: &'static str,
) -> LuaResult<Option<u32>> {
    match field(state, table, key) {
        Val::Nil => Ok(None),
        value => read_u32(value).map(Some).ok_or_else(|| invalid_field(key)),
    }
}

fn read_u32(value: Val) -> Option<u32> {
    match value {
        Val::Num(number)
            if number.is_finite()
                && number.fract() == 0.0
                && (0.0..=f64::from(u32::MAX)).contains(&number) =>
        {
            Some(number as u32)
        }
        _ => None,
    }
}

fn read_field_number(
    state: &mut LuaState,
    table: GcRef<Table>,
    key: &'static str,
) -> LuaResult<f64> {
    match field(state, table, key) {
        Val::Num(number) if number.is_finite() && number >= 0.0 => Ok(number),
        _ => Err(invalid_field(key)),
    }
}

fn read_field_string(
    state: &mut LuaState,
    table: GcRef<Table>,
    key: &'static str,
) -> LuaResult<String> {
    let value = field(state, table, key);
    val_to_string(state, value).ok_or_else(|| invalid_field(key))
}

fn read_field_bool(
    state: &mut LuaState,
    table: GcRef<Table>,
    key: &'static str,
) -> LuaResult<bool> {
    match field(state, table, key) {
        Val::Bool(value) => Ok(value),
        _ => Err(invalid_field(key)),
    }
}

fn field(state: &mut LuaState, table: GcRef<Table>, key: &'static str) -> Val {
    table_get(state, Val::Table(table), key)
}

fn invalid_field(key: &str) -> rilua::LuaError {
    runtime_error(format!("A_Admin.SetGarrisonMissions: invalid {key}"))
}
