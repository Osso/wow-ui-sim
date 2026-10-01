//! Bounded Retail 12.0.5 aura filter query inputs, not native parity proof.
//! Strict representation, missing-instance and security results are inferred.
#![cfg(feature = "retail-12-0-5")]

use rilua::LuaApiMut;
use rilua::table_security::{wrap_host_secret_number, wrap_host_secret_string};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::AuraInfo;

fn fixture_aura(id: i32, helpful: bool, from_player: bool) -> AuraInfo {
    AuraInfo {
        name: format!("Filter fixture {id}"),
        spell_id: 99000 + id,
        icon: 134973,
        duration: 30.0,
        expiration_time: 45.0,
        applications: 3,
        source_unit: if from_player { "pet" } else { "party1" }.into(),
        is_helpful: helpful,
        is_raid: false,
        is_nameplate_only: false,
        is_stealable: false,
        can_apply_aura: true,
        is_from_player_or_player_pet: from_player,
        dispel_type: Some("Magic".into()),
        aura_instance_id: id,
    }
}

fn assert_aura_records(actual: &[AuraInfo], expected: &[AuraInfo]) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
        assert_eq!(
            (
                &actual.name,
                actual.spell_id,
                actual.icon,
                actual.duration,
                actual.expiration_time,
                actual.applications,
                &actual.source_unit,
                actual.aura_instance_id
            ),
            (
                &expected.name,
                expected.spell_id,
                expected.icon,
                expected.duration,
                expected.expiration_time,
                expected.applications,
                &expected.source_unit,
                expected.aura_instance_id
            ),
        );
        assert_eq!(
            (
                actual.is_helpful,
                actual.is_raid,
                actual.is_nameplate_only,
                actual.is_stealable,
                actual.can_apply_aura,
                actual.is_from_player_or_player_pet,
                &actual.dispel_type
            ),
            (
                expected.is_helpful,
                expected.is_raid,
                expected.is_nameplate_only,
                expected.is_stealable,
                expected.can_apply_aura,
                expected.is_from_player_or_player_pet,
                &expected.dispel_type
            ),
        );
    }
}

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create aura filter environment");
    {
        let mut state = env.state().borrow_mut();
        state.player.buffs = vec![
            fixture_aura(101, true, true),
            fixture_aura(102, true, false),
            fixture_aura(103, false, true),
            fixture_aura(104, false, false),
        ];
        let party = state
            .party_members
            .first_mut()
            .expect("existing seeded roster");
        party.buffs = vec![
            fixture_aura(201, true, true),
            fixture_aura(202, true, false),
        ];
        party.debuffs = vec![
            fixture_aura(203, false, true),
            fixture_aura(204, false, false),
        ];
    }
    env.exec(
        r#"
        function AssertAuraFiltered(unit, id, filter, expected)
            local function check(...)
                assert(select('#', ...) == 1, 'exactly one query result')
                local value = ...
                assert(type(value) == 'boolean', 'boolean result')
                assert(not issecretvalue(value), 'public result')
                assert(value == expected, 'filter result mismatch')
            end
            check(C_UnitAuras.IsAuraFilteredOutByInstanceID(unit, id, filter))
        end
        function RejectAuraFilter(...)
            assert(type(C_UnitAuras.IsAuraFilteredOutByInstanceID) == 'function',
                'real query must be registered before testing rejection')
            local ok, err = pcall(C_UnitAuras.IsAuraFilteredOutByInstanceID, ...)
            assert(not ok and type(err) == 'string' and #err > 0)
        end
        "#,
    )
    .expect("install assertions without replacing query or vendor functions");
    env
}

fn install_host_secrets(env: &WowLuaEnv) {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).expect("install VM security helpers");
    for (name, text) in [
        ("SecretFilterUnit", "player"),
        ("SecretAuraFilter", "HELPFUL|PLAYER"),
        ("SecretUnknownFilterUnit", "missing-unit"),
    ] {
        let value = wrap_host_secret_string(lua.state_mut(), text);
        lua.state_mut().push(value);
        let inserted = lua.set_global_val(name, value);
        lua.state_mut().pop();
        inserted.expect("root authentic host-secret STRING");
    }
    let value = wrap_host_secret_number(lua.state_mut(), 101.0);
    lua.state_mut().push(value);
    let inserted = lua.set_global_val("SecretFilterID", value);
    lua.state_mut().pop();
    inserted.expect("root authentic host-secret NUMBER");
}

