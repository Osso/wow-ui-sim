//! Row394 and its consumer prerequisite. Carryover, nil/error and secret policies
//! are INFERRED simulator contracts, not native-verified semantics.
#![cfg(feature = "retail-12-0-5")]

use rilua::LuaApiMut;
use rilua::table_security::wrap_host_secret_number;
use wow_ui_sim::c_api::aura_duration::SpellAuraDuration;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{AuraInfo, PartyMember};

fn fixture_aura(spell_id: i32, instance_id: i32, helpful: bool, expiration: f64) -> AuraInfo {
    AuraInfo {
        name: format!("Duration fixture {instance_id}"),
        spell_id,
        icon: 134973,
        duration: 99.0,
        expiration_time: expiration,
        applications: 1,
        source_unit: "player".into(),
        is_helpful: helpful,
        is_raid: false,
        is_nameplate_only: false,
        is_stealable: false,
        can_apply_aura: true,
        is_from_player_or_player_pet: true,
        dispel_type: None,
        aura_instance_id: instance_id,
    }
}

fn fixture_party(expiration: f64) -> PartyMember {
    PartyMember {
        map_position: None,
        name: "Duration fixture member".into(),
        name_cached: true,
        connected: true,
        class_index: 2,
        level: 80,
        health: 100,
        health_max: 100,
        power: 100,
        power_max: 100,
        power_type: 0,
        power_type_name: "MANA".into(),
        is_leader: false,
        dead_since: None,
        buffs: vec![fixture_aura(99002, 101, true, expiration)],
        debuffs: vec![fixture_aura(99001, 202, false, expiration)],
    }
}

fn seeded_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create duration environment");
    {
        let mut state = env.state().borrow_mut();
        let expiration = state.start_time.elapsed().as_secs_f64() + 600.0;
        state.player.buffs = vec![
            fixture_aura(99001, 101, true, expiration),
            fixture_aura(99002, 102, false, expiration),
        ];
        state.party_members = vec![fixture_party(expiration)];
        state.spell_aura_durations.insert(
            99001,
            SpellAuraDuration {
                base_duration_seconds: 20.0,
                max_carryover_seconds: 6.0,
            },
        );
        state.spell_aura_durations.insert(
            99002,
            SpellAuraDuration {
                base_duration_seconds: 40.0,
                max_carryover_seconds: 5.0,
            },
        );
        state.spell_id_aliases.clear();
        state
            .spell_id_aliases
            .insert("duration override".into(), 99002);
    }
    env
}

fn assert_known_duration_outputs(env: &WowLuaEnv) {
    env.exec(
        r#"
        assert(C_UnitAuras.GetAuraBaseDuration('player', 101) == 20)
        assert(C_UnitAuras.GetRefreshExtendedDuration('player', 101) == 26)
        "#,
    )
    .expect("both getters exist and recover with known public inputs");
}

#[test]
fn empty_metadata_never_infers_base_from_current_aura_duration() {
    let env = seeded_env();
    env.state().borrow_mut().spell_aura_durations.clear();
    let fresh = WowLuaEnv::new().unwrap();
    assert!(fresh.state().borrow().spell_aura_durations.is_empty());
    env.exec(
        r#"
        assert(C_UnitAuras.GetAuraBaseDuration('player', 101) == nil)
        assert(C_UnitAuras.GetRefreshExtendedDuration('player', 101) == nil)
        assert(C_UnitAuras.GetAuraBaseDuration('player', 101, 99002) == nil)
        assert(C_UnitAuras.GetRefreshExtendedDuration('player', 101, 99002) == nil)
        "#,
    )
    .expect("unknown duration metadata stays unknown despite current duration 99");
}

