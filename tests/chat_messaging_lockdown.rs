//! Pending RED fixtures for the Retail 12.0.5 single-boolean chat lockdown contract.
#![cfg(all(feature = "retail-12-0-5", feature = "profile-retail"))]

use wow_ui_sim::lua_api::WowLuaEnv;

fn assert_lockdown(env: &WowLuaEnv, expected: bool) {
    env.exec(&format!(
        r#"
        assert(select('#', C_ChatInfo.InChatMessagingLockdown()) == 1,
            "chat lockdown must return exactly one value")
        local restricted, reason = C_ChatInfo.InChatMessagingLockdown()
        assert(type(restricted) == "boolean", "chat lockdown must return a boolean")
        assert(restricted == {expected}, "chat lockdown must match explicit input")
        assert(reason == nil, "chat lockdown must not return a second reason")
        "#,
    ))
    .expect("single boolean chat lockdown result");
}

#[test]
fn fresh_state_returns_exactly_one_false() {
    let env = WowLuaEnv::new().expect("create chat lockdown environment");
    assert!(!env.state().borrow().chat_messaging_lockdown);
    assert_lockdown(&env, false);
}

#[test]
fn explicit_true_false_true_transitions_return_single_boolean() {
    let env = WowLuaEnv::new().expect("create chat lockdown environment");
    for restricted in [true, false, true] {
        env.state().borrow_mut().chat_messaging_lockdown = restricted;
        assert_lockdown(&env, restricted);
    }
}

#[test]
fn all_combat_and_lockdown_combinations_remain_independent() {
    let env = WowLuaEnv::new().expect("create chat lockdown environment");
    for (combat, restricted) in [(false, false), (false, true), (true, false), (true, true)] {
        {
            let mut state = env.state().borrow_mut();
            state.player.in_combat = combat;
            state.chat_messaging_lockdown = restricted;
        }
        assert_lockdown(&env, restricted);
        assert_eq!(env.state().borrow().player.in_combat, combat);
    }
}

#[test]
fn lockdown_input_is_isolated_between_environments() {
    let first = WowLuaEnv::new().expect("create first chat lockdown environment");
    let second = WowLuaEnv::new().expect("create second chat lockdown environment");
    first.state().borrow_mut().chat_messaging_lockdown = true;
    assert_lockdown(&first, true);
    assert_lockdown(&second, false);
    second.state().borrow_mut().chat_messaging_lockdown = true;
    first.state().borrow_mut().chat_messaging_lockdown = false;
    assert_lockdown(&first, false);
    assert_lockdown(&second, true);
}

#[test]
fn ordinary_addon_caller_preserves_stack_taint() {
    let env = WowLuaEnv::new().expect("create chat lockdown environment");
    for restricted in [false, true] {
        env.state().borrow_mut().chat_messaging_lockdown = restricted;
        env.exec(&format!(
            r#"
            local function addon()
                assert(debug.getstacktaint() == "ChatLockdownFixture")
                assert(select('#', C_ChatInfo.InChatMessagingLockdown()) == 1)
                local value, reason = C_ChatInfo.InChatMessagingLockdown()
                assert(type(value) == "boolean" and value == {restricted})
                assert(reason == nil)
                assert(debug.getstacktaint() == "ChatLockdownFixture",
                    "chat lockdown query must preserve ordinary caller taint")
            end
            debug.setobjecttaint(addon, "ChatLockdownFixture")
            addon()
            "#,
        ))
        .expect("ordinary addon query preserves caller taint");
    }
}
