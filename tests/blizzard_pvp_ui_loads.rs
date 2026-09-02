use std::path::PathBuf;

use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::loader::{discover_all_blizzard_addons, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn pvp_ui_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_PVPUI")
}

fn pvp_ui_mainline_toc() -> PathBuf {
    pvp_ui_dir().join("Blizzard_PVPUI_Mainline.toc")
}

fn pvp_ui_mists_toc() -> PathBuf {
    pvp_ui_dir().join("Blizzard_PVPUI_Mists.toc")
}

const MAINLINE_TOC_FILES: &[&str] = &[
    "Mainline/Blizzard_PVPUI_Bootstrap.lua",
    "Mainline/Blizzard_PVPUI.lua",
    "Mainline/Blizzard_PVPUI.xml",
    "Localization.lua",
];

const MISTS_TOC_FILES: &[&str] = &[
    "Mists/Blizzard_PVPUI_Bootstrap.lua",
    "Mists/Blizzard_PVPUI.lua",
    "Mists/Blizzard_PVPUI.xml",
    "Localization.lua",
];

const MAINLINE_DEPS: &[&str] = &[
    "Blizzard_HelpPlate",
    "Blizzard_GroupFinder",
    "Blizzard_MatchmakingQueueDisplay",
    "Blizzard_FrameXMLUtil",
];

const MISTS_DEPS: &[&str] = &["Blizzard_HelpPlate", "Blizzard_GroupFinder"];

const PUBLIC_MIXIN_GLOBALS: &[&str] = &[
    "PVPCasualActivityButtonMixin",
    "PVPSpecialEventButtonMixin",
    "PVPSpecialEventLabelMixin",
    "PVPStandardRewardMixin",
    "PVPUIHonorInsetMixin",
    "PVPUIHonorLevelDisplayMixin",
    "PVPAchievementRewardMixin",
    "PVPConquestBarMixin",
    "NewPvpSeasonMixin",
    "PVPWeeklyCasualPanelMixin",
    "PVPWeeklyRatedPanelMixin",
    "PlunderstormQueueFrameMixin",
    "StartPlunderstormQueueButtonMixin",
    "PlunderstormPanelMixin",
    "PVPQuestRewardMixin",
    "PVPTalentPrestigeLevelDialogCloseButtonMixin",
    "PVPRewardRoleShortageBonusMixin",
    "TrainingGroundsFrameMixin",
    "BonusTrainingGroundListMixin",
    "TrainingGroundActivityButtonMixin",
    "SpecificTrainingGroundListMixin",
    "PVPSpecificTrainingGroundButtonMixin",
];

const VIRTUAL_TEMPLATES_SAMPLE: &[&str] = &[
    "SeasonRewardFrameTemplate",
    "PVPSeasonChangesNoticeTemplate",
    "PVPInstanceListHeaderButtonTemplate",
    "PVPInstanceListEntryButtonTemplate",
    "PVPWarGameButtonTemplate",
    "PVPSpecificBattlegroundButtonTemplate",
    "PVPSpecificTrainingGroundButtonTemplate",
    "PVPBonusBattlegroundContentsTemplate",
    "PVPRewardTemplate",
    "PVPStandardRewardTemplate",
    "PVPQuestRewardTemplate",
    "PVPAchievementRewardTemplate",
    "PVPCurrencyDisplayTemplate",
    "PVPCurrencyRewardTemplate",
    "PVPQueueFrameButtonTemplate",
    "PVPCasualActivityButton",
    "PVPCasualStandardButtonTemplate",
    "PVPCasualSpecialEventButtonTemplate",
    "PVPTrainingGroundActivityButtonTemplate",
    "PVPRatedActivityButtonTemplate",
];

const PUBLIC_NAMED_FRAMES: &[&str] = &[
    "PVPUIFrame",
    "PVPQueueFrame",
    "ConquestFrame",
    "ConquestTooltip",
    "PvPObjectiveBannerFrame",
    "HonorFrame",
];

fn load_full_game_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);

    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }

    wow_ui_sim::xml::register_intrinsic_templates();

    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    for (name, toc_path) in &addons {
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);

    env
}

