//! `C_AchievementInfo` probe surface backed by `SimState.achievements`
//! and `WorldState.earned_achievements`, plus legacy global achievement
//! helpers still used by older Blizzard code and tests.
//!
//! Migrates `C_AchievementInfo.GetAchievementInfo`,
//! `C_AchievementInfo.GetRewardItemID`, and
//! `C_AchievementInfo.IsValidAchievement` off the namespace stubs, then
//! layers the legacy globals (`GetAchievementInfo`,
//! `GetCategoryList`, `GetCategoryInfo`,
//! `GetCategoryNumAchievements`, `GetAchievementNumCriteria`,
//! `GetAchievementCriteriaInfo`) on top of the same seeded data.

mod categories;
mod category_points;
mod registration;

use super::{ensure_namespace, set_table_array};
use crate::event::Event;
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, frame_id_from_stack, val_to_string,
};
use crate::lua_api::state::AchievementInfo;
use crate::lua_bridge::{FromStack, stack_val, table_set_rust_fn_static};
use categories::{
    AchievementCriterion, CategoryListKind, achievement_categories, categories_for,
    categories_for_view, category_id_for_achievement, collect_category_achievement_ids,
    criteria_for_achievement, criterion_at, criterion_by_id, find_category, next_achievement_id,
    previous_achievement_id,
};
use category_points::get_category_achievement_points;
use registration::register_legacy_achievement_globals;
use rilua::vm::state::LuaState;
use rilua::vm::table::Table;
use rilua::{LuaResult, Val};
use std::collections::HashSet;

const DEFAULT_PORTRAIT_PATH: &str = "Interface\\Icons\\Achievement_Character_Default";

const COMPLETION_MONTH: f64 = 1.0;
const COMPLETION_DAY: f64 = 15.0;
const COMPLETION_YEAR: f64 = 2025.0;

pub(super) fn register_achievement_info_surface(state: &mut LuaState) -> LuaResult<()> {
    let table_ref = ensure_namespace(state, "C_AchievementInfo")?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetAchievementInfo",
        c_achievement_info_get_achievement_info,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "GetRewardItemID",
        c_achievement_info_get_reward_item_id,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "IsValidAchievement",
        c_achievement_info_is_valid_achievement,
    )?;
    table_set_rust_fn_static(
        state,
        table_ref,
        "SetPortraitTexture",
        c_achievement_info_set_portrait_texture,
    )?;
    register_legacy_achievement_globals(state)?;
    Ok(())
}

fn c_achievement_info_get_achievement_info(state: &mut LuaState) -> LuaResult<u32> {
    let achievement_id = i32::from_stack(state, 1)?;
    let exists = {
        let sim = borrow_state(state)?;
        sim.achievements.contains_key(&achievement_id)
    };
    if !exists {
        return Ok(0);
    }
    push_achievement_info_for_id(state, achievement_id)
}

