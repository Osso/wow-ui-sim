//! Additional per-result public metadata; absence of score records is an empty sequence.
use crate::lua_api::methods::{create_string, create_table, table_set};
use rilua::Val;
use rilua::vm::state::LuaState;

#[derive(Debug, Clone, Default)]
pub struct SearchMetadata {
    pub has_self: bool,
    pub dungeon_scores: Vec<DungeonScore>,
    pub pvp_ratings: Vec<PvpRating>,
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
    publish_dungeon_scores(state, info, &metadata.dungeon_scores);
    publish_pvp_ratings(state, info, &metadata.pvp_ratings);
}

fn publish_dungeon_scores(state: &mut LuaState, info: Val, scores: &[DungeonScore]) {
    let dungeons = create_table(state);
    table_set(state, info, "leaderDungeonScoreInfo", dungeons);
    for (index, score) in scores.iter().enumerate() {
        let row = create_table(state);
        super::helpers::set_table_array(state, dungeons, index as i64 + 1, row);
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
