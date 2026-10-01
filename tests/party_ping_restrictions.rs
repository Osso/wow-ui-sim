//! Pending Retail 12.0.5 contract fixtures; strict rejection is simulator policy.
#![cfg(feature = "retail-12-0-5")]

use rilua::table_security::wrap_host_secret_number;
use wow_ui_sim::lua_api::WowLuaEnv;

fn ping_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create party ping environment");
    env.exec(
        r#"
        assert(type(C_PartyInfo.GetRestrictPings) == 'function', 'getter must exist')
        assert(type(C_PartyInfo.SetRestrictPings) == 'function', 'setter must exist')
        "#,
    )
    .expect("public party ping methods must be registered");
    env
}

fn assert_restriction(env: &WowLuaEnv, expected: u8) {
    env.exec(&format!(
        r#"
        assert(select('#', C_PartyInfo.GetRestrictPings()) == 1)
        local value = C_PartyInfo.GetRestrictPings()
        assert(type(value) == 'number', 'required enum must be numeric')
        assert(value == {expected}, 'getter must read this environment input')
        "#,
    ))
    .expect("exactly one required numeric restriction");
}

#[test]
fn fresh_environment_returns_one_numeric_none() {
    let env = ping_env();
    assert_eq!(env.state().borrow().party_ping_restriction, 0);
    assert_restriction(&env, 0);
}

#[test]
fn getter_reads_explicit_per_environment_input() {
    let env = ping_env();
    for restriction in [3, 1, 2, 0] {
        env.state().borrow_mut().party_ping_restriction = restriction;
        assert_restriction(&env, restriction);
    }
}

#[test]
fn setter_accepts_all_four_enums_returns_nothing_and_resets_to_none() {
    let env = ping_env();
    env.exec(
        r#"
        local restrictions = Enum.RestrictPingsTo
        assert(restrictions.None == 0 and restrictions.Lead == 1)
        assert(restrictions.Assist == 2 and restrictions.TankHealer == 3)
        for _, restriction in ipairs({restrictions.None, restrictions.Lead,
                restrictions.Assist, restrictions.TankHealer}) do
            assert(select('#', C_PartyInfo.SetRestrictPings(restriction)) == 0)
            assert(C_PartyInfo.GetRestrictPings() == restriction)
            assert(select('#', C_PartyInfo.SetRestrictPings(restriction)) == 0)
            assert(C_PartyInfo.GetRestrictPings() == restriction,
                'same selection reset belongs to the caller, not the setter')
            assert(select('#', C_PartyInfo.SetRestrictPings(restrictions.None)) == 0)
            assert(C_PartyInfo.GetRestrictPings() == restrictions.None)
        end
        "#,
    )
    .expect("enum setter and caller-controlled reset");
}

#[test]
fn public_mutations_are_isolated_between_environments() {
    let first = ping_env();
    let second = ping_env();
    first.exec("C_PartyInfo.SetRestrictPings(1)").unwrap();
    assert_restriction(&first, 1);
    assert_restriction(&second, 0);
    second.exec("C_PartyInfo.SetRestrictPings(3)").unwrap();
    first.exec("C_PartyInfo.SetRestrictPings(2)").unwrap();
    assert_restriction(&first, 2);
    assert_restriction(&second, 3);
}

#[test]
fn lockdown_blocks_mutation_but_not_reads_independently_of_combat() {
    let env = ping_env();
    for (combat, lockdown) in [(false, false), (false, true), (true, false), (true, true)] {
        {
            let mut state = env.state().borrow_mut();
            state.party_ping_restriction = 2;
            state.player.in_combat = combat;
            state.chat_messaging_lockdown = lockdown;
        }
        assert_restriction(&env, 2);
        env.exec(&format!(
            r#"
            local ok, err = pcall(C_PartyInfo.SetRestrictPings, 1)
            assert(ok == {allowed}, 'only chat messaging lockdown blocks setter')
            if not ok then
                assert(type(err) == 'string' and #err > 0, 'rejection must be explicit')
            end
            "#,
            allowed = !lockdown,
        ))
        .expect("setter restriction axis");
        let expected = if lockdown { 2 } else { 1 };
        assert_restriction(&env, expected);
        let state = env.state();
        let state = state.borrow();
        assert_eq!(state.party_ping_restriction, expected);
        assert_eq!(state.player.in_combat, combat);
        assert_eq!(state.chat_messaging_lockdown, lockdown);
    }
}

#[test]
fn malformed_enum_arguments_reject_atomically() {
    let env = ping_env();
    env.exec(
        r#"
        C_PartyInfo.SetRestrictPings(2)
        local function reject(...)
            local ok, err = pcall(C_PartyInfo.SetRestrictPings, ...)
            assert(not ok, 'malformed enum must reject')
            assert(type(err) == 'string' and #err > 0)
            assert(C_PartyInfo.GetRestrictPings() == 2, 'rejection must be atomic')
        end
        reject()
        reject(nil)
        for _, value in ipairs({0.5, -1, 4, '1', true, false, {}, function() end}) do
            reject(value)
        end
        "#,
    )
    .expect("strict simulator enum validation without coercion");
    assert_eq!(env.state().borrow().party_ping_restriction, 2);
}

#[test]
fn secret_enum_rejection_preserves_input_and_caller_taint() {
    let env = ping_env();
    {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        let secret = wrap_host_secret_number(lua.state_mut(), 1.0);
        lua.state_mut().push(secret);
        let inserted = lua.set_global_val("SecretPingRestriction", secret);
        lua.state_mut().pop();
        inserted.expect("install actual secret enum fixture");
    }
    env.exec(
        r#"
        C_PartyInfo.SetRestrictPings(3)
        collectgarbage('collect')
        local function reject()
            local ok, err = pcall(C_PartyInfo.SetRestrictPings, SecretPingRestriction)
            assert(not ok, 'conservative simulator policy rejects secret enums')
            assert(type(err) == 'string' and #err > 0)
            assert(issecretvalue(SecretPingRestriction))
            assert(C_PartyInfo.GetRestrictPings() == 3)
        end
        assert(issecure())
        reject()
        assert(issecure())
        local function addon()
            assert(debug.getstacktaint() == 'PartyPingFixture')
            reject()
            assert(debug.getstacktaint() == 'PartyPingFixture',
                'rejection must not clear or replace caller taint')
        end
        debug.setobjecttaint(addon, 'PartyPingFixture')
        addon()
        assert(issecure())
        "#,
    )
    .expect("secret rejection is atomic in secure and tainted callers");
    assert_eq!(env.state().borrow().party_ping_restriction, 3);
}
