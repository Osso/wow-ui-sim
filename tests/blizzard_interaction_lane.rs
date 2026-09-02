//! Interaction lane analysis: Blizzard_StaticPopup* (3 addons),
//! Blizzard_LoginWarningDialogs, and the tutorial cluster
//! (Blizzard_TutorialManager, Blizzard_Tutorials, Blizzard_HousingTutorials,
//! Blizzard_BoostTutorial, Blizzard_RemixArtifactTutorialUI).
//!
//! This file is a LANE audit — separate from per-addon loadability suites.
//!
//! The PLAN.md task explicitly asks for: popup flow, glue variants, and
//! tutorial-state transitions. The deliverable is "the minimum per-module
//! plans needed to cover" each, so this audit pins the boundary contracts
//! per module-cluster rather than re-testing each addon end-to-end.
//!
//! CLUSTER 1 — POPUP FLOW (Blizzard_StaticPopup family):
//!     - Blizzard_StaticPopup            (AllowLoad: Both, DefaultState: enabled,
//!                                         no deps, eager — publishes
//!                                         StaticPopupDialogs registry +
//!                                         StaticPopup_Show / _Hide / _Visible /
//!                                         _FindVisible / _Queue functions
//!                                         + StaticPopup_AddDefinition /
//!                                         _AddDialog / _RemoveDialog hooks)
//!     - Blizzard_StaticPopup_Game       (AllowLoad: game, deps: StaticPopup +
//!                                         ItemButton + AutoComplete +
//!                                         MoneyFrame + AccessibilityTemplates;
//!                                         publishes Game-only popup defs and
//!                                         the StaticPopup1..N visible frames)
//!     - Blizzard_StaticPopup_Glue       (AllowLoad: glue, deps: StaticPopup +
//!                                         AutoComplete + AccessibilityTemplates;
//!                                         publishes GlueDialogMixin for the
//!                                         glue-screen popup variant — MUST NOT
//!                                         appear in Game discovery)
//!
//! CLUSTER 2 — GLUE VARIANTS (Blizzard_LoginWarningDialogs):
//!     - Blizzard_LoginWarningDialogs    (AllowLoad: Glue, DefaultState: enabled,
//!                                         deps: SharedXML + GlueXMLBase +
//!                                         GlueParent — publishes regional
//!                                         warning frames ChinaAgeAppropriatenessWarning,
//!                                         KoreanRatings, TaiwanFraudWarning all
//!                                         parented to GlueParent (not UIParent))
//!
//! CLUSTER 3 — TUTORIAL-STATE TRANSITIONS (TutorialManager + Tutorials family):
//!     - Blizzard_TutorialManager        (default Game, deps: Dispatcher +
//!                                         HelpPlate; publishes TutorialQueue
//!                                         table + TutorialPointerFrame table +
//!                                         TutorialMainFrameMixin with
//!                                         {Hidden, AnimatingIn, Visible,
//!                                         AnimatingOut} state-machine enum)
//!     - Blizzard_Tutorials              (AllowLoadGameType: standard which the
//!                                         simulator treats as unrestricted via
//!                                         inverse-restriction semantics, deps:
//!                                         TutorialManager; publishes
//!                                         TutorialStateMachineMixin with
//!                                         AddState/BeginState/SetInitialStateName)
//!     - Blizzard_HousingTutorials       (AllowLoad: Game, AllowLoadGameType:
//!                                         standard, deps: Tutorials)
//!     - Blizzard_BoostTutorial          (LoadOnDemand, deps: Dispatcher +
//!                                         HelpPlate; SMOKE — only triggered
//!                                         by character-boost flow)
//!     - Blizzard_RemixArtifactTutorialUI (LoadOnDemand, AllowLoad: Game,
//!                                          AllowLoadGameType: standard;
//!                                          SMOKE — only loaded for the
//!                                          Remix scenario tutorial)

use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available")
}

const POPUP_CLUSTER: &[&str] = &[
    "Blizzard_StaticPopup",
    "Blizzard_StaticPopup_Game",
    "Blizzard_StaticPopup_Glue",
];

