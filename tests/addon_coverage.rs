//! Blizzard UI addon-bootstrap lane.
//!
//! Keep this file for behaviors that only exist after the relevant Blizzard
//! addons and their startup sequence have loaded.

use crate::common;
#[path = "addon_coverage/panel_open.rs"]
mod panel_open;

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::PathBuf;
use wow_ui_sim::blizzard_ui_sync::manifest_entries;
use wow_ui_sim::loader::{
    discover_all_blizzard_addons, discover_blizzard_addon_closure_for_screen,
    discover_blizzard_startup_addons_for_screen, load_addon, load_startup_addon,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_errors::grouped_errors_by_addon;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;
use wow_ui_sim::xml::{clear_templates, register_intrinsic_templates};

const PANEL_COVERAGE_ADDONS: &[(&str, &str)] = &[
    ("Blizzard_SharedXMLBase", "Blizzard_SharedXMLBase.toc"),
    ("Blizzard_Colors", "Blizzard_Colors_Mainline.toc"),
    ("Blizzard_SharedXML", "Blizzard_SharedXML_Mainline.toc"),
    (
        "Blizzard_SharedXMLGame",
        "Blizzard_SharedXMLGame_Mainline.toc",
    ),
    (
        "Blizzard_UIPanelTemplates",
        "Blizzard_UIPanelTemplates_Mainline.toc",
    ),
    (
        "Blizzard_FrameXMLBase",
        "Blizzard_FrameXMLBase_Mainline.toc",
    ),
    ("Blizzard_FrameEffects", "Blizzard_FrameEffects.toc"),
    ("Blizzard_LoadLocale", "Blizzard_LoadLocale.toc"),
    ("Blizzard_Fonts_Shared", "Blizzard_Fonts_Shared.toc"),
    ("Blizzard_HelpPlate", "Blizzard_HelpPlate.toc"),
    (
        "Blizzard_AccessibilityTemplates",
        "Blizzard_AccessibilityTemplates.toc",
    ),
    ("Blizzard_ObjectAPI", "Blizzard_ObjectAPI_Mainline.toc"),
    ("Blizzard_UIParent", "Blizzard_UIParent_Mainline.toc"),
    ("Blizzard_TextStatusBar", "Blizzard_TextStatusBar.toc"),
    ("Blizzard_MoneyFrame", "Blizzard_MoneyFrame_Mainline.toc"),
    ("Blizzard_POIButton", "Blizzard_POIButton.toc"),
    ("Blizzard_Flyout", "Blizzard_Flyout.toc"),
    ("Blizzard_StoreUI", "Blizzard_StoreUI_Mainline.toc"),
    ("Blizzard_MicroMenu", "Blizzard_MicroMenu_Mainline.toc"),
    ("Blizzard_EditMode", "Blizzard_EditMode.toc"),
    ("Blizzard_GarrisonBase", "Blizzard_GarrisonBase.toc"),
    ("Blizzard_GameTooltip", "Blizzard_GameTooltip_Mainline.toc"),
    (
        "Blizzard_UIParentPanelManager",
        "Blizzard_UIParentPanelManager_Mainline.toc",
    ),
    (
        "Blizzard_Settings_Shared",
        "Blizzard_Settings_Shared_Mainline.toc",
    ),
    (
        "Blizzard_SettingsDefinitions_Shared",
        "Blizzard_SettingsDefinitions_Shared.toc",
    ),
    (
        "Blizzard_SettingsDefinitions_Frame",
        "Blizzard_SettingsDefinitions_Frame_Mainline.toc",
    ),
    ("Blizzard_FrameXMLUtil", "Blizzard_FrameXMLUtil.toc"),
    ("Blizzard_Menu", "Blizzard_Menu.toc"),
    ("Blizzard_Minimap", "Blizzard_Minimap_Mainline.toc"),
    ("Blizzard_StaticPopup", "Blizzard_StaticPopup.toc"),
    ("Blizzard_TimeManager", "Blizzard_TimeManager_Mainline.toc"),
    ("Blizzard_ItemButton", "Blizzard_ItemButton_Mainline.toc"),
    ("Blizzard_QuickKeybind", "Blizzard_QuickKeybind.toc"),
    ("Blizzard_Collections", "Blizzard_Collections_Mainline.toc"),
    ("Blizzard_FrameXML", "Blizzard_FrameXML_Mainline.toc"),
    (
        "Blizzard_UIPanels_Game",
        "Blizzard_UIPanels_Game_Mainline.toc",
    ),
];

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn panel_coverage_roots() -> Vec<&'static str> {
    PANEL_COVERAGE_ADDONS
        .iter()
        .map(|(addon_name, _)| *addon_name)
        .collect()
}

