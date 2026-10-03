//! Bounded Retail 12.0.5 `C_Spell.GetSpellMaxCumulativeAuraApplications`, not native parity.
//! The maximum map, zero-on-miss and the restriction flag are explicit simulator inputs.
#![cfg(feature = "retail-12-0-5")]

use rilua::LuaApiMut;
use rilua::table_security::wrap_host_secret_number;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::AuraInfo;

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create cumulative aura environment");
    env.exec(
        r#"
        function AssertMaxApplications(identifier, expected, restricted)
            local function check(...)
                assert(select('#', ...) == 1, 'exactly one result')
                local value = ...
                assert(issecretvalue(value) == restricted, 'secrecy')
                assert(secretunwrap(value) == expected, 'maximum')
                return value
            end
            return check(C_Spell.GetSpellMaxCumulativeAuraApplications(identifier))
        end
        function RejectMaxApplications(...)
            local query = C_Spell.GetSpellMaxCumulativeAuraApplications
            assert(type(query) == 'function', 'real registered query required')
            local ok, err = pcall(query, ...)
            assert(not ok and type(err) == 'string' and #err > 0, 'argument rejection')
        end
        "#,
    )
    .expect("install assertions without replacing the query");
    env
}

fn seeded_env() -> WowLuaEnv {
    let env = fixture_env();
    {
        let mut state = env.state().borrow_mut();
        state
            .spell_max_cumulative_aura_applications
            .extend([(101, 7), (202, 13)]);
        state.spell_id_aliases.clear();
        state
            .spell_id_aliases
            .insert("maximum fixture".into(), 101);
    }
    env
}

#[test]
fn numeric_and_seeded_name_identifiers_return_the_declared_maximum() {
    let env = seeded_env();
    env.exec(
        r#"
        AssertMaxApplications(101, 7, false)
        AssertMaxApplications(202, 13, false)
        AssertMaxApplications('maximum fixture', 7, false)
        AssertMaxApplications('MAXIMUM Fixture', 7, false)
        "#,
    )
    .expect("public number and case-normalized alias resolve the same key");
}

#[test]
fn numeric_alias_takes_precedence_over_numeric_identity() {
    let env = seeded_env();
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .insert("101".into(), 202);
    env.exec("AssertMaxApplications(101, 13, false); AssertMaxApplications('101', 13, false)")
        .expect("alias-first resolution");
}

#[test]
fn maximum_is_independent_of_currently_applied_stacks() {
    let env = seeded_env();
    env.state().borrow_mut().player.buffs = vec![AuraInfo {
        name: "Stacking fixture".into(),
        spell_id: 101,
        icon: 134973,
        duration: 30.0,
        expiration_time: 45.0,
        applications: 2,
        source_unit: "player".into(),
        is_helpful: true,
        is_raid: false,
        is_nameplate_only: false,
        is_stealable: false,
        can_apply_aura: true,
        is_from_player_or_player_pet: true,
        dispel_type: None,
        aura_instance_id: 501,
    }];
    env.exec("AssertMaxApplications(101, 7, false)")
        .expect("declared maximum, not the active application count");
}

#[test]
fn misses_return_one_zero_and_map_changes_are_live() {
    let env = seeded_env();
    env.exec(
        r#"
        AssertMaxApplications(99999, 0, false)
        AssertMaxApplications('unseeded name', 0, false)
        AssertMaxApplications(0, 0, false)
        "#,
    )
    .expect("inferred zero for an undeclared spell");
    {
        let mut state = env.state().borrow_mut();
        state
            .spell_max_cumulative_aura_applications
            .insert(101, 9);
        state.spell_max_cumulative_aura_applications.remove(&202);
    }
    env.exec("AssertMaxApplications(101, 9, false); AssertMaxApplications(202, 0, false)")
        .expect("replace and remove are immediate");
    let fresh = fixture_env();
    fresh
        .exec("AssertMaxApplications(101, 0, false)")
        .expect("maxima do not leak across environments");
}

