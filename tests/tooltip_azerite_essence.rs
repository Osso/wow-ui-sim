#![cfg(all(
    feature = "client-retail",
    feature = "retail-12-1-5",
    feature = "optional-tests"
))]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{AzeriteEssenceInfo, AzeriteEssenceMilestoneInfo};
use wow_ui_sim::screen::ScreenKind;

#[path = "tooltip_full_env_helpers.rs"]
mod tooltip_full_env_helpers;

const ESSENCE_TWO: i32 = 2;
const ESSENCE_THREE: i32 = 3;
const RANK_ONE_MAJOR: u32 = 293_019;
const RANK_ONE_MINOR: u32 = 294_668;
const RANK_TWO_MAJOR_ACTUAL: u32 = 298_080;
const RANK_TWO_MINOR_ACTUAL: u32 = 298_082;
const ESSENCE_THREE_RANK_TWO_MAJOR_ACTUAL: u32 = 300_009;
const ESSENCE_THREE_RANK_TWO_MINOR_ACTUAL: u32 = 300_012;

fn original_azerite_tooltip_env() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let selected = wow_ui_sim::loader::discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    assert!(
        selected
            .iter()
            .any(|(name, _)| name == "Blizzard_SharedXMLGame"),
        "selected Game TOCs must include Blizzard_SharedXMLGame"
    );

    let env = tooltip_full_env_helpers::setup_full_env();
    let loaded =
        load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_AzeriteEssenceUI"], &[]);
    for addon in ["Blizzard_AzeriteEssenceUI"] {
        assert!(
            loaded.iter().any(|name| name == addon),
            "{addon}: {loaded:?}"
        );
    }
    env.exec(
        r#"
        assert(C_AddOns.IsAddOnLoaded('Blizzard_SharedXMLGame'))
        assert(C_AddOns.IsAddOnLoaded('Blizzard_AzeriteEssenceUI'))
        local buttonSource = debug.getinfo(AzeriteEssenceButtonMixin.OnEnter, 'S').source:gsub('\\', '/')
        assert(buttonSource:find('Blizzard_AzeriteEssenceUI.lua', 1, true), buttonSource)
        local slotSource = debug.getinfo(AzeriteMilestoneSlotMixin.OnEnter, 'S').source:gsub('\\', '/')
        assert(slotSource:find('Blizzard_AzeriteEssenceUI.lua', 1, true), slotSource)
        local essenceSource = debug.getinfo(GameTooltip.SetAzeriteEssence, 'S').source:gsub('\\', '/')
        local slotGetterSource = debug.getinfo(GameTooltip.SetAzeriteEssenceSlot, 'S').source:gsub('\\', '/')
        assert(essenceSource:find('TooltipDataHandler.lua', 1, true), essenceSource)
        assert(slotGetterSource:find('TooltipDataHandler.lua', 1, true), slotGetterSource)
        "#,
    )
    .expect("selected Azerite UI and TooltipDataHandler consumers should load");
    env
}

