#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{
    discover_all_blizzard_addons, discover_blizzard_addons_for_screen, find_toc_file, load_addon,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn npe_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_NewPlayerExperience")
}

fn npe_toc() -> PathBuf {
    npe_dir().join("Blizzard_NewPlayerExperience.toc")
}

const NPE_TOC_FILES: &[&str] = &[
    "Blizzard_NewPlayerExperience_Bootstrap.lua",
    "Blizzard_TutorialKeyboardMouseFrame.xml",
    "Blizzard_TutorialData.lua",
    "Blizzard_TutorialWatchers.lua",
    "Blizzard_TutorialServices.lua",
    "Blizzard_TutorialTutorials.lua",
    "Blizzard_TutorialLogic.lua",
    "Blizzard_Tutorial.lua",
];

const PUBLIC_TABLES: &[&str] = &[
    "NewPlayerExperience",
    "TutorialLogic",
    "TutorialKeyboardMouseFrameMixin",
    "TutorialWalkMixin",
];

const NAMED_FRAMES: &[&str] = &["TutorialKeyboardMouseFrame_Frame", "TutorialWalk_Frame"];

fn load_full_game_ui_then_request_npe() -> WowLuaEnv {
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

    load_addon(&env.loader_env(), &npe_toc())
        .expect("Blizzard_NewPlayerExperience load_addon succeeds after eager Game-screen sweep");

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);

    env
}

#[test]
fn blizzard_npe_find_toc_resolves_bare_variant() {
    let resolved = find_toc_file(&npe_dir()).expect("Blizzard_NewPlayerExperience TOC resolves");
    assert_eq!(
        resolved,
        npe_toc(),
        "Blizzard_NewPlayerExperience ships exactly one bare TOC — no `_Mainline.toc` and no \
         `_Classic.toc`. The leveling-tutorial overlay is a retail-only feature (NPEv2 is the \
         retail leveling experience), but its retail-onliness is expressed via the absence of \
         classic-flavor TOCs rather than a flavor split — `find_toc_file` resolves the bare \
         TOC after the `_Mainline.toc` lookup misses"
    );

    let mainline = npe_dir().join("Blizzard_NewPlayerExperience_Mainline.toc");
    assert!(
        !mainline.exists(),
        "There must be NO `_Mainline.toc` at {} — the retail-only NPEv2 overlay ships a single \
         bare TOC; flavor restriction is enforced at the dependency layer (RequiredDep on \
         Blizzard_TutorialManager, which is itself retail-only) rather than at the TOC layer",
        mainline.display()
    );
}

#[test]
fn blizzard_npe_toc_declares_load_on_demand_with_current_required_deps() {
    let toc = TocFile::from_file(&npe_toc()).expect("Blizzard_NewPlayerExperience TOC parses");
    assert!(
        toc.is_load_on_demand(),
        "TOC declares `## LoadOnDemand: 1` — the NPEv2 overlay defers load until the \
         TutorialManager decides the player is eligible. Eager-loading would waste resources \
         on every veteran character whose `showTutorials` CVar or NPE_AchievementID makes \
         the overlay a no-op"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());

    let deps = toc.dependencies();
    assert_eq!(
        deps,
        vec![
            "middleclass",
            "Blizzard_TutorialManager",
            "Blizzard_LFGUtil"
        ],
        "Retail 12.1.0.69497 declares middleclass, Blizzard_TutorialManager, and \
         Blizzard_LFGUtil through its singular RequiredDep line"
    );

    assert!(
        toc.optional_deps().is_empty(),
        "Zero `## OptionalDeps:` — NPEv2 has exactly one hard dependency (TutorialManager) and \
         no soft siblings. The other surfaces it touches (Dispatcher, HelpPlate, EventRegistry) \
         are pulled in transitively via TutorialManager's own dep list"
    );
    assert!(
        toc.saved_variables().is_empty(),
        "Zero saved variables — NPE completion state is server-driven via the achievement \
         system (NPE_AchievementID, IsPlayerEligibleForNPEv2, IsPlayerNPERestricted). \
         `showTutorials` is a CVar (engine-side persistence), not a Lua SavedVariable"
    );
    assert!(
        !toc.is_game_type_restricted(),
        "TOC omits `## AllowLoadGameType:` — the NPEv2 overlay is implicitly retail-only via \
         its TutorialManager RequiredDep, but the file-list itself loads on every game type \
         that resolves the dep. `is_game_type_restricted()` at src/toc.rs:294 returns false \
         when the metadata key is absent"
    );
}

