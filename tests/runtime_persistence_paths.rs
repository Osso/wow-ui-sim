//! Exercise explicit session paths with isolated import and legacy environments.

use std::path::{Path, PathBuf};
use wow_ui_sim::lua_api::{AddonInfo, WowLuaEnv};
use wow_ui_sim::saved_variables::{SavedVariablesManager, WtfConfig};

#[test]
fn explicit_session_roots_do_not_share_preferences_or_cvars() {
    let fixture = tempfile::tempdir().unwrap();
    let sessions = [
        (fixture.path().join("first-WTF"), "Mage", "37"),
        (fixture.path().join("second-WTF"), "Rogue", "41"),
    ];
    for (session, class, value) in &sessions {
        let cvars = wow_ui_sim::cvars::CVarStorage::with_session_wtf_path(session);
        assert_eq!(cvars.get("runtimeIsolationProbe"), None);
        cvars.set("runtimeIsolationProbe", value);
        let mut config = wow_ui_sim::config::SimConfig::load_from_session_wtf(session);
        config.player_class = (*class).into();
        config.save();
    }
    for (session, class, value) in &sessions {
        assert_eq!(
            wow_ui_sim::cvars::CVarStorage::with_session_wtf_path(session)
                .get("runtimeIsolationProbe")
                .as_deref(),
            Some(*value)
        );
        assert_eq!(
            wow_ui_sim::config::SimConfig::load_from_session_wtf(session).player_class,
            *class
        );
    }
}

#[test]
fn session_wtf_root_isolates_persistence_and_keeps_imports_read_only() {
    const CHILD_ROOT: &str = "WOW_SIM_PATH_ISOLATION_TEST_ROOT";
    if let Some(root) = std::env::var_os(CHILD_ROOT).map(PathBuf::from) {
        verify_isolated_persistence(&root);
        return;
    }

    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path();
    let source = WtfConfig::new(root.join("real-WTF"), "Account", "Realm", "Hero");
    let account_input = b"RuntimeIsolationProbe = { value = 37 }\n";
    let character_input = b"CharacterIsolationProbe = { value = 41 }\n";
    write(
        &source.account_saved_vars_file("IsolationProbe"),
        account_input,
    );
    write(
        &source.character_saved_vars_file("IsolationProbe"),
        character_input,
    );
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "session_wtf_root_isolates_persistence_and_keeps_imports_read_only",
            "--exact",
            "--nocapture",
        ])
        .env(CHILD_ROOT, root)
        .env("WOW_SIM_SESSION_WTF_PATH", root.join("legacy/session-WTF"))
        .env("WOW_SIM_WTF_PATH", &source.wtf_path)
        .env("WOW_SIM_WTF_ACCOUNT", &source.account)
        .env("WOW_SIM_WTF_REALM", &source.realm)
        .env("WOW_SIM_WTF_CHARACTER", &source.character)
        // Removed overrides cannot send writes outside the session WTF root.
        .env("WOW_SIM_STORAGE_DIR", root.join("legacy/SavedVariables"))
        .env("WOW_SIM_CVAR_PATH", root.join("legacy/cvars.json"))
        .env("WOW_SIM_CONFIG_PATH", root.join("legacy/config.json"))
        .env_remove("WOW_SIM_ADDONS_TXT")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "isolated persistence failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!root.join("legacy").exists());
    assert_eq!(
        std::fs::read(source.account_saved_vars_file("IsolationProbe")).unwrap(),
        account_input
    );
    assert_eq!(
        std::fs::read(source.character_saved_vars_file("IsolationProbe")).unwrap(),
        character_input
    );
    assert!(!source.wtf_path.join("config.json").exists());
    assert!(!source.wtf_path.join("cvars.json").exists());
    assert!(!source.wtf_path.join("AddOns.txt").exists());
}