#[test]
fn player_helpful_and_harmful_queries_return_exactly_one_public_boolean() {
    let env = fixture_env();
    env.exec(
        r#"
        for _, id in ipairs({101, 102}) do
            AssertAuraFiltered('player', id, 'HELPFUL', false)
            AssertAuraFiltered('player', id, 'HARMFUL', true)
        end
        for _, id in ipairs({103, 104}) do
            AssertAuraFiltered('player', id, 'HELPFUL', true)
            AssertAuraFiltered('player', id, 'HARMFUL', false)
        end
        "#,
    )
    .expect("stored player polarity, arity and public boolean");
}

#[test]
fn player_source_filter_uses_existing_from_player_or_pet_field() {
    let env = fixture_env();
    assert_eq!(env.state().borrow().player.buffs[0].source_unit, "pet");
    env.exec(
        r#"
        assert(AuraUtil.GetAuraDataByAuraInstanceID('player', 101).isFromPlayerOrPlayerPet)
        AssertAuraFiltered('player', 101, 'HELPFUL|PLAYER', false)
        AssertAuraFiltered('player', 102, 'HELPFUL|PLAYER', true)
        AssertAuraFiltered('player', 103, 'HARMFUL|PLAYER', false)
        AssertAuraFiltered('player', 104, 'HARMFUL|PLAYER', true)
        AssertAuraFiltered('player', 103, 'HELPFUL|PLAYER', true)
        AssertAuraFiltered('player', 101, 'HARMFUL|PLAYER', true)
        AssertAuraFiltered('player', 101, 'PLAYER', false)
        AssertAuraFiltered('player', 102, 'PLAYER', true)
        "#,
    )
    .expect("existing PLAYER predicate includes player-pet flag, not literal source token");
}

#[test]
fn seeded_party_records_obey_both_polarities_and_player_source() {
    let env = fixture_env();
    env.exec(
        r#"
        assert(AuraUtil.GetAuraDataByAuraInstanceID('party1', 201).isHelpful)
        assert(AuraUtil.GetAuraDataByAuraInstanceID('party1', 203).isHarmful)
        AssertAuraFiltered('party1', 201, 'HELPFUL', false)
        AssertAuraFiltered('party1', 202, 'HELPFUL', false)
        AssertAuraFiltered('party1', 203, 'HARMFUL', false)
        AssertAuraFiltered('party1', 204, 'HARMFUL', false)
        AssertAuraFiltered('party1', 201, 'HARMFUL', true)
        AssertAuraFiltered('party1', 203, 'HELPFUL', true)
        AssertAuraFiltered('party1', 201, 'HELPFUL|PLAYER', false)
        AssertAuraFiltered('party1', 202, 'HELPFUL|PLAYER', true)
        AssertAuraFiltered('party1', 203, 'HARMFUL|PLAYER', false)
        AssertAuraFiltered('party1', 204, 'HARMFUL|PLAYER', true)
        "#,
    )
    .expect("concrete records in existing party stores");
}

#[test]
fn recognized_filter_case_and_player_token_order_remain_unchanged() {
    let env = fixture_env();
    env.exec(
        r#"
        AssertAuraFiltered('player', 101, 'helpful|player', false)
        AssertAuraFiltered('player', 102, 'PLAYER|HELPFUL', true)
        AssertAuraFiltered('party1', 203, 'player|harmful', false)
        AssertAuraFiltered('party1', 204, 'PLAYER|HARMFUL', true)
        "#,
    )
    .expect("existing helper case-insensitivity and recognized token combinations only");
}

