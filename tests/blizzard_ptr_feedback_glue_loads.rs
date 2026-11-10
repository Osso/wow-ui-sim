use std::path::PathBuf;

use wow_ui_sim::loader::discover_blizzard_addons_for_screen;
use wow_ui_sim::loader::{discover_all_blizzard_addons, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::settle_headless_startup;
use wow_ui_sim::toc::TocFile;

#[path = "common/toc_layout.rs"]
mod toc_layout;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn ptr_feedback_glue_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_PTRFeedbackGlue")
}

fn ptr_feedback_glue_toc() -> PathBuf {
    ptr_feedback_glue_dir().join("Blizzard_PTRFeedbackGlue.toc")
}

const PTR_FEEDBACK_GLUE_TOC_FILES: &[&str] = &[
    "../Blizzard_PTRFeedback/Blizzard_PTRFeedback.lua",
    "../Blizzard_PTRFeedback/Blizzard_PTRFeedback_Frames.lua",
    "../Blizzard_PTRFeedback/Blizzard_Reports.lua",
    "Blizzard_Reports_Glue.lua",
    "Blizzard_PTRFeedback_Events_Glue.lua",
];

const SAVED_VARIABLES: &[&str] = &["Blizzard_PTRIssueReporter_Saved"];

// Literal cached inputs absent from Wrath manifest1-382/cache; behavior cases stay enabled.
retail_ptr_forever_toc_fixture! {
#[test]
fn blizzard_ptr_feedback_glue_find_toc_resolves_bare_variant() {
    let resolved =
        find_toc_file(&ptr_feedback_glue_dir()).expect("Blizzard_PTRFeedbackGlue TOC resolves");
    assert_eq!(
        resolved,
        ptr_feedback_glue_toc(),
        "Blizzard_PTRFeedbackGlue ships exactly one bare TOC — no `_Mainline.toc` variant"
    );

    let mainline = ptr_feedback_glue_dir().join("Blizzard_PTRFeedbackGlue_Mainline.toc");
    assert!(
        !mainline.exists(),
        "There must be NO `_Mainline.toc` at {}",
        mainline.display()
    );
}
}

retail_ptr_forever_toc_fixture! {
#[test]
fn blizzard_ptr_feedback_glue_toc_marks_addon_as_ptr_and_beta_only() {
    let toc =
        TocFile::from_file(&ptr_feedback_glue_toc()).expect("Blizzard_PTRFeedbackGlue TOC parses");

    assert!(
        toc.metadata.get("OnlyBetaAndPTR").map(String::as_str) == Some("1"),
        "TOC must declare `## OnlyBetaAndPTR: 1`. R26 establishes that this gate checks \
         client build classification, not realms; the same directive on the Glue addon \
         excludes normal builds and admits PTR/beta builds"
    );
    assert_eq!(
        toc.is_ptr_only(),
        wow_ui_sim::client_profile::ACTIVE != wow_ui_sim::client_profile::ClientProfile::Ptr,
        "R26: OnlyBetaAndPTR is a build-classification gate. PTR passes it; Forever 70707 \
         and Retail 70688 are normal builds and remove the addon. Unified TOC layout does \
         not affect `is_ptr_only()`"
    );
}
}

retail_ptr_forever_toc_fixture! {
#[test]
fn blizzard_ptr_feedback_glue_toc_pins_glue_only_screen_gate() {
    let toc =
        TocFile::from_file(&ptr_feedback_glue_toc()).expect("Blizzard_PTRFeedbackGlue TOC parses");

    assert!(
        toc.is_glue_only(),
        "TOC must declare `## AllowLoad: Glue` — `is_glue_only()` at src/toc.rs:276-280 \
         flips true. This is the inverse gate of the Game-side Blizzard_PTRFeedback \
         (which has no AllowLoad and defaults to Game-only). The Glue TOC is the \
         pre-login counterpart that wires up bug-reporting on the character select / \
         character create / login screens"
    );

    assert!(
        !toc.allows_screen(ScreenKind::Game),
        "`## AllowLoad: Glue` must EXCLUDE Game — `allows_screen(Game)` returns false \
         per src/toc.rs:309. The Game-side Blizzard_PTRFeedback handles in-game \
         reporting; this glue addon hands off pre-login surveys"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            toc.allows_screen(screen),
            "`## AllowLoad: Glue` must INCLUDE {screen:?} — `allows_screen` at \
             src/toc.rs:309 routes glue gates through `screen.is_glue()` which returns \
             true for all 3 glue screens"
        );
    }
}
}

