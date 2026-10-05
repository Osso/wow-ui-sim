use std::path::PathBuf;

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path().expect("Blizzard UI cache should be available")
}

fn tutorial_manager_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_TutorialManager")
}

fn tutorial_manager_toc() -> PathBuf {
    tutorial_manager_dir().join("Blizzard_TutorialManager.toc")
}

const GLUE_SCREENS: &[ScreenKind] = &[
    ScreenKind::Login,
    ScreenKind::CharacterSelect,
    ScreenKind::CharacterCreate,
];

const TOC_DEPENDENCIES: &[&str] = &["middleclass", "Blizzard_Dispatcher", "Blizzard_HelpPlate"];

const MODULE_TABLES: &[&str] = &[
    "TutorialManager",
    "TutorialQueue",
    "TutorialRangeManager",
    "TutorialHelper",
    "TutorialPointerFrame",
    "TutorialQuestBangGlow",
    "TutorialDragButton",
];

const MIXINS: &[&str] = &[
    "TutorialQuestManagerMixin",
    "TutorialMainFrameMixin",
    "TutorialSingleKeyMixin",
    "TutorialDoubleKeyMixin",
];

const TUTORIAL_MANAGER_METHODS: &[&str] = &[
    "Initialize",
    "Begin",
    "Shutdown",
    "OnSettingsLoaded",
    "ResetTutorials",
    "OnCVARsUpdated",
    "DebugLog",
    "GetIsActive",
    "AddTutorial",
    "RemoveTutorial",
    "GetTutorial",
    "Queue",
    "Finished",
    "ShutdownTutorial",
    "AddWatcher",
    "RemoveWatcher",
    "GetWatcher",
    "StartWatcher",
    "StopWatcher",
    "ShutdownWatcher",
    "CheckHasCompletedFrameTutorial",
    "CheckHasCompletedTutorial",
    "TutorialStatus",
];

const NAMED_NON_VIRTUAL_FRAMES: &[&str] = &[
    "TutorialMainFrame_Frame",
    "TutorialSingleKey_Frame",
    "TutorialDoubleKey_Frame",
    "TutorialDragOriginFrame",
    "TutorialDragTargetFrame",
    "TutorialDragAnimationFrame",
];

fn fresh_game_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![blizzard_ui_dir()];
    }
    wow_ui_sim::xml::register_intrinsic_templates();
    env
}