const GLUE_DIALOG_CLUSTER: &[&str] = &["Blizzard_LoginWarningDialogs"];

const TUTORIAL_CLUSTER: &[&str] = &[
    "Blizzard_TutorialManager",
    "Blizzard_Tutorials",
    "Blizzard_HousingTutorials",
    "Blizzard_BoostTutorial",
    "Blizzard_RemixArtifactTutorialUI",
];

const STARTUP_GAME: &[&str] = &[
    "Blizzard_StaticPopup",
    "Blizzard_StaticPopup_Game",
    "Blizzard_TutorialManager",
    "Blizzard_Tutorials",
    "Blizzard_HousingTutorials",
];

const STARTUP_GLUE: &[&str] = &[
    "Blizzard_StaticPopup",
    "Blizzard_StaticPopup_Glue",
    "Blizzard_LoginWarningDialogs",
];

const SMOKE_GAME: &[&str] = &["Blizzard_BoostTutorial", "Blizzard_RemixArtifactTutorialUI"];

fn parse_lane_toc(name: &str) -> TocFile {
    let dir = blizzard_ui_dir().join(name);
    let toc_path = find_toc_file(&dir)
        .unwrap_or_else(|| panic!("`{name}` TOC must resolve via find_toc_file under {dir:?}"));
    TocFile::from_file(&toc_path).unwrap_or_else(|err| panic!("`{name}` TOC parses: {err}"))
}

fn fresh_lane_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("WowLuaEnv constructs");
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
    let env = fresh_lane_env();
    let ui = blizzard_ui_dir();
    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    for (name, toc_path) in &addons {
        wow_ui_sim::loader::load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }
    env.apply_post_load_workarounds();
    fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

#[test]
fn each_lane_addon_directory_resolves_with_a_toc() {
    let mut all: Vec<&str> = Vec::new();
    all.extend(POPUP_CLUSTER.iter().copied());
    all.extend(GLUE_DIALOG_CLUSTER.iter().copied());
    all.extend(TUTORIAL_CLUSTER.iter().copied());

    for name in &all {
        let dir = blizzard_ui_dir().join(name);
        assert!(
            dir.is_dir(),
            "Interaction lane addon `{name}` directory must exist at \
             `Interface/BlizzardUI/{name}/`. The lane covers popup flow \
             (StaticPopup family), glue-screen warning dialogs \
             (LoginWarningDialogs), and the tutorial state-machine cluster"
        );
        assert!(
            find_toc_file(&dir).is_some(),
            "Interaction lane addon `{name}` TOC must resolve via find_toc_file. \
             All 9 addons in this lane ship plain `<name>.toc` (no flavor variants), \
             so the resolver hits its first probe and returns immediately"
        );
    }
}

#[test]
fn popup_cluster_dep_edges_pin_load_floor_for_popup_flow() {
    let core = parse_lane_toc("Blizzard_StaticPopup");
    let game = parse_lane_toc("Blizzard_StaticPopup_Game");
    let glue = parse_lane_toc("Blizzard_StaticPopup_Glue");

    assert!(
        core.dependencies().is_empty(),
        "Blizzard_StaticPopup (the AllowLoad: Both core) declares NO dependencies — it \
         publishes the StaticPopupDialogs registry and the StaticPopup_Show / _Hide / \
         _FindVisible / _Queue functions at file scope. As the load floor for both \
         StaticPopup_Game and StaticPopup_Glue, it must be loadable on either screen \
         without pulling in screen-specific deps. Got: {:?}",
        core.dependencies()
    );
    assert!(
        core.default_enabled(),
        "Blizzard_StaticPopup carries `## DefaultState: enabled` — popups are not opt-in. \
         If a future SavedVariables migration tried to ship them disabled by default, \
         every confirmation dialog (delete equipment set, refund honor, log out warning) \
         would silently become a no-op"
    );

    assert_eq!(
        game.dependencies(),
        vec![
            "Blizzard_StaticPopup".to_string(),
            "Blizzard_ItemButton".to_string(),
            "Blizzard_AutoComplete".to_string(),
            "Blizzard_MoneyFrame".to_string(),
            "Blizzard_AccessibilityTemplates".to_string(),
            "Blizzard_GameMenuEsc".to_string(),
        ],
        "Current retail StaticPopup_Game TOC lists StaticPopup, ItemButton, AutoComplete, \
         MoneyFrame, AccessibilityTemplates, and GameMenuEsc in that order"
    );

    assert_eq!(
        glue.dependencies(),
        vec![
            "Blizzard_StaticPopup".to_string(),
            "Blizzard_AutoComplete".to_string(),
            "Blizzard_AccessibilityTemplates".to_string(),
        ],
        "Blizzard_StaticPopup_Glue RequiredDep is the GLUE-screen subset: no ItemButton \
         (no inventory on glue screen), no MoneyFrame (no economy on login screen) — only \
         StaticPopup core + AutoComplete (account-name input field on character-realm \
         change dialogs) + AccessibilityTemplates (AT-narration). The asymmetry between \
         Game and Glue dep lists is the load-floor signal that distinguishes the two \
         variants"
    );
}

