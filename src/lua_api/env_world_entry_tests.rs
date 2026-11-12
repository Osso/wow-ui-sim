//! Engine host-policy tests, not native lifecycle/event-order conformance tests.

use super::WowLuaEnv;
use crate::c_api::c_login::character_roster::CharacterSlot;
use crate::screen::ScreenKind;
use std::cell::Cell;
use std::rc::Rc;

fn selected_glue_env() -> (tempfile::TempDir, WowLuaEnv) {
    let session = tempfile::tempdir().unwrap();
    let env = WowLuaEnv::with_session_wtf_path(session.path().to_path_buf()).unwrap();
    env.set_screen_mode(ScreenKind::Login);
    let mut account = env.state().borrow().bnet_friends[0].game_accounts[0].clone();
    account.character_name = "Roster".into();
    account.surname = "Family".into();
    account.realm_name = "Roster Realm".into();
    env.state().borrow_mut().glue_login.characters = vec![CharacterSlot::from(account)];
    env.exec("SelectCharacter(1)").unwrap();
    (session, env)
}

#[test]
fn host_world_entry_reuses_owner_loader_with_selected_identity() {
    let (_session, mut env) = selected_glue_env();
    let loads = Rc::new(Cell::new(0));
    let loaded = Rc::clone(&loads);
    env.set_reload_loader(move |env, _| {
        loaded.set(loaded.get() + 1);
        assert_eq!(env.state().borrow().screen_kind, ScreenKind::Game);
        assert!(env.has_host_world_entry_identity());
        assert!(env.is_host_world_entry_bootstrap());
        assert_eq!(env.state().borrow().player.name, "Roster");
        assert_eq!(
            env.state().borrow().player.surname.as_deref(),
            Some("Family")
        );
        assert_eq!(
            env.state().borrow().player.realm_name.as_deref(),
            Some("Roster Realm")
        );
        env.exec("HostBootstrapPlayerName = UnitName('player')")
    });
    env.state().borrow_mut().player.level = 42;
    let errors_before = env.state().borrow().lua_errors.len();
    env.exec("EnterWorld()").unwrap();
    // Deferral until the mutable owner boundary is simulator scheduling policy.
    assert_eq!(loads.get(), 0);
    assert!(env.service_ui_transition(&mut None).unwrap());
    assert_eq!(loads.get(), 1);
    assert_eq!(
        env.eval::<String>("return HostBootstrapPlayerName")
            .unwrap(),
        "Roster"
    );
    assert_eq!(env.state().borrow().player.level, 42);
    assert_eq!(env.state().borrow().lua_errors.len(), errors_before);
    assert!(!env.service_ui_transition(&mut None).unwrap());
}

#[test]
fn missing_host_loader_preserves_request_without_changing_the_ui() {
    let (_session, mut env) = selected_glue_env();
    env.exec("EnterWorld()").unwrap();
    let error = env.service_ui_transition(&mut None).unwrap_err();
    assert_eq!(
        error.to_string(),
        "host world entry requires a startup loader"
    );
    assert_eq!(env.state().borrow().screen_kind, ScreenKind::Login);
    assert!(env.is_ui_environment_usable());
    assert!(!env.has_host_world_entry_identity());
    env.set_reload_loader(|_, _| Ok(()));
    assert!(env.service_ui_transition(&mut None).unwrap());
    assert_eq!(env.state().borrow().player.name, "Roster");
}

#[test]
fn failed_host_bootstrap_marks_the_environment_incomplete() {
    let (_session, mut env) = selected_glue_env();
    env.set_reload_loader(|_, _| Err(crate::Error::Other("host fixture failure".into())));
    env.exec("EnterWorld()").unwrap();
    assert!(env.service_ui_transition(&mut None).is_err());
    assert!(!env.is_ui_environment_usable());
}

/// Queueing and one request per owner boundary are host policy, not native cadence.
fn queue_two_host_entries(env: &WowLuaEnv) {
    env.exec("EnterWorld()").unwrap();
    env.state().borrow_mut().glue_login.characters[0]
        .record
        .as_mut()
        .unwrap()
        .account
        .character_name = "Second Roster".into();
    env.exec("EnterWorld()").unwrap();
    // Later source mutation must not alter either accepted snapshot.
    env.state().borrow_mut().glue_login.characters.clear();
}