#[test]
fn omitted_and_nil_identifier_use_matched_aura_and_return_one_number() {
    let env = seeded_env();
    env.exec(
        r#"
        for _, getter in ipairs({C_UnitAuras.GetAuraBaseDuration,
                                  C_UnitAuras.GetRefreshExtendedDuration}) do
            assert(select('#', getter('player', 101)) == 1)
            assert(select('#', getter('player', 101, nil)) == 1)
            assert(type(getter('player', 101)) == 'number')
            assert(getter('player', 101) == getter('player', 101, nil))
        end
        assert(C_UnitAuras.GetAuraBaseDuration('player', 101) == 20)
        assert(C_UnitAuras.GetRefreshExtendedDuration('player', 101) == 26)
        "#,
    )
    .expect("omitted/nil use aura spell, not current duration 99");
}

#[test]
fn explicit_numeric_and_seeded_alias_override_duration_without_matching_aura_spell() {
    let env = seeded_env();
    env.exec(
        r#"
        for _, identifier in ipairs({99002, 'DURATION Override'}) do
            assert(C_Spell.GetSpellIDForSpellIdentifier(identifier) == 99002)
            assert(C_UnitAuras.GetAuraBaseDuration('player', 101, identifier) == 40)
            assert(C_UnitAuras.GetRefreshExtendedDuration('player', 101, identifier) == 45)
        end
        "#,
    )
    .expect("explicit identifiers choose recast metadata, not active spell metadata");
}

#[test]
fn numeric_alias_applies_only_to_explicit_identifier_not_omitted_aura_spell() {
    let env = seeded_env();
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .insert("99001".into(), 99002);
    env.exec(
        r#"
        assert(C_UnitAuras.GetAuraBaseDuration('player', 101) == 20)
        assert(C_UnitAuras.GetRefreshExtendedDuration('player', 101, nil) == 26)
        assert(C_UnitAuras.GetAuraBaseDuration('player', 101, 99001) == 40)
        assert(C_UnitAuras.GetRefreshExtendedDuration('player', 101, '99001') == 45)
        "#,
    )
    .expect("explicit alias-first resolver does not rewrite matched aura spell");
}

#[test]
fn unknown_alias_or_metadata_returns_one_nil_without_falling_back_to_aura_spell() {
    let env = seeded_env();
    env.state()
        .borrow_mut()
        .spell_id_aliases
        .insert("unknown duration".into(), 99999);
    env.exec(
        r#"
        for _, getter in ipairs({C_UnitAuras.GetAuraBaseDuration,
                                  C_UnitAuras.GetRefreshExtendedDuration}) do
            for _, identifier in ipairs({99999, 'unknown duration', 'unseeded alias', ''}) do
                assert(getter('player', 101, identifier) == nil)
                assert(select('#', getter('player', 101, identifier)) == 1)
            end
            assert(getter('player', 101) ~= nil)
        end
        "#,
    )
    .expect("unresolved override never falls back");
    assert_known_duration_outputs(&env);
}

#[test]
fn unsaturated_refresh_uses_environment_elapsed_clock_with_moving_time_bounds() {
    let env = seeded_env();
    let expiration = env.state().borrow().start_time.elapsed().as_secs_f64() + 3.0;
    env.state().borrow_mut().player.buffs[0].expiration_time = expiration;
    let before = env.state().borrow().start_time.elapsed().as_secs_f64();
    let actual: f64 = env
        .eval("return C_UnitAuras.GetRefreshExtendedDuration('player', 101)")
        .unwrap();
    let after = env.state().borrow().start_time.elapsed().as_secs_f64();
    let tolerance = 0.001;
    assert!(actual >= 20.0 + (expiration - after).max(0.0) - tolerance);
    assert!(actual <= 20.0 + (expiration - before).max(0.0) + tolerance);
    assert!(actual <= 23.0 + tolerance);
}

