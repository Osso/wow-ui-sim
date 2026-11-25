//! Integration tests for the top-level quest-count globals routed through
//! `src/lua_api/globals/quest_surface.rs`.
//!
//! Covers the three globals migrated off `GLOBAL_ZERO_STUBS`:
//! - `GetNumQuestLogEntries`
//! - `QuestMapUpdateAllQuests`
//! - `GetQuestLogTimeLeft`

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn get_num_quest_log_entries_matches_c_quest_log_method() {
    let env = env();
    let (top_entries, top_quests, c_entries, c_quests): (i32, i32, i32, i32) = env
        .eval(
            r#"
            local ta, tb = GetNumQuestLogEntries()
            local ca, cb = C_QuestLog.GetNumQuestLogEntries()
            return ta, tb, ca, cb
            "#,
        )
        .unwrap();
    assert_eq!(top_entries + 1, c_entries);
    assert_eq!(top_quests, c_quests);
    assert!(
        top_entries > 0,
        "seeded quest log should expose at least one entry"
    );
    assert!(
        top_quests > 0,
        "seeded quest log should expose at least one non-header quest"
    );
    assert!(
        c_entries >= c_quests,
        "total entries (including headers) cannot be below quest count"
    );
}

#[test]
fn quest_map_update_all_quests_returns_seeded_quest_count() {
    let env = env();
    let (num_pois, num_quests): (i32, i32) = env
        .eval(
            r#"
            local pois = QuestMapUpdateAllQuests()
            local _, quests = C_QuestLog.GetNumQuestLogEntries()
            return pois, quests
            "#,
        )
        .unwrap();
    assert_eq!(
        num_pois, num_quests,
        "sim treats every quest-log entry as having a POI"
    );
}

#[test]
fn quest_has_poi_info_matches_seeded_quest_info() {
    let env = env();
    let (has_poi, info_has_poi, missing_has_poi): (bool, bool, bool) = env
        .eval(
            r#"
            local questID = C_QuestLog.GetQuestIDForLogIndex(2)
            local info = C_QuestLog.GetInfo(2)
            return QuestHasPOIInfo(questID), info.hasLocalPOI, QuestHasPOIInfo(999999)
            "#,
        )
        .unwrap();
    assert_eq!(has_poi, info_has_poi);
    assert!(!missing_has_poi);
}

#[test]
fn quest_watch_poi_probe_handles_all_watched_quests() {
    let env = env();
    let result: String = env
        .eval(
            r#"
            for index = 1, C_QuestLog.GetNumQuestWatches() do
                local questID = C_QuestLog.GetQuestIDForQuestWatchIndex(index)
                local hasPoi = QuestHasPOIInfo(questID)
                if type(hasPoi) ~= "boolean" then
                    return "poi_type_" .. tostring(index) .. "=" .. type(hasPoi)
                end
            end
            return "ok"
            "#,
        )
        .unwrap();
    assert_eq!(result, "ok");
}

#[cfg(feature = "client-mists")]
#[test]
fn get_quest_pois_fills_legacy_output_table() {
    let env = env();
    let (count, first_id, first_map_id): (i32, i32, i32) = env
        .eval(
            r#"
            local pois = {}
            GetQuestPOIs(pois)
            return #pois, pois[1].questID, pois[1].mapID
            "#,
        )
        .unwrap();
    assert!(count > 0, "legacy quest POI table should be populated");
    assert_eq!(first_id, 80000);
    assert_eq!(first_map_id, 2248);
}

#[test]
fn get_quest_log_time_left_nil_when_no_selection() {
    let env = env();
    let v: Option<f64> = env.eval("return GetQuestLogTimeLeft()").unwrap();
    assert_eq!(
        v, None,
        "unselected quest log should not expose a time limit"
    );
}

#[test]
fn get_quest_log_time_left_nil_when_selected_quest_is_not_world_quest() {
    let env = env();
    // Pick a known non-world quest log index (2 = first quest after header).
    env.exec(
        r#"
        local qid = C_QuestLog.GetQuestIDForLogIndex(2)
        if qid then C_QuestLog.SetSelectedQuest(qid) end
        "#,
    )
    .unwrap();
    let v: Option<f64> = env.eval("return GetQuestLogTimeLeft()").unwrap();
    assert_eq!(
        v, None,
        "a normal selected quest should not report a time limit"
    );
}

#[cfg(feature = "client-retail")]
#[test]
fn get_quest_log_time_left_returns_one_nil_without_a_retail_timer_model() {
    let env = env();
    let (count, is_nil): (i32, bool) = env
        .eval("local n = select('#', GetQuestLogTimeLeft()); return n, GetQuestLogTimeLeft() == nil")
        .unwrap();
    assert_eq!(count, 1);
    assert!(is_nil);
}
