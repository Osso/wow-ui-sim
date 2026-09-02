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

fn professions_book_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_ProfessionsBook")
}

fn professions_book_toc() -> PathBuf {
    professions_book_dir().join("Blizzard_ProfessionsBook.toc")
}

const PROFESSIONS_BOOK_TOC_FILES: &[&str] = &[
    "Blizzard_ProfessionsBook_Bootstrap.lua",
    "Blizzard_ProfessionsBook.lua",
    "Blizzard_ProfessionsBook.xml",
];

const REQUIRED_DEPS: &[&str] = &["Blizzard_HelpPlate"];

const PUBLIC_MIXIN_GLOBALS: &[&str] = &[
    "ProfessionSpellButtonMixin",
    "ProfessionsUnlearnButtonMixin",
];

const PUBLIC_GLOBAL_FUNCTIONS: &[&str] = &[
    "ProfessionsBookFrame_OnLoad",
    "ProfessionsBookFrame_OnEvent",
    "ProfessionsBookFrame_OnShow",
    "ProfessionsBookFrame_OnHide",
    "ProfessionsBookFrame_Update",
    "ProfessionsBookFrame_PlayOpenSound",
    "ProfessionsBookFrame_PlayCloseSound",
    "ProfessionsBook_GetSpellBookItemSlot",
    "ProfessionsBook_ToggleTutorial",
    "FormatProfession",
];

const PUBLIC_GLOBAL_TABLES: &[&str] = &["PROFESSION_RANKS", "ProfessionsFrame_HelpPlate"];

const NAMED_NON_VIRTUAL_TOP_LEVEL_FRAMES: &[&str] = &[
    "ProfessionsBookFrame",
    "ProfessionsContentFrame",
    "PrimaryProfession1",
    "PrimaryProfession2",
    "SecondaryProfession1",
    "SecondaryProfession2",
    "SecondaryProfession3",
    "ProfessionsBookPage1",
    "ProfessionsBookPage2",
];

const VIRTUAL_TEMPLATES_NOT_IN_GLOBALS: &[&str] = &[
    "ProfessionButtonTemplate",
    "ProfessionTrialCapTemplate",
    "ProfessionStatusBarTemplate",
    "PrimaryProfessionTemplate",
    "SecondaryProfessionTemplate",
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
fn blizzard_professions_book_find_toc_resolves_bare_variant() {
    let resolved =
        find_toc_file(&professions_book_dir()).expect("Blizzard_ProfessionsBook TOC resolves");
    assert_eq!(
        resolved,
        professions_book_toc(),
        "Blizzard_ProfessionsBook ships a single bare TOC — no `_Mainline.toc` variant. \
         The classic spellbook-style profession panel is universal across retail flavors \
         (vs. the heavier Blizzard_Professions which carries `## AllowLoadGameType: \
         mainline`)"
    );

    let mainline = professions_book_dir().join("Blizzard_ProfessionsBook_Mainline.toc");
    assert!(
        !mainline.exists(),
        "There must be NO `_Mainline.toc` at {} — the addon is unflavored",
        mainline.display()
    );
}

#[test]
fn blizzard_professions_book_toc_declares_lod_game_only_addon() {
    let toc =
        TocFile::from_file(&professions_book_toc()).expect("Blizzard_ProfessionsBook TOC parses");

    assert!(
        toc.is_load_on_demand(),
        "TOC must declare `## LoadOnDemand: 1` — the spellbook-style profession panel is \
         loaded explicitly via the ProfessionMicroButton OnClick path that calls \
         ToggleProfessionsBook → UIParentLoadAddOn('Blizzard_ProfessionsBook')"
    );
    assert!(!toc.is_load_first());
    assert!(
        !toc.is_secure_env(),
        "TOC must NOT declare `## UseSecureEnvironment:` — non-protected display panel"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "TOC has no `## AllowLoadGameType:` directive at all — `is_game_type_restricted` \
         at src/toc.rs:294-302 returns FALSE when the metadata key is missing entirely \
         (cross-flavor addon)"
    );

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "TOC has no `## AllowLoad:` directive — `allows_screen` at src/toc.rs:306-313 \
         defaults to Game-only when the key is absent. The professions spellbook only \
         makes sense in-world where the player has a real spellbook + skill-line state"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Default Game-only screen gate must EXCLUDE {screen:?} — no spellbook on \
             glue screens"
        );
    }
}

