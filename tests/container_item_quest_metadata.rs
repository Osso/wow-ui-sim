//! Supplied-state contract only: not a native quest-item catalogue oracle.
use wow_ui_sim::c_api::container_quest::ContainerItemQuestMetadata;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::BagItem;

const QUEST_ITEM: u32 = 990001;
const ORDINARY_ITEM: u32 = 990002;
const QUEST_ID: i32 = 990101;

fn supply_item(env: &WowLuaEnv, bag: i32, slot: i32, item_id: u32) {
    env.state().borrow_mut().bag_items.insert(
        (bag, slot),
        BagItem {
            item_id,
            stack_count: 1,
            hyperlink: None,
            durability: None,
            applied_transmog: None,
        },
    );
}

fn fixture(quest_id: Option<i32>) -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create isolated quest-item environment");
    supply_item(&env, 0, 1, QUEST_ITEM);
    supply_item(&env, 0, 2, ORDINARY_ITEM);
    {
        let mut sim = env.state().borrow_mut();
        sim.bag_items.remove(&(0, 16));
        sim.container_item_quests.insert(
            QUEST_ITEM,
            ContainerItemQuestMetadata {
                is_quest_item: true,
                quest_id,
            },
        );
    }
    env
}

fn assert_table(env: &WowLuaEnv, bag: i32, slot: i32, expected: (bool, Option<i32>, bool)) {
    let actual: (bool, Option<i32>, bool) = env
        .eval(&format!(
            "local info = C_Container.GetContainerItemQuestInfo({bag}, {slot}); \
             assert(type(info) == 'table'); \
             assert(type(info.isQuestItem) == 'boolean'); \
             assert(type(info.isActive) == 'boolean'); \
             assert(info.questID == nil or type(info.questID) == 'number'); \
             return info.isQuestItem, info.questID, info.isActive"
        ))
        .expect("read documented quest-info fields");
    assert_eq!(actual, expected, "bag {bag}, slot {slot}");
}

fn accept(env: &WowLuaEnv) {
    env.exec(&format!(
        "A_Admin.OpenQuestNpc({QUEST_ID}, 'Supplied relic quest'); \
         SelectAvailableQuest(1); ConfirmAcceptQuest()"
    ))
    .expect("accept supplied quest through existing verbs");
    let sim = env.state().borrow();
    assert!(sim.quest_log.contains(&(QUEST_ID as u32)));
    assert!(
        sim.quest_log_entries
            .entries
            .iter()
            .any(|row| row.quest_id == QUEST_ID)
    );
}

fn abandon(env: &WowLuaEnv) {
    env.exec(&format!(
        "QuestMapLogTitleButton_OnClick({QUEST_ID}); SetAbandonQuest()"
    ))
    .expect("abandon supplied quest through existing verbs");
    // Wrath QuestLogFrame.lua:615 marks; StaticPopup.lua:1692 confirms.
    if wow_ui_sim::client_profile::ACTIVE == wow_ui_sim::client_profile::ClientProfile::Wrath {
        env.exec("AbandonQuest()")
            .expect("confirm the marked Wrath quest abandonment");
    }
    let sim = env.state().borrow();
    assert!(!sim.quest_log.contains(&(QUEST_ID as u32)));
    assert!(
        !sim.quest_log_entries
            .entries
            .iter()
            .any(|row| row.quest_id == QUEST_ID)
    );
}

#[test]
fn supplied_quest_item_table_tracks_accept_and_abandon() {
    let env = fixture(Some(QUEST_ID));
    assert_table(&env, 0, 1, (true, Some(QUEST_ID), false));
    accept(&env);
    assert_table(&env, 0, 1, (true, Some(QUEST_ID), true));
    abandon(&env);
    assert_table(&env, 0, 1, (true, Some(QUEST_ID), false));
}

#[test]
fn supplied_quest_item_without_quest_id_retains_flag() {
    let env = fixture(None);
    assert_table(&env, 0, 1, (true, None, false));
}

#[test]
fn active_quest_item_uses_rich_membership_not_completed_history() {
    let env = fixture(Some(80001));
    // Seeded 80001 is both in the rich active log and completion history.
    assert_table(&env, 0, 1, (true, Some(80001), true));
    {
        let mut sim = env.state().borrow_mut();
        sim.quest_log_entries
            .entries
            .retain(|row| row.quest_id != 80001);
        assert!(sim.quest_log_entries.completed_quest_ids.contains(&80001));
        sim.quest_log.push(80001);
    }
    assert_table(&env, 0, 1, (true, Some(80001), false));
}

#[test]
fn nonquest_empty_and_invalid_slots_have_false_table_shape() {
    let env = fixture(Some(QUEST_ID));
    // Even stale occupied entries outside supplied capacity are invalid slots.
    for (bag, slot) in [(0, 0), (0, -1), (0, 17), (999, 1)] {
        supply_item(&env, bag, slot, QUEST_ITEM);
    }
    for (bag, slot) in [(0, 2), (0, 16), (0, 0), (0, -1), (0, 17), (999, 1)] {
        assert_table(&env, bag, slot, (false, None, false));
    }
}

#[test]
fn quest_metadata_follows_item_identity_and_current_slot_contents() {
    let env = fixture(Some(QUEST_ID));
    assert_table(&env, 0, 1, (true, Some(QUEST_ID), false));
    let item = env.state().borrow_mut().bag_items.remove(&(0, 1)).unwrap();
    env.state().borrow_mut().bag_items.insert((-1, 1), item);
    assert_table(&env, 0, 1, (false, None, false));
    assert_table(&env, -1, 1, (true, Some(QUEST_ID), false));
    supply_item(&env, -1, 1, ORDINARY_ITEM);
    assert_table(&env, -1, 1, (false, None, false));
}

#[cfg(feature = "client-wrath")]
fn assert_legacy(env: &WowLuaEnv, slot: i32, expected: (bool, Option<i32>, bool)) {
    let actual: (i64, bool, Option<i32>, bool) = env
        .eval(&format!(
            "assert(type(GetContainerItemQuestInfo) == 'function'); \
             local function capture(...) return select('#', ...), ... end; \
             return capture(GetContainerItemQuestInfo(0, {slot}))"
        ))
        .expect("legacy quest-info call must succeed without masking errors");
    assert_eq!(
        actual,
        (3, expected.0, expected.1, expected.2),
        "legacy slot {slot}"
    );
}

#[cfg(feature = "client-wrath")]
#[test]
fn legacy_quest_item_returns_three_values_and_tracks_accept_abandon() {
    let env = fixture(Some(QUEST_ID));
    assert_legacy(&env, 1, (true, Some(QUEST_ID), false));
    accept(&env);
    assert_legacy(&env, 1, (true, Some(QUEST_ID), true));
    abandon(&env);
    assert_legacy(&env, 1, (true, Some(QUEST_ID), false));
}

#[cfg(feature = "client-wrath")]
#[test]
fn legacy_nonquest_empty_and_invalid_slots_return_false_nil_false() {
    let env = fixture(Some(QUEST_ID));
    for slot in [2, 16, 0, -1, 17] {
        assert_legacy(&env, slot, (false, None, false));
    }
}
