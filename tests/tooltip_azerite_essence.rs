#![cfg(all(
    feature = "client-retail",
    feature = "retail-12-1-5",
    feature = "optional-tests"
))]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{AzeriteEssenceInfo, AzeriteEssenceMilestoneInfo};
use wow_ui_sim::screen::ScreenKind;

const ESSENCE_TWO: i32 = 2;
const ESSENCE_THREE: i32 = 3;

const ESSENCE_LUA_HELPERS: &str = r#"
function AzeriteTestContainsSpell(data, spellID)
    local spell = C_TooltipInfo.GetSpellByID(spellID)
    if not spell or not spell.lines then return false end
    for start = 1, #data.lines - #spell.lines + 1 do
        local matches = true
        for offset, expected in ipairs(spell.lines) do
            local actual = data.lines[start + offset - 1]
            if not actual or actual.leftText ~= expected.leftText
                or actual.rightText ~= expected.rightText then
                matches = false
                break
            end
        end
        if matches then return true end
    end
    return false
end

function AzeriteTestSpellIDs(data, candidateIDs)
    local selected = {}
    for _, spellID in ipairs(candidateIDs) do
        if AzeriteTestContainsSpell(data, spellID) then
            selected[spellID] = true
        end
    end
    return selected
end

function AzeriteTestSpellCount(spellIDs)
    local count = 0
    for _ in pairs(spellIDs) do count = count + 1 end
    return count
end

function AzeriteTestSameSpellIDs(left, right)
    for spellID in pairs(left) do
        if not right[spellID] then return false end
    end
    for spellID in pairs(right) do
        if not left[spellID] then return false end
    end
    return true
end

function AzeriteTestAssertSpellCount(data, candidates, expected, message)
    local spellIDs = AzeriteTestSpellIDs(data, candidates)
    assert(AzeriteTestSpellCount(spellIDs) == expected, message)
    return spellIDs
end

function AzeriteTestFindLine(data, text)
    for _, line in ipairs(data.lines) do
        if line.leftText and string.find(line.leftText, text, 1, true) then
            return line
        end
    end
end

function AzeriteTestFindPowerLine(data, powerState)
    for _, line in ipairs(data.lines) do
        if line.powerState == powerState then return line end
    end
end

function AzeriteTestFindEmptyPlainLine(data)
    for _, line in ipairs(data.lines) do
        if line.type == 0 and line.leftText == "" then return line end
    end
end

function AzeriteTestLineIndex(data, target)
    for index, line in ipairs(data.lines) do
        if line == target then return index end
    end
end

function AzeriteTestShowEssence(essenceID, rank)
    local button = setmetatable(
        { essenceID = essenceID, rank = rank },
        { __index = AzeriteEssenceButtonMixin }
    )
    button:OnEnter()
    local info = GameTooltip:GetPrimaryTooltipInfo()
    local data = GameTooltip:GetPrimaryTooltipData()
    if not data then return nil end
    assert(info and info.getterName == 'GetAzeriteEssence',
        'original Azerite essence accessor must run')
    assert(data.type == 8 and data.id == essenceID,
        'essence data must preserve type 8 and its essence ID')
    return data
end

function AzeriteTestShowSlot(milestoneID, slot, isMajorSlot)
    local button = setmetatable({
        milestoneID = milestoneID,
        slot = slot,
        isMajorSlot = isMajorSlot,
        isDraggable = false,
    }, { __index = AzeriteMilestoneSlotMixin })
    button:OnEnter()
    local info = GameTooltip:GetPrimaryTooltipInfo()
    local data = GameTooltip:GetPrimaryTooltipData()
    if not data then return nil end
    assert(info and info.getterName == 'GetAzeriteEssenceSlot',
        'original Azerite slot accessor must run')
    assert(data.type == 8, 'slot must preserve type-8 essence tooltip data')
    return data
end

AzeriteTestCandidates = {
    essenceTwoRankOne = {293019, 294668, 293019, 294668},
    essenceTwoRankTwo = {294653, 294687, 298080, 298082},
    essenceThreeRankOne = {293031, 294910, 293031, 294910},
    essenceThreeRankTwo = {294906, 294919, 300009, 300012},
}
"#;

