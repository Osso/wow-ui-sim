//! Observable contracts unlocked by the pinned rilua host primitives.
#![cfg(feature = "retail-12-0-0")]

use rilua::LuaApiMut;
use wow_ui_sim::lua_api::{AddonInfo, WowLuaEnv};

fn secret_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).unwrap();
    let secret =
        rilua::table_security::wrap_host_secret_string(lua.state_mut(), "Alessio-Silvermoon");
    lua.state_mut().push(secret);
    lua.set_global_val("SecretName", secret).unwrap();
    lua.state_mut().pop();
    drop(lua);
    env
}

#[test]
fn rilua_rows_drop_access_denies_descendants_without_taint_and_recovers() {
    secret_env()
        .exec(
            r#"
        local function denied()
            assert(canaccesssecrets() and issecure())
            assert(secretunwrap(SecretName) == 'Alessio-Silvermoon')
            assert(select('#', dropsecretaccess()) == 0)
            assert(not canaccesssecrets() and issecure())
            local function probe()
                assert(not canaccesssecrets())
                local ok, err = pcall(secretunwrap, SecretName)
                assert(not ok and string.find(err, 'revoked access', 1, true))
            end
            probe()
            securecallfunction(probe)
            local thread = coroutine.create(probe)
            assert(coroutine.resume(thread))
            assert(debug.getstacktaint() == nil)
        end
        denied()
        assert(canaccesssecrets() and issecure())
        assert(secretunwrap(SecretName) == 'Alessio-Silvermoon')
        local function addon()
            local before = debug.getstacktaint()
            assert(not canaccesssecrets())
            dropsecretaccess()
            assert(not canaccesssecrets() and debug.getstacktaint() == before)
        end
        debug.setobjecttaint(addon, 'RevokedAddon'); addon()
        assert(canaccesssecrets() and debug.getstacktaint() == nil)
        local function fail()
            dropsecretaccess()
            error('fixture failure')
        end
        assert(not pcall(fail))
        assert(canaccesssecrets())
    "#,
        )
        .unwrap();
}

#[test]
fn rilua_rows_secret_table_predicate_classifies_metadata_in_both_callers() {
    secret_env()
        .exec(
            r#"
        local ordinary = {value = secretwrap(17)}
        local wrapped = secretwrap({value = 17})
        local contents = {value = 17}
        settablesecurity(contents, 2)
        local function probe()
            local before = debug.getstacktaint()
            assert(issecrettable(wrapped) and issecrettable(contents))
            assert(not issecrettable(ordinary) and not issecrettable(SecretName))
            assert(not issecrettable(nil) and not issecrettable(17))
            assert(issecretvalue(contents.value))
            assert(debug.getstacktaint() == before)
        end
        probe()
        local function addon() probe() end
        debug.setobjecttaint(addon, 'TableAddon'); addon()
        collectgarbage('collect'); probe()
        local function revoked() dropsecretaccess(); probe() end
        revoked()
        assert(canaccesssecrets())
    "#,
        )
        .unwrap();
}

#[cfg(feature = "retail-12-0-5")]
#[test]
fn rilua_rows_ambiguate_transforms_secret_names_without_revealing_or_changing_taint() {
    secret_env()
        .exec(
            r#"
        local function probe()
            local before = debug.getstacktaint()
            ShortName = Ambiguate(SecretName, 'short')
            FullName = Ambiguate(SecretName, 'none')
            assert(issecretvalue(ShortName) and issecretvalue(FullName))
            assert(debug.getstacktaint() == before)
            local ok, err = pcall(Ambiguate, nil, SecretName)
            assert(not ok and string.find(err, 'argument #2 must not be secret', 1, true))
        end
        probe()
        assert(secretunwrap(ShortName) == 'Alessio')
        assert(secretunwrap(FullName) == 'Alessio-Silvermoon')
        local function addon() probe(); assert(not pcall(secretunwrap, ShortName)) end
        debug.setobjecttaint(addon, 'NameAddon'); addon()
        assert(secretunwrap(ShortName) == 'Alessio')
        local function revoked() dropsecretaccess(); probe() end
        revoked()
        collectgarbage('collect')
        assert(secretunwrap(ShortName) == 'Alessio')
        assert(debug.getstacktaint() == nil)
    "#,
        )
        .unwrap();
}

