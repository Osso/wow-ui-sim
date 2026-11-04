//! File-cache import before whole Game startup; no mocked EditMode consumer.
#![cfg(any(feature = "client-retail", feature = "client-ptr"))]

use std::path::Path;
use std::process::Command;
use wow_ui_sim::loader::{
    StartupAddonLoadKind, discover_blizzard_startup_addons_for_screen, load_startup_addon,
};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::saved_variables::{SavedVariablesManager, WtfConfig};
use wow_ui_sim::screen::ScreenKind;

const ACCOUNT_CACHE: &[u8] =
    include_bytes!("fixtures/frame_position_replay/edit-mode-cache-account.txt");
const CHARACTER_CACHE: &[u8] = b"2 2 2 2 0 0\0";

fn import_cache(
    env: &WowLuaEnv,
    root: &Path,
    snapshot: Option<&str>,
    account_cache: &[u8],
    character_cache: &[u8],
) {
    let wtf = root.join("WTF");
    let account = wtf.join("Account/TestAccount");
    let character = account.join("Test Realm/Testchar");
    std::fs::create_dir_all(&character).expect("create isolated cache tree");
    std::fs::write(account.join("edit-mode-cache-account.txt"), account_cache)
        .expect("write supplied account cache");
    std::fs::write(
        character.join("edit-mode-cache-character.txt"),
        character_cache,
    )
    .expect("write native character-cache indices");
    let mut manager = SavedVariablesManager::with_storage_dir(root.join("local-sv"));
    manager.set_wtf_config(WtfConfig::new(
        &wtf,
        "TestAccount",
        "Test Realm",
        "Testchar",
    ));
    let loaded = env
        .loader_env()
        .with_state(|state| manager.load_edit_mode_cache(state, 2, snapshot))
        .expect("production file-cache import");
    assert!(loaded, "provided cache must be imported");
}

fn load_and_dispatch_startup_addon(
    env: &WowLuaEnv,
    name: &str,
    toc: &Path,
    kind: StartupAddonLoadKind,
) -> Vec<String> {
    let result = load_startup_addon(&env.loader_env(), toc, kind, None)
        .unwrap_or_else(|error| panic!("whole Game startup load {name}: {error}"));
    for warning in &result.warnings {
        eprintln!("startup {name}: {warning}");
    }
    if kind == StartupAddonLoadKind::Full {
        env.fire_event_with_args("ADDON_LOADED", &[env.lua_string(name)])
            .unwrap_or_else(|error| panic!("startup ADDON_LOADED {name}: {error}"));
    }
    if name == "Blizzard_EnvironmentCleanup" {
        env.restore_post_cleanup_globals();
    }
    result.warnings
}

pub(crate) fn load_whole_game_ui(env: &WowLuaEnv) -> Vec<String> {
    let ui = wow_ui_sim::client_profile::blizzard_ui_addons_dir();
    env.set_screen_mode(ScreenKind::Game);
    env.state().borrow_mut().addon_base_paths = vec![ui.clone()];
    wow_ui_sim::xml::register_intrinsic_templates();
    let mut warnings = Vec::new();
    if let Some(toc) = wow_ui_sim::client_profile::blizzard_ui_framexml_toc() {
        warnings.extend(load_and_dispatch_startup_addon(
            env,
            "FrameXML",
            &toc,
            StartupAddonLoadKind::Full,
        ));
    }
    for addon in discover_blizzard_startup_addons_for_screen(&ui, ScreenKind::Game) {
        warnings.extend(load_and_dispatch_startup_addon(
            env,
            &addon.name,
            &addon.toc_path,
            addon.kind,
        ));
    }
    env.sync_string_metatable_to_global_string();
    env.sync_addon_names_to_lua();
    env.apply_post_load_workarounds();
    wow_ui_sim::startup::fire_startup_events(env);
    wow_ui_sim::startup::process_pending_timers(env);
    wow_ui_sim::startup::fire_one_on_update_tick(env);
    warnings
}