const ESSENCE_ID_SELECTOR_LUA: &str = r#"
assert(AzeriteTestShowEssence(0, 1) == nil,
    'the original essence hover must keep an absent ID empty')
local two = AzeriteTestShowEssence(2, 1)
local twoIDs = AzeriteTestAssertSpellCount(
    two, AzeriteTestCandidates.essenceTwoRankOne, 2,
    'the first essence must produce both spell blocks')
local three = AzeriteTestShowEssence(3, 1)
local threeIDs = AzeriteTestAssertSpellCount(
    three, AzeriteTestCandidates.essenceThreeRankOne, 2,
    'the second essence must produce both spell blocks')
assert(not AzeriteTestSameSpellIDs(twoIDs, threeIDs),
    'distinct essence IDs must select distinct spell records')
"#;

const ESSENCE_RANK_SELECTOR_LUA: &str = r#"
assert(AzeriteTestShowEssence(2, 5) == nil,
    'the original essence hover must keep an absent rank empty')
local rankOne = AzeriteTestShowEssence(2, 1)
local rankOneIDs = AzeriteTestAssertSpellCount(
    rankOne, AzeriteTestCandidates.essenceTwoRankOne, 2,
    'rank one must produce both spell blocks')
local rankTwo = AzeriteTestShowEssence(2, 2)
local rankTwoIDs = AzeriteTestAssertSpellCount(
    rankTwo, AzeriteTestCandidates.essenceTwoRankTwo, 2,
    'rank two must produce both spell blocks')
assert(not AzeriteTestSameSpellIDs(rankOneIDs, rankTwoIDs),
    'distinct ranks must select distinct spell records')
"#;

const ACTIVE_MAJOR_SELECTOR_LUA: &str = r#"
local activeMajor = AzeriteTestShowEssence(2, 2)
local rankOne = AzeriteTestSpellIDs(activeMajor, AzeriteTestCandidates.essenceTwoRankOne)
local rankTwo = AzeriteTestSpellIDs(activeMajor, AzeriteTestCandidates.essenceTwoRankTwo)
assert(AzeriteTestSpellCount(rankOne) == 2,
    'an active major essence must use both rank-one description blocks')
assert(AzeriteTestSpellCount(rankTwo) == 0,
    'an active major essence must not use selected-rank actual blocks')
"#;

const ACTIVE_MINOR_SELECTOR_LUA: &str = r#"
local activeMinor = AzeriteTestShowEssence(3, 2)
local rankOne = AzeriteTestSpellIDs(activeMinor, AzeriteTestCandidates.essenceThreeRankOne)
local rankTwo = AzeriteTestSpellIDs(activeMinor, AzeriteTestCandidates.essenceThreeRankTwo)
assert(AzeriteTestSpellCount(rankOne) >= 1 and AzeriteTestSpellCount(rankTwo) >= 1,
    'an active minor essence must use its passive rank-one and major actual blocks')
"#;

const SOURCE_HEADING_LUA: &str = r#"
local nextRankTwo = AzeriteTestShowEssence(3, 1)
assert(AzeriteTestFindLine(nextRankTwo, 'Next Rank:'),
    'the verified next-rank stage must select its heading')
local rankZero = AzeriteTestShowEssence(3, 0)
assert(AzeriteTestFindLine(rankZero, 'Source:'),
    'the verified rank-zero stage must select the source heading')
local nextRankFour = AzeriteTestShowEssence(3, 3)
assert(not AzeriteTestFindLine(nextRankFour, 'Next Rank:'),
    'next rank four must disable the source-heading stage')
assert(select('#', C_TooltipInfo.GetAzeriteEssence(3, 4)) == 0,
    'a missing exact next-rank record must return zero values')
"#;

const SOURCE_SELECTION_LUA: &str = r#"
local fallback = AzeriteTestFindLine(AzeriteTestSourceFallback, 'Eternal Palace Raid')
assert(fallback and fallback.wrapText == true,
    'an empty second source field must use the first source')
