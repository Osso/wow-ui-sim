#![cfg(feature = "retail-12-0-5")]
//! Row 181: explicit active-Delve host input feeds the registered instance query.

use rilua::LuaApiMut;
use rilua::table_security::{wrap_host_secret_number, wrap_host_secret_string};
use wow_ui_sim::lua_api::WowLuaEnv;

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create Delve instance environment");
    env.exec(
        r#"
        function CheckInstance(expectedFlag, expectedKind, ...)
            local function check(...)
                assert(select('#', ...) == 2, 'exact instance return arity')
                local flag, kind = ...
                assert(type(flag) == 'boolean' and not issecretvalue(flag))
                assert(type(kind) == 'string' and not issecretvalue(kind))
                assert(flag == expectedFlag, 'instance flag mismatch')
                assert(kind == expectedKind, 'host instance type mismatch')
            end
            check(IsInInstance(...))
        end
        "#,
    )
    .expect("install observable result assertions");
    env
}

#[test]
fn active_delve_entry_and_exit_drive_instance_flag_without_legacy_flag_changes() {
    let env = fixture_env();
    env.exec("CheckInstance(false, 'none'); assert(not C_DelvesUI.HasActiveDelve())")
        .expect("open world");
    {
        let mut state = env.state().borrow_mut();
        assert!(!state.world.in_instance);
        state.world.instance_type = "party".into();
        state.has_active_delve = true;
    }
    env.exec("assert(C_DelvesUI.HasActiveDelve()); CheckInstance(true, 'party')")
        .expect("Delve host entry is an instance even with false legacy flag");
    assert!(!env.state().borrow().world.in_instance);
    {
        let mut state = env.state().borrow_mut();
        state.has_active_delve = false;
        state.world.instance_type = "none".into();
    }
    env.exec("assert(not C_DelvesUI.HasActiveDelve()); CheckInstance(false, 'none')")
        .expect("host Delve exit restores open world");
}

#[test]
fn ending_delve_preserves_independently_active_dungeon() {
    let env = fixture_env();
    {
        let mut state = env.state().borrow_mut();
        state.world.in_instance = true;
        state.world.instance_type = "party".into();
        state.has_active_delve = true;
    }
    env.exec("CheckInstance(true, 'party')")
        .expect("both host flags active");
    env.state().borrow_mut().has_active_delve = false;
    env.exec("CheckInstance(true, 'party')")
        .expect("INFERRED composition retains independent dungeon");
    env.state().borrow_mut().world.in_instance = false;
    env.exec("CheckInstance(false, 'party')")
        .expect("existing type read-through unchanged when no instance flag is active");
}

#[test]
fn delve_query_preserves_live_host_type_and_does_not_repair_host_state() {
    let env = fixture_env();
    env.state().borrow_mut().has_active_delve = true;
    env.exec("CheckInstance(true, 'none')")
        .expect("no invented Delve instance type for inconsistent host input");
    env.state().borrow_mut().world.instance_type = "scenario".into();
    env.exec("CheckInstance(true, 'scenario'); CheckInstance(true, 'scenario')")
        .expect("live host type, not a wrapped constant");
    let state = env.state().borrow();
    assert!(state.has_active_delve);
    assert!(!state.world.in_instance);
    assert_eq!(state.world.instance_type, "scenario");
}

#[test]
fn delve_query_ignores_extra_arguments_and_preserves_secret_wrappers_and_taint() {
    let env = fixture_env();
    env.state().borrow_mut().has_active_delve = true;
    {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        rilua::table_security::register_table_security(&mut lua).expect("VM security helpers");
        let number = wrap_host_secret_number(lua.state_mut(), 2339.0);
        lua.state_mut().push(number);
        let inserted = lua.set_global_val("SecretDelveMap", number);
        lua.state_mut().pop();
        inserted.expect("root authentic secret number");
        let text = wrap_host_secret_string(lua.state_mut(), "delve fixture");
        lua.state_mut().push(text);
        let inserted = lua.set_global_val("SecretDelveText", text);
        lua.state_mut().pop();
        inserted.expect("root authentic secret string");
    }
    env.exec(
        r#"
        assert(debug.getstacktaint() == nil)
        CheckInstance(true, 'none', nil, {}, true, 'bad')
        CheckInstance(true, 'none', SecretDelveMap, SecretDelveText)
        collectgarbage('collect')
        local function probe()
            assert(debug.getstacktaint() == 'DelveInstanceProbe')
            CheckInstance(true, 'none', {}, false, 'bad')
            CheckInstance(true, 'none', SecretDelveMap, SecretDelveText)
            assert(issecretvalue(SecretDelveMap) and issecretvalue(SecretDelveText))
            assert(not pcall(secretunwrap, SecretDelveMap))
            assert(debug.getstacktaint() == 'DelveInstanceProbe')
        end
        debug.setobjecttaint(probe, 'DelveInstanceProbe')
        probe()
        assert(debug.getstacktaint() == nil)
        assert(issecretvalue(SecretDelveMap) and secretunwrap(SecretDelveMap) == 2339)
        assert(issecretvalue(SecretDelveText) and secretunwrap(SecretDelveText) == 'delve fixture')
        "#,
    )
    .expect("no declared arguments to authenticate; retain caller and wrapper security");
    assert!(!env.state().borrow().world.in_instance);
    assert!(env.state().borrow().has_active_delve);
}

#[test]
fn active_delve_instance_state_is_environment_local() {
    let first = fixture_env();
    let second = fixture_env();
    first.state().borrow_mut().has_active_delve = true;
    first
        .exec("CheckInstance(true, 'none')")
        .expect("first environment active");
    second
        .exec("CheckInstance(false, 'none')")
        .expect("second environment open world");
    second.state().borrow_mut().has_active_delve = true;
    first.state().borrow_mut().has_active_delve = false;
    first
        .exec("CheckInstance(false, 'none')")
        .expect("first environment reset");
    second
        .exec("CheckInstance(true, 'none')")
        .expect("second environment remains active");
}