#[test]
fn blizzard_professions_book_toc_declares_one_dependency() {
    let toc =
        TocFile::from_file(&professions_book_toc()).expect("Blizzard_ProfessionsBook TOC parses");

    let dependencies = toc.dependencies();
    let deps: Vec<&str> = dependencies.iter().map(|s| s.as_str()).collect();
    assert_eq!(
        deps, REQUIRED_DEPS,
        "TOC must declare exactly 1 dep: Blizzard_HelpPlate — supplies the HelpPlate \
         tutorial-overlay system referenced by ProfessionsBook_ToggleTutorial which \
         calls HelpPlate.Show / HelpPlate.Hide / HelpPlate.IsShowingHelpInfo against the \
         module-local ProfessionsFrame_HelpPlate config table"
    );

    assert!(
        toc.optional_deps().is_empty(),
        "Zero `## OptionalDeps:` declared"
    );
}

#[test]
fn blizzard_professions_book_toc_declares_no_saved_variables() {
    let toc =
        TocFile::from_file(&professions_book_toc()).expect("Blizzard_ProfessionsBook TOC parses");

    assert!(
        toc.saved_variables().is_empty(),
        "TOC must declare zero account-wide `## SavedVariables:` — pure stateless \
         display panel; all profession state comes from the live C_TradeSkillUI / \
         C_SpellBook / GetProfessions queries"
    );
    assert!(
        toc.saved_variables_per_character().is_empty(),
        "TOC must declare zero `## SavedVariablesPerCharacter:` — UI is fully derived \
         from server-authoritative skill-line / spellbook state"
    );
}

#[test]
fn blizzard_professions_book_toc_declares_metadata_in_raw_bytes() {
    let raw = std::fs::read_to_string(professions_book_toc())
        .expect("Blizzard_ProfessionsBook TOC reads utf-8");
    assert!(
        raw.contains("## Title: Blizzard Professions Book"),
        "TOC must declare `## Title: Blizzard Professions Book` — space-and-prose form, \
         player-facing panel name"
    );
    assert!(
        raw.contains("## LoadOnDemand: 1"),
        "TOC must declare `## LoadOnDemand: 1` exactly"
    );
    assert!(
        raw.contains("## Dependencies: Blizzard_HelpPlate"),
        "TOC must declare `## Dependencies: Blizzard_HelpPlate` — single hard dep on \
         the help-overlay machinery"
    );

    assert!(
        !raw.contains("## Author"),
        "TOC must NOT declare `## Author:` — minimal header (this addon is one of the \
         few Blizzard_* addons without the `## Author: Blizzard Entertainment` line)"
    );
    assert!(
        !raw.contains("## AllowLoad"),
        "TOC must NOT declare `## AllowLoad:` — relies on the `allows_screen` parser \
         default which is Game-only when the key is absent"
    );
    assert!(
        !raw.contains("## AllowLoadGameType"),
        "TOC must NOT declare `## AllowLoadGameType:` — cross-flavor"
    );
    assert!(
        !raw.contains("## SavedVariables"),
        "TOC must NOT declare `## SavedVariables:` or `## SavedVariablesPerCharacter:` \
         — fully stateless"
    );
    assert!(
        !raw.contains("## OptionalDeps"),
        "TOC must NOT declare `## OptionalDeps:` — only the single hard Dependencies \
         line"
    );
    assert!(
        !raw.contains("## UseSecureEnvironment"),
        "TOC must NOT declare `## UseSecureEnvironment:` — non-protected"
    );
    assert!(
        !raw.contains("## DefaultState"),
        "TOC must NOT declare `## DefaultState:` — defaults to enabled when omitted"
    );
}

