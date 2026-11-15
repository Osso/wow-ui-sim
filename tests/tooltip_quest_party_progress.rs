//! Retail tooltip party-progress tests use the selected ObjectiveTracker and
//! the original TooltipDataHandler accessor. All progress values are supplied.
#![cfg(all(feature = "optional-tests", feature = "client-retail"))]

use wow_ui_sim::c_api::quest_party_progress::{
    QuestPartyProgressHeading, QuestPartyProgressParticipant, QuestPartyProgressRow,
    QuestPartyProgressSnapshot, QuestPartyProgressText,
};
use wow_ui_sim::c_api::quest_special_item::{
    QuestSpecialItemCacheRecord, QuestSpecialItemCacheUpdate, QuestSpecialItemCallback,
    QuestSpecialItemObject, QuestSpecialItemRequestToken,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::BagItem;

const RED: (f64, f64, f64) = (1.0, 0.0, 0.0);
const GREEN: (f64, f64, f64) = (0.0, 1.0, 0.0);
const QUEST_A: i32 = 990_701;
const QUEST_B: i32 = 990_702;
const ITEM_A: u32 = 6948;
const ITEM_B: u32 = 7005;

const QUEST_ITEM_HOVER_FIXTURE: &str = r#"
local function requireSource(fn, suffix)
    local source = debug.getinfo(fn, 'S').source:gsub('\\', '/')
    assert(source:find(suffix, 1, true), source)
end
requireSource(QuestObjectiveItemButtonMixin.OnEnter, 'Blizzard_ObjectiveTrackerShared.lua')
requireSource(GameTooltip.SetQuestLogSpecialItem, 'TooltipDataHandler.lua')
QuestLogUpdateCount = 0
local observer = CreateFrame('Frame')
observer:RegisterEvent('QUEST_LOG_UPDATE')
observer:SetScript('OnEvent', function(_, event, ...)
    assert(event == 'QUEST_LOG_UPDATE')
    assert(select('#', ...) == 0)
    QuestLogUpdateCount = QuestLogUpdateCount + 1
end)
function QuestSpecialItemHover(index)
    local button = CreateFrame('Button')
    button:SetAttribute('questLogIndex', index)
    GameTooltip:Hide()
    GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
    QuestObjectiveItemButtonMixin:OnEnter(button)
    return GameTooltip:IsShown(), GameTooltip:GetPrimaryTooltipData()
end
"#;

fn prepare_quest_item_hover(env: &WowLuaEnv) {
    env.exec(QUEST_ITEM_HOVER_FIXTURE)
        .expect("original ObjectiveTracker hover consumer should load");
    env.state().borrow_mut().events.drain();
}

fn supply_quest_special_item(
    env: &WowLuaEnv,
    log_index: i32,
    quest_id: i32,
    item_id: u32,
    slot: i32,
) {
    let mut sim = env.state().borrow_mut();
    let inputs = &mut sim.quest_log_entries.quest_special_item_inputs;
    inputs.quest_ids_by_log_index.insert(log_index, quest_id);
    inputs.cached_quest_records.insert(
        quest_id,
        QuestSpecialItemCacheRecord {
            special_item_id: Some(item_id),
            ..Default::default()
        },
    );
    inputs.selected_item_objects.insert(
        quest_id,
        QuestSpecialItemObject {
            item_id,
            bag: 0,
            slot,
        },
    );
    sim.bag_items.insert((0, slot), bag_item(item_id));
}

fn mark_quest_record_not_ready(env: &WowLuaEnv, quest_id: i32) {
    env.state()
        .borrow_mut()
        .quest_log_entries
        .quest_special_item_inputs
        .not_ready_cache_records
        .insert(quest_id);
}

fn set_request_token(env: &WowLuaEnv, quest_id: i32, words: [u64; 2], marked: bool) {
    env.state()
        .borrow_mut()
        .quest_log_entries
        .quest_special_item_inputs
        .request_tokens_by_quest_id
        .insert(
            quest_id,
            QuestSpecialItemRequestToken {
                words,
                high_part_marker: marked,
            },
        );
}

fn queue_quest_record_arrival(env: &WowLuaEnv, quest_id: i32, item_id: u32) {
    let record = QuestSpecialItemCacheRecord {
        special_item_id: Some(item_id),
        ..Default::default()
    };
    env.state()
        .borrow_mut()
        .quest_log_entries
        .quest_special_item_inputs
        .host_cache_updates
        .push(QuestSpecialItemCacheUpdate::RecordArrived {
            quest_id,
            record,
            success: true,
        });
}

fn queue_quest_record_destruction(env: &WowLuaEnv, quest_id: i32) {
    env.state()
        .borrow_mut()
        .quest_log_entries
        .quest_special_item_inputs
        .host_cache_updates
        .push(QuestSpecialItemCacheUpdate::RecordDestroyed { quest_id });
}

fn assert_quest_item_hover_hidden(env: &WowLuaEnv, log_index: i32) {
    env.exec(&format!(
        "local shown, data = QuestSpecialItemHover({log_index}); assert(not shown and data == nil, 'quest item tooltip must be hidden')"
    ))
    .expect("original consumer should hide a quest item without ready data");
}

fn assert_quest_item_hover_shows(env: &WowLuaEnv, log_index: i32, item_id: u32) {
    env.exec(&format!(
        "local shown, data = QuestSpecialItemHover({log_index}); assert(shown and data and data.id == {item_id}, 'hover must show its supplied item')"
    ))
    .expect("original consumer should show the selected supplied item");
}

fn assert_pending_callbacks(env: &WowLuaEnv, quest_id: i32, expected: usize) {
    let count = env
        .state()
        .borrow()
        .quest_log_entries
        .quest_special_item_inputs
        .pending_callbacks
        .get(&quest_id)
        .map(Vec::len)
        .unwrap_or_default();
    assert_eq!(count, expected, "quest {quest_id} pending callback count");
}

fn fire_queued_quest_updates(env: &WowLuaEnv) -> usize {
    let events = env.state().borrow_mut().events.drain();
    for event in &events {
        assert_eq!(event.name, "QUEST_LOG_UPDATE");
        assert!(event.args.is_empty());
        env.fire_event(&event.name)
            .expect("queued quest update should dispatch");
    }
    events.len()
}

fn selected_objective_tracker() -> WowLuaEnv {
    let cache = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path()
        .expect("Retail Blizzard cache is required");
    let (env, loaded) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &cache,
        &["Blizzard_ObjectiveTracker"],
        &[],
    );
    for addon in ["Blizzard_SharedXMLGame", "Blizzard_ObjectiveTracker"] {
        assert!(
            loaded.iter().any(|name| name == addon),
            "selected dependency closure must load {addon}: {loaded:?}"
        );
    }
    env.exec("A_Admin.SetPartySize(1)")
        .expect("supply group membership for the original header consumer");
    env
}

