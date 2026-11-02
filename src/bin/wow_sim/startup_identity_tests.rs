//! Startup identity through real WTF discovery, addon loading and the reload loader.

use super::*;
use clap::Parser;
use std::path::{Path, PathBuf};
use std::process::Command;

const CHILD_ROOT: &str = "WOW_SIM_STARTUP_IDENTITY_TEST_ROOT";
const CHILD_CASE: &str = "WOW_SIM_STARTUP_IDENTITY_TEST_CASE";
const CHARACTER: &str = "Wörkspacé-Family";
const REALM: &str = "Test Realm-One";

fn selected_name() -> &'static str {
    if wow_ui_sim::client_profile::ACTIVE == wow_ui_sim::client_profile::ClientProfile::WowForever {
        "Wörkspacé"
    } else {
        CHARACTER
    }
}

fn selected_surname() -> Option<String> {
    (wow_ui_sim::client_profile::ACTIVE == wow_ui_sim::client_profile::ClientProfile::WowForever)
        .then(|| "Family".into())
}

fn write(path: &Path, contents: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, contents).unwrap();
}

fn spawn_identity_case(test: &str, character: Option<&str>, missing_root: bool) {
    spawn_identity_cases(test, character, missing_root, &["default"]);
}

fn spawn_identity_cases(test: &str, character: Option<&str>, missing_root: bool, cases: &[&str]) {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path();
    let wtf = root.join("WTF");
    for (character, profile) in [(CHARACTER, "selected"), ("OtherCharacter", "wrong")] {
        write(
            &wtf.join("Account/IdentityAccount")
                .join(REALM)
                .join(character)
                .join("SavedVariables/IdentityProbe.lua"),
            &format!("IdentityProbeDB = {{ profile = '{profile}' }}\n"),
        );
    }
    let addon = root.join("addons/IdentityProbe");
    write(
        &addon.join("IdentityProbe.toc"),
        &format!(
            "## Interface: {}\n## SavedVariablesPerCharacter: IdentityProbeDB\nCore.lua\n",
            wow_ui_sim::toc::ACTIVE_INTERFACE_VERSION
        ),
    );
    write(
        &addon.join("Core.lua"),
        r#"
        IdentityProbeDB = IdentityProbeDB or { profile = "fresh" }
        local function capture()
            local first, surname = UnitName("player")
            local full, second = UnitFullName("player")
            local _, class = UnitClass("player")
            local _, race = UnitRace("player")
            return {
                first = first, surname = surname, full = full, second = second,
                realmName = GetRealmName(), normalized = GetNormalizedRealmName(),
                arity = select('#', UnitName("player")),
                profile = IdentityProbeDB.profile, class = class, race = race,
            }
        end
        IdentityProbeFile = capture()
        local observer = CreateFrame("Frame")
        observer:RegisterEvent("ADDON_LOADED")
        observer:SetScript("OnEvent", function(_, _, name)
            if name == "IdentityProbe" then IdentityProbeLoaded = capture() end
        end)
        "#,
    );

    for case in cases {
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                &format!("startup::identity_tests::{test}"),
                "--nocapture",
                "--test-threads=1",
            ])
            .env(CHILD_ROOT, root)
            .env(CHILD_CASE, case)
            .env("WOW_SIM_STORAGE_DIR", root.join("session/SavedVariables"))
            .env(
                "WOW_SIM_WTF_PATH",
                if missing_root {
                    root.join("missing-WTF")
                } else {
                    wtf.clone()
                },
            )
            .env("WOW_SIM_WTF_ACCOUNT", "IdentityAccount")
            .env("WOW_SIM_WTF_REALM", REALM)
            .env("WOW_SIM_NO_SOUND", "1")
            .env("WOW_SIM_CASC", "0")
            .env_remove("WOW_SIM_NO_ADDONS")
            .env_remove("WOW_SIM_NO_SAVED_VARS")
            .env_remove("WOW_SIM_ADDONS_TXT")
            .env_remove("WOW_SIM_EDIT_MODE_LAYOUT");
        if let Some(character) = character {
            command.env("WOW_SIM_WTF_CHARACTER", character);
        } else {
            command.env_remove("WOW_SIM_WTF_CHARACTER");
        }
        let output = command.output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            output.status.success()
                && stdout
                    .lines()
                    .any(|line| line.starts_with("test result: ok. 1 passed; 0 failed;")),
            "identity startup child {test}/{case} did not pass exactly one test:\n{stdout}\n{}",
            String::from_utf8_lossy(&output.stderr),
        );
    }
}

