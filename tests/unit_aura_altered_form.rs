//! Bounded row407 contract; defaults, identity misses and strict input errors
//! are INFERRED simulator policies, not native-client observations.
#![cfg(feature = "retail-12-0-5")]

use rilua::LuaApiMut;
use rilua::table_security::wrap_host_secret_string;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{PlayerState, ShapeshiftForm};

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create altered-form environment");
    env.exec(
        r#"
        function AssertAlteredForm(unit, expected)
            local function check(...)
                assert(select('#', ...) == 1, 'exactly one result')
                local value = ...
                assert(type(value) == 'boolean', 'boolean result')
                assert(not issecretvalue(value), 'public result')
                assert(value == expected, 'configured altered-form value')
            end
            check(C_UnitAuras.WantsAlteredForm(unit))
        end
        "#,
    )
    .expect("install assertions without replacing query or vendor code");
    env
}

fn install_host_secret_unit(env: &WowLuaEnv) {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).expect("install VM security helpers");
    let secret = wrap_host_secret_string(lua.state_mut(), "player");
    lua.state_mut().push(secret);
    let inserted = lua.set_global_val("SecretAlteredFormUnit", secret);
    lua.state_mut().pop();
    inserted.expect("root actual host-secret unit STRING");
}

#[test]
fn player_input_defaults_false_in_empty_and_seeded_state() {
    assert!(!PlayerState::default().wants_altered_form);
    assert!(!PlayerState::seeded().wants_altered_form);
    let env = fixture_env();
    assert!(!env.state().borrow().player.wants_altered_form);
}

#[test]
fn player_query_returns_one_public_boolean_for_false_true_false() {
    let env = fixture_env();
    for expected in [false, true, false] {
        env.state().borrow_mut().player.wants_altered_form = expected;
        env.exec(&format!("AssertAlteredForm('player', {expected})"))
            .expect("query reflects explicit state with exact public boolean arity");
        assert_eq!(env.state().borrow().player.wants_altered_form, expected);
    }
}

#[test]
fn target_and_focus_follow_existing_player_guid_identity() {
    let env = fixture_env();
    env.exec(
        "TargetUnit('player'); FocusUnit('target'); \
         assert(UnitGUID('target') == UnitGUID('player')); \
         assert(UnitGUID('focus') == UnitGUID('player'))",
    )
    .expect("existing target/focus operations resolve player GUID");
    for expected in [false, true, false] {
        env.state().borrow_mut().player.wants_altered_form = expected;
        env.exec(&format!(
            "AssertAlteredForm('player', {expected}); \
             AssertAlteredForm('target', {expected}); AssertAlteredForm('focus', {expected})"
        ))
        .expect("aliases read player state by identity, not token spelling");
    }
    env.state().borrow_mut().player.wants_altered_form = true;
    env.exec(
        r#"
        A_Admin.SetTarget('Hogger', 11, 1, true)
        assert(UnitGUID('target') ~= UnitGUID('player'))
        assert(UnitGUID('focus') == UnitGUID('player'))
        AssertAlteredForm('target', false)
        AssertAlteredForm('focus', true)
        A_Admin.ClearTarget()
        A_Admin.ClearFocus()
        AssertAlteredForm('target', false)
        AssertAlteredForm('focus', false)
        AssertAlteredForm('player', true)
        "#,
    )
    .expect("retargeting and clearing change identity without changing player input");
    assert!(env.state().borrow().player.wants_altered_form);
}

#[test]
fn other_unknown_and_unmodeled_identities_return_false() {
    let env = fixture_env();
    {
        let mut state = env.state().borrow_mut();
        state.player.wants_altered_form = true;
        state.party_group_active = true;
        assert!(!state.party_members.is_empty(), "existing seeded roster");
    }
    env.exec(
        r#"
        assert(UnitExists('party1') and UnitGUID('party1') ~= UnitGUID('player'))
        A_Admin.SetTarget('Hogger', 11, 1, true)
        FocusUnit('target')
        assert(UnitGUID('focus') == UnitGUID('target'))
        for _, unit in ipairs({'party1', 'raid1', 'target', 'focus', 'pet',
            'vehicle', 'self', 'raidplayer', 'unknown-unit', 'party99', ''}) do
            AssertAlteredForm(unit, false)
        end
        AssertAlteredForm('player', true)
        "#,
    )
    .expect("INFERRED false for identities lacking modeled altered-form input");
}

#[test]
fn explicit_input_is_independent_of_race_stance_and_legacy_form_flags() {
    let env = fixture_env();
    for race in [0, 10, 13] {
        for active in [false, true] {
            for expected in [false, true] {
                {
                    let mut state = env.state().borrow_mut();
                    state.player.race_index = race;
                    state.player.wants_altered_form = expected;
                    state.player.is_alternate_form = active;
                    state.player.alternate_form_is_default = active;
                    state.shapeshift_forms = vec![ShapeshiftForm {
                        name: "Bear Form".into(),
                        texture: "Interface/Icons/Ability_Racial_BearForm".into(),
                        spell_id: 5487,
                        is_active: active,
                        is_castable: true,
                    }];
                }
                env.exec(&format!(
                    "assert(GetShapeshiftForm() == {}); AssertAlteredForm('player', {expected})",
                    if active { 1 } else { 0 }
                ))
                .expect("race/stance/legacy flags must not derive or replace explicit input");
                let state = env.state().borrow();
                assert_eq!(state.player.race_index, race);
                assert_eq!(state.shapeshift_forms[0].is_active, active);
                assert_eq!(state.player.is_alternate_form, active);
                assert_eq!(state.player.alternate_form_is_default, active);
                assert_eq!(state.player.wants_altered_form, expected);
            }
        }
    }
}

