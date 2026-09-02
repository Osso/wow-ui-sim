use std::path::PathBuf;

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

fn tutorials_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_Tutorials")
}

fn tutorials_toc() -> PathBuf {
    tutorials_dir().join("Blizzard_Tutorials.toc")
}

const GLUE_SCREENS: &[ScreenKind] = &[
    ScreenKind::Login,
    ScreenKind::CharacterSelect,
    ScreenKind::CharacterCreate,
];

const TOC_REQUIRED_DEPS: &[&str] = &[
    "middleclass",
    "Blizzard_TutorialManager",
    "Blizzard_UIFrameManager",
];

const MODULE_TABLES: &[&str] = &[
    "GameTutorials",
    "BagTutorialQueue",
    "BagTutorialHelpTipKeys",
];

const MIXINS: &[&str] = &[
    "TutorialStateMachineMixin",
    "StateMachineBasedTutorialMixin",
    "HelpTipStateMachineBasedTutorialMixin",
    "BagTutorialBaseMixin",
    "RPETutorialInterruptMixin",
];

const FREE_FUNCTIONS: &[&str] = &[
    "AddDragonridingTutorials",
    "AddSpecAndTalentTutorials",
    "AddFrameTutorials",
    "AddEvokerTutorials",
    "AddPerksProgramTutorials",
    "AddRPETutorials",
    "CanShowProfessionEquipmentTutorial",
    "PlayerHasPrimaryProfession",
];

const TUTORIAL_CLASSES: &[&str] = &[
    "Class_AssistedHighlight_RPE_Watcher",
    "Class_ChangeSpec",
    "Class_ChangeSpec_NPE",
    "Class_DragonRidingWatcher",
    "Class_Dragonriding_RPE_Watcher",
    "Class_EquipProfessionGear",
    "Class_EvokerEmpoweredSpellWatcher",
    "Class_EvokerEssenceWatcher",
    "Class_EvokerLowHealthWatcher",
    "Class_FirstProfessionTutorial",
    "Class_FirstProfessionWatcher",
    "Class_InteractKeyNoKeybindWatcher",
    "Class_InteractKeyWatcher",
    "Class_Interrupt_RPE_Watcher",
    "Class_PerksProgramActivitiesOpenWatcher",
    "Class_PerksProgramActivitiesPromptWatcher",
    "Class_PerksProgramActivitiesTrackingWatcher",
    "Class_PerksProgramFreezeItemWatcher",
    "Class_PerksProgramOverwriteFrozenItemWatcher",
    "Class_PerksProgramProductPurchased",
    "Class_PerksProgramProductSelectedWatcher",
    "Class_ProfessionGearCheckingService",
    "Class_ProfessionInventoryWatcher",
    "Class_StarterTalentWatcher",
    "Class_StarterTalentWatcher_NPE",
    "Class_TalentPoints",
];

const NAMED_NON_VIRTUAL_FRAMES: &[&str] = &["RPETutorialInterrupt_Frame"];

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
    let resolved = find_toc_file(&tutorials_dir()).expect("Tutorials TOC resolves");
    assert_eq!(
        resolved,
        tutorials_toc(),
        "Bare TOC — no flavor suffix. Resolved via the bare-TOC path \
         in find_toc_file at src/loader/mod.rs:65-95"
    );
}

#[test]
fn toc_is_eager_with_three_required_deps() {
    let toc = TocFile::from_file(&tutorials_toc()).expect("TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "No `## LoadOnDemand` directive → eagerly loaded on Game. The \
         tutorials addon must be live at world entry to wire \
         GameTutorials:Initialize before TutorialManager.TutorialsEnabled \
         fires"
    );

    let deps = toc.dependencies();
    assert_eq!(
        deps.len(),
        TOC_REQUIRED_DEPS.len(),
        "Repeated `## Dep:` lines are merged by `dependencies()`. Expected \
         exactly {} entries. Got: {:?}",
        TOC_REQUIRED_DEPS.len(),
        deps
    );
    for expected in TOC_REQUIRED_DEPS {
        assert!(
            deps.iter().any(|d| d == expected),
            "TOC must declare `{expected}` via a repeated `## Dep:` line. \
             TutorialManager supplies tutorial orchestration and UIFrameManager \
             supplies the in-game frame-management surface. Got: {deps:?}"
        );
    }

    assert!(toc.optional_deps().is_empty());
    assert!(toc.saved_variables().is_empty());
    assert!(toc.saved_variables_per_character().is_empty());
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(toc.default_enabled());
}