fn seed_essence(env: &WowLuaEnv, id: i32, rank: i32) {
    env.state().borrow_mut().azerite_essence.essences.insert(
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

    env.exec(&format!(
        r#"
        local function spellName(spellID)
            local data = C_TooltipInfo.GetSpellByID(spellID)
            assert(data and data.lines[1] and data.lines[1].leftText,
                'DB2 spell fixture must have a spell name: ' .. spellID)
            return data.lines[1].leftText
        end
        local function hasSpell(data, spellID)
            local name = spellName(spellID)
            for _, line in ipairs(data.lines) do
                if line.leftText == name then return true end
            end
            return false
        end
        local function showEssence(essenceID, rank)
            local button = setmetatable({{ essenceID = essenceID, rank = rank }},
                {{ __index = AzeriteEssenceButtonMixin }})
            button:OnEnter()
            local info = GameTooltip:GetPrimaryTooltipInfo()
            local data = GameTooltip:GetPrimaryTooltipData()
            assert(info and info.getterName == 'GetAzeriteEssence',
                'original Azerite essence accessor must run')
            assert(data, 'RED: selected essence/rank must publish tooltip data')
            assert(data.type == 8 and data.id == essenceID,
                'essence data must preserve type 8 and its essence ID')
            return data
        end

        local rankOne = showEssence({ESSENCE_TWO}, 1)
        assert(hasSpell(rankOne, {RANK_ONE_MAJOR}) and hasSpell(rankOne, {RANK_ONE_MINOR}),
            'rank one must select its two actual spells')
        assert(not hasSpell(rankOne, {RANK_TWO_MAJOR_ACTUAL})
            and not hasSpell(rankOne, {RANK_TWO_MINOR_ACTUAL}),
            'rank one must not fall through to rank two')

        -- The player-state rank is 1; this original UI passes rank 2 explicitly.
        local rankTwo = showEssence({ESSENCE_TWO}, 2)
        assert(hasSpell(rankTwo, {RANK_TWO_MAJOR_ACTUAL})
            and hasSpell(rankTwo, {RANK_TWO_MINOR_ACTUAL}),
            'the supplied rank must select the exact rank-two spells')
        assert(not hasSpell(rankTwo, {RANK_ONE_MAJOR})
            and not hasSpell(rankTwo, {RANK_ONE_MINOR}),
            'the supplied rank must bypass current-rank selection')

        local otherEssence = showEssence({ESSENCE_THREE}, 2)
        assert(hasSpell(otherEssence, {ESSENCE_THREE_RANK_TWO_MAJOR_ACTUAL})
            and hasSpell(otherEssence, {ESSENCE_THREE_RANK_TWO_MINOR_ACTUAL}),
            'a second essence must select its own rank-two spells')

        local currentRank = C_TooltipInfo.GetAzeriteEssence({ESSENCE_TWO})
        assert(currentRank and currentRank.type == 8 and currentRank.id == {ESSENCE_TWO})
        assert(hasSpell(currentRank, {RANK_ONE_MAJOR}) and hasSpell(currentRank, {RANK_ONE_MINOR}),
            'an omitted rank must use the current player rank')

        local zeroRank = C_TooltipInfo.GetAzeriteEssence({ESSENCE_TWO}, 0)
        assert(zeroRank and zeroRank.type == 8 and zeroRank.id == {ESSENCE_TWO})
        assert(hasSpell(zeroRank, {RANK_ONE_MAJOR}) and hasSpell(zeroRank, {RANK_ONE_MINOR}),
            'rank zero must select rank one')

        assert(select('#', C_TooltipInfo.GetAzeriteEssence({ESSENCE_TWO}, 5)) == 0,
            'a missing exact rank must not select the lower rank')
        "#
    ))
    .expect("the original Azerite essence consumer must select the DB2 essence and rank");

    env.state()
        .borrow_mut()
        .azerite_essence
        .essences
        .remove(&ESSENCE_TWO);
    env.exec(&format!(
        r#"
        local data = C_TooltipInfo.GetAzeriteEssence({ESSENCE_TWO})
        assert(data and data.type == 8 and data.id == {ESSENCE_TWO},
            'a player-list miss must still use rank one')
        local rankOneMajor = C_TooltipInfo.GetSpellByID({RANK_ONE_MAJOR}).lines[1].leftText
        local rankOneMinor = C_TooltipInfo.GetSpellByID({RANK_ONE_MINOR}).lines[1].leftText
        local foundMajor, foundMinor = false, false
        for _, line in ipairs(data.lines) do
            foundMajor = foundMajor or line.leftText == rankOneMajor
            foundMinor = foundMinor or line.leftText == rankOneMinor
        end
        assert(foundMajor and foundMinor, 'list miss rank zero must be raised to rank one')
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

    env.exec(&format!(
        r#"
        local function hasSpell(data, spellID)
            local name = C_TooltipInfo.GetSpellByID(spellID).lines[1].leftText
            for _, line in ipairs(data.lines) do
                if line.leftText == name then return true end
            end
            return false
        end
        local function showSlot(milestoneID, slot, isMajorSlot)
            local button = setmetatable({{
                milestoneID = milestoneID,
                slot = slot,
                isMajorSlot = isMajorSlot,
                isDraggable = false,
            }}, {{ __index = AzeriteMilestoneSlotMixin }})
            button:OnEnter()
            local info = GameTooltip:GetPrimaryTooltipInfo()
            local data = GameTooltip:GetPrimaryTooltipData()
            assert(info and info.getterName == 'GetAzeriteEssenceSlot',
                'original Azerite slot accessor must run')
            assert(data and data.type == 8,
                'slot must publish type-8 essence tooltip data')
            return data
        end

        local majorSlot = showSlot(101, 0, true)
        assert(majorSlot.id == {ESSENCE_TWO})
        assert(hasSpell(majorSlot, {RANK_ONE_MAJOR})
            and hasSpell(majorSlot, {RANK_ONE_MINOR}),
            'slot zero keeps main and passive rank-one description spells')
        assert(not hasSpell(majorSlot, {RANK_TWO_MAJOR_ACTUAL}),
            'active major slot uses the rank-one description spell')

        local passiveSlot = showSlot(102, 1, false)
        assert(passiveSlot.id == {ESSENCE_THREE})
        assert(hasSpell(passiveSlot, 294910),
            'a passive slot keeps its rank-one minor description spell')
        assert(not hasSpell(passiveSlot, {ESSENCE_THREE_RANK_TWO_MAJOR_ACTUAL}),
            'nonzero slots omit the main-power block')
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

    env.state().borrow_mut().azerite_essence.essences.clear();
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
