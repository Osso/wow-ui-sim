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

// Raw column candidates from the exact-build DB2 export. These tests do not
// assert the INFERRED role-to-column mapping.
const ESSENCE_TWO_RANK_ONE_CANDIDATES: [u32; 4] = [293_019, 294_668, 293_019, 294_668];
const ESSENCE_TWO_RANK_TWO_CANDIDATES: [u32; 4] = [294_653, 294_687, 298_080, 298_082];
const ESSENCE_THREE_RANK_ONE_CANDIDATES: [u32; 4] = [293_031, 294_910, 293_031, 294_910];
const ESSENCE_THREE_RANK_TWO_CANDIDATES: [u32; 4] = [294_906, 294_919, 300_009, 300_012];

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
    for _ in pairs(spellIDs) do
        count = count + 1
    end
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

function AzeriteTestShowEssence(essenceID, rank)
    local button = setmetatable(
        { essenceID = essenceID, rank = rank },
        { __index = AzeriteEssenceButtonMixin }
    )
    button:OnEnter()
    local info = GameTooltip:GetPrimaryTooltipInfo()
    local data = GameTooltip:GetPrimaryTooltipData()
    assert(data, 'RED: selected essence/rank must publish tooltip data')
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
    assert(data and data.type == 8,
        'slot must publish type-8 essence tooltip data')
    assert(info and info.getterName == 'GetAzeriteEssenceSlot',
        'original Azerite slot accessor must run')
    return data
end
"#;

fn original_azerite_tooltip_env() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let selected = wow_ui_sim::loader::discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    assert!(
        selected
            .iter()
            .any(|(name, _)| name == "Blizzard_SharedXMLGame"),
        "selected Game TOCs must include Blizzard_SharedXMLGame"
    );

    let selected_ui = wow_ui_sim::loader::discover_blizzard_addon_closure_for_screen_with_overrides(
        &ui,
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

    let env = new_blizzard_addon_env(&ui);
    env.exec("UIPanelWindows = UIPanelWindows or {}")
        .expect("the selected UI environment must provide UIPanelWindows");
    let shared = load_blizzard_addon_closure_into_env(
        &env,
        &ui,
        &[
            "Blizzard_SharedXMLGame",
            "Blizzard_FrameXMLBase",
            "Blizzard_GameTooltip",
            "Blizzard_LoadLocale",
        ],
        &[],
    );
    assert!(
        shared.iter().any(|name| name == "Blizzard_SharedXMLGame"),
        "selected Game TOCs must load Blizzard_SharedXMLGame"
    );
    assert!(
        shared.iter().any(|name| name == "Blizzard_GameTooltip"),
        "the selected GameTooltip addon must attach its native handler mixin"
    );

    let source_path = ui.join("Blizzard_AzeriteEssenceUI/Blizzard_AzeriteEssenceUI.lua");
    let source = std::fs::read_to_string(source_path)
        .expect("the selected Azerite UI source must be present in the profile cache");
    env.exec_named(
        &source,
        "@Interface/AddOns/Blizzard_AzeriteEssenceUI/Blizzard_AzeriteEssenceUI.lua",
    )
    .expect("the original Azerite UI mixins should load without its unrelated Bootstrap/XML");
    env.exec(
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
    )
    .expect("the original Azerite UI consumers and TooltipDataHandler accessors should load");
    env.exec(ESSENCE_LUA_HELPERS)
        .expect("Azerite selector test helpers should load");
    env
}

