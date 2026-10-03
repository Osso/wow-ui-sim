#![cfg(feature = "retail-12-0-5")]
//! Bounded Delves inputs; INFERRED host model and validation, not native parity.

use rilua::LuaApiMut;
use rilua::table_security::{wrap_host_secret_number, wrap_host_secret_string};
use wow_ui_sim::lua_api::WowLuaEnv;

const LINK: &str = "|Hspell:7001:3|h[Fixture Curio]|h";

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create Delves environment");
    {
        let mut state = env.state().borrow_mut();
        state.spell_id_aliases.clear();
        state.spell_id_aliases.insert("fixture curio".into(), 7001);
        state.curio_links.extend([
            ((7001, 3), LINK.into()),
            ((7001, 4), "epic fixture link".into()),
            ((7002, 3), "other fixture link".into()),
        ]);
    }
    env.exec(
        r#"
        function CheckActive(expected, ...)
            local function check(...)
                assert(select('#', ...) == 1)
                local value = ...
                assert(type(value) == 'boolean' and not issecretvalue(value))
                assert(value == expected)
            end
            check(C_DelvesUI.HasActiveDelve(...))
        end
        function CheckCurio(identifier, rarity, expected)
            local function check(...)
                assert(select('#', ...) == 1)
                local value = ...
                assert(type(value) == 'string' and not issecretvalue(value))
                assert(value == expected)
            end
            check(C_DelvesUI.GetCurioLink(identifier, rarity))
        end
        function RejectDelve(query, ...)
            assert(type(query) == 'function', 'registered API required')
            local ok, err = pcall(query, ...)
            assert(not ok and type(err) == 'string' and #err > 0)
            return err
        end
        "#,
    )
    .expect("install behavioral assertions");
    env
}

fn install_host_secrets(env: &WowLuaEnv) {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).expect("VM security helpers");
    for (name, number) in [("SecretMap", 2339.0), ("SecretRarity", 3.0), ("SecretSpell", 7001.0)] {
        let value = wrap_host_secret_number(lua.state_mut(), number);
        lua.state_mut().push(value);
        let inserted = lua.set_global_val(name, value);
        lua.state_mut().pop();
        inserted.expect("root authentic secret NUMBER");
    }
    for (name, text) in [("SecretMapString", "2339"), ("SecretRarityString", "3"), ("SecretSpellName", "fixture curio")] {
        let value = wrap_host_secret_string(lua.state_mut(), text);
        lua.state_mut().push(value);
        let inserted = lua.set_global_val(name, value);
        lua.state_mut().pop();
        inserted.expect("root authentic secret STRING");
    }
}

#[test]
fn active_delve_reads_live_host_boolean_and_ignores_all_extras() {
    let env = fixture_env();
    env.exec("CheckActive(false); CheckActive(false, 2339); CheckActive(false, {}, true)")
        .expect("default false regardless of former map argument");
    env.state().borrow_mut().has_active_delve = true;
    env.exec("CheckActive(true); CheckActive(true, nil); CheckActive(true, 0); CheckActive(true, 'bad', {})")
        .expect("explicit host activation, no argument conversion");
    env.state().borrow_mut().has_active_delve = false;
    env.exec("CheckActive(false, 2339)").expect("live reset");
}

#[test]
fn active_delve_does_not_access_secret_extras_for_either_caller() {
    let env = fixture_env();
    install_host_secrets(&env);
    env.state().borrow_mut().has_active_delve = true;
    env.exec(
        r#"
        assert(debug.getstacktaint() == nil)
        CheckActive(true, SecretMap, SecretMapString)
        local function probe()
            assert(debug.getstacktaint() == 'DelvesActiveProbe')
            CheckActive(true, SecretMap)
            CheckActive(true, SecretMapString, SecretRarity)
            assert(issecretvalue(SecretMap) and issecretvalue(SecretMapString))
            assert(not pcall(secretunwrap, SecretMap))
            assert(debug.getstacktaint() == 'DelvesActiveProbe')
        end
        debug.setobjecttaint(probe, 'DelvesActiveProbe')
        probe()
        assert(debug.getstacktaint() == nil)
        assert(issecretvalue(SecretMap) and secretunwrap(SecretMap) == 2339)
        "#,
    )
    .expect("undeclared secret arguments stay untouched");
}

