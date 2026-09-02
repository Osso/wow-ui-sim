#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn mirror_timer_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_MirrorTimer")
}

fn mirror_timer_toc() -> PathBuf {
    mirror_timer_dir().join("Blizzard_MirrorTimer_Mainline.toc")
}

const MIRROR_TIMER_TOC_FILES: &[&str] = &["Mainline/MirrorTimer.lua", "Mainline/MirrorTimer.xml"];

const MIRROR_TIMER_ATLAS_KEYS: &[(&str, &str)] = &[
    ("EXHAUSTION", "ui-castingbar-filling-standard"),
    ("BREATH", "ui-castingbar-filling-applyingcrafting"),
    ("DEATH", "ui-castingbar-filling-standard"),
    ("FEIGNDEATH", "ui-castingbar-filling-channel"),
];

const CONTAINER_MIXIN_METHODS: &[&str] = &[
    "OnLoad",
    "OnEvent",
    "SetupTimer",
    "ClearTimer",
    "GetActiveTimer",
    "GetAvailableTimer",
    "ForceUpdateTimers",
    "ShouldShow",
    "SetIsInEditMode",
    "HasAnyTimersShowing",
];

const TIMER_MIXIN_METHODS: &[&str] = &[
    "OnUpdate",
    "OnShow",
    "OnHide",
    "Setup",
    "Clear",
    "SetPaused",
    "UpdateStatusBarValue",
    "HasTimer",
    "SetIsInEditModeInternal",
    "SetIsInEditMode",
    "ShouldShow",
    "UpdateShownState",
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
fn blizzard_mirror_timer_find_toc_resolves_mainline_variant() {
    let resolved =
        find_toc_file(&mirror_timer_dir()).expect("Blizzard_MirrorTimer TOC should resolve");
    assert_eq!(
        resolved,
        mirror_timer_toc(),
        "Blizzard_MirrorTimer ships Mainline and Classic TOC variants; retail resolution \
         selects Blizzard_MirrorTimer_Mainline.toc"
    );
}

#[test]
fn blizzard_mirror_timer_toc_declares_default_state_with_edit_mode_dep() {
    let toc = TocFile::from_file(&mirror_timer_toc()).expect("Blizzard_MirrorTimer TOC parses");
    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_MirrorTimer omits `## LoadOnDemand:` — `## DefaultState: enabled` makes it \
         eager-load. The container frame must be live before the first MIRROR_TIMER_START \
         event fires (breath / fatigue / drowning timers are core gameplay surfaces) and \
         must register with EditMode at boot so the Edit Mode UI can pin its position"
    );
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());

    let deps = toc.dependencies();
    assert_eq!(
        deps,
        vec!["Blizzard_EditMode".to_string()],
        "TOC must declare exactly one `## Dependencies:` entry — Blizzard_EditMode. \
         MirrorTimerContainer inherits the `EditModeTimerBarsSystemTemplate` virtual template \
         (XML line 57), so EditMode's template registry MUST be resolved before XML loads. \
         Got {deps:?}"
    );

    assert!(toc.optional_deps().is_empty());
    assert!(
        toc.saved_variables().is_empty(),
        "Zero saved variables — mirror-timer state is server-driven via MIRROR_TIMER_* events; \
         there's no client-side persistence (the EditMode position lives under EditMode's \
         own SVs, not this addon's)"
    );
}

#[test]
fn blizzard_mirror_timer_toc_declares_allow_load_game_capitalized_with_mainline_only() {
    let raw = std::fs::read_to_string(mirror_timer_toc()).expect("Mirror timer TOC reads");
    assert!(
        raw.contains("## AllowLoad: Game"),
        "TOC must declare `## AllowLoad: Game` exactly with capital `G`. The mirror timers are \
         in-world only; the case-insensitive matcher at src/toc.rs:308 normalizes through \
         `eq_ignore_ascii_case`, but the raw spelling is the visible reminder that the \
         simulator's parser must tolerate either capitalization"
    );
    assert!(
        raw.contains("## AllowLoadGameType: mainline"),
        "TOC must declare `## AllowLoadGameType: mainline` — Classic clients carry their own \
         legacy mirror-timer surface that pre-dates the EditMode-driven TimerBars system"
    );
}

#[test]
fn blizzard_mirror_timer_toc_is_unrestricted_on_mainline_screens() {
    let toc = TocFile::from_file(&mirror_timer_toc()).expect("Blizzard_MirrorTimer TOC parses");
    assert!(
        !toc.is_game_type_restricted(),
        "TOC declares `## AllowLoadGameType: mainline`; the simulator's \
         `is_game_type_restricted` (src/toc.rs:294) treats `mainline` as the retail target \
         and returns false. A `true` return would silently skip MirrorTimer from retail \
         discovery and break breath / fatigue / feign-death timer display"
    );

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "`## AllowLoad: Game` must allow Game screen via the case-insensitive matcher"
    );
    for glue in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        assert!(
            !toc.allows_screen(glue),
            "`## AllowLoad: Game` must NOT allow glue screen {glue:?}. Mirror timers are \
             driven by MIRROR_TIMER_START events that only fire post-PLAYER_ENTERING_WORLD; \
             glue screens have no mirror surface"
        );
    }
}

