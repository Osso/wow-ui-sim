use std::path::PathBuf;

use wow_ui_sim::loader::find_toc_file;
use wow_ui_sim::loader::{discover_all_blizzard_addons, discover_blizzard_addons_for_screen};
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

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

const TOC_FILES: &[&str] = &["Blizzard_QuestTimer.lua", "Blizzard_QuestTimer.xml"];

const REQUIRED_DEPS: &[&str] = &["Blizzard_GameTooltip", "Blizzard_ManagedFrameSystem"];

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

#[test]
fn blizzard_quest_timer_toc_pins_classic_only_with_default_state_enabled() {
    let toc = TocFile::from_file(&quest_timer_toc()).expect("Blizzard_QuestTimer TOC parses");

    assert!(
        toc.is_game_type_restricted(),
        "TOC declares `## AllowLoadGameType: classic` — `is_game_type_restricted()` \
         at src/toc.rs:294-302 returns TRUE because `classic` is NOT in the \
         `mainline | standard` cross-flavor allowlist. The loader filter at \
         src/loader/mod.rs:527 rejects this addon on retail; it only loads on \
         the Classic Era client where the game type matches"
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
    assert!(raw.contains("## Dependencies: Blizzard_GameTooltip, Blizzard_ManagedFrameSystem"));
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

#[test]
fn blizzard_quest_timer_toc_lists_two_files_lua_then_xml() {
    let toc = TocFile::from_file(&quest_timer_toc()).expect("TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, TOC_FILES,
        "TOC body lists EXACTLY 2 files in canonical Lua-then-XML order: \
         Blizzard_QuestTimer.lua FIRST (declares both QuestTimerMixin and \
         QuestTimerButtonMixin at file scope, with the QuestTimerMixin owning \
         the OnLoad/OnEvent/OnUpdate/Update/UpdateQuestTimers state machine), \
         Blizzard_QuestTimer.xml SECOND (publishes QuestTimerButtonTemplate as \
         a virtual button template plus the QuestTimerFrame toplevel container \
         with 25 numbered child Buttons QuestTimer1..QuestTimer25 each \
         inheriting QuestTimerButtonTemplate and anchored to the previous \
         numbered button via `relativeTo=\"QuestTimer<n-1>\"`). UNLIKE \
         Blizzard_QuestNavigation, this addon DOES list its companion Lua \
         explicitly in the TOC body — the older eager-load pattern"
    );
}

#[test]
fn blizzard_quest_timer_excluded_from_eager_game_discovery() {
    let ui = blizzard_ui_dir();

    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&ui, screen);
        let found = addons.iter().any(|(name, _)| name == "Blizzard_QuestTimer");
        assert!(
            !found,
            "Blizzard_QuestTimer MUST NOT appear in eager discovery for \
             {screen:?} — `## AllowLoadGameType: classic` flips \
             `is_game_type_restricted()` true on retail, and the loader filter \
             at src/loader/mod.rs:527 rejects classic-only addons during \
             eager-pool construction. The simulator emulates a retail/mainline \
             client, so classic-gated addons stay dormant on every screen"
        );
    }
}

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

#[test]
fn blizzard_quest_timer_xml_declares_twenty_five_named_buttons() {
    let xml_path = quest_timer_dir().join("Blizzard_QuestTimer.xml");
    let raw = std::fs::read_to_string(&xml_path).expect("XML reads utf-8");

    for index in 1..=25 {
        let needle = format!("name=\"QuestTimer{index}\"");
        assert!(
            raw.contains(&needle),
            "QuestTimerFrame XML must declare button `QuestTimer{index}` — the \
             addon hardcodes 25 numbered buttons (QuestTimer1..QuestTimer25) \
             that `QuestTimerMixin:UpdateQuestTimers` indexes via the \
             classic-era global lookup `_G[\"QuestTimer\" .. i]`. The hard \
             upper bound matches the legacy `MAX_QUESTS = 25` constant — \
             classic clients capped a player's quest log at 25 entries; the \
             addon allocates exactly that many timer slots up-front and \
             toggles their visibility in the per-tick UpdateQuestTimers loop"
        );
    }

    assert!(
        raw.contains("QuestTimerButtonTemplate"),
        "XML must declare the QuestTimerButtonTemplate virtual button — the \
         template carries the GameTooltip OnEnter / OnLeave scripts and the \
         OnClick handler that opens the quest log via ShowUIPanel(QuestLogFrame)"
    );
    assert!(
        raw.contains("QuestTimerMixin") && raw.contains("QuestTimerButtonMixin"),
        "XML must reference both Lua-defined mixins via `mixin=...` attributes"
    );
}