fn format_per_addon_report(grouped_errors: &BTreeMap<String, Vec<String>>) -> String {
    let mut rows: Vec<_> = grouped_errors.iter().collect();
    rows.sort_by(|(left_name, left_errors), (right_name, right_errors)| {
        right_errors
            .len()
            .cmp(&left_errors.len())
            .then_with(|| left_name.cmp(right_name))
    });

    rows.into_iter()
        .map(|(addon_name, errors)| {
            let sample = errors.first().map(String::as_str).unwrap_or("<no sample>");
            format!("{addon_name}: {} error(s); sample: {sample}", errors.len())
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_full_per_addon_report(grouped_errors: &BTreeMap<String, Vec<String>>) -> String {
    format!(
        "Per-addon Lua error report (sorted by error count):\n{}",
        format_per_addon_report(grouped_errors)
    )
}

fn known_error_counts() -> BTreeMap<String, usize> {
    common::addon_coverage_baseline::KNOWN_ERRORS
        .iter()
        .map(|(addon_name, count)| ((*addon_name).to_string(), *count))
        .collect()
}

fn actual_error_counts(grouped_errors: &BTreeMap<String, Vec<String>>) -> BTreeMap<String, usize> {
    grouped_errors
        .iter()
        .map(|(addon_name, errors)| (addon_name.clone(), errors.len()))
        .collect()
}

fn format_error_count_map(error_counts: &BTreeMap<String, usize>) -> String {
    error_counts
        .iter()
        .map(|(addon_name, count)| format!("(\"{addon_name}\", {count})"))
        .collect::<Vec<_>>()
        .join(", ")
}

#[derive(Debug, PartialEq, Eq)]
struct ErrorCountChanges {
    increased: Vec<(String, usize, usize)>,
    decreased: Vec<(String, usize, usize)>,
}

fn classify_error_count_changes(
    known: &BTreeMap<String, usize>,
    actual: &BTreeMap<String, usize>,
) -> ErrorCountChanges {
    let mut increased = Vec::new();
    let mut decreased = Vec::new();

    for addon_name in known.keys().chain(actual.keys()).collect::<BTreeSet<_>>() {
        let known_count = known.get(addon_name).copied().unwrap_or(0);
        let actual_count = actual.get(addon_name).copied().unwrap_or(0);
        match actual_count.cmp(&known_count) {
            std::cmp::Ordering::Greater => {
                increased.push((addon_name.clone(), known_count, actual_count));
            }
            std::cmp::Ordering::Less => {
                decreased.push((addon_name.clone(), known_count, actual_count));
            }
            std::cmp::Ordering::Equal => {}
        }
    }

    ErrorCountChanges {
        increased,
        decreased,
    }
}

fn format_error_count_changes(changes: &[(String, usize, usize)]) -> String {
    changes
        .iter()
        .map(|(addon_name, old_count, new_count)| {
            format!("{addon_name}: {old_count} -> {new_count}")
        })
        .collect::<Vec<_>>()
        .join(", ")
}

#[test]
fn error_count_ratchet_detects_increases_and_decreases() {
    let known = BTreeMap::from([
        ("Blizzard_A".to_string(), 2),
        ("Blizzard_B".to_string(), 4),
        ("Blizzard_C".to_string(), 1),
    ]);
    let actual = BTreeMap::from([
        ("Blizzard_A".to_string(), 3),
        ("Blizzard_B".to_string(), 4),
        ("Blizzard_C".to_string(), 0),
    ]);

    let changes = classify_error_count_changes(&known, &actual);

    assert_eq!(changes.increased, vec![("Blizzard_A".to_string(), 2, 3)],);
    assert_eq!(changes.decreased, vec![("Blizzard_C".to_string(), 1, 0)],);
}

#[test]
fn error_count_ratchet_rejects_unlisted_addon_errors() {
    let known = BTreeMap::from([("Blizzard_Collections".to_string(), 2)]);
    let actual = BTreeMap::from([
        ("Blizzard_Collections".to_string(), 2),
        ("Blizzard_EncounterJournal".to_string(), 2),
    ]);
    let changes = classify_error_count_changes(&known, &actual);
    assert_eq!(
        changes.increased,
        vec![("Blizzard_EncounterJournal".to_string(), 0, 2)]
    );
}

#[test]
fn full_per_addon_report_lists_highest_error_counts_first() {
    let grouped_errors = BTreeMap::from([
        ("Blizzard_B".to_string(), vec!["second".to_string()]),
        (
            "Blizzard_A".to_string(),
            vec!["first".to_string(), "another".to_string()],
        ),
        ("Blizzard_C".to_string(), vec!["third".to_string()]),
    ]);

    let report = format_full_per_addon_report(&grouped_errors);
    let lines: Vec<_> = report.lines().collect();

    assert_eq!(
        lines[0],
        "Per-addon Lua error report (sorted by error count):"
    );
    assert_eq!(lines[1], "Blizzard_A: 2 error(s); sample: first");
    assert_eq!(lines[2], "Blizzard_B: 1 error(s); sample: second");
    assert_eq!(lines[3], "Blizzard_C: 1 error(s); sample: third");
}

fn cached_blizzard_addon_roots() -> BTreeSet<String> {
    std::fs::read_dir(blizzard_ui_dir())
        .expect("BlizzardUI directory should be readable")
        .map(|entry| entry.expect("BlizzardUI directory entry should be readable"))
        .filter(|entry| entry.path().is_dir())
        .map(|entry| {
            entry
                .file_name()
                .into_string()
                .expect("Blizzard addon directory name should be UTF-8")
        })
        .filter(|name| name.starts_with("Blizzard_"))
        .collect()
}

fn active_manifest_blizzard_addon_roots() -> BTreeSet<String> {
    manifest_entries()
        .map(|entry| {
            entry
                .split('/')
                .next()
                .expect("manifest entry should have an addon root")
        })
        .filter(|name| name.starts_with("Blizzard_"))
        .map(str::to_string)
        .collect()
}

fn clear_lua_error_tracking(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state.lua_errors.clear();
    state.lua_error_records.clear();
    state.lua_error_counts.clear();
}

fn silence_lua_error_handler(env: &WowLuaEnv) {
    env.exec("seterrorhandler(function() end)")
        .expect("seterrorhandler should accept a no-op test handler");
}

fn reset_template_state() {
    clear_templates();
    register_intrinsic_templates();
}

fn fire_panel_harness_startup_events(env: &WowLuaEnv) {
    common::fire_addon_loaded(env, "WoWUISim");
    for event in ["VARIABLES_LOADED", "PLAYER_LOGIN"] {
        let _ = env.fire_event(event);
    }
    common::fire_player_entering_world(env, true, false);
    for event in [
        "UPDATE_BINDINGS",
        "DISPLAY_SIZE_CHANGED",
        "UI_SCALE_CHANGED",
    ] {
        let _ = env.fire_event(event);
    }
}

fn load_panel_harness_blizzard_ui(env: &WowLuaEnv) -> HashSet<String> {
    reset_template_state();
    common::load_profile_framexml(env);
    let ui = blizzard_ui_dir();
    let roots = panel_coverage_roots();
    let closure = discover_blizzard_addon_closure_for_screen(&ui, ScreenKind::Game, &roots);
    for (addon_name, toc_path) in &closure {
        if let Err(error) = load_addon(&env.loader_env(), toc_path) {
            panic!("{addon_name} should load for the panel harness: {error}");
        }
    }

    env.apply_post_load_workarounds();
    fire_panel_harness_startup_events(env);
    silence_lua_error_handler(env);

    closure.into_iter().map(|(name, _)| name).collect()
}

fn force_load_game_addons() -> Vec<(String, PathBuf)> {
    discover_all_blizzard_addons(&blizzard_ui_dir())
        .into_iter()
        .filter(|(_, path)| {
            let toc = TocFile::from_file(path).expect("parse addon TOC");
            toc.allows_screen(ScreenKind::Game)
                && toc.allows_text_locale()
                && !toc.is_ptr_only()
                && !toc.is_game_type_restricted()
        })
        .collect()
}

#[test]
fn force_load_coverage_selects_only_game_eligible_addons() {
    let addons = force_load_game_addons();
    let selected: HashSet<_> = addons.iter().map(|(name, _)| name.as_str()).collect();
    let excluded: Vec<_> = discover_all_blizzard_addons(&blizzard_ui_dir())
        .into_iter()
        .map(|(name, _)| name)
        .filter(|name| !selected.contains(name.as_str()))
        .collect();
    println!("Excluded from Game coverage: {excluded:?}");
    assert!(addons.iter().any(|(name, _)| name == "Blizzard_ClickBindingUI"));
    for (name, path) in addons {
        let toc = TocFile::from_file(&path).expect("parse addon TOC");
        assert!(
            toc.allows_screen(ScreenKind::Game)
                && toc.allows_text_locale()
                && !toc.is_ptr_only()
                && !toc.is_game_type_restricted(),
            "{name} is not eligible for the Game screen"
        );
    }
}

fn load_force_load_startup(env: &WowLuaEnv) {
    for addon in discover_blizzard_startup_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game) {
        load_startup_addon(&env.loader_env(), &addon.toc_path, addon.kind, None)
            .unwrap_or_else(|error| panic!("{} startup load failed: {error}", addon.name));
    }
}

#[cfg(feature = "client-retail")]
#[test]
fn force_load_click_binding_runs_macro_bootstrap_first() {
    let env = WowLuaEnv::new().expect("create Lua environment");
    env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];
    reset_template_state();
    common::load_profile_framexml(&env);
    load_force_load_startup(&env);
    for (name, toc) in force_load_game_addons() {
        load_addon(&env.loader_env(), &toc).expect("force-load Blizzard addon");
        if name == "Blizzard_ClickBindingUI" {
            let errors = env.state().borrow().lua_errors.clone();
            assert!(
                !errors.iter().any(|error| error.contains("Blizzard_ClickBindingUI.lua:594")),
                "Click Binding OnLoad must not fail at the macro bootstrap call: {errors:?}"
            );
            let present: bool = env.eval("return type(MacroFrame_LoadUI) == 'function' and ClickBindingFrame ~= nil").unwrap();
            assert!(present, "the macro bootstrap and Click Binding frame must exist");
            return;
        }
    }
    panic!("Click Binding addon missing from force-load discovery");
}

#[cfg(feature = "client-retail")]
#[test]
fn loot_journal_displays_empty_runeforge_catalog_for_distinct_filters() {
    let env = WowLuaEnv::new().expect("create Lua environment");
    env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];
    reset_template_state();
    common::load_profile_framexml(&env);
    load_force_load_startup(&env);
    let (_, toc) = force_load_game_addons()
        .into_iter()
        .find(|(name, _)| name == "Blizzard_EncounterJournal")
        .expect("Encounter Journal is eligible for the Game screen");
    load_addon(&env.loader_env(), &toc).expect("load Encounter Journal");
    let ok: bool = env.eval(r#"
        local journal = setmetatable({classFilter=1, specFilter=71,
            runeforgePowerFilter=Enum.RuneforgePowerFilter.All,
            IsShown=function() return false end}, {__index=LootJournalMixin})
        journal:UpdatePowers()
        local first = journal.powers
        journal.classFilter, journal.specFilter = 2, 65
        journal.runeforgePowerFilter = Enum.RuneforgePowerFilter.Relevant
        journal:UpdatePowers()
        return type(first) == 'table' and next(first) == nil
            and type(journal.powers) == 'table' and next(journal.powers) == nil
            and first ~= journal.powers
    "#).expect("Loot Journal updates both runeforge filters");
    assert!(ok);
}

#[cfg(feature = "client-retail")]
#[test]
fn wardrobe_latest_appearance_tracks_collections() {
    let env = WowLuaEnv::new().expect("create Lua environment");
    env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];
    reset_template_state();
    common::load_profile_framexml(&env);
    load_force_load_startup(&env);
    let (_, toc) = force_load_game_addons()
        .into_iter()
        .find(|(name, _)| name == "Blizzard_Collections")
        .expect("Collections is eligible for the Game screen");
    load_addon(&env.loader_env(), &toc).expect("load Collections");
    let ok: bool = env.eval(r#"
        local wardrobe = setmetatable({}, {__index=WardrobeItemsCollectionMixin})
        wardrobe:CheckLatestAppearance(false)
        if wardrobe.latestAppearanceID ~= nil then return false end
        A_Admin.AddTransmogAppearance(9901, 1, 60001)
        local first = C_TransmogCollection.GetSourceInfo(9901)
        wardrobe:CheckLatestAppearance(false)
        if wardrobe.latestAppearanceID ~= first.visualID
            or wardrobe.jumpToLatestCategoryID ~= 1 then return false end
        A_Admin.AddTransmogAppearance(9902, 4, 60002)
        local second = C_TransmogCollection.GetSourceInfo(9902)
        wardrobe:CheckLatestAppearance(false)
        return first.visualID ~= second.visualID
            and wardrobe.latestAppearanceID == second.visualID
            and wardrobe.jumpToLatestAppearanceID == second.visualID
            and wardrobe.jumpToLatestCategoryID == 4
    "#).expect("Wardrobe updates the latest collected appearance");
    assert!(ok);
}

#[cfg(feature = "client-retail")]
#[test]
fn map_click_to_zoom_uses_best_guess_help_text_position() {
    let env = WowLuaEnv::new().expect("create Lua environment");
    env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];
    reset_template_state();
    common::load_profile_framexml(&env);
    load_force_load_startup(&env);
    let (_, toc) = force_load_game_addons()
        .into_iter()
        .find(|(name, _)| name == "Blizzard_SharedMapDataProviders")
        .expect("map data providers are eligible for the Game screen");
    load_addon(&env.loader_env(), &toc).expect("load map data providers");
    let ok: bool = env.eval(r#"
        local mapID = 84
        local provider = setmetatable({GetMap=function()
            return {GetMapID=function() return mapID end}
        end}, {__index=ClickToZoomDataProviderMixin})
        local first = provider:GetClickToZoomStyle()
        mapID = 2022
        local second = provider:GetClickToZoomStyle()
        mapID = nil
        return first == Enum.MapCanvasPosition.BottomRight
            and second == Enum.MapCanvasPosition.BottomRight
            and provider:GetClickToZoomStyle() == first
    "#).expect("map consumer selects the help-text position");
    assert!(ok);
    assert!(env.exec("C_Map.GetMapArtHelpTextPosition('invalid')").is_err());
}

#[cfg(feature = "client-retail")]
#[test]
fn commentator_camera_presets_accept_two_numeric_speeds() {
    let env = WowLuaEnv::new().expect("create Lua environment");
    env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];
    reset_template_state();
    common::load_profile_framexml(&env);
    load_force_load_startup(&env);
    let (_, toc) = force_load_game_addons()
        .into_iter()
        .find(|(name, _)| name == "Blizzard_Commentator")
        .expect("Commentator is eligible for the Game screen");
    load_addon(&env.loader_env(), &toc).expect("load Commentator");
    env.exec("SetFollowCameraTransitionPreset(1); SetFollowCameraTransitionPreset(2)")
        .expect("Commentator applies both camera presets");
    assert!(env.exec("C_Commentator.SetFollowCameraSpeeds('bad', 20)").is_err());
    assert!(env.exec("C_Commentator.SetFollowCameraSpeeds(0.05, 'bad')").is_err());
}

