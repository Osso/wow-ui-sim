//! Supplied challenge state through the selected original tracker consumers.
#![cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::lua_api::WowLuaEnv;

fn load_tracker() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    let loaded =
        load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_ObjectiveTracker"], &[]);
    assert!(
        loaded
            .iter()
            .any(|name| name == "Blizzard_ObjectiveTracker")
    );
    env.exec(r#"
        assert(ScenarioTimerFrame and ScenarioObjectiveTracker.ChallengeModeBlock)
        assert(ScenarioTimerFrame.CheckTimers == ScenarioTimerMixin.CheckTimers)
        assert(ScenarioObjectiveTracker.ChallengeModeBlock.Activate == ScenarioObjectiveTrackerChallengeModeMixin.Activate)
        local source = debug.getinfo(ScenarioTimerFrame.CheckTimers, 'S').source:gsub('\\', '/')
        assert(source:find('Blizzard_ObjectiveTracker/Blizzard_ScenarioObjectiveTracker.lua', 1, true), source)
    "#).unwrap();
    env
}

const START: &str = r#"
    A_Admin.SetChallengeModeMapInfo(9001, {
        name='Supplied forecast dungeon', id=9001, timeLimit=1500,
        texture=123, backgroundTexture=456, mapID=789,
    })
    A_Admin.StartChallengeRun({mapChallengeModeID=9001, eventMapID=7001, level=12, affixIDs={},
        wasActiveKeystoneCharged=true, numDeaths=2, timeLost=30})
    A_Admin.StartWorldElapsedTimer(1, 'Supplied timer', 1, 120)
"#;

#[test]
fn original_challenge_timer_reads_supplied_run_and_reset() {
    let env = load_tracker();
    env.exec(START).unwrap();
    env.exec(
        r#"
        local block = ScenarioObjectiveTracker.ChallengeModeBlock
        assert(block:IsActive() and block.timerID == 1)
        assert(block.timeLimit == 1500 and block.Level:GetText():find('12', 1, true))
        assert(block.StatusBar:GetValue() == 1380)
        assert(block.deathCount == 2 and block.timeLost == 30)
        assert(not block.wasDepleted)
        A_Admin.SetWorldElapsedTime(1, 345)
        ScenarioTimerFrame:StartTimer(block)
        ScenarioTimerFrame:OnUpdate(0)
        assert(block.StatusBar:GetValue() == 1155)
        A_Admin.ClearChallengeRun()
        assert(not block:IsActive() and not ScenarioTimerFrame:IsShown())
        assert(C_ChallengeMode.GetActiveChallengeMapID() == nil)
        assert(not C_ChallengeMode.IsChallengeModeActive())
        assert(select('#', GetWorldElapsedTimers()) == 0)
    "#,
    )
    .unwrap();
    env.exec(
        r#"
        A_Admin.SetChallengeModeMapInfo(9002, {name='Second supplied dungeon', id=9002,
            timeLimit=1800, texture=321, backgroundTexture=654, mapID=987})
        A_Admin.StartChallengeRun({mapChallengeModeID=9002, eventMapID=7002, level=5, affixIDs={9},
            wasActiveKeystoneCharged=false, numDeaths=1, timeLost=8})
        A_Admin.StartWorldElapsedTimer(2, 'Second timer', 1, 330)
        local block = ScenarioObjectiveTracker.ChallengeModeBlock
        assert(block.timerID == 2 and block.timeLimit == 1800)
        assert(block.StatusBar:GetValue() == 1470)
        assert(block.Level:GetText():find('5', 1, true) and block.wasDepleted)
        assert(block.deathCount == 1 and block.timeLost == 8)
        assert(select(2, C_ChallengeMode.GetActiveKeystoneInfo())[1] == 9)
        A_Admin.CompleteChallengeRun({mapChallengeModeID=9002, level=5,
            time=1234000, onTime=true, keystoneUpgradeLevels=2, practiceRun=false,
            isMapRecord=false, isAffixRecord=false, isEligibleForScore=true, members={}})
        assert(C_ChallengeMode.GetChallengeCompletionInfo().time == 1234000)
        assert(not C_ChallengeMode.IsChallengeModeActive())
        assert(not ScenarioObjectiveTracker.ChallengeModeBlock:IsActive())
    "#,
    )
    .unwrap();
}