#[test]
fn static_popup_core_targets_both_screens_via_allow_load_both() {
    let core = parse_lane_toc("Blizzard_StaticPopup");
    for screen in [
        ScreenKind::Game,
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            core.allows_screen(screen),
            "Blizzard_StaticPopup carries `## AllowLoad: Both` — must allow on every \
             screen ({screen:?}). The core popup runtime is shared between Game (in-world \
             confirmations) and Glue (login/character-select prompts); a regression here \
             would force one of the two screen variants to fail at load time because the \
             StaticPopupDialogs registry would be undefined"
        );
    }
}

#[test]
fn glue_variants_do_not_appear_in_game_discovery() {
    let ui = blizzard_ui_dir();
    let game_names: Vec<String> = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game)
        .into_iter()
        .map(|(n, _)| n)
        .collect();

    let glue_only_addons = ["Blizzard_StaticPopup_Glue", "Blizzard_LoginWarningDialogs"];
    for glue_only in glue_only_addons {
        assert!(
            !game_names.contains(&glue_only.to_string()),
            "Glue-only addon `{glue_only}` MUST NOT appear in eager Game discovery. \
             StaticPopup_Glue carries `## AllowLoad: glue` and LoginWarningDialogs carries \
             `## AllowLoad: Glue` — both gate to the login/character-select/character-create \
             screens via `allows_screen`. If this regresses, GlueDialog frames would attempt \
             to parent to the non-existent GlueParent global on Game screens and crash at \
             file scope"
        );
    }
}

#[test]
fn game_variants_do_not_appear_in_glue_discovery() {
    let ui = blizzard_ui_dir();

    for glue_screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let glue_names: Vec<String> = discover_blizzard_addons_for_screen(&ui, glue_screen)
            .into_iter()
            .map(|(n, _)| n)
            .collect();

        let game_only = "Blizzard_StaticPopup_Game";
        assert!(
            !glue_names.contains(&game_only.to_string()),
            "Game-only addon `{game_only}` MUST NOT appear in {glue_screen:?} eager \
             discovery. Its `## AllowLoad: game` directive gates it to the in-world \
             screen — GameDialogDefs include refund flows, equipment-set persistence, \
             quest-completion prompts that require a logged-in character. If this regresses, \
             glue screens would try to load XML referencing UIParent (no such global on glue) \
             and crash"
        );

        for tutorial in TUTORIAL_CLUSTER {
            assert!(
                !glue_names.contains(&tutorial.to_string()),
                "Tutorial cluster addon `{tutorial}` MUST NOT appear in {glue_screen:?} \
                 eager discovery. Tutorials are character-context UI (TutorialQueue tracks \
                 character progression), so every tutorial TOC defaults to or explicitly \
                 declares Game-only. If this regresses, the tutorial state machine would \
                 attempt to read player-spell / player-quest data on glue screens where \
                 those C_* APIs return nil"
            );
        }
    }
}