fn child_root() -> Option<PathBuf> {
    std::env::var_os(CHILD_ROOT).map(PathBuf::from)
}

fn startup_args(root: &Path, no_saved_vars: bool) -> Args {
    let mut argv = vec![
        "wow-sim".into(),
        "--addons-path".into(),
        root.join("addons").into_os_string(),
        "--class".into(),
        "Mage".into(),
        "--race".into(),
        "Orc".into(),
    ];
    if no_saved_vars {
        argv.push("--no-saved-vars".into());
    }
    argv.push("dump-tree".into());
    Args::try_parse_from(argv).unwrap()
}

fn assert_addon_identity(env: &WowLuaEnv, profile: &str, class: &str, race: &str) {
    for observation in ["IdentityProbeFile", "IdentityProbeLoaded"] {
        let actual: (String, Option<String>, String, String, String, i32) = env
            .eval(&format!(
                "return {observation}.first, {observation}.surname, \
                 {observation}.full, {observation}.second, \
                 {observation}.normalized, {observation}.arity"
            ))
            .unwrap();
        assert_eq!(
            actual,
            (
                selected_name().into(),
                selected_surname(),
                selected_name().into(),
                selected_surname().unwrap_or_else(|| REALM.into()),
                "TestRealmOne".into(),
                if selected_surname().is_some() { 2 } else { 1 },
            ),
            "{observation}"
        );
        let actual: (String, String, String, String) = env
            .eval(&format!(
                "return {observation}.realmName, {observation}.profile, \
                 {observation}.class, {observation}.race"
            ))
            .unwrap();
        // Wrath restores SavedVariables after addon files, before ADDON_LOADED.
        let expected_profile = if wow_ui_sim::client_profile::ACTIVE
            == wow_ui_sim::client_profile::ClientProfile::Wrath
            && observation == "IdentityProbeFile"
        {
            "fresh"
        } else {
            profile
        };
        assert_eq!(
            actual,
            (
                REALM.into(),
                expected_profile.into(),
                class.into(),
                race.into()
            ),
            "{observation}"
        );
    }
}

#[test]
fn selected_character_reaches_real_startup_and_reload() {
    let Some(root) = child_root() else {
        spawn_identity_case(
            "selected_character_reaches_real_startup_and_reload",
            Some(CHARACTER),
            false,
        );
        return;
    };
    let args = startup_args(&root, false);
    let (mut env, _font_system, mut saved_vars) =
        init_and_load(&args, ScreenKind::Game, root.join("session")).unwrap();
    assert!(saved_vars.is_some());
    assert_addon_identity(&env, "selected", "MAGE", "Orc");

    let troll = wow_ui_sim::lua_api::state::RACE_DATA
        .iter()
        .position(|(name, _, _)| *name == "Troll")
        .unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.player.name = "ChangedAtRuntime".into();
        state.player.surname = Some("Changed".into());
        state.player.realm_name = Some("Changed Realm".into());
        state.player.class_index = 4;
        state.player.race_index = troll;
        state.player.active_spec_index = 2;
    }
    env.exec("IdentityProbeDB.profile = 'edited'; IdentityReloadSentinel = true; ReloadUI()")
        .unwrap();
    assert!(env.service_reload(&mut saved_vars).unwrap());
    assert!(
        env.eval::<bool>("return IdentityReloadSentinel == nil")
            .unwrap()
    );
    assert_addon_identity(&env, "edited", "ROGUE", "Troll");
    assert_eq!(env.state().borrow().player.active_spec_index, 2);
    let imported = root
        .join("WTF/Account/IdentityAccount")
        .join(REALM)
        .join(CHARACTER)
        .join("SavedVariables/IdentityProbe.lua");
    assert_eq!(
        std::fs::read_to_string(imported).unwrap(),
        "IdentityProbeDB = { profile = 'selected' }\n"
    );
}

#[test]
fn selected_character_is_seeded_with_persistence_disabled() {
    let Some(root) = child_root() else {
        spawn_identity_case(
            "selected_character_is_seeded_with_persistence_disabled",
            Some(CHARACTER),
            false,
        );
        return;
    };
    let args = startup_args(&root, true);
    let (mut env, _font_system, mut saved_vars) =
        init_and_load(&args, ScreenKind::Game, root.join("session")).unwrap();
    assert!(saved_vars.is_none());
    assert_addon_identity(&env, "fresh", "MAGE", "Orc");
    env.state().borrow_mut().player.name = "ChangedAtRuntime".into();
    env.exec("ReloadUI()").unwrap();
    assert!(env.service_reload(&mut saved_vars).unwrap());
    assert_addon_identity(&env, "fresh", "MAGE", "Orc");
    assert!(!root.join("session/SavedVariables").exists());
}