fn load_full_game_ui() -> WowLuaEnv {
    let env = fresh_game_env();

    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    for (name, toc_path) in &addons {
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

#[test]
fn find_toc_file_resolves_bare_toc() {
    let resolved = find_toc_file(&tutorial_manager_dir()).expect("TutorialManager TOC resolves");
    assert_eq!(
        resolved,
        tutorial_manager_toc(),
        "Bare TOC — no flavor suffix. Resolved via the bare-TOC path \
         in find_toc_file at src/loader/mod.rs:65-95"
    );
}

#[test]
fn toc_is_eager_with_three_dependencies() {
    let toc = TocFile::from_file(&tutorial_manager_toc()).expect("TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "No `## LoadOnDemand` directive → eagerly loaded on Game. The \
         tutorial system has to be live the moment the player can \
         interact with the world so it can intercept frame events for \
         the New Player Experience and the in-game Tutorials addon"
    );

    let deps = toc.dependencies();
    assert_eq!(
        deps.len(),
        TOC_DEPENDENCIES.len(),
        "Must declare exactly {} hard deps. Got {}: {:?}",
        TOC_DEPENDENCIES.len(),
        deps.len(),
        deps
    );
    for expected in TOC_DEPENDENCIES {
        assert!(
            deps.iter().any(|d| d == expected),
            "TOC must declare `{expected}` — Blizzard_Dispatcher \
             provides the EventRegistry/CallbackRegistry plumbing the \
             tutorial queue subscribes to; Blizzard_HelpPlate provides \
             the help-plate overlays the tutorial pointer leans on. \
             Got: {deps:?}"
        );
    }

    assert!(toc.optional_deps().is_empty());
    assert!(toc.saved_variables().is_empty());
    assert!(toc.saved_variables_per_character().is_empty());
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(
        !toc.is_game_type_restricted(),
        "AllowLoadGameType absent → not restricted"
    );
    assert!(toc.default_enabled());
}

#[test]
fn allow_load_absent_implies_game_only() {
    let toc = TocFile::from_file(&tutorial_manager_toc()).expect("TOC parses");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "AllowLoad absent → toc.rs:305-313 None branch defaults to \
         Game-only. The tutorial system never runs on glue screens"
    );
    for screen in GLUE_SCREENS {
        assert!(
            !toc.allows_screen(*screen),
            "Glue screen {screen:?} must be excluded — no AllowLoad \
             directive means Game-only via the None branch"
        );
    }
}

#[test]
fn toc_raw_bytes_pin_three_directives_and_nine_body_files() {
    let raw = std::fs::read_to_string(tutorial_manager_toc()).expect("TOC reads utf-8");

    let expected_lines = [
        "## Title: Blizzard_TutorialManager",
        "## Dep: middleclass",
        "## Dep: Blizzard_Dispatcher",
        "## Dep: Blizzard_HelpPlate",
        "Blizzard_TutorialQueue.lua",
        "Blizzard_TutorialQuestManager.lua",
        "Blizzard_TutorialRangeManager.lua",
        "Blizzard_TutorialMainFrame.xml",
        "Blizzard_TutorialPointerFrame.xml",
        "Blizzard_TutorialEffects.xml",
        "Blizzard_TutorialBase.lua",
        "Blizzard_TutorialHelper.lua",
        "Blizzard_TutorialManager.lua",
    ];

    for line in expected_lines {
        assert!(
            raw.contains(line),
            "Raw TOC must pin `{line}` — body order matters: 3 \
             support-lua loaders first (queue/quest/range), then 3 \
             XML files (main/pointer/effects), then 3 implementation \
             lua (base/helper/manager). The trailing manager lua \
             ties everything together by registering itself with the \
             Settings system after queue + frames are live"
        );
    }

    assert!(!raw.contains("## Author"));
    assert!(!raw.contains("## LoadOnDemand"));
    assert!(!raw.contains("## AllowLoad"));
    assert!(!raw.contains("## DefaultState"));
    assert!(!raw.contains("## OptionalDeps"));
    assert!(!raw.contains("## RequiredDep"));
    assert!(!raw.contains("## SavedVariables"));
    assert!(!raw.contains("## AllowLoadGameType"));
    assert!(!raw.contains("## UseSecureEnvironment"));
    assert!(!raw.contains("## LoadFirst"));
    assert!(!raw.contains("## Version"));
    assert!(!raw.contains("[Family]"));
}

#[test]
fn game_discovery_loads_middleclass_before_tutorial_manager_and_other_builtin_roots() {
    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let middleclass_index = addons
        .iter()
        .position(|(name, _)| name == "middleclass")
        .expect("TutorialManager hard dependency `middleclass` must be discovered");
    let tutorial_manager_index = addons
        .iter()
        .position(|(name, _)| name == "Blizzard_TutorialManager")
        .expect(
            "Blizzard_TutorialManager must appear in Game eager discovery — non-LoD addon \
             with no AllowLoad (Game-only)",
        );

    assert!(
        middleclass_index < tutorial_manager_index,
        "middleclass must load before Blizzard_TutorialManager so Blizzard_TutorialBase.lua \
         can call class(\"TutorialBase\")"
    );
    assert!(
        addons
            .iter()
            .any(|(name, _)| name == "Deprecated_PaperDoll"),
        "built-in non-LoD Deprecated_PaperDoll ships in the client interface data, so it \
         is a Game startup root despite lacking the `Blizzard_` prefix"
    );
}

#[test]
fn absent_from_glue_screens_eager_discovery() {
    for screen in GLUE_SCREENS {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), *screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_TutorialManager");
        assert!(
            !found,
            "Blizzard_TutorialManager must NOT appear on {screen:?} — \
             AllowLoad absent → Game-only via toc.rs:305-313 None \
             branch"
        );
    }
}

#[test]
fn dep_directories_exist_on_disk() {
    for dep in TOC_DEPENDENCIES {
        let dir = blizzard_ui_dir().join(dep);
        assert!(
            dir.is_dir(),
            "Hard-dep directory `{dep}` must exist on disk"
        );
        assert!(
            find_toc_file(&dir).is_some(),
            "{dep} must have a discoverable TOC"
        );
    }
}