fn c_achievement_info_get_reward_item_id(state: &mut LuaState) -> LuaResult<u32> {
    let achievement_id = i32::from_stack(state, 1)?;
    let reward = borrow_state(state)?
        .achievements
        .get(&achievement_id)
        .and_then(|a| a.reward_item_id);
    match reward {
        Some(id) => state.push(Val::Num(id as f64)),
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn c_achievement_info_is_valid_achievement(state: &mut LuaState) -> LuaResult<u32> {
    let achievement_id = i32::from_stack(state, 1)?;
    let valid = borrow_state(state)?
        .achievements
        .contains_key(&achievement_id);
    state.push(Val::Bool(valid));
    Ok(1)
}

fn c_achievement_info_set_portrait_texture(state: &mut LuaState) -> LuaResult<u32> {
    let Ok(texture_id) = frame_id_from_stack(state, 1) else {
        state.push(Val::Bool(false));
        return Ok(1);
    };
    let mut sim = borrow_state_mut(state)?;
    let Some(frame) = sim.widgets.get_mut_visual(texture_id) else {
        drop(sim);
        state.push(Val::Bool(false));
        return Ok(1);
    };
    frame.texture = Some(DEFAULT_PORTRAIT_PATH.to_string());
    frame.texture_file_data_id = None;
    frame.color_texture = None;
    frame.atlas = None;
    frame.atlas_tex_coords = None;
    drop(sim);
    state.push(Val::Bool(true));
    Ok(1)
}

fn get_achievement_info_global(state: &mut LuaState) -> LuaResult<u32> {
    let nargs = (state.top as i32 - state.base as i32).max(0);
    let first_arg = i32::from_stack(state, 1)?;

    if nargs >= 2 {
        let index = i32::from_stack(state, 2)?;
        let Some(category) = find_category(first_arg) else {
            return Ok(0);
        };
        let achievement_ids = collect_category_achievement_ids(category.category_id);
        let Some(achievement_id) = achievement_ids.get((index - 1).max(0) as usize).copied() else {
            return Ok(0);
        };
        return push_achievement_info_for_id(state, achievement_id);
    }

    if let Some(category) = find_category(first_arg) {
        let name = create_string(state, category.name);
        let empty = create_string(state, "");
        state.push(Val::Num(category.category_id as f64));
        state.push(name);
        state.push(Val::Num(0.0));
        state.push(Val::Bool(false));
        state.push(Val::Num(0.0));
        state.push(Val::Num(0.0));
        state.push(Val::Num(0.0));
        state.push(empty.clone());
        state.push(Val::Num(category.flags as f64));
        state.push(Val::Num(0.0));
        state.push(empty.clone());
        state.push(Val::Bool(false));
        state.push(Val::Bool(false));
        state.push(empty);
        state.push(Val::Bool(false));
        return Ok(15);
    }

    push_achievement_info_for_id(state, first_arg)
}

fn get_category_list(state: &mut LuaState) -> LuaResult<u32> {
    push_category_list(state, CategoryListKind::Achievement);
    Ok(1)
}

fn get_guild_category_list(state: &mut LuaState) -> LuaResult<u32> {
    push_category_list(state, CategoryListKind::Guild);
    Ok(1)
}

fn get_statistics_category_list(state: &mut LuaState) -> LuaResult<u32> {
    push_category_list(state, CategoryListKind::Statistics);
    Ok(1)
}

fn get_category_info(state: &mut LuaState) -> LuaResult<u32> {
    let category_id = i32::from_stack(state, 1)?;
    let Some(category) = find_category(category_id) else {
        return Ok(0);
    };
    let name = create_string(state, category.name);
    state.push(name);
    state.push(Val::Num(category.parent_id as f64));
    state.push(Val::Num(category.flags as f64));
    Ok(3)
}

fn get_achievement_category(state: &mut LuaState) -> LuaResult<u32> {
    let achievement_id = i32::from_stack(state, 1)?;
    match category_id_for_achievement(achievement_id) {
        Some(category_id) => state.push(Val::Num(category_id as f64)),
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn get_category_num_achievements(state: &mut LuaState) -> LuaResult<u32> {
    let category_id = i32::from_stack(state, 1)?;
    let Some(category) = find_category(category_id) else {
        push_category_counts(state, 0, 0);
        return Ok(3);
    };
    let achievement_ids = collect_category_achievement_ids(category.category_id);
    let completed = count_completed_achievements(state, &achievement_ids)?;
    push_category_counts(state, achievement_ids.len() as i32, completed);
    Ok(3)
}

fn get_achievement_num_criteria(state: &mut LuaState) -> LuaResult<u32> {
    let criteria_len = criteria_for_achievement(i32::from_stack(state, 1)?)
        .map(|criteria| criteria.len())
        .unwrap_or_default();
    state.push(Val::Num(criteria_len as f64));
    Ok(1)
}

fn get_achievement_criteria_info(state: &mut LuaState) -> LuaResult<u32> {
    let achievement_id = i32::from_stack(state, 1)?;
    let criterion_index = i32::from_stack(state, 2)?;
    push_criterion_result(
        state,
        achievement_id,
        criterion_at(achievement_id, criterion_index),
    )
}

fn get_achievement_criteria_info_by_id(state: &mut LuaState) -> LuaResult<u32> {
    let achievement_id = i32::from_stack(state, 1)?;
    let criterion_id = i32::from_stack(state, 2)?;
    push_criterion_result(
        state,
        achievement_id,
        criterion_by_id(achievement_id, criterion_id),
    )
}

fn push_criterion_result(
    state: &mut LuaState,
    achievement_id: i32,
    criterion: Option<&AchievementCriterion>,
) -> LuaResult<u32> {
    let Some(criterion) = criterion else {
        state.push(Val::Nil);
        return Ok(1);
    };
    push_criterion_multiret(state, achievement_id, criterion)?;
    Ok(13)
}

fn get_previous_achievement(state: &mut LuaState) -> LuaResult<u32> {
    let achievement_id = i32::from_stack(state, 1)?;
    match previous_achievement_id(achievement_id) {
        Some(previous_id) => state.push(Val::Num(previous_id as f64)),
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn get_next_achievement(state: &mut LuaState) -> LuaResult<u32> {
    let achievement_id = i32::from_stack(state, 1)?;
    match next_achievement_id(achievement_id) {
        Some(next_id) => {
            let completed = {
                let sim = borrow_state(state)?;
                is_achievement_completed(&sim.world.earned_achievements, next_id)
            };
            state.push(Val::Num(next_id as f64));
            state.push(Val::Bool(completed));
        }
        None => {
            state.push(Val::Nil);
            state.push(Val::Bool(false));
        }
    }
    Ok(2)
}

fn get_latest_completed_achievements(state: &mut LuaState) -> LuaResult<u32> {
    let _guild_view = bool::from_stack(state, 1).unwrap_or(false);
    let mut earned_ids = {
        let sim = borrow_state(state)?;
        sim.world
            .earned_achievements
            .iter()
            .copied()
            .collect::<Vec<_>>()
    };
    earned_ids.sort_unstable();
    let count = earned_ids.len() as u32;

    for achievement_id in &earned_ids {
        state.push(Val::Num((*achievement_id) as f64));
    }
    Ok(count)
}

fn get_achievement_guild_rep(state: &mut LuaState) -> LuaResult<u32> {
    let achievement_id = i32::from_stack(state, 1)?;
    let entry = borrow_state(state)?
        .achievement_guild_rep
        .get(&achievement_id)
        .cloned()
        .unwrap_or_default();
    state.push(Val::Bool(entry.requires_rep));
    state.push(Val::Bool(entry.has_rep));
    state.push(match entry.rep_level {
        Some(level) => Val::Num(level as f64),
        None => Val::Nil,
    });
    Ok(3)
}

fn get_statistic(state: &mut LuaState) -> LuaResult<u32> {
    let nargs = (state.top as i32 - state.base as i32).max(0);
    if nargs >= 2 {
        state.push(Val::Nil);
        state.push(Val::Bool(true));
        state.push(Val::Nil);
        return Ok(3);
    }
    let achievement_id = i32::from_stack(state, 1)?;
    let entry = borrow_state(state)?
        .achievement_statistics
        .get(&achievement_id)
        .cloned();
    match entry {
        Some(stat) => {
            let quantity = create_string(state, &stat.quantity);
            state.push(quantity);
            state.push(Val::Bool(stat.is_counter));
        }
        None => {
            state.push(Val::Nil);
            state.push(Val::Bool(false));
        }
    }
    Ok(2)
}

fn get_num_completed_achievements(state: &mut LuaState) -> LuaResult<u32> {
    let is_guild_view = bool::from_stack(state, 1).unwrap_or(false);
    let categories = categories_for_view(is_guild_view);
    let total: i32 = categories
        .iter()
        .map(|category| category.achievement_ids.len() as i32)
        .sum();
    let completed = {
        let sim = borrow_state(state)?;
        categories
            .iter()
            .flat_map(|category| category.achievement_ids.iter())
            .filter(|achievement_id| {
                is_achievement_completed(&sim.world.earned_achievements, **achievement_id)
            })
            .count() as i32
    };
    state.push(Val::Num(total as f64));
    state.push(Val::Num(completed as f64));
    Ok(2)
}

fn get_achievement_link(state: &mut LuaState) -> LuaResult<u32> {
    let Some(achievement_id) = achievement_id_from_stack(state, 1) else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let name = borrow_state(state)?
        .achievements
        .get(&achievement_id)
        .map(|info| info.name.clone());
    let Some(name) = name else {
        state.push(Val::Nil);
        return Ok(1);
    };
    let link = format!(
        "|cffffff00|Hachievement:{achievement_id}:Player-1-00000001:1:1:15:2025:0:0:0:0|h[{name}]|h|r"
    );
    let link_val = create_string(state, &link);
    state.push(link_val);
    Ok(1)
}

fn achievement_id_from_stack(state: &LuaState, index: i32) -> Option<i32> {
    let value = stack_val(state, index);
    match value {
        Val::Num(id) => Some(id as i32),
        Val::Str(_) => val_to_string(state, value)?.parse::<i32>().ok(),
        _ => None,
    }
}

fn set_achievement_comparison_unit(state: &mut LuaState) -> LuaResult<u32> {
    let unit = String::from_stack(state, 1).ok().filter(|s| !s.is_empty());
    let Some(unit) = unit else {
        state.push(Val::Bool(false));
        return Ok(1);
    };
    {
        let mut sim = borrow_state_mut(state)?;
        sim.achievement_comparison_unit = Some(unit);
        sim.events.push(Event {
            name: "INSPECT_ACHIEVEMENT_READY".to_string(),
            args: Vec::new(),
        });
    }
    state.push(Val::Bool(true));
    Ok(1)
}

fn clear_achievement_comparison_unit(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.achievement_comparison_unit = None;
    Ok(0)
}

fn get_achievement_comparison_info(state: &mut LuaState) -> LuaResult<u32> {
    let achievement_id = i32::from_stack(state, 1)?;
    let date = {
        let sim = borrow_state(state)?;
        if sim.achievement_comparison_unit.is_none() {
            None
        } else if !sim
            .achievement_comparison_data
            .earned
            .contains(&achievement_id)
        {
            None
        } else {
            Some(
                sim.achievement_comparison_data
                    .completion_dates
                    .get(&achievement_id)
                    .copied()
                    .unwrap_or((0, 0, 0)),
            )
        }
    };
    let Some((month, day, year)) = date else {
        state.push(Val::Bool(false));
        state.push(Val::Nil);
        state.push(Val::Nil);
        state.push(Val::Nil);
        return Ok(4);
    };
    state.push(Val::Bool(true));
    state.push(Val::Num(month as f64));
    state.push(Val::Num(day as f64));
    state.push(Val::Num(year as f64));
    Ok(4)
}

fn get_comparison_achievement_points(state: &mut LuaState) -> LuaResult<u32> {
    let total: i32 = {
        let sim = borrow_state(state)?;
        if sim.achievement_comparison_unit.is_none() {
            0
        } else {
            achievement_categories()
                .iter()
                .flat_map(|category| category.achievement_ids.iter())
                .filter(|id| sim.achievement_comparison_data.earned.contains(id))
                .filter_map(|id| sim.achievements.get(id))
                .map(|info| info.points)
                .sum()
        }
    };
    state.push(Val::Num(total as f64));
    Ok(1)
}

fn get_comparison_category_num_achievements(state: &mut LuaState) -> LuaResult<u32> {
    let category_id = i32::from_stack(state, 1)?;
    let achievement_ids = collect_category_achievement_ids(category_id);
    let completed = {
        let sim = borrow_state(state)?;
        if sim.achievement_comparison_unit.is_none() {
            0
        } else {
            achievement_ids
                .iter()
                .filter(|id| sim.achievement_comparison_data.earned.contains(id))
                .count() as i32
        }
    };
    state.push(Val::Num(completed as f64));
    Ok(1)
}

fn get_comparison_statistic(state: &mut LuaState) -> LuaResult<u32> {
    let achievement_id = i32::from_stack(state, 1)?;
    let quantity = {
        let sim = borrow_state(state)?;
        if sim.achievement_comparison_unit.is_none() {
            None
        } else {
            sim.achievement_comparison_data
                .statistics
                .get(&achievement_id)
                .cloned()
        }
    };
    match quantity {
        Some(text) => {
            let val = create_string(state, &text);
            state.push(val);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn get_guild_achievement_num_members(state: &mut LuaState) -> LuaResult<u32> {
    let achievement_id = i32::from_stack(state, 1)?;
    let count = borrow_state(state)?
        .guild_achievement_members
        .get(&achievement_id)
        .map(|members| members.len() as i32)
        .unwrap_or(0);
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn get_guild_achievement_members(_state: &mut LuaState) -> LuaResult<u32> {
    Ok(0)
}

fn get_guild_achievement_member_info(state: &mut LuaState) -> LuaResult<u32> {
    let achievement_id = i32::from_stack(state, 1)?;
    let index = i32::from_stack(state, 2)?;
    let name = {
        let sim = borrow_state(state)?;
        sim.guild_achievement_members
            .get(&achievement_id)
            .and_then(|members| {
                usize::try_from(index - 1)
                    .ok()
                    .and_then(|idx| members.get(idx))
            })
            .cloned()
    };
    match name {
        Some(text) => {
            let val = create_string(state, &text);
            state.push(val);
        }
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn set_achievement_search_string(state: &mut LuaState) -> LuaResult<u32> {
    let query = String::from_stack(state, 1).unwrap_or_default();
    let mut sim = borrow_state_mut(state)?;
    let filtered_ids = match_achievements_by_query(&sim.achievements, &query);
    let count = filtered_ids.len() as i32;
    sim.achievement_search.query = query;
    sim.achievement_search.progress = count;
    sim.achievement_search.size = count;
    sim.achievement_search.filtered_ids = filtered_ids;
    drop(sim);
    state.push(Val::Bool(true));
    Ok(1)
}

fn match_achievements_by_query(
    achievements: &::std::collections::HashMap<i32, AchievementInfo>,
    query: &str,
) -> Vec<i32> {
    if query.is_empty() {
        return Vec::new();
    }
    let needle = query.to_lowercase();
    let mut matches: Vec<i32> = achievements
        .values()
        .filter(|info| info.name.to_lowercase().contains(&needle))
        .map(|info| info.achievement_id)
        .collect();
    matches.sort();
    matches
}

fn get_achievement_search_progress(state: &mut LuaState) -> LuaResult<u32> {
    let progress = borrow_state(state)?.achievement_search.progress;
    state.push(Val::Num(progress as f64));
    Ok(1)
}

fn get_achievement_search_size(state: &mut LuaState) -> LuaResult<u32> {
    let size = borrow_state(state)?.achievement_search.size;
    state.push(Val::Num(size as f64));
    Ok(1)
}

fn get_num_filtered_achievements(state: &mut LuaState) -> LuaResult<u32> {
    let count = borrow_state(state)?.achievement_search.filtered_ids.len() as i32;
    state.push(Val::Num(count as f64));
    Ok(1)
}

fn get_filtered_achievement_id(state: &mut LuaState) -> LuaResult<u32> {
    let index = i32::from_stack(state, 1)?;
    let id = {
        let sim = borrow_state(state)?;
        usize::try_from(index - 1)
            .ok()
            .and_then(|idx| sim.achievement_search.filtered_ids.get(idx).copied())
    };
    match id {
        Some(value) => state.push(Val::Num(value as f64)),
        None => state.push(Val::Nil),
    }
    Ok(1)
}

fn set_focused_achievement(state: &mut LuaState) -> LuaResult<u32> {
    let achievement_id = i32::from_stack(state, 1).ok();
    borrow_state_mut(state)?.focused_achievement = achievement_id;
    Ok(0)
}

fn get_total_achievement_points(state: &mut LuaState) -> LuaResult<u32> {
    let is_guild_view = bool::from_stack(state, 1).unwrap_or(false);
    let categories = categories_for_view(is_guild_view);
    let total_points: i32 = {
        let sim = borrow_state(state)?;
        categories
            .iter()
            .flat_map(|category| category.achievement_ids.iter())
            .filter(|achievement_id| sim.world.earned_achievements.contains(achievement_id))
            .filter_map(|achievement_id| sim.achievements.get(achievement_id))
            .map(|info| info.points)
            .sum()
    };
    state.push(Val::Num(total_points as f64));
    Ok(1)
}

fn push_achievement_info_for_id(state: &mut LuaState, achievement_id: i32) -> LuaResult<u32> {
    let row = {
        let sim = borrow_state(state)?;
        let info = sim
            .achievements
            .get(&achievement_id)
            .cloned()
            .unwrap_or(AchievementInfo {
                achievement_id,
                name: String::new(),
                points: 0,
                description: String::new(),
                flags: 0,
                icon: 0,
                reward_text: String::new(),
                is_guild: false,
                is_statistic: false,
                reward_item_id: None,
            });
        let completed = sim.world.earned_achievements.contains(&achievement_id);
        (info, completed)
    };
    push_achievement_multiret(state, &row.0, row.1);
    Ok(15)
}

fn push_category_list(state: &mut LuaState, kind: CategoryListKind) {
    let table_ref = state.gc.alloc_table(Table::new());
    let table = Val::Table(table_ref);
    for (index, category) in categories_for(kind).iter().enumerate() {
        set_table_array(
            state,
            table,
            (index + 1) as i64,
            Val::Num(category.category_id as f64),
        );
    }
    state.push(table);
}

fn count_completed_achievements(state: &mut LuaState, achievement_ids: &[i32]) -> LuaResult<i32> {
    let sim = borrow_state(state)?;
    Ok(achievement_ids
        .iter()
        .filter(|achievement_id| {
            is_achievement_completed(&sim.world.earned_achievements, **achievement_id)
        })
        .count() as i32)
}

fn is_achievement_completed(earned_achievements: &HashSet<i32>, achievement_id: i32) -> bool {
    earned_achievements.contains(&achievement_id)
}

fn push_category_counts(state: &mut LuaState, total: i32, completed: i32) {
    state.push(Val::Num(total as f64));
    state.push(Val::Num(completed as f64));
    state.push(Val::Num((total - completed) as f64));
}

fn push_criterion_multiret(
    state: &mut LuaState,
    achievement_id: i32,
    criterion: &AchievementCriterion,
) -> LuaResult<()> {
    let completed = borrow_state(state)?
        .world
        .earned_achievements
        .contains(&achievement_id);
    let quantity = if completed {
        criterion.required_quantity
    } else {
        0
    };
    let name = create_string(state, criterion.name);
    let char_name = create_string(state, "");
    let quantity_string = create_string(
        state,
        &format!("{quantity}/{}", criterion.required_quantity),
    );
    state.push(name);
    state.push(Val::Num(0.0));
    state.push(Val::Bool(completed));
    state.push(Val::Num(quantity as f64));
    state.push(Val::Num(criterion.required_quantity as f64));
    state.push(char_name);
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    state.push(quantity_string);
    state.push(Val::Num(criterion.id as f64));
    state.push(Val::Bool(true));
    state.push(Val::Num(0.0));
    state.push(Val::Num(0.0));
    Ok(())
}

fn push_achievement_multiret(state: &mut LuaState, info: &AchievementInfo, completed: bool) {
    let name = create_string(state, &info.name);
    let description = create_string(state, &info.description);
    let reward_text = create_string(state, &info.reward_text);
    let earned_by = create_string(state, if completed { "player" } else { "" });

    state.push(Val::Num(info.achievement_id as f64));
    state.push(name);
    state.push(Val::Num(info.points as f64));
    state.push(Val::Bool(completed));
    push_completion_date(state, completed);
    state.push(description);
    state.push(Val::Num(info.flags as f64));
    state.push(Val::Num(info.icon as f64));
    state.push(reward_text);
    state.push(Val::Bool(info.is_guild));
    state.push(Val::Bool(completed));
    state.push(earned_by);
    state.push(Val::Bool(info.is_statistic));
}

fn push_completion_date(state: &mut LuaState, completed: bool) {
    let (month, day, year) = if completed {
        (COMPLETION_MONTH, COMPLETION_DAY, COMPLETION_YEAR)
    } else {
        (0.0, 0.0, 0.0)
    };
    state.push(Val::Num(month));
    state.push(Val::Num(day));
    state.push(Val::Num(year));
}
