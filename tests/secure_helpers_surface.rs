#![cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn selected_mixin_wraps_untrusted_delegate_arguments() {
    let env = WowLuaEnv::new().unwrap();
    let addons = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let source = std::fs::read_to_string(addons.join("Blizzard_SharedXMLBase/Mixin.lua")).unwrap();
    env.exec(&source).unwrap();
    env.exec(
        r#"
        local originalCallback = function() return issecure() end
        debug.setobjecttaint(originalCallback, 'UntrustedCallback')
        local methods = { run = function(callback, value)
            assert(issecure())
            assert(callback ~= originalCallback, 'untrusted callback must be wrapped')
            return callback(), value, nil, 'end'
        end }
        ApplySecureDelegatesToTable(methods)
        local function addon()
            local secure, value, empty, last = methods.run(originalCallback, 42)
            assert(secure == false and value == 42 and empty == nil and last == 'end')
            assert(not issecure(), 'caller taint must be restored')
        end
        debug.setobjecttaint(addon, 'AddonCaller')
        addon()
        "#,
    )
    .unwrap();
}

#[test]
fn securecopy_enforces_requested_traversal_depth() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local source = {nested = {value = 42}}
        assert(not pcall(securecopy, source, {maxTraversalDepth = 0}))
        local copy = securecopy(source, {maxTraversalDepth = 10})
        assert(copy ~= source and copy.nested ~= source.nested and copy.nested.value == 42)
        "#,
    )
    .unwrap();
}

#[test]
fn securecopy_preserves_shared_and_recursive_references() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local shared = {n = 1}
        local source = {a = shared, b = shared, [shared] = 'key'}
        source.self = source
        local copy = securecopy(source)
        assert(copy ~= source and copy.a ~= shared and copy.a.n == 1)
        assert(copy.a == copy.b, 'shared reference must stay shared')
        assert(copy.self == copy, 'recursive reference must point at the copy')
        assert(copy[copy.a] == 'key', 'table keys are copied with the graph')
        assert(securecopy(7) == 7 and securecopy('text') == 'text' and securecopy(nil) == nil)
        "#,
    )
    .unwrap();
}

#[test]
fn securecopy_slots_receive_current_execution_taint() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local source = {}
        local function addonWrite() source.name = 'addon'; source[1] = 'first' end
        debug.setobjecttaint(addonWrite, 'WriterAddon')
        addonWrite()
        assert(not issecurevariable(source, 'name'), 'fixture slot must be tainted')

        local secureCopy = securecopy(source)
        assert(secureCopy.name == 'addon' and secureCopy[1] == 'first')
        assert(issecurevariable(secureCopy, 'name') and issecurevariable(secureCopy, 1),
            'a secure caller receives secure copies')

        local function addonCopy() return securecopy({name = 'x', [1] = 'y'}) end
        debug.setobjecttaint(addonCopy, 'CopyAddon')
        local taintedCopy = addonCopy()
        local secureName, nameTaint = issecurevariable(taintedCopy, 'name')
        local secureIndex, indexTaint = issecurevariable(taintedCopy, 1)
        assert(not secureName and nameTaint == 'CopyAddon')
        assert(not secureIndex and indexTaint == 'CopyAddon')
        "#,
    )
    .unwrap();
}

#[test]
fn securecopy_wraps_untrusted_functions_with_original_taint() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local untrusted = function() return issecure(), debug.getstacktaint() end
        debug.setobjecttaint(untrusted, 'FunctionAddon')
        local trusted = function() return 'trusted' end
        local copy = securecopy({untrusted = untrusted, trusted = trusted})
        assert(copy.untrusted ~= untrusted and copy.trusted == trusted)
        local secure, taint = copy.untrusted()
        assert(secure == false and taint == 'FunctionAddon')
        local plain = securecopy({untrusted = untrusted}, {wrapUntrustedFunctions = false})
        assert(plain.untrusted == untrusted)
        "#,
    )
    .unwrap();
}

#[test]
fn securecopy_rejects_script_objects_unless_permitted() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local frame = CreateFrame('Frame')
        assert(not pcall(securecopy, {frame = frame}))
        local kept = securecopy({frame = frame}, {permitScriptObjects = true})
        assert(kept.frame == frame)
        "#,
    )
    .unwrap();
}

#[test]
fn secure_delegate_honours_wrap_option_and_restores_taint_on_error() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local untrusted = function() end
        debug.setobjecttaint(untrusted, 'CallbackAddon')
        local received
        local unwrapped = CreateSecureDelegate(function(callback) received = callback end,
            {wrapUntrustedFunctions = false})
        unwrapped(untrusted)
        assert(received == untrusted)

        local failing = CreateSecureDelegate(function() error('delegate failure') end)
        local function addon()
            local ok, message = pcall(failing)
            return ok, message, issecure()
        end
        debug.setobjecttaint(addon, 'ErrorAddon')
        local ok, message, secure = addon()
        assert(not ok and string.find(message, 'delegate failure', 1, true))
        assert(secure == false, 'caller taint must be restored after an error')
        "#,
    )
    .unwrap();
}