#[cfg(feature = "client-retail")]
#[test]
fn garrison_mission_frames_use_db2_currency_types() {
    let env = WowLuaEnv::new().expect("create Lua environment");
    env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];
    reset_template_state();
    common::load_profile_framexml(&env);
    load_force_load_startup(&env);
    clear_lua_error_tracking(&env);
    for (name, toc) in force_load_game_addons() {
        load_addon(&env.loader_env(), &toc).expect("force-load Blizzard addon");
        if name == "Blizzard_GarrisonUI" {
            let errors = env.state().borrow().lua_errors.clone();
            assert!(
                !errors.iter().any(|error| error.contains("Blizzard_OrderHallMissionUI.lua:36")),
                "mission frames must resolve garrison currency types: {errors:?}"
            );
            let ok: bool = env.eval(r#"
                local order = OrderHallMissionFrame.MissionTab.MissionPage.CostFrame
                local bfa = BFAMissionFrame.MissionTab.MissionPage.CostFrame
                local orderInfo = C_CurrencyInfo.GetCurrencyInfo(1220)
                local bfaInfo = C_CurrencyInfo.GetCurrencyInfo(1560)
                local garrisonPrimary, garrisonSecondary = C_Garrison.GetCurrencyTypes(2)
                local covenantPrimary, covenantSecondary = C_Garrison.GetCurrencyTypes(111)
                local _, orderSecondary = C_Garrison.GetCurrencyTypes(3)
                return garrisonPrimary == 824 and garrisonSecondary == 1101
                    and covenantPrimary == 1810 and covenantSecondary == 1813
                    and orderSecondary == 0
                    and order.currency == 1220 and bfa.currency == 1560
                    and order.CostIcon:GetTexture() == orderInfo.iconFileID
                    and bfa.CostIcon:GetTexture() == bfaInfo.iconFileID
                    and orderInfo.quantity == 0 and bfaInfo.quantity == 0
            "#).expect("mission frames expose currency and icon state");
            assert!(ok);
            return;
        }
    }
    panic!("GarrisonUI missing from Game coverage");
}