#[test]
fn addon_selection_override_keeps_session_addons_file_unchanged() {
    const CHILD_ROOT: &str = "WOW_SIM_ADDON_SELECTION_TEST_ROOT";
    if let Some(root) = std::env::var_os(CHILD_ROOT).map(PathBuf::from) {
        let session = root.join("session-WTF");
        let selected = root.join("selection/AddOns.txt");
        let path = wow_ui_sim::addon_enable_state::local_addons_txt_path(&session);
        assert_eq!(path, selected);
        let overrides = wow_ui_sim::addon_enable_state::read_addon_enable_overrides(&path).unwrap();
        assert_eq!(overrides.get("SelectionProbe"), Some(&false));

        let env = WowLuaEnv::with_session_wtf_path(session.clone()).unwrap();
        env.register_addon(AddonInfo {
            folder_name: "SelectionProbe".into(),
            enabled: overrides["SelectionProbe"],
            ..Default::default()
        });
        env.exec("C_AddOns.EnableAddOn('SelectionProbe'); C_AddOns.SaveAddOns()")
            .unwrap();
        assert_eq!(
            std::fs::read_to_string(selected).unwrap(),
            "SelectionProbe: enabled\n"
        );
        assert_eq!(
            std::fs::read(session.join("AddOns.txt")).unwrap(),
            b"SessionProbe: disabled\n"
        );
        return;
    }

    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path();
    let selected = root.join("selection/AddOns.txt");
    write(&selected, b"SelectionProbe: disabled\n");
    write(
        &root.join("session-WTF/AddOns.txt"),
        b"SessionProbe: disabled\n",
    );
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "addon_selection_override_keeps_session_addons_file_unchanged",
            "--exact",
            "--nocapture",
        ])
        .env(CHILD_ROOT, root)
        .env("WOW_SIM_ADDONS_TXT", selected)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "addon selection override failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn verify_isolated_persistence(root: &Path) {
    let session = root.join("session-WTF");
    let cvars = wow_ui_sim::cvars::CVarStorage::with_session_wtf_path(&session);
    cvars.set("runtimeIsolationProbe", "37");
    let contents = std::fs::read(session.join("cvars.json")).unwrap();
    let persisted: serde_json::Value = serde_json::from_slice(&contents).unwrap();
    assert_eq!(persisted["runtimeisolationprobe"], "37");
    assert_eq!(
        wow_ui_sim::cvars::CVarStorage::with_session_wtf_path(&session)
            .get("runtimeIsolationProbe"),
        Some("37".into())
    );

    let mut config = wow_ui_sim::config::SimConfig::load_from_session_wtf(&session);
    config.player_class = "Mage".into();
    config.save();
    let contents = std::fs::read(session.join("config.json")).unwrap();
    let persisted: serde_json::Value = serde_json::from_slice(&contents).unwrap();
    assert_eq!(persisted["player_class"], "Mage");
    assert_eq!(
        wow_ui_sim::config::SimConfig::load_from_session_wtf(&session).player_class,
        "Mage"
    );
    wow_ui_sim::addon_enable_state::save_local_addon_enable_overrides(
        &session,
        &[AddonInfo {
            folder_name: "DisabledProbe".into(),
            enabled: false,
            ..Default::default()
        }],
    )
    .unwrap();
    assert_eq!(
        std::fs::read_to_string(session.join("AddOns.txt")).unwrap(),
        "DisabledProbe: disabled\n"
    );

    let source = wow_ui_sim::paths::default_wtf_config().unwrap();
    assert_eq!(
        source.wtf_path,
        root.join("real-WTF").canonicalize().unwrap()
    );
    let mut env = WowLuaEnv::with_session_wtf_path(session.clone()).unwrap();
    let mut manager = SavedVariablesManager::with_session_wtf_path(&session);
    manager.set_wtf_config(source);
    register_probe(&env, &mut manager).unwrap();
    env.exec(
        "assert(RuntimeIsolationProbe.value == 37 and CharacterIsolationProbe.value == 41); \
         RuntimeIsolationProbe.value = 73; CharacterIsolationProbe.value = 91",
    )
    .unwrap();
    #[cfg(not(feature = "client-wrath"))]
    env.exec("C_TTSSettings.SetSpeechRate(3); C_TTSSettings.SetSpeechVolume(43)")
        .unwrap();
    env.set_reload_loader(|env, manager| register_probe(env, manager.as_mut().unwrap()));
    let mut manager = Some(manager);
    env.exec("ReloadUI()").unwrap();
    assert!(env.service_reload(&mut manager).unwrap());
    env.exec(
        "assert(RuntimeIsolationProbe.value == 73 and CharacterIsolationProbe.value == 91); \
         RuntimeIsolationProbe.value = 74; CharacterIsolationProbe.value = 92",
    )
    .unwrap();
    env.finalize_session(&mut manager).unwrap();
    #[cfg(feature = "client-wrath")]
    assert!(
        session
            .join("SavedVariables/Realm/Hero/layout-local.txt")
            .is_file()
    );

    let mut readback = rilua::Lua::new().unwrap();
    readback
        .exec(&std::fs::read_to_string(session.join("SavedVariables/IsolationProbe.lua")).unwrap())
        .unwrap();
    readback
        .exec(
            &std::fs::read_to_string(session.join("SavedVariables/Realm/Hero/IsolationProbe.lua"))
                .unwrap(),
        )
        .unwrap();
    readback
        .exec("assert(RuntimeIsolationProbe.value == 74 and CharacterIsolationProbe.value == 92)")
        .unwrap();

    let restored = WowLuaEnv::with_session_wtf_path(session.clone()).unwrap();
    let mut manager = SavedVariablesManager::with_session_wtf_path(&session);
    manager.set_wtf_config(wow_ui_sim::paths::default_wtf_config().unwrap());
    register_probe(&restored, &mut manager).unwrap();
    restored
        .exec("assert(RuntimeIsolationProbe.value == 74 and CharacterIsolationProbe.value == 92)")
        .unwrap();
    #[cfg(not(feature = "client-wrath"))]
    {
        let profile = wow_ui_sim::client_profile::ACTIVE.cache_subdir();
        assert!(
            session
                .join("cvars")
                .join(profile)
                .join("SavedVariables/TTSSettings.lua")
                .is_file()
        );
        restored
            .exec("assert(C_TTSSettings.GetSpeechRate() == 3 and C_TTSSettings.GetSpeechVolume() == 43)")
            .unwrap();
    }
}

