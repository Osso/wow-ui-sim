//! `C_RecruitAFriend` probe surface with disabled-by-default RAF data.

use super::{ensure_namespace, set_table_array};
use crate::lua_api::methods::{create_table, table_set};
use crate::lua_bridge::table_set_rust_fn_static;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(super) fn register_recruit_a_friend_surface(state: &mut LuaState) -> LuaResult<()> {
    let ns = ensure_namespace(state, "C_RecruitAFriend")?;
    table_set_rust_fn_static(state, ns, "GetRAFInfo", get_raf_info)?;
    table_set_rust_fn_static(state, ns, "GetRAFSystemInfo", get_raf_system_info)?;
    table_set_rust_fn_static(state, ns, "GetRecruitInfo", get_recruit_info)?;
    table_set_rust_fn_static(
        state,
        ns,
        "GetSummonFriendCooldown",
        get_summon_friend_cooldown,
    )?;
    #[cfg(not(feature = "retail-12-1-0"))]
    table_set_rust_fn_static(state, ns, "IsEnabled", is_enabled)?;
    #[cfg(feature = "retail-12-1-0")]
    table_set_rust_fn_static(state, ns, "IsSystemEnabled", is_enabled)?;
    #[cfg(feature = "retail-12-1-0")]
    table_set_rust_fn_static(state, ns, "IsSystemSupported", is_system_supported)?;
    table_set_rust_fn_static(state, ns, "IsRecruitingEnabled", is_recruiting_enabled)?;
    table_set_rust_fn_static(state, ns, "CanSummonFriend", can_summon_friend)?;
    table_set_rust_fn_static(state, ns, "SummonFriend", summon_friend)?;
    Ok(())
}

fn get_raf_system_info(state: &mut LuaState) -> LuaResult<u32> {
    let info = create_table(state);
    table_set(state, info, "maxRecruits", Val::Num(0.0));
    table_set(state, info, "maxRecruitMonths", Val::Num(0.0));
    table_set(state, info, "maxRecruitmentUses", Val::Num(0.0));
    table_set(state, info, "daysInCycle", Val::Num(0.0));
    state.push(info);
    Ok(1)
}

fn get_raf_info(state: &mut LuaState) -> LuaResult<u32> {
    let info = create_table(state);
    let versions = empty_versions(state);
    table_set(state, info, "versions", versions);
    table_set(state, info, "recruitmentInfo", Val::Nil);
    let recruits = empty_recruits(state);
    table_set(state, info, "recruits", recruits);
    table_set(state, info, "claimInProgress", Val::Bool(false));
    state.push(info);
    Ok(1)
}

fn empty_versions(state: &mut LuaState) -> Val {
    let versions = create_table(state);
    let sample = create_table(state);
    let month_count = create_table(state);
    table_set(state, month_count, "lifetimeMonths", Val::Num(0.0));
    table_set(state, month_count, "spentMonths", Val::Num(0.0));
    table_set(state, month_count, "availableMonths", Val::Num(0.0));
    table_set(state, sample, "rafVersion", Val::Num(1.0));
    table_set(state, sample, "monthCount", month_count);
    let rewards = create_table(state);
    table_set(state, sample, "rewards", rewards);
    table_set(state, sample, "nextReward", Val::Nil);
    table_set(state, sample, "numRecruits", Val::Num(0.0));
    table_set(state, sample, "numAffordableRewards", Val::Num(0.0));
    set_table_array(state, versions, 1, sample);
    versions
}

fn empty_recruits(state: &mut LuaState) -> Val {
    create_table(state)
}

fn is_enabled(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

#[cfg(feature = "retail-12-1-0")]
fn is_system_supported(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

fn is_recruiting_enabled(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    Ok(1)
}

fn get_recruit_info(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    state.push(Val::Nil);
    Ok(2)
}

fn get_summon_friend_cooldown(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    Ok(2)
}

fn can_summon_friend(state: &mut LuaState) -> LuaResult<u32> {
    state.push(Val::Bool(false));
    state.push(Val::Nil);
    Ok(2)
}

fn summon_friend(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}