#[cfg(feature = "retail-12-0-5")]
#[test]
fn rilua_rows_chat_expressions_replace_icons_and_reject_secret_flags() {
    secret_env().exec(r#"
        local text = '{rt1} hello'
        local expected = '|TInterface\\TargetingFrame\\UI-RaidTargetingIcon_1:0|t hello'
        assert(C_ChatInfo.ReplaceIconAndGroupExpressions(text) == expected)
        local secret = secretwrap(text)
        local secretFlag = secretwrap(false)
        local function probe()
            local before = debug.getstacktaint()
            ChatExpanded = C_ChatInfo.ReplaceIconAndGroupExpressions(secret)
            ChatUnchanged = C_ChatInfo.ReplaceIconAndGroupExpressions(secret, true)
            assert(issecretvalue(ChatExpanded) and issecretvalue(ChatUnchanged))
            for index = 2,3 do
                local ok, err
                if index == 2 then
                    ok, err = pcall(C_ChatInfo.ReplaceIconAndGroupExpressions, nil, secretFlag)
                else
                    ok, err = pcall(C_ChatInfo.ReplaceIconAndGroupExpressions, nil, false, secretFlag)
                end
                assert(not ok and string.find(err, 'must not be secret', 1, true))
            end
            assert(debug.getstacktaint() == before)
        end
        probe()
        assert(secretunwrap(ChatExpanded) == expected and secretunwrap(ChatUnchanged) == text)
        local function addon() probe(); assert(not pcall(secretunwrap, ChatExpanded)) end
        debug.setobjecttaint(addon, 'ChatAddon'); addon()
        assert(secretunwrap(ChatExpanded) == expected)
        local function revoked() dropsecretaccess(); probe() end
        revoked()
        assert(secretunwrap(ChatExpanded) == expected)
    "#).unwrap();
}

#[cfg(feature = "retail-12-0-5")]
fn budget_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut sim = env.state().borrow_mut();
        let index = sim.addons.len() as u16;
        sim.addons.push(AddonInfo {
            folder_name: "BudgetAddon".into(),
            ..Default::default()
        });
        sim.loading_addon_index = Some(index);
    }
    env.exec(
        r#"
        BudgetFrame = CreateFrame('Frame', 'BudgetFrame')
        BudgetFrame:RegisterEvent('PLAYER_LOGOUT')
        BudgetFrame:RegisterEvent('ADDONS_UNLOADING')
        BudgetFrame:RegisterEvent('PLAYER_LOGIN')
        Completed = 0
        Iterations = 1000
        seterrorhandler(function(err) BudgetError = err end)
        local function handler(self, event)
            assert(debug.getstacktaint() == 'BudgetAddon')
            local n = 0
            for i=1,Iterations do n = n + i end
            Completed = Completed + 1
        end
        debug.setobjecttaint(handler, 'BudgetAddon')
        BudgetFrame:SetScript('OnEvent', handler)
    "#,
    )
    .unwrap();
    env.state().borrow_mut().loading_addon_index = None;
    env.loader_env()
        .with_state(|state| {
            state.set_instruction_budget("BudgetAddon", Some(1000));
            Ok::<_, rilua::LuaError>(())
        })
        .unwrap();
    env
}

#[cfg(feature = "retail-12-0-5")]
fn assert_event_budget(named: bool) {
    let env = budget_env();
    let dispatch = |event| {
        if named {
            env.loader_env().fire_event_with_args(event, &[])
        } else {
            env.fire_event(event)
        }
        .unwrap();
    };
    dispatch("PLAYER_LOGIN");
    env.exec("assert(Completed == 0, 'completed='..tostring(Completed)..' error='..tostring(BudgetError)); assert(string.find(BudgetError, 'instruction budget exhausted', 1, true), tostring(BudgetError))").unwrap();
    dispatch("PLAYER_LOGOUT");
    dispatch("ADDONS_UNLOADING");
    env.exec("assert(Completed == 2)").unwrap();
    dispatch("PLAYER_LOGIN");
    env.exec("assert(Completed == 2); Iterations = 1").unwrap();
    env.loader_env()
        .with_state(|state| state.reset_instruction_usage("BudgetAddon"))
        .unwrap();
    dispatch("PLAYER_LOGIN");
    env.exec("assert(Completed == 3, 'completed='..tostring(Completed)..' error='..tostring(BudgetError)); assert(debug.getstacktaint() == nil)")
        .unwrap();
}

