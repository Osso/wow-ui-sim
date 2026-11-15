//! R88i.6 VERIFIED request semantics, plus an original runtime-selected consumer.
//! Host throttle inputs are neutral seams, not assertions of native windows.
//! Party return arity is deliberately not asserted (UNKNOWN).
#![cfg(feature = "client-retail")]
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn lfd_lock_requests_api_player_boolean_arity_and_independent_throttles() {
    let env = WowLuaEnv::new().unwrap();
    for (first, second, expected) in [
        (false, false, true),
        (true, false, false),
        (false, true, false),
        (true, true, false),
    ] {
        {
            let mut sim = env.state().borrow_mut();
            sim.lfd_lock_requests.first_throttle_blocked = first;
            sim.lfd_lock_requests.second_throttle_blocked = second;
        }
        // Verified: no argument parsing; even arbitrary surplus values are ignored.
        let (arity, result): (i32, bool) = env.eval(
            "return select('#', RequestLFDPlayerLockInfo({}, 'extra', nil)), RequestLFDPlayerLockInfo(false)"
        ).unwrap();
        assert_eq!(arity, 1);
        assert_eq!(result, expected);
        let party: bool = env
            .eval("return RequestLFDPartyLockInfo({}, 'extra', nil)")
            .unwrap();
        assert_eq!(party, expected);
    }
    // Admitted player calls succeed without a connection and submit nothing.
    assert!(
        env.state()
            .borrow()
            .lfd_lock_requests
            .submitted_player_flags
            .is_empty()
    );
}

#[test]
fn lfd_lock_requests_api_connected_flags_without_synchronous_lock_data() {
    let env = WowLuaEnv::new().unwrap();
    env.state()
        .borrow_mut()
        .lfd_lock_requests
        .connection_present = true;
    env.exec(
        r#"
        LFGLockList = { [42] = {reason = 7} }
        local before = LFGLockList
        local beforeChoice = { GetLFDLockInfo(1, 1) }
        assert(RequestLFDPlayerLockInfo())
        assert(RequestLFDPartyLockInfo())
        assert(LFGLockList == before and LFGLockList[42].reason == 7)
        local afterChoice = { GetLFDLockInfo(1, 1) }
        assert(#beforeChoice == #afterChoice)
        for i, value in ipairs(beforeChoice) do assert(afterChoice[i] == value) end
    "#,
    )
    .unwrap();
    assert_eq!(
        env.state()
            .borrow()
            .lfd_lock_requests
            .submitted_player_flags,
        vec![true, false]
    );
    env.state()
        .borrow_mut()
        .lfd_lock_requests
        .second_throttle_blocked = true;
    env.exec("assert(not RequestLFDPlayerLockInfo()); assert(not RequestLFDPartyLockInfo())")
        .unwrap();
    assert_eq!(
        env.state()
            .borrow()
            .lfd_lock_requests
            .submitted_player_flags,
        vec![true, false]
    );
}

#[test]
fn lfd_lock_requests_original_group_finder_queue_updater_submits_both_flags() {
    use wow_ui_sim::loader::{discover_blizzard_addons_for_screen, find_toc_file};
    use wow_ui_sim::screen::ScreenKind;
    use wow_ui_sim::toc::TocFile;
    let root = wow_ui_sim::client_profile::blizzard_ui_addons_dir();
    let dir = root.join("Blizzard_GroupFinder");
    let selected = find_toc_file(&dir).expect("SETUP: selected GroupFinder TOC");
    assert!(selected.ends_with("Blizzard_GroupFinder_Mainline.toc"));
    assert!(
        discover_blizzard_addons_for_screen(&root, ScreenKind::Game)
            .iter()
            .any(|(name, path)| name == "Blizzard_GroupFinder" && path == &selected)
    );
    let toc = TocFile::from_file(&selected).unwrap();
    assert!(
        toc.files
            .iter()
            .any(|p| p.to_string_lossy().replace('\\', "/") == "Mainline/QueueUpdater.lua")
    );
    let path = dir.join("Mainline/QueueUpdater.lua");
    let env = WowLuaEnv::new().unwrap();
    env.exec_named(
        &std::fs::read_to_string(&path).unwrap(),
        &format!("@{}", path.display()),
    )
    .unwrap();
    env.state()
        .borrow_mut()
        .lfd_lock_requests
        .connection_present = true;
    env.exec("QueueUpdater:RequestInfo()").unwrap();
    assert_eq!(
        env.state()
            .borrow()
            .lfd_lock_requests
            .submitted_player_flags,
        vec![true, false]
    );
    env.state()
        .borrow_mut()
        .lfd_lock_requests
        .first_throttle_blocked = true;
    env.exec("QueueUpdater:RequestInfo()").unwrap();
    assert_eq!(
        env.state()
            .borrow()
            .lfd_lock_requests
            .submitted_player_flags,
        vec![true, false]
    );
}