#[test]
fn blizzard_mirror_timer_toc_lists_two_files_one_lua_one_xml() {
    let toc = TocFile::from_file(&mirror_timer_toc()).expect("Blizzard_MirrorTimer TOC parses");
    let listed: Vec<String> = toc
        .files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        listed, MIRROR_TIMER_TOC_FILES,
        "Retail Mainline TOC must list Mainline/MirrorTimer.lua then \
         Mainline/MirrorTimer.xml"
    );
}

#[test]
fn blizzard_mirror_timer_appears_only_on_game_screen_discovery() {
    let game_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = game_addons
        .iter()
        .any(|(name, _)| name == "Blizzard_MirrorTimer");
    assert!(
        in_game,
        "Blizzard_MirrorTimer (`## AllowLoad: Game`, `## DefaultState: enabled`, non-LOD) must \
         appear in Game-screen discovery"
    );

    for glue in [
        ScreenKind::Login,
        ScreenKind::CharacterSelect,
        ScreenKind::CharacterCreate,
    ] {
        let glue_addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), glue);
        let leaks_to_glue = glue_addons
            .iter()
            .any(|(name, _)| name == "Blizzard_MirrorTimer");
        assert!(
            !leaks_to_glue,
            "Blizzard_MirrorTimer must NOT appear in {glue:?} discovery — `## AllowLoad: Game` \
             gates it out of glue screens at the discovery layer"
        );
    }
}

#[test]
fn blizzard_mirror_timer_edit_mode_dep_appears_in_game_discovery() {
    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let edit_mode = addons.iter().any(|(name, _)| name == "Blizzard_EditMode");
    assert!(
        edit_mode,
        "Blizzard_EditMode (the sole declared dependency) must auto-discover on Game so the \
         loader can satisfy the dependency edge before booting MirrorTimer's XML \
         (MirrorTimerContainer inherits EditModeTimerBarsSystemTemplate)"
    );
}

prefork_full_ui_case! {
fn blizzard_mirror_timer_loads_without_addon_specific_lua_errors(env: &WowLuaEnv) {

    let addon_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| {
            message.contains("Blizzard_MirrorTimer")
                || message.contains("MirrorTimer")
                || message.contains("MirrorTimerContainer")
                || message.contains("MirrorTimerAtlas")
        })
        .cloned()
        .collect();
    assert!(
        addon_errors.is_empty(),
        "Blizzard_MirrorTimer emitted addon-specific Lua errors during load:\n  {}",
        addon_errors.join("\n  ")
    );
}
}

prefork_full_ui_case! {
fn blizzard_mirror_timer_is_addon_loaded_after_auto_discovery(env: &WowLuaEnv) {
    let loaded: bool = env
        .eval("return C_AddOns.IsAddOnLoaded('Blizzard_MirrorTimer')")
        .expect("IsAddOnLoaded probe succeeds");
    assert!(
        loaded,
        "C_AddOns.IsAddOnLoaded('Blizzard_MirrorTimer') must return true after the eager \
         auto-discovery sweep — proves the timer-bar addon registers with the loaded-set \
         during the standard Game-screen boot pipeline"
    );
}
}