#[test]
fn invalid_identifier_representations_are_rejected() {
    let env = seeded_env();
    env.exec(
        r#"
        RejectMaxApplications()
        RejectMaxApplications(nil)
        RejectMaxApplications({})
        RejectMaxApplications(true)
        RejectMaxApplications(-1)
        RejectMaxApplications(101.5)
        RejectMaxApplications(0/0)
        AssertMaxApplications(101, 7, false)
        "#,
    )
    .expect("strict public representations, then ordinary recovery");
}

#[test]
fn unit_aura_restriction_alone_makes_the_result_secret() {
    let env = seeded_env();
    {
        let mut state = env.state().borrow_mut();
        state.cooldowns_restricted = true;
        state.unit_stats_restricted = true;
    }
    env.exec("AssertMaxApplications(101, 7, false)")
        .expect("cooldown and unit-stat restrictions are unrelated");
    env.state().borrow_mut().unit_auras_restricted = true;
    env.exec(
        r#"
        Saved = AssertMaxApplications(101, 7, true)
        AssertMaxApplications('maximum fixture', 7, true)
        AssertMaxApplications(99999, 0, true)
        "#,
    )
    .expect("restricted results, including the miss, are secret host numbers");
    env.state().borrow_mut().unit_auras_restricted = false;
    env.exec(
        r#"
        AssertMaxApplications(101, 7, false)
        collectgarbage('collect')
        assert(issecretvalue(Saved) and secretunwrap(Saved) == 7, 'earlier result stays secret')
        "#,
    )
    .expect("plain again; an earlier secret result is not declassified");
}

#[test]
fn tainted_callers_keep_their_taint_and_cannot_open_restricted_results() {
    let env = seeded_env();
    env.exec(
        r#"
        local function probe()
            assert(debug.getstacktaint() == 'MaxApplicationsProbe')
            AssertMaxApplications(101, 7, false)
            AssertMaxApplications('maximum fixture', 7, false)
            assert(debug.getstacktaint() == 'MaxApplicationsProbe')
        end
        debug.setobjecttaint(probe, 'MaxApplicationsProbe')
        probe()
        assert(debug.getstacktaint() == nil)
        "#,
    )
    .expect("public queries work under taint without clearing it");
    env.state().borrow_mut().unit_auras_restricted = true;
    env.exec(
        r#"
        local function probe()
            local value = C_Spell.GetSpellMaxCumulativeAuraApplications(101)
            assert(issecretvalue(value) and not canaccessvalue(value))
            assert(not pcall(secretunwrap, value))
            assert(not pcall(function() return value + 1 end))
            assert(debug.getstacktaint() == 'MaxApplicationsProbe')
        end
        debug.setobjecttaint(probe, 'MaxApplicationsProbe')
        probe()
        AssertMaxApplications(101, 7, true)
        "#,
    )
    .expect("restricted result is opaque to a tainted caller");
}

#[test]
fn secret_identifiers_are_rejected_as_unmodeled_policy() {
    let env = seeded_env();
    {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        rilua::table_security::register_table_security(&mut lua)
            .expect("install VM security helpers");
        let value = wrap_host_secret_number(lua.state_mut(), 101.0);
        lua.state_mut().push(value);
        let inserted = lua.set_global_val("SecretMaxSpell", value);
        lua.state_mut().pop();
        inserted.expect("root authentic host-secret NUMBER");
    }
    env.exec(
        r#"
        RejectMaxApplications(SecretMaxSpell)
        local function probe()
            RejectMaxApplications(SecretMaxSpell)
            assert(debug.getstacktaint() == 'MaxApplicationsSecretProbe')
        end
        debug.setobjecttaint(probe, 'MaxApplicationsSecretProbe')
        probe()
        assert(issecretvalue(SecretMaxSpell) and secretunwrap(SecretMaxSpell) == 101)
        AssertMaxApplications(101, 7, false)
        "#,
    )
    .expect("conservative rejection without declassification; public recovery");
}