fn snapshot(title: &str, player: &str, guid: &str, objective: &str) -> QuestPartyProgressSnapshot {
    QuestPartyProgressSnapshot {
        title: Some(QuestPartyProgressText {
            text: title.to_string(),
            color: RED,
        }),
        participants: vec![participant(player, guid, objective, GREEN, false)],
    }
}

fn participant(
    name: &str,
    guid: &str,
    objective: &str,
    color: (f64, f64, f64),
    exclude_when_ignoring_active_player: bool,
) -> QuestPartyProgressParticipant {
    QuestPartyProgressParticipant {
        heading: Some(QuestPartyProgressHeading {
            name: name.to_string(),
            guid: guid.to_string(),
            color,
        }),
        exclude_when_ignoring_active_player,
        rows: vec![QuestPartyProgressRow {
            text: objective.to_string(),
            color,
            completed: false,
            num_required: Some(3),
            num_fulfilled: Some(1),
            progress_bar_percent: Some(33.0),
        }],
    }
}

fn supply_snapshot(env: &WowLuaEnv, quest_id: i32, value: QuestPartyProgressSnapshot) {
    env.state()
        .borrow_mut()
        .quest_log_entries
        .party_progress
        .insert(quest_id, value);
}

#[test]
fn selected_objective_tracker_publishes_party_progress_for_the_requested_quest() {
    let env = selected_objective_tracker();
    supply_snapshot(
        &env,
        QUEST_A,
        snapshot(
            "First supplied title",
            "First member",
            "Player-1-0001",
            "First objective",
        ),
    );
    supply_snapshot(
        &env,
        QUEST_B,
        snapshot(
            "Second supplied title",
            "Second member",
            "Player-1-0002",
            "Second objective",
        ),
    );
    env.exec(&format!(
        r#"
        local function requireSource(fn, suffix)
            local source = debug.getinfo(fn, 'S').source:gsub('\\', '/')
            assert(source:find(suffix, 1, true), source)
        end
        requireSource(QuestObjectiveTrackerMixin.OnBlockHeaderEnter,
            'Blizzard_QuestObjectiveTracker.lua')
        requireSource(GameTooltip.SetQuestPartyProgress, 'TooltipDataHandler.lua')
        local function hover(questID)
            local block = CreateFrame('Frame')
            block.id = questID
            QuestObjectiveTrackerMixin:OnBlockHeaderEnter(block)
            return GameTooltip:GetPrimaryTooltipData()
        end
        local first = hover({QUEST_A})
        assert(first, 'RED: supplied quest progress must reach the original hover consumer')
        assert(first.type == 24 and first.id == {QUEST_A})
        assert(first.lines[1].type == 17 and first.lines[1].id == {QUEST_A})
        assert(first.lines[2].type == 18 and first.lines[2].guid == 'Player-1-0001')
        assert(first.lines[3].type == 8 and first.lines[3].completed == false)
        assert(first.lines[3].numRequired == 3 and first.lines[3].numFulfilled == 1)
        assert(first.lines[3].progressBarPercent == 33)

        local second = hover({QUEST_B})
        assert(second and second.id == {QUEST_B})
        assert(second.lines[1].id == {QUEST_B})
        assert(second.lines[2].guid == 'Player-1-0002')
        assert(second.lines[3].completed == false)
        "#
    ))
    .expect("selected ObjectiveTracker hover must publish the supplied quest snapshot");
}

