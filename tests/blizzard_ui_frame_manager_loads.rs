use std::path::PathBuf;

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be synced")
}

fn ui_frame_manager_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_UIFrameManager")
}

fn ui_frame_manager_toc() -> PathBuf {
    ui_frame_manager_dir().join("Blizzard_UIFrameManager.toc")
}

const GLUE_SCREENS: &[ScreenKind] = &[
    ScreenKind::Login,
    ScreenKind::CharacterSelect,
    ScreenKind::CharacterCreate,
];

const MIXINS: &[&str] = &["UIFrameManagerMixin", "UIFrameManager_ManagedFrameMixin"];

const UI_FRAME_MANAGER_MIXIN_METHODS: &[&str] = &["OnLoad", "OnEvent", "RegisterFrameForFrameType"];

const MANAGED_FRAME_MIXIN_METHODS: &[&str] = &["OnLoad", "UpdateFrameState"];

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

// Literal cached inputs absent from Wrath manifest1-382/cache; behavior cases stay enabled.
retail_ptr_forever_toc_fixture! {
#[test]
fn find_toc_file_resolves_bare_toc() {
    let resolved = find_toc_file(&ui_frame_manager_dir()).expect("UIFrameManager TOC resolves");
    assert_eq!(
        resolved,
        ui_frame_manager_toc(),
        "Bare TOC — no flavor suffix. Resolved via the bare-TOC path \
         in find_toc_file at src/loader/mod.rs:65-95"
    );
}
}

retail_ptr_forever_toc_fixture! {
#[test]
fn toc_is_eager_with_no_dependencies() {
    let toc = TocFile::from_file(&ui_frame_manager_toc()).expect("TOC parses");

    assert!(
        !toc.is_load_on_demand(),
        "`## LoadOnDemand: 0` is intentional self-documentation — \
         toc.rs:259-264 only matches `1`/`true`, so the explicit `0` \
         keeps the addon eager. Required because two RequiredDep \
         consumers (Blizzard_MawBuffs, Blizzard_TieredEntranceTraits) \
         are themselves LoD and need this manager already on disk to \
         promote them when their LoadAddOn fires"
    );

    assert!(
        toc.dependencies().is_empty(),
        "No `## Dependencies:` / `## RequiredDep:` — the manager only \
         touches always-available globals (C_FrameManager + RegisterEvent \
         on a Frame). Got: {:?}",
        toc.dependencies()
    );
    assert!(toc.optional_deps().is_empty());
    assert!(toc.saved_variables().is_empty());
    assert!(toc.saved_variables_per_character().is_empty());
    assert!(!toc.is_load_first());
    assert!(!toc.is_secure_env());
    assert!(
        !toc.is_game_type_restricted(),
        "AllowLoadGameType absent → not flavor-restricted"
    );
    assert!(toc.default_enabled());
}
}

post_wrath_ui_fixture! {
#[test]
fn allow_load_absent_implies_game_only() {
    let toc = TocFile::from_file(&ui_frame_manager_toc()).expect("TOC parses");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "AllowLoad absent → toc.rs:305-313 None branch defaults to \
         Game-only. The frame-visibility manager runs only in-world; \
         glue screens have no Enum.UIFrameType frames"
    );
    for screen in GLUE_SCREENS {
        assert!(
            !toc.allows_screen(*screen),
            "Glue screen {screen:?} must be excluded — no AllowLoad \
             directive means Game-only via the None branch"
        );
    }
}
}

retail_ptr_forever_toc_fixture! {
#[test]
fn toc_raw_bytes_pin_two_directives_and_two_body_files() {
    let raw = std::fs::read_to_string(ui_frame_manager_toc()).expect("TOC reads utf-8");

    let expected_lines = [
        "## Title: Blizzard_UIFrameManager",
        "## LoadOnDemand: 0",
        "Blizzard_UIFrameManager.lua",
        "Blizzard_UIFrameManager.xml",
    ];

    for line in expected_lines {
        assert!(
            raw.contains(line),
            "Raw TOC must pin `{line}` — body order is lua-before-xml \
             so the XML's `mixin=\"UIFrameManagerMixin\"` and \
             `mixin=\"UIFrameManager_ManagedFrameMixin\"` attributes \
             resolve against published mixin tables when the loader \
             instantiates the named frame"
        );
    }

    assert!(!raw.contains("## Author"));
    assert!(!raw.contains("## Version"));
    assert!(!raw.contains("## Dependencies"));
    assert!(!raw.contains("## RequiredDep"));
    assert!(!raw.contains("## OptionalDeps"));
    assert!(!raw.contains("## AllowLoad"));
    assert!(!raw.contains("## AllowLoadGameType"));
    assert!(!raw.contains("## SavedVariables"));
    assert!(!raw.contains("## DefaultState"));
    assert!(!raw.contains("## UseSecureEnvironment"));
    assert!(!raw.contains("## LoadFirst"));
    assert!(!raw.contains("[Family]"));
}
}