#[test]
fn allow_load_absent_implies_game_only() {
    let toc = TocFile::from_file(&tutorials_toc()).expect("TOC parses");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "AllowLoad absent → toc.rs:305-313 None branch defaults to \
         Game-only. Tutorials never run on glue screens"
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
fn allow_load_game_type_standard_is_not_restricted() {
    let toc = TocFile::from_file(&tutorials_toc()).expect("TOC parses");

    assert!(
        !toc.is_game_type_restricted(),
        "`## AllowLoadGameType: standard` lists `standard` which is \
         recognised as a non-restricting flavor at toc.rs:294-302 \
         (standard|mainline). On a mainline build the addon loads; only \
         classic/plunderstorm-style values would gate it out"
    );
}

#[test]
fn toc_raw_bytes_pin_five_directives_and_fifteen_body_files() {
    let raw = std::fs::read_to_string(tutorials_toc()).expect("TOC reads utf-8");

    let expected_lines = [
        "## Title: Blizzard_Tutorials",
        "## Dep: middleclass",
        "## Dep: Blizzard_TutorialManager",
        "## Dep: Blizzard_UIFrameManager",
        "## AllowLoadGameType: standard",
        "StateMachineUtil.lua",
        "StateMachineTutorialUtil.lua",
        "BagTutorialUtil.lua",
        "Blizzard_Tutorials_Classes.lua",
        "Blizzard_Tutorials_Dracthyr.lua",
        "Blizzard_Tutorials_Dragonriding.lua",
        "Blizzard_Tutorials_RPE.lua",
        "Blizzard_TutorialReagentBag.lua",
        "Blizzard_Tutorials_Professions.lua",
        "Blizzard_Tutorials_Frame_Tutorials.lua",
        "Blizzard_Tutorials_PerksProgram.lua",
        "Blizzard_TutorialSupertrack.lua",
        "Blizzard_Tutorials_Managed.lua",
        "Blizzard_Tutorials_Managed.xml",
        "Blizzard_Tutorials.lua",
    ];

    for line in expected_lines {
        assert!(
            raw.contains(line),
            "Raw TOC must pin `{line}` — body order is dependency-driven: \
             3 utility files first (StateMachineUtil → \
             StateMachineTutorialUtil → BagTutorialUtil establish the \
             mixin chain TutorialStateMachineMixin ← \
             StateMachineBasedTutorialMixin ← \
             HelpTipStateMachineBasedTutorialMixin), then content files \
             that declare Class_* subclasses + free Add*Tutorials() \
             builders, then the Managed pair (lua before xml is \
             intentional — the lua defines RPETutorialInterruptMixin \
             which the xml's `mixin=` attribute resolves), and finally \
             Blizzard_Tutorials.lua which calls GameTutorials:Initialize \
             after every helper is published"
        );
    }

    assert!(!raw.contains("## Author"));
    assert!(!raw.contains("## LoadOnDemand"));
    assert!(!raw.contains("## AllowLoad: "));
    assert!(!raw.contains("## DefaultState"));
    assert!(!raw.contains("## OptionalDeps"));
    assert!(!raw.contains("## Dependencies:"));
    assert!(!raw.contains("## SavedVariables"));
    assert!(!raw.contains("## UseSecureEnvironment"));
    assert!(!raw.contains("## LoadFirst"));
    assert!(!raw.contains("## Version"));
    assert!(!raw.contains("[Family]"));
}

#[test]
fn appears_in_game_eager_discovery() {
    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let found = addons.iter().any(|(name, _)| name == "Blizzard_Tutorials");
    assert!(
        found,
        "Blizzard_Tutorials must appear in Game eager discovery — \
         non-LoD addon with no AllowLoad (Game-only) and \
         AllowLoadGameType:standard which a mainline build does NOT \
         restrict. Has one downstream consumer: Blizzard_HousingTutorials \
         declares `## Dependencies: Blizzard_Tutorials`"
    );
}

#[test]
fn absent_from_glue_screens_eager_discovery() {
    for screen in GLUE_SCREENS {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), *screen);
        let found = addons.iter().any(|(name, _)| name == "Blizzard_Tutorials");
        assert!(
            !found,
            "Blizzard_Tutorials must NOT appear on {screen:?} — \
             AllowLoad absent → Game-only via toc.rs:305-313 None branch, \
             checked at loader/mod.rs:527 BEFORE pool partitioning"
        );
    }
}

