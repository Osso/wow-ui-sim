use std::path::PathBuf;

use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::loader::{discover_all_blizzard_addons, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

#[path = "common/toc_layout.rs"]
mod toc_layout;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn ptr_feedback_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_PTRFeedback")
}

fn ptr_feedback_toc() -> PathBuf {
    ptr_feedback_dir().join("Blizzard_PTRFeedback.toc")
}

const PTR_FEEDBACK_TOC_FILES: &[&str] = &[
    "Blizzard_PTRFeedback_FrameTemplates.xml",
    "Blizzard_PTRFeedback.lua",
    "Blizzard_PTRFeedback_Tooltips.lua",
    "Blizzard_PTRFeedback_Frames.lua",
    "Blizzard_PTRFeedback_Events.lua",
    "Blizzard_Reports.lua",
];

// The unified PTR TOC also lists the Gamepad plugin Lua.
const UNIFIED_PTR_FEEDBACK_TOC_FILES: &[&str] = &[
    "Blizzard_PTRFeedback_FrameTemplates.xml",
    "Blizzard_PTRFeedback.lua",
    "Blizzard_PTRFeedback_Tooltips.lua",
    "Blizzard_PTRFeedback_Frames.lua",
    "Blizzard_PTRFeedback_Events.lua",
    "Blizzard_PTRFeedback_Gamepad.lua",
    "Blizzard_Reports.lua",
];

// The classic TOC lists the six legacy Lua files without the FrameTemplates
// XML prelude.
const CLASSIC_PTR_FEEDBACK_TOC_FILES: &[&str] = &[
    "Blizzard_PTRFeedback.lua",
    "Blizzard_PTRFeedback_Tooltips.lua",
    "Blizzard_PTRFeedback_Frames.lua",
    "Blizzard_PTRFeedback_Events.lua",
    "Blizzard_PTRFeedback_Constants.lua",
    "Blizzard_Reports.lua",
];

fn ptr_feedback_toc_files() -> &'static [&'static str] {
    if toc_layout::classic_cache() {
        CLASSIC_PTR_FEEDBACK_TOC_FILES
    } else if toc_layout::unified_toc_tree() {
        UNIFIED_PTR_FEEDBACK_TOC_FILES
    } else {
        PTR_FEEDBACK_TOC_FILES
    }
}

const REQUIRED_DEPS: &[&str] = &["Blizzard_HelpPlate"];

const SAVED_VARIABLES: &[&str] = &["Blizzard_PTRIssueReporter_Saved"];

fn load_ptr_feedback_with_deps(env: &WowLuaEnv) {
    let helpplate_toc = blizzard_ui_dir()
        .join("Blizzard_HelpPlate")
        .join("Blizzard_HelpPlate.toc");
    load_addon(&env.loader_env(), &helpplate_toc).expect("Blizzard_HelpPlate loads cleanly");
    load_addon(&env.loader_env(), &ptr_feedback_toc()).expect("Blizzard_PTRFeedback loads cleanly");
}

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

