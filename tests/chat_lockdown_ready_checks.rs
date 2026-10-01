//! Input-only fixtures: actual compiled RED and ready-check guards remain parent-owned.
#![cfg(all(feature = "retail-12-0-5", feature = "profile-retail"))]

use wow_ui_sim::lua_api::WowLuaEnv;

fn create_observed_env(combat: bool) -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create ready-check lockdown environment");
    env.state().borrow_mut().player.in_combat = combat;
    env.exec(
        r#"
        readyEvents = {}
        readyObserver = CreateFrame("Frame")
        readyObserver:RegisterEvent("READY_CHECK")
        readyObserver:RegisterEvent("READY_CHECK_CONFIRM")
        readyObserver:RegisterEvent("READY_CHECK_FINISHED")
        readyObserver:SetScript("OnEvent", function(_, event, ...)
            table.insert(readyEvents, {
                event = event, count = select('#', ...), payload = { ... },
                status = GetReadyCheckStatus("player"),
                timeLeft = GetReadyCheckTimeLeft(),
            })
        end)
        function assertReadyEvent(index, event, count, status, active, response)
            local observed = readyEvents[index]
            assert(observed ~= nil, "missing ready-check event")
            assert(observed.event == event, "wrong ready-check event order")
            assert(observed.count == count, "wrong ready-check payload arity")
            assert(observed.status == status, "handler saw wrong ready-check status")
            assert((observed.timeLeft > 0) == active, "handler saw wrong active state")
            if count == 2 then
                assert(observed.payload[1] == "player", "wrong confirming unit")
                assert(observed.payload[2] == response, "wrong confirming response")
            end
        end
        "#,
    )
    .expect("register real ready-check event observer");
    env
}

fn set_ready_input(env: &WowLuaEnv, restricted: bool, active: bool, response: Option<bool>) {
    let mut state = env.state().borrow_mut();
    state.chat_messaging_lockdown = restricted;
    state.ready_check.active = active;
    state.ready_check.response = response;
}

fn assert_ready_state(env: &WowLuaEnv, active: bool, response: Option<bool>) {
    {
        let state = env.state().borrow();
        assert_eq!(state.ready_check.active, active);
        assert_eq!(state.ready_check.response, response);
    }
    let status = match response {
        Some(true) => "'ready'",
        Some(false) => "'notready'",
        None if active => "'waiting'",
        None => "nil",
    };
    env.exec(&format!(
        r#"
        assert(GetReadyCheckStatus("player") == {status}, "wrong queried status")
        assert((GetReadyCheckTimeLeft() > 0) == {active}, "wrong queried active state")
        "#,
    ))
    .expect("query ready-check state");
}