fn dispatch_specialization_change(env: &WowLuaEnv) {
    env.exec("beforeEditModeDispatch = EditModeManagerFrame.layoutInfo")
        .expect("record prior layout");
    crate::common::panel_fixtures::clear_recorded_lua_errors(env);
    env.fire_event_with_args("PLAYER_SPECIALIZATION_CHANGED", &[env.lua_string("player")])
        .expect("dispatch specialization change");
    let errors = crate::common::panel_fixtures::recorded_lua_errors(env);
    assert!(
        errors.is_empty(),
        "specialization dispatch errors: {errors:?}"
    );
    env.exec("assert(EditModeManagerFrame.layoutInfo ~= beforeEditModeDispatch, 'EditMode handler did not run')")
        .expect("registered EditMode handler ran");
}

#[test]
fn truncated_account_cache_does_not_fabricate_layouts() {
    test_timeout! {
        let text = std::str::from_utf8(ACCOUNT_CACHE).expect("fixture text");
        let cache = text.split_whitespace().take(561).collect::<Vec<_>>().join(" ");
        for name in ["LayoutOne", "LayoutTwo"] {
            let cache = cache.replace("Ultrawide", name);
            let temp = tempfile::tempdir().expect("isolated cache");
            let env = WowLuaEnv::new().expect("fresh environment");
            import_cache(&env, temp.path(), None, cache.as_bytes(), CHARACTER_CACHE);
            load_whole_game_ui(&env);
            dispatch_specialization_change(&env);
            env.exec(&format!(r#"
                local layouts = C_EditMode.GetLayouts().layouts
                assert(#layouts == 1, 'one real layout and no phantom saved layouts')
                assert(layouts[1].layoutName == '{name}' and #layouts[1].systems == 52,
                    'complete first layout must survive')
                -- PTR manager :5-32 includes Gamepad; Retail has two presets.
                -- Shared/EditModeManager.lua:1018-1020 prepends that source list.
                local presets = EditModePresetLayoutManager:GetCopyOfPresetLayouts()
                assert(#EditModeManagerFrame.layoutInfo.layouts == #presets + 1,
                    'presets plus one saved layout')
                assert(PlayerFrame:GetNumPoints() > 0, 'PlayerFrame must retain anchors')
            "#)).expect("original EditMode manager initialization");
        }
    }
}

#[test]
fn truncated_account_cache_warns_during_default_startup() {
    test_timeout! {
        let text = std::str::from_utf8(ACCOUNT_CACHE).expect("fixture text");
        let header = text.split_whitespace().take(38).collect::<Vec<_>>();
        for count in [1, 2] {
            let temp = tempfile::tempdir().expect("isolated cache");
            let wtf = temp.path().join("WTF");
            let account = wtf.join("Account/TestAccount");
            let character = account.join("Test Realm/Testchar");
            std::fs::create_dir_all(&character).expect("create cache tree");
            let cache = format!("{count} {}", header[1..].join(" "));
            std::fs::write(account.join("edit-mode-cache-account.txt"), cache).expect("write cache");
            std::fs::write(character.join("edit-mode-cache-character.txt"), CHARACTER_CACHE).expect("write character cache");
            let output = Command::new("timeout")
                .args(["90", env!("CARGO_BIN_EXE_wow-sim"), "--session-wtf-path"])
                .arg(temp.path().join("data/wow-sim"))
                .args(["--no-addons", "--no-saved-vars", "lua-errors"])
                .env("WOW_SIM_WTF_PATH", &wtf)
                .env("WOW_SIM_WTF_ACCOUNT", "TestAccount")
                .env("WOW_SIM_WTF_REALM", "Test Realm")
                .env("WOW_SIM_WTF_CHARACTER", "Testchar")
                .env_remove("RUST_LOG")
                .env_remove("WOW_SIM_EDIT_MODE_LAYOUT")
                .output().expect("run default startup");
            assert!(output.status.success(),
                "startup failed for count {count}: {}\nstdout: {}\nstderr: {}",
                output.status,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr));
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(stderr.contains("EditMode account cache declares layout rows that are missing"),
                "missing default warning for count {count}: {stderr}");
        }
    }
}

#[test]
fn complete_account_cache_preserves_both_layouts() {
    test_timeout! {
        let text = std::str::from_utf8(ACCOUNT_CACHE).expect("fixture text");
        let text = text.replace("Ultrawide", "LayoutOne").replace("Widescreen", "LayoutTwo");
        let temp = tempfile::tempdir().expect("isolated cache");
        let env = WowLuaEnv::new().expect("fresh environment");
        import_cache(&env, temp.path(), None, text.as_bytes(), CHARACTER_CACHE);
        load_whole_game_ui(&env);
        dispatch_specialization_change(&env);
        env.exec(r#"
            local layouts = C_EditMode.GetLayouts().layouts
            assert(#layouts == 2, 'both saved layouts')
            assert(layouts[1].layoutName == 'LayoutOne' and #layouts[1].systems == 52)
            assert(layouts[2].layoutName == 'LayoutTwo' and #layouts[2].systems == 52)
            local presets = EditModePresetLayoutManager:GetCopyOfPresetLayouts()
            assert(#EditModeManagerFrame.layoutInfo.layouts == #presets + 2,
                'presets and saved layouts')
            assert(PlayerFrame:GetNumPoints() > 0, 'PlayerFrame must retain anchors')
        "#).expect("original EditMode manager initialization");
    }
}

#[test]
fn native_preset_index_two_keeps_classic_and_horizontal_micro_menu() {
    test_timeout! {
        let temp = tempfile::tempdir().expect("create isolated cache fixture");
        let env = WowLuaEnv::new().expect("fresh pre-addon environment");
        env.set_display_size(1920.0, 1080.0).expect("display input");
        import_cache(&env, temp.path(), None, ACCOUNT_CACHE, CHARACTER_CACHE);
        env.exec("assert(C_EditMode.GetLayouts().activeLayout == 2, 'native cache-index readiness')")
            .expect("unaltered character-cache active index");
        let warnings = load_whole_game_ui(&env);
        env.exec(r#"
            local layout = EditModeManagerFrame:GetActiveLayoutInfo()
            local classic = EditModePresetLayoutManager:GetCopyOfPresetLayouts()[2]
            assert(layout.layoutName == classic.layoutName,
                'native cached preset index 2 must select Classic, not saved Widescreen')
            local width, height = MicroMenu:GetSize()
            assert(width > 0 and height > 0, 'actual micro-menu geometry readiness')
            assert(width > height, 'Classic micro menu must have horizontal geometry')
            local _, firstY = CharacterMicroButton:GetCenter()
            local _, secondY = PlayerSpellsMicroButton:GetCenter()
            assert(firstY and secondY and math.abs(firstY - secondY) <= 1,
                'first microbuttons must share a horizontal row')
        "#).expect("unchanged Classic/micro-menu consumer result");
        assert!(warnings.is_empty(), "startup warnings retained: {warnings:?}");
    }
}

#[test]
fn explicit_layout_overrides_keep_widescreen_through_whole_startup() {
    test_timeout! {
        for preferred in ["Widescreen", "wIdEsCrEeN", "2", "10"] {
            let temp = tempfile::tempdir().expect("create isolated override fixture");
            let env = WowLuaEnv::new().expect("fresh pre-addon environment");
            env.set_display_size(1920.0, 1080.0).expect("display input");
            import_cache(&env, temp.path(), Some(preferred), ACCOUNT_CACHE, CHARACTER_CACHE);
            let warnings = load_whole_game_ui(&env);
            env.exec(r#"
                local presets = EditModePresetLayoutManager:GetCopyOfPresetLayouts()
                assert(EditModeManagerFrame.layoutInfo.activeLayout == #presets + 2,
                    'explicit Widescreen override must address the combined list')
                assert(EditModeManagerFrame:GetActiveLayoutInfo().layoutName == 'Widescreen',
                    'explicit override identity must survive preset insertion')
                local width, height = MicroMenu:GetSize()
                assert(width > 0 and height > width,
                    'explicit Widescreen retains its vertical Micro Menu')
            "#).unwrap_or_else(|error| panic!("whole startup override {preferred}: {error}"));
            assert!(warnings.is_empty(), "startup warnings for {preferred}: {warnings:?}");
        }
    }
}