#[test]
fn explicit_session_restores_changes_after_process_restart() {
    let Some(root) = child_root() else {
        spawn_identity_cases(
            "explicit_session_restores_changes_after_process_restart",
            Some(CHARACTER),
            false,
            &["save", "restore"],
        );
        return;
    };
    let mut args = startup_args(&root, false);
    let session = root.join("session");
    args.session_wtf_path = Some(session.clone());
    let (mut env, _font_system, mut saved_vars) =
        init_and_load(&args, ScreenKind::Game, session).unwrap();
    match std::env::var(CHILD_CASE).unwrap().as_str() {
        "save" => {
            assert_addon_identity(&env, "selected", "MAGE", "Orc");
            env.exec("IdentityProbeDB.profile = 'saved-session'")
                .unwrap();
            env.finalize_session(&mut saved_vars).unwrap();
        }
        "restore" => assert_addon_identity(&env, "saved-session", "MAGE", "Orc"),
        case => panic!("unexpected session restoration case: {case}"),
    }
    let imported = root
        .join("WTF/Account/IdentityAccount")
        .join(REALM)
        .join(CHARACTER)
        .join("SavedVariables/IdentityProbe.lua");
    assert_eq!(
        std::fs::read_to_string(imported).unwrap(),
        "IdentityProbeDB = { profile = 'selected' }\n"
    );
}

fn assert_startup_selection_error(root: &Path, expected: &str) {
    let no_saved_vars = match std::env::var(CHILD_CASE).unwrap().as_str() {
        "saved-vars" => false,
        "no-saved-vars" => true,
        case => panic!("unexpected selection failure case: {case}"),
    };
    let args = startup_args(root, no_saved_vars);
    let env = WowLuaEnv::with_session_wtf_path(root.join("session")).unwrap();
    let state = env.state().clone();
    let initial_addons: Vec<_> = state
        .borrow()
        .addons
        .iter()
        .map(|addon| (addon.folder_name.clone(), addon.loaded))
        .collect();
    let result = initialize_and_load_env(&args, ScreenKind::Game, env);
    let error = match result {
        Ok(_) => panic!("invalid explicit identity unexpectedly completed startup"),
        Err(error) => error.to_string(),
    };
    assert!(error.contains(expected), "{error}");
    let final_addons: Vec<_> = state
        .borrow()
        .addons
        .iter()
        .map(|addon| (addon.folder_name.clone(), addon.loaded))
        .collect();
    assert_eq!(
        final_addons, initial_addons,
        "addons ran before selection failed"
    );
    assert!(!root.join("session/SavedVariables").exists());
}

#[test]
fn missing_selected_character_rejects_startup_without_fallback() {
    let Some(root) = child_root() else {
        spawn_identity_cases(
            "missing_selected_character_rejects_startup_without_fallback",
            Some("AbsentCharacter"),
            false,
            &["saved-vars", "no-saved-vars"],
        );
        return;
    };
    assert_startup_selection_error(&root, "AbsentCharacter");
}

#[test]
fn missing_selected_wtf_root_rejects_startup_without_fallback() {
    let Some(root) = child_root() else {
        spawn_identity_cases(
            "missing_selected_wtf_root_rejects_startup_without_fallback",
            Some(CHARACTER),
            true,
            &["saved-vars", "no-saved-vars"],
        );
        return;
    };
    assert_startup_selection_error(&root, "missing-WTF");
}

#[test]
fn absent_or_empty_character_selection_keeps_default_identity() {
    let Some(root) = child_root() else {
        for character in [None, Some("")] {
            spawn_identity_case(
                "absent_or_empty_character_selection_keeps_default_identity",
                character,
                false,
            );
        }
        return;
    };
    let args = startup_args(&root, false);
    let env = WowLuaEnv::with_session_wtf_path(root.join("session")).unwrap();
    let original: (String, Option<String>, String, String) = env
        .eval("return UnitName('player'), nil, GetRealmName(), GetNormalizedRealmName()")
        .unwrap();
    let (env, _font_system, _saved_vars) =
        initialize_and_load_env(&args, ScreenKind::Game, env).unwrap();
    for observation in ["IdentityProbeFile", "IdentityProbeLoaded"] {
        let actual: (String, Option<String>, String, String) = env
            .eval(&format!(
                "return {observation}.first, {observation}.surname, \
                 {observation}.realmName, {observation}.normalized"
            ))
            .unwrap();
        assert_eq!(actual, original, "{observation}");
    }
}
