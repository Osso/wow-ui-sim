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

fn ui_parent_dir() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_UIParent")
}

fn ui_parent_toc() -> PathBuf {
    ui_parent_dir().join("Blizzard_UIParent.toc")
}

const GLUE_SCREENS: &[ScreenKind] = &[
    ScreenKind::Login,
    ScreenKind::CharacterSelect,
    ScreenKind::CharacterCreate,
];

const TOC_DEPENDENCIES: &[&str] = &["Blizzard_SharedXMLBase"];

const MIXINS: &[&str] = &[
    "ManagedFrameMixin",
    "ManagedFrameContainerMixin",
];

const VIRTUAL_TEMPLATES: &[&str] = &[
    "ChatBubbleTemplate",
    "ManagedFrameTemplate",
    "BottomManagedFrameTemplate",
    "RightManagedFrameTemplate",
    "ManagedFrameContainerBaseTemplate",
    "ManagedFrameContainer",
];

const NAMED_NON_VIRTUAL_FRAMES: &[&str] = &[
    "UIParent",
    "WorldFrame",
    "BottomManagedFrameContainer",
    "RightManagedFrameContainer",
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

#[cfg(feature = "client-ptr")]
#[test]
fn ptr_mainline_excludes_mists_time_string_helper() {
    let mainline_toc =
        std::fs::read_to_string(ui_parent_dir().join("Blizzard_UIParent_Mainline.toc"))
            .expect("mainline TOC should read");
    let mists_toc = std::fs::read_to_string(ui_parent_dir().join("Blizzard_UIParent_Mists.toc"))
        .expect("mists TOC should read");
    assert!(!mainline_toc.contains("Mists\\UIParent.lua"));
    assert!(mists_toc.contains("Mists\\UIParent.lua"));

    let env = fresh_game_env();
    let assert_helper_absent = |stage: &str| {
        let helper_is_absent: bool = env
            .eval("return GetTimeStringFromSeconds == nil")
            .expect("time string helper visibility should be queryable");
        assert!(
            helper_is_absent,
            "Mists-only GetTimeStringFromSeconds leaked into PTR mainline during {stage}"
        );
    };
    assert_helper_absent("environment initialization");

    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    for (name, toc_path) in &addons {
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }
    env.apply_post_load_workarounds();
    assert_helper_absent("Blizzard loading or post-load compatibility");

    fire_startup_events_for_screen(&env, ScreenKind::Game);
    assert_helper_absent("settled startup");
}

#[test]
fn find_toc_file_resolves_bare_toc() {
    let resolved = find_toc_file(&ui_parent_dir()).expect("UIParent TOC resolves");
    assert_eq!(
        resolved,
        ui_parent_toc(),
        "Retail 12.1 ships one bare Blizzard_UIParent.toc; loader discovery must select it"
    );
}

#[test]
fn toc_is_eager_with_shared_xml_base_dependency() {
    let toc = TocFile::from_file(&ui_parent_toc()).expect("TOC parses");

    assert!(!toc.is_load_on_demand());
    assert_eq!(toc.dependencies(), vec!["Blizzard_SharedXMLBase".to_string()]);
    assert!(toc.optional_deps().is_empty());
    assert!(toc.saved_variables().is_empty());
    assert!(toc.saved_variables_per_character().is_empty());
    assert!(!toc.is_load_first());
    assert!(!toc.is_game_type_restricted());
    assert!(toc.default_enabled());
}

#[test]
fn toc_without_allow_load_defaults_to_game_screen_only() {
    let toc = TocFile::from_file(&ui_parent_toc()).expect("TOC parses");

    assert!(
        toc.allows_screen(ScreenKind::Game),
        "A TOC without AllowLoad must load on the game screen"
    );

    for screen in GLUE_SCREENS {
        assert!(
            !toc.allows_screen(*screen),
            "A TOC without AllowLoad must not load on glue screen {screen:?}"
        );
    }
}

#[test]
fn toc_body_resolves_to_single_ui_parent_lua_file() {
    let toc = TocFile::from_file(&ui_parent_toc()).expect("TOC parses");
    assert_eq!(
        toc.files,
        vec![PathBuf::from("UIParent.lua")],
        "Retail 12.1 UIParent TOC must resolve exactly one body file"
    );
}

#[test]
fn toc_raw_bytes_pin_current_three_line_contract() {
    let raw = std::fs::read_to_string(ui_parent_toc()).expect("TOC reads utf-8");
    assert_eq!(
        raw.lines().collect::<Vec<_>>(),
        [
            "## Title: Blizzard_UIParent",
            "## Dep: Blizzard_SharedXMLBase",
            "UIParent.lua",
        ],
        "Retail 12.1 UIParent TOC must remain the current two-directive, one-body-file contract"
    );
}

#[test]
fn appears_in_game_eager_discovery() {
    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let found = addons.iter().any(|(name, _)| name == "Blizzard_UIParent");
    assert!(
        found,
        "Blizzard_UIParent must appear in Game eager discovery — without \
         it no in-world panel can resolve `parent=\"UIParent\"` and the \
         entire UI tree fails to compose"
    );
}

#[test]
fn absent_from_glue_screens_eager_discovery() {
    for screen in GLUE_SCREENS {
        let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), *screen);
        let found = addons.iter().any(|(name, _)| name == "Blizzard_UIParent");
        assert!(
            !found,
            "Blizzard_UIParent must NOT appear on {screen:?} — \
             AllowLoad:game restricts to in-world via toc.rs:308, \
             checked at loader/mod.rs:527 BEFORE pool partitioning"
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
fn full_game_load_publishes_mixins(env: &WowLuaEnv) {

    for mixin in MIXINS {
        let kind: String = env
            .eval(&format!("return type({mixin})"))
            .unwrap_or_else(|err| panic!("{mixin} probe failed: {err}"));
        assert_eq!(
            kind, "table",
            "{mixin} must be a global table after load. ManagedFrameSystem publishes \
             ManagedFrameMixin for OnShow/OnHide updates and ManagedFrameContainerMixin \
             for the managedFrames collection and its layout updates"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_keeps_ui_parent_visible(env: &WowLuaEnv) {
    let visible: bool = env
        .eval("return UIParent:IsShown()")
        .expect("UIParent visibility query should succeed");
    assert!(
        visible,
        "Retail 12.1 Blizzard_UIParent asserts that the top-level UIParent is already shown when \
         its visibility callbacks are installed"
    );
}
}

prefork_full_ui_case! {
fn full_game_load_installs_top_level_parent_visibility_scripts(env: &WowLuaEnv) {
    let scripts_installed: bool = env
        .eval(
            "return type(UIParent:GetScript('OnShow')) == 'function' \
                and type(UIParent:GetScript('OnHide')) == 'function'",
        )
        .expect("UIParent visibility script query should succeed");
    assert!(
        scripts_installed,
        "Retail 12.1 Blizzard_UIParent installs OnShow and OnHide callbacks directly on UIParent \
         to publish UI.TopLevelParentShown and UI.TopLevelParentHidden EventRegistry events"
    );
}
}

prefork_full_ui_case! {
fn full_game_load_does_not_restore_retired_uiparent_globals(env: &WowLuaEnv) {
    let retired_globals_absent: bool = env
        .eval("return PULSEBUTTONS == nil and UIParent_OnLoad == nil")
        .expect("retired UIParent globals query should succeed");
    assert!(
        retired_globals_absent,
        "Retail 12.1 replaces the legacy UIParent-owned table and named-handler surface with \
         direct script installation; loading Blizzard_UIParent must not restore those retired \
         globals. TOOLTIP_UPDATE_TIME remains a valid SharedXML constant and is intentionally \
         outside this assertion"
    );
}
}

prefork_full_ui_case! {
fn world_frame_onupdate_runs_without_mirror_timer_constant_errors(env: &WowLuaEnv) {

    let before = env.state().borrow().lua_errors.len();
    env.fire_on_update(0.016)
        .expect("WorldFrame OnUpdate should tick");
    let new_errors = env.state().borrow().lua_errors[before..].to_vec();

    assert!(
        new_errors.is_empty(),
        "WorldFrame_OnUpdate should not emit Lua errors after a frame tick. \
         Mists 5.5.4 iterates 1..MIRRORTIMER_NUMTIMERS while processing \
         hidden mirror timers. New errors: {new_errors:?}"
    );
}
}

prefork_full_ui_case! {
fn full_game_load_registers_virtual_templates(env: &WowLuaEnv) {
    let _env = env;

    for template in VIRTUAL_TEMPLATES {
        let entry = wow_ui_sim::xml::get_template(template);
        assert!(
            entry.is_some(),
            "{template} must be a registered virtual template. \
             ChatBubbleTemplate (ChatBubbleTemplates.xml) is the nine-slice chat-bubble \
             chassis with ARTWORK-layer FontString and a tail texture. ManagedFrameTemplate \
             is the base for managed frames; the Bottom and Right variants add layout \
             KeyValues. ManagedFrameContainerBaseTemplate and ManagedFrameContainer provide \
             the layout-container inheritance chain"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_publishes_named_non_virtual_frames(env: &WowLuaEnv) {

    for name in NAMED_NON_VIRTUAL_FRAMES {
        let exists: bool = env
            .eval(&format!("return _G[{name:?}] ~= nil"))
            .unwrap_or_else(|err| panic!("{name} probe failed: {err}"));
        assert!(
            exists,
            "{name} must exist as a global frame after load. UIParent \
             (UIParent.xml:4) is the protected `setAllPoints` MEDIUM-strata \
             root with `preventSecretValues=\"true\"` and a ScopedModifier \
             wrapping (`addToSecureEnv=\"true\"`). WorldFrame \
             (WorldFrame.xml:21) is the unique world-render container \
             with `clipChildren=\"true\"` and `propagateMouseInput=\"Both\"`. \
             BottomManagedFrameContainer / RightManagedFrameContainer are concrete managed \
             frame containers at frameStrata=\"LOW\" — they host the layout slots populated by \
             managed-frame consumers"
        );
    }
}
}

prefork_full_ui_case! {
fn full_game_load_emits_no_addon_specific_errors(env: &WowLuaEnv) {

    let errors: Vec<String> = env.state().borrow().lua_errors.clone();
    let addon_specific: Vec<&String> = errors
        .iter()
        .filter(|e| e.contains("Blizzard_UIParent/"))
        .collect();

    assert!(
        addon_specific.is_empty(),
        "Full game-screen load with UIParent in dependency order must \
         emit zero UIParent-body errors. The 7 body files (3 lua + 3 xml \
         + 1 localization-overlay lua, ~4000 lines) include \
         Mainline/UIParent.lua at 2842 lines with ~131 free functions \
         and the UIParent_OnEvent megaswitch; all must execute cleanly. \
         Found: {addon_specific:?}"
    );
}
}