#[test]
fn saturated_refresh_uses_explicit_cap_not_thirty_percent_or_current_duration() {
    let env = seeded_env();
    env.exec(
        r#"
        assert(C_UnitAuras.GetRefreshExtendedDuration('player', 102) == 45)
        assert(C_UnitAuras.GetAuraBaseDuration('player', 102) == 40)
        "#,
    )
    .expect("INFERRED explicit cap 5, not 30 percent of base 40 or current 99");
}

#[test]
fn expired_timed_aura_returns_exact_base_without_mutating_expiration() {
    let env = seeded_env();
    env.state().borrow_mut().player.buffs[0].expiration_time = -1.0;
    env.exec(
        r#"
        assert(C_UnitAuras.GetRefreshExtendedDuration('player', 101) == 20)
        assert(C_UnitAuras.GetAuraBaseDuration('player', 101) == 20)
        "#,
    )
    .expect("INFERRED expired aura has zero carryover");
    assert_eq!(env.state().borrow().player.buffs[0].expiration_time, -1.0);
}

#[test]
fn permanent_duration_or_expiration_returns_nil_refresh_but_known_base() {
    let env = seeded_env();
    for (duration, expiration) in [(0.0, 600.0), (99.0, 0.0), (0.0, 0.0)] {
        {
            let mut state = env.state().borrow_mut();
            state.player.buffs[0].duration = duration;
            state.player.buffs[0].expiration_time = expiration;
        }
        env.exec(
            r#"
            assert(C_UnitAuras.GetRefreshExtendedDuration('player', 101) == nil)
            assert(C_UnitAuras.GetAuraBaseDuration('player', 101) == 20)
            "#,
        )
        .expect("INFERRED permanent aura refresh ineligible; base remains metadata");
    }
}

#[test]
fn party_polarities_and_blocking_are_unit_and_instance_isolated() {
    let env = seeded_env();
    env.exec(
        r#"
        assert(C_UnitAuras.GetAuraBaseDuration('party1', 101) == 40)
        assert(C_UnitAuras.GetRefreshExtendedDuration('party1', 101) == 45)
        assert(C_UnitAuras.GetAuraBaseDuration('party1', 202) == 20)
        assert(C_UnitAuras.GetRefreshExtendedDuration('party1', 202) == 26)
        C_UnitAuras.AddBlockedAura('party1', 101)
        for _, getter in ipairs({C_UnitAuras.GetAuraBaseDuration,
                                  C_UnitAuras.GetRefreshExtendedDuration}) do
            assert(getter('party1', 101) == nil)
            assert(getter('party1', 101, 99001) == nil)
            assert(getter('player', 101) ~= nil)
            assert(getter('party1', 202) ~= nil)
        end
        C_UnitAuras.AddBlockedAura('player', 102)
        assert(C_UnitAuras.GetAuraBaseDuration('player', 102) == nil)
        assert(C_UnitAuras.GetRefreshExtendedDuration('player', 102) == nil)
        assert(C_UnitAuras.GetRefreshExtendedDuration('player', 101) == 26)
        "#,
    )
    .expect("existing filtered public store, including harmful rows and unit-local blocking");
}

#[test]
fn unknown_units_and_instances_stay_nil_even_with_known_override() {
    let env = seeded_env();
    env.exec(
        r#"
        for _, getter in ipairs({C_UnitAuras.GetAuraBaseDuration,
                                  C_UnitAuras.GetRefreshExtendedDuration}) do
            for _, unit in ipairs({'missing-unit', 'party2', 'pet', 'raid1', ''}) do
                assert(getter(unit, 101, 99001) == nil)
            end
            assert(getter(nil, 101, 99001) == nil)
            assert(getter('player', 99999, 99001) == nil)
            assert(getter('player', 202, 99001) == nil)
            assert(getter('party1', 102, 99001) == nil)
            assert(getter('player', 101) ~= nil)
        end
        "#,
    )
    .expect("metadata cannot create an aura or cross unit/instance identity");
    assert_known_duration_outputs(&env);
}