#[test]
fn blizzard_npe_toc_declares_load_on_demand_in_raw_bytes() {
    let raw = std::fs::read_to_string(npe_toc())
        .expect("Blizzard_NewPlayerExperience TOC reads as utf-8");
    assert!(
        raw.contains("## LoadOnDemand: 1"),
        "TOC must declare `## LoadOnDemand: 1` exactly. The explicit `1` (rather than \
         omitting / `## LoadOnDemand: 0`) is what routes the addon to the lod_pool at \
         src/loader/mod.rs:530-534, keeping it out of the eager Game-screen discovery sweep"
    );
    assert!(
        raw.contains("## RequiredDep: middleclass, Blizzard_TutorialManager, Blizzard_LFGUtil"),
        "Retail 12.1.0.69497 declares all three RequiredDep values on one singular \
         RequiredDep line"
    );
    assert!(
        !raw.contains("## AllowLoad:"),
        "TOC must NOT declare `## AllowLoad:`. With `## AllowLoad:` omitted, `allows_screen` \
         at src/toc.rs:311 defaults to Game-only (`screen == ScreenKind::Game`) — but the \
         LoadOnDemand routing means the addon does not appear in eager discovery on any \
         screen anyway; the AllowLoad default applies to all addons regardless of LoD status"
    );
}

#[test]
fn blizzard_npe_toc_lists_bootstrap_then_seven_files() {
    let toc = TocFile::from_file(&npe_toc()).expect("Blizzard_NewPlayerExperience TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, NPE_TOC_FILES,
        "Retail 12.1.0.69497 lists its bootstrap before the XML and seven NPE source files; \
         Blizzard_TutorialKeyboardMouseFrame.xml remains the first non-bootstrap entry and \
         Blizzard_Tutorial.lua remains last"
    );

    assert_eq!(
        listed.get(1).map(String::as_str),
        Some("Blizzard_TutorialKeyboardMouseFrame.xml"),
        "The first non-bootstrap entry must be the keyboard/mouse XML frame definition"
    );
    assert_eq!(
        listed.last().map(String::as_str),
        Some("Blizzard_Tutorial.lua"),
        "Last file MUST be Blizzard_Tutorial.lua — its line 66 contains the only \
         module-top-level call site `NewPlayerExperience:Initialize()`. Initialize → Begin \
         registers EventRegistry callbacks against TutorialManager events; ordering this last \
         guarantees TutorialLogic / NewPlayerExperience tables are populated before the \
         callback chain can fire"
    );
}

#[test]
fn blizzard_npe_does_not_appear_in_eager_discovery_on_any_screen() {
    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_NewPlayerExperience");
        assert!(
            !found,
            "Blizzard_NewPlayerExperience must NOT auto-discover on screen {screen:?} — \
             `## LoadOnDemand: 1` routes the addon to the lod_pool at src/loader/mod.rs:530-534, \
             not the eager set. NPE is loaded on demand by Blizzard_TutorialManager when the \
             player is eligible (achievement-uncompleted AND showTutorials CVar AND \
             IsPlayerEligibleForNPEv2 — the gating chain in Blizzard_Tutorial.lua line 12-28)"
        );
    }
}

#[test]
fn blizzard_npe_appears_in_discover_all_blizzard_addons() {
    let all = discover_all_blizzard_addons(&blizzard_ui_dir());
    let found = all
        .iter()
        .any(|(name, _)| name == "Blizzard_NewPlayerExperience");
    assert!(
        found,
        "Blizzard_NewPlayerExperience must appear in `discover_all_blizzard_addons` — that \
         helper enumerates every `Blizzard_*` directory regardless of LoD or screen \
         restriction. The addon-management UI relies on this exhaustive sweep to render every \
         addon row, including LoD addons that are not eagerly discovered"
    );
}

#[test]
fn blizzard_npe_loads_without_addon_specific_lua_errors() {
    let env = load_full_game_ui_then_request_npe();

    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_NewPlayerExperience")
                || message.contains("NewPlayerExperience")
                || message.contains("TutorialLogic")
                || message.contains("TutorialKeyboardMouseFrame")
                || message.contains("TutorialWalk")
                || message.starts_with("Class_Intro_")
                || message.starts_with("Class_AddSpellToActionBarService")
        })
        .cloned()
        .collect();
    assert!(
        load_errors.is_empty(),
        "Blizzard_NewPlayerExperience emitted addon-specific Lua errors during load:\n  {}",
        load_errors.join("\n  ")
    );
}