#[test]
fn party_progress_omit_title_selects_the_title_row() {
    let env = selected_objective_tracker();
    supply_snapshot(
        &env,
        QUEST_A,
        snapshot(
            "Optional supplied title",
            "Member",
            "Player-1-0011",
            "Objective",
        ),
    );
    env.exec(&format!(
        r#"
        local source = debug.getinfo(GameTooltip.SetQuestPartyProgress, 'S').source:gsub('\\', '/')
        assert(source:find('TooltipDataHandler.lua', 1, true), source)
        local function query(omitTitle)
            GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
            GameTooltip:SetQuestPartyProgress({QUEST_A}, omitTitle, false)
            return GameTooltip:GetPrimaryTooltipData()
        end
        local included = query(false)
        assert(included, 'RED: omitTitle=false must publish supplied data')
        assert(included.lines[1].type == 17 and included.lines[1].id == {QUEST_A})
        local omitted = query(true)
        assert(omitted and omitted.id == {QUEST_A})
        assert(#omitted.lines == #included.lines - 1)
        assert(omitted.lines[1].type == 18 and omitted.lines[1].guid == 'Player-1-0011')
        "#
    ))
    .expect("the original tooltip accessor must honor the verified omitTitle selector");
}

#[test]
fn party_progress_ignore_active_player_selects_supplied_participants() {
    let env = selected_objective_tracker();
    supply_snapshot(
        &env,
        QUEST_A,
        QuestPartyProgressSnapshot {
            title: Some(QuestPartyProgressText {
                text: "Party title".to_string(),
                color: RED,
            }),
            participants: vec![
                participant(
                    "Filtered participant",
                    "Player-1-0021",
                    "Filtered objective",
                    GREEN,
                    true,
                ),
                participant(
                    "Retained participant",
                    "Player-1-0022",
                    "Retained objective",
                    GREEN,
                    false,
                ),
            ],
        },
    );
    env.exec(&format!(
        r#"
        local source = debug.getinfo(GameTooltip.SetQuestPartyProgress, 'S').source:gsub('\\', '/')
        assert(source:find('TooltipDataHandler.lua', 1, true), source)
        local function query(ignoreActivePlayer)
            GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
            GameTooltip:SetQuestPartyProgress({QUEST_A}, false, ignoreActivePlayer)
            return GameTooltip:GetPrimaryTooltipData()
        end
        local all = query(false)
        assert(all, 'RED: ignoreActivePlayer=false must publish supplied data')
        assert(all.lines[2].guid == 'Player-1-0021')
        assert(all.lines[3].completed == false)
        assert(all.lines[4].guid == 'Player-1-0022')
        assert(all.lines[5].completed == false)
        local filtered = query(true)
        assert(filtered and filtered.id == {QUEST_A})
        assert(filtered.lines[2].guid == 'Player-1-0022')
        assert(filtered.lines[3].completed == false)
        assert(#filtered.lines == #all.lines - 2)
        "#
    ))
    .expect("the original tooltip accessor must honor the verified participant filter");
}

#[test]
fn quest_tooltip_missing_sources_return_zero_lua_values() {
    let env = WowLuaEnv::new().expect("Retail Lua environment");
    env.exec(&format!(
        "assert(select('#', C_TooltipInfo.GetQuestLogSpecialItem(1)) == 0); assert(select('#', C_TooltipInfo.GetQuestPartyProgress({QUEST_A})) == 0)"
    ))
    .expect("R88n.1 verified zero-value missing-source results");
}

#[test]
fn selected_quest_item_hover_uses_host_supplied_owned_item() {
    let env = selected_objective_tracker();
    {
        let mut sim = env.state().borrow_mut();
        assert_ne!(
            sim.quest_log_entries.entries[0].quest_id, QUEST_A,
            "fixture keeps quest-cache identity separate from the quest-log row"
        );
        let inputs = &mut sim.quest_log_entries.quest_special_item_inputs;
        inputs.quest_ids_by_log_index.insert(1, QUEST_A);
        inputs.quest_ids_by_log_index.insert(2, QUEST_B);
        inputs.cached_quest_records.insert(
            QUEST_A,
            QuestSpecialItemCacheRecord {
                special_item_id: Some(ITEM_A),
                ..Default::default()
            },
        );
        inputs.cached_quest_records.insert(
            QUEST_B,
            QuestSpecialItemCacheRecord {
                has_candidate_items: true,
                candidate_item_ids: [Some(ITEM_B), None, None, None],
                ..Default::default()
            },
        );
        inputs.selected_item_objects.insert(
            QUEST_A,
            QuestSpecialItemObject {
                item_id: ITEM_A,
                bag: 0,
                slot: 2,
            },
        );
        inputs.selected_item_objects.insert(
            QUEST_B,
            QuestSpecialItemObject {
                item_id: ITEM_B,
                bag: 0,
                slot: 3,
            },
        );
        sim.bag_items.insert((0, 2), bag_item(ITEM_A));
        sim.bag_items.insert((0, 3), bag_item(ITEM_B));
    }
    env.exec(
        r#"
        local function requireSource(fn, suffix)
            local source = debug.getinfo(fn, 'S').source:gsub('\\', '/')
            assert(source:find(suffix, 1, true), source)
        end
        requireSource(QuestObjectiveItemButtonMixin.OnEnter,
            'Blizzard_ObjectiveTrackerShared.lua')
        requireSource(GameTooltip.SetQuestLogSpecialItem, 'TooltipDataHandler.lua')
        local function hover(index)
            local button = CreateFrame('Button')
            button:SetAttribute('questLogIndex', index)
            GameTooltip:SetOwner(UIParent, 'ANCHOR_NONE')
            QuestObjectiveItemButtonMixin.OnEnter(button)
            return GameTooltip:GetPrimaryTooltipData()
        end
        local first = hover(1)
        assert(first, 'RED: supplied quest item must reach the original hover consumer')
        assert(first.id == 6948 and type(first.guid) == 'string')
        local second = hover(2)
        assert(second and second.id == 7005)
        assert(type(second.guid) == 'string' and second.guid ~= first.guid)
        "#,
    )
    .expect("the selected original item hover must publish each supplied owned item");

    env.state().borrow_mut().bag_items.remove(&(0, 2));
    env.exec("assert(select('#', C_TooltipInfo.GetQuestLogSpecialItem(1)) == 0)")
        .expect("a missing selected item object returns exactly zero values");

    {
        let mut sim = env.state().borrow_mut();
        sim.bag_items.insert((0, 2), bag_item(ITEM_A));
        sim.quest_log_entries
            .quest_special_item_inputs
            .cached_quest_records
            .get_mut(&QUEST_A)
            .unwrap()
            .special_item_id = Some(ITEM_B);
        sim.quest_log_entries
            .quest_special_item_inputs
            .cached_quest_records
            .get_mut(&QUEST_B)
            .unwrap()
            .has_candidate_items = false;
    }
    env.exec(
        "assert(select('#', C_TooltipInfo.GetQuestLogSpecialItem(1)) == 0); assert(select('#', C_TooltipInfo.GetQuestLogSpecialItem(2)) == 0)"
    )
    .expect("selected objects must match the primary or enabled candidate cache IDs");
}

fn assert_pending_token(
    env: &WowLuaEnv,
    quest_id: i32,
    entry_index: usize,
    words: [u64; 2],
    marked: bool,
) {
    let sim = env.state().borrow();
    let entry = &sim
        .quest_log_entries
        .quest_special_item_inputs
        .pending_callbacks[&quest_id][entry_index];
    assert_eq!(entry.callback, QuestSpecialItemCallback::QuestRecordData);
    assert_eq!(entry.token.words, words);
    assert_eq!(entry.token.high_part_marker, marked);
    assert!(entry.extra_context.is_empty());
}

fn exercise_quest_item_callback_dedup(env: &WowLuaEnv) {
    set_request_token(env, QUEST_A, [11, 12], false);
    assert_quest_item_hover_hidden(env, 1);
    assert_pending_token(env, QUEST_A, 0, [11, 12], false);
    set_request_token(env, QUEST_A, [31, 32], false);
    assert_quest_item_hover_hidden(env, 1);
    assert_pending_callbacks(env, QUEST_A, 1);
    assert_pending_token(env, QUEST_A, 0, [11, 12], false);
    set_request_token(env, QUEST_B, [21, 22], true);
    assert_quest_item_hover_hidden(env, 2);
    set_request_token(env, QUEST_B, [21, 23], true);
    assert_quest_item_hover_hidden(env, 2);
    assert_pending_callbacks(env, QUEST_B, 2);
    assert_pending_token(env, QUEST_B, 0, [21, 22], true);
    assert_pending_token(env, QUEST_B, 1, [21, 23], true);
    assert_quest_item_hover_hidden(env, 2);
    assert_pending_callbacks(env, QUEST_B, 2);
}

fn arrive_quest_item(env: &WowLuaEnv, index: i32, quest_id: i32, item_id: u32, count: i32) {
    queue_quest_record_arrival(env, quest_id, item_id);
    env.fire_on_update(0.0)
        .expect("host quest-record arrival should run");
    assert_eq!(fire_queued_quest_updates(env), 1);
    assert_eq!(
        env.eval::<i32>("return QuestLogUpdateCount").unwrap(),
        count
    );
    assert_pending_callbacks(env, quest_id, 0);
    assert_quest_item_hover_shows(env, index, item_id);
}

#[test]
fn selected_objective_tracker_retries_quest_item_after_host_cache_arrival() {
    let env = selected_objective_tracker();
    prepare_quest_item_hover(&env);
    supply_quest_special_item(&env, 1, QUEST_A, ITEM_A, 2);
    supply_quest_special_item(&env, 2, QUEST_B, ITEM_B, 3);
    mark_quest_record_not_ready(&env, QUEST_A);
    mark_quest_record_not_ready(&env, QUEST_B);
    exercise_quest_item_callback_dedup(&env);
    assert_quest_item_hover_hidden(&env, 3);
    arrive_quest_item(&env, 1, QUEST_A, ITEM_A, 1);
    arrive_quest_item(&env, 2, QUEST_B, ITEM_B, 2);
    assert_quest_item_hover_hidden(&env, 3);
}

#[test]
fn selected_objective_tracker_cleans_callbacks_when_host_destroys_unready_records() {
    let env = selected_objective_tracker();
    prepare_quest_item_hover(&env);
    supply_quest_special_item(&env, 1, QUEST_A, ITEM_A, 2);
    supply_quest_special_item(&env, 2, QUEST_B, ITEM_B, 3);
    mark_quest_record_not_ready(&env, QUEST_A);
    mark_quest_record_not_ready(&env, QUEST_B);
    assert_quest_item_hover_hidden(&env, 1);
    assert_quest_item_hover_hidden(&env, 2);
    assert_quest_item_hover_hidden(&env, 3);
    assert_pending_callbacks(&env, QUEST_A, 1);
    assert_pending_callbacks(&env, QUEST_B, 1);
    queue_quest_record_destruction(&env, QUEST_A);
    queue_quest_record_destruction(&env, QUEST_B);
    env.fire_on_update(0.0)
        .expect("host record destruction should run");
    assert_eq!(fire_queued_quest_updates(&env), 0);
    assert_pending_callbacks(&env, QUEST_A, 0);
    assert_pending_callbacks(&env, QUEST_B, 0);
    assert_quest_item_hover_hidden(&env, 1);
    assert_quest_item_hover_hidden(&env, 2);
    assert_quest_item_hover_hidden(&env, 3);
    let inputs = &env
        .state()
        .borrow()
        .quest_log_entries
        .quest_special_item_inputs;
    assert!(!inputs.cached_quest_records.contains_key(&QUEST_A));
    assert!(!inputs.cached_quest_records.contains_key(&QUEST_B));
}

fn bag_item(item_id: u32) -> BagItem {
    BagItem {
        owned_identity: None,
        item_id,
        stack_count: 1,
        hyperlink: None,
        durability: None,
        applied_transmog: None,
    }
}