fn lua_number_array(values: &[u32]) -> String {
    let values = values
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{{values}}}")
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
fn retail_azerite_essence_original_consumer_selects_exact_essence_and_rank() {
    let env = original_azerite_tooltip_env();
    seed_essence(&env, ESSENCE_TWO, 1);
    seed_essence(&env, ESSENCE_THREE, 1);

    let essence_two_rank_one = lua_number_array(&ESSENCE_TWO_RANK_ONE_CANDIDATES);
    let essence_two_rank_two = lua_number_array(&ESSENCE_TWO_RANK_TWO_CANDIDATES);
    let essence_three_rank_two = lua_number_array(&ESSENCE_THREE_RANK_TWO_CANDIDATES);
    env.exec(&format!(
        r#"
        local rankOne = AzeriteTestShowEssence({ESSENCE_TWO}, 1)
        assert(rankOne.id == {ESSENCE_TWO})
        local rankOneIDs = AzeriteTestAssertSpellCount(
            rankOne, {essence_two_rank_one}, 2, 'rank one must produce both spell blocks')

        -- The player-state rank is 1; this original UI passes rank 2 explicitly.
        local rankTwo = AzeriteTestShowEssence({ESSENCE_TWO}, 2)
        assert(rankTwo.id == {ESSENCE_TWO})
        local rankTwoIDs = AzeriteTestAssertSpellCount(
            rankTwo, {essence_two_rank_two}, 2, 'rank two must produce both spell blocks')
        assert(not AzeriteTestSameSpellIDs(rankOneIDs, rankTwoIDs),
            'different exact ranks must select different spell records')

        local otherEssence = AzeriteTestShowEssence({ESSENCE_THREE}, 2)
        local otherIDs = AzeriteTestAssertSpellCount(
            otherEssence, {essence_three_rank_two}, 2,
            'the second essence must produce both spell blocks')
        assert(not AzeriteTestSameSpellIDs(rankTwoIDs, otherIDs),
            'different essence IDs must select different spell records')

        local currentRank = C_TooltipInfo.GetAzeriteEssence({ESSENCE_TWO})
        assert(currentRank and currentRank.type == 8 and currentRank.id == {ESSENCE_TWO})
        local currentIDs = AzeriteTestSpellIDs(currentRank, {essence_two_rank_one})
        assert(AzeriteTestSameSpellIDs(rankOneIDs, currentIDs),
            'an omitted rank must use the current player rank')

        local zeroRank = C_TooltipInfo.GetAzeriteEssence({ESSENCE_TWO}, 0)
        assert(zeroRank and zeroRank.type == 8 and zeroRank.id == {ESSENCE_TWO})
        local zeroIDs = AzeriteTestSpellIDs(zeroRank, {essence_two_rank_one})
        assert(AzeriteTestSameSpellIDs(rankOneIDs, zeroIDs),
            'rank zero must select rank one')

        assert(select('#', C_TooltipInfo.GetAzeriteEssence({ESSENCE_TWO}, 5)) == 0,
            'a missing exact rank must not select a lower rank')
        "#
    ))
    .expect("the original Azerite essence consumer must select the DB2 essence and rank");

    {
        let mut state = env.state().borrow_mut();
        state.player.present = false;
        state.azerite_essence.essences.clear();
    }
    let essence_two_rank_two = lua_number_array(&ESSENCE_TWO_RANK_TWO_CANDIDATES);
    env.exec(&format!(
        r#"
        local explicit = C_TooltipInfo.GetAzeriteEssence({ESSENCE_TWO}, 2)
        assert(explicit and explicit.type == 8 and explicit.id == {ESSENCE_TWO},
            'an explicit rank must not require current player state')
        AzeriteTestAssertSpellCount(explicit, {essence_two_rank_two}, 2,
            'an explicit rank must still produce its selected spells')
        "#
    ))
    .expect("an explicit rank should bypass missing current-player rank data");

    env.state().borrow_mut().player.present = true;
    seed_essence(&env, ESSENCE_THREE, 1);
    let essence_two_rank_one = lua_number_array(&ESSENCE_TWO_RANK_ONE_CANDIDATES);
    env.exec(&format!(
        r#"
        local listMiss = C_TooltipInfo.GetAzeriteEssence({ESSENCE_TWO})
        assert(listMiss and listMiss.type == 8 and listMiss.id == {ESSENCE_TWO},
            'a player-list miss must still use rank one')
        local listMissIDs = AzeriteTestSpellIDs(listMiss, {essence_two_rank_one})
        local rankOne = C_TooltipInfo.GetAzeriteEssence({ESSENCE_TWO}, 1)
        local rankOneIDs = AzeriteTestSpellIDs(rankOne, {essence_two_rank_one})
        assert(AzeriteTestSameSpellIDs(listMissIDs, rankOneIDs),
            'a list miss rank zero must be raised to rank one')

        local savedFormat = format
        format = function() error('injected tooltip producer failure') end
        local failedCount = select('#', C_TooltipInfo.GetAzeriteEssence({ESSENCE_TWO}, 2))
        format = savedFormat
        assert(failedCount == 0,
            'a failed tooltip producer must return zero Lua values')
        "#
    ))
    .expect("a missing essence from a present player rank list should use rank one");
}

