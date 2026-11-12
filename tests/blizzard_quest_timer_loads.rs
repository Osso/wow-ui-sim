use std::path::PathBuf;

use wow_ui_sim::loader::find_toc_file;
use wow_ui_sim::loader::{discover_all_blizzard_addons, discover_blizzard_addons_for_screen};
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

#[path = "common/toc_layout.rs"]
mod toc_layout;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn quest_timer_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_QuestTimer")
}

fn quest_timer_toc() -> PathBuf {
    quest_timer_dir().join("Blizzard_QuestTimer.toc")
}

fn quest_timer_xml() -> PathBuf {
    let relative_path = if toc_layout::unified_toc_tree() {
        "Mainline/Blizzard_QuestTimer.xml"
    } else {
        "Blizzard_QuestTimer.xml"
    };
    quest_timer_dir().join(relative_path)
}

const TOC_FILES: &[&str] = &["Blizzard_QuestTimer.lua", "Blizzard_QuestTimer.xml"];

// The unified PTR TOC lists the family overrides and the shared button
// template before the core Lua/XML pair.
const PTR_TOC_FILES: &[&str] = &[
    "Mainline/Blizzard_QuestTimerOverrides.lua",
    "QuestTimerButtonTemplate.xml",
    "Blizzard_QuestTimer.lua",
    "Mainline/Blizzard_QuestTimer.xml",
];

fn quest_timer_files() -> &'static [&'static str] {
    if toc_layout::unified_toc_tree() {
        PTR_TOC_FILES
    } else {
        TOC_FILES
    }
}

const REQUIRED_DEPS: &[&str] = &["Blizzard_GameTooltip", "Blizzard_ManagedFrameSystem"];

// Literal cached inputs absent from Wrath manifest1-382/cache; behavior cases stay enabled.
post_wrath_ui_fixture! {
#[test]
fn blizzard_quest_timer_find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&quest_timer_dir()).expect("Blizzard_QuestTimer TOC resolves");
    assert_eq!(
        resolved,
        quest_timer_toc(),
        "Blizzard_QuestTimer ships a SINGLE bare `Blizzard_QuestTimer.toc` (NO \
         `_Mainline.toc` / `_Classic.toc` variant — the classic-only gate is \
         carried inside the bare TOC via `## AllowLoadGameType: classic` rather \
         than via a `_Classic.toc` filename suffix). `find_toc_file` walks the \
         suffix-priority list `[_Mainline.toc, .toc]` and falls through to the \
         bare form because no Mainline-suffixed variant exists"
    );

    for variant_suffix in ["_Mainline.toc", "_Mists.toc", "_Wrath.toc", "_Classic.toc"] {
        let variant = quest_timer_dir().join(format!("Blizzard_QuestTimer{variant_suffix}"));
        assert!(
            !variant.exists(),
            "Blizzard_QuestTimer must NOT ship a {variant_suffix} variant — \
             single bare TOC only with the classic gate inside the metadata"
        );
    }
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_quest_timer_toc_pins_classic_only_with_default_state_enabled() {
    let toc = TocFile::from_file(&quest_timer_toc()).expect("Blizzard_QuestTimer TOC parses");

    assert_eq!(
        toc.is_game_type_restricted(),
        !(toc_layout::admits_game_type("classic") || toc_layout::admits_game_type("camelot")),
        "TOC declares `## AllowLoadGameType: classic, camelot`. Retail and PTR admit \
         neither token, so is_game_type_restricted returns TRUE there and the loader \
         filter at src/loader/mod.rs:527 rejects the addon. Forever admits `camelot`, \
         so the same directive does not restrict it on that profile"
    );

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "TOC declares `## AllowLoad: game` (lowercase) — `allows_screen` at \
         src/toc.rs:308 routes via `eq_ignore_ascii_case(\"game\")` so the \
         lowercase form resolves the same as `## AllowLoad: Game`"
    );
    for screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(screen),
            "Game-only screen gate must EXCLUDE {screen:?} — quest timer is an \
             in-world frame"
        );
    }

    assert!(!toc.is_load_on_demand());
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(!toc.is_ptr_only());
    assert!(toc.optional_deps().is_empty());
    assert!(toc.load_with().is_empty());
    assert!(
        toc.saved_variables().is_empty(),
        "TOC must declare ZERO `## SavedVariables:` — pure stateless display: \
         every timer pulls from live `GetQuestTimers()` each frame"
    );
    assert!(toc.saved_variables_per_character().is_empty());
}
}