#[test]
fn eligibility_request_records_map_and_returns_zero_values() {
    let env = fixture_env();
    assert_eq!(env.state().borrow().last_delve_eligibility_map_id, None);
    env.exec("assert(select('#', C_DelvesUI.RequestPartyEligibilityForDelveTiers(2339)) == 0)")
        .expect("zero request results");
    assert_eq!(env.state().borrow().last_delve_eligibility_map_id, Some(2339));
    env.exec("C_DelvesUI.RequestPartyEligibilityForDelveTiers(2340)")
        .expect("replace last request");
    assert_eq!(env.state().borrow().last_delve_eligibility_map_id, Some(2340));
    // INFERRED signed i32 storage domain, not native map-catalog validity.
    for map_id in [0, -1, i32::MIN, i32::MAX] {
        env.exec(&format!(
            "assert(select('#', C_DelvesUI.RequestPartyEligibilityForDelveTiers({map_id})) == 0)"
        ))
        .expect("finite integral storage boundaries accepted");
        assert_eq!(env.state().borrow().last_delve_eligibility_map_id, Some(map_id));
    }
}

#[test]
fn eligibility_invalid_inputs_preserve_last_request() {
    let env = fixture_env();
    env.state().borrow_mut().last_delve_eligibility_map_id = Some(2340);
    // INFERRED i32 storage range; finite integral numeric representation is required.
    for args in [
        "", "nil", "'2339'", "true", "{}", "1.5", "0/0", "math.huge",
        "-math.huge", "2147483648", "-2147483649",
    ] {
        let suffix = if args.is_empty() {
            String::new()
        } else {
            format!(", {args}")
        };
        env.exec(&format!(
            "RejectDelve(C_DelvesUI.RequestPartyEligibilityForDelveTiers{suffix})"
        ))
            .expect("invalid request errors");
        assert_eq!(env.state().borrow().last_delve_eligibility_map_id, Some(2340));
    }
}

#[test]
fn eligibility_authenticates_secret_map_before_validation_or_mutation() {
    let env = fixture_env();
    install_host_secrets(&env);
    env.exec("assert(select('#', C_DelvesUI.RequestPartyEligibilityForDelveTiers(SecretMap)) == 0)")
        .expect("untainted secret numeric map accepted");
    assert_eq!(env.state().borrow().last_delve_eligibility_map_id, Some(2339));
    env.exec(
        r#"
        RejectDelve(C_DelvesUI.RequestPartyEligibilityForDelveTiers, SecretMapString)
        local function probe()
            assert(debug.getstacktaint() == 'DelvesRequestProbe')
            local denial = RejectDelve(C_DelvesUI.RequestPartyEligibilityForDelveTiers, SecretMap)
            assert(RejectDelve(C_DelvesUI.RequestPartyEligibilityForDelveTiers, SecretMapString) == denial,
                'authenticate before numeric validation')
            assert(issecretvalue(SecretMap) and issecretvalue(SecretMapString))
            assert(not pcall(secretunwrap, SecretMap))
            assert(debug.getstacktaint() == 'DelvesRequestProbe')
        end
        debug.setobjecttaint(probe, 'DelvesRequestProbe')
        probe()
        assert(debug.getstacktaint() == nil)
        assert(issecretvalue(SecretMap) and secretunwrap(SecretMap) == 2339)
        "#,
    )
    .expect("denied and malformed secrets do not mutate request");
    assert_eq!(env.state().borrow().last_delve_eligibility_map_id, Some(2339));
    env.exec(
        r#"
        local function recover()
            assert(select('#', C_DelvesUI.RequestPartyEligibilityForDelveTiers(2340)) == 0)
            assert(debug.getstacktaint() == 'DelvesRequestRecovery')
        end
        debug.setobjecttaint(recover, 'DelvesRequestRecovery')
        recover()
        "#,
    )
    .expect("tainted public request remains usable");
    assert_eq!(env.state().borrow().last_delve_eligibility_map_id, Some(2340));
}