#[test]
fn blizzard_professions_book_toc_lists_bootstrap_then_two_files_in_canonical_order() {
    let toc =
        TocFile::from_file(&professions_book_toc()).expect("Blizzard_ProfessionsBook TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, PROFESSIONS_BOOK_TOC_FILES,
        "Retail 12.1.0.69497 lists its bootstrap first, then Blizzard_ProfessionsBook.lua \
         before Blizzard_ProfessionsBook.xml so mixins and script-bound functions exist \
         before XML parsing"
    );
}

#[test]
fn blizzard_professions_book_does_not_appear_in_eager_discovery() {
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
            .any(|(name, _)| name == "Blizzard_ProfessionsBook");
        assert!(
            !found,
            "Blizzard_ProfessionsBook must NOT appear in eager discovery for {screen:?} \
             — `## LoadOnDemand: 1` excludes it. Loaded explicitly via the \
             ProfessionMicroButton OnClick → ToggleProfessionsBook path"
        );
    }
}

#[test]
fn blizzard_professions_book_appears_in_full_addon_inventory() {
    let inventory = discover_all_blizzard_addons(&blizzard_ui_dir());
    let found = inventory
        .iter()
        .any(|(name, _)| name == "Blizzard_ProfessionsBook");
    assert!(
        found,
        "Blizzard_ProfessionsBook must appear in `discover_all_blizzard_addons` — the \
         unfiltered inventory lists every parseable Blizzard_* TOC regardless of \
         LoadOnDemand gating"
    );
}