local second = AzeriteTestFindLine(AzeriteTestSourceZero, 'Xander Silberman')
assert(second and second.wrapText == true,
    'selector result zero must use the second source field')
local first = AzeriteTestFindLine(AzeriteTestSourceOne, 'Marshal Gabriel')
assert(first and first.wrapText == true,
    'a nonzero selector result must use the first source field')
local empty = AzeriteTestSourceEmpty
assert(AzeriteTestFindLine(empty, 'Source:') == nil,
    'an empty next-rank-one source must omit SOURCE')
assert(AzeriteTestFindEmptyPlainLine(empty) == nil,
    'an empty next-rank-one source must omit its description row')
assert(AzeriteTestFindPowerLine(empty, 'major') == nil
    and AzeriteTestFindPowerLine(empty, 'minor') == nil,
    'next-rank one must not add inline power rows')
assert(AzeriteTestSourceAbsent == false,
    'an absent essence must stay absent')
"#;

const NEXT_RANK_INLINE_LUA: &str = r#"
assert(AzeriteTestShowEssence(3, 4) == nil,
    'an absent next-rank record must stay absent')
local rankTwo = AzeriteTestShowEssence(2, 1)
local rankThree = AzeriteTestShowEssence(3, 2)
local function assertOrder(data, sourceText, majorSpell, minorSpell)
    local heading = AzeriteTestFindLine(data, 'Next Rank:')
    local source = AzeriteTestFindLine(data, sourceText)
    local major = AzeriteTestFindPowerLine(data, 'major')
    local minor = AzeriteTestFindPowerLine(data, 'minor')
    local majorDescription = C_Spell.GetSpellDescription(majorSpell)
    local minorDescription = C_Spell.GetSpellDescription(minorSpell)
    assert(heading and source and major and minor,
        'the next-rank rows must include heading, source, major and minor')
    assert(type(majorDescription) == 'string' and majorDescription ~= ''
        and major.type == 5 and major.wrapText == true
        and major.leftText:find(majorDescription, 1, true),
        'the available major description must be wrapped type 5 with major state')
    assert(type(minorDescription) == 'string' and minorDescription ~= ''
        and minor.type == 5 and minor.wrapText == true
        and minor.leftText:find(minorDescription, 1, true),
        'the available minor description must be wrapped type 5 with minor state')
    assert(AzeriteTestLineIndex(data, heading) < AzeriteTestLineIndex(data, source)
        and AzeriteTestLineIndex(data, source) < AzeriteTestLineIndex(data, major)
        and AzeriteTestLineIndex(data, major) < AzeriteTestLineIndex(data, minor),
        'next-rank row order must be heading, source, major, minor')
end
assertOrder(rankTwo, 'Eternal Palace Raid', 294653, 294687)
assertOrder(rankThree, 'Battlegrounds', 294907, 294920)
"#;

const EMPTY_NEXT_RANK_SOURCE_LUA: &str = r#"
assert(AzeriteTestShowEssence(3, 4) == nil,
    'an absent next-rank record must stay absent')
local nonempty = AzeriteTestShowEssence(2, 1)
local empty = AzeriteTestShowEssence(3, 1)
assert(AzeriteTestFindLine(nonempty, 'Eternal Palace Raid'),
    'a distinct catalog source must stay visible')
local heading = AzeriteTestFindLine(empty, 'Next Rank:')
local source = AzeriteTestFindEmptyPlainLine(empty)
assert(heading and source and source.wrapText == true,
    'an empty higher-rank source must keep a wrapped type-0 row')
assert(AzeriteTestLineIndex(empty, heading) < AzeriteTestLineIndex(empty, source),
    'the empty source row must follow the next-rank heading')
"#;

const UNOWNED_INLINE_LUA: &str = r#"
local data = AzeriteTestShowEssence(3, 2)
assert(data and not AzeriteTestFindPowerLine(data, 'major')
    and not AzeriteTestFindPowerLine(data, 'minor'),
    'unowned essence must not add inline powers')
