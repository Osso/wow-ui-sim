//! March 25 leadership restriction, separate from March 31 chat-lockdown APIs.
#![cfg(feature = "retail-12-0-7")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn leadership_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        LeaveParty()
        InviteToGroup('Ada-Realm')
        InviteToGroup('Bea-Realm')
        C_PartyInfo.PromoteToAssistant('party1')
        calls = {
            function() C_PartyInfo.PromoteToAssistant('party2') end,
            function() C_PartyInfo.DemoteAssistant('party1') end,
            function() C_PartyInfo.PromoteToLeader('party2') end,
            function() C_PartyInfo.SetEveryoneIsAssistant(true) end,
        }
        local function addon(index)
            assert(debug.getstacktaint() == 'LeadershipAddon')
            calls[index]()
            assert(debug.getstacktaint() == 'LeadershipAddon')
        end
        debug.setobjecttaint(addon, 'LeadershipAddon')
        callFromAddon = addon
        "#,
    )
    .unwrap();
    env.state().borrow_mut().events.drain();
    env
}

#[test]
fn leadership_addon_combat_denial_preserves_roles_leader_events_and_caller() {
    let env = leadership_env();
    env.state().borrow_mut().player.in_combat = true;
    env.exec(
        r#"
        for index = 1, #calls do
            local ok, failure = pcall(callFromAddon, index)
            assert(not ok, 'addon leadership mutation must deny during combat: '..index)
            assert(type(failure) == 'string' and #failure > 0)
            assert(debug.getstacktaint() == nil, 'error restores secure caller')
            assert(UnitIsGroupAssistant('party1'))
            assert(not UnitIsGroupAssistant('party2'))
            assert(UnitIsGroupLeader('player'))
            assert(not UnitIsGroupLeader('party2'))
            assert(not IsEveryoneAssistant())
        end
        "#,
    )
    .unwrap();
    assert!(env.state().borrow().events.is_empty());
    env.state().borrow_mut().player.in_combat = false;
    env.exec(
        r#"
        callFromAddon(1)
        assert(UnitIsGroupAssistant('party2'), 'out-of-combat recovery after denial')
        assert(debug.getstacktaint() == nil)
        "#,
    )
    .unwrap();
}

#[test]
fn leadership_secure_combat_and_addon_out_of_combat_calls_mutate_live_state() {
    for combat in [true, false] {
        let env = leadership_env();
        env.state().borrow_mut().player.in_combat = combat;
        let invoke = if combat { "calls[index]()" } else { "callFromAddon(index)" };
        env.exec(&format!(
            r#"
            for index = 1, #calls do
                {invoke}
                assert(debug.getstacktaint() == nil)
            end
            assert(UnitIsGroupAssistant('party2'))
            assert(UnitIsGroupLeader('party2'))
            assert(not UnitIsGroupLeader('player'))
            assert(IsEveryoneAssistant())
            "#,
        ))
        .unwrap();
    }
}