/// The host intentionally installs Game APIs before the shared loader runs.
/// This does not establish native Glue VM lifetime or native API absence.
fn assert_host_game_registration(env: &WowLuaEnv, expected_name: &str) {
    assert_eq!(env.state().borrow().screen_kind, ScreenKind::Game);
    assert!(env.has_host_world_entry_identity());
    assert!(env.is_host_world_entry_bootstrap());
    assert!(
        env.eval::<bool>(
            "return InGlue() == false and EnterWorld == nil and type(UnitName) == 'function'"
        )
        .unwrap()
    );
    assert_eq!(
        env.eval::<String>("return UnitName('player')").unwrap(),
        expected_name
    );
    assert_eq!(env.state().borrow().player.level, 42);
    assert_eq!(env.eval::<i32>("return UnitLevel('player')").unwrap(), 42);
}

#[test]
fn host_queue_services_two_owned_records_at_separate_owner_boundaries() {
    let (_session, mut env) = selected_glue_env();
    env.state().borrow_mut().player.level = 42;
    let loads = Rc::new(Cell::new(0));
    let loaded = Rc::clone(&loads);
    env.set_reload_loader(move |env, _| {
        let expected = match loaded.get() {
            0 => "Roster",
            1 => "Second Roster",
            _ => panic!("unexpected extra host bootstrap"),
        };
        assert_host_game_registration(env, expected);
        loaded.set(loaded.get() + 1);
        Ok(())
    });
    queue_two_host_entries(&env);
    assert_eq!(loads.get(), 0);
    assert!(env.service_ui_transition(&mut None).unwrap());
    assert_eq!(loads.get(), 1);
    assert_eq!(env.state().borrow().player.name, "Roster");
    assert!(env.service_ui_transition(&mut None).unwrap());
    assert_eq!(loads.get(), 2);
    assert_eq!(env.state().borrow().player.name, "Second Roster");
    assert!(env.is_ui_environment_usable());
    assert!(!env.service_ui_transition(&mut None).unwrap());
    assert_eq!(loads.get(), 2);
}

#[test]
fn second_queued_host_entry_recovers_from_first_loader_failure() {
    let (_session, mut env) = selected_glue_env();
    env.state().borrow_mut().player.level = 42;
    let attempts = Rc::new(Cell::new(0));
    let attempted = Rc::clone(&attempts);
    env.set_reload_loader(move |env, _| {
        let attempt = attempted.get();
        attempted.set(attempt + 1);
        match attempt {
            0 => {
                assert_host_game_registration(env, "Roster");
                env.exec("FailedHostVmSentinel = true")?;
                Err(crate::Error::Other("first host bootstrap failed".into()))
            }
            1 => {
                assert_host_game_registration(env, "Second Roster");
                env.exec("assert(FailedHostVmSentinel == nil)")
            }
            _ => panic!("unexpected extra host bootstrap"),
        }
    });
    queue_two_host_entries(&env);
    let error = env.service_ui_transition(&mut None).unwrap_err();
    assert_eq!(error.to_string(), "first host bootstrap failed");
    assert_eq!(attempts.get(), 1);
    assert!(!env.is_ui_environment_usable());
    assert!(env.service_ui_transition(&mut None).unwrap());
    assert_eq!(attempts.get(), 2);
    assert!(env.is_ui_environment_usable());
    assert_eq!(env.state().borrow().player.name, "Second Roster");
    assert!(!env.service_ui_transition(&mut None).unwrap());
}

#[test]
fn host_world_entry_takes_priority_and_retires_a_simultaneous_reload_request() {
    let (_session, mut env) = selected_glue_env();
    env.state().borrow_mut().player.level = 42;
    let loads = Rc::new(Cell::new(0));
    let loaded = Rc::clone(&loads);
    env.set_reload_loader(move |env, _| {
        assert_host_game_registration(env, "Roster");
        assert!(!env.state().borrow().ui_reload_requested);
        assert!(!env.state().borrow().ui_reloaded);
        loaded.set(loaded.get() + 1);
        Ok(())
    });
    env.exec("ReloadUI(); EnterWorld()").unwrap();
    assert!(env.state().borrow().ui_reload_requested);
    // Explicit host arbitration: world entry replaces the old UI and retires
    // its pending reload, rather than reconstructing Game twice. Not native proof.
    assert!(env.service_ui_transition(&mut None).unwrap());
    assert_eq!(loads.get(), 1);
    assert!(env.is_ui_environment_usable());
    assert!(!env.service_ui_transition(&mut None).unwrap());
    assert_eq!(loads.get(), 1);
}