#[test]
fn required_unit_string_rejects_missing_nil_and_wrong_types() {
    let env = fixture_env();
    env.state().borrow_mut().player.wants_altered_form = true;
    env.exec(
        r#"
        local query = C_UnitAuras.WantsAlteredForm
        assert(type(query) == 'function', 'real query must be registered')
        local function reject(...)
            local ok, err = pcall(query, ...)
            assert(not ok and type(err) == 'string' and #err > 0)
        end
        reject()
        reject(nil)
        for _, unit in ipairs({false, true, 0, 1, {}, function() end}) do
            reject(unit)
        end
        AssertAlteredForm('player', true)
        "#,
    )
    .expect("INFERRED strict required string, including no numeric coercion");
    assert!(env.state().borrow().player.wants_altered_form);
}

#[test]
fn untainted_caller_accepts_actual_host_secret_unit_string() {
    let env = fixture_env();
    install_host_secret_unit(&env);
    for expected in [false, true, false] {
        env.state().borrow_mut().player.wants_altered_form = expected;
        env.exec(&format!(
            "local before = debug.getstacktaint(); assert(before == nil); \
             assert(issecretvalue(SecretAlteredFormUnit)); \
             assert(secretunwrap(SecretAlteredFormUnit) == 'player'); \
             AssertAlteredForm(SecretAlteredFormUnit, {expected}); \
             assert(debug.getstacktaint() == before); \
             assert(issecretvalue(SecretAlteredFormUnit))"
        ))
        .expect("AllowedWhenUntainted accepts authentic VM secret string without declassifying it");
    }
}

#[test]
fn tainted_caller_rejects_host_secret_string_and_recovers_with_public_input() {
    let env = fixture_env();
    install_host_secret_unit(&env);
    env.state().borrow_mut().player.wants_altered_form = true;
    env.exec(
        r#"
        assert(type(C_UnitAuras.WantsAlteredForm) == 'function')
        local function probe()
            local before = debug.getstacktaint()
            assert(before == 'AlteredFormProbe')
            local ok, err = pcall(C_UnitAuras.WantsAlteredForm, SecretAlteredFormUnit)
            assert(not ok and type(err) == 'string' and #err > 0)
            assert(debug.getstacktaint() == before, 'rejection retains taint')
            assert(issecretvalue(SecretAlteredFormUnit))
            assert(not pcall(secretunwrap, SecretAlteredFormUnit))
            AssertAlteredForm('player', true)
            assert(debug.getstacktaint() == before, 'public recovery retains taint')
        end
        debug.setobjecttaint(probe, 'AlteredFormProbe')
        probe()
        assert(debug.getstacktaint() == nil)
        AssertAlteredForm(SecretAlteredFormUnit, true)
        AssertAlteredForm('player', true)
        "#,
    )
    .expect("VM authenticates secret access, does not clear caller taint");
    assert!(env.state().borrow().player.wants_altered_form);
}

#[test]
fn rooted_secret_string_survives_gc_with_identity_security_and_public_recovery() {
    let env = fixture_env();
    install_host_secret_unit(&env);
    env.state().borrow_mut().player.wants_altered_form = true;
    env.exec(
        r#"
        local retained = SecretAlteredFormUnit
        collectgarbage('collect')
        collectgarbage('collect')
        assert(rawequal(retained, SecretAlteredFormUnit))
        assert(issecretvalue(retained) and secretunwrap(retained) == 'player')
        AssertAlteredForm(retained, true)
        assert(issecretvalue(retained))
        local function probe()
            assert(debug.getstacktaint() == 'AlteredFormGCProbe')
            local ok, err = pcall(C_UnitAuras.WantsAlteredForm, retained)
            assert(not ok and type(err) == 'string' and #err > 0)
            assert(debug.getstacktaint() == 'AlteredFormGCProbe')
            assert(issecretvalue(retained) and not pcall(secretunwrap, retained))
            AssertAlteredForm('player', true)
            assert(debug.getstacktaint() == 'AlteredFormGCProbe')
        end
        debug.setobjecttaint(probe, 'AlteredFormGCProbe')
        probe()
        assert(debug.getstacktaint() == nil)
        AssertAlteredForm(retained, true)
        AssertAlteredForm('player', true)
        assert(rawequal(retained, SecretAlteredFormUnit) and issecretvalue(retained))
        "#,
    )
    .expect("GC preserves rooted host-secret unit and authenticated query boundary");
    assert!(env.state().borrow().player.wants_altered_form);
}

#[test]
fn cached_unit_util_consumer_uses_input_for_worgen_and_dracthyr_only() {
    let env = fixture_env();
    let cache = wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("consumer fixture requires populated active-profile Blizzard UI cache");
    let path = cache.join("Blizzard_SharedXML/UnitUtil.lua");
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read unmodified consumer {}: {error}", path.display()));
    env.exec(&source)
        .expect("load complete unmodified UnitUtil.lua");
    for (race, filename, ordinary) in [
        (0, "Human", true),
        (10, "Worgen", false),
        (13, "Dracthyr", false),
    ] {
        for wants in [false, true, false] {
            {
                let mut state = env.state().borrow_mut();
                state.player.race_index = race;
                state.player.wants_altered_form = wants;
            }
            env.exec(&format!(
                "assert(select(2, UnitRace('player')) == '{filename}'); \
                 AssertAlteredForm('player', {wants}); \
                 assert(PlayerUtil.ShouldUseNativeFormInModelScene() == {})",
                ordinary || wants
            ))
            .expect(
                "cached consumer selects native form from real race and query, no 3D rendering",
            );
            assert_eq!(env.state().borrow().player.wants_altered_form, wants);
        }
    }
}