#[test]
fn metadata_and_alias_mutations_are_environment_local() {
    let first = seeded_env();
    let second = seeded_env();
    {
        let mut state = first.state().borrow_mut();
        state
            .spell_aura_durations
            .get_mut(&99001)
            .unwrap()
            .base_duration_seconds = 10.0;
        state
            .spell_id_aliases
            .insert("duration override".into(), 99001);
    }
    first
        .exec(
            r#"
        assert(C_UnitAuras.GetAuraBaseDuration('player', 101, 'duration override') == 10)
        assert(C_UnitAuras.GetRefreshExtendedDuration('player', 101) == 16)
        "#,
        )
        .expect("first environment observes its metadata");
    second
        .exec(
            r#"
        assert(C_UnitAuras.GetAuraBaseDuration('player', 101, 'duration override') == 40)
        assert(C_UnitAuras.GetRefreshExtendedDuration('player', 101) == 26)
        "#,
        )
        .expect("second environment retains independent metadata and alias");
}

#[test]
fn invalid_metadata_never_exposes_negative_or_nonfinite_public_numbers() {
    let env = seeded_env();
    for invalid in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for metadata in [
            SpellAuraDuration {
                base_duration_seconds: invalid,
                max_carryover_seconds: 6.0,
            },
            SpellAuraDuration {
                base_duration_seconds: 20.0,
                max_carryover_seconds: invalid,
            },
        ] {
            env.state()
                .borrow_mut()
                .spell_aura_durations
                .insert(99001, metadata);
            env.exec(
                r#"
                assert(C_UnitAuras.GetAuraBaseDuration('player', 101) == nil)
                assert(C_UnitAuras.GetRefreshExtendedDuration('player', 101) == nil)
                assert(C_UnitAuras.GetRefreshExtendedDuration('player', 102) == 45)
                "#,
            )
            .expect("INFERRED invalid metadata yields nil without poisoning other records");
        }
    }
}

#[test]
fn finite_metadata_sum_overflow_never_becomes_public_infinity() {
    let env = seeded_env();
    {
        let mut state = env.state().borrow_mut();
        state.spell_aura_durations.insert(
            99001,
            SpellAuraDuration {
                base_duration_seconds: f64::MAX,
                max_carryover_seconds: f64::MAX,
            },
        );
        state.player.buffs[0].expiration_time = f64::MAX;
    }
    env.exec(
        r#"
        assert(C_UnitAuras.GetAuraBaseDuration('player', 101) == 1.7976931348623157e308)
        assert(C_UnitAuras.GetRefreshExtendedDuration('player', 101) == nil)
        "#,
    )
    .expect("INFERRED overflow yields nil rather than nonfinite public refresh output");
}

#[test]
fn zero_base_and_zero_cap_are_valid_explicit_metadata() {
    let env = seeded_env();
    env.state().borrow_mut().spell_aura_durations.insert(
        99001,
        SpellAuraDuration {
            base_duration_seconds: 0.0,
            max_carryover_seconds: 0.0,
        },
    );
    env.exec(
        r#"
        assert(C_UnitAuras.GetAuraBaseDuration('player', 101) == 0)
        assert(C_UnitAuras.GetRefreshExtendedDuration('player', 101) == 0)
        "#,
    )
    .expect("zero metadata does not imply permanent active aura");
}