#[cfg(feature = "retail-12-0-5")]
#[test]
fn rilua_rows_host_event_budget_exhaustion_shutdown_exemption_and_recovery() {
    assert_event_budget(false);
}

#[cfg(feature = "retail-12-0-5")]
#[test]
fn rilua_rows_named_event_budget_exhaustion_shutdown_exemption_and_recovery() {
    assert_event_budget(true);
}

#[cfg(feature = "retail-12-0-5")]
#[test]
fn rilua_rows_chat_host_vocabulary_flags_live_updates_and_isolation() {
    let env = secret_env();
    {
        let mut state = env.state().borrow_mut();
        state
            .chat_expression_inputs
            .icons
            .insert(b"fixture-icon".to_vec(), b"ICON".to_vec());
        state
            .chat_expression_inputs
            .groups
            .insert(b"g1".to_vec(), b"Alessio, Osso".to_vec());
    }
    env.exec(r#"
        local text = '{fixture-icon} {G1} {unknown} {'
        assert(C_ChatInfo.ReplaceIconAndGroupExpressions(text) == 'ICON Alessio, Osso {unknown} {')
        assert(C_ChatInfo.ReplaceIconAndGroupExpressions(text, nil, false) == 'ICON Alessio, Osso {unknown} {')
        assert(C_ChatInfo.ReplaceIconAndGroupExpressions(text, true) == '{fixture-icon} Alessio, Osso {unknown} {')
        assert(C_ChatInfo.ReplaceIconAndGroupExpressions(text, false, true) == 'ICON {G1} {unknown} {')
        assert(C_ChatInfo.ReplaceIconAndGroupExpressions(text, true, true) == text)
        local secret = secretwrap(text)
        local function addon()
            GroupText = C_ChatInfo.ReplaceIconAndGroupExpressions(secret)
            assert(issecretvalue(GroupText) and not pcall(secretunwrap, GroupText))
            assert(debug.getstacktaint() == 'GroupAddon')
        end
        debug.setobjecttaint(addon, 'GroupAddon'); addon()
        assert(secretunwrap(GroupText) == 'ICON Alessio, Osso {unknown} {')
    "#).unwrap();
    env.state()
        .borrow_mut()
        .chat_expression_inputs
        .groups
        .insert(b"g1".to_vec(), b"Osso".to_vec());
    env.exec("assert(C_ChatInfo.ReplaceIconAndGroupExpressions('{g1}') == 'Osso')")
        .unwrap();
    secret_env().exec("assert(C_ChatInfo.ReplaceIconAndGroupExpressions('{fixture-icon} {g1}') == '{fixture-icon} {g1}')").unwrap();
}

#[cfg(feature = "retail-12-0-5")]
#[test]
fn rilua_rows_string_transforms_preserve_bytes_and_reject_nonstring_secrets() {
    secret_env().exec(r#"
        local binary = string.char(255, 0)..'{rt1}'
        local secret = secretwrap(binary)
        -- Create typed wrappers while secure; addon callers cannot create them.
        local badValues = {secretwrap(17), secretwrap(true), secretwrap({})}
        local function probe()
            BinaryChat = C_ChatInfo.ReplaceIconAndGroupExpressions(secret)
            assert(issecretvalue(BinaryChat))
            for _, value in ipairs(badValues) do
                assert(not pcall(Ambiguate, value, 'short'))
                assert(not pcall(C_ChatInfo.ReplaceIconAndGroupExpressions, value))
            end
            assert(debug.getstacktaint() == 'BinaryAddon')
        end
        debug.setobjecttaint(probe, 'BinaryAddon'); probe()
        collectgarbage('collect')
        assert(secretunwrap(BinaryChat) == string.char(255, 0)..'|TInterface\\TargetingFrame\\UI-RaidTargetingIcon_1:0|t')
        assert(secretunwrap(secret) == binary)
        assert(not pcall(C_ChatInfo.ReplaceIconAndGroupExpressions, 'text', 1))
        assert(not pcall(C_ChatInfo.ReplaceIconAndGroupExpressions, 'text', false, {}))
    "#).unwrap();
}