prefork_full_ui_case! {
fn blizzard_mirror_timer_publishes_atlas_table_with_four_canonical_keys(env: &WowLuaEnv) {
    let kind: String = env
        .eval("return type(_G.MirrorTimerAtlas)")
        .expect("MirrorTimerAtlas probe");
    assert_eq!(
        kind, "table",
        "MirrorTimerAtlas must publish at `_G` as a table. MirrorTimer.lua line 3 declares \
         the timer→atlas-key map that drives StatusBar:SetStatusBarTexture in Setup. The 4 \
         canonical keys are Blizzard's full set of mirror-timer types — no other consumer \
         creates new entries"
    );

    for (key, expected_atlas) in MIRROR_TIMER_ATLAS_KEYS {
        let atlas: String = env
            .eval(&format!("return MirrorTimerAtlas.{key}"))
            .unwrap_or_else(|err| panic!("MirrorTimerAtlas.{key} probe failed: {err}"));
        assert_eq!(
            atlas, *expected_atlas,
            "MirrorTimerAtlas.{key} must equal `{expected_atlas}`. The 4 timer types map to \
             3 distinct atlas keys: EXHAUSTION + DEATH share the standard fill atlas, BREATH \
             uses the crafting-application variant for the underwater drowning bar, \
             FEIGNDEATH uses the channel atlas for the hunter feign-death state"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_mirror_timer_publishes_container_mixin_with_ten_methods(env: &WowLuaEnv) {
    let kind: String = env
        .eval("return type(_G.MirrorTimerContainerMixin)")
        .expect("MirrorTimerContainerMixin probe");
    assert_eq!(
        kind, "table",
        "MirrorTimerContainerMixin must publish at `_G` as a table — XML line 57 references \
         it via `mixin=\"MirrorTimerContainerMixin\"` on the MirrorTimerContainer frame"
    );
    for method in CONTAINER_MIXIN_METHODS {
        let method_kind: String = env
            .eval(&format!("return type(MirrorTimerContainerMixin.{method})"))
            .unwrap_or_else(|err| panic!("MirrorTimerContainerMixin.{method} probe failed: {err}"));
        assert_eq!(
            method_kind, "function",
            "MirrorTimerContainerMixin.{method} must be a function. The 10 methods cover the \
             container's full responsibility surface: OnLoad / OnEvent are the script \
             handlers (registers PLAYER_ENTERING_WORLD + the 3 MIRROR_TIMER_* events), \
             SetupTimer / ClearTimer / GetActiveTimer / GetAvailableTimer manage the active \
             timer pool, ForceUpdateTimers drives manual refresh, ShouldShow / \
             SetIsInEditMode / HasAnyTimersShowing handle EditMode visibility"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_mirror_timer_publishes_timer_mixin_with_twelve_methods(env: &WowLuaEnv) {
    let kind: String = env
        .eval("return type(_G.MirrorTimerMixin)")
        .expect("MirrorTimerMixin probe");
    assert_eq!(
        kind, "table",
        "MirrorTimerMixin must publish at `_G` as a table — XML line 3 references it via \
         `mixin=\"MirrorTimerMixin\"` on the MirrorTimerTemplate virtual template"
    );
    for method in TIMER_MIXIN_METHODS {
        let method_kind: String = env
            .eval(&format!("return type(MirrorTimerMixin.{method})"))
            .unwrap_or_else(|err| panic!("MirrorTimerMixin.{method} probe failed: {err}"));
        assert_eq!(
            method_kind, "function",
            "MirrorTimerMixin.{method} must be a function. The 12 methods cover the \
             per-timer-bar lifecycle: OnUpdate / OnShow / OnHide are XML script handlers, \
             Setup / Clear / SetPaused drive the timer state machine, UpdateStatusBarValue \
             pulls live progress via GetMirrorTimerProgress, HasTimer probes occupancy, \
             SetIsInEditModeInternal / SetIsInEditMode / ShouldShow / UpdateShownState \
             handle EditMode visibility"
        );
    }
}
}

prefork_full_ui_case! {
fn blizzard_mirror_timer_container_frame_publishes_with_correct_parent_and_mixin_table(env: &WowLuaEnv) {
    let probe = "\
        return type(_G.MirrorTimerContainer) == 'table' \
            and MirrorTimerContainer:GetParent() == UIParent \
            and type(MirrorTimerContainer.mirrorTimers) == 'table' \
            and #MirrorTimerContainer.mirrorTimers == 3 \
            and type(MirrorTimerContainer.activeTimers) == 'table'";
    let ok: bool = env
        .eval(probe)
        .expect("MirrorTimerContainer structural probe succeeds");
    assert!(
        ok,
        "MirrorTimerContainer must publish at `_G` as a table parented to UIParent, with \
         `mirrorTimers` array carrying exactly 3 child timer frames (XML lines 63-77 declare \
         3 MirrorTimerTemplate inline children with parentArray=\"mirrorTimers\" inherited \
         from the template at line 3) and `activeTimers` table allocated by \
         MirrorTimerContainerMixin:OnLoad. The 3 fixed slots match the Blizzard contract: \
         numMirrorTimerTypes=3 (Exhaustion / Breath / Death are the canonical mirror types; \
         FEIGNDEATH reuses an Exhaustion slot — Blizzard never bumped the count past 3)"
    );
}
}

prefork_full_ui_case! {
fn blizzard_mirror_timer_template_does_not_leak_as_global(env: &WowLuaEnv) {
    let kind: String = env
        .eval("return type(_G.MirrorTimerTemplate)")
        .expect("MirrorTimerTemplate global probe succeeds");
    assert_eq!(
        kind, "nil",
        "MirrorTimerTemplate is declared `virtual=\"true\"` (XML line 3) so it must register \
         in the XML template registry but MUST NOT leak as a `_G.*` global. Virtual templates \
         are inheritance shells consumed via `inherits=\"MirrorTimerTemplate\"` at template \
         instantiation time; they're not directly addressable from Lua code"
    );
}
}
