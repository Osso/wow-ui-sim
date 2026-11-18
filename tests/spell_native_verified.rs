//! Optional Retail tests for the bounded R88k native routes.
//! The LoD SpellSearch consumer is runtime-selected through SharedTalentUI's startup dependency.
#![cfg(all(
    feature = "client-retail",
    feature = "retail-12-1-5",
    feature = "optional-tests"
))]

use std::collections::HashMap;

use crate::common::blizzard_addon_harness::{blizzard_ui_dir, build_blizzard_addon_closure_env};
use rilua::table_security::wrap_host_secret_number;
use rilua::{LuaApiMut, Val};
use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::loader::discover_blizzard_startup_addons_for_screen;
use wow_ui_sim::lua_api::{WowLuaEnv, state::BagItem};
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn load_selected_consumers(roots: &[&str], function: &str, source_suffix: &str) -> WowLuaEnv {
    assert_eq!(ACTIVE, ClientProfile::Retail);
    let ui_dir = blizzard_ui_dir();
    let startup_addons = discover_blizzard_startup_addons_for_screen(&ui_dir, ScreenKind::Game);
    for root in roots {
        assert!(
            startup_addons.iter().any(|addon| addon.name == *root),
            "{root} must be selected by Retail Game-screen addon discovery"
        );
    }

    let (env, loaded) = build_blizzard_addon_closure_env(&ui_dir, roots, &[]);
    for root in roots {
        assert!(loaded.iter().any(|name| name == root));
    }
    env.exec(&format!(
        r#"
        local fn = {function}
        local source = debug.getinfo(fn, "S").source
        local suffix = "{source_suffix}"
        assert(type(fn) == "function" and source:sub(-#suffix) == suffix,
            "expected original selected consumer source, got " .. tostring(source))
        "#
    ))
    .expect("selected consumer source must be loaded from the pinned cache");
    env
}

fn add_bag_stack(env: &WowLuaEnv, slot: i32, item_id: u32, stack_count: i32) {
    env.state().borrow_mut().bag_items.insert(
        (0, slot),
        BagItem {
            owned_identity: None,
            item_id,
            stack_count,
            hyperlink: None,
            durability: None,
            applied_transmog: None,
        },
    );
}

fn publish_secret_number(env: &WowLuaEnv, name: &str, number: f64) {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    let value = wrap_host_secret_number(lua.state_mut(), number);
    lua.state_mut().push(value);
    lua.set_global_val(name, value).unwrap();
    lua.state_mut().pop();
}

const R88R_SECRET_READER_OVERLAY: &str = r#"
local castUsage = 'Usage: local castCount = C_Spell.GetSpellCastCount(spellIdentifier)'
local displayUsage = 'Usage: local displayCount = C_Spell.GetSpellDisplayCount(spellIdentifier [, maxDisplayCount, replacementString])'
local function expectUsage(call, usage)
    local ok, err = pcall(call)
    assert(not ok and tostring(err):find(usage, 1, true), tostring(err))
end
local function expectRange(call)
    local ok, err = pcall(call)
    assert(not ok and tostring(err):find('outside of expected range -2147483648 to 2147483647', 1, true), tostring(err))
end
expectUsage(function() C_Spell.GetSpellCastCount(R88RLowA) end, castUsage)
expectUsage(function() C_Spell.GetSpellCastCount(R88RLowB) end, castUsage)
assert(C_Spell.GetSpellCastCount(R88RHighA) == 7)
assert(C_Spell.GetSpellCastCount(R88RHighB) == 2)
expectUsage(function() C_Spell.GetSpellCastCount() end, castUsage)
expectUsage(function() C_Spell.GetSpellDisplayCount(R88RLowA) end, displayUsage)
assert(C_Spell.GetSpellDisplayCount(R88RHighA, R88RMaxOne, 'OVER') == 'OVER')
for _, maximum in ipairs({R88RMaxOne, R88RMaxZero, R88RMaxMinusOne}) do
    assert(C_Spell.GetSpellDisplayCount(R88RHighA, maximum, 'OVER') == 'OVER')
end
expectRange(function() C_Spell.GetSpellDisplayCount(R88RHighA, R88RMaxHigh) end)
expectRange(function() C_Spell.GetSpellDisplayCount(R88RHighA, R88RMaxLow) end)
assert(C_Spell.GetSpellDisplayCount(R88RHighA) == '7')
"#;

const MASK2_CACHE_ASSERTIONS: &str = r#"
    for _, case in ipairs(cases) do
        local spellID, expected = case[1], case[2]
        local item = {
            GetSpellChargeInfo = function() return nil end,
            GetSpellID = function() return spellID end,
            SetCachedChargeValues = function(self, count, shown)
                self.count, self.shown = count, shown
            end,
        }
        CooldownViewerCooldownItemMixin.CacheChargeValues(item)
        assert(item.count == expected,
            'spell=' .. spellID .. ' count=' .. tostring(item.count) .. ', expected=' .. expected)
        assert(item.shown == (expected > 0))
    end
"#;

fn seed_mask2_reagent(
    env: &WowLuaEnv,
    spell_id: u32,
    reagent_id: i32,
    quantity: i32,
    first_slot: i32,
    candidates: &[(i64, i32)],
) {
    assert!(wow_ui_sim::spells::get_spell(spell_id).is_some());
    let property_candidates = candidates
        .iter()
        .map(|&(signed_value, available_count)| {
            wow_ui_sim::c_api::spell_reagent_inventory::SpellReagentPropertyCandidate {
                signed_value,
                available_count: i64::from(available_count),
            }
        })
        .collect::<Vec<_>>();
    {
        let mut sim = env.state().borrow_mut();
        sim.spell_cast_context_is_null = Some(false);
        sim.spell_reagent_pairs
            .insert(spell_id, vec![(reagent_id, quantity)]);
        sim.spell_reagent_inventory
            .metadata_masks
            .insert(reagent_id, 2);
        if !property_candidates.is_empty() {
            sim.spell_reagent_inventory
                .property_candidates
                .insert(reagent_id, property_candidates);
        }
    }
    for (offset, &(_, count)) in candidates.iter().enumerate() {
        add_bag_stack(env, first_slot + offset as i32, reagent_id as u32, count);
    }
}

fn assert_mask2_cached_counts(env: &WowLuaEnv, cases: &str) {
    env.exec(&format!("local cases = {cases}\n{MASK2_CACHE_ASSERTIONS}"))
        .expect("the original CooldownViewer must cache every expected reagent count");
}

#[test]
fn r88r_secret_selector_and_display_maximum_readers() {
    let env = WowLuaEnv::new().unwrap();
    seed_mask2_reagent(&env, 19_750, 31_001, 1, 1, &[(1, 7)]);
    seed_mask2_reagent(&env, 642, 31_002, 1, 2, &[(1, 2)]);
    for (name, number) in [
        ("R88RLowA", 0.5),
        ("R88RLowB", 1.0),
        ("R88RHighA", 19_750.5),
        ("R88RHighB", 642.5),
        ("R88RMaxOne", 1.0),
        ("R88RMaxZero", 0.0),
        ("R88RMaxMinusOne", -1.0),
        ("R88RMaxHigh", 2_147_483_648.0),
        ("R88RMaxLow", -2_147_483_649.0),
    ] {
        publish_secret_number(&env, name, number);
    }
    env.exec("assert(C_Spell.GetSpellCastCount(R88RLowB) == 0)")
        .expect("empty metadata source skips the selector gate");
    env.state().borrow_mut().spell_identifier_metadata_source = Some(0);
    env.exec("assert(C_Spell.GetSpellCastCount(R88RLowB) == 0)")
        .expect("zero metadata source skips the selector gate");
    env.state().borrow_mut().spell_identifier_metadata_source = Some(1);
    env.exec(R88R_SECRET_READER_OVERLAY)
        .expect("R88r.r1 secret selector and maximum readers");
}

#[test]
fn cast_count_consumers_preserve_ordinary_and_absent_inputs() {
    let env = load_selected_consumers(
        &["Blizzard_CooldownViewer"],
        "CooldownViewerCooldownItemMixin.CacheChargeValues",
        "Blizzard_CooldownViewer/CooldownViewer.lua",
    );
    seed_mask2_reagent(&env, 19_750, 31_001, 1, 1, &[(1, 7)]);
    seed_mask2_reagent(&env, 642, 31_002, 1, 2, &[(1, 2)]);
    env.exec(
        r#"
        for _, case in ipairs({{19750, 7}, {642, 2}}) do
            local item = {
                GetSpellChargeInfo = function() return nil end,
                GetSpellID = function() return case[1] end,
                SetCachedChargeValues = function(self, count, shown)
                    self.count, self.shown = count, shown
                end,
            }
            CooldownViewerCooldownItemMixin.CacheChargeValues(item)
            assert(item.count == case[2] and item.shown)
        end
        local absent = {
            cooldownChargesCount = 11,
            GetSpellChargeInfo = function() return nil end,
            GetSpellID = function() return nil end,
            SetCachedChargeValues = function(self, count, shown)
                self.count, self.shown = count, shown
            end,
        }
        CooldownViewerCooldownItemMixin.CacheChargeValues(absent)
        assert(absent.count == 11 and not absent.shown)
    "#,
    )
    .unwrap();
}

#[test]
fn cast_count_mask2_divides_candidate_counts_before_summing_in_cooldown_viewer() {
    let env = load_selected_consumers(
        &["Blizzard_CooldownViewer"],
        "CooldownViewerCooldownItemMixin.CacheChargeValues",
        "Blizzard_CooldownViewer/CooldownViewer.lua",
    );
    seed_mask2_reagent(&env, 19_750, 31_001, 3, 1, &[(2, 7), (-3, 11)]);
    seed_mask2_reagent(&env, 642, 31_002, 4, 3, &[(-2, 5), (4, 15)]);
    assert_mask2_cached_counts(&env, "{{19750, 5}, {642, 4}}");
}

#[test]
fn cast_count_mask2_with_empty_candidates_keeps_sum_then_divide_in_cooldown_viewer() {
    let env = load_selected_consumers(
        &["Blizzard_CooldownViewer"],
        "CooldownViewerCooldownItemMixin.CacheChargeValues",
        "Blizzard_CooldownViewer/CooldownViewer.lua",
    );
    seed_mask2_reagent(&env, 19_750, 31_001, 3, 1, &[]);
    add_bag_stack(&env, 1, 31_001, 7);
    add_bag_stack(&env, 2, 31_001, 11);
    assert_mask2_cached_counts(&env, "{{19750, 6}}");
}

#[test]
fn cast_count_mask2_without_qualifying_property_keeps_sum_then_divide_in_cooldown_viewer() {
    let env = load_selected_consumers(
        &["Blizzard_CooldownViewer"],
        "CooldownViewerCooldownItemMixin.CacheChargeValues",
        "Blizzard_CooldownViewer/CooldownViewer.lua",
    );
    seed_mask2_reagent(&env, 19_750, 31_001, 3, 1, &[(1, 7), (-1, 11)]);
    assert_mask2_cached_counts(&env, "{{19750, 6}}");
}

#[test]
fn ranged_auto_attack_spell_consumer_uses_the_shared_identifier() {
    let ui_dir = blizzard_ui_dir();
    let search_toc =
        TocFile::from_file(&ui_dir.join("Blizzard_SpellSearch/Blizzard_SpellSearch.toc"))
            .expect("Retail SpellSearch TOC");
    let shared_toc =
        TocFile::from_file(&ui_dir.join("Blizzard_SharedTalentUI/Blizzard_SharedTalentUI.toc"))
            .expect("Retail SharedTalentUI TOC");
    assert!(search_toc.is_load_on_demand());
    assert!(!shared_toc.is_load_on_demand());
    assert!(
        shared_toc
            .dependencies()
            .iter()
            .any(|dependency| dependency == "Blizzard_SpellSearch")
    );
    let startup_addons = discover_blizzard_startup_addons_for_screen(&ui_dir, ScreenKind::Game);
    for name in ["Blizzard_SharedTalentUI", "Blizzard_SpellSearch"] {
        assert!(startup_addons.iter().any(|addon| addon.name == name));
    }

    let env = load_selected_consumers(
        &["Blizzard_SharedTalentUI", "Blizzard_ActionBar"],
        "SpellSearchUtil.GetActionbarStatusForSpellBookItemInfo",
        "Blizzard_SpellSearch/Blizzard_SpellSearchUtil.lua",
    );
    assert_eq!(env.state().borrow().ranged_auto_attack_spell_id, 0);
    let search_loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_SpellSearch')")
        .expect("SpellSearch is the selected SharedTalentUI dependency");
    assert!(search_loaded);
    {
        let mut state = env.state().borrow_mut();
        state.action_bars.clear();
        state.ranged_auto_attack_spell_id = 31_337;
    }
    env.exec(
        r#"
        local itemType = Enum.SpellBookItemType.Spell
        local function status(spellID)
            return SpellSearchUtil.GetActionbarStatusForSpellBookItemInfo({
                itemType = itemType,
                isPassive = false,
                isOffSpec = false,
                spellID = spellID,
                actionID = spellID,
            })
        end
        local statuses = ActionButtonUtil.ActionBarActionStatus
        assert(status(31337) == statuses.NotMissing,
            'matching spell status=' .. tostring(status(31337)))
        assert(status(31338) == statuses.MissingFromAllBars,
            'different spell status=' .. tostring(status(31338)))
        assert(C_Spell.IsRangedAutoAttackSpell(31337) == true)
        assert(C_Spell.IsRangedAutoAttackSpell(31338) == false)
        "#,
    )
    .expect("the original SpellSearch consumer must exempt only the shared ranged spell ID");

    env.state().borrow_mut().ranged_auto_attack_spell_id = 0;
    env.exec(
        r#"
        assert(C_Spell.IsRangedAutoAttackSpell(0) == true,
            'the verified initial shared identifier is zero')
        "#,
    )
    .expect("identifier zero matches the verified initial zero value");
}

#[test]
fn cast_count_null_context_returns_zero_through_cooldown_viewer() {
    let env = load_selected_consumers(
        &["Blizzard_CooldownViewer"],
        "CooldownViewerCooldownItemMixin.CacheChargeValues",
        "Blizzard_CooldownViewer/CooldownViewer.lua",
    );
    {
        let mut state = env.state().borrow_mut();
        state.spell_reagent_pairs.insert(19_750, vec![(31_001, 1)]);
        state.spell_cast_context_is_null = Some(true);
    }
    add_bag_stack(&env, 1, 31_001, 7);

    env.exec(
        r#"
        R88KCooldownItem = {
            GetSpellChargeInfo = function() return nil end,
            GetSpellID = function() return 19750 end,
            SetCachedChargeValues = function(self, count, shown)
                self.count = count
                self.shown = shown
            end,
        }
        CooldownViewerCooldownItemMixin.CacheChargeValues(R88KCooldownItem)
        assert(R88KCooldownItem.count == 0 and R88KCooldownItem.shown == false,
            'count=' .. tostring(R88KCooldownItem.count) .. ', shown=' .. tostring(R88KCooldownItem.shown))
        assert(select('#', C_Spell.GetSpellCastCount(19750)) == 1)
        assert(type(C_Spell.GetSpellCastCount(19750)) == 'number')
        "#,
    )
    .expect("the original CooldownViewer method must receive one numeric zero from a null context");

    env.state().borrow_mut().spell_cast_context_is_null = Some(false);
    env.exec(
        r#"
        CooldownViewerCooldownItemMixin.CacheChargeValues(R88KCooldownItem)
        assert(R88KCooldownItem.count == 7 and R88KCooldownItem.shown == true)
        "#,
    )
    .expect("the explicit non-null context reads the supplied reagent inventory");
}

#[test]
fn cast_count_uses_the_eight_reagent_minimum_at_the_api_boundary() {
    let env = WowLuaEnv::new().expect("real spell namespace");
    {
        let mut state = env.state().borrow_mut();
        state.spell_cast_context_is_null = Some(false);
        state.spell_cast_counts.insert(19_750, 99);
        state.spell_reagent_pairs.insert(
            19_750,
            vec![
                (31_001, 2),
                (31_002, 3),
                (0, 1),
                (-2, 1),
                (31_003, 0),
                (31_004, -1),
                (31_005, 2),
                (31_006, 1),
                (31_007, 1),
            ],
        );
    }
    for (slot, item_id, count) in [
        (1, 31_001, 9),
        (2, 31_002, 10),
        (3, 31_005, 20),
        (4, 31_006, 99),
        (5, 31_007, 0),
    ] {
        add_bag_stack(&env, slot, item_id, count);
    }

    let (arity, count): (i64, i64) = env
        .eval("return select('#', C_Spell.GetSpellCastCount(19750)), C_Spell.GetSpellCastCount(19750)")
        .expect("cast count must publish a number");
    assert_eq!((arity, count), (1, 3));
    let converted: (i64, i64) = env
        .eval(
            "return C_Spell.GetSpellCastCount('19750'), \
             C_Spell.GetSpellCastCount(19750.9)",
        )
        .expect("Retail spell identifiers accept numeric strings and truncate fractions");
    assert_eq!(converted, (3, 3));

    {
        let mut state = env.state().borrow_mut();
        state
            .spell_reagent_pairs
            .insert(19_750, vec![(0, 1), (-2, 1)]);
    }
    let empty: i64 = env
        .eval("return C_Spell.GetSpellCastCount(19750)")
        .expect("no eligible reagent limit returns numeric zero");
    assert_eq!(empty, 0);

    {
        let mut state = env.state().borrow_mut();
        state
            .spell_reagent_pairs
            .insert(999_999_999, vec![(31_001, 1)]);
        state.spell_cast_counts.insert(999_999_999, 11);
    }
    let missing_spell: i64 = env
        .eval("return C_Spell.GetSpellCastCount(999999999)")
        .expect("a Spell-store miss returns numeric zero");
    assert_eq!(missing_spell, 0);
}

#[test]
fn retail_spell_cast_count_accepts_numeric_strings_and_truncates_fractions() {
    let env = WowLuaEnv::new().expect("real spell namespace");
    {
        let mut state = env.state().borrow_mut();
        state.spell_cast_context_is_null = Some(false);
        state.spell_reagent_pairs.insert(19_750, vec![(31_001, 2)]);
    }
    add_bag_stack(&env, 1, 31_001, 10);
    let counts: (i64, i64) = env
        .eval(
            "return C_Spell.GetSpellCastCount('19750'), \
             C_Spell.GetSpellCastCount(19750.9)",
        )
        .expect("Retail spell identifiers accept numeric strings and truncate fractions");
    assert_eq!(counts, (5, 5));
    let displays: (String, String) = env
        .eval(
            "return C_Spell.GetSpellDisplayCount('19750'), \
             C_Spell.GetSpellDisplayCount(19750.9)",
        )
        .expect("display count uses the same Retail spell identifier conversion");
    assert_eq!(displays, ("5".to_string(), "5".to_string()));
}

#[test]
fn inferred_default_spell_context_uses_the_active_player_inventory() {
    let env = WowLuaEnv::new().expect("real spell namespace");
    {
        let mut state = env.state().borrow_mut();
        assert_eq!(state.spell_cast_context_is_null, None);
        state.spell_reagent_pairs.insert(19_750, vec![(31_001, 2)]);
    }
    add_bag_stack(&env, 1, 31_001, 10);
    let count: i64 = env
        .eval("return C_Spell.GetSpellCastCount(19750)")
        .expect("the inferred active-player context reads the current bag inventory");
    assert_eq!(count, 5);
}

fn assert_spell_flyout_text(env: &WowLuaEnv, expected: &str) {
    env.exec("R88KSpellCount.calls = 0; SpellFlyoutPopupButtonMixin.UpdateCount(R88KSpellButton)")
        .expect("the original SpellFlyout UpdateCount method must execute");
    env.exec(&format!(
        r#"
        assert(R88KSpellCount.calls == 1 and R88KSpellCount.arguments == 1,
            'calls=' .. tostring(R88KSpellCount.calls) .. ', args=' .. tostring(R88KSpellCount.arguments))
        assert(R88KSpellCount.kind == 'string' and R88KSpellCount.value == {expected:?},
            'kind=' .. tostring(R88KSpellCount.kind) .. ', value=' .. tostring(R88KSpellCount.value))
        "#
    ))
    .expect("the original consumer must receive exactly one display string");
}

#[test]
fn display_count_uses_verified_reagent_mode_before_charge_mode() {
    let env = load_selected_consumers(
        &["Blizzard_ActionBar"],
        "SpellFlyoutPopupButtonMixin.UpdateCount",
        "Blizzard_ActionBar/Shared/SpellFlyout.lua",
    );
    {
        let mut state = env.state().borrow_mut();
        state.spell_cast_counts.insert(19_750, 7);
        state.spell_cast_context_is_null = Some(false);
        state.spell_charges.insert(
            19_750,
            wow_ui_sim::c_api::charge_state::SpellChargeState {
                current_charges: 2,
                max_charges: 3,
                recharge_start: 0.0,
                recharge_duration: 0.0,
                charge_mod_rate: 1.0,
            },
        );
        state.spell_reagent_pairs.insert(19_750, vec![(-2, 1)]);
    }
    env.exec(
        r#"
        R88KSpellButton = {
            spellID = 19750,
            maxDisplayCount = 9999,
            GetName = function() return 'R88KSpell' end,
        }
        R88KSpellCount = { calls = 0 }
        function R88KSpellCount:SetText(...)
            self.calls = self.calls + 1
            self.arguments = select('#', ...)
            self.value = select(1, ...)
            self.kind = type(self.value)
        end
        "#,
    )
    .unwrap();

    // VERIFIED: reagent -2 selects display cast mode, then the cast helper excludes it.
    assert_spell_flyout_text(&env, "0");

    {
        let mut state = env.state().borrow_mut();
        state
            .spell_reagent_pairs
            .insert(19_750, vec![(0, 1), (-2, 0)]);
    }
    assert_spell_flyout_text(&env, "2");

    {
        let mut state = env.state().borrow_mut();
        state.spell_reagent_pairs.insert(
            19_750,
            vec![(0, 1); 8].into_iter().chain([(-2, 1)]).collect(),
        );
    }
    assert_spell_flyout_text(&env, "2");

    {
        let mut state = env.state().borrow_mut();
        state.spell_reagent_pairs.insert(19_750, vec![(-2, 1)]);
        state.spell_cast_context_is_null = Some(true);
    }
    assert_spell_flyout_text(&env, "0");
}

#[test]
fn visibility_info_failure_and_success_keep_distinct_return_shapes() {
    let env = load_selected_consumers(
        &["Blizzard_FrameXMLUtil"],
        "AuraUtil.ShouldDisplayDebuff",
        "Blizzard_FrameXMLUtil/AuraUtil.lua",
    );
    let (raid_in, raid_out): (i64, i64) = env
        .eval(
            "return Enum.SpellAuraVisibilityType.RaidInCombat, \
             Enum.SpellAuraVisibilityType.RaidOutOfCombat",
        )
        .expect("Retail visibility enums");

    {
        let mut state = env.state().borrow_mut();
        state.spell_cast_context_is_null = Some(true);
        state.spell_visibility_results = Some(HashMap::new());
    }
    env.exec(&format!(
        "local count = select('#', C_Spell.GetVisibilityInfo(19750, {raid_out}))\n\
         assert(count == 0, 'null-context return count=' .. tostring(count))\n\
         assert(C_Spell.GetSpellCastCount(19750) == 0)\n\
         assert(AuraUtil.ShouldDisplayDebuff('party1', 19750) == true)"
    ))
    .expect("the shared null context returns no visibility values through AuraUtil");

    env.state().borrow_mut().spell_cast_context_is_null = Some(false);
    env.state().borrow_mut().spell_visibility_results = Some(HashMap::new());
    env.exec(&format!(
        r#"
        local count = select('#', C_Spell.GetVisibilityInfo(19750, {raid_out}))
        local hasCustom, alwaysShowMine, showForMySpec =
            C_Spell.GetVisibilityInfo(19750, {raid_out})
        assert(count == 3)
        assert(type(hasCustom) == 'boolean' and hasCustom == false)
        assert(type(alwaysShowMine) == 'boolean' and alwaysShowMine == false)
        assert(type(showForMySpec) == 'boolean' and showForMySpec == false)
        assert(AuraUtil.ShouldDisplayDebuff('party1', 19750) == true)
        "#
    ))
    .unwrap();

    let mut results = HashMap::new();
    for visibility_type in [raid_in as u8, raid_out as u8] {
        results.insert((19_750, visibility_type), (true, false, true));
        results.insert((642, visibility_type), (true, true, false));
    }
    env.state().borrow_mut().spell_visibility_results = Some(results);
    env.exec(
        r#"
        assert(AuraUtil.ShouldDisplayDebuff('party1', 19750) == true)
        assert(AuraUtil.ShouldDisplayDebuff('party1', 642) == false)
        assert(AuraUtil.ShouldDisplayDebuff('player', 642) == true)
        "#,
    )
    .expect("AuraUtil must consume each of the three supplied visibility results");
}
