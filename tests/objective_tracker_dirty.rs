#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
#[test]
fn objective_tracker_renders_two_watched_seed_quests() {
    crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape(
        &["Blizzard_ObjectiveTracker", "Blizzard_SplashFrame"],
        &[],
        |env, loaded| {
            assert!(loaded.iter().any(|name| name == "Blizzard_SplashFrame"));
            supply_watched_quests(env);
            advance_tracker(env);
            assert_seeded_quest_blocks(env);
        },
    );
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn supply_watched_quests(env: &WowLuaEnv) {
    env.exec(
        r#"
        C_QuestLog.AddQuestWatch(80000)
        C_QuestLog.AddQuestWatch(80001)
        A_Admin.FireEvent("QUEST_WATCH_LIST_CHANGED", 80000, true)
        A_Admin.FireEvent("QUEST_WATCH_LIST_CHANGED", 80001, true)
        local watched = {}
        for index = 1, C_QuestLog.GetNumQuestWatches() do
            watched[C_QuestLog.GetQuestIDForQuestWatchIndex(index)] = true
        end
        assert(watched[80000] and watched[80001], "both supplied quests must be watched")
        "#,
    )
    .expect("client APIs and admin events should supply both watched quests");
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn advance_tracker(env: &WowLuaEnv) {
    wow_ui_sim::startup::process_pending_timers(env);
    for _ in 0..3 {
        wow_ui_sim::startup::fire_one_on_update_tick(env);
        wow_ui_sim::startup::process_pending_timers(env);
    }
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
#[test]
fn objective_tracker_renders_supplied_right_edge_buttons() {
    crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape(
        &["Blizzard_ObjectiveTracker", "Blizzard_SplashFrame"],
        &[],
        |env, loaded| {
            assert!(loaded.iter().any(|name| name == "Blizzard_SplashFrame"));
            supply_button_quests(env);
            advance_tracker(env);
            assert_right_edge_buttons(env);
            let first = read_item_cooldown(env);
            assert_item_cooldown(first, 31_000.0);
            update_item_cooldown(env);
            advance_tracker(env);
            let second = read_item_cooldown(env);
            assert!(second.0 > first.0, "second item cooldown should start later");
            assert_item_cooldown(second, 47_000.0);
        },
    );
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
const SUPPLY_BUTTON_QUESTS: &str = r#"
A_Admin.SetQuestLogEntry(81000, 'Group and item', false)
A_Admin.SetQuestLogEntry(81001, 'No buttons', false)
A_Admin.SetQuestGroupCreatable(81000, true)
A_Admin.SetQuestGroupCreatable(81001, false)
local itemIndex = C_QuestLog.GetLogIndexForQuestID(81000)
A_Admin.SetQuestLogSpecialItem(itemIndex, 19001)
A_Admin.SetItemCooldown(19001, 31)
local start, duration, enable = GetQuestLogSpecialItemCooldown(itemIndex)
assert(start > 0 and duration == 31)
assert(type(enable) == "number" and enable == 1)
assert(GetQuestLogSpecialItemCooldown(C_QuestLog.GetLogIndexForQuestID(81001)) == nil)
C_QuestLog.AddQuestWatch(81000)
C_QuestLog.AddQuestWatch(81001)
local watched = {}
for index = 1, C_QuestLog.GetNumQuestWatches() do
    watched[C_QuestLog.GetQuestIDForQuestWatchIndex(index)] = true
end
assert(watched[81000] and watched[81001], "both supplied quests must be watched")
A_Admin.FireEvent("QUEST_WATCH_LIST_CHANGED", 81000, true)
A_Admin.FireEvent("QUEST_WATCH_LIST_CHANGED", 81001, true)
"#;

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn supply_button_quests(env: &WowLuaEnv) {
    env.exec(SUPPLY_BUTTON_QUESTS)
        .expect("admin inputs should supply the two watched quests");
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
const UPDATE_ITEM_COOLDOWN: &str = r#"
A_Admin.SetItemCooldown(19001, 47)
A_Admin.FireEvent("QUEST_LOG_UPDATE")
"#;

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
const READ_ITEM_COOLDOWN: &str = r#"
local block = QuestObjectiveTracker.usedBlocks[QuestObjectiveTracker.blockTemplate][81000]
local item = block.ItemButton
local start, duration = item.Cooldown:GetCooldownTimes()
local red, green, blue = item.icon:GetVertexColor()
return start, duration, red, green, blue
"#;

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn update_item_cooldown(env: &WowLuaEnv) {
    env.exec(UPDATE_ITEM_COOLDOWN)
        .expect("the tracker should receive its quest log refresh event");
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn read_item_cooldown(env: &WowLuaEnv) -> (f64, f64, f64, f64, f64) {
    env.eval(READ_ITEM_COOLDOWN)
        .expect("the item button cooldown and texture color should be readable")
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn assert_item_cooldown(actual: (f64, f64, f64, f64, f64), duration: f64) {
    assert!(actual.0 > 0.0, "cooldown start should be positive");
    assert_eq!(actual.1, duration);
    assert_eq!((actual.2, actual.3, actual.4), (1.0, 1.0, 1.0));
}

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
const READ_TRACKER_BUTTONS: &str = r#"
local module = QuestObjectiveTracker
local blocks = module.usedBlocks[module.blockTemplate]
local first, second = blocks[81000], blocks[81001]
local groupA, groupB
for _, button in pairs(module.usedRightEdgeFrames or {}) do
    if button:GetAttribute("questLogIndex") == nil then
        local id = button:GetAttribute("questID")
        if id == 81000 then groupA = button end
        if id == 81001 then groupB = button end
    end
end
local item = first and first.ItemButton
local index = C_QuestLog.GetLogIndexForQuestID(81000)
local _, itemID = GetQuestLogSpecialItemInfo(index)
return groupA and groupA:IsVisible() or false,
       groupB and groupB:IsVisible() or false,
       item and item:IsVisible() or false,
       second and second.ItemButton and second.ItemButton:IsVisible() or false,
       index, item and item:GetAttribute("questLogIndex") or 0,
       itemID or 0, item and tostring(item.icon:GetTexture()) or ""
"#;

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn assert_right_edge_buttons(env: &WowLuaEnv) {
    let actual: (bool, bool, bool, bool, i64, i64, i64, String) = env
        .eval(READ_TRACKER_BUTTONS)
        .expect("original tracker button state should be readable");
    assert_eq!((actual.0, actual.1, actual.2, actual.3), (true, false, true, false));
    assert_eq!(actual.4, actual.5);
    assert_eq!(actual.6, 19001);
    assert_eq!(actual.7, "19001");
}
#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
fn assert_seeded_quest_blocks(env: &WowLuaEnv) {
    let (
        first_exists,
        first_visible,
        first_title,
        second_exists,
        second_visible,
        second_title,
    ): (bool, bool, String, bool, bool, String) = env
        .eval(
            r#"
            local blocks = QuestObjectiveTracker.usedBlocks[QuestObjectiveTracker.blockTemplate]
            local first = blocks and blocks[80000]
            local second = blocks and blocks[80001]
            return first ~= nil, first and first:IsVisible() or false,
                   first and first.HeaderText:GetText() or "",
                   second ~= nil, second and second:IsVisible() or false,
                   second and second.HeaderText:GetText() or ""
            "#,
        )
        .expect("the original tracker block state should be readable");
    let expected_titles = if cfg!(feature = "client-wowforever") {
        ("[80] The Lost Expedition", "[80] Defending the Gates")
    } else {
        ("The Lost Expedition", "Defending the Gates")
    };
    assert!(first_exists, "quest 80000 should have a tracker block");
    assert!(first_visible, "quest 80000 tracker block should be visible");
    assert_eq!(first_title, expected_titles.0);
    assert!(second_exists, "quest 80001 should have a tracker block");
    assert!(second_visible, "quest 80001 tracker block should be visible");
    assert_eq!(second_title, expected_titles.1);
}