#[test]
fn retail_azerite_essence_slot_original_consumer_applies_verified_modes() {
    let env = original_azerite_tooltip_env();
    seed_essence(&env, ESSENCE_TWO, 2);
    seed_essence(&env, ESSENCE_THREE, 2);
    seed_milestone(&env, 101, 0, ESSENCE_TWO);
    seed_milestone(&env, 102, 1, ESSENCE_THREE);

    let essence_two_rank_one = lua_number_array(&ESSENCE_TWO_RANK_ONE_CANDIDATES);
    let essence_two_rank_two = lua_number_array(&ESSENCE_TWO_RANK_TWO_CANDIDATES);
    let essence_three_rank_one = lua_number_array(&ESSENCE_THREE_RANK_ONE_CANDIDATES);
    let essence_three_rank_two = lua_number_array(&ESSENCE_THREE_RANK_TWO_CANDIDATES);
    env.exec(&format!(
        r#"
        local majorSlot = AzeriteTestShowSlot(101, 0, true)
        assert(majorSlot.id == {ESSENCE_TWO})
        AzeriteTestAssertSpellCount(majorSlot, {essence_two_rank_one}, 2,
            'slot zero must keep main and passive rank-one spell blocks')
        assert(AzeriteTestSpellCount(
            AzeriteTestSpellIDs(majorSlot, {essence_two_rank_two})) == 0,
            'active major slot must use rank-one description spells')

        local passiveSlot = AzeriteTestShowSlot(102, 1, false)
        assert(passiveSlot.id == {ESSENCE_THREE})
        AzeriteTestAssertSpellCount(passiveSlot, {essence_three_rank_one}, 1,
            'a nonzero slot must keep its passive rank-one spell block')
        assert(AzeriteTestSpellCount(
            AzeriteTestSpellIDs(passiveSlot, {essence_three_rank_two})) == 0,
            'nonzero slots must omit the main-power block')
        "#
    ))
    .expect("the original Azerite slot consumer must select its active essence and mode");
}

#[test]
fn retail_azerite_essence_getters_keep_native_zero_value_miss_arity() {
    let env = WowLuaEnv::new().expect("Retail Lua environment should initialize");
    seed_essence(&env, ESSENCE_TWO, 4);
    seed_milestone(&env, 101, 0, ESSENCE_TWO);

    env.exec(&format!(
        r#"
        assert(select('#', C_TooltipInfo.GetAzeriteEssence(-1, 1)) == 0,
            'negative essence ID is a zero-value miss')
        assert(select('#', C_TooltipInfo.GetAzeriteEssence(999999, 1)) == 0,
            'missing essence record is a zero-value miss')
        assert(select('#', C_TooltipInfo.GetAzeriteEssence({ESSENCE_TWO}, 5)) == 0,
            'missing selected rank is a zero-value miss')
        "#
    ))
    .expect("the Azerite essence API should preserve missing-record arity");

    env.state().borrow_mut().player.present = false;
    env.exec(&format!(
        r#"
        assert(select('#', C_TooltipInfo.GetAzeriteEssence({ESSENCE_TWO})) == 0,
            'a missing player is a zero-value omitted-rank miss')
        assert(select('#', C_TooltipInfo.GetAzeriteEssenceSlot(0)) == 0,
            'a missing player is a zero-value slot miss')
        "#
    ))
    .expect("explicit rank should bypass the player rank lookup");

    {
        let mut state = env.state().borrow_mut();
        state.player.present = true;
        state.azerite_essence.essences.clear();
    }
    env.exec(&format!(
        r#"
        assert(select('#', C_TooltipInfo.GetAzeriteEssence({ESSENCE_TWO})) == 0,
            'missing current-player essence state is a zero-value miss')
        assert(select('#', C_TooltipInfo.GetAzeriteEssenceSlot(0)) == 0,
            'missing current-player rank state is a zero-value miss')
        assert(select('#', C_TooltipInfo.GetAzeriteEssenceSlot(1)) == 0,
            'an absent active slot is a zero-value miss')
        "#
    ))
    .expect("the Azerite getter miss paths should return exactly zero values");
}
