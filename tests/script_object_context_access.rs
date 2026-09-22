//! Caller-context access queries on the shared Forever script-object method surface.
#![cfg(feature = "client-wowforever")]

use rilua::{LuaApi, Val};
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn ordinary_and_forbidden_frames_use_caller_taint_without_exposing_secret_results() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        local ordinary = CreateFrame('Frame')
        local forbidden = CreateFrame('Frame')
        forbidden:SetForbidden()
        assert(ordinary:CanBeAccessedInContext() == true)
        local secureResult = forbidden:CanBeAccessedInContext()
        assert(issecretvalue(secureResult) and secureResult == true)
        local function addon()
            assert(not issecure())
            assert(ordinary:CanBeAccessedInContext() == true)
            local denied = forbidden:CanBeAccessedInContext()
            assert(issecretvalue(denied))
            assert(not pcall(function() if denied then error('denial leaked') end end))
            assert(not pcall(function() return denied == false end))
            return denied
        end
        debug.setobjecttaint(addon, 'ContextAccessProbe')
        local deniedResult = addon()
        assert(deniedResult == false)
        local proxy = GetForbiddenObjectTable(forbidden)
        assert(issecretvalue(proxy:CanBeAccessedInContext()))
        assert(proxy:CanBeAccessedInContext() == true)
        "#,
    )
    .unwrap();
}

#[test]
fn trusted_host_can_inspect_exact_secret_access_result_without_clearing_caller_taint() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        ContextSecureFrame = CreateFrame('Frame')
        ContextSecureFrame:SetForbidden()
        local function addon()
            return ContextSecureFrame:CanBeAccessedInContext()
        end
        debug.setobjecttaint(addon, 'ContextAccessProbe')
        ContextTaintedCaller = addon
        "#,
    )
    .unwrap();

    let allowed: Val = env
        .eval("return ContextSecureFrame:CanBeAccessedInContext()")
        .unwrap();
    {
        let lua = env.lua();
        let state = lua.state();
        assert!(rilua::table_security::is_secret_value(state, allowed));
        assert_eq!(
            rilua::table_security::unwrap_secret(state, allowed).unwrap(),
            Val::Bool(true)
        );
    }

    let denied: Val = env.eval("return ContextTaintedCaller()").unwrap();
    let lua = env.lua();
    let state = lua.state();
    assert!(rilua::table_security::is_secret_value(state, denied));
    assert_eq!(
        rilua::table_security::unwrap_secret(state, denied).unwrap(),
        Val::Bool(false)
    );
}

#[test]
fn restriction_only_denies_tainted_access_when_aura_context_is_active() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        ContextRestricted = CreateFrame('Frame')
        ContextRestricted:AddAccessRestrictions(
            Enum.ScriptObjectAccessRestriction.DenyTaintedAccessWhenAurasAreSecret)
        ContextUnrestricted = CreateFrame('Frame')
        assert(ContextRestricted:GetAccessRestrictions() == 1)
        assert(not ContextRestricted:IsForbidden())
        local function addon()
            local restricted = ContextRestricted:CanBeAccessedInContext()
            local unrestricted = ContextUnrestricted:CanBeAccessedInContext()
            assert(unrestricted == true)
            ContextRestrictedResult = restricted
        end
        debug.setobjecttaint(addon, 'ContextAccessProbe')
        ContextRestrictedCaller = addon
        "#,
    )
    .unwrap();
    env.exec("ContextRestrictedCaller(); assert(ContextRestrictedResult == true)")
        .unwrap();
    env.state().borrow_mut().auras_secret_in_context = true;
    env.exec(
        r#"
        assert(ContextRestricted:CanBeAccessedInContext() == true)
        ContextRestrictedCaller()
        assert(ContextRestrictedResult == false)
        assert(not ContextUnrestricted:HasAccessConstraints())
        "#,
    )
    .unwrap();
    env.state().borrow_mut().auras_secret_in_context = false;
    env.exec("ContextRestrictedCaller(); assert(ContextRestrictedResult == true)")
        .unwrap();
    assert!(
        !WowLuaEnv::new()
            .unwrap()
            .state()
            .borrow()
            .auras_secret_in_context
    );
}

#[test]
fn protected_and_explicit_object_security_tag_the_return_without_denial() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        ContextProtected = CreateFrame('Frame', 'ContextProtected')
        ContextExplicit = CreateFrame('Frame')
        ContextExplicit:AddSecretAspect(Enum.SecretAspect.ObjectSecurity)
        ContextProtected:SetPreventSecretValues(true)
        "#,
    )
    .unwrap();
    {
        let state = env.state();
        let mut state = state.borrow_mut();
        let id = state.widgets.get_id_by_name("ContextProtected").unwrap();
        state.widgets.get_mut(id).unwrap().is_protected = true;
    }
    env.exec(
        r#"
        local function checkSecure(frame)
            local result = frame:CanBeAccessedInContext()
            assert(issecretvalue(result) and result == true)
        end
        checkSecure(ContextProtected)
        checkSecure(ContextExplicit)
        local function addon()
            for _, frame in ipairs({ContextProtected, ContextExplicit}) do
                local result = frame:CanBeAccessedInContext()
                assert(issecretvalue(result))
                assert(not pcall(function() return not result end))
            end
        end
        debug.setobjecttaint(addon, 'ContextAccessProbe')
        addon()
        "#,
    )
    .unwrap();
}