#[test]
fn blizzard_npe_is_addon_loaded_after_explicit_load_addon_call() {
    let env = load_full_game_ui_then_request_npe();

    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_NewPlayerExperience')")
        .expect("IsAddOnLoaded probe succeeds");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_NewPlayerExperience') must return true after the \
         explicit load_addon call — proves the LoadOnDemand routing reaches the loaded-set \
         only via explicit request, not via eager discovery. The TutorialManager addon \
         orchestrates this load_addon equivalent on retail when the player meets the \
         eligibility gate"
    );
}

#[test]
fn blizzard_npe_publishes_four_top_level_tables() {
    let env = load_full_game_ui_then_request_npe();

    for global in PUBLIC_TABLES {
        let kind: String = env
            .eval(&format!("return type(_G.{global})"))
            .unwrap_or_else(|err| panic!("type(_G.{global}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{global} must publish as a table after Blizzard_NewPlayerExperience loads. \
             `NewPlayerExperience` is the addon entry point declared at \
             Blizzard_Tutorial.lua line 1; `TutorialLogic` is the orchestrator declared at \
             Blizzard_TutorialLogic.lua line 4; `TutorialKeyboardMouseFrameMixin` is the \
             keyboard-mouse-tutorial frame mixin at Blizzard_TutorialKeyboardMouseFrame.lua \
             line 8; `TutorialWalkMixin` is the walk-tutorial frame mixin extending \
             TutorialMainFrameMixin (TutorialManager-owned) at line 90. All four are seed \
             tables addressed by name from XML mixin attributes / EventRegistry callbacks"
        );
    }
}

#[test]
fn blizzard_npe_publishes_two_named_xml_frames_as_globals() {
    let env = load_full_game_ui_then_request_npe();

    for frame in NAMED_FRAMES {
        let kind: String = env
            .eval(&format!("return type(_G.{frame})"))
            .unwrap_or_else(|err| panic!("type(_G.{frame}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{frame} must publish as a userdata-backed Frame — XML \
             Blizzard_TutorialKeyboardMouseFrame.xml lines 5 and 46 declare \
             `<Frame name=\"{frame}\" ...>` with parent=UIParent. Both inherit \
             ResizeLayoutFrame and host the keyboard-binding visualization grid; the \
             keyboard-mouse frame uses frameLevel=300 + toplevel=true to render above the \
             world scene, the walk frame anchors BOTTOM y=232 to sit just above the action \
             bar"
        );
    }
}

#[test]
fn blizzard_npe_publishes_dispatcher_and_event_registry_dependencies() {
    let env = load_full_game_ui_then_request_npe();

    for global in &["EventRegistry", "Dispatcher", "TutorialManager", "HelpTip"] {
        let kind: String = env
            .eval(&format!("return type(_G.{global})"))
            .unwrap_or_else(|err| panic!("type(_G.{global}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{global} must publish as a table — the NPE entry point \
             (Blizzard_Tutorial.lua) calls EventRegistry:RegisterCallback at line 8-9 (the \
             TutorialsEnabled / TutorialsDisabled bus), Dispatcher:RegisterEvent at line 30 \
             (PLAYER_LEVEL_UP wiring), TutorialManager.NPE_AchievementID at line 13 (the \
             eligibility gate), and HelpTip:SetHelpTipsEnabled at line 31 (the help-tip \
             suppression). All four are pulled in transitively: TutorialManager loads as the \
             RequiredDep, dragging in Dispatcher + HelpPlate via its own Dependencies; \
             EventRegistry / HelpTip are foundational SharedXML / SharedXMLBase globals \
             published before any addon Lua runs"
        );
    }
}

#[test]
fn blizzard_npe_intro_class_prototypes_are_constructed_at_load() {
    let env = load_full_game_ui_then_request_npe();

    for class_name in &[
        "Class_Intro_KeyboardMouse",
        "Class_Intro_CameraLook",
        "Class_Intro_ApproachQuestGiver",
        "Class_AddSpellToActionBarService",
        "Class_AcceptQuestWatcher",
    ] {
        let kind: String = env
            .eval(&format!("return type(_G.{class_name})"))
            .unwrap_or_else(|err| panic!("type(_G.{class_name}) probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "_G.{class_name} must publish as a table — the NPE files use the `class(...)` \
             prototype helper from TutorialManager's Blizzard_TutorialBase.lua to declare \
             52 Class_* globals across TutorialServices.lua / TutorialWatchers.lua / \
             TutorialTutorials.lua. Each represents one tutorial step (Intro_*) or one \
             watcher (Watcher / Service). If `class()` is missing or the parent prototype \
             `Class_TutorialBase` is nil, the file fails at module top-level. This probe \
             samples 5 representative prototypes spanning all 3 module files"
        );
    }
}
