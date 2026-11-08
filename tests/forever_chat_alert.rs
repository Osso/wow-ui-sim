//! Forever must expose `AlertContainerMixin` before the chat alert consumer runs.
//!
//! Pinned consumer: `Blizzard_ChatFrameBase/Shared/ChatAlertFrameMixin.lua:4`
//! reads `AlertContainerMixin`. Provider: `Blizzard_FrameXML/Mainline/AlertFrames.lua`
//! (TOC `[Family]\AlertFrames.lua`, `[Family]` = Mainline on Forever).
#![cfg(feature = "client-wowforever")]

use std::path::PathBuf;

use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, load_addon};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn blizzard_ui_dir() -> PathBuf {
    wow_ui_sim::client_profile::blizzard_ui_addons_dir_under(std::path::Path::new(env!(
        "CARGO_MANIFEST_DIR"
    )))
}

fn load_full_game_ui() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    env.set_screen_mode(ScreenKind::Game);

    let ui = blizzard_ui_dir();
    {
        let mut state = env.state().borrow_mut();
        state.addon_base_paths = vec![ui.clone()];
    }

    wow_ui_sim::xml::register_intrinsic_templates();

    let addons = discover_blizzard_addons_for_screen(&ui, ScreenKind::Game);
    for (name, toc_path) in &addons {
        load_addon(&env.loader_env(), toc_path)
            .unwrap_or_else(|err| panic!("[load {name}] FAILED: {err}"));
    }

    env.apply_post_load_workarounds();
    wow_ui_sim::startup::fire_startup_events_for_screen(&env, ScreenKind::Game);
    env
}

#[test]
fn forever_frame_xml_toc_lists_the_alert_frames_provider() {
    let ui = blizzard_ui_dir();
    let toc_path = wow_ui_sim::loader::find_toc_file(&ui.join("Blizzard_FrameXML"))
        .expect("Forever Blizzard_FrameXML TOC should exist");
    let toc = TocFile::from_file(&toc_path).expect("Forever Blizzard_FrameXML TOC should parse");
    let files: Vec<String> = toc
        .files
        .iter()
        .map(|path| path.to_string_lossy().replace('\\', "/"))
        .collect();

    assert!(
        files.iter().any(|file| file == "Mainline/AlertFrames.lua"),
        "pinned Blizzard_FrameXML.toc:49 lists `[Family]\\AlertFrames.lua`, and Forever's \
         family is Mainline, so the parsed TOC must contain Mainline/AlertFrames.lua; got {files:?}"
    );
}

#[test]
fn forever_chat_alert_consumer_finds_alert_container_mixin() {
    let env = load_full_game_ui();

    let (mixin_type, consumer_type): (String, String) = env
        .eval(
            "return type(AlertContainerMixin), type(ChatAlertFrameMixin)",
        )
        .expect("mixin probe should succeed");
    assert_eq!(
        mixin_type, "table",
        "Blizzard_FrameXML/Mainline/AlertFrames.lua:264 defines AlertContainerMixin; \
         Blizzard_ChatFrameBase/Shared/ChatAlertFrameMixin.lua:4 reads it during OnLoad"
    );
    assert_eq!(consumer_type, "table", "ChatAlertFrameMixin should load");

    let frame_exists: bool = env
        .eval("return ChatAlertFrame ~= nil")
        .expect("ChatAlertFrame probe should succeed");
    if frame_exists {
        let (ok, err): (bool, Option<String>) = env
            .eval(
                "local ok, err = pcall(ChatAlertFrameMixin.OnLoad, ChatAlertFrame); \
                 return ok, ok and nil or tostring(err)",
            )
            .expect("ChatAlertFrameMixin.OnLoad probe should succeed");
        assert!(ok, "ChatAlertFrameMixin.OnLoad failed: {err:?}");
    }
}