#[test]
fn curio_link_uses_aliases_rarity_and_live_host_records_without_mutation() {
    let env = fixture_env();
    let links = env.state().borrow().curio_links.clone();
    let aliases = env.state().borrow().spell_id_aliases.clone();
    env.exec(
        r#"
        CheckCurio(7001, 3, '|Hspell:7001:3|h[Fixture Curio]|h')
        CheckCurio('fixture curio', 3, '|Hspell:7001:3|h[Fixture Curio]|h')
        CheckCurio('FIXTURE CURIO', 4, 'epic fixture link')
        RejectDelve(C_DelvesUI.GetCurioLink, 9999, 3)
        RejectDelve(C_DelvesUI.GetCurioLink, 'unknown curio', 3)
        RejectDelve(C_DelvesUI.GetCurioLink, 7001, 2)
        "#,
    )
    .expect("exact seeded links; INFERRED misses error");
    assert_eq!(env.state().borrow().curio_links, links);
    assert_eq!(env.state().borrow().spell_id_aliases, aliases);
    env.state().borrow_mut().spell_id_aliases.insert("7001".into(), 7002);
    env.exec("CheckCurio(7001, 3, 'other fixture link'); CheckCurio('7001', 3, 'other fixture link')")
        .expect("alias-first numeric resolution");
    env.state().borrow_mut().curio_links.insert((7002, 3), "updated fixture link".into());
    env.exec("CheckCurio(7001, 3, 'updated fixture link')").expect("live link replacement");
    env.state().borrow_mut().curio_links.remove(&(7002, 3));
    env.exec("RejectDelve(C_DelvesUI.GetCurioLink, 7001, 3)").expect("no fabricated link after removal");
}

#[test]
fn curio_link_rejects_invalid_public_arguments() {
    let env = fixture_env();
    env.exec(
        r#"
        RejectDelve(C_DelvesUI.GetCurioLink)
        for _, bad in ipairs({true, {}, -1, 1.5, 0/0, math.huge, 4294967296}) do
            RejectDelve(C_DelvesUI.GetCurioLink, bad, 3)
        end
        RejectDelve(C_DelvesUI.GetCurioLink, nil, 3)
        RejectDelve(C_DelvesUI.GetCurioLink, 7001)
        RejectDelve(C_DelvesUI.GetCurioLink, 7001, nil)
        for _, bad in ipairs({'3', true, {}, -1, 1.5, 0/0, math.huge, 4294967296}) do
            RejectDelve(C_DelvesUI.GetCurioLink, 7001, bad)
        end
        CheckCurio(7001, 3, '|Hspell:7001:3|h[Fixture Curio]|h')
        "#,
    )
    .expect("INFERRED strict public identifier and u32 rarity representation");
}

#[test]
fn curio_rarity_is_never_secret_even_after_gc_and_for_tainted_callers() {
    let env = fixture_env();
    install_host_secrets(&env);
    let links = env.state().borrow().curio_links.clone();
    let aliases = env.state().borrow().spell_id_aliases.clone();
    env.exec(
        r#"
        local roots = {SecretRarity, SecretRarityString}
        collectgarbage('collect')
        local function probe()
            local before = debug.getstacktaint()
            for _, value in ipairs(roots) do
                assert(issecretvalue(value))
                local err = RejectDelve(C_DelvesUI.GetCurioLink, 7001, value)
                assert(string.find(err, 'argument 2 is NeverSecret', 1, true))
                assert(RejectDelve(C_DelvesUI.GetCurioLink, {}, value) == err,
                    'NeverSecret rejection before conversion')
                assert(issecretvalue(value))
            end
            -- INFERRED shared reader denies secret identifiers; native AllowedWhenTainted unmodeled.
            RejectDelve(C_DelvesUI.GetCurioLink, SecretSpell, 3)
            RejectDelve(C_DelvesUI.GetCurioLink, SecretSpellName, 3)
            CheckCurio(7001, 3, '|Hspell:7001:3|h[Fixture Curio]|h')
            assert(debug.getstacktaint() == before)
        end
        probe()
        debug.setobjecttaint(probe, 'DelvesCurioProbe')
        probe()
        assert(debug.getstacktaint() == nil)
        assert(rawequal(roots[1], SecretRarity) and rawequal(roots[2], SecretRarityString))
        assert(secretunwrap(SecretRarity) == 3 and secretunwrap(SecretRarityString) == '3')
        "#,
    )
    .expect("authentic NeverSecret boundary independent of caller taint");
    assert_eq!(env.state().borrow().curio_links, links);
    assert_eq!(env.state().borrow().spell_id_aliases, aliases);
}

#[test]
fn delve_inputs_are_isolated_between_environments() {
    let first = fixture_env();
    let second = fixture_env();
    {
        let mut state = first.state().borrow_mut();
        state.has_active_delve = true;
        state.curio_links.clear();
    }
    first.exec("C_DelvesUI.RequestPartyEligibilityForDelveTiers(2339)").expect("first request");
    second.exec("CheckActive(false); CheckCurio(7001, 3, '|Hspell:7001:3|h[Fixture Curio]|h')")
        .expect("second environment retains its own inputs");
    assert_eq!(second.state().borrow().last_delve_eligibility_map_id, None);
}
