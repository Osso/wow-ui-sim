//! Optional Retail tests for the bounded R88k native routes.
//! Ranged auto-attack is API-level VERIFIED; no runtime-selected consumer exists because
//! the original SpellSearch TOC is LoadOnDemand. Other tests invoke selected Retail consumers.
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

fn load_selected_consumer(root: &str, function: &str, source_suffix: &str) -> WowLuaEnv {
    assert_eq!(ACTIVE, ClientProfile::Retail);
    let ui_dir = blizzard_ui_dir();
    let startup_addons = discover_blizzard_startup_addons_for_screen(&ui_dir, ScreenKind::Game);
    assert!(
        startup_addons.iter().any(|addon| addon.name == root),
        "{root} must be selected by Retail Game-screen addon discovery"
    );

    let (env, loaded) = build_blizzard_addon_closure_env(&ui_dir, &[root], &[]);
    assert!(loaded.iter().any(|name| name == root));
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
fn ranged_auto_attack_api_compares_the_shared_identifier() {
    // API-level VERIFIED; no runtime-selected consumer: Blizzard_SpellSearch.toc is LoD.
    let ui_dir = blizzard_ui_dir();
    let toc_path = ui_dir.join("Blizzard_SpellSearch/Blizzard_SpellSearch.toc");
    let toc = TocFile::from_file(&toc_path).expect("Retail SpellSearch TOC");
    assert!(toc.is_load_on_demand());
    let startup_addons = discover_blizzard_startup_addons_for_screen(&ui_dir, ScreenKind::Game);
    assert!(
        !startup_addons
            .iter()
            .any(|addon| addon.name == "Blizzard_SpellSearch")
    );

    let env = WowLuaEnv::new().expect("Retail C_Spell surface");
    env.state().borrow_mut().ranged_auto_attack_spell_id = 31_337;
    env.exec(
        r#"
        local matching = C_Spell.IsRangedAutoAttackSpell(31337)
        local different = C_Spell.IsRangedAutoAttackSpell(31338)
        assert(select('#', C_Spell.IsRangedAutoAttackSpell(31337)) == 1)
        assert(type(matching) == 'boolean' and matching)
        assert(type(different) == 'boolean' and not different)
        "#,
    )
    .expect("the query compares against the explicitly supplied shared identifier");
}

#[test]
fn cast_count_null_context_returns_zero_through_cooldown_viewer() {
    let env = load_selected_consumer(
        "Blizzard_CooldownViewer",
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
        assert(R88KCooldownItem.count == 0 and R88KCooldownItem.shown == false)
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
        "assert(R88KSpellCount.calls == 1 and R88KSpellCount.arguments == 1)\n\
         assert(R88KSpellCount.kind == 'string' and R88KSpellCount.value == {expected:?})"
    ))
    .expect("the original consumer must receive exactly one display string");
}

#[test]
fn display_count_uses_verified_reagent_mode_before_charge_mode() {
    let env = load_selected_consumer(
        "Blizzard_ActionBar",
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
    let env = load_selected_consumer(
        "Blizzard_FrameXMLUtil",
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
        "assert(select('#', C_Spell.GetVisibilityInfo(19750, {raid_out})) == 0)\n\
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