retail_ptr_forever_toc_fixture! {
#[test]
fn blizzard_ptr_feedback_glue_toc_declares_eager_no_lod_no_deps() {
    let toc =
        TocFile::from_file(&ptr_feedback_glue_toc()).expect("Blizzard_PTRFeedbackGlue TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "TOC must NOT declare `## LoadOnDemand:` — eager-loaded so the bug-report \
         tooling is wired up the moment a PTR/beta tester reaches the character \
         select screen"
    );
    assert!(!toc.is_load_first());
    assert!(
        !toc.is_secure_env(),
        "TOC must NOT declare `## UseSecureEnvironment:` — insecure addon"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "TOC has no `## AllowLoadGameType:` directive — the gate is OnlyBetaAndPTR + \
         AllowLoad: Glue, not AllowLoadGameType"
    );

    assert!(
        toc.dependencies().is_empty(),
        "TOC must declare ZERO `## Dependencies:` — unlike the Game-side variant \
         (which depends on Blizzard_HelpPlate for tutorial overlays), the glue-screen \
         counterpart has no upstream addon to wait for. HelpPlate is not available \
         on glue screens"
    );

    assert!(
        toc.optional_deps().is_empty(),
        "Zero `## OptionalDeps:` declared"
    );

    assert!(
        toc.load_with().is_empty(),
        "Zero `## LoadWith:` declared — no reverse-dep trigger; this addon is its \
         own root in the eager-load graph (gated only by screen + PTR)"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_ptr_feedback_glue_shares_saved_variable_with_game_variant() {
    let toc =
        TocFile::from_file(&ptr_feedback_glue_toc()).expect("Blizzard_PTRFeedbackGlue TOC parses");

    let saved_vars = toc.saved_variables();
    let saved: Vec<&str> = saved_vars.iter().map(|s| s.as_str()).collect();
    assert_eq!(
        saved, SAVED_VARIABLES,
        "TOC must declare exactly 1 account-wide `## SavedVariables: \
         Blizzard_PTRIssueReporter_Saved` — the SAME global name as the Game-side \
         Blizzard_PTRFeedback. Sharing the saved-variable global lets a tester's \
         suppressed-locations list flow across the glue/game session boundary: a \
         bug suppressed on the character select screen stays suppressed once the \
         tester logs into the world"
    );
    assert!(
        toc.saved_variables_per_character().is_empty(),
        "TOC must declare zero `## SavedVariablesPerCharacter:` — bug-report \
         suppressions are account-wide (and the glue screens have no character \
         scope to begin with)"
    );
}
}

retail_ptr_forever_toc_fixture! {
#[test]
fn blizzard_ptr_feedback_glue_toc_declares_metadata_in_raw_bytes() {
    let raw = std::fs::read_to_string(ptr_feedback_glue_toc())
        .expect("Blizzard_PTRFeedbackGlue TOC reads utf-8");

    assert!(
        raw.contains("## Title: PTR Bug Reporter Glue"),
        "TOC must declare `## Title: PTR Bug Reporter Glue` — note this differs from \
         the Game-side variant's `PTR Issue Reporter`. The Glue title says `Bug \
         Reporter` (player-facing terminology) where the Game variant says `Issue \
         Reporter` (matches the PTR_IssueReporter global table name). The two TOCs \
         were authored at different times, hence the inconsistency"
    );
    assert!(
        raw.contains("## OnlyBetaAndPTR: 1"),
        "TOC must declare `## OnlyBetaAndPTR: 1` exactly"
    );
    assert!(
        raw.contains("## AllowLoad: Glue"),
        "TOC must declare `## AllowLoad: Glue` exactly"
    );
    assert!(
        raw.contains("## Notes: Tool for gathering bugs from PTR"),
        "TOC must declare `## Notes:` — note again the wording differs from the \
         Game-side variant (`Tool for gathering issues from PTR`). The Glue Notes \
         line uses `bugs`, the Game one uses `issues`"
    );
    assert!(
        raw.contains("## SavedVariables: Blizzard_PTRIssueReporter_Saved"),
        "TOC must declare `## SavedVariables:` with the historical Game-side global \
         name — shared persistence"
    );

    assert!(
        !raw.contains("## Author"),
        "TOC must NOT declare `## Author:` — matches the Game-side variant's \
         missing author line"
    );
    assert!(
        !raw.contains("## LoadOnDemand"),
        "TOC must NOT declare `## LoadOnDemand:` — eager-loaded"
    );
    assert!(
        !raw.contains("## Dependencies"),
        "TOC must NOT declare `## Dependencies:` — no upstream addon (HelpPlate \
         is not available on glue screens)"
    );
    assert!(
        !raw.contains("## UseSecureEnvironment"),
        "TOC must NOT declare `## UseSecureEnvironment:`"
    );
    assert!(
        !raw.contains("## DefaultState"),
        "TOC must NOT declare `## DefaultState:`"
    );
    assert!(
        !raw.contains("## AllowLoadGameType"),
        "TOC must NOT declare `## AllowLoadGameType:` — gate is screen+PTR, not \
         game type"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_ptr_feedback_glue_lists_five_files_with_cross_addon_parent_paths() {
    let toc =
        TocFile::from_file(&ptr_feedback_glue_toc()).expect("Blizzard_PTRFeedbackGlue TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, PTR_FEEDBACK_GLUE_TOC_FILES,
        "TOC must list exactly 5 files in canonical order (paths normalized to \
         forward slashes by src/toc.rs:147 — raw bytes use `\\`). The first 3 use \
         `../Blizzard_PTRFeedback/` parent-directory paths to import sibling \
         Lua files from the Game-side addon directory: \
         Blizzard_PTRFeedback.lua (declares the PTR_IssueReporter root frame + \
         .Data/.Assets/.DataCollectorTypes namespaces), \
         Blizzard_PTRFeedback_Frames.lua (the survey-frame UI logic), \
         Blizzard_Reports.lua (the report-survey definitions). The last 2 are \
         local Glue-suffixed files: Blizzard_Reports_Glue.lua (glue-specific \
         survey definitions for character creation / customization), \
         Blizzard_PTRFeedback_Events_Glue.lua (glue-screen event hooks for \
         GameMenuFrame / character-customization frames). \
         `file_paths()` at src/toc.rs:345 resolves each entry via \
         `resolve_path_case_insensitive(&self.addon_dir, f)` which composes \
         `addon_dir.join(...)` allowing `..` to walk out of the Glue directory \
         into the sibling PTRFeedback directory. No Bindings.xml on the glue \
         side — keybindings are not used at the character-select screen"
    );
}
}

#[test]
fn blizzard_ptr_feedback_glue_appears_only_on_ptr_glue_discovery() {
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
            .any(|(name, _)| name == "Blizzard_PTRFeedbackGlue");
        let is_glue_screen = screen != ScreenKind::Game;
        let expected = is_glue_screen
            && wow_ui_sim::client_profile::ACTIVE == wow_ui_sim::client_profile::ClientProfile::Ptr;
        assert_eq!(
            found, expected,
            "R26: PTR build classification passes OnlyBetaAndPTR on Glue screens; \
             normal builds including Forever are filtered, and AllowLoad: Glue excludes \
             the Game screen"
        );
    }
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_ptr_feedback_glue_appears_in_full_addon_inventory() {
    let inventory = discover_all_blizzard_addons(&blizzard_ui_dir());
    let found = inventory
        .iter()
        .any(|(name, _)| name == "Blizzard_PTRFeedbackGlue");
    assert!(
        found,
        "Blizzard_PTRFeedbackGlue MUST appear in `discover_all_blizzard_addons` \
         even though it is PTR-only and glue-only — the full-inventory accessor at \
         src/loader/mod.rs:309-343 does NOT apply the `is_ptr_only()` or screen \
         filters, so tests can still load the addon explicitly via `load_addon` \
         for coverage"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_ptr_feedback_glue_loads_explicitly_on_glue_screen() {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::CharacterCreate);

    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::CharacterCreate);
    for (name, toc_path) in &addons {
        if let Err(err) = load_addon(&env.loader_env(), toc_path) {
            panic!("[load {name}] FAILED: {err}");
        }
    }
    env.apply_post_load_workarounds();
    settle_headless_startup(&env);

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    load_addon(&env.loader_env(), &ptr_feedback_glue_toc())
        .expect("Blizzard_PTRFeedbackGlue loads cleanly on top of the CharacterCreate glue stack");

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_PTRFeedbackGlue")
                || message.contains("Blizzard_PTRFeedback")
                || message.contains("PTR_IssueReporter")
                || message.contains("Blizzard_PTRIssueReporter_Saved")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_PTRFeedbackGlue emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );

    let kind: String = env
        .eval("return type(_G.PTR_IssueReporter)")
        .expect("PTR_IssueReporter type probe succeeds");
    assert_eq!(
        kind, "table",
        "_G.PTR_IssueReporter must publish as a frame table — even though the Glue \
         TOC has no `## Dependencies: Blizzard_PTRFeedback`, it imports the \
         sibling addon's Lua files directly via `..\\Blizzard_PTRFeedback\\` paths. \
         The first imported file (Blizzard_PTRFeedback.lua) declares the root \
         frame at file scope, so loading just the Glue TOC is sufficient to \
         publish the entire PTR_IssueReporter namespace on a fresh env"
    );
}
}

// The Mists/Era/Anniversary caches ship 218 addons each and Wrath ships 23;
// none contains Blizzard_PTRFeedbackGlue, so the pinned TOC facts above describe no file the
// classic loaders can read.
classic_cache_absent_toc_fixture! {
#[test]
fn blizzard_ptrfeedbackglue_toc_is_absent_from_classic_caches() {
    let dir = ptr_feedback_glue_dir();
    assert!(
        !dir.is_dir(),
        "classic profile caches must not contain Blizzard_PTRFeedbackGlue"
    );
    assert!(wow_ui_sim::loader::find_toc_file(&dir).is_none());
}
}