#[test]
fn maw_and_external_defensive_match_none_of_existing_stored_records() {
    let env = fixture_env();
    env.exec(
        r#"
        for _, filter in ipairs({'MAW', 'EXTERNAL_DEFENSIVE',
            'HELPFUL|MAW', 'HARMFUL|EXTERNAL_DEFENSIVE', 'MAW|PLAYER'}) do
            for _, id in ipairs({101, 102, 103, 104}) do
                AssertAuraFiltered('player', id, filter, true)
            end
            for _, id in ipairs({201, 202, 203, 204}) do
                AssertAuraFiltered('party1', id, filter, true)
            end
        end
        "#,
    )
    .expect("existing modeled match-none policy, not native category coverage");
}

#[test]
fn unknown_units_and_ids_including_negative_i32_return_true() {
    let env = fixture_env();
    env.exec(
        r#"
        for _, unit in ipairs({'missing-unit', '', 'pet', 'party99'}) do
            AssertAuraFiltered(unit, 101, 'HELPFUL', true)
        end
        for _, id in ipairs({99999, 0, -1, -2147483648, 2147483647}) do
            AssertAuraFiltered('player', id, 'HELPFUL', true)
            AssertAuraFiltered('party1', id, 'HARMFUL', true)
        end
        AssertAuraFiltered('player', 201, 'HELPFUL', true)
        AssertAuraFiltered('party1', 101, 'HELPFUL', true)
        "#,
    )
    .expect("INFERRED absent-record true; negative integers are valid representations");
}

#[test]
fn blocked_instances_remain_retrievable_and_filter_by_polarity_and_source() {
    let env = fixture_env();
    env.exec(
        r#"
        C_UnitAuras.AddBlockedAura('player', 101)
        C_UnitAuras.AddBlockedAura('player', 104)
        C_UnitAuras.AddBlockedAura('party1', 201)
        assert(C_UnitAuras.GetAuraDataByIndex('player', 1, 'HELPFUL').auraInstanceID == 102)
        assert(AuraUtil.GetAuraDataByAuraInstanceID('player', 101).auraInstanceID == 101)
        assert(AuraUtil.GetAuraDataByAuraInstanceID('player', 104).auraInstanceID == 104)
        assert(AuraUtil.GetAuraDataByAuraInstanceID('party1', 201).auraInstanceID == 201)
        AssertAuraFiltered('player', 101, 'HELPFUL|PLAYER', false)
        AssertAuraFiltered('player', 101, 'HARMFUL', true)
        AssertAuraFiltered('player', 104, 'HARMFUL', false)
        AssertAuraFiltered('player', 104, 'HARMFUL|PLAYER', true)
        AssertAuraFiltered('party1', 201, 'HELPFUL|PLAYER', false)
        AssertAuraFiltered('party1', 201, 'HARMFUL', true)
        "#,
    )
    .expect("unfiltered instance lookup must not use blocked public enumeration");
}

#[test]
fn required_string_arguments_reject_missing_nil_and_wrong_representations() {
    let env = fixture_env();
    env.exec(
        r#"
        RejectAuraFilter()
        RejectAuraFilter('player')
        RejectAuraFilter('player', 101)
        RejectAuraFilter(nil, 101, 'HELPFUL')
        RejectAuraFilter('player', nil, 'HELPFUL')
        RejectAuraFilter('player', 101, nil)
        for _, value in ipairs({false, true, 0, 101, {}, function() end}) do
            RejectAuraFilter(value, 101, 'HELPFUL')
            RejectAuraFilter('player', 101, value)
            RejectAuraFilter('missing-unit', 101, value)
        end
        AssertAuraFiltered('player', 101, 'HELPFUL', false)
        "#,
    )
    .expect("INFERRED required strings before lookup; no error-wording parity claim");
}

#[test]
fn instance_id_rejects_lossy_casts_and_non_numeric_representations_before_lookup() {
    let env = fixture_env();
    env.exec(
        r#"
        for _, unit in ipairs({'player', 'missing-unit'}) do
            for _, id in ipairs({101.5, -1.5, 0/0, math.huge, -math.huge,
                2147483648, -2147483649, '101', false, {}, function() end}) do
                RejectAuraFilter(unit, id, 'HELPFUL')
            end
            RejectAuraFilter(unit, nil, 'HELPFUL')
        end
        AssertAuraFiltered('player', 101, 'HELPFUL', false)
        "#,
    )
    .expect("INFERRED finite integral i32 representation; no truncation or saturation");
}