post_wrath_ui_fixture! {
#[test]
fn appears_in_game_eager_discovery() {
    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let found = addons
        .iter()
        .any(|(name, _)| name == "Blizzard_UIFrameManager");
    assert!(
        found,
        "Blizzard_UIFrameManager must appear in Game eager discovery \
         — non-LoD with no AllowLoad (Game-only). Two RequiredDep \
         consumers are LoD addons (Blizzard_MawBuffs, \
         Blizzard_TieredEntranceTraits) that depend on this manager \
         being present at world entry. Blizzard_Tutorials' \
         RPETutorialInterruptMixin also CreateFromMixins this addon's \
         UIFrameManager_ManagedFrameMixin without declaring a TOC dep \
         — relies on the manager's eager load-order winning"
    );
}
}

#[test]
fn absent_from_glue_screens_eager_discovery() {
    for screen in GLUE_SCREENS {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), *screen);
        let found = addons
            .iter()
            .any(|(name, _)| name == "Blizzard_UIFrameManager");
        assert!(
            !found,
            "Blizzard_UIFrameManager must NOT appear on {screen:?} — \
             AllowLoad absent → Game-only via toc.rs:305-313 None \
             branch, checked at loader/mod.rs:527 BEFORE pool \
             partitioning"
        );
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
            "{mixin} must be a global table. UIFrameManagerMixin is the \
             singleton's behaviour (registered to UIFrameManager via XML \
             `mixin=`). UIFrameManager_ManagedFrameMixin is the base for \
             every C_FrameManager-driven frame — Blizzard_MawBuffs's \
             MawBuffsContainer, Blizzard_TieredEntranceTraits's panels, \
             and Blizzard_Tutorials' RPETutorialInterrupt_Frame all \
             CreateFromMixins this and set frameType via KeyValues"
        );
    }
}
}

prefork_full_ui_case! {
fn ui_frame_manager_mixin_methods_published(env: &WowLuaEnv) {

    for method in UI_FRAME_MANAGER_MIXIN_METHODS {
        let kind: String = env
            .eval(&format!("return type(UIFrameManagerMixin.{method})"))
            .unwrap_or_else(|err| panic!("UIFrameManagerMixin.{method} probe failed: {err}"));
        assert_eq!(
            kind, "function",
            "UIFrameManagerMixin.{method} must be a function. OnLoad \
             initialises 2 dispatch tables (registeredFrames + \
             registeredFrameTypeToFrames keyed by Enum.UIFrameType) and \
             registers FRAME_MANAGER_UPDATE_ALL + \
             FRAME_MANAGER_UPDATE_FRAME events. OnEvent dispatches \
             UPDATE_ALL by walking every frameType through \
             C_FrameManager.GetFrameVisibilityState, while \
             UPDATE_FRAME(frameType, show) targets only the frames \
             registered under that frameType. RegisterFrameForFrameType \
             is the dedup-on-first-call entry point that captures a \
             frame and immediately syncs its initial visibility"
        );
    }
}
}

prefork_full_ui_case! {
fn managed_frame_mixin_methods_published(env: &WowLuaEnv) {

    for method in MANAGED_FRAME_MIXIN_METHODS {
        let kind: String = env
            .eval(&format!(
                "return type(UIFrameManager_ManagedFrameMixin.{method})"
            ))
            .unwrap_or_else(|err| {
                panic!("UIFrameManager_ManagedFrameMixin.{method} probe failed: {err}")
            });
        assert_eq!(
            kind, "function",
            "UIFrameManager_ManagedFrameMixin.{method} must be a function. \
             OnLoad self-registers via \
             `UIFrameManager:RegisterFrameForFrameType(self, self.frameType)` \
             — `self.frameType` is bound by the inheriting XML's \
             `<KeyValues><KeyValue key=\"frameType\" .../>` (the comment \
             in the addon's XML at lines 11-15 documents this contract). \
             UpdateFrameState is the default visibility hook \
             (`self:SetShown(show)`); inheritors override it for custom \
             show/hide logic"
        );
    }
}
}

