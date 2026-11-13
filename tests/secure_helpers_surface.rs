#![cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn selected_mixin_wraps_every_function_argument_with_caller_source() {
    let env = WowLuaEnv::new().unwrap();
    let addons = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let source = std::fs::read_to_string(addons.join("Blizzard_SharedXMLBase/Mixin.lua")).unwrap();
    env.exec(&source).unwrap();
    env.exec(
        r#"
        local seen = {}
        local originalCallback = function()
            return debug.getstacktaint()
        end
        local methods = { run = function(callback)
            seen.wrapped = callback ~= originalCallback and not issecretvalue(callback)
            seen.callbackSource = callback()
        end }
        ApplySecureDelegatesToTable(methods)
        local function invoke(source)
            local addon = function()
                methods.run(originalCallback)
                seen.callerSource = debug.getstacktaint()
            end
            debug.setobjecttaint(addon, source)
            addon()
            assert(seen.wrapped, 'every function argument must be replaced by a fresh closure')
            assert(seen.callbackSource == source,
                'the wrapper must use the source present before it is created')
            assert(seen.callerSource == source, 'normal return must restore caller source')
        end
        invoke('FirstAddon')
        invoke('SecondAddon')
        "#,
    )
    .unwrap();
}

#[test]
fn secure_delegate_tainted_construction_returns_zero_values() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local function addon()
            return select('#', CreateSecureDelegate(function() end))
        end
        debug.setobjecttaint(addon, 'DelegateConstructorProbe')
        assert(addon() == 0, 'tainted construction must return zero values')
        "#,
    )
    .unwrap();
}

#[test]
fn secure_delegate_untainted_construction_returns_one_function() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local count = select('#', CreateSecureDelegate(function() end))
        local delegate = CreateSecureDelegate(function() end)
        assert(count == 1 and type(delegate) == 'function')
        "#,
    )
    .unwrap();
}

#[test]
fn secure_delegate_rejects_nil_target() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local ok = pcall(CreateSecureDelegate, nil)
        assert(not ok, 'the required target argument cannot be nil')
        "#,
    )
    .unwrap();
}

// INFERRED: S4s7b's emulation accepted numeric targets; invocation remains unknown.
#[test]
fn secure_delegate_constructs_for_numeric_target_inferred() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local count = select('#', CreateSecureDelegate(42))
        local delegate = CreateSecureDelegate(42)
        assert(count == 1 and type(delegate) == 'function')
        "#,
    )
    .unwrap();
}

// INFERRED: S4s7b emulation let a tainted target supply the captured source.
#[test]
fn secure_delegate_captures_tainted_target_source_inferred() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local observed = {}
        local function target(value)
            observed.value = value
            observed.source = debug.getstacktaint()
        end
        debug.setobjecttaint(target, 'TargetSource')
        local delegate = CreateSecureDelegate(target)
        local function addon() delegate(42) end
        debug.setobjecttaint(addon, 'CallerSource')
        addon()
        local secure, source = issecurevariable(observed, 'value')
        assert(observed.value == 42 and observed.source == 'TargetSource')
        assert(not secure and source == 'TargetSource')
        "#,
    )
    .unwrap();
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn secure_delegate_rejects_inaccessible_secret_target() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local target = secretwrap(function() end)
        dropsecretaccess()
        local ok, message = pcall(CreateSecureDelegate, target)
        assert(not ok and tostring(message):find('secret access', 1, true), tostring(message))
        "#,
    )
    .unwrap();
}

#[cfg(feature = "retail-12-1-0")]
#[test]
fn secure_delegate_preserves_secret_argument_markers() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local secret = secretwrap(17)
        local observed
        local delegate = CreateSecureDelegate(function(value)
            observed = issecretvalue(value)
        end)
        delegate(secret)
        assert(observed, 'changing argument provenance must not clear secrecy')
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