"#;

const CURRENT_RANK_LUA: &str = r#"
local current = C_TooltipInfo.GetAzeriteEssence(2)
local currentIDs = AzeriteTestAssertSpellCount(
    current, AzeriteTestCandidates.essenceTwoRankTwo, 2,
    'omitted rank must use the current player rank')
local rankZero = C_TooltipInfo.GetAzeriteEssence(2, 0)
local rankOneIDs = AzeriteTestAssertSpellCount(
    rankZero, AzeriteTestCandidates.essenceTwoRankOne, 2,
    'rank zero must select rank one')
assert(AzeriteTestSpellCount(currentIDs) == 2 and AzeriteTestSpellCount(rankOneIDs) == 2)
"#;

const LIST_MISS_RANK_LUA: &str = r#"
local listMiss = C_TooltipInfo.GetAzeriteEssence(2)
local listMissIDs = AzeriteTestAssertSpellCount(
    listMiss, AzeriteTestCandidates.essenceTwoRankOne, 2,
    'a present-player list miss must select rank one')
local rankOne = C_TooltipInfo.GetAzeriteEssence(2, 1)
local rankOneIDs = AzeriteTestSpellIDs(rankOne, AzeriteTestCandidates.essenceTwoRankOne)
assert(AzeriteTestSameSpellIDs(listMissIDs, rankOneIDs),
    'a player-list miss must select the rank-one spell records')
"#;

const EXPLICIT_RANK_WITHOUT_PLAYER_LUA: &str = r#"
local explicit = C_TooltipInfo.GetAzeriteEssence(2, 2)
assert(explicit and explicit.type == 8 and explicit.id == 2,
    'an explicit rank must bypass current-player rank lookup')
AzeriteTestAssertSpellCount(explicit, AzeriteTestCandidates.essenceTwoRankTwo, 2,
    'an explicit rank must produce its selected spell blocks')
assert(select('#', C_TooltipInfo.GetAzeriteEssence(2)) == 0,
    'an omitted rank must miss without player state')
"#;

const ESSENCE_MISS_ARITY_LUA: &str = r#"
assert(select('#', C_TooltipInfo.GetAzeriteEssence(-1, 1)) == 0,
    'a negative essence ID must return zero values')
assert(select('#', C_TooltipInfo.GetAzeriteEssence(0, 1)) == 0,
    'a missing essence record must return zero values')
assert(select('#', C_TooltipInfo.GetAzeriteEssence(999999)) == 0,
    'an omitted-rank missing essence must return zero values')
assert(select('#', C_TooltipInfo.GetAzeriteEssence(2, 5)) == 0,
    'a missing selected rank must return zero values')
local savedFormat = format
format = function() error('injected tooltip producer failure') end
local failedCount = select('#', C_TooltipInfo.GetAzeriteEssence(2, 2))
format = savedFormat
assert(failedCount == 0, 'a failed producer must return zero values')
"#;

const PLAYER_STATE_MISS_LUA: &str = r#"
assert(select('#', C_TooltipInfo.GetAzeriteEssence(2)) == 0,
    'an omitted-rank missing-player path must return zero values')
assert(select('#', C_TooltipInfo.GetAzeriteEssenceSlot(0)) == 0,
    'a missing-player slot path must return zero values')
"#;

const SLOT_STATE_MISS_LUA: &str = r#"
assert(select('#', C_TooltipInfo.GetAzeriteEssence(2)) == 0,
    'an omitted-rank missing-state path must return zero values')
assert(select('#', C_TooltipInfo.GetAzeriteEssenceSlot(0)) == 0,
    'a missing-rank-state slot path must return zero values')
"#;

const SLOT_MISS_ARITY_LUA: &str = r#"
assert(select('#', C_TooltipInfo.GetAzeriteEssenceSlot(1)) == 0,
    'an unconfigured slot must return zero values')
assert(select('#', C_TooltipInfo.GetAzeriteEssenceSlot(2)) == 0,
    'an empty essence record must return zero values')
"#;

