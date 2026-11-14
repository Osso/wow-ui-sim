//! Optional Retail tests for the bounded R88k native routes.
//! The LoD SpellSearch consumer is runtime-selected through SharedTalentUI's startup dependency.
#![cfg(all(
    feature = "client-retail",
    feature = "retail-12-0-5",
    feature = "optional-tests"
))]

use std::collections::HashMap;

use crate::common::blizzard_addon_harness::{blizzard_ui_dir, build_blizzard_addon_closure_env};
use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
use wow_ui_sim::loader::discover_blizzard_startup_addons_for_screen;
use wow_ui_sim::lua_api::WowLuaEnv;
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
        state.spell_cast_counts.insert(19_750, 7);
        state.spell_cast_context_is_null = Some(true);
    }

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
    .expect("a supplied completed count remains visible when the context is non-null");
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

    // Reagent -2 qualifies for display-mode selection even though the later cast helper excludes it.
    assert_spell_flyout_text(&env, "7");

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

    env.state().borrow_mut().spell_visibility_results = None;
    env.exec(&format!(
        "local count = select('#', C_Spell.GetVisibilityInfo(19750, {raid_out}))\n\
         assert(count == 0, 'null-context return count=' .. tostring(count))\n\
         assert(AuraUtil.ShouldDisplayDebuff('party1', 19750) == true)"
    ))
    .expect("the null-context failure returns no values through AuraUtil");

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