#[test]
fn blizzard_pvp_ui_find_toc_resolves_mainline_variant() {
    let resolved = find_toc_file(&pvp_ui_dir()).expect("Blizzard_PVPUI TOC resolves");
    assert_eq!(
        resolved,
        pvp_ui_mainline_toc(),
        "Blizzard_PVPUI ships Mainline + Mists variants and NO bare TOC. \
         `find_toc_file` at src/loader/mod.rs:65-95 prefers the `_Mainline.toc` \
         suffix first, so the resolver returns the Mainline variant — confirmed \
         by the explicit suffix-priority list `[_Mainline.toc, .toc]` in \
         `toc_variants`. The Mists variant exists for the Mists of Pandaria \
         classic flavor and is excluded by the `_Mists` skip-list in the \
         fallback walker at src/loader/mod.rs:87"
    );

    let bare = pvp_ui_dir().join("Blizzard_PVPUI.toc");
    assert!(
        !bare.exists(),
        "There must be NO bare `Blizzard_PVPUI.toc` — variant TOCs only"
    );
}

#[test]
fn blizzard_pvp_ui_mainline_toc_pins_lod_game_only_mainline_only() {
    let toc =
        TocFile::from_file(&pvp_ui_mainline_toc()).expect("Blizzard_PVPUI_Mainline TOC parses");

    assert!(
        toc.is_load_on_demand(),
        "Mainline TOC must declare `## LoadOnDemand: 1` — the PVP UI is loaded \
         lazily when the player opens the PvP tab inside the PVEFrame via \
         `UIParentLoadAddOn(\"Blizzard_PVPUI\")` from the GroupFinder dispatch"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(!toc.is_ptr_only());

    assert!(
        !toc.is_game_type_restricted(),
        "Mainline variant: `## AllowLoadGameType: mainline` — \
         `is_game_type_restricted()` returns FALSE because `mainline` is in the \
         cross-flavor allowlist at src/toc.rs:299. This is the canonical retail \
         variant"
    );

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "`## AllowLoad: game` (lowercase) — `allows_screen` at src/toc.rs:308 \
         routes via `eq_ignore_ascii_case(\"game\")` so the lowercase form \
         resolves the same as `## AllowLoad: Game`"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Game-only screen gate must EXCLUDE {screen:?} — PVP UI only \
             matters in-world"
        );
    }
}

#[test]
fn blizzard_pvp_ui_mists_toc_pins_classic_flavor_restriction() {
    let toc = TocFile::from_file(&pvp_ui_mists_toc()).expect("Blizzard_PVPUI_Mists TOC parses");

    assert!(
        toc.is_load_on_demand(),
        "Mists TOC must also declare `## LoadOnDemand: 1`"
    );

    assert!(
        toc.is_game_type_restricted(),
        "Mists variant: `## AllowLoadGameType: mists` — \
         `is_game_type_restricted()` returns TRUE because `mists` is NOT in the \
         `mainline | standard` cross-flavor allowlist at src/toc.rs:299. The \
         loader filter at src/loader/mod.rs:527 rejects this variant on retail. \
         It only loads on the Mists of Pandaria classic client where the game \
         type matches"
    );

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "Mists also has `## AllowLoad: game`"
    );
}

#[test]
fn blizzard_pvp_ui_mainline_declares_four_dependencies() {
    let toc =
        TocFile::from_file(&pvp_ui_mainline_toc()).expect("Blizzard_PVPUI_Mainline TOC parses");

    let dependencies = toc.dependencies();
    let deps: Vec<&str> = dependencies.iter().map(|s| s.as_str()).collect();
    assert_eq!(
        deps, MAINLINE_DEPS,
        "Retail 12.1.0.69497 Mainline TOC declares HelpPlate, GroupFinder, \
         MatchmakingQueueDisplay, and FrameXMLUtil in published order"
    );

    assert_eq!(
        toc.optional_deps(),
        vec!["Blizzard_Plunderstorm".to_string()]
    );
    assert!(toc.load_with().is_empty());
}

