//! Draft only: panel LoD retirement. Optional Retail; no API or original-Lua stubs.
//! Summary veto candidate is intentionally separate/external: model lacks four
//! original default rows. No runnable failing summary-data precondition here.
#![cfg(all(feature = "optional-tests", feature = "client-retail"))]

use crate::common::blizzard_addon_harness::new_blizzard_addon_env;
use wow_ui_sim::loader::{find_toc_file, load_addon_bootstrap_from_toc};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn panel_bootstrap(addon: &str, file: &str) -> WowLuaEnv {
    let cache = wow_ui_sim::paths::default_blizzard_ui_addons_path().unwrap();
    let env = new_blizzard_addon_env(&cache);
    wow_ui_sim::xml::register_intrinsic_templates();
    // This helper loads actual selected panel dependency closures; DO NOT call
    // install_lua_harness_stubs or overwrite LoadAddOnWithErrorHandling.
    crate::common::panel_fixtures::load_panel_addons(&env);
    let path = find_toc_file(&cache.join(addon)).expect("selected Retail panel TOC");
    let toc = TocFile::from_file(&path).unwrap();
    assert!(toc.allows_screen(ScreenKind::Game));
    assert!(!toc.is_game_type_restricted());
    assert!(toc.files.iter().any(|p| p.ends_with(file)));
    println!("O next panel selected TOC {}", path.display());
    let result = load_addon_bootstrap_from_toc(&env.loader_env(), &toc).unwrap();
    assert!(
        result.warnings.is_empty(),
        "bootstrap warnings: {:?}",
        result.warnings
    );
    let state = env.state().borrow();
    let entry = state
        .addons
        .iter()
        .find(|a| a.folder_name == addon)
        .unwrap();
    assert!(entry.bootstrap_loaded, "original bootstrap not executed");
    assert!(
        !entry.loaded,
        "fixture preloaded full target addon, not LoD"
    );
    drop(state);
    // Deliberately include real post-load rewriting boundary: source ownership
    // tests must catch these replacements, not simply reload originals over them.
    env.apply_post_load_workarounds();
    for error in &env.state().borrow().lua_errors {
        eprintln!("O next panel dependency error: {error:?}");
    }
    env
}

fn original_source(env: &WowLuaEnv, expression: &str, suffix: &str) {
    env.exec(&format!(
        r#"
        local fn = {expression}
        assert(type(fn) == 'function', 'missing {expression}')
        local s = debug.getinfo(fn, 'S').source:gsub('\\', '/')
        assert(s:find('{suffix}', 1, true), '{expression}: ' .. s)
    "#
    ))
    .unwrap();
}

#[test]
fn original_journal_bootstraps_survive_postload_and_runtime_hooks() {
    for (addon, file, toggle) in [
        (
            "Blizzard_Collections",
            "Blizzard_Collections_Bootstrap.lua",
            "ToggleCollectionsJournal",
        ),
        (
            "Blizzard_EncounterJournal",
            "Blizzard_EncounterJournal_Bootstrap.lua",
            "ToggleEncounterJournal",
        ),
        (
            "Blizzard_AchievementUI",
            "Blizzard_AchievementUI_Bootstrap.lua",
            "ToggleAchievementFrame",
        ),
    ] {
        let env = panel_bootstrap(addon, file);
        original_source(&env, toggle, &format!("{addon}/{file}"));
        env.apply_runtime_addon_load_workarounds(addon);
        original_source(&env, toggle, &format!("{addon}/{file}"));
    }
}