#[test]
fn tutorial_manager_dep_directory_exists_on_disk() {
    for dep in TOC_REQUIRED_DEPS {
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
fn full_game_load_publishes_module_tables(env: &WowLuaEnv) {

    for module in MODULE_TABLES {
        let kind: String = env
            .eval(&format!("return type({module})"))
            .unwrap_or_else(|err| panic!("{module} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{module} must be a global table — declared at module-load. \
             GameTutorials is the orchestrator singleton in \
             Blizzard_Tutorials.lua that wires \
             TutorialManager.TutorialsEnabled/Disabled/Init callbacks. \
             BagTutorialQueue is the BagTutorialUtil.lua FIFO queue for \
             chained help-tip bag tutorials. BagTutorialHelpTipKeys is \
             the OpenBagsInfo/ItemInfo enum keying into a derived \
             tutorial's helpTipInfos table"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_publishes_mixins(env: &WowLuaEnv) {

    for mixin in MIXINS {
        let kind: String = env
            .eval(&format!("return type({mixin})"))
            .unwrap_or_else(|err| panic!("{mixin} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{mixin} must be a global table. Inheritance chain is built \
             via CreateFromMixins (shallow copy): \
             TutorialStateMachineMixin (StateMachineUtil.lua, base FSM \
             with AddState/BeginState/Deactivate) ← \
             StateMachineBasedTutorialMixin (adds CVarBitfield-backed \
             completion via SetTutorialFlagType + MarkTutorialComplete) \
             ← HelpTipStateMachineBasedTutorialMixin (adds Init() + \
             ShowHelpTipByState/HideHelpTipByState wrappers around \
             HelpTip:Show/Hide). BagTutorialBaseMixin extends \
             StateMachineBasedTutorialMixin with a 4-state bag-opening \
             machine. RPETutorialInterruptMixin extends \
             UIFrameManager_ManagedFrameMixin and is the only mixin used \
             as an XML `mixin=` attribute (on \
             RPETutorialInterrupt_Frame)"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_publishes_class_subclasses(env: &WowLuaEnv) {

    for class in TUTORIAL_CLASSES {
        let kind: String = env
            .eval(&format!("return type({class})"))
            .unwrap_or_else(|err| panic!("{class} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{class} must be a middleclass class table. All 26 concrete \
             tutorial/watcher subclasses are built via \
             `class(\"<Name>\", Class_TutorialBase)` (or in 2 cases via \
             a deeper parent: Class_ChangeSpec_NPE extends \
             Class_ChangeSpec, Class_StarterTalentWatcher_NPE extends \
             Class_StarterTalentWatcher) — middleclass walks the \
             super-chain at lookup time"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_publishes_free_functions(env: &WowLuaEnv) {

    for func in FREE_FUNCTIONS {
        let kind: String = env
            .eval(&format!("return type({func})"))
            .unwrap_or_else(|err| panic!("{func} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "{func} must be a global function. The Add*Tutorials() \
             builders are dispatched from \
             GameTutorials:OnTutorialsEnabled/OnTutorialsInit and \
             instantiate the corresponding Class_* subclasses via \
             `Class_X:new()` then call \
             `TutorialManager:AddTutorial/AddWatcher`. \
             CanShowProfessionEquipmentTutorial + \
             PlayerHasPrimaryProfession are gating predicates checked \
             before Class_FirstProfessionTutorial / \
             Class_EquipProfessionGear instantiation"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_creates_named_non_virtual_frames(env: &WowLuaEnv) {

    for frame in NAMED_NON_VIRTUAL_FRAMES {
        let exists: bool = env
            .eval(&format!("return {frame} ~= nil"))
            .unwrap_or_else(|err| panic!("{frame} probe failed: {err}"));
        assert!(
            exists,
            "{frame} must exist as a named global after XML load. \
             Blizzard_Tutorials_Managed.xml defines a single named \
             non-virtual top-level Frame: RPETutorialInterrupt_Frame \
             (inherits TutorialMainFrameTemplate, mixin = \
             RPETutorialInterruptMixin, KeyValues binds \
             frameType=Enum.UIFrameType.InterruptTutorial — the \
             UIFrameManager registers it under that frameType so \
             ShowFrame/HideFrame route to it). The mixin's OnLoad \
             chains UIFrameManager_ManagedFrameMixin.OnLoad + \
             TutorialMainFrameMixin.OnLoad"
        );
    }
}
}

prefork_full_ui_case! {
fn game_tutorials_initialize_method_exists(env: &WowLuaEnv) {

    let kind: String = env
        .eval("return type(GameTutorials.Initialize)")
        .expect("GameTutorials.Initialize probe");
    assert_eq!(
        kind, "function",
        "GameTutorials:Initialize is the entrypoint at \
         Blizzard_Tutorials.lua:3-7. Self-invoked at end-of-file \
         (lua:48). Subscribes to 3 EventRegistry callbacks that \
         TutorialManager fires during its lifecycle: \
         TutorialManager.TutorialsEnabled, \
         TutorialManager.TutorialsDisabled, \
         TutorialManager.TutorialsInit"
    );
}
}

prefork_full_ui_case! {
fn full_game_load_emits_no_addon_specific_errors(env: &WowLuaEnv) {

    let errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let addon_specific: Vec<&String> = errors
        .iter()
        .filter(|e| e.contains("Blizzard_Tutorials/"))
        .collect();

    assert!(
        addon_specific.is_empty(),
        "Full game-screen load with Blizzard_Tutorials in dependency \
         order must emit zero Tutorials-body errors. The 16 body files \
         include 1 XML file (managed frame), 26 Class_ subclass \
         declarations, 8 Add*Tutorials() free builders, and several \
         module-load self-registrations (Blizzard_TutorialReagentBag \
         calls TutorialManager:CheckHasCompletedFrameTutorial; \
         Blizzard_TutorialSupertrack registers TutorialManager.* \
         callbacks + invokes CheckBeginTutorial directly). All of those \
         must tolerate the simulator's Enum/CVar/C_AddOns/C_Item stubs \
         without raising. Found: {addon_specific:?}"
    );
}
}
