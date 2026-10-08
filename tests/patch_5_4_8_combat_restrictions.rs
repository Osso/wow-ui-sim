//! Bounded retail combat protection: state and side effects, not native error parity.
#![cfg(feature = "client-retail")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn assert_cvar_combat_restrictions(env: &WowLuaEnv) {
    let register: serde_json::Value = serde_json::from_str(include_str!(
        "../data/patch-api/sources/5.4.8-wikitext-register.json"
    ))
    .unwrap();
    for row in register["entries"].as_array().unwrap() {
        if row["section"] != "cvars" {
            continue;
        }
        let name = row["symbol"].as_str().unwrap();
        // Retired/absent CVars have no current readable model to claim.
        if env.state().borrow().cvars.get(name).is_none() {
            continue;
        }
        env.state().borrow_mut().player.in_combat = false;
        env.exec(&format!("assert(SetCVar('{name}', '0'))"))
            .unwrap();
        env.state().borrow_mut().player.in_combat = true;
        let code = format!(
            r#"
            local events = 0
            local frame = CreateFrame('Frame')
            frame:RegisterEvent('CVAR_UPDATE')
            frame:RegisterEvent('UI_SCALE_CHANGED')
            frame:RegisterEvent('DISPLAY_SIZE_CHANGED')
            frame:SetScript('OnEvent', function() events = events + 1 end)
            local invoke = function(setter)
                assert(debug.getstacktaint() == 'Patch548Addon')
                setter('{name}', '1')
            end
            debug.setobjecttaint(invoke, 'Patch548Addon')
            local globalOK = pcall(invoke, SetCVar)
            local namespaceOK = pcall(invoke, C_CVar.SetCVar)
            frame:UnregisterAllEvents()
            return globalOK, namespaceOK, events
            "#
        );
        let (global_ok, namespace_ok, events): (bool, bool, i32) = env.eval(&code).unwrap();
        assert!(!global_ok && !namespace_ok, "{name}: insecure combat writes blocked");
        assert_eq!(events, 0, "{name}: blocked write emits no event");
        assert_eq!(env.state().borrow().cvars.get(name).as_deref(), Some("0"));
        env.exec(&format!("assert(issecure()); assert(C_CVar.SetCVar('{name}', '1'))"))
            .unwrap();
        assert_eq!(env.state().borrow().cvars.get(name).as_deref(), Some("1"));
        env.state().borrow_mut().player.in_combat = false;
        env.exec(&format!(
            "local f=function() SetCVar('{name}', '0') end; debug.setobjecttaint(f, 'Patch548Addon'); f()"
        ))
        .unwrap();
        assert_eq!(env.state().borrow().cvars.get(name).as_deref(), Some("0"));
    }
    env.state().borrow_mut().player.in_combat = false;
}

fn assert_visibility_combat_restrictions(env: &WowLuaEnv) {
    env.exec("SetUIVisibility(true)").unwrap();
    env.state().borrow_mut().player.in_combat = true;
    let (hidden, shown, visible): (bool, bool, bool) = env
        .eval(
            r#"
            local invoke = function(value)
                assert(debug.getstacktaint() == 'Patch548Addon')
                SetUIVisibility(value)
            end
            debug.setobjecttaint(invoke, 'Patch548Addon')
            local hideOK = pcall(invoke, false)
            local showOK = pcall(invoke, true)
            return hideOK, showOK, UIParent:IsShown()
            "#,
        )
        .unwrap();
    assert!(!hidden, "insecure combat hide must be blocked");
    assert!(shown && visible, "insecure combat show remains allowed");
    env.exec("assert(issecure()); SetUIVisibility(false); assert(not UIParent:IsShown())")
        .unwrap();
    env.state().borrow_mut().player.in_combat = false;
    env.exec(
        r#"
        SetUIVisibility(true)
        local hide = function() SetUIVisibility(false) end
        debug.setobjecttaint(hide, 'Patch548Addon')
        hide()
        assert(not UIParent:IsShown())
        SetUIVisibility(true)
        "#,
    )
    .unwrap();
}

#[test]
fn patch_5_4_8_bare_combat_restrictions() {
    let env = WowLuaEnv::new().expect("create environment");
    assert_cvar_combat_restrictions(&env);
    assert_visibility_combat_restrictions(&env);
}

prefork_full_ui_case! {
fn patch_5_4_8_cached_combat_restrictions(env: &WowLuaEnv) {
    assert_cvar_combat_restrictions(env);
    assert_visibility_combat_restrictions(env);
}
}