prefork_full_ui_case! {
fn blizzard_professions_book_loads_explicitly_after_dependencies(env: &WowLuaEnv) {

    {
        let mut state = env.state().borrow_mut();
        state.lua_errors.clear();
        state.lua_error_records.clear();
        state.lua_error_counts.clear();
    }

    load_addon(&env.loader_env(), &professions_book_toc())
        .expect("Blizzard_ProfessionsBook loads via Rust loader");

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_ProfessionsBook")
                || message.contains("ProfessionsBookFrame")
                || message.contains("ProfessionSpellButton")
                || message.contains("ProfessionsUnlearn")
                || message.contains("PROFESSION_RANKS")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_ProfessionsBook emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_professions_book_publishes_two_mixin_globals(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &professions_book_toc())
        .expect("Blizzard_ProfessionsBook loads cleanly");

    for name in PUBLIC_MIXIN_GLOBALS {
        let kind: String = env
            .eval(&format!("return type(_G.{name})"))
            .unwrap_or_else(|err| panic!("type(_G.{name}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{name} must publish as a table — the addon's XML mixin attributes on \
             ProfessionButtonTemplate (mixin=ProfessionSpellButtonMixin) and the \
             UnlearnButton inline mixin (mixin=ProfessionsUnlearnButtonMixin) must \
             resolve at parse time"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_professions_book_publishes_global_functions_and_tables(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &professions_book_toc())
        .expect("Blizzard_ProfessionsBook loads cleanly");

    for name in PUBLIC_GLOBAL_FUNCTIONS {
        let kind: String = env
            .eval(&format!("return type(_G.{name})"))
            .unwrap_or_else(|err| panic!("type(_G.{name}) probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "_G.{name} must publish as a function — the XML script bindings \
             (function=\"ProfessionsBookFrame_OnLoad\" etc.) resolve symbols against \
             _G at parse time"
        );
    }

    for name in PUBLIC_GLOBAL_TABLES {
        let kind: String = env
            .eval(&format!("return type(_G.{name})"))
            .unwrap_or_else(|err| panic!("type(_G.{name}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{name} must publish as a table — PROFESSION_RANKS is the 11-entry \
             classic ranks table (Apprentice 75 → BfA Master 950); \
             ProfessionsFrame_HelpPlate is the 2-entry HelpPlate config consumed by \
             ProfessionsBook_ToggleTutorial"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_professions_book_named_top_level_frames_are_in_global_env(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &professions_book_toc())
        .expect("Blizzard_ProfessionsBook loads cleanly");

    for frame in NAMED_NON_VIRTUAL_TOP_LEVEL_FRAMES {
        let kind: String = env
            .eval(&format!("return type(_G.{frame})"))
            .unwrap_or_else(|err| panic!("type(_G.{frame}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{frame} must exist as a frame table — Blizzard_ProfessionsBook \
             defines named non-virtual frames at top scope: ProfessionsBookFrame (the \
             root toplevel Frame inheriting ButtonFrameTemplate, parent=UIParent, \
             frameStrata=MEDIUM, hidden=true initially), ProfessionsContentFrame (the \
             content Frame inside ProfessionsBookFrame), 2x PrimaryProfession{{1,2}} + \
             3x SecondaryProfession{{1,2,3}} (the 5 profession-slot panels inside \
             ProfessionsContentFrame, each inheriting either PrimaryProfessionTemplate \
             or SecondaryProfessionTemplate), and 2x ProfessionsBookPage{{1,2}} (the \
             BACKGROUND-layer book-page art textures with file references to \
             Interface\\\\Spellbook\\\\Professions-Book-{{Left,Right}})"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_professions_book_virtual_templates_not_in_global_env(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &professions_book_toc())
        .expect("Blizzard_ProfessionsBook loads cleanly");

    for template in VIRTUAL_TEMPLATES_NOT_IN_GLOBALS {
        let kind: String = env
            .eval(&format!("return type(_G.{template})"))
            .unwrap_or_else(|err| panic!("type(_G.{template}) probe failed: {err}"));
        assert_eq!(
            kind, "nil",
            "_G.{template} must be nil — virtual templates live in the template \
             registry (consumed by `inherits=\"...\"` attributes on instances), NOT \
             in the global environment. Blizzard_ProfessionsBook ships 5 virtual \
             templates: ProfessionButtonTemplate (the SecureFrameTemplate-inheriting \
             FlyoutButtonTemplate-extending profession spell button), \
             ProfessionTrialCapTemplate (the trial-account cap-reached lock indicator), \
             ProfessionStatusBarTemplate (the rank status bar with capRight + capped \
             children), PrimaryProfessionTemplate (the 437x81 primary slot panel \
             hosting 2 spell buttons + status bar + unlearn button), \
             SecondaryProfessionTemplate (the 437x46 single-row secondary slot panel \
             hosting 2 spell buttons + status bar)"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_professions_book_profession_ranks_table_has_eleven_entries(env: &WowLuaEnv) {
    load_addon(&env.loader_env(), &professions_book_toc())
        .expect("Blizzard_ProfessionsBook loads cleanly");

    let count: i64 = env
        .eval("return #PROFESSION_RANKS")
        .expect("PROFESSION_RANKS length probe succeeds");
    assert_eq!(
        count, 11,
        "PROFESSION_RANKS must have exactly 11 entries — one per expansion's max-skill \
         tier from Apprentice (75) through Battle for Azeroth Master (950). Each entry \
         is `{{value, title}}` where `value` is the max skill cap and `title` is the \
         localized tier name. FormatProfession iterates this table to map a \
         profession's maxRank to its tier title when skillLineName isn't provided"
    );

    let first_value: i64 = env
        .eval("return PROFESSION_RANKS[1][1]")
        .expect("PROFESSION_RANKS[1][1] probe succeeds");
    assert_eq!(
        first_value, 75,
        "PROFESSION_RANKS[1][1] must be 75 — the Apprentice tier cap"
    );
    let last_value: i64 = env
        .eval("return PROFESSION_RANKS[11][1]")
        .expect("PROFESSION_RANKS[11][1] probe succeeds");
    assert_eq!(
        last_value, 950,
        "PROFESSION_RANKS[11][1] must be 950 — the BfA Master tier cap, the most \
         recent classic-style tier before the Dragonflight ranks system superseded \
         the linear cap progression"
    );
}
}