#[test]
fn original_collections_lod_tab_switch_and_close_lifecycle() {
    let env = panel_bootstrap("Blizzard_Collections", "Blizzard_Collections_Bootstrap.lua");
    env.exec(r#"
        assert(not C_AddOns.IsAddOnLoaded('Blizzard_Collections'))
        assert(CollectionsJournal == nil, 'fixture must start before real XML')
        local events = {}
        local owner = {}
        EventRegistry:RegisterCallback('CollectionsJournal.TabSet', function(_, frame, tab)
            events[#events + 1] = {'tab', frame, tab}
        end, owner)
        EventRegistry:RegisterCallback('CollectionsJournal.OnHide', function(_, tab)
            events[#events + 1] = {'hide', tab}
        end, owner)
        local mounts, pets = COLLECTIONS_JOURNAL_TAB_INDEX_MOUNTS, COLLECTIONS_JOURNAL_TAB_INDEX_PETS
        assert(type(mounts) == 'number' and type(pets) == 'number' and mounts ~= pets)
        ToggleCollectionsJournal(mounts)
        assert(C_AddOns.IsAddOnLoaded('Blizzard_Collections'), 'toggle did not load real addon')
        assert(CollectionsJournal and CollectionsJournal:IsShown(), 'first toggle did not show real journal')
        assert(PanelTemplates_GetSelectedTab(CollectionsJournal) == mounts)
        assert(MountJournal:IsShown() and not PetJournal:IsShown())
        assert(tonumber(GetCVar('petJournalTab')) == mounts)
        ToggleCollectionsJournal(mounts)
        assert(not CollectionsJournal:IsShown(), 'same-tab toggle must close')
        local foundHide = false
        for _, e in ipairs(events) do if e[1] == 'hide' then foundHide = true end end
        assert(foundHide, 'original Collections OnHide event missing')
        ToggleCollectionsJournal(mounts)
        ToggleCollectionsJournal(pets)
        assert(CollectionsJournal:IsShown(), 'different-tab toggle must keep journal open')
        assert(PanelTemplates_GetSelectedTab(CollectionsJournal) == pets)
        assert(PetJournal:IsShown() and not MountJournal:IsShown())
        assert(tonumber(GetCVar('petJournalTab')) == pets)
        local foundTab = false
        for _, e in ipairs(events) do
            if e[1] == 'tab' and e[2] == CollectionsJournal and e[3] == pets then foundTab = true end
        end
        assert(foundTab, 'original tab event/payload missing')
        ToggleCollectionsJournal(pets)
        assert(not CollectionsJournal:IsShown())
        EventRegistry:UnregisterCallback('CollectionsJournal.TabSet', owner)
        EventRegistry:UnregisterCallback('CollectionsJournal.OnHide', owner)
    "#).expect("real Collections bootstrap, XML and tab lifecycle");
    original_source(
        &env,
        "CollectionsJournal_SetTab",
        "Blizzard_Collections/Mainline/Blizzard_Collections.lua",
    );
    env.apply_runtime_addon_load_workarounds("Blizzard_Collections");
    original_source(
        &env,
        "ToggleCollectionsJournal",
        "Blizzard_Collections/Blizzard_Collections_Bootstrap.lua",
    );
}

#[test]
fn original_achievement_lod_regular_and_statistics_lifecycle() {
    let env = panel_bootstrap(
        "Blizzard_AchievementUI",
        "Blizzard_AchievementUI_Bootstrap.lua",
    );
    env.exec(r#"
        assert(not C_AddOns.IsAddOnLoaded('Blizzard_AchievementUI'))
        assert(AchievementFrame == nil, 'fixture must start before original XML')
        assert(not C_GameRules.IsGameRuleActive(Enum.GameRule.AchievementsPanelDisabled), 'fixture requires enabled UI')
        ToggleAchievementFrame(false)
        assert(C_AddOns.IsAddOnLoaded('Blizzard_AchievementUI'))
        assert(AchievementFrame:IsShown())
        -- Original Mainline toggle215-248 selects1 regular /3 statistics.
        -- Never assign selectedTab or replace original tab handler.
        assert(AchievementFrame.selectedTab == 1)
        ToggleAchievementFrame(false)
        assert(not AchievementFrame:IsShown())
        ToggleAchievementFrame(true)
        assert(AchievementFrame:IsShown() and AchievementFrame.selectedTab == 3)
        ToggleAchievementFrame(false)
        assert(AchievementFrame:IsShown() and AchievementFrame.selectedTab == 1)
        ToggleAchievementFrame(false)
        assert(not AchievementFrame:IsShown())
    "#).expect("actual achievement toggle/tab lifecycle");
    original_source(
        &env,
        "AchievementFrame_ToggleAchievementFrame",
        "Blizzard_AchievementUI/Mainline/Blizzard_AchievementUI.lua",
    );
    original_source(
        &env,
        "AchievementFrameSummary_UpdateAchievements",
        "Blizzard_AchievementUI/Mainline/Blizzard_AchievementUI.lua",
    );
    original_source(
        &env,
        "ToggleAchievementFrame",
        "Blizzard_AchievementUI/Blizzard_AchievementUI_Bootstrap.lua",
    );
}

#[test]
fn original_encounter_journal_lod_toggle_return_and_visibility() {
    let env = panel_bootstrap(
        "Blizzard_EncounterJournal",
        "Blizzard_EncounterJournal_Bootstrap.lua",
    );
    env.exec(
        r#"
        assert(not C_AddOns.IsAddOnLoaded('Blizzard_EncounterJournal'))
        assert(EncounterJournal == nil, 'fixture must start before original XML')
        assert(ToggleEncounterJournal() == true)
        assert(C_AddOns.IsAddOnLoaded('Blizzard_EncounterJournal'))
        assert(EncounterJournal:IsShown())
        assert(ToggleEncounterJournal() == true)
        assert(not EncounterJournal:IsShown())
    "#,
    )
    .expect("original Encounter bootstrap toggle path (not microbutton eligibility proof)");
    env.apply_runtime_addon_load_workarounds("Blizzard_EncounterJournal");
    original_source(
        &env,
        "ToggleEncounterJournal",
        "Blizzard_EncounterJournal/Blizzard_EncounterJournal_Bootstrap.lua",
    );
}