prefork_full_ui_case! {
fn explicit_load_publishes_module_tables(env: &WowLuaEnv) {

    for module in MODULE_TABLES {
        let kind: String = env
            .eval(&format!("return type({module})"))
            .unwrap_or_else(|err| panic!("{module} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{module} must be a global table — declared at module-load \
             in its respective body file. TutorialManager+TutorialQueue+\
             TutorialRangeManager are runtime singletons; TutorialHelper \
             is a stateless utility namespace; TutorialPointerFrame is \
             a pool-managing facade over the virtual `Tutorial_Pointer*` \
             XML templates; TutorialQuestBangGlow + TutorialDragButton \
             are effect-helper tables in TutorialEffects.lua"
        );
    }
}
}

prefork_full_ui_case! {
fn explicit_load_publishes_mixin_tables(env: &WowLuaEnv) {

    for mixin in MIXINS {
        let kind: String = env
            .eval(&format!("return type({mixin})"))
            .unwrap_or_else(|err| panic!("{mixin} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{mixin} must be a global table. \
             TutorialQuestManagerMixin + TutorialMainFrameMixin are the \
             base mixins; TutorialSingleKeyMixin and \
             TutorialDoubleKeyMixin are CreateFromMixins(TutorialMainFrameMixin) \
             extensions for keybind-prompt frames"
        );
    }
}
}

prefork_full_ui_case! {
fn tutorial_quest_manager_is_built_from_mixin(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(TutorialQuestManager)")
        .expect("TutorialQuestManager probe");
    assert_eq!(
        kind, "table",
        "TutorialQuestManager must be a table built via \
         `CreateFromMixins(TutorialQuestManagerMixin)` at \
         Blizzard_TutorialQuestManager.lua:184. CreateFromMixins is a \
         shallow copy, so all 25 methods of the mixin appear directly \
         on this instance"
    );
}
}

prefork_full_ui_case! {
fn explicit_load_publishes_tutorial_manager_methods(env: &WowLuaEnv) {

    for method in TUTORIAL_MANAGER_METHODS {
        let kind: String = env
            .eval(&format!("return type(TutorialManager.{method})"))
            .unwrap_or_else(|err| panic!("TutorialManager.{method} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "TutorialManager.{method} must be a function — declared in \
             Blizzard_TutorialManager.lua. Tutorial/Watcher pair \
             symmetry: Add/Remove/Get/Queue/Finished/Shutdown for \
             tutorials; Add/Remove/Get/Start/Stop/Shutdown for watchers"
        );
    }
}
}

prefork_full_ui_case! {
fn explicit_load_publishes_class_tutorial_base(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(Class_TutorialBase)")
        .expect("Class_TutorialBase probe");
    assert_eq!(
        kind, "table",
        "Class_TutorialBase must be a table — declared at \
         Blizzard_TutorialBase.lua:4 via `class(\"TutorialBase\")` from \
         the middleclass library at Blizzard_SharedXML/middleclass.lua. \
         Provides the OO base for every concrete Tutorial/Watcher (e.g. \
         the New Player Experience extends this via class hierarchy). \
         Has IsDebugging + IsGlobalEnabled static fields and \
         GlobalEnable/GlobalDisable/MakeExclusive/Debug/ColorDebugText \
         static methods"
    );
}
}

prefork_full_ui_case! {
fn explicit_load_publishes_debug_tutorials_function(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(DebugTutorials)")
        .expect("DebugTutorials probe");
    assert_eq!(
        kind, "function",
        "DebugTutorials must be a global function — declared at \
         Blizzard_TutorialManager.lua:224. Used as a slash-command \
         entrypoint: `/run DebugTutorials(true)` flips Class_TutorialBase \
         debug logging for live troubleshooting"
    );
}
}

prefork_full_ui_case! {
fn explicit_load_creates_named_non_virtual_frames(env: &WowLuaEnv) {

    for frame in NAMED_NON_VIRTUAL_FRAMES {
        let exists: bool = env
            .eval(&format!("return {frame} ~= nil"))
            .unwrap_or_else(|err| panic!("{frame} probe failed: {err}"));
        assert!(
            exists,
            "{frame} must exist as a named global after XML load. \
             TutorialMainFrame_Frame inherits TutorialMainFrameTemplate \
             (the visible tutorial popup); TutorialSingleKey_Frame + \
             TutorialDoubleKey_Frame are key-binding prompt instances. \
             TutorialDragOriginFrame + TutorialDragTargetFrame + \
             TutorialDragAnimationFrame back the drag-cursor effect \
             (TutorialDragButton:Show animates a cursor between the \
             origin and target slots)"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_emits_no_addon_specific_errors(env: &WowLuaEnv) {

    let errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let addon_specific: Vec<&String> = errors
        .iter()
        .filter(|e| e.contains("Blizzard_TutorialManager/"))
        .collect();

    assert!(
        addon_specific.is_empty(),
        "Full game-screen load with TutorialManager in dependency \
         order must emit zero TutorialManager-body errors. The 9 body \
         files include 3 XML files with virtual templates that must \
         register cleanly; the lua files must tolerate unknown \
         C_TutorialState/C_QuestLog/etc. stubs without raising. \
         Found: {addon_specific:?}"
    );
}
}
