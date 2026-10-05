//! Tests for world quest (C_TaskQuest) data seeding and related APIs.

use wow_ui_sim::lua_api::WowLuaEnv;

fn env() -> WowLuaEnv {
    WowLuaEnv::new().expect("Failed to create Lua environment")
}

#[test]
fn test_task_quest_get_quests_on_map_returns_seeded_data() {
    let env = env();
    let count: i32 = env
        .eval("return #C_TaskQuest.GetQuestsOnMap(2025)")
        .unwrap();
    assert_eq!(count, 2, "Thaldraszus (2025) should have 2 world quests");

    let count: i32 = env
        .eval("return #C_TaskQuest.GetQuestsOnMap(2024)")
        .unwrap();
    assert_eq!(count, 2, "Azure Span (2024) should have 2 world quests");

    let count: i32 = env
        .eval("return #C_TaskQuest.GetQuestsOnMap(9999)")
        .unwrap();
    assert_eq!(count, 0, "Unknown map should have 0 world quests");
}

#[test]
fn test_task_quest_info_fields() {
    let env = env();
    let quest_id: i32 = env
        .eval("return C_TaskQuest.GetQuestsOnMap(2025)[1].questID")
        .unwrap();
    assert_eq!(quest_id, 90001);

    let x: f64 = env
        .eval("return C_TaskQuest.GetQuestsOnMap(2025)[1].x")
        .unwrap();
    assert!((x - 0.52).abs() < 0.001, "x should be ~0.52, got {x}");
}

#[test]
fn test_task_quest_get_quest_info_by_id() {
    let env = env();
    let title: String = env
        .eval("return C_TaskQuest.GetQuestInfoByQuestID(90001)")
        .unwrap();
    assert_eq!(title, "Glittering Geodes");
}

#[test]
fn test_task_quest_get_quest_location() {
    let env = env();
    let (x, y): (f64, f64) = env
        .eval("return C_TaskQuest.GetQuestLocation(90001, 2025)")
        .unwrap();
    assert!((x - 0.52).abs() < 0.001);
    assert!((y - 0.63).abs() < 0.001);
}

#[test]
fn test_task_quest_is_active() {
    let env = env();
    let active: bool = env.eval("return C_TaskQuest.IsActive(90001)").unwrap();
    assert!(active, "Seeded world quest should be active");

    let active: bool = env.eval("return C_TaskQuest.IsActive(12345)").unwrap();
    assert!(!active, "Non-seeded quest should not be active");
}

#[test]
fn test_quest_log_is_world_quest() {
    let env = env();
    let is_wq: bool = env.eval("return C_QuestLog.IsWorldQuest(90001)").unwrap();
    assert!(is_wq, "Seeded world quest should return true");

    let is_wq: bool = env.eval("return C_QuestLog.IsWorldQuest(80000)").unwrap();
    assert!(!is_wq, "Regular quest should return false");
}

#[test]
fn test_quest_log_get_quest_tag_info_world_quest() {
    let env = env();
    let world_quest_type: i32 = env
        .eval("return C_QuestLog.GetQuestTagInfo(90001).worldQuestType")
        .unwrap();
    assert_eq!(world_quest_type, 2, "World quest type should be Normal (2)");

    let display_exp: bool = env
        .eval("return C_QuestLog.GetQuestTagInfo(90001).displayExpiration")
        .unwrap();
    assert!(display_exp, "World quest should display expiration");
}

#[test]
fn test_quest_log_get_quest_tag_info_regular_quest() {
    let env = env();
    let is_nil: bool = env
        .eval("return C_QuestLog.GetQuestTagInfo(80000).worldQuestType == nil")
        .unwrap();
    assert!(is_nil, "Regular quest should have nil worldQuestType");
}

#[test]
fn test_have_quest_data_returns_true_for_seeded() {
    let env = env();
    let have: bool = env.eval("return HaveQuestData(90001)").unwrap();
    assert!(have, "HaveQuestData should return true for world quests");

    let have: bool = env.eval("return HaveQuestData(80000)").unwrap();
    assert!(
        have,
        "HaveQuestData should return true for quest log quests"
    );

    let have: bool = env.eval("return HaveQuestRewardData(90001)").unwrap();
    assert!(
        have,
        "HaveQuestRewardData should return true for world quests"
    );
}

#[cfg(not(feature = "retail-12-0-0"))]
#[test]
fn test_get_quests_for_player_by_map_id_matches_get_quests_on_map() {
    let env = env();
    let on_map: i32 = env
        .eval("return #C_TaskQuest.GetQuestsOnMap(2023)")
        .unwrap();
    let for_player: i32 = env
        .eval("return #C_TaskQuest.GetQuestsForPlayerByMapID(2023)")
        .unwrap();
    assert_eq!(on_map, for_player, "Both APIs should return same count");
    assert_eq!(on_map, 2, "Ohn'ahran Plains should have 2 quests");
}