// Literal cached inputs absent from Wrath manifest1-382/cache; behavior cases stay enabled.
post_wrath_ui_fixture! {
#[test]
fn blizzard_ptr_feedback_find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&ptr_feedback_dir()).expect("Blizzard_PTRFeedback TOC resolves");
    assert_eq!(
        resolved,
        ptr_feedback_toc(),
        "Blizzard_PTRFeedback ships exactly one bare TOC — no `_Mainline.toc` variant"
    );

    let mainline = ptr_feedback_dir().join("Blizzard_PTRFeedback_Mainline.toc");
    assert!(
        !mainline.exists(),
        "There must be NO `_Mainline.toc` at {}",
        mainline.display()
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_ptr_feedback_toc_marks_addon_as_ptr_and_beta_only() {
    let toc = TocFile::from_file(&ptr_feedback_toc()).expect("Blizzard_PTRFeedback TOC parses");

    assert!(
        toc.metadata.get("OnlyBetaAndPTR").map(String::as_str) == Some("1"),
        "TOC must declare `## OnlyBetaAndPTR: 1`. The PTR Issue Reporter is a developer tool \
         that ships with the live client tree but is gated to PTR / beta realms only"
    );
    assert_eq!(
        toc.is_ptr_only(),
        !toc_layout::unified_toc_tree(),
        "`is_ptr_only()` gates the addon out of eager discovery on profiles that do not \
         permit PTR-only addons; the PTR and Forever profiles admit them, so the gate is \
         inactive there (src/toc/mod.rs permits_ptr_only)"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_ptr_feedback_toc_declares_eager_no_lod_with_one_dep() {
    let toc = TocFile::from_file(&ptr_feedback_toc()).expect("Blizzard_PTRFeedback TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "TOC must NOT declare `## LoadOnDemand:` — when running on a PTR/beta client \
         the issue reporter loads eagerly so the F6 bug-button overlay is wired up \
         from session start"
    );
    assert!(!toc.is_load_first());
    assert!(
        !toc.is_secure_env(),
        "TOC must NOT declare `## UseSecureEnvironment:` — insecure addon"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "TOC has no `## AllowLoadGameType:` directive — `is_game_type_restricted()` \
         returns false; the PTR/beta gate is OnlyBetaAndPTR, not AllowLoadGameType"
    );

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "TOC has no `## AllowLoad:` directive — `allows_screen` defaults to Game-only"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Default Game-only screen gate must EXCLUDE {screen:?}"
        );
    }

    let dependencies = toc.dependencies();
    let deps: Vec<&str> = dependencies.iter().map(|s| s.as_str()).collect();
    let expected_deps: &[&str] = if toc_layout::classic_cache() {
        &[]
    } else {
        REQUIRED_DEPS
    };
    assert_eq!(
        deps, expected_deps,
        "TOC must declare exactly 1 dep: Blizzard_HelpPlate on Retail/PTR/Forever; the \
         classic TOC declares none"
    );

    assert!(
        toc.optional_deps().is_empty(),
        "Zero `## OptionalDeps:` declared"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_ptr_feedback_toc_declares_one_account_wide_saved_variable() {
    let toc = TocFile::from_file(&ptr_feedback_toc()).expect("Blizzard_PTRFeedback TOC parses");

    let saved_vars = toc.saved_variables();
    let saved: Vec<&str> = saved_vars.iter().map(|s| s.as_str()).collect();
    assert_eq!(
        saved, SAVED_VARIABLES,
        "TOC must declare exactly 1 account-wide `## SavedVariables: \
         Blizzard_PTRIssueReporter_Saved` — persists the account-wide list of \
         suppressed bug-report locations across PTR sessions so testers don't \
         re-report the same issue"
    );
    assert!(
        toc.saved_variables_per_character().is_empty(),
        "TOC must declare zero `## SavedVariablesPerCharacter:` — bug-report \
         suppressions are account-wide"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_ptr_feedback_toc_declares_metadata_in_raw_bytes() {
    let raw =
        std::fs::read_to_string(ptr_feedback_toc()).expect("Blizzard_PTRFeedback TOC reads utf-8");

    let (expected_title, expected_notes) = if toc_layout::classic_cache() {
        ("## Title: PTR Bug Reporter", "## Notes: Tool for gathering bugs from PTR")
    } else {
        (
            "## Title: PTR Issue Reporter",
            "## Notes: Tool for gathering issues from PTR",
        )
    };
    assert!(
        raw.contains(expected_title),
        "TOC must declare the pinned player-facing reporter title"
    );
    assert!(
        raw.contains("## OnlyBetaAndPTR: 1"),
        "TOC must declare `## OnlyBetaAndPTR: 1` exactly"
    );
    assert!(
        raw.contains(expected_notes),
        "TOC must declare `## Notes:` — one of the few Blizzard_* addons that \
         carries a Notes line. The Notes field is consumed by the AddOn list UI \
         to render a tooltip describing the addon's purpose"
    );
    assert!(
        raw.contains("## SavedVariables: Blizzard_PTRIssueReporter_Saved"),
        "TOC must declare `## SavedVariables:` with the historical \
         `Blizzard_PTRIssueReporter_Saved` global name"
    );
    if !toc_layout::classic_cache() {
        assert!(
            raw.contains("## Dependencies: Blizzard_HelpPlate"),
            "TOC must declare `## Dependencies: Blizzard_HelpPlate`"
        );
    }

    assert!(
        !raw.contains("## Author"),
        "TOC must NOT declare `## Author:` — one of the few Blizzard_* addons \
         WITHOUT the canonical author line"
    );
    assert!(
        !raw.contains("## LoadOnDemand"),
        "TOC must NOT declare `## LoadOnDemand:` — eager-loaded on PTR/beta"
    );
    assert!(
        !raw.contains("## AllowLoad"),
        "TOC must NOT declare `## AllowLoad:` or `## AllowLoadGameType:`"
    );
    assert!(
        !raw.contains("## UseSecureEnvironment"),
        "TOC must NOT declare `## UseSecureEnvironment:`"
    );
    assert!(
        !raw.contains("## DefaultState"),
        "TOC must NOT declare `## DefaultState:`"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_ptr_feedback_toc_lists_six_files_in_canonical_order() {
    let toc = TocFile::from_file(&ptr_feedback_toc()).expect("Blizzard_PTRFeedback TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed,
        ptr_feedback_toc_files(),
        "TOC must list the FrameTemplates XML, the root Lua, then the plugin \
         Lua files in dependency order — Tooltips → Frames → Events → Gamepad (unified PTR \
         TOC only) → Reports. Note that Bindings.xml exists in the addon directory but is \
         NOT listed in the TOC body — it is auto-loaded by the keybinding subsystem via the \
         special-case filename lookup, not via the TOC file enumeration. The Bindings.xml \
         registers the F6 default keybinding for `PTR_IssueReporter.HandleTooltipKeypress()`"
    );
}
}

#[test]
fn blizzard_ptr_feedback_does_not_appear_in_eager_discovery() {
    let ui = blizzard_ui_dir();

    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_PTRFeedback");
        let expected = toc_layout::unified_toc_tree() && screen == ScreenKind::Game;
        assert_eq!(
            found, expected,
            "PTR/Forever Blizzard_PTRFeedback.toc:2,4-14 permits the PTR-only Game \
             addon; its implicit screen gate excludes glue. Other profile caches either \
             omit the addon or reject its OnlyBetaAndPTR declaration. Screen: {screen:?}"
        );
    }
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_ptr_feedback_appears_in_full_addon_inventory() {
    let inventory = discover_all_blizzard_addons(&blizzard_ui_dir());
    let found = inventory
        .iter()
        .any(|(name, _)| name == "Blizzard_PTRFeedback");
    assert!(
        found,
        "Blizzard_PTRFeedback MUST appear in `discover_all_blizzard_addons` even \
         though it is PTR-only — the full-inventory accessor at \
         src/loader/mod.rs:309-343 does NOT apply the `is_ptr_only()` filter, so \
         tests can still load the addon explicitly via `load_addon` for coverage. \
         This is the canonical asymmetry: eager discovery hides PTR addons but \
         full inventory exposes them"
    );
}
}

prefork_full_ui_case! {
fn blizzard_ptr_feedback_loads_explicitly_after_dependencies(env: &WowLuaEnv) {

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    load_ptr_feedback_with_deps(&env);

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_PTRFeedback")
                || message.contains("PTR_IssueReporter")
                || message.contains("UIPanelBugButton")
                || message.contains("Blizzard_PTRIssueReporter_Saved")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_PTRFeedback emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_ptr_feedback_publishes_issue_reporter_root_frame(env: &WowLuaEnv) {
    load_ptr_feedback_with_deps(&env);

    let kind: String = env
        .eval("return type(_G.PTR_IssueReporter)")
        .expect("PTR_IssueReporter type probe succeeds");
    assert_eq!(
        kind, "table",
        "_G.PTR_IssueReporter must publish as a frame table — Blizzard_PTRFeedback.lua \
         line 1 calls `PTR_IssueReporter = CreateFrame('Frame', nil, \
         GetAppropriateTopLevelParent())` at file scope, anchoring the issue-reporter \
         to the appropriate top-level parent (UIParent on retail). The frame holds \
         3 namespace tables: PTR_IssueReporter.Data (the rolling state — current/ \
         previous map IDs, registered survey lists, default keybind 'F6', \
         suppressed locations, frame-component pool), PTR_IssueReporter.Assets \
         (the texture path table — InfoIcon, BugReportIcon, BackgroundTexture \
         etc.), and PTR_IssueReporter.DataCollectorTypes (the survey-data type \
         taxonomy)"
    );

    let has_namespace_tables: (String, String, String) = env
        .eval(
            "return type(PTR_IssueReporter.Data), \
                    type(PTR_IssueReporter.Assets), \
                    type(PTR_IssueReporter.DataCollectorTypes)",
        )
        .expect("PTR_IssueReporter namespace tables probe succeeds");
    assert_eq!(
        has_namespace_tables,
        (
            "table".to_string(),
            "table".to_string(),
            "table".to_string()
        ),
        "PTR_IssueReporter.Data / .Assets / .DataCollectorTypes must all publish as \
         tables — pinned at lines 2 / 38 / (later) of Blizzard_PTRFeedback.lua"
    );

    let default_keybind: String = env
        .eval("return PTR_IssueReporter.Data.DefaultKeybind")
        .expect("DefaultKeybind probe succeeds");
    assert_eq!(
        default_keybind, "F6",
        "PTR_IssueReporter.Data.DefaultKeybind pins to 'F6' — matches the \
         Bindings.xml `default='F6'` declaration"
    );
}
}

prefork_full_ui_case! {
fn blizzard_ptr_feedback_virtual_template_not_in_global_env(env: &WowLuaEnv) {
    load_ptr_feedback_with_deps(&env);

    let kind: String = env
        .eval("return type(_G.UIPanelBugButton)")
        .expect("UIPanelBugButton type probe succeeds");
    assert_eq!(
        kind, "nil",
        "_G.UIPanelBugButton must be nil — the addon ships exactly 1 virtual \
         template (UIPanelBugButton — a Button with frameLevel=510 that the \
         issue-reporter instantiates per zone-bug context). Virtual templates \
         live in the template registry, NOT in the global environment"
    );
}
}
