#![cfg(feature = "client-retail")]
use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::startup::fire_startup_events_for_screen;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn declension_frame_toc() -> PathBuf {
    blizzard_ui_dir().join("Blizzard_DeclensionFrame/Blizzard_DeclensionFrame_Mainline.toc")
}

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
fn blizzard_declension_frame_toc_has_no_dependencies_and_is_mainline_only() {
    let toc = TocFile::from_file(&declension_frame_toc())
        .expect("Blizzard_DeclensionFrame_Mainline TOC should parse");
    assert!(
        !toc.is_load_on_demand(),
        "Blizzard_DeclensionFrame is non-LOD — the mainline stub auto-loads on Game-screen \
         bring-up so locale-specific addons can override its globals before any \
         BATTLEPET_FORCE_NAME_DECLENSION / PET_FORCE_NAME_DECLENSION event fires"
    );
    assert!(
        !toc.is_secure_env(),
        "Blizzard_DeclensionFrame does not declare UseSecureEnvironment"
    );
    let deps = toc.dependencies();
    assert!(
        deps.is_empty(),
        "Blizzard_DeclensionFrame has no dependency metadata in retail 12.1.0.69497; its \
         mainline files are locale-override placeholders. Got: {deps:?}"
    );

    let toc_text = std::fs::read_to_string(declension_frame_toc())
        .expect("Blizzard_DeclensionFrame TOC should read");
    assert!(
        toc_text.contains("## AllowLoad: game"),
        "Blizzard_DeclensionFrame declares `## AllowLoad: game` (the mainline stub is \
         in-game-only — there is no Glue declension UI)"
    );
    assert!(
        toc_text.contains("## AllowLoadGameType: mainline"),
        "Blizzard_DeclensionFrame declares `## AllowLoadGameType: mainline` (Classic flavors \
         ship their own DeclensionFrame variants — only mainline retail loads this stub)"
    );
}

#[test]
fn blizzard_declension_frame_appears_in_game_discovery() {
    let addons = discover_blizzard_addons_for_screen(&blizzard_ui_dir(), ScreenKind::Game);
    let in_game = addons
        .iter()
        .any(|(name, _)| name == "Blizzard_DeclensionFrame");
    assert!(
        in_game,
        "Blizzard_DeclensionFrame is non-LOD with `## AllowLoad: game` and \
         `## AllowLoadGameType: mainline`, so its locale-override placeholders should appear in \
         Game-screen auto-discovery"
    );
}

prefork_full_ui_case! {
fn blizzard_declension_frame_loads_without_errors(env: &WowLuaEnv) {

    let addon_errors: Vec<String> = env
        .state()
        .borrow()
        .lua_errors
        .iter()
        .filter(|message| message.contains("DeclensionFrame"))
        .cloned()
        .collect();
    assert!(
        addon_errors.is_empty(),
        "Blizzard_DeclensionFrame emitted Lua errors during load:\n  {}",
        addon_errors.join("\n  ")
    );
}
}

#[test]
fn blizzard_declension_frame_mainline_stub_files_are_intentionally_empty() {
    let lua_path = blizzard_ui_dir().join("Blizzard_DeclensionFrame/Mainline/DeclensionFrame.lua");
    let xml_path = blizzard_ui_dir().join("Blizzard_DeclensionFrame/Mainline/DeclensionFrame.xml");

    let lua_text =
        std::fs::read_to_string(&lua_path).expect("Mainline/DeclensionFrame.lua should read");
    let xml_text =
        std::fs::read_to_string(&xml_path).expect("Mainline/DeclensionFrame.xml should read");

    assert!(
        lua_text.contains("Overridden by the locale-specific versions"),
        "Mainline/DeclensionFrame.lua is a single-line placeholder comment by design — the \
         real declension UI ships only with locale builds (ruRU / koKR / zhCN handle \
         declension; enUS/etc don't need it). Got:\n{lua_text}"
    );
    assert!(
        xml_text.contains("Overridden by the locale-specific versions"),
        "Mainline/DeclensionFrame.xml is a body-less <Ui> document by design — the real \
         declension XML ships only with locale builds. Got:\n{xml_text}"
    );
}

prefork_full_ui_case! {
fn blizzard_declension_frame_publishes_no_globals_in_mainline_stub(env: &WowLuaEnv) {

    let no_globals: bool = env
        .eval(
            "return _G.DeclensionFrame == nil \
                and _G.DeclensionFrameMixin == nil \
                and _G.DeclensionFrame_Initialize == nil \
                and _G.DeclensionFrame_Update == nil \
                and _G.DeclensionFrame_OnLoad == nil \
                and _G.DeclensionFrame_Show == nil \
                and _G.DeclensionFrame_Hide == nil",
        )
        .expect("Mainline stub global query should succeed");
    assert!(
        no_globals,
        "Blizzard_DeclensionFrame's mainline stub should publish NO globals — \
         Mainline/DeclensionFrame.lua is just `-- Overridden by the locale-specific versions`. \
         The locale builds (ruRU / koKR / zhCN) ship their own DeclensionFrame.{{lua,xml}} \
         that define the actual `DeclensionFrame` toplevel + `DeclensionFrame_*` functions \
         for BATTLEPET_FORCE_NAME_DECLENSION / PET_FORCE_NAME_DECLENSION handling — those \
         events are still registered by FrameXML but go unhandled in non-declension locales"
    );
}
}

#[test]
fn blizzard_declension_frame_force_name_declension_events_are_registerable() {
    assert!(
        wow_ui_sim::event::is_registerable_event("BATTLEPET_FORCE_NAME_DECLENSION"),
        "BATTLEPET_FORCE_NAME_DECLENSION should be a registerable event (listed in \
         src/event/valid_events_a.rs:179) — fired by the client when a battle pet name \
         needs locale-specific declension. The mainline DeclensionFrame stub does not handle \
         it; the ruRU/koKR/zhCN locale overrides do"
    );
    assert!(
        wow_ui_sim::event::is_registerable_event("PET_FORCE_NAME_DECLENSION"),
        "PET_FORCE_NAME_DECLENSION should be a registerable event (listed in \
         src/event/valid_events_b.rs:400) — the hunter-pet counterpart, same locale-only \
         handling story"
    );
}
