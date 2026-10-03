#![cfg(feature = "retail-12-0-5")]

// INFERRED eligibility/miss/default and strict secret policies, not native parity.

use rilua::LuaApiMut;
use rilua::table_security::{wrap_host_secret_number, wrap_host_secret_string};
use wow_ui_sim::lua_api::WowLuaEnv;

const LINK: &str = "|cff71d5ff|Hspell:7001|h[Fixture Click]|h|r";

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("click-binding environment");
    env.exec(
        r#"
        function CheckClickBinding(identifier, expected)
            local function check(...)
                assert(select('#', ...) == 1, 'exactly one result')
                local value = ...
                assert(type(value) == 'boolean', 'required boolean')
                assert(not issecretvalue(value), 'public boolean')
                assert(value == expected, 'host-declared eligibility')
            end
            check(C_ClickBindings.CanSpellBeClickBound(identifier))
        end
        function RejectClickBinding(...)
            local ok, err = pcall(C_ClickBindings.CanSpellBeClickBound, ...)
            assert(not ok and type(err) == 'string', 'identifier rejected')
            assert(string.find(err, 'C_ClickBindings.CanSpellBeClickBound:', 1, true), 'API error context')
        end
        "#,
    )
    .expect("assertion helpers without replacing API");
    env
}

fn seeded_env() -> WowLuaEnv {
    let env = fixture_env();
    {
        let mut state = env.state().borrow_mut();
        state.click_bindable_spells.clear();
        state.click_bindable_spells.insert(7001);
        state.spell_id_aliases.clear();
        state.spell_id_aliases.extend([
            ("fixture click".into(), 7001),
            ("blocked click".into(), 7002),
            (LINK.to_lowercase(), 7002),
        ]);
    }
    env
}

#[test]
fn empty_eligibility_preserves_default_interaction_profile() {
    let env = fixture_env();
    assert!(env.state().borrow().click_bindable_spells.is_empty());
    env.exec(
        r#"
        CheckClickBinding(7001, false)
        CheckClickBinding('unknown', false)
        local profile = C_ClickBindings.GetProfileInfo()
        assert(#profile == 2, 'interaction defaults remain present')
        assert(profile[1].type == Enum.ClickBindingType.Interaction)
        assert(profile[2].type == Enum.ClickBindingType.Interaction)
        assert(C_ClickBindings.GetBindingType('LeftButton', 0) == Enum.ClickBindingType.Interaction)
        assert(C_ClickBindings.GetBindingType('RightButton', 0) == Enum.ClickBindingType.Interaction)
        "#,
    )
    .unwrap();
}

#[test]
fn explicit_membership_resolves_names_and_miss_controls() {
    seeded_env()
        .exec(
            r#"
            CheckClickBinding(7001, true)
            CheckClickBinding('FIXTURE Click', true)
            CheckClickBinding(7002, false)
            CheckClickBinding('blocked click', false)
            CheckClickBinding(9999, false)
            CheckClickBinding('unknown', false)
            CheckClickBinding('', false)
            CheckClickBinding('7001', false)
            "#,
        )
        .unwrap();
}

#[test]
fn full_link_alias_is_explicit_not_parsed() {
    let env = seeded_env();
    env.exec(&format!("CheckClickBinding('{LINK}', false)"))
        .unwrap();
    env.state().borrow_mut().click_bindable_spells.insert(7002);
    env.exec(&format!("CheckClickBinding('{LINK}', true)"))
        .unwrap();
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .remove(&LINK.to_lowercase());
    env.exec(&format!(
        "CheckClickBinding('{LINK}', false); CheckClickBinding(7001, true)"
    ))
    .unwrap();
}

#[test]
fn numeric_alias_precedence_and_removal_are_live() {
    let env = seeded_env();
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .insert("7001".into(), 7002);
    env.exec("CheckClickBinding(7001, false); CheckClickBinding('7001', false)")
        .unwrap();
    env.state().borrow_mut().click_bindable_spells.insert(7002);
    env.exec("CheckClickBinding(7001, true); CheckClickBinding('7001', true)")
        .unwrap();
    env.state().borrow_mut().spell_id_aliases.remove("7001");
    env.exec("CheckClickBinding(7001, true); CheckClickBinding('7001', false)")
        .unwrap();
}

#[test]
fn membership_and_named_alias_mutations_are_live_and_environment_local() {
    let env = seeded_env();
    let other = seeded_env();
    env.state().borrow_mut().click_bindable_spells.remove(&7001);
    env.exec("CheckClickBinding('fixture click', false)").unwrap();
    other.exec("CheckClickBinding('fixture click', true)").unwrap();
    env.state().borrow_mut().click_bindable_spells.insert(7002);
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .insert("fixture click".into(), 7002);
    let spells = env.state().borrow().click_bindable_spells.clone();
    let aliases = env.state().borrow().spell_id_aliases.clone();
    env.exec("CheckClickBinding('fixture click', true); CheckClickBinding(7001, false)")
        .unwrap();
    assert_eq!(env.state().borrow().click_bindable_spells, spells);
    assert_eq!(env.state().borrow().spell_id_aliases, aliases);
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .remove("fixture click");
    env.exec("CheckClickBinding('fixture click', false); CheckClickBinding(7002, true)")
        .unwrap();
    other.exec("CheckClickBinding('fixture click', true); CheckClickBinding(7002, false)")
        .unwrap();
}

#[test]
fn existing_spell_profile_does_not_declare_eligibility() {
    let env = seeded_env();
    env.exec(
        r#"
        C_ClickBindings.SetProfileByInfo({
            {type = Enum.ClickBindingType.Spell, actionID = 7002, button = 'Button4', modifiers = 0},
        })
        assert(C_ClickBindings.GetBindingType('Button4', 0) == Enum.ClickBindingType.Spell)
        CheckClickBinding(7002, false)
        CheckClickBinding(7001, true)
        C_ClickBindings.ResetCurrentProfile()
        CheckClickBinding(7001, true)
        CheckClickBinding(7002, false)
        "#,
    )
    .unwrap();
}

#[test]
fn strict_public_identifiers_validate_before_alias_resolution() {
    let env = seeded_env();
    env.state()
        .borrow_mut()
        .click_bindable_spells
        .extend([0, u32::MAX]);
    env.state().borrow_mut().spell_id_aliases.extend([
        ("-1".into(), 7001),
        ("4294967296".into(), 7001),
    ]);
    env.exec(
        r#"
        CheckClickBinding(0, true)
        CheckClickBinding(4294967295, true)
        RejectClickBinding()
        RejectClickBinding(nil)
        RejectClickBinding(true)
        RejectClickBinding({})
        RejectClickBinding(CreateFrame('Frame'))
        RejectClickBinding(function() end)
        RejectClickBinding(coroutine.create(function() end))
        RejectClickBinding(-1)
        RejectClickBinding(7001.5)
        RejectClickBinding(4294967296)
        RejectClickBinding(0/0)
        RejectClickBinding(math.huge)
        RejectClickBinding(-math.huge)
        RejectClickBinding(string.char(255))
        CheckClickBinding(7001, true)
        "#,
    )
    .unwrap();
}

#[test]
fn public_tainted_calls_preserve_taint_and_boolean_contract() {
    let env = seeded_env();
    env.exec(&format!(
        r#"
        local function probe()
            local before = debug.getstacktaint()
            assert(before == 'ClickBindingFixtureAddon', 'actually tainted caller')
            CheckClickBinding(7001, true)
            CheckClickBinding('FIXTURE CLICK', true)
            CheckClickBinding('{LINK}', false)
            CheckClickBinding('unknown', false)
            assert(debug.getstacktaint() == before, 'caller taint preserved')
        end
        debug.setobjecttaint(probe, 'ClickBindingFixtureAddon')
        probe()
        "#,
    ))
    .unwrap();
}

fn install_host_secrets(env: &WowLuaEnv) {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).expect("VM security helpers");
    for (name, text) in [
        ("ClickSecretName", "fixture click"),
        ("ClickSecretLink", LINK),
        ("ClickSecretUnknown", "unknown"),
    ] {
        let value = wrap_host_secret_string(lua.state_mut(), text);
        lua.state_mut().push(value);
        let inserted = lua.set_global_val(name, value);
        lua.state_mut().pop();
        inserted.expect("root authentic secret string");
    }
    for (name, number) in [("ClickSecretNumber", 7001.0), ("ClickSecretMissing", 9999.0)] {
        let value = wrap_host_secret_number(lua.state_mut(), number);
        lua.state_mut().push(value);
        let inserted = lua.set_global_val(name, value);
        lua.state_mut().pop();
        inserted.expect("root authentic secret number");
    }
}

