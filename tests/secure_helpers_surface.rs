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
