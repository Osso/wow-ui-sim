//! Taint boundary for explicitly secure, forbidden-partition XML delegates.
#![cfg(feature = "forbidden-aspects")]

use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn secure_xml_delegate_preserves_callback_and_caller_taint() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local frame = CreateFrame('Frame')
        local child = CreateFrame('Cooldown', nil, frame)
        local private = GetForbiddenObjectTable(frame)
        local privateChild = GetForbiddenObjectTable(child)
        local observedCallback = false
        local function delegate(self, argument, callback, trailing)
            assert(issecure(), 'native delegate must enter without caller taint')
            assert(self == private and argument == privateChild)
            assert(trailing == nil)
            local map = {}
            settablesecurity(map, Enum.TableSecurityOption.DisallowSecretKeys)
            securecallfunction(callback)
            assert(issecure(), 'callback taint must not leak into native delegate')
            return 'native-result', nil, 17
        end
        __wow_apply_xml_mixin(frame, {Run=delegate}, 'public', 'forbidden', true)
        __wow_apply_xml_mixin(frame, {Ordinary=delegate}, 'public', 'forbidden', false)
        local function callback()
            assert(not issecure(), 'addon callback must keep its closure taint')
            assert(debug.getstacktaint() == 'DelegateProbe')
            observedCallback = true
        end
        debug.setobjecttaint(callback, 'DelegateProbe')
        local function caller()
            assert(not issecure())
            local first, middle, last = frame:Run(child, callback, nil)
            assert(first == 'native-result' and middle == nil and last == 17)
            assert(observedCallback)
            assert(not issecure() and debug.getstacktaint() == 'DelegateProbe',
                'native delegate must restore caller taint')
            local allowed = pcall(settablesecurity, {}, Enum.TableSecurityOption.DisallowSecretKeys)
            assert(not allowed, 'direct tainted table security must still reject')
            local ordinary = pcall(frame.Ordinary, frame, child, callback, nil)
            assert(not ordinary, 'receiver-only delegate must not gain a secure boundary')
        end
        debug.setobjecttaint(caller, 'DelegateProbe')
        caller()
        assert(issecure())
        "#,
    )
    .expect("explicit secure XML delegates isolate caller taint without sanitizing callbacks");
}

#[test]
fn secure_xml_delegate_restores_caller_after_error() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local frame = CreateFrame('Frame')
        __wow_apply_xml_mixin(frame, {Fail=function()
            assert(issecure())
            error('delegate failure')
        end}, 'public', 'forbidden', true)
        local function caller()
            local ok, err = pcall(frame.Fail, frame)
            assert(not ok and tostring(err):find('delegate failure', 1, true))
            assert(not issecure() and debug.getstacktaint() == 'DelegateErrorProbe')
        end
        debug.setobjecttaint(caller, 'DelegateErrorProbe')
        caller()
        assert(issecure())
        "#,
    )
    .expect("secure delegate errors propagate without losing caller taint");
}
