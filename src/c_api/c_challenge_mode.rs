//! Last challenge completion snapshot, independent of historical weekly bests.

use crate::c_api::ensure_namespace;
use crate::lua_api::methods::{
    borrow_state, create_string, create_table, create_table_with_capacity, table_set_num,
    table_set_static,
};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

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
    )
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
        table_set_num(state, members, (index + 1) as f64, row);
        let guid = create_string(state, &member.member_guid);
        table_set_static(state, row, "memberGUID", guid);
        let name = create_string(state, &member.name);
        table_set_static(state, row, "name", name);
    }
    Ok(1)
}

fn set_completion_fields(state: &mut LuaState, info: Val, completion: &ChallengeCompletionInfo) {
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
    for (name, value) in [
        ("onTime", completion.on_time),
        ("practiceRun", completion.practice_run),
        ("isMapRecord", completion.is_map_record),
        ("isAffixRecord", completion.is_affix_record),
        ("isEligibleForScore", completion.is_eligible_for_score),
    ] {
        table_set_static(state, info, name, Val::Bool(value));
    }
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
