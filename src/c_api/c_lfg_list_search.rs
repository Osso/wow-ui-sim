//! C_LFGList search lifecycle and per-result metadata.
use crate::lua_api::globals::lfg_list::{defer_lfg_event_with_id, fire_event_with_args};
use crate::lua_api::methods::{
    borrow_state, borrow_state_mut, create_string, create_table, table_set,
};
use crate::lua_api::timer_layout;
use crate::lua_bridge::FromStack;
use rilua::vm::state::LuaState;
use rilua::{LuaResult, Val};

pub(crate) fn search(state: &mut LuaState) -> LuaResult<u32> {
    let pending = std::mem::take(&mut borrow_state_mut(state)?.lfg_search_timer_ids);
    let pending = pending
        .into_iter()
        .filter(|id| !matches!(timer_layout::get_timer_callback(state, *id), Val::Nil))
        .collect();
    borrow_state_mut(state)?.lfg_search_timer_ids = pending;
    let ids = {
        let sim = borrow_state(state)?;
        sim.world
            .premade_listings
            .iter()
            .filter(|listing| {
                sim.lfg_search_text_filter
                    .is_none_or(|filter| filter.matches(listing))
            })
            .map(|listing| listing.search_result_id)
            .collect()
    };
    borrow_state_mut(state)?.lfg_search_result_ids = ids;
    let id = defer_lfg_event_with_id(
        state,
        dispatch_search_results,
        "C_LFGList.SearchResultsReady",
    )?;
    borrow_state_mut(state)?.lfg_search_timer_ids.push(id);
    Ok(0)
}

fn dispatch_search_results(state: &mut LuaState) -> LuaResult<u32> {
    fire_event_with_args(state, "LFG_LIST_SEARCH_RESULTS_RECEIVED", Vec::new())?;
    Ok(0)
}

pub(crate) fn clear_search_results(state: &mut LuaState) -> LuaResult<u32> {
    let ids = {
        let mut sim = borrow_state_mut(state)?;
        sim.lfg_search_result_ids.clear();
        let ids = std::mem::take(&mut sim.lfg_search_timer_ids);
        sim.rilua_timers.retain(|timer| !ids.contains(&timer.id));
        ids
    };
    for id in ids {
        timer_layout::remove_timer_callback(state, id);
    }
    Ok(0)
}

pub(crate) fn notify_listing_changed(state: &mut LuaState, result_id: u32) -> LuaResult<()> {
    if borrow_state(state)?
        .lfg_search_result_ids
        .contains(&result_id)
    {
        fire_event_with_args(
            state,
            "LFG_LIST_SEARCH_RESULT_UPDATED",
            vec![Val::Num(result_id as f64)],
        )?;
    }
    Ok(())
}

#[derive(Debug, Clone, Default)]
pub struct SearchMetadata {
    pub has_self: bool,
    pub quest_id: Option<u32>,
    pub overall_dungeon_score: Option<f64>,
    pub best_dungeon_score: Option<DungeonScore>,
    pub dungeon_scores: Vec<DungeonScore>,
    pub pvp_ratings: Vec<PvpRating>,
}

#[derive(Debug, Clone, Copy)]
pub enum SearchTextFilter {
    Activity(u32),
    Quest(u32),
}

pub(crate) fn clear_search_text_fields(state: &mut LuaState) -> LuaResult<u32> {
    borrow_state_mut(state)?.lfg_search_text_filter = None;
    Ok(0)
}

pub(crate) fn set_search_to_activity(state: &mut LuaState) -> LuaResult<u32> {
    let id = u32::from_stack(state, 1)?;
    borrow_state_mut(state)?.lfg_search_text_filter = Some(SearchTextFilter::Activity(id));
    Ok(0)
}

pub(crate) fn set_search_to_quest(state: &mut LuaState) -> LuaResult<u32> {
    let id = u32::from_stack(state, 1)?;
    borrow_state_mut(state)?.lfg_search_text_filter = Some(SearchTextFilter::Quest(id));
    Ok(0)
}

impl SearchTextFilter {
    pub fn matches(self, listing: &crate::lua_api::state_types::PremadeListing) -> bool {
        match self {
            Self::Activity(id) => listing.activity_id == id,
            Self::Quest(id) => listing.metadata.quest_id == Some(id),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DungeonScore {
    pub map_score: f64,
    pub map_name: String,
    pub best_run_level: i32,
    pub finished_success: bool,
    pub best_run_duration_ms: f64,
    pub best_level_increment: i32,
}

#[derive(Debug, Clone, Default)]
pub struct PvpRating {
    pub bracket: i32,
    pub rating: i32,
    pub activity_name: String,
    pub tier: i32,
}

pub(crate) fn publish(state: &mut LuaState, info: Val, metadata: &SearchMetadata) {
    table_set(state, info, "hasSelf", Val::Bool(metadata.has_self));
    if let Some(score) = metadata.overall_dungeon_score {
        table_set(state, info, "leaderOverallDungeonScore", Val::Num(score));
    }
    if let Some(score) = &metadata.best_dungeon_score {
        let best = create_table(state);
        table_set(state, info, "leaderBestDungeonScoreInfo", best);
        set_dungeon_score(state, best, score);
    }
    publish_dungeon_scores(state, info, &metadata.dungeon_scores);
    publish_pvp_ratings(state, info, &metadata.pvp_ratings);
}

fn publish_dungeon_scores(state: &mut LuaState, info: Val, scores: &[DungeonScore]) {
    let dungeons = create_table(state);
    table_set(state, info, "leaderDungeonScoreInfo", dungeons);
    for (index, score) in scores.iter().enumerate() {
        let row = create_table(state);
        super::helpers::set_table_array(state, dungeons, index as i64 + 1, row);
        set_dungeon_score(state, row, score);
    }
}

fn set_dungeon_score(state: &mut LuaState, row: Val, score: &DungeonScore) {
    for (key, value) in [
        ("mapScore", score.map_score),
        ("bestRunLevel", f64::from(score.best_run_level)),
        ("bestRunDurationMs", score.best_run_duration_ms),
        ("bestLevelIncrement", f64::from(score.best_level_increment)),
    ] {
        table_set(state, row, key, Val::Num(value));
    }
    let name = create_string(state, &score.map_name);
    table_set(state, row, "mapName", name);
    table_set(
        state,
        row,
        "finishedSuccess",
        Val::Bool(score.finished_success),
    );
}

fn publish_pvp_ratings(state: &mut LuaState, info: Val, ratings_info: &[PvpRating]) {
    let ratings = create_table(state);
    table_set(state, info, "leaderPvpRatingInfo", ratings);
    for (index, rating) in ratings_info.iter().enumerate() {
        let row = create_table(state);
        super::helpers::set_table_array(state, ratings, index as i64 + 1, row);
        for (key, value) in [
            ("bracket", rating.bracket),
            ("rating", rating.rating),
            ("tier", rating.tier),
        ] {
            table_set(state, row, key, Val::Num(f64::from(value)));
        }
        let name = create_string(state, &rating.activity_name);
        table_set(state, row, "activityName", name);
    }
}