#[cfg(feature = "client-retail")]
#[test]
fn garrison_consumers_clear_and_read_empty_mission_lists() {
    let env = WowLuaEnv::new().expect("create Lua environment");
    env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];
    reset_template_state();
    common::load_profile_framexml(&env);
    load_force_load_startup(&env);
    for (name, toc) in force_load_game_addons() {
        load_addon(&env.loader_env(), &toc).expect("force-load Blizzard addon");
        if name == "Blizzard_GarrisonUI" { break; }
    }
    let (_, toc) = force_load_game_addons().into_iter()
        .find(|(name, _)| name == "Blizzard_AdventureMap")
        .expect("Adventure Map is eligible for the Game screen");
    load_addon(&env.loader_env(), &toc).expect("load Adventure Map");
    let ok: bool = env.eval(r#"
        local texture = {SetDesaturated=function() end}
        local tab1 = {SetText=function(self, value) self.text=value end}
        local tab2 = {SetText=function(self, value) self.text=value end,
            SetEnabled=function(self, value) self.enabled=value end,
            Left=texture, Right=texture, Middle=texture,
            Text={SetTextColor=function() end}}
        local list = setmetatable({inProgressMissions={123}, availableMissions={456},
            Tab1=tab1, Tab2=tab2,
            GetMissionFrame=function() return {followerTypeID=1} end,
            UpdateCombatAllyMission=function() end, Update=function() end},
            {__index=GarrisonMissionListMixin})
        list:UpdateMissions()
        local provider = setmetatable({GetMap=function()
            return {GetMapID=function() return 84 end}
        end}, {__index=AdventureMap_ZoneSummaryProviderMixin})
        provider:GatherMissions()
        return next(list.inProgressMissions) == nil and next(list.availableMissions) == nil
            and tab2.enabled == false and next(provider.missionsByZone) == nil
    "#).expect("original mission list and Adventure Map consumers run");
    assert!(ok);
    assert!(env.exec("C_Garrison.GetInProgressMissions({}, 'bad')").is_err());
    assert!(env.exec("C_Garrison.GetAvailableMissions(true, 1)").is_err());
}

