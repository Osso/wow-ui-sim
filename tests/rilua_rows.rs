//! Observable contracts unlocked by the pinned rilua host primitives.
#![cfg(feature = "retail-12-0-0")]

use rilua::LuaApiMut;
use wow_ui_sim::lua_api::WowLuaEnv;

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

#[test]
fn rilua_rows_bucket_uses_normal_limits_out_of_combat() {
    let directory = tempfile::tempdir().unwrap();
    let toc = create_script_probe(
        directory.path(),
        "NormalThrottleProbe",
        "local frame = CreateFrame('Frame'); frame:RegisterEvent('PLAYER_ENTERING_WORLD'); \
         frame:SetScript('OnEvent', function() \
             ThrottleCalls = (ThrottleCalls or 0) + 1; \
             local total = 0; \
             for i = 1, 200000 do total = total + i end \
         end)",
    );
    let env = WowLuaEnv::new().unwrap();
    env.exec("SetCVar('luaScriptBucketThrottleEnabled', '0'); seterrorhandler(function(err) ThrottleError = err end)")
        .unwrap();
    let loader = env.loader_env();
    wow_ui_sim::loader::load_addon(&loader, &toc).unwrap();
    // Out of combat the Normal limits apply, whatever the Restricted ones are.
    env.exec("SetCVar('luaScriptBucketThrottleEnabled', '1'); SetCVar('luaScriptBucketThrottleMaxMsPerSecondNormal', '0'); SetCVar('luaScriptBucketThrottleMaxMsBurstNormal', '1'); ThrottleCalls = 0")
        .unwrap();

    env.fire_event("PLAYER_ENTERING_WORLD").unwrap();
    env.exec("assert(ThrottleCalls == 1 and ThrottleError == nil)")
        .unwrap();
    env.fire_event("PLAYER_ENTERING_WORLD").unwrap();
    env.exec("assert(ThrottleCalls == 2 and string.find(ThrottleError, 'insecure scripts exceeded execution limit for addon NormalThrottleProbe', 1, true))")
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

fn create_script_probe(
    root: &std::path::Path,
    addon_name: &str,
    source: &str,
) -> std::path::PathBuf {
    let addon_dir = root.join(addon_name);
    std::fs::create_dir_all(&addon_dir).unwrap();
    let toc_path = addon_dir.join(format!("{addon_name}.toc"));
    std::fs::write(
        &toc_path,
        format!("## Interface: 120000\n{addon_name}.lua\n"),
    )
    .unwrap();
    std::fs::write(addon_dir.join(format!("{addon_name}.lua")), source).unwrap();
    toc_path
}

#[test]
fn rilua_rows_owner_watchdog_loads_file_then_rejects_dispatch_overrun() {
    let directory = tempfile::tempdir().unwrap();
    let toc = create_script_probe(
        directory.path(),
        "WatchdogProbe",
        "local frame = CreateFrame('Frame'); frame:RegisterEvent('PLAYER_LOGIN'); \
         frame:SetScript('OnEvent', function() while true do end end)",
    );
    let env = WowLuaEnv::new().unwrap();
    env.set_script_watchdog_timeout_for_tests(std::time::Duration::from_secs(60));
    env.exec("SetCVar('luaScriptBucketThrottleEnabled', '0'); seterrorhandler(function(err) WatchdogError = err end)")
        .unwrap();
    let loader = env.loader_env();
    let loaded = wow_ui_sim::loader::load_addon(&loader, &toc).unwrap();
    assert_eq!(loaded.lua_files, 1);
    env.set_script_watchdog_timeout_for_tests(std::time::Duration::ZERO);
    env.fire_event("PLAYER_LOGIN").unwrap();
    env.exec("assert(string.find(WatchdogError, 'script ran too long', 1, true))")
        .unwrap();
}

#[test]
fn rilua_rows_bucket_skips_loading_and_throttles_after_world_entry() {
    let directory = tempfile::tempdir().unwrap();
    let toc = create_script_probe(
        directory.path(),
        "ThrottleProbe",
        "local frame = CreateFrame('Frame'); frame:RegisterEvent('PLAYER_ENTERING_WORLD'); \
         frame:SetScript('OnEvent', function() \
             ThrottleCalls = (ThrottleCalls or 0) + 1; \
             local total = 0; \
             for i = 1, 200000 do total = total + i end; \
             ThrottleTotal = total \
         end)",
    );
    let env = WowLuaEnv::new().unwrap();
    env.exec("SetCVar('luaScriptBucketThrottleEnabled', '0'); seterrorhandler(function(err) ThrottleError = err end)")
        .unwrap();
    let loader = env.loader_env();
    wow_ui_sim::loader::load_addon(&loader, &toc).unwrap();
    env.exec("SetCVar('luaScriptBucketThrottleEnabled', '1'); SetCVar('luaScriptBucketThrottleMaxMsPerSecondRestricted', '0'); SetCVar('luaScriptBucketThrottleMaxMsBurstRestricted', '1'); ThrottleCalls = 0")
        .unwrap();
    env.state().borrow_mut().player.in_combat = true;

    env.fire_event("PLAYER_ENTERING_WORLD").unwrap();
    env.exec("assert(ThrottleCalls == 1 and ThrottleError == nil)")
        .unwrap();
    env.fire_event("PLAYER_ENTERING_WORLD").unwrap();
    env.exec("assert(ThrottleCalls == 2 and string.find(ThrottleError, 'insecure scripts exceeded execution limit for addon ThrottleProbe', 1, true))")
        .unwrap();
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
    "#).unwrap();
    assert_secret_bytes(&env, "GroupText", b"ICON Alessio, Osso {unknown} {");
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
    let env = secret_env();
    env.exec(
        r#"
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
        assert(secretunwrap(secret) == binary)
        assert(not pcall(C_ChatInfo.ReplaceIconAndGroupExpressions, 'text', 1))
        assert(not pcall(C_ChatInfo.ReplaceIconAndGroupExpressions, 'text', false, {}))
    "#,
    )
    .unwrap();
    assert_secret_bytes(
        &env,
        "BinaryChat",
        b"\xff\0|TInterface\\TargetingFrame\\UI-RaidTargetingIcon_1:0|t",
    );
}

#[cfg(feature = "retail-12-0-5")]
fn assert_secret_bytes(env: &WowLuaEnv, global: &str, expected: &[u8]) {
    // Reading an addon-created global from Lua taints that observer. Inspect the
    // result from the trusted host instead; never clear or override caller taint.
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    let value = lua.get_global_val(global);
    assert!(rilua::table_security::is_secret_value(
        lua.state_mut(),
        value
    ));
    let payload = rilua::table_security::unwrap_secret(lua.state_mut(), value).unwrap();
    let rilua::Val::Str(string) = payload else {
        panic!("expected secret string result")
    };
    assert_eq!(
        lua.state_mut().gc.string_arena.get(string).unwrap().data(),
        expected
    );
}