retail_toc_metadata_fixture! {
#[test]
fn blizzard_quest_timer_toc_declares_game_tooltip_and_managed_frame_dependencies() {
    let toc = TocFile::from_file(&quest_timer_toc()).expect("TOC parses");
    let dependencies = toc.dependencies();
    let deps: Vec<&str> = dependencies.iter().map(|s| s.as_str()).collect();
    assert_eq!(
        deps, REQUIRED_DEPS,
        "Retail 12.1.0.69497 declares Blizzard_GameTooltip and \
         Blizzard_ManagedFrameSystem in published order"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_quest_timer_toc_declares_metadata_in_raw_bytes() {
    let raw = std::fs::read_to_string(quest_timer_toc()).expect("TOC reads utf-8");

    assert!(raw.contains("## Title: Blizzard Quest Timer"));
    assert!(raw.contains("## Author: Blizzard Entertainment"));
    assert!(
        raw.contains("## DefaultState: enabled"),
        "TOC must declare `## DefaultState: enabled` — the AddOn list UI shows \
         the addon enabled by default for users who toggle it manually"
    );
    if toc_layout::classic_cache() {
        assert!(raw.contains("## Dependencies: Blizzard_UIParent, Blizzard_GameTooltip"));
    } else {
        assert!(raw.contains("## Dependencies: Blizzard_GameTooltip, Blizzard_ManagedFrameSystem"));
    }
    assert!(raw.contains("## AllowLoad: game"));
    assert!(raw.contains("## AllowLoadGameType: classic"));

    assert!(
        !raw.contains("## LoadOnDemand"),
        "TOC must NOT carry any LoadOnDemand directive (eager-loaded on classic)"
    );
    assert!(
        !raw.contains("## SavedVariables"),
        "TOC must NOT carry any SavedVariables directive"
    );
    assert!(
        !raw.contains("## OnlyBetaAndPTR"),
        "TOC must NOT carry OnlyBetaAndPTR — ships on live classic"
    );
    assert!(
        !raw.contains("## Version"),
        "TOC must NOT carry a Version directive — one of the few Blizzard_* \
         addons missing the canonical version line"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_quest_timer_toc_lists_two_files_lua_then_xml() {
    let toc = TocFile::from_file(&quest_timer_toc()).expect("TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed,
        quest_timer_files(),
        "TOC body lists the mixin Lua before the XML that consumes it: \
         Blizzard_QuestTimer.lua declares both QuestTimerMixin and \
         QuestTimerButtonMixin at file scope, with the QuestTimerMixin owning \
         the OnLoad/OnEvent/OnUpdate/Update/UpdateQuestTimers state machine. \
         Blizzard_QuestTimer.xml then publishes QuestTimerButtonTemplate as \
         a virtual button template plus the QuestTimerFrame toplevel container \
         with 25 numbered child Buttons QuestTimer1..QuestTimer25 each \
         inheriting QuestTimerButtonTemplate and anchored to the previous \
         numbered button via `relativeTo=\"QuestTimer<n-1>\"`. The unified \
         PTR TOC also lists the family overrides Lua and the shared button \
         template before the core pair"
    );
}
}

#[test]
fn quest_timer_eager_discovery_matches_profile_game_types() {
    let ui = blizzard_ui_dir();

    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let found = addons.iter().any(|(name, _)| name == "Blizzard_QuestTimer");
        let classic_profile_has_addon = toc_layout::classic_cache() && !toc_layout::wrath();
        let expected =
            (classic_profile_has_addon || toc_layout::forever()) && screen == ScreenKind::Game;
        assert_eq!(
            found, expected,
            "QuestTimer allows camelot on Forever and is Game-only; source TOC allows classic, camelot"
        );
    }
}

post_wrath_ui_fixture! {
#[test]
fn blizzard_quest_timer_appears_in_full_addon_inventory() {
    let inventory = discover_all_blizzard_addons(&blizzard_ui_dir());
    let found = inventory
        .iter()
        .any(|(name, _)| name == "Blizzard_QuestTimer");
    assert!(
        found,
        "Blizzard_QuestTimer MUST appear in `discover_all_blizzard_addons` — \
         the full-inventory accessor at src/loader/mod.rs:309-343 does NOT \
         apply the `is_game_type_restricted()` filter; only \
         `discover_blizzard_addons_for_screen` does. This asymmetry lets \
         classic-flavor addons show up in admin / debug surfaces while still \
         being excluded from the eager retail load path"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn quest_timer_xml_matches_profile_button_creation() {
    let raw = std::fs::read_to_string(quest_timer_xml()).expect("XML reads utf-8");

    if toc_layout::unified_toc_tree() {
        assert!(raw.contains("name=\"QuestTimerFrame\""));
        assert!(raw.contains("mixin=\"QuestTimerMixin\""));
        for index in 1..=25 {
            assert!(
                !raw.contains(&format!("name=\"QuestTimer{index}\"")),
                "unified PTR/Forever QuestTimer XML must not declare static row {index}"
            );
        }

        let template = std::fs::read_to_string(
            quest_timer_dir().join("QuestTimerButtonTemplate.xml"),
        )
        .expect("shared button template reads");
        assert!(template.contains("name=\"QuestTimerButtonTemplate\""));
        assert!(template.contains("mixin=\"QuestTimerButtonMixin\""));

    } else {
        for index in 1..=25 {
            assert!(
                raw.contains(&format!("name=\"QuestTimer{index}\"")),
                "flavor TOC QuestTimer XML must declare static row {index}"
            );
        }
        assert!(raw.contains("QuestTimerButtonTemplate"));
        assert!(raw.contains("QuestTimerMixin"));
        assert!(raw.contains("QuestTimerButtonMixin"));
    }
}
}