#[cfg(feature = "client-retail")]
fn load_garrison_original_consumers() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create Lua environment");
    env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];
    reset_template_state();
    common::load_profile_framexml(&env);
    load_force_load_startup(&env);
    for (name, toc) in force_load_game_addons() {
        load_addon(&env.loader_env(), &toc).expect("force-load Blizzard addon");
        if name == "Blizzard_GarrisonUI" { return env; }
    }
    panic!("GarrisonUI missing from Game coverage");
}

#[cfg(feature = "client-retail")]
#[test]
fn shipyard_consumer_reads_empty_in_progress_missions() {
    let env = load_garrison_original_consumers();
    let ok: bool = env.eval(r#"
        -- Test-only fixture for the missing GetAllBonusAbilityEffects API.
        C_Garrison.GetAllBonusAbilityEffects = function() return {} end
        local list = {missions={123}, missionFrames={}, bonusFrames={},
            CompleteDialog={IsShown=function() return false end}}
        GarrisonShipyardFrame = {MissionTab={MissionList=list}}
        GarrisonShipyardMap_UpdateMissions()
        return next(list.missions) == nil and next(list.bonusEffects) == nil
    "#).expect("original Shipyard mission consumer runs");
    assert!(ok);
}

#[cfg(feature = "client-retail")]
#[test]
fn landing_page_consumer_reads_empty_available_missions() {
    let env = load_garrison_original_consumers();
    let ok: bool = env.eval(r#"
        -- Test-only fixture for the missing GetLandingPageItems API.
        C_Garrison.GetLandingPageItems = function() return {} end
        GarrisonLandingPage = {garrTypeID=3}
        local function text() return {SetFormattedText=function() end, SetText=function() end} end
        local list = {EmptyMissionText=text(), ScrollBox={SetDataProvider=function(self, provider)
            self.size=provider:GetSize()
        end}}
        local available = {Text=text()}
        GarrisonLandingPageReport = {List=list, InProgress={Text=text()}, Available=available,
            selectedTab=available, GetParent=function() return GarrisonLandingPage end}
        GarrisonLandingPageReportList_UpdateItems()
        return next(list.AvailableItems) == nil and list.ScrollBox.size == 0
    "#).expect("original Landing Page mission consumer runs");
    assert!(ok);
}

#[cfg(feature = "client-retail")]
#[test]
fn order_hall_hides_missing_combat_ally_and_uses_full_height() {
    let env = load_garrison_original_consumers();
    let ok: bool = env.eval(r#"
        local ally = setmetatable({Hide=function(self) self.hidden=true end},
            {__index=OrderHallCombatAllyMixin})
        local list = setmetatable({combatAllyMission={stale=true}, CombatAllyUI=ally,
            GetMissionFrame=function() return {followerTypeID=1} end,
            SetHeight=function(self, height) self.height=height end},
            {__index=OrderHallMissionListMixin})
        list:UpdateCombatAllyMission()
        return list.height == 565 and list.combatAllyMission == nil
            and ally.missionInfo == nil and ally.hidden == true
    "#).expect("Order Hall updates its combat ally layout");
    assert!(ok);
    assert!(env.exec("C_Garrison.GetCombatAllyMission('bad')").is_err());
}

#[cfg(feature = "client-retail")]
#[test]
fn pvp_queue_rated_button_reflects_eligibility() {
    for (eligible, reason) in [(false, "Rated unavailable"), (true, "")] {
        let env = WowLuaEnv::new().expect("create Lua environment");
        {
            let mut state = env.state().borrow_mut();
            state.addon_base_paths = vec![blizzard_ui_dir()];
            state.pvp_catalog.rated_eligible = eligible;
            state.pvp_catalog.rated_failure_reason = reason.to_string();
        }
        reset_template_state();
        common::load_profile_framexml(&env);
        load_force_load_startup(&env);
        clear_lua_error_tracking(&env);
        for (name, toc) in force_load_game_addons() {
            load_addon(&env.loader_env(), &toc).expect("force-load Blizzard addon");
            if name == "Blizzard_PVPUI" { break; }
        }
        let errors = env.state().borrow().lua_errors.clone();
        assert!(
            !errors.iter().any(|error| error.contains("Blizzard_PVPUI.lua:359")),
            "rated PVP query must be available during OnLoad: {errors:?}"
        );
        let (enabled, tooltip): (bool, Option<String>) = env.eval(
            "return PVPQueueFrame.CategoryButton2:IsEnabled(), PVPQueueFrame.CategoryButton2.tooltip"
        ).expect("read rated button state");
        assert_eq!(enabled, eligible);
        assert_eq!(tooltip.as_deref(), (!eligible).then_some(reason));
    }
}

#[cfg(feature = "client-retail")]
#[test]
fn garrison_cinematic_models_expose_only_cinematic_camera_methods() {
    let env = load_garrison_original_consumers();
    let ok: bool = env.eval(r#"
        local cluster = CreateFrame('Frame', nil, UIParent, 'GarrisonMissionCompleteModelCluster')
        if #cluster.Model == 0 then return false end
        cluster:SetFacingLeft(true)
        local cinematic = CreateFrame('CinematicModel')
        local player = CreateFrame('PlayerModel')
        local dressup = CreateFrame('DressUpModel')
        local plain = CreateFrame('Frame')
        if type(cinematic.SetFacingLeft) ~= 'function'
            or type(cinematic.SetTargetDistance) ~= 'function' then return false end
        for _, frame in ipairs({player, dressup, plain}) do
            if frame.SetFacingLeft ~= nil or frame.SetTargetDistance ~= nil then return false end
        end
        cinematic:SetTargetDistance(0)
        _G.CinematicMethodProbe = cinematic
        return true
    "#).expect("Garrison CinematicModel consumer executes");
    assert!(ok);
    assert!(env.exec("CinematicMethodProbe:SetFacingLeft('bad')").is_err());
    assert!(env.exec("CinematicMethodProbe:SetTargetDistance('bad')").is_err());
}

#[test]
fn all_blizzard_addon_load_errors_are_tracked_per_addon_name() {
    common::with_perf_lock(|| {
        common::with_timeout(600, move || {
            let env = WowLuaEnv::new().expect("Failed to create Lua environment");
            env.set_screen_size(1024.0, 768.0);
            env.state().borrow_mut().addon_base_paths = vec![blizzard_ui_dir()];
            reset_template_state();
            if let Some(toc) = wow_ui_sim::client_profile::blizzard_ui_framexml_toc() {
                load_addon(&env.loader_env(), &toc)
                    .expect("profile FrameXML should load before addons");
            }

            load_force_load_startup(&env);

            let cached_addon_roots = cached_blizzard_addon_roots();
            let manifest_addon_roots = active_manifest_blizzard_addon_roots();
            let cache_only: Vec<_> = cached_addon_roots
                .difference(&manifest_addon_roots)
                .cloned()
                .collect();
            let manifest_only: Vec<_> = manifest_addon_roots
                .difference(&cached_addon_roots)
                .cloned()
                .collect();
            assert_eq!(
                cached_addon_roots, manifest_addon_roots,
                "active Blizzard UI cache roots must match committed manifest roots\ncache only: {cache_only:?}\nmanifest only: {manifest_only:?}"
            );

            let addons = force_load_game_addons();
            let known_addons: HashSet<_> = addons.iter().map(|(name, _)| name.clone()).collect();
            let mut load_failures = Vec::new();

            for (name, toc_path) in &addons {
                if let Err(error) = load_addon(&env.loader_env(), toc_path) {
                    load_failures.push(format!("{name}: {error}"));
                }
            }

            assert!(
                load_failures.is_empty(),
                "force-loading all Blizzard addons should not have hard TOC load failures:\n{}",
                load_failures.join("\n"),
            );

            let state = env.state().borrow();
            let grouped_errors = grouped_errors_by_addon(&state);
            println!("{}", format_full_per_addon_report(&grouped_errors));
            eprintln!("Complete force-load Lua errors: {grouped_errors:#?}");
            let known_counts = known_error_counts();
            let actual_counts = actual_error_counts(&grouped_errors);
            let changes = classify_error_count_changes(&known_counts, &actual_counts);
            let unknown_count = grouped_errors.get("<unknown>").map_or(0, Vec::len);
            let invalid_addons: Vec<_> = grouped_errors
                .keys()
                .filter(|addon_name| {
                    addon_name.as_str() != "<unknown>" && !known_addons.contains(*addon_name)
                })
                .cloned()
                .collect();

            assert!(
                unknown_count == 0,
                "full Blizzard load should attribute Lua errors to addon names, not <unknown>.\n{}",
                format_per_addon_report(&grouped_errors),
            );
            assert!(
                invalid_addons.is_empty(),
                "full Blizzard load attributed Lua errors to names outside the {} loadable Blizzard addons: {:?}\n{}",
                known_addons.len(),
                invalid_addons,
                format_per_addon_report(&grouped_errors),
            );
            assert!(
                changes.increased.is_empty(),
                "full Blizzard load increased per-addon Lua errors.\nincreased: [{}]\nactual counts: [{}]\n{}",
                format_error_count_changes(&changes.increased),
                format_error_count_map(&actual_counts),
                format_per_addon_report(&grouped_errors),
            );
        })
    })
}