#[test]
fn original_scenario_criteria_render_order_quantity_and_completion() {
    let env = load_tracker();
    env.exec(r#"
        ScenarioObjectiveTracker:InitModule()
        A_Admin.SetScenarioCriteria({stepID=1, title='Fixture stage', description='Fixture objectives'}, {
            {criteriaID=901, description='First boss', criteriaType=165, quantity=0, totalQuantity=1, quantityString='0'},
            {criteriaID=902, description='Enemy forces', quantity=25, totalQuantity=100,
                quantityString='25%', isWeightedProgress=true},
        })
        assert(ScenarioObjectiveTracker:ShouldShowCriteria() == true)
        local block = ScenarioObjectiveTracker.ObjectivesBlock
        -- Blizzard_ScenarioObjectiveTracker.lua resets the block before every UpdateCriteria.
        block:Reset()
        ScenarioObjectiveTracker:UpdateCriteria(C_ScenarioInfo.GetScenarioStepInfo().numCriteria)
        assert(block:GetExistingLine(1).Text:GetText() == '0/1 First boss')
        assert(block:GetExistingLine(2).Text:GetText() == 'Enemy forces')
        assert(block:GetExistingLine(2).progressBar.Bar:GetValue() == 25)
        assert(C_ScenarioInfo.GetCriteriaInfo(2).quantityString == '25%')
        assert(C_ScenarioInfo.GetCriteriaInfoByStep(1, 1).criteriaID == 901)
        A_Admin.SetScenarioCriteria({stepID=1, title='Fixture stage', description='Fixture objectives'}, {
            {criteriaID=901, description='First boss', criteriaType=165, quantity=1, totalQuantity=1, quantityString='1', completed=true},
            {criteriaID=902, description='Enemy forces', quantity=70, totalQuantity=100,
                quantityString='70%', isWeightedProgress=true},
        })
        block:Reset()
        ScenarioObjectiveTracker:UpdateCriteria(2)
        assert(block:GetExistingLine(1).Text:GetText() == '1/1 First boss')
        assert(block:GetExistingLine(2).progressBar.Bar:GetValue() == 70)
        assert(C_ScenarioInfo.GetCriteriaInfo(1).completed)
        assert(C_ScenarioInfo.GetCriteriaInfo(2).quantity == 70)
        assert(C_ScenarioInfo.GetCriteriaInfo(3) == nil)
        local row = C_ScenarioInfo.GetCriteriaInfo(2)
        row.quantity = 999
        assert(C_ScenarioInfo.GetCriteriaInfo(2).quantity == 70)
        assert(C_Scenario.GetCriteriaInfo == nil and C_Scenario.GetCriteriaInfoByStep == nil)
    "#).unwrap();
    let independent = WowLuaEnv::new().unwrap();
    independent.exec("assert(C_ScenarioInfo.GetCriteriaInfo(1) == nil); assert(C_ChallengeMode.GetActiveChallengeMapID() == nil)").unwrap();
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn original_challenges_dungeon_icon_reads_supplied_map_metadata() {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    let loaded = load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_ChallengesUI"], &[]);
    assert!(loaded.iter().any(|name| name == "Blizzard_ChallengesUI"));
    env.exec(
        r#"
        local icon = CreateFrame('Frame', nil, UIParent, 'ChallengesDungeonIconFrameTemplate')
        assert(icon.SetUp == ChallengesDungeonIconMixin.SetUp)
        local source = debug.getinfo(icon.SetUp, 'S').source:gsub('\\\\', '/')
        assert(source:find('Blizzard_ChallengesUI/', 1, true), source)
        A_Admin.SetChallengeModeMapInfo(9001, {name='First supplied dungeon', id=9001,
            timeLimit=1500, texture=123, backgroundTexture=456, mapID=789})
        icon:SetUp({id=9001, level=12}, true)
        assert(icon.Icon:GetTexture() == 123 and icon.HighestLevel:GetText() == '12')
        A_Admin.SetChallengeModeMapInfo(9001, {name='Changed supplied dungeon', id=9001,
            timeLimit=1800, texture=321, backgroundTexture=654, mapID=987})
        icon:SetUp({id=9001, level=5}, true)
        assert(icon.Icon:GetTexture() == 321 and icon.HighestLevel:GetText() == '5')
        local name, id, limit, texture, background, map = C_ChallengeMode.GetMapUIInfo(9001)
        assert(name == 'Changed supplied dungeon' and id == 9001 and limit == 1800)
        assert(texture == 321 and background == 654 and map == 987)
    "#,
    )
    .unwrap();
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
fn load_map_icon() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&ui);
    let loaded = load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_ChallengesUI"], &[]);
    assert!(loaded.iter().any(|name| name == "Blizzard_ChallengesUI"));
    env.exec(r#"
        icon = CreateFrame('Frame', nil, UIParent, 'ChallengesDungeonIconFrameTemplate')
        assert(icon.SetUp == ChallengesDungeonIconMixin.SetUp)
        local source = debug.getinfo(icon.SetUp, 'S').source:gsub('\\', '/')
        assert(source:find('Blizzard_ChallengesUI/', 1, true), source)
        -- FileDataIDs: committed listfile, achievement_bg_wineos_underxminutes and challenges-background.
        function suppliedMap()
            return {name='Supplied dungeon', id=9001, timeLimit=1500,
                texture=236393, backgroundTexture=629538, mapID=789}
        end
        A_Admin.SetChallengeModeMapInfo(9001, {name='Previous dungeon', id=9002,
            timeLimit=1800, backgroundTexture=627119, mapID=790})
        local name, id, limit, texture, background, map = C_ChallengeMode.GetMapUIInfo(9001)
        assert(name == 'Previous dungeon' and id == 9002 and limit == 1800)
        assert(texture == nil and background == 627119 and map == 790)
        icon:SetUp({id=9001, level=5}, true)
        assert(icon.Icon:GetTexture() == nil and icon.HighestLevel:GetText() == '5')
        A_Admin.SetChallengeModeMapInfo(9001, suppliedMap())
        function checkSuppliedMap()
            local name, id, limit, texture, background, map = C_ChallengeMode.GetMapUIInfo(9001)
            assert(name == 'Supplied dungeon' and id == 9001 and limit == 1500)
            assert(texture == 236393 and background == 629538 and map == 789)
            icon:SetUp({id=9001, level=12}, true)
            assert(icon.Icon:GetTexture() == 236393 and icon.HighestLevel:GetText() == '12')
        end
        checkSuppliedMap()
    "#).unwrap();
    env
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn original_map_icon_accepts_omitted_texture_and_supplied_background() {
    let env = load_map_icon();
    env.exec(
        r#"
        local row = suppliedMap()
        row.texture = nil
        A_Admin.SetChallengeModeMapInfo(9001, row)
        local name, id, limit, texture, background, map = C_ChallengeMode.GetMapUIInfo(9001)
        assert(name == row.name and id == row.id and limit == row.timeLimit and map == row.mapID)
        assert(texture == nil and background == 629538)
        assert(select('#', C_ChallengeMode.GetMapUIInfo(9001)) == 6)
        icon:SetUp({id=9001, level=5}, true)
        assert(icon.Icon:GetTexture() == nil and icon.HighestLevel:GetText() == '5')
        A_Admin.SetChallengeModeMapInfo(9001, suppliedMap())
        checkSuppliedMap()
    "#,
    )
    .unwrap();
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn original_map_icon_preserves_map_after_named_missing_background_error() {
    let env = load_map_icon();
    env.exec(r#"
        local row = suppliedMap()
        row.backgroundTexture = nil
        local ok, message = pcall(A_Admin.SetChallengeModeMapInfo, 9001, row)
        assert(not ok)
        assert(message:find('mapInfo.backgroundTexture: expected number, got nil', 1, true), message)
        assert(not message:find('argument -1', 1, true), message)
        checkSuppliedMap()
    "#).unwrap();
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn original_map_icon_preserves_map_after_named_invalid_field_errors() {
    let env = load_map_icon();
    env.exec(r#"
        local function reject(field, value, expected)
            local row = suppliedMap()
            row[field] = value
            local ok, message = pcall(A_Admin.SetChallengeModeMapInfo, 9001, row)
            assert(not ok, field)
            assert(message:find('mapInfo.'..field..': '..expected, 1, true), message)
            assert(not message:find('argument -1', 1, true), message)
            checkSuppliedMap()
        end
        for _, field in ipairs({'name', 'id', 'timeLimit', 'texture', 'backgroundTexture', 'mapID'}) do
            local expectedType = field == 'name' and 'string' or 'number'
            reject(field, false, 'expected '..expectedType..', got boolean')
            if field ~= 'texture' then
                reject(field, nil, 'expected '..expectedType..', got nil')
            end
        end
        reject('timeLimit', -1, 'supplied seconds must be finite and nonnegative')
        reject('timeLimit', math.huge, 'supplied seconds must be finite and nonnegative')
        reject('id', 1.5, 'expected integer, got non-integer number')
        reject('mapID', 2147483648, 'expected i32')
        local ok, message = pcall(A_Admin.SetChallengeModeMapInfo, 9001, false)
        assert(not ok and message:find('mapInfo: expected table, got boolean', 1, true), message)
        checkSuppliedMap()
    "#).unwrap();
}

#[test]
fn challenge_admin_events_observe_updated_state_and_reject_invalid_input() {
    let env = WowLuaEnv::new().unwrap();
    let errors_before = env.state().borrow().lua_errors.len();
    env.exec(r#"
        local events = {}
        local f = CreateFrame('Frame')
        for _, event in ipairs({'CHALLENGE_MODE_START', 'WORLD_STATE_TIMER_START', 'SCENARIO_CRITERIA_UPDATE', 'CHALLENGE_MODE_RESET', 'CHALLENGE_MODE_COMPLETED'}) do f:RegisterEvent(event) end
        f:SetScript('OnEvent', function(_, event, id)
            events[#events+1] = event
            if event == 'CHALLENGE_MODE_START' then assert(id == 7001 and C_ChallengeMode.GetActiveChallengeMapID() == 9001)
            elseif event == 'WORLD_STATE_TIMER_START' then
                assert(id == 1 and GetWorldElapsedTimers() == 1)
                local name, elapsed, kind = GetWorldElapsedTime(id)
                assert(name == 'Supplied timer' and elapsed == 120 and kind == 1)
            elseif event == 'SCENARIO_CRITERIA_UPDATE' then assert(id == 901 and C_ScenarioInfo.GetCriteriaInfo(1).quantity == 7)
            else assert(id == 7001 and not C_ChallengeMode.IsChallengeModeActive()) end
        end)
        A_Admin.StartChallengeRun({mapChallengeModeID=9001, eventMapID=7001, level=2, affixIDs={9,10}, wasActiveKeystoneCharged=false, numDeaths=0, timeLost=0})
        local level, affixes, charged = C_ChallengeMode.GetActiveKeystoneInfo()
        assert(level == 2 and #affixes == 2 and affixes[2] == 10 and charged == false)
        affixes[1] = 999
        assert(select(2, C_ChallengeMode.GetActiveKeystoneInfo())[1] == 9)
        A_Admin.StartWorldElapsedTimer(1, 'Supplied timer', 1, 120)
        A_Admin.AdvanceWorldElapsedTime(1, 10)
        assert(select(2, GetWorldElapsedTime(1)) == 130)
        A_Admin.SetScenarioCriteria({stepID=1, title='Supplied', description='Supplied'}, {{criteriaID=901, description='Supplied', quantity=7, quantityString='7'}})
        A_Admin.ClearChallengeRun()
        assert(table.concat(events, ',') == 'CHALLENGE_MODE_START,WORLD_STATE_TIMER_START,SCENARIO_CRITERIA_UPDATE,CHALLENGE_MODE_RESET')
        assert(not pcall(A_Admin.AdvanceWorldElapsedTime, 1, -1))
        assert(not pcall(A_Admin.StartWorldElapsedTimer, 1, 'bad', 1, 0))
    "#).unwrap();
    assert_eq!(env.state().borrow().lua_errors.len(), errors_before);
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
#[test]
fn original_scenario_tracker_reads_supplied_criteria_visibility_on_init() {
    let env = load_tracker();
    env.exec(
        r#"
        A_Admin.SetScenarioCriteriaShown(false)
        assert(C_Scenario.ShouldShowCriteria() == false)
        ScenarioObjectiveTracker:InitModule()
        assert(ScenarioObjectiveTracker:ShouldShowCriteria() == false)
        A_Admin.SetScenarioCriteriaShown(true)
        assert(C_Scenario.ShouldShowCriteria() == true)
        ScenarioObjectiveTracker:InitModule()
        assert(ScenarioObjectiveTracker:ShouldShowCriteria() == true)
    "#,
    )
    .unwrap();
    let independent = WowLuaEnv::new().unwrap();
    let shown: bool = independent.eval("return C_Scenario.ShouldShowCriteria()").unwrap();
    assert!(shown, "criteria default to shown in a fresh environment");
}