prefork_full_ui_case! {
fn ui_frame_manager_singleton_frame_exists(env: &WowLuaEnv) {

    let exists: bool = env
        .eval("return UIFrameManager ~= nil")
        .expect("UIFrameManager probe");
    assert!(
        exists,
        "UIFrameManager must exist as a named global Frame after XML \
         load. Defined as a top-level non-virtual `<Frame \
         name=\"UIFrameManager\" toplevel=\"true\" \
         mixin=\"UIFrameManagerMixin\">` in \
         Blizzard_UIFrameManager.xml:3 with OnLoad/OnEvent script \
         bindings — the OnLoad runs at frame creation, populating the \
         registry tables before any consumer's frame can call \
         UIFrameManager:RegisterFrameForFrameType"
    );

    let kind: String = env
        .eval("return type(UIFrameManager.RegisterFrameForFrameType)")
        .expect("RegisterFrameForFrameType probe");
    assert_eq!(
        kind, "function",
        "UIFrameManager:RegisterFrameForFrameType must dispatch through \
         the mixin's __index — the named frame is built with \
         `mixin=\"UIFrameManagerMixin\"` so all 3 mixin methods reach \
         the frame instance"
    );
}
}

prefork_full_ui_case! {
fn managed_frame_template_registered(env: &WowLuaEnv) {
    let _env = env;

    let template = wow_ui_sim::xml::get_template("UIFrameManager_ManagedFrame");
    assert!(
        template.is_some(),
        "UIFrameManager_ManagedFrame must be a registered virtual \
         template. Defined as `<Frame name=\"UIFrameManager_ManagedFrame\" \
         mixin=\"UIFrameManager_ManagedFrameMixin\" virtual=\"true\">` \
         in Blizzard_UIFrameManager.xml:9 with an OnLoad script binding. \
         The template's OnLoad calls \
         `UIFrameManager:RegisterFrameForFrameType(self, self.frameType)` \
         where `self.frameType` is bound by the inheriting concrete \
         frame's KeyValues — the template itself never gets \
         instantiated directly, only inherited (consumers like \
         Blizzard_Tutorials' RPETutorialInterrupt_Frame chain template \
         inheritance via TutorialMainFrameTemplate AND mixin \
         inheritance via UIFrameManager_ManagedFrameMixin)"
    );
}
}

post_wrath_ui_fixture! {
#[test]
fn dependent_addons_exist_on_disk() {
    for dependent in &["Blizzard_MawBuffs"] {
        let dir = blizzard_ui_dir().join(dependent);
        assert!(
            dir.is_dir(),
            "Dependent directory `{dependent}` must exist on disk — it \
             RequiredDeps Blizzard_UIFrameManager"
        );
        let toc = TocFile::from_file(&find_toc_file(&dir).expect("dep TOC")).expect("TOC parses");
        assert!(
            toc.dependencies()
                .iter()
                .any(|d| d == "Blizzard_UIFrameManager"),
            "{dependent} must declare Blizzard_UIFrameManager via \
             RequiredDep — toc.rs:209-217 reads RequiredDep / \
             RequiredDeps / Dependencies as interchangeable"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_emits_no_addon_specific_errors(env: &WowLuaEnv) {

    let errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let addon_specific: Vec<&String> = errors
        .iter()
        .filter(|e| e.contains("Blizzard_UIFrameManager/"))
        .collect();

    assert!(
        addon_specific.is_empty(),
        "Full game-screen load with UIFrameManager in dependency order \
         must emit zero UIFrameManager-body errors. The 2 body files \
         (lua + xml) must register cleanly; the OnLoad must initialise \
         the dispatch tables and register FRAME_MANAGER_UPDATE_* events \
         without raising. Found: {addon_specific:?}"
    );
}
}

// The Mists/Era/Anniversary caches ship 218 addons each and Wrath ships 23;
// none contains Blizzard_UIFrameManager, so the pinned TOC facts above describe no file the
// classic loaders can read.
classic_cache_absent_toc_fixture! {
#[test]
fn blizzard_uiframemanager_toc_is_absent_from_classic_caches() {
    let dir = ui_frame_manager_dir();
    assert!(
        !dir.is_dir(),
        "classic profile caches must not contain Blizzard_UIFrameManager"
    );
    assert!(wow_ui_sim::loader::find_toc_file(&dir).is_none());
}
}