const SLOT_SELECTOR_LUA: &str = r#"
local major = AzeriteTestShowSlot(101, 0, true)
assert(major.id == 2, 'slot zero must select its active essence')
AzeriteTestAssertSpellCount(major, AzeriteTestCandidates.essenceTwoRankOne, 2,
    'slot zero must keep both rank-one spell blocks')
local passive = AzeriteTestShowSlot(102, 1, false)
assert(passive.id == 3, 'slot one must select its distinct active essence')
AzeriteTestAssertSpellCount(passive, AzeriteTestCandidates.essenceThreeRankOne, 1,
    'a nonzero slot must keep only the passive rank-one block')
assert(AzeriteTestSpellCount(
    AzeriteTestSpellIDs(passive, AzeriteTestCandidates.essenceThreeRankTwo)) == 0,
    'a nonzero slot must omit the main-power block')
"#;

const EMPTY_SLOT_HOVER_LUA: &str = r#"
assert(AzeriteTestShowSlot(103, 2, false) == nil,
    'the original slot hover must keep an empty essence result absent')
"#;

const ARGUMENT_ERRORS_LUA: &str = r#"
local ok, message = pcall(C_TooltipInfo.GetAzeriteEssence)
assert(not ok and tostring(message):find(
    'Usage: local data = C_TooltipInfo.GetAzeriteEssence', 1, true),
    'a missing required essence ID must raise the verified usage error')
local rankOK, rankMessage = pcall(C_TooltipInfo.GetAzeriteEssence, 2, 4294967296)
assert(not rankOK and tostring(rankMessage):find(
    'outside of expected range 0 to 4294967295', 1, true),
    'an out-of-range unsigned rank must raise the verified reader error')
local slotOK, slotMessage = pcall(C_TooltipInfo.GetAzeriteEssenceSlot)
assert(not slotOK and tostring(slotMessage):find(
    'Usage: local data = C_TooltipInfo.GetAzeriteEssenceSlot', 1, true),
    'a missing required slot must raise the verified usage error')
local rangeOK, rangeMessage = pcall(C_TooltipInfo.GetAzeriteEssenceSlot, 4)
assert(not rangeOK and tostring(rangeMessage):find(
    'outside of expected range 0 to 3', 1, true),
    'slot four must raise the verified range error')
"#;

fn capture_source_variants(env: &WowLuaEnv) {
    capture_absent_source(env);
    set_source_selector_result(env, 0);
    run_azerite_script(
        env,
        concat!(
            "AzeriteTestSourceFallback = AzeriteTestShowEssence(2, 0); ",
            "AzeriteTestSourceZero = AzeriteTestShowEssence(3, 0)"
        ),
        "the original Azerite hover should produce source candidates",
    );
    set_source_selector_result(env, 1);
    run_azerite_script(
        env,
        "AzeriteTestSourceOne = AzeriteTestShowEssence(3, 0)",
        "the original Azerite hover should use the supplied source selector",
    );
    override_tooltip_source(env, ESSENCE_THREE, 1);
    run_azerite_script(
        env,
        "AzeriteTestSourceEmpty = AzeriteTestShowEssence(3, 0)",
        "the original Azerite hover should accept empty source fields",
    );
}

fn capture_absent_source(env: &WowLuaEnv) {
    run_azerite_script(
        env,
        "AzeriteTestSourceAbsent = AzeriteTestShowEssence(999, 0) or false",
        "the original Azerite hover should keep an absent essence empty",
    );
}

fn set_source_selector_result(env: &WowLuaEnv, result: i32) {
    env.state()
        .borrow_mut()
        .azerite_essence
        .source_faction_selector_result = Some(result);
}

fn override_tooltip_source(env: &WowLuaEnv, essence_id: i32, rank: u32) {
    env.state()
        .borrow_mut()
        .azerite_essence
        .tooltip_source_overrides
        .insert((essence_id, rank), (String::new(), String::new()));
}

fn run_azerite_script(env: &WowLuaEnv, script: &str, message: &str) {
    env.exec(script).expect(message);
}

