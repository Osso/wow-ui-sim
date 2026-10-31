#![cfg(feature = "client-wowforever")]

use wow_ui_sim::loader::load_addon;
use wow_ui_sim::lua_api::WowLuaEnv;

fn load_secure_map_source(env: &WowLuaEnv) {
    let path = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path()
        .unwrap()
        .join("Blizzard_CooldownViewer/CooldownViewerSecure.lua");
    let source = std::fs::read_to_string(path).unwrap();
    env.exec(&format!(
        "ForeverSecureMaps = {{}}; (function(...)\n{source}\nend)('Blizzard_CooldownViewer', ForeverSecureMaps)"
    ))
    .unwrap();
}

#[test]
fn forever_vendor_secure_map_preserves_wrapped_key_identity() {
    let env = WowLuaEnv::new().unwrap();
    load_secure_map_source(&env);
    env.exec(
        r#"
        local map = ForeverSecureMaps.CreateSecureAuraInstanceMap()
        local key = secretwrap(1234)
        assert(issecretvalue(key), 'vendor map check 1')
        map[key] = 'aura'
        assert(map[1234] == 'aura', 'vendor map check 2')
        assert(map[secretwrap(1234)] == 'aura', 'vendor map check 3')
        map[1234] = 'updated'
        assert(map[key] == 'updated', 'vendor map check 4')
        local backing = {}
        settablesecurity(backing, Enum.TableSecurityOption.DisallowSecretKeys)
        assert(not pcall(function() backing[key] = 1 end), 'vendor map check 5')
        assert(not pcall(function() return backing[key] end), 'vendor map check 6')
        backing[secretunwrap(key)] = 7
        assert(backing[1234] == 7, 'vendor map check 7')
        assert(not pcall(settablesecurity, {}, Enum.TableSecurityOption.SecretWrapContents), 'vendor map check 8')
        assert(__secureenv.secretwrap == secretwrap, 'vendor map check 9')
        assert(__secureenv.secretunwrap == secretunwrap, 'vendor map check 10')
        assert(__secureenv.settablesecurity == settablesecurity, 'vendor map check 11')
        assert(__secureenv.issecretvalue == issecretvalue, 'vendor map check 12')
    "#,
    )
    .unwrap();
}

#[test]
fn forever_cleanup_restore_preserves_native_security_functions() {
    let env = WowLuaEnv::new().unwrap();
    env.exec("SavedTableSecurity = { settablesecurity, secretwrap, secretunwrap, issecretvalue }")
        .unwrap();
    env.restore_post_cleanup_globals();
    env.exec(
        r#"
        assert(SavedTableSecurity[1] == settablesecurity)
        assert(SavedTableSecurity[2] == secretwrap)
        assert(SavedTableSecurity[3] == secretunwrap)
        assert(SavedTableSecurity[4] == issecretvalue)
        local key = secretwrap(77)
        assert(issecretvalue(key) and secretunwrap(key) == 77)
    "#,
    )
    .unwrap();
}

#[test]
fn forever_secured_map_rejects_explicit_caller_taint() {
    let env = WowLuaEnv::new().unwrap();
    load_secure_map_source(&env);
    env.exec(
        r#"
        local map = ForeverSecureMaps.CreateSecureAuraInstanceMap()
        map[42] = 'safe'
        local ok, message = pcall(function()
            debug.setstacktaint('SecurityProbe')
            return map[42]
        end)
        assert(not ok, 'tainted reader must be rejected')
        assert(string.find(message, "attempted to index a table that cannot be accessed while tainted (execution tainted by 'SecurityProbe')", 1, true), message)
    "#,
    )
    .unwrap();
}

#[test]
fn forever_secured_map_rejects_real_addon_loader_taint() {
    let env = WowLuaEnv::new().unwrap();
    load_secure_map_source(&env);
    env.exec("ForeverProtectedMap = ForeverSecureMaps.CreateSecureAuraInstanceMap(); ForeverProtectedMap[42] = 'safe'").unwrap();
    let root = tempfile::tempdir().unwrap();
    let addon = root.path().join("SecurityProbe");
    std::fs::create_dir(&addon).unwrap();
    let toc = addon.join("SecurityProbe.toc");
    std::fs::write(&toc, "## Title: SecurityProbe\nProbe.lua\n").unwrap();
    std::fs::write(
        addon.join("Probe.lua"),
        r#"
        local ok, message = pcall(function() return ForeverProtectedMap[42] end)
        assert(not ok, 'loader-stamped addon must not read secured map')
        assert(string.find(message, "attempted to index a table that cannot be accessed while tainted (execution tainted by 'SecurityProbe')", 1, true), message)
        ForeverLoaderSecurityRejected = true
    "#,
    )
    .unwrap();
    let result = load_addon(&env.loader_env(), &toc).unwrap();
    assert!(result.warnings.is_empty(), "{:?}", result.warnings);
    assert!(
        env.eval::<bool>("return ForeverLoaderSecurityRejected == true")
            .unwrap()
    );
}