#[test]
fn malformed_arguments_error_before_missing_unit_short_circuit() {
    let env = seeded_env();
    env.exec(
        r#"
        local function reject(getter, ...)
            local ok, err = pcall(getter, ...)
            assert(not ok and type(err) == 'string' and #err > 0)
        end
        for _, getter in ipairs({C_UnitAuras.GetAuraBaseDuration,
                                  C_UnitAuras.GetRefreshExtendedDuration}) do
            reject(getter)
            reject(getter, 'player')
            reject(getter, 'player', nil)
            for _, value in ipairs({false, true, {}, function() end, 0/0, math.huge, -math.huge}) do
                reject(getter, value, 101)
                reject(getter, 'player', value)
                reject(getter, 'missing-unit', value)
                reject(getter, 'player', 101, value)
                reject(getter, 'missing-unit', 101, value)
            end
            reject(getter, 1, 101)
            reject(getter, 'player', '101')
            assert(getter('player', 101) ~= nil)
        end
        "#,
    ).expect("INFERRED public unit string/nil, finite numeric instance and optional identifier validation");
    assert_known_duration_outputs(&env);
}

#[test]
fn actual_secret_arguments_are_rejected_without_declassification_or_taint_changes() {
    let env = seeded_env();
    {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        for (name, number) in [
            ("SecretDurationInstance", 101.0),
            ("SecretDurationSpell", 99001.0),
        ] {
            let secret = wrap_host_secret_number(lua.state_mut(), number);
            lua.state_mut().push(secret);
            let inserted = lua.set_global_val(name, secret);
            lua.state_mut().pop();
            inserted.expect("install real host-secret duration argument");
        }
    }
    env.exec(
        r#"
        collectgarbage('collect')
        local function probe()
            local before = debug.getstacktaint()
            for _, getter in ipairs({C_UnitAuras.GetAuraBaseDuration,
                                      C_UnitAuras.GetRefreshExtendedDuration}) do
                for _, call in ipairs({
                    function() return getter('player', SecretDurationInstance) end,
                    function() return getter('missing-unit', SecretDurationInstance) end,
                    function() return getter('player', 101, SecretDurationSpell) end,
                    function() return getter('missing-unit', 101, SecretDurationSpell) end,
                    function() return getter(SecretDurationSpell, 101) end,
                }) do
                    local ok, err = pcall(call)
                    assert(not ok and type(err) == 'string' and #err > 0)
                    assert(debug.getstacktaint() == before)
                    assert(issecretvalue(SecretDurationInstance))
                    assert(issecretvalue(SecretDurationSpell))
                end
                assert(getter('player', 101) ~= nil)
                assert(debug.getstacktaint() == before)
            end
        end
        assert(issecure())
        probe()
        local function addon()
            assert(debug.getstacktaint() == 'AuraDurationFixture')
            probe()
        end
        debug.setobjecttaint(addon, 'AuraDurationFixture')
        addon()
        assert(issecure())
        "#,
    )
    .expect("INFERRED conservative rejection; not native AllowedWhenTainted parity");
    assert_known_duration_outputs(&env);
}

#[test]
fn getters_leave_aura_metadata_aliases_and_events_unchanged_across_gc() {
    let env = seeded_env();
    let (metadata, aliases, expiration, event_count) = {
        let state = env.state().borrow();
        (
            state.spell_aura_durations.clone(),
            state.spell_id_aliases.clone(),
            state.player.buffs[0].expiration_time,
            state.events.pending().len(),
        )
    };
    env.exec(
        r#"
        for i = 1, 3 do
            assert(C_UnitAuras.GetAuraBaseDuration('player', 101) == 20)
            assert(C_UnitAuras.GetRefreshExtendedDuration('player', 101) == 26)
            assert(C_UnitAuras.GetAuraBaseDuration('party1', 202, 'duration override') == 40)
            collectgarbage('collect')
        end
        "#,
    )
    .expect("duration getters are immutable queries");
    let state = env.state().borrow();
    assert_eq!(state.spell_aura_durations, metadata);
    assert_eq!(state.spell_id_aliases, aliases);
    assert_eq!(state.player.buffs.len(), 2);
    assert_eq!(state.player.buffs[0].spell_id, 99001);
    assert_eq!(state.player.buffs[0].aura_instance_id, 101);
    assert_eq!(state.player.buffs[0].duration, 99.0);
    assert_eq!(state.player.buffs[0].expiration_time, expiration);
    assert_eq!(state.events.pending().len(), event_count);
}