fn original_azerite_tooltip_env() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    assert_azerite_ui_selected(&ui);
    let env = load_selected_azerite_tooltip_addons(&ui);
    load_original_azerite_consumers(&env, &ui);
    run_azerite_script(
        &env,
        ESSENCE_LUA_HELPERS,
        "Azerite test helpers should load",
    );
    env
}

fn assert_azerite_ui_selected(ui: &std::path::Path) {
    let selected = wow_ui_sim::loader::discover_blizzard_addons_for_screen(ui, ScreenKind::Game);
    assert!(
        selected
            .iter()
            .any(|(name, _)| name == "Blizzard_SharedXMLGame"),
        "selected Game TOCs must include Blizzard_SharedXMLGame"
    );

    let selected_ui = wow_ui_sim::loader::discover_blizzard_addon_closure_for_screen_with_overrides(
        ui,
        ScreenKind::Game,
        &["Blizzard_AzeriteEssenceUI"],
        &[],
    );
    assert!(
        selected_ui
            .iter()
            .any(|(name, _)| name == "Blizzard_AzeriteEssenceUI"),
        "the Game load-on-demand closure must select Blizzard_AzeriteEssenceUI"
    );
}

fn load_selected_azerite_tooltip_addons(ui: &std::path::Path) -> WowLuaEnv {
    let env = new_blizzard_addon_env(ui);
    env.exec("UIPanelWindows = UIPanelWindows or {}")
        .expect("the selected UI environment must provide UIPanelWindows");
    let loaded = load_blizzard_addon_closure_into_env(
        &env,
        ui,
        &[
            "Blizzard_SharedXMLGame",
            "Blizzard_FrameXMLBase",
            "Blizzard_GameTooltip",
            "Blizzard_LoadLocale",
        ],
        &[],
    );
    assert!(
        loaded.iter().any(|name| name == "Blizzard_SharedXMLGame"),
        "selected Game TOCs must load Blizzard_SharedXMLGame"
    );
    assert!(
        loaded.iter().any(|name| name == "Blizzard_GameTooltip"),
        "the selected GameTooltip addon must attach its native handler mixin"
    );
    env
}

fn load_original_azerite_consumers(env: &WowLuaEnv, ui: &std::path::Path) {
    let source_path = ui.join("Blizzard_AzeriteEssenceUI/Blizzard_AzeriteEssenceUI.lua");
    let source = std::fs::read_to_string(source_path)
        .expect("the selected Azerite UI source must be present in the profile cache");
    env.exec_named(
        &source,
        "@Interface/AddOns/Blizzard_AzeriteEssenceUI/Blizzard_AzeriteEssenceUI.lua",
    )
    .expect("the original Azerite UI mixins should load without its unrelated Bootstrap/XML");
    assert_original_azerite_consumer_sources(env);
}

fn assert_original_azerite_consumer_sources(env: &WowLuaEnv) {
    run_azerite_script(
        env,
        r#"
        local buttonSource = debug.getinfo(AzeriteEssenceButtonMixin.OnEnter, 'S').source
        assert(buttonSource:find('Blizzard_AzeriteEssenceUI.lua', 1, true), buttonSource)
        local slotSource = debug.getinfo(AzeriteMilestoneSlotMixin.OnEnter, 'S').source
        assert(slotSource:find('Blizzard_AzeriteEssenceUI.lua', 1, true), slotSource)
        local essenceSource = debug.getinfo(GameTooltip.SetAzeriteEssence, 'S').source
        local slotGetterSource = debug.getinfo(GameTooltip.SetAzeriteEssenceSlot, 'S').source
        assert(essenceSource:find('TooltipDataHandler.lua', 1, true), essenceSource)
        assert(slotGetterSource:find('TooltipDataHandler.lua', 1, true), slotGetterSource)
        "#,
        "the original Azerite consumers and accessors should load",
    );
}