#[test]
fn blizzard_pvp_ui_mists_declares_two_dependencies_dropping_queue_display() {
    let toc = TocFile::from_file(&pvp_ui_mists_toc()).expect("Blizzard_PVPUI_Mists TOC parses");

    let dependencies = toc.dependencies();
    let deps: Vec<&str> = dependencies.iter().map(|s| s.as_str()).collect();
    assert_eq!(
        deps, MISTS_DEPS,
        "Mists TOC must declare exactly 2 hard deps: Blizzard_HelpPlate + \
         Blizzard_GroupFinder. The Mists variant DROPS Blizzard_MatchmakingQueueDisplay \
         (the modern queue-display widget that ships only on retail). This is \
         the canonical example of a cross-flavor variant TOC trimming \
         retail-only deps to fit a classic client"
    );
}

#[test]
fn blizzard_pvp_ui_toc_declares_no_saved_variables() {
    for toc_path in [pvp_ui_mainline_toc(), pvp_ui_mists_toc()] {
        let toc = TocFile::from_file(&toc_path).expect("PVPUI TOC parses");

        assert!(
            toc.saved_variables().is_empty(),
            "TOC must declare zero `## SavedVariables:` — pure stateless UI; \
             every PvP state pulls from the live C_PvP / GetPVPLifetimeStats / \
             RequestRatedInfo queries"
        );
        assert!(toc.saved_variables_per_character().is_empty());
    }
}

#[test]
fn blizzard_pvp_ui_mainline_toc_declares_metadata_in_raw_bytes() {
    let raw = std::fs::read_to_string(pvp_ui_mainline_toc()).expect("Mainline TOC reads utf-8");

    assert!(raw.contains("## Title: Blizzard PVP UI"));
    assert!(raw.contains("## Author: Blizzard Entertainment"));
    assert!(raw.contains("## Version: 1.0"));
    assert!(raw.contains("## LoadOnDemand: 1"));
    assert!(raw.contains("## AllowLoad: game"));
    assert!(raw.contains(
        "## Dependencies: Blizzard_HelpPlate, Blizzard_GroupFinder, Blizzard_MatchmakingQueueDisplay, Blizzard_FrameXMLUtil"
    ));
    assert!(raw.contains("## AllowLoadGameType: mainline"));

    assert!(
        !raw.contains("## OnlyBetaAndPTR"),
        "TOC must NOT declare `## OnlyBetaAndPTR:` — ships on live"
    );
    assert!(
        !raw.contains("## SavedVariables"),
        "TOC must NOT declare any SavedVariables directive"
    );
    assert!(
        raw.contains("## OptionalDeps: Blizzard_Plunderstorm"),
        "Retail 12.1.0.69497 declares Blizzard_Plunderstorm as an optional dependency"
    );
}

#[test]
fn blizzard_pvp_ui_mainline_lists_bootstrap_then_three_files_with_variant_subdirectory() {
    let toc = TocFile::from_file(&pvp_ui_mainline_toc()).expect("Mainline TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, MAINLINE_TOC_FILES,
        "Retail 12.1.0.69497 Mainline TOC lists its bootstrap before the Lua, XML, \
         and shared localization files"
    );
}

#[test]
fn blizzard_pvp_ui_mists_lists_bootstrap_then_three_files_with_mists_subdirectory() {
    let toc = TocFile::from_file(&pvp_ui_mists_toc()).expect("Mists TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, MISTS_TOC_FILES,
        "Retail 12.1.0.69497 Mists TOC lists its bootstrap before the flavor files \
         and shared localization"
    );
}

#[test]
fn blizzard_pvp_ui_does_not_appear_in_eager_discovery() {
    let ui = blizzard_ui_dir();

    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let found = addons.iter().any(|(name, _)| name == "Blizzard_PVPUI");
        assert!(
            !found,
            "Blizzard_PVPUI must NOT appear in eager discovery for {screen:?} — \
             `## LoadOnDemand: 1` flips `is_load_on_demand()` true and the \
             loader filters LOD addons out of the eager pool. The addon loads \
             only when a consumer (the GroupFinder PvP tab) calls \
             UIParentLoadAddOn explicitly"
        );
    }
}