fn register_probe(env: &WowLuaEnv, manager: &mut SavedVariablesManager) -> wow_ui_sim::Result<()> {
    env.loader_env().with_state(|state| {
        manager.init_for_addon(
            state,
            "IsolationProbe",
            &["RuntimeIsolationProbe".into()],
            &["CharacterIsolationProbe".into()],
        )
    })
}

fn write(path: &Path, bytes: &[u8]) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, bytes).unwrap();
}


// Windows checks out a repository with `LongPathsEnabled=0` only when every
// tracked path fits `MAX_PATH` (260) including the clone root. Budget 60
// characters for a `C:\Users\<user>\source\repos\wow-ui-sim\` clone root and
// Git temporary names, leaving a hard limit of 200 characters per
// repo-relative path. `git ls-files` covers tracked paths even when a Windows
// checkout already dropped files.
#[test]
fn tracked_paths_fit_windows_max_path_budget() {
    const MAX_REPO_RELATIVE_PATH: usize = 200;
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output = match std::process::Command::new("git")
        .arg("-C")
        .arg(&repo)
        .args(["ls-files", "-z"])
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            eprintln!("skipping path guard: cannot run git: {error}");
            return;
        }
    };
    if !output.status.success() {
        eprintln!(
            "skipping path guard: git ls-files failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
        return;
    }
    let mut offenders: Vec<(usize, String)> = String::from_utf8_lossy(&output.stdout)
        .split('\0')
        .filter(|path| !path.is_empty())
        .map(|path| (path.chars().count(), path.to_string()))
        .filter(|(len, _)| *len > MAX_REPO_RELATIVE_PATH)
        .collect();
    offenders.sort_by(|a, b| b.cmp(a));
    assert!(
        offenders.is_empty(),
        "{} tracked paths exceed the {MAX_REPO_RELATIVE_PATH}-character Windows path budget \
         (clone root plus overhead is budgeted at {}):\n{}",
        offenders.len(),
        260 - MAX_REPO_RELATIVE_PATH,
        offenders
            .iter()
            .map(|(len, path)| format!("  {len}  {path}"))
            .collect::<Vec<_>>()
            .join("\n"),
    );
}