#[test]
fn authentic_secret_identifiers_reject_after_gc_in_secure_and_tainted_calls() {
    let env = seeded_env();
    install_host_secrets(&env);
    let spells = env.state().borrow().click_bindable_spells.clone();
    let aliases = env.state().borrow().spell_id_aliases.clone();
    env.exec(
        r#"
        ClickSecretRoots = {ClickSecretName, ClickSecretLink, ClickSecretUnknown,
            ClickSecretNumber, ClickSecretMissing}
        local function probe(expectedTaint)
            local before = debug.getstacktaint()
            assert(before == expectedTaint, 'actual caller context')
            collectgarbage('collect')
            local globals = {ClickSecretName, ClickSecretLink, ClickSecretUnknown,
                ClickSecretNumber, ClickSecretMissing}
            for index, value in ipairs(ClickSecretRoots) do
                assert(rawequal(value, globals[index]), 'rooted identity across GC')
                assert(issecretvalue(value), 'real VM secret')
                local ok, err = pcall(C_ClickBindings.CanSpellBeClickBound, value)
                assert(not ok and type(err) == 'string', 'secret rejection')
                assert(string.find(err, 'secret spell identifier access is not modeled', 1, true), 'secret-specific rejection')
                assert(rawequal(value, globals[index]) and issecretvalue(value), 'identity and secrecy retained')
                assert(debug.getstacktaint() == before, 'rejection preserves taint')
                CheckClickBinding(7001, true)
                CheckClickBinding('fixture click', true)
                CheckClickBinding('unknown', false)
                assert(debug.getstacktaint() == before, 'public recovery preserves taint')
            end
        end
        probe(nil)
        debug.setobjecttaint(probe, 'ClickBindingFixtureAddon')
        probe('ClickBindingFixtureAddon')
        "#,
    )
    .unwrap();
    assert_eq!(env.state().borrow().click_bindable_spells, spells);
    assert_eq!(env.state().borrow().spell_id_aliases, aliases);
}