#[test]
fn blizzard_pvp_ui_appears_in_full_addon_inventory() {
    let inventory = discover_all_blizzard_addons(&blizzard_ui_dir());
    let found = inventory.iter().any(|(name, _)| name == "Blizzard_PVPUI");
    assert!(
        found,
        "Blizzard_PVPUI MUST appear in `discover_all_blizzard_addons`"
    );
}

prefork_full_ui_case! {
fn blizzard_pvp_ui_loads_explicitly_after_eager_game_sweep(env: &WowLuaEnv) {

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    load_addon(&env.loader_env(), &pvp_ui_mainline_toc())
        .expect("Blizzard_PVPUI loads cleanly on top of the eager Game-screen stack");

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_PVPUI")
                || message.contains("PVPUIFrame")
                || message.contains("PVPQueueFrame")
                || message.contains("ConquestFrame")
                || message.contains("HonorFrame")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_PVPUI emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_pvp_ui_publishes_twenty_two_mixin_globals(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &pvp_ui_mainline_toc()).expect("Blizzard_PVPUI loads cleanly");

    for mixin in PUBLIC_MIXIN_GLOBALS {
        let kind: String = env
            .eval(&format!("return type(_G[{mixin:?}])"))
            .unwrap_or_else(|err| panic!("type probe for {mixin} failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{mixin} must publish as a table — Blizzard_PVPUI ships exactly \
             22 public Mixin globals across the Mainline Blizzard_PVPUI.lua \
             file: button mixins (PVPCasualActivityButtonMixin / \
             PVPSpecialEventButtonMixin / TrainingGroundActivityButtonMixin / \
             PVPSpecificTrainingGroundButtonMixin), reward mixins \
             (PVPStandardRewardMixin / PVPQuestRewardMixin / \
             PVPAchievementRewardMixin / PVPRewardRoleShortageBonusMixin), \
             panel mixins (PVPWeeklyCasualPanelMixin / PVPWeeklyRatedPanelMixin / \
             TrainingGroundsFrameMixin / PlunderstormPanelMixin), display \
             mixins (PVPUIHonorInsetMixin / PVPUIHonorLevelDisplayMixin / \
             PVPConquestBarMixin), the season-change notice mixin \
             (NewPvpSeasonMixin), the special-event new-feature label mixin \
             (PVPSpecialEventLabelMixin), the talent-prestige dialog close \
             button mixin, and the Plunderstorm queue button + frame mixins"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_pvp_ui_publishes_named_top_level_frames(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &pvp_ui_mainline_toc()).expect("Blizzard_PVPUI loads cleanly");

    for frame_name in PUBLIC_NAMED_FRAMES {
        let kind: String = env
            .eval(&format!("return type(_G[{frame_name:?}])"))
            .unwrap_or_else(|err| panic!("type probe for {frame_name} failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{frame_name} must publish as a frame — PVPUIFrame is the root \
             container parented to PVEFrame (NOT UIParent — the addon attaches \
             to the unified PvE/PvP tabbed panel from Blizzard_GroupFinder), \
             PVPQueueFrame is the inner container for casual/rated/training/ \
             plunderstorm sub-panels, ConquestFrame is the toplevel rated \
             tab content, ConquestTooltip is the TOOLTIP-strata tooltip \
             parented to UIParent, PvPObjectiveBannerFrame is the in-world \
             objective-capture banner parented to UIParent, HonorFrame is the \
             casual-tab content panel"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_pvp_ui_virtual_templates_not_in_global_env(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &pvp_ui_mainline_toc()).expect("Blizzard_PVPUI loads cleanly");

    for template in VIRTUAL_TEMPLATES_SAMPLE {
        let kind: String = env
            .eval(&format!("return type(_G[{template:?}])"))
            .unwrap_or_else(|err| panic!("type probe for {template} failed: {err}"));
        assert_eq!(
            kind, "nil",
            "_G.{template} must be nil — virtual templates live in the \
             template registry, NOT the global environment. Sampled here is \
             a representative subset of the ~30+ virtual templates Mainline \
             Blizzard_PVPUI.xml ships covering reward / instance-list / \
             casual-button / rated-button / currency-display families"
        );
    }
}
}