#[test]
fn untainted_query_accepts_each_authentic_secret_argument_and_combination() {
    let env = fixture_env();
    install_host_secrets(&env);
    env.exec(
        r#"
        assert(debug.getstacktaint() == nil)
        assert(issecretvalue(SecretFilterUnit) and secretunwrap(SecretFilterUnit) == 'player')
        assert(issecretvalue(SecretAuraFilter) and secretunwrap(SecretAuraFilter) == 'HELPFUL|PLAYER')
        assert(issecretvalue(SecretFilterID) and secretunwrap(SecretFilterID) == 101)
        AssertAuraFiltered(SecretFilterUnit, 101, 'HELPFUL', false)
        AssertAuraFiltered('player', SecretFilterID, 'HELPFUL', false)
        AssertAuraFiltered('player', 101, SecretAuraFilter, false)
        AssertAuraFiltered(SecretFilterUnit, SecretFilterID, SecretAuraFilter, false)
        AssertAuraFiltered(SecretUnknownFilterUnit, SecretFilterID, SecretAuraFilter, true)
        assert(debug.getstacktaint() == nil)
        assert(issecretvalue(SecretFilterUnit) and issecretvalue(SecretAuraFilter))
        assert(issecretvalue(SecretFilterID) and issecretvalue(SecretUnknownFilterUnit))
        AssertAuraFiltered('player', 101, 'HELPFUL', false)
        "#,
    )
    .expect("AllowedWhenUntainted authenticates secrets; does not declassify inputs");
}

#[test]
fn tainted_query_denies_each_secret_even_before_unknown_unit_lookup() {
    let env = fixture_env();
    install_host_secrets(&env);
    env.exec(
        r#"
        local function probe()
            local before = debug.getstacktaint()
            assert(before == 'AuraFilterProbe')
            RejectAuraFilter(SecretFilterUnit, 101, 'HELPFUL')
            RejectAuraFilter('player', SecretFilterID, 'HELPFUL')
            RejectAuraFilter('player', 101, SecretAuraFilter)
            RejectAuraFilter(SecretFilterUnit, SecretFilterID, SecretAuraFilter)
            RejectAuraFilter(SecretUnknownFilterUnit, 101, 'HELPFUL')
            RejectAuraFilter('missing-unit', SecretFilterID, 'HELPFUL')
            RejectAuraFilter('missing-unit', 101, SecretAuraFilter)
            assert(debug.getstacktaint() == before)
            for _, value in ipairs({SecretFilterUnit, SecretFilterID, SecretAuraFilter}) do
                assert(issecretvalue(value) and not pcall(secretunwrap, value))
            end
            AssertAuraFiltered('player', 101, 'HELPFUL', false)
            AssertAuraFiltered('missing-unit', 101, 'HELPFUL', true)
            assert(debug.getstacktaint() == before, 'public recovery must retain taint')
        end
        debug.setobjecttaint(probe, 'AuraFilterProbe')
        probe()
        assert(debug.getstacktaint() == nil)
        AssertAuraFiltered(SecretFilterUnit, SecretFilterID, SecretAuraFilter, false)
        "#,
    )
    .expect("VM-authenticated denial before lookup, unchanged taint and public recovery");
}

