use super::*;
use rilua::vm::gc::arena::GcRef;
use rilua::vm::table::Table;

pub(super) fn register_legacy_achievement_globals(state: &mut LuaState) -> LuaResult<()> {
    let globals = state.global;
    table_set_rust_fn_static(
        state,
        globals,
        "GetAchievementInfo",
        get_achievement_info_global,
    )?;
    register_category_globals(state, globals)?;
    register_criteria_globals(state, globals)?;
    register_traversal_globals(state, globals)?;
    register_summary_globals(state, globals)?;
    register_comparison_globals(state, globals)?;
    register_guild_member_globals(state, globals)?;
    register_search_globals(state, globals)?;
    register_focus_globals(state, globals)?;
    Ok(())
}

fn register_focus_globals(state: &mut LuaState, globals: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        globals,
        "SetFocusedAchievement",
        set_focused_achievement,
    )?;
    Ok(())
}

fn register_search_globals(state: &mut LuaState, globals: GcRef<Table>) -> LuaResult<()> {
    register_search_setter(state, globals)?;
    register_search_getters(state, globals)?;
    Ok(())
}

fn register_search_setter(state: &mut LuaState, globals: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        globals,
        "SetAchievementSearchString",
        set_achievement_search_string,
    )?;
    Ok(())
}

fn register_search_getters(state: &mut LuaState, globals: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        globals,
        "GetAchievementSearchProgress",
        get_achievement_search_progress,
    )?;
    table_set_rust_fn_static(
        state,
        globals,
        "GetAchievementSearchSize",
        get_achievement_search_size,
    )?;
    table_set_rust_fn_static(
        state,
        globals,
        "GetNumFilteredAchievements",
        get_num_filtered_achievements,
    )?;
    table_set_rust_fn_static(
        state,
        globals,
        "GetFilteredAchievementID",
        get_filtered_achievement_id,
    )?;
    Ok(())
}

fn register_guild_member_globals(state: &mut LuaState, globals: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        globals,
        "GetGuildAchievementNumMembers",
        get_guild_achievement_num_members,
    )?;
    table_set_rust_fn_static(
        state,
        globals,
        "GetGuildAchievementMembers",
        get_guild_achievement_members,
    )?;
    table_set_rust_fn_static(
        state,
        globals,
        "GetGuildAchievementMemberInfo",
        get_guild_achievement_member_info,
    )?;
    Ok(())
}

fn register_comparison_globals(state: &mut LuaState, globals: GcRef<Table>) -> LuaResult<()> {
    register_comparison_unit_mutators(state, globals)?;
    register_comparison_getters(state, globals)?;
    Ok(())
}

fn register_comparison_unit_mutators(state: &mut LuaState, globals: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        globals,
        "SetAchievementComparisonUnit",
        set_achievement_comparison_unit,
    )?;
    table_set_rust_fn_static(
        state,
        globals,
        "ClearAchievementComparisonUnit",
        clear_achievement_comparison_unit,
    )?;
    Ok(())
}

fn register_comparison_getters(state: &mut LuaState, globals: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        globals,
        "GetAchievementComparisonInfo",
        get_achievement_comparison_info,
    )?;
    table_set_rust_fn_static(
        state,
        globals,
        "GetComparisonAchievementPoints",
        get_comparison_achievement_points,
    )?;
    table_set_rust_fn_static(
        state,
        globals,
        "GetComparisonCategoryNumAchievements",
        get_comparison_category_num_achievements,
    )?;
    table_set_rust_fn_static(
        state,
        globals,
        "GetComparisonStatistic",
        get_comparison_statistic,
    )?;
    Ok(())
}

fn register_summary_globals(state: &mut LuaState, globals: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        globals,
        "GetAchievementGuildRep",
        get_achievement_guild_rep,
    )?;
    table_set_rust_fn_static(
        state,
        globals,
        "GetNumCompletedAchievements",
        get_num_completed_achievements,
    )?;
    table_set_rust_fn_static(
        state,
        globals,
        "GetTotalAchievementPoints",
        get_total_achievement_points,
    )?;
    table_set_rust_fn_static(
        state,
        globals,
        "GetCategoryAchievementPoints",
        get_category_achievement_points,
    )?;
    table_set_rust_fn_static(state, globals, "GetAchievementLink", get_achievement_link)?;
    table_set_rust_fn_static(state, globals, "GetStatistic", get_statistic)?;
    Ok(())
}

fn register_category_globals(state: &mut LuaState, globals: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(state, globals, "GetCategoryList", get_category_list)?;
    table_set_rust_fn_static(
        state,
        globals,
        "GetGuildCategoryList",
        get_guild_category_list,
    )?;
    table_set_rust_fn_static(
        state,
        globals,
        "GetStatisticsCategoryList",
        get_statistics_category_list,
    )?;
    table_set_rust_fn_static(state, globals, "GetCategoryInfo", get_category_info)?;
    table_set_rust_fn_static(
        state,
        globals,
        "GetAchievementCategory",
        get_achievement_category,
    )?;
    table_set_rust_fn_static(
        state,
        globals,
        "GetCategoryNumAchievements",
        get_category_num_achievements,
    )?;
    Ok(())
}

fn register_criteria_globals(state: &mut LuaState, globals: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        globals,
        "GetAchievementNumCriteria",
        get_achievement_num_criteria,
    )?;
    table_set_rust_fn_static(
        state,
        globals,
        "GetAchievementCriteriaInfo",
        get_achievement_criteria_info,
    )?;
    table_set_rust_fn_static(
        state,
        globals,
        "GetAchievementCriteriaInfoByID",
        get_achievement_criteria_info_by_id,
    )?;
    Ok(())
}

fn register_traversal_globals(state: &mut LuaState, globals: GcRef<Table>) -> LuaResult<()> {
    table_set_rust_fn_static(
        state,
        globals,
        "GetPreviousAchievement",
        get_previous_achievement,
    )?;
    table_set_rust_fn_static(state, globals, "GetNextAchievement", get_next_achievement)?;
    table_set_rust_fn_static(
        state,
        globals,
        "GetLatestCompletedAchievements",
        get_latest_completed_achievements,
    )?;
    Ok(())
}
