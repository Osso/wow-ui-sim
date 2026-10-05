#![cfg(feature = "retail-12-0-7")]
//! Retail 12.0.7 `C_PartyInfo.ConfirmReadyCheck`: AllowedWhenUntainted secret
//! arguments and a non-nilable boolean, authenticated before the lockdown check.
use rilua::LuaApiMut;
use wow_ui_sim::lua_api::WowLuaEnv;

fn observed_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    for (name, payload) in [("SecretReady", true), ("SecretExtra", false)] {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        let value = rilua::table_security::wrap_secret(lua.state_mut(), rilua::Val::Bool(payload))
            .expect("host opaque boolean");
        lua.state_mut().push(value);
        lua.set_global_val(name, value).unwrap();
        lua.state_mut().pop();
    }
    env.exec(
        r#"
        ReadyEvents = {}
        local observer = CreateFrame('Frame')
        observer:RegisterEvent('READY_CHECK_CONFIRM')
        observer:SetScript('OnEvent', function(_, _, unit, ready)
            ReadyEvents[#ReadyEvents + 1] = {unit = unit, ready = ready}
        end)
        C_PartyInfo.DoReadyCheck()
        "#,
    )
    .unwrap();
    env
}

#[test]
fn untainted_caller_confirms_with_secret_boolean() {
    let env = observed_env();
    env.exec(
        r#"
        assert(issecure() and issecretvalue(SecretReady))
        C_PartyInfo.ConfirmReadyCheck(SecretReady)
        assert(#ReadyEvents == 1 and ReadyEvents[1].unit == 'player')
        assert(ReadyEvents[1].ready == true and not issecretvalue(ReadyEvents[1].ready))
        assert(GetReadyCheckStatus('player') == 'ready')
        assert(issecretvalue(SecretReady))
        "#,
    )
    .unwrap();
}

#[test]
fn tainted_secret_arguments_are_rejected_without_state_change() {
    let env = observed_env();
    env.exec(
        r#"
        local function addon()
            assert(not pcall(C_PartyInfo.ConfirmReadyCheck, SecretReady))
            assert(not pcall(C_PartyInfo.ConfirmReadyCheck, true, SecretExtra))
            assert(debug.getstacktaint() == 'ReadyProbe')
            assert(#ReadyEvents == 0 and GetReadyCheckStatus('player') == 'waiting')
            C_PartyInfo.ConfirmReadyCheck(false)
            assert(#ReadyEvents == 1 and ReadyEvents[1].ready == false)
        end
        debug.setobjecttaint(addon, 'ReadyProbe')
        addon()
        "#,
    )
    .unwrap();
}

#[test]
fn non_boolean_response_is_rejected_before_lockdown() {
    let env = observed_env();
    env.state().borrow_mut().chat_messaging_lockdown = true;
    env.exec(
        r#"
        local ok, err = pcall(C_PartyInfo.ConfirmReadyCheck)
        assert(not ok and string.find(err, 'isReady must be a boolean', 1, true))
        ok, err = pcall(C_PartyInfo.ConfirmReadyCheck, 1)
        assert(not ok and string.find(err, 'isReady must be a boolean', 1, true))
        ok, err = pcall(C_PartyInfo.ConfirmReadyCheck, true)
        assert(not ok and string.find(err, 'lockdown', 1, true))
        assert(#ReadyEvents == 0 and GetReadyCheckStatus('player') == 'waiting')
        "#,
    )
    .unwrap();
}