fn seed_essence(env: &WowLuaEnv, id: i32, rank: i32) {
    let mut state = env.state().borrow_mut();
    state.player.present = true;
    state.azerite_essence.essences.insert(
        id,
        AzeriteEssenceInfo {
            id,
            name: format!("Fixture essence {id}"),
            rank,
            icon: 1,
            unlocked: true,
            valid: true,
            access_rank: 4,
            has_never_activated: false,
        },
    );
}

fn seed_milestone(env: &WowLuaEnv, milestone_id: i32, slot: i32, essence_id: i32) {
    env.state()
        .borrow_mut()
        .azerite_essence
        .milestones
        .push(AzeriteEssenceMilestoneInfo {
            id: milestone_id,
            required_level: 1,
            slot: Some(slot),
            unlocked: true,
            can_unlock: false,
            is_major_slot: slot == 0,
            swirl_scale: 1.0,
            requires_only_aura: false,
            spell_id: 1,
            rank: None,
            active_essence_id: Some(essence_id),
        });
}

#[test]
fn retail_azerite_essence_original_hover_selects_distinct_essences_and_absent() {
    let env = original_azerite_tooltip_env();
    seed_essence(&env, ESSENCE_TWO, 1);
    seed_essence(&env, ESSENCE_THREE, 1);
    run_azerite_script(
        &env,
        ESSENCE_ID_SELECTOR_LUA,
        "the original Azerite hover must select distinct essence IDs",
    );
}

#[test]
fn retail_azerite_essence_original_hover_selects_distinct_ranks_and_absent() {
    let env = original_azerite_tooltip_env();
    seed_essence(&env, ESSENCE_TWO, 1);
    run_azerite_script(
        &env,
        ESSENCE_RANK_SELECTOR_LUA,
        "the original Azerite hover must select distinct exact ranks",
    );
}

#[test]
fn retail_azerite_essence_original_hover_applies_active_slot_spell_selectors() {
    let env = original_azerite_tooltip_env();
    seed_essence(&env, ESSENCE_TWO, 2);
    seed_essence(&env, ESSENCE_THREE, 2);
    seed_milestone(&env, 103, 0, ESSENCE_TWO);
    run_azerite_script(
        &env,
        ACTIVE_MAJOR_SELECTOR_LUA,
        "major-slot spell selection failed",
    );
    env.state().borrow_mut().azerite_essence.milestones.clear();
    seed_milestone(&env, 104, 1, ESSENCE_THREE);
    run_azerite_script(
        &env,
        ACTIVE_MINOR_SELECTOR_LUA,
        "minor-slot spell selection failed",
    );
}

#[test]
fn retail_azerite_essence_original_hover_keeps_verified_next_rank_headings() {
    let env = original_azerite_tooltip_env();
    seed_essence(&env, ESSENCE_THREE, 1);
    run_azerite_script(
        &env,
        SOURCE_HEADING_LUA,
        "the original Azerite hover must follow verified next-rank heading conditions",
    );
}

#[test]
fn retail_azerite_essence_original_hover_selects_verified_source_rows_and_absent() {
    let env = original_azerite_tooltip_env();
    seed_essence(&env, ESSENCE_TWO, 1);
    seed_essence(&env, ESSENCE_THREE, 1);
    capture_source_variants(&env);
    run_azerite_script(
        &env,
        SOURCE_SELECTION_LUA,
        "the original Azerite hover must select verified source rows",
    );
}

#[test]
fn retail_azerite_essence_original_hover_orders_next_rank_power_rows() {
    let env = original_azerite_tooltip_env();
    seed_essence(&env, ESSENCE_TWO, 1);
    seed_essence(&env, ESSENCE_THREE, 2);
    run_azerite_script(
        &env,
        NEXT_RANK_INLINE_LUA,
        "the original Azerite hover must order next-rank power rows",
    );
    env.state()
        .borrow_mut()
        .azerite_essence
        .essences
        .get_mut(&ESSENCE_THREE)
        .expect("the owned essence fixture must remain present")
        .unlocked = false;
    run_azerite_script(
        &env,
        UNOWNED_INLINE_LUA,
        "the next-rank inline rows must use existing ownership state",
    );
}