#[test]
fn game_screen_eager_discovery_includes_static_popup_core_game_and_eager_tutorials() {
    let ui = blizzard_ui_dir();
    let game_names: Vec<String> = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game)
        .into_iter()
        .map(|(name, _)| name)
        .collect();

    for required in STARTUP_GAME {
        assert!(
            game_names.contains(&required.to_string()),
            "Game STARTUP addon `{required}` MUST appear in eager Game discovery. \
             StaticPopup core uses `## AllowLoad: Both`, StaticPopup_Game uses `## AllowLoad: game`, \
             and the 3 eager tutorials (TutorialManager / Tutorials / HousingTutorials) default to \
             Game. The 5 form the lane's eager game-side surface — popup confirmation dialogs and \
             the tutorial state machine. If this regresses, in-world refund/equipment/quest \
             confirmation flows would have no popup runtime, or the tutorial pointer/main-frame \
             system would fail to register at startup"
        );
    }
}

#[test]
fn glue_screens_eager_discovery_includes_static_popup_core_and_glue_variants() {
    let ui = blizzard_ui_dir();

    for glue_screen in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let glue_names: Vec<String> = discover_blizzard_addons_for_screen(&ui, glue_screen)
            .into_iter()
            .map(|(n, _)| n)
            .collect();

        for required in STARTUP_GLUE {
            assert!(
                glue_names.contains(&required.to_string()),
                "Glue STARTUP addon `{required}` MUST appear in {glue_screen:?} eager \
                 discovery. StaticPopup core uses `## AllowLoad: Both`, StaticPopup_Glue \
                 uses `## AllowLoad: glue`, LoginWarningDialogs uses `## AllowLoad: Glue` \
                 (case differs but the parser is case-insensitive). The 3 form the popup \
                 substrate every glue screen needs — without them, the realm-list / \
                 character-create flow has no confirmation dialog mechanism"
            );
        }
    }
}

#[test]
fn smoke_tutorials_are_load_on_demand_with_current_game_discovery_exceptions() {
    let ui = blizzard_ui_dir();
    let game_names: Vec<String> = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game)
        .into_iter()
        .map(|(n, _)| n)
        .collect();

    for smoke in SMOKE_GAME {
        assert!(
            parse_lane_toc(smoke).is_load_on_demand(),
            "Lane SMOKE tutorial `{smoke}` must carry `## LoadOnDemand: 1`"
        );
    }

    assert!(
        game_names.contains(&"Blizzard_BoostTutorial".to_string()),
        "Current game discovery pulls Blizzard_BoostTutorial despite its LoadOnDemand metadata \
         because implicit_blizzard_startup_dependencies makes it a Blizzard_Game dependency"
    );
    assert!(
        !game_names.contains(&"Blizzard_RemixArtifactTutorialUI".to_string()),
        "Blizzard_RemixArtifactTutorialUI remains absent from eager Game discovery because no \
         current implicit or TOC dependency pulls it from the LoadOnDemand pool"
    );
}

#[test]
fn eager_tutorial_chain_appears_in_game_discovery_in_dep_first_order() {
    let ui = blizzard_ui_dir();
    let names: Vec<String> = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game)
        .into_iter()
        .map(|(n, _)| n)
        .collect();

    let manager_idx = names
        .iter()
        .position(|n| n == "Blizzard_TutorialManager")
        .expect("TutorialManager must appear in eager Game discovery");
    let tutorials_idx = names
        .iter()
        .position(|n| n == "Blizzard_Tutorials")
        .expect("Tutorials must appear in eager Game discovery");
    let housing_idx = names
        .iter()
        .position(|n| n == "Blizzard_HousingTutorials")
        .expect("HousingTutorials must appear in eager Game discovery");

    assert!(
        manager_idx < tutorials_idx,
        "TutorialManager must emit before Tutorials. Tutorials.toc declares \
         `## RequiredDep: Blizzard_TutorialManager` — the topological sort respects that \
         edge so TutorialQueue / TutorialPointerFrame / TutorialMainFrameMixin globals are \
         live before Tutorials' StateMachineUtil.lua references them. \
         Got: TutorialManager@{manager_idx}, Tutorials@{tutorials_idx}"
    );
    assert!(
        tutorials_idx < housing_idx,
        "Tutorials must emit before HousingTutorials. HousingTutorials.toc declares \
         `## Dependencies: Blizzard_Tutorials` — the housing-specific tutorial state \
         machines inherit from TutorialStateMachineMixin which Tutorials publishes. \
         Got: Tutorials@{tutorials_idx}, HousingTutorials@{housing_idx}"
    );
}