fn assert_blocked(env: &WowLuaEnv, operation: &str, active: bool, response: Option<bool>) {
    env.exec(&format!(
        r#"
        blockedOk, blockedError = pcall(function() {operation} end)
        "#,
    ))
    .expect("capture restriction failure through pcall");
    assert_ready_state(env, active, response);
    env.exec(
        r#"
        assert(#readyEvents == 0, "blocked operation dispatched a ready-check event")
        assert(blockedOk == false, "lockdown must explicitly reject the operation")
        assert(blockedError ~= nil, "restriction must report an error")
        "#,
    )
    .expect("blocked call leaves all observed events untouched and reports failure");
}

fn assert_unlocked_start(env: &WowLuaEnv, operation: &str) {
    env.exec(&format!(
        r#"
        local ok, err = pcall(function() {operation} end)
        assert(ok, tostring(err))
        assert(#readyEvents == 1, "start must dispatch exactly one event")
        assertReadyEvent(1, "READY_CHECK", 0, "waiting", true)
        "#,
    ))
    .expect("unlocked start publishes existing synchronous behavior");
    assert_ready_state(env, true, None);
}

fn assert_unlocked_confirm(env: &WowLuaEnv, response: bool, previous_events: usize) {
    let status = if response { "ready" } else { "notready" };
    let confirm_index = previous_events + 1;
    let finished_index = previous_events + 2;
    env.exec(&format!(
        r#"
        local ok, err = pcall(C_PartyInfo.ConfirmReadyCheck, {response})
        assert(ok, tostring(err))
        assert(#readyEvents == {finished_index}, "confirm must dispatch exactly two events")
        assertReadyEvent({confirm_index}, "READY_CHECK_CONFIRM", 2, "{status}", false, {response})
        assertReadyEvent({finished_index}, "READY_CHECK_FINISHED", 0, "{status}", false)
        "#,
    ))
    .expect("unlocked confirmation publishes response and finish payloads");
    assert_ready_state(env, false, Some(response));
}

#[test]
fn public_start_blocks_fresh_state_on_both_combat_axes() {
    for combat in [false, true] {
        let env = create_observed_env(combat);
        set_ready_input(&env, true, false, None);
        assert_blocked(&env, "C_PartyInfo.DoReadyCheck()", false, None);
    }
}

#[test]
fn public_confirm_blocks_inactive_state_for_both_responses_and_combat_axes() {
    for combat in [false, true] {
        for response in [false, true] {
            let env = create_observed_env(combat);
            set_ready_input(&env, true, false, None);
            assert_blocked(
                &env,
                &format!("C_PartyInfo.ConfirmReadyCheck({response})"),
                false,
                None,
            );
        }
    }
}

#[test]
fn public_start_preserves_active_response_then_recovers_after_unlock() {
    for combat in [false, true] {
        for response in [false, true] {
            let env = create_observed_env(combat);
            set_ready_input(&env, true, true, Some(response));
            assert_ready_state(&env, true, Some(response));
            assert_blocked(&env, "C_PartyInfo.DoReadyCheck()", true, Some(response));
            env.state().borrow_mut().chat_messaging_lockdown = false;
            assert_unlocked_start(&env, "C_PartyInfo.DoReadyCheck()");
            assert_unlocked_confirm(&env, response, 1);
            assert_eq!(env.state().borrow().player.in_combat, combat);
        }
    }
}

#[test]
fn public_confirm_preserves_active_response_then_recovers_after_unlock() {
    for combat in [false, true] {
        for response in [false, true] {
            let env = create_observed_env(combat);
            let previous_response = !response;
            set_ready_input(&env, true, true, Some(previous_response));
            assert_ready_state(&env, true, Some(previous_response));
            assert_blocked(
                &env,
                &format!("C_PartyInfo.ConfirmReadyCheck({response})"),
                true,
                Some(previous_response),
            );
            env.state().borrow_mut().chat_messaging_lockdown = false;
            assert_unlocked_confirm(&env, response, 0);
            assert_eq!(env.state().borrow().player.in_combat, combat);
        }
    }
}

#[test]
fn unlocked_public_lifecycle_ignores_combat_for_both_responses() {
    for combat in [false, true] {
        for response in [false, true] {
            let env = create_observed_env(combat);
            set_ready_input(&env, false, false, None);
            assert_ready_state(&env, false, None);
            assert_unlocked_start(&env, "C_PartyInfo.DoReadyCheck()");
            assert_unlocked_confirm(&env, response, 1);
            assert_eq!(env.state().borrow().player.in_combat, combat);
        }
    }
}

#[test]
fn legacy_ready_check_cannot_bypass_lockdown_and_recovers_after_unlock() {
    for combat in [false, true] {
        for (active, response) in [(false, None), (true, Some(false)), (true, Some(true))] {
            let env = create_observed_env(combat);
            set_ready_input(&env, true, active, response);
            assert_ready_state(&env, active, response);
            assert_blocked(&env, "ReadyCheck()", active, response);
            env.state().borrow_mut().chat_messaging_lockdown = false;
            assert_unlocked_start(&env, "ReadyCheck()");
            assert_unlocked_confirm(&env, true, 1);
            assert_eq!(env.state().borrow().player.in_combat, combat);
        }
    }
}