#[test]
fn retail_azerite_essence_original_hover_keeps_empty_higher_rank_source_row() {
    let env = original_azerite_tooltip_env();
    seed_essence(&env, ESSENCE_TWO, 1);
    seed_essence(&env, ESSENCE_THREE, 1);
    env.state()
        .borrow_mut()
        .azerite_essence
        .tooltip_source_overrides
        .insert((ESSENCE_THREE, 2), (String::new(), String::new()));
    run_azerite_script(
        &env,
        EMPTY_NEXT_RANK_SOURCE_LUA,
        "the original Azerite hover must keep an empty higher-rank source row",
    );
}

#[test]
fn retail_azerite_essence_slot_original_hover_selects_two_slots_and_absent() {
    let absent_env = original_azerite_tooltip_env();
    seed_essence(&absent_env, ESSENCE_TWO, 1);
    seed_milestone(&absent_env, 103, 2, 0);
    run_azerite_script(
        &absent_env,
        EMPTY_SLOT_HOVER_LUA,
        "empty slot hover should stay absent",
    );

    let env = original_azerite_tooltip_env();
    seed_essence(&env, ESSENCE_TWO, 2);
    seed_essence(&env, ESSENCE_THREE, 2);
    seed_milestone(&env, 101, 0, ESSENCE_TWO);
    seed_milestone(&env, 102, 1, ESSENCE_THREE);
    run_azerite_script(
        &env,
        SLOT_SELECTOR_LUA,
        "the original slot hover must select each slot",
    );
}

#[test]
fn retail_azerite_essence_current_and_explicit_rank_selection() {
    let env = WowLuaEnv::new().expect("Retail Lua environment should initialize");
    run_azerite_script(
        &env,
        ESSENCE_LUA_HELPERS,
        "Azerite test helpers should load",
    );
    seed_essence(&env, ESSENCE_TWO, 2);
    run_azerite_script(
        &env,
        CURRENT_RANK_LUA,
        "current and zero rank selection failed",
    );
    env.state().borrow_mut().azerite_essence.essences.clear();
    seed_essence(&env, ESSENCE_THREE, 1);
    run_azerite_script(
        &env,
        LIST_MISS_RANK_LUA,
        "rank-list miss must select rank one",
    );
    env.state().borrow_mut().player.present = false;
    env.state().borrow_mut().azerite_essence.essences.clear();
    run_azerite_script(
        &env,
        EXPLICIT_RANK_WITHOUT_PLAYER_LUA,
        "explicit rank selection must bypass player rank state",
    );
}

#[test]
fn retail_azerite_essence_failures_keep_zero_value_arity() {
    let env = WowLuaEnv::new().expect("Retail Lua environment should initialize");
    seed_essence(&env, ESSENCE_TWO, 4);
    run_azerite_script(
        &env,
        ESSENCE_MISS_ARITY_LUA,
        "essence failures must return zero values",
    );
    env.state().borrow_mut().player.present = false;
    run_azerite_script(
        &env,
        PLAYER_STATE_MISS_LUA,
        "missing player must return zero values",
    );
    {
        let mut state = env.state().borrow_mut();
        state.player.present = true;
        state.azerite_essence.essences.clear();
    }
    run_azerite_script(
        &env,
        SLOT_STATE_MISS_LUA,
        "missing player state must return zero values",
    );
}

#[test]
fn retail_azerite_slot_population_misses_keep_zero_value_arity() {
    let env = WowLuaEnv::new().expect("Retail Lua environment should initialize");
    seed_essence(&env, ESSENCE_TWO, 1);
    seed_milestone(&env, 105, 2, 0);
    run_azerite_script(
        &env,
        SLOT_MISS_ARITY_LUA,
        "slot population misses must return zero values",
    );
}

#[test]
fn retail_azerite_getters_raise_only_verified_argument_errors() {
    let env = WowLuaEnv::new().expect("Retail Lua environment should initialize");
    run_azerite_script(
        &env,
        ARGUMENT_ERRORS_LUA,
        "verified argument errors must be raised",
    );
}