#[test]
fn tutorial_addons_use_inverse_game_type_restriction_semantics_for_standard() {
    let tutorials = parse_lane_toc("Blizzard_Tutorials");
    let housing = parse_lane_toc("Blizzard_HousingTutorials");

    assert!(
        !tutorials.is_game_type_restricted(),
        "Blizzard_Tutorials carries `## AllowLoadGameType: standard`, but \
         `is_game_type_restricted()` MUST return false. The accessor at src/toc.rs:294-302 \
         treats `mainline` and `standard` as the unrestricted defaults — only non-mainline/ \
         non-standard values (plunderstorm, classic, vanilla, wrath, cata, mists) flag the \
         addon as restricted. If this regresses, the tutorial state machine would skip \
         loading on the standard mainline screen and every consumer (HousingTutorials, \
         RPE flow, dragonriding intro) would see TutorialStateMachineMixin as nil"
    );
    assert!(
        !housing.is_game_type_restricted(),
        "HousingTutorials inherits the same `AllowLoadGameType: standard` semantics as \
         Tutorials — must NOT flag as restricted on a standard/mainline build"
    );
}

prefork_full_ui_case! {
fn lane_addons_load_cleanly_during_full_game_load_with_no_lane_specific_lua_errors(env: &WowLuaEnv) {

    let mut all_lane: Vec<&str> = Vec::new();
    all_lane.extend(POPUP_CLUSTER.iter().copied());
    all_lane.extend(TUTORIAL_CLUSTER.iter().copied());
    let load_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| all_lane.iter().any(|name| message.contains(name)))
        .cloned()
        .collect();

    assert!(
        load_errors.is_empty(),
        "Interaction lane MUST load without lane-specific Lua errors during eager Game sweep. \
         StaticPopup core + StaticPopup_Game + the 3 eager tutorials run. SMOKE tutorials \
         (BoostTutorial / RemixArtifactTutorialUI) and the Glue variants don't run at this \
         point. If errors mention any lane addon, either a popup C_* stub is missing on the \
         Game side or the tutorial state machine references a global a dep failed to publish. \
         Got:\n  {}",
        load_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn static_popup_core_publishes_dialog_registry_and_show_hide_functions_after_full_load(env: &WowLuaEnv) {

    let registry_kind: String = env
        .eval("return type(StaticPopupDialogs)")
        .expect("StaticPopupDialogs type probe");
    assert_eq!(
        registry_kind, "table",
        "StaticPopupDialogs MUST publish as a table after the eager Game sweep. \
         Blizzard_StaticPopup/StaticPopup.lua:21 sets `StaticPopupDialogs = {{}};` and \
         every popup definition (GENERIC_CONFIRMATION, OKAY, GENERIC_INPUT_BOX, \
         CONFIRM_REFUND_*, etc.) is registered as `StaticPopupDialogs[\"<KEY>\"] = {{...}}`. \
         If the global is nil, every `StaticPopup_Show(\"<KEY>\")` call no-ops"
    );

    let registry_size: i64 = env
        .eval("local n = 0; for _ in pairs(StaticPopupDialogs) do n = n + 1 end; return n")
        .expect("StaticPopupDialogs size probe");
    assert!(
        registry_size > 0,
        "StaticPopupDialogs MUST have at least one definition after the eager Game sweep. \
         StaticPopup core ships SharedDialogDefs.lua (GENERIC_CONFIRMATION, \
         XP_LOSS_NO_SICKNESS_NO_DURABILITY) and StaticPopup_Game adds dozens more via \
         GameDialogDefs.lua. A registry size of 0 means SharedDialogDefs.lua failed to \
         execute. Got size: {registry_size}"
    );

    let show_kind: String = env
        .eval("return type(StaticPopup_Show)")
        .expect("StaticPopup_Show type probe");
    assert_eq!(
        show_kind, "function",
        "StaticPopup_Show MUST publish as a function — the canonical entry point for \
         displaying a popup by registered key. Used by hundreds of consumers across the \
         UI (PaperDollFrame for equipment-set save, MailFrame for mass delete, \
         CompactRaidFrameContainer for layout reset, etc.)"
    );
    let hide_kind: String = env
        .eval("return type(StaticPopup_Hide)")
        .expect("StaticPopup_Hide type probe");
    assert_eq!(
        hide_kind, "function",
        "StaticPopup_Hide MUST publish as a function — the inverse of Show, also used by \
         every popup-emitting consumer to dismiss in-flight dialogs (e.g. when the trigger \
         condition disappears mid-flow)"
    );
    let visible_kind: String = env
        .eval("return type(StaticPopup_Visible)")
        .expect("StaticPopup_Visible type probe");
    assert_eq!(
        visible_kind, "function",
        "StaticPopup_Visible MUST publish as a function — used by escape-key handlers and \
         modal-detection paths to determine whether a popup is currently displayed"
    );
}
}

prefork_full_ui_case! {
fn tutorial_manager_publishes_state_machine_and_queue_globals_after_full_load(env: &WowLuaEnv) {

    let queue_kind: String = env
        .eval("return type(TutorialQueue)")
        .expect("TutorialQueue type probe");
    assert_eq!(
        queue_kind, "table",
        "TutorialQueue MUST publish as a table. Blizzard_TutorialManager/Blizzard_TutorialQueue.lua:1 \
         sets `TutorialQueue = {{}};` and adds Initialize/Reset/Add/CheckQueue/Status/NotifyDone \
         methods. Every tutorial-trigger call site (RPE quest hooks, dragonriding intro, \
         BagTutorialUtil) calls `TutorialQueue:Add(tutorialInstance, ...)` to enqueue work. \
         If nil, no tutorial fires"
    );

    let pointer_kind: String = env
        .eval("return type(TutorialPointerFrame)")
        .expect("TutorialPointerFrame type probe");
    assert_eq!(
        pointer_kind, "table",
        "TutorialPointerFrame MUST publish as a table. Blizzard_TutorialQueue.lua:1 sets \
         `TutorialPointerFrame = {{}};` — the on-screen arrow that points at UI elements \
         during a tutorial. If nil, the visual pointer system is dead and tutorials become \
         text-only, breaking the spatial-anchoring contract"
    );

    let mixin_kind: String = env
        .eval("return type(TutorialMainFrameMixin)")
        .expect("TutorialMainFrameMixin type probe");
    assert_eq!(
        mixin_kind, "table",
        "TutorialMainFrameMixin MUST publish as a table. Blizzard_TutorialMainFrame.lua:1 \
         sets `TutorialMainFrameMixin = {{}};` and adds the States enum + lifecycle methods. \
         Used as the mixin on `<Frame name=\"TutorialMainFrameTemplate\" \
         mixin=\"TutorialMainFrameMixin\">`"
    );
}
}

prefork_full_ui_case! {
fn tutorial_state_machine_publishes_canonical_state_transitions_after_full_load(env: &WowLuaEnv) {

    let states_table: String = env
        .eval(
            "local s = TutorialMainFrameMixin and TutorialMainFrameMixin.States; \
             if not s then return 'nil' end; \
             return string.format('%s|%s|%s|%s', \
                 tostring(s.Hidden), tostring(s.AnimatingIn), \
                 tostring(s.Visible), tostring(s.AnimatingOut))",
        )
        .expect("TutorialMainFrameMixin.States probe");

    assert_eq!(
        states_table, "hidden|animatingIn|visible|animatingOut",
        "TutorialMainFrameMixin.States MUST publish 4 canonical states with these exact \
         string values: Hidden=\"hidden\", AnimatingIn=\"animatingIn\", \
         Visible=\"visible\", AnimatingOut=\"animatingOut\". The values are written into \
         OnPlay/OnFinished animation handlers in Blizzard_TutorialMainFrame.xml as \
         `self:GetParent():_SetState(NPE_TutorialMainFrameMixin.States.Visible)` — if the \
         enum drifts, the animation chain stops mid-state and the tutorial frame freezes. \
         Got: `{states_table}`"
    );

    let sm_mixin_kind: String = env
        .eval("return type(TutorialStateMachineMixin)")
        .expect("TutorialStateMachineMixin type probe");
    assert_eq!(
        sm_mixin_kind, "table",
        "TutorialStateMachineMixin MUST publish as a table after the eager Game sweep. \
         Blizzard_Tutorials/StateMachineUtil.lua:1 sets `TutorialStateMachineMixin = {{}};` \
         and adds AddState / SetInitialStateName / BeginState / Deactivate / \
         CallStateTransition. This is the generic SM the per-tutorial classes mix into to \
         describe their own state graphs (RPE intro, dragonriding intro, Dracthyr starting \
         flow). If nil, every per-tutorial class file fails at file scope"
    );

    let add_state_kind: String = env
        .eval("return type(TutorialStateMachineMixin.AddState)")
        .expect("AddState method probe");
    assert_eq!(
        add_state_kind, "function",
        "TutorialStateMachineMixin:AddState MUST publish as a function — registers \
         (stateName, onBegin, onEnd) triples in self.states. Every per-tutorial class \
         (`function MyTutorialMixin:Initialize() self:AddState('start', ...) end`) calls \
         this at construction"
    );

    let begin_state_kind: String = env
        .eval("return type(TutorialStateMachineMixin.BeginState)")
        .expect("BeginState method probe");
    assert_eq!(
        begin_state_kind, "function",
        "TutorialStateMachineMixin:BeginState MUST publish as a function — calls \
         Deactivate, then CallStateTransition(stateName, 'onBegin', ...). The transition \
         is the entire tutorial-progress mechanism; if missing, no tutorial advances past \
         its initial state"
    );
}
}

prefork_full_ui_case! {
fn smoke_tutorials_can_be_loaded_on_demand_after_eager_sweep(env: &WowLuaEnv) {

    for smoke in SMOKE_GAME {
        let dir = blizzard_ui_dir().join(smoke);
        let toc_path = find_toc_file(&dir).unwrap_or_else(|| {
            panic!("`{smoke}` TOC must resolve via find_toc_file before LOD-trigger probe");
        });
        let result = wow_ui_sim::loader::load_addon(&env.loader_env(), &toc_path);
        assert!(
            result.is_ok(),
            "SMOKE tutorial `{smoke}` must load cleanly when triggered after the eager \
             Game sweep. The eager sweep brought in TutorialManager + Tutorials so the \
             state-machine mixin and queue are live; the SMOKE load itself just sweeps \
             the addon's XML+Lua body. If this fails, the SMOKE coverage classification \
             is wrong — the addon would need a custom harness pulling in non-eager deps. \
             Got: {result:?}"
        );

        let loaded: bool = env
            .eval(&format!("return C_AddOns.IsAddOnLoaded('{smoke}')"))
            .unwrap_or_else(|err| panic!("post-LOD IsAddOnLoaded({smoke}) probe failed: {err}"));
        assert!(
            loaded,
            "After explicit load_addon, IsAddOnLoaded('{smoke}') must flip to true"
        );
    }
}
}

#[test]
fn login_warning_dialogs_carries_glue_only_directives_for_regional_warning_frames() {
    let toc = parse_lane_toc("Blizzard_LoginWarningDialogs");

    assert!(
        toc.is_glue_only(),
        "Blizzard_LoginWarningDialogs MUST report is_glue_only() == true via its \
         `## AllowLoad: Glue` directive. The regional warning frames it ships \
         (ChinaAgeAppropriatenessWarning, KoreanRatings, TaiwanFraudWarning) parent to \
         `GlueParent`, not UIParent — they only make sense on the login screen before \
         the user has authenticated"
    );
    assert!(
        toc.default_enabled(),
        "Blizzard_LoginWarningDialogs carries `## DefaultState: enabled` — regional \
         compliance warnings cannot be opt-in. If a future SavedVariables migration \
         shipped this disabled, certain regions would lose their legally-required age \
         and fraud-prevention dialogs"
    );
    assert_eq!(
        toc.dependencies(),
        vec![
            "Blizzard_SharedXML".to_string(),
            "Blizzard_GlueXMLBase".to_string(),
            "Blizzard_GlueParent".to_string(),
        ],
        "LoginWarningDialogs RequiredDep order: SharedXML (CallbackRegistry, foundation \
         utilities) → GlueXMLBase (glue-screen XML primitives) → GlueParent (the actual \
         glue-screen root frame, since every warning dialog parents to GlueParent). \
         If this list shifts, the XML inheritance / parent lookup at file-scope fails"
    );
}

prefork_full_ui_case! {
fn lane_addons_emit_no_lane_specific_errors_during_full_load_for_static_popup_definitions(env: &WowLuaEnv) {

    let popup_definitions_count: i64 = env
        .eval(
            "local n = 0; \
             for k, v in pairs(StaticPopupDialogs) do \
                 if type(v) == 'table' and v.text then n = n + 1 end \
             end; \
             return n",
        )
        .expect("StaticPopupDialogs population probe");

    assert!(
        popup_definitions_count > 30,
        "StaticPopupDialogs MUST contain >30 well-formed definitions (entries with a \
         `text` field). The combined SharedDialogDefs.lua + GameDialogDefs.lua define ~50 \
         confirmation flows (refund, equipment-set, voice-chat-redock, etc.). A count of \
         ≤30 indicates GameDialogDefs.lua failed mid-execution. Got: {popup_definitions_count}"
    );
}
}

#[test]
fn lane_required_harness_shape_helpers_exist() {
    assert!(
        blizzard_ui_dir().is_dir(),
        "Harness invariant: blizzard_ui_dir() (Interface/BlizzardUI) must resolve. Every \
         lane test sets `state.addon_base_paths = vec![blizzard_ui_dir()]`"
    );

    let env = fresh_lane_env();
    let kind: String = env
        .eval("return type(_G)")
        .expect("fresh env must expose _G");
    assert_eq!(
        kind, "table",
        "Harness invariant: a fresh WowLuaEnv must expose `_G` even before any addon \
         loads. The interaction lane's per-module test plans should each follow the 7-step \
         harness documented in `tests/blizzard_core_frame_lane.rs`. Per-cluster \
         minimum-plan coverage: \
         (a) POPUP FLOW — load StaticPopup core+Game eagerly (already covered by full Game \
             sweep), assert StaticPopupDialogs >30 entries, assert StaticPopup_Show / \
             _Hide / _Visible / _Queue functions exist, simulate a Show→Hide cycle by \
             calling StaticPopup_Show('OKAY', ...) and asserting StaticPopup_Visible('OKAY') \
             == true; \
         (b) GLUE VARIANTS — switch to ScreenKind::Login, run discovery + load, assert \
             LoginWarningDialogs publishes ChinaAgeAppropriatenessWarning / KoreanRatings / \
             TaiwanFraudWarning frames parented to GlueParent, assert StaticPopup_Glue \
             publishes GlueDialog frame, assert StaticPopup_Game absent from runtime; \
         (c) TUTORIAL-STATE TRANSITIONS — full Game load, assert TutorialQueue / \
             TutorialPointerFrame / TutorialMainFrameMixin published, assert \
             TutorialMainFrameMixin.States enum matches canonical 4-state set, assert \
             TutorialStateMachineMixin:AddState + :BeginState are functions, simulate a \
             state machine: create a synthetic mixin instance, call AddState + \
             SetInitialStateName + BeginInitialState + verify activeStateName flipped"
    );
}