#[test]
fn secret_unit_and_filter_gc_roots_retain_identity_secrecy_and_public_recovery() {
    let env = fixture_env();
    install_host_secrets(&env);
    env.exec(
        r#"
        local unit, filter, id = SecretFilterUnit, SecretAuraFilter, SecretFilterID
        collectgarbage('collect')
        collectgarbage('collect')
        assert(rawequal(unit, SecretFilterUnit) and rawequal(filter, SecretAuraFilter))
        assert(issecretvalue(unit) and secretunwrap(unit) == 'player')
        assert(issecretvalue(filter) and secretunwrap(filter) == 'HELPFUL|PLAYER')
        assert(issecretvalue(id) and secretunwrap(id) == 101)
        AssertAuraFiltered(unit, id, filter, false)
        local function probe()
            assert(debug.getstacktaint() == 'AuraFilterGCProbe')
            RejectAuraFilter(unit, 101, 'HELPFUL')
            RejectAuraFilter('player', 101, filter)
            assert(issecretvalue(unit) and not pcall(secretunwrap, unit))
            assert(issecretvalue(filter) and not pcall(secretunwrap, filter))
            AssertAuraFiltered('player', 101, 'HELPFUL', false)
            assert(debug.getstacktaint() == 'AuraFilterGCProbe')
        end
        debug.setobjecttaint(probe, 'AuraFilterGCProbe')
        probe()
        assert(debug.getstacktaint() == nil)
        AssertAuraFiltered(unit, id, filter, false)
        assert(issecretvalue(unit) and issecretvalue(filter) and issecretvalue(id))
        "#,
    )
    .expect("rooted real string secrets survive GC without declassification");
}

#[test]
fn queries_leave_aura_records_block_list_and_provider_selection_unchanged() {
    let env = fixture_env();
    env.exec(
        r#"
        C_UnitAuras.AddBlockedAura('player', 101)
        C_UnitAuras.SwitchAuraDataProvider()
        assert(AuraUtil.GetAuraDataByAuraInstanceID('player', 102) == nil)
        AssertAuraFiltered('player', 101, 'HELPFUL', false)
        AssertAuraFiltered('player', 104, 'HARMFUL|PLAYER', true)
        AssertAuraFiltered('party1', 203, 'HARMFUL|PLAYER', false)
        AssertAuraFiltered('missing-unit', 101, 'HELPFUL', true)
        RejectAuraFilter('player', 101.5, 'HELPFUL')
        assert(AuraUtil.GetAuraDataByAuraInstanceID('player', 102) == nil)
        C_UnitAuras.ResetAuraDataProvider()
        assert(C_UnitAuras.GetAuraDataByIndex('player', 1, 'HELPFUL').auraInstanceID == 102)
        local aura = AuraUtil.GetAuraDataByAuraInstanceID('player', 101)
        assert(aura.auraInstanceID == 101 and aura.name == 'Filter fixture 101')
        assert(aura.duration == 30 and aura.expirationTime == 45 and aura.applications == 3)
        assert(aura.isFromPlayerOrPlayerPet and aura.isHelpful and not aura.isHarmful)
        "#,
    )
    .expect("queries do not mutate provider or blocked enumeration behavior");
    let state = env.state().borrow();
    assert_aura_records(
        &state.player.buffs,
        &[
            fixture_aura(101, true, true),
            fixture_aura(102, true, false),
            fixture_aura(103, false, true),
            fixture_aura(104, false, false),
        ],
    );
    assert_aura_records(
        &state.party_members[0].buffs,
        &[
            fixture_aura(201, true, true),
            fixture_aura(202, true, false),
        ],
    );
    assert_aura_records(
        &state.party_members[0].debuffs,
        &[
            fixture_aura(203, false, true),
            fixture_aura(204, false, false),
        ],
    );
}

#[test]
fn environments_read_only_their_own_existing_aura_records() {
    let first = fixture_env();
    let second = fixture_env();
    second.state().borrow_mut().player.buffs.clear();
    second.state().borrow_mut().party_members[0].debuffs.clear();
    first.exec("AssertAuraFiltered('player', 101, 'HELPFUL', false); AssertAuraFiltered('party1', 203, 'HARMFUL', false)")
        .expect("first environment retains records");
    second.exec("AssertAuraFiltered('player', 101, 'HELPFUL', true); AssertAuraFiltered('party1', 203, 'HARMFUL', true)")
        .expect("second environment has no matching records");
    first.exec("AssertAuraFiltered('player', 101, 'HELPFUL', false); AssertAuraFiltered('party1', 203, 'HARMFUL', false)")
        .expect("second environment cannot change first");
}
