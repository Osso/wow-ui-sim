//! Retail 12.0.5 rows 367–369; fixture/spec slice, not native permission proof.
//! Decimal formatting, missing-instance and strict representation policies are inferred.
#![cfg(feature = "retail-12-0-5")]

use rilua::LuaApiMut;
use rilua::table_security::{wrap_host_secret_number, wrap_host_secret_string};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::AuraInfo;

fn fixture_aura(id: i32, applications: i32, helpful: bool) -> AuraInfo {
    AuraInfo {
        name: format!("Display count fixture {id}"),
        spell_id: 99000 + id,
        icon: 134973,
        duration: 30.0,
        expiration_time: 45.0,
        applications,
        source_unit: "pet".into(),
        is_helpful: helpful,
        is_raid: false,
        is_nameplate_only: false,
        is_stealable: false,
        can_apply_aura: true,
        is_from_player_or_player_pet: true,
        dispel_type: Some("Magic".into()),
        aura_instance_id: id,
    }
}

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create display count environment");
    {
        let mut state = env.state().borrow_mut();
        state.player.buffs = [0, 1, 2, 5, 6]
            .into_iter()
            .enumerate()
            .map(|(index, count)| fixture_aura(101 + index as i32, count, true))
            .collect();
        let party = state
            .party_members
            .first_mut()
            .expect("seeded party roster");
        party.buffs = vec![fixture_aura(201, 5, true)];
        party.debuffs = vec![fixture_aura(202, 6, false)];
    }
    env.exec(
        r#"
        function AssertDisplayResult(expected, ...)
            assert(select('#', ...) == 1, 'exactly one display count result')
            local result = ...
            assert(type(result) == 'string' and not issecretvalue(result))
            assert(result == expected, 'display count mismatch')
        end
        function CheckDisplay(expected, ...)
            AssertDisplayResult(expected, C_UnitAuras.GetAuraApplicationDisplayCount(...))
        end
        function RejectDisplay(...)
            assert(type(C_UnitAuras.GetAuraApplicationDisplayCount) == 'function')
            local ok, err = pcall(C_UnitAuras.GetAuraApplicationDisplayCount, ...)
            assert(not ok and type(err) == 'string' and #err > 0)
        end
        "#,
    )
    .expect("install assertions without replacing providers");
    env
}

fn install_host_secrets(env: &WowLuaEnv) {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).expect("actual VM security helpers");
    for (name, text) in [
        ("DisplaySecretUnit", "player"),
        ("DisplaySecretParty", "party1"),
        ("DisplaySecretUnknown", "missing-unit"),
    ] {
        let value = wrap_host_secret_string(lua.state_mut(), text);
        lua.state_mut().push(value);
        let inserted = lua.set_global_val(name, value);
        lua.state_mut().pop();
        inserted.expect("root authentic host-secret STRING");
    }
    for (name, number) in [
        ("DisplaySecretID", 104.0),
        ("DisplaySecretPartyID", 202.0),
        ("DisplaySecretMin", 2.0),
        ("DisplaySecretMax", 5.0),
        ("DisplaySecretFractionID", 104.5),
    ] {
        let value = wrap_host_secret_number(lua.state_mut(), number);
        lua.state_mut().push(value);
        let inserted = lua.set_global_val(name, value);
        lua.state_mut().pop();
        inserted.expect("root authentic host-secret NUMBER");
    }
}

#[test]
fn defaults_format_zero_one_two_five_six_as_exactly_one_public_string() {
    let env = fixture_env();
    env.exec(
        r#"
        for _, case in ipairs({{101, ''}, {102, ''}, {103, '2'}, {104, '5'}, {105, '6'}}) do
            CheckDisplay(case[2], 'player', case[1])
        end
        CheckDisplay('0', 'player', 101, 0)
        CheckDisplay('1', 'player', 102, 1)
        CheckDisplay('2', 'player', 103, 2)
        CheckDisplay('5', 'player', 104, 2, 5)
        CheckDisplay('6', 'player', 105, 2, nil)
        CheckDisplay('*', 'player', 103, 1, 1)
        "#,
    )
    .expect("documented defaults plus INFERRED decimal/public output");
}

#[test]
fn party_helpful_and_harmful_counts_use_concrete_existing_records() {
    let env = fixture_env();
    env.exec(
        r#"
        CheckDisplay('5', 'party1', 201)
        CheckDisplay('6', 'party1', 202)
        CheckDisplay('*', 'party1', 202, 2, 5)
        CheckDisplay('', 'party1', 201, 6)
        "#,
    )
    .expect("both existing party stores are count sources");
}

#[test]
fn threshold_equality_is_inclusive_and_exceeding_max_is_strict() {
    let env = fixture_env();
    env.exec(
        r#"
        CheckDisplay('', 'player', 102, 2, 5)
        CheckDisplay('2', 'player', 103, 2, 5)
        CheckDisplay('5', 'player', 104, 2, 5)
        CheckDisplay('*', 'player', 105, 2, 5)
        CheckDisplay('6', 'player', 105, 6, 6)
        "#,
    )
    .expect("below/above are strict comparisons");
}

#[test]
fn finite_fractional_negative_and_large_thresholds_apply_min_before_max() {
    let env = fixture_env();
    env.exec(
        r#"
        CheckDisplay('', 'player', 103, 2.5, 1.5)
        CheckDisplay('*', 'player', 103, 1.5, 1.5)
        CheckDisplay('2', 'player', 103, 1.5, 2.5)
        CheckDisplay('0', 'player', 101, -0.5, 0.5)
        CheckDisplay('*', 'player', 101, -2, -1)
        CheckDisplay('', 'player', 104, 6, 2)
        CheckDisplay('*', 'player', 104, 5, 2)
        CheckDisplay('', 'player', 105, 1e100, -1e100)
        CheckDisplay('6', 'player', 105, -1e100, 1e100)
        "#,
    )
    .expect("INFERRED min-first ordering; no integer/positive/i32 threshold cap");
}

#[test]
fn unknown_units_and_missing_signed_i32_instances_return_empty_string() {
    let env = fixture_env();
    env.exec(
        r#"
        for _, unit in ipairs({'missing-unit', '', 'pet', 'party99'}) do
            CheckDisplay('', unit, 104, -1, -2)
        end
        for _, id in ipairs({0, -1, -2147483648, 2147483647, 99999}) do
            CheckDisplay('', 'player', id)
        end
        CheckDisplay('', 'player', 201)
        CheckDisplay('', 'party1', 104)
        "#,
    )
    .expect("INFERRED miss policy; valid signed IDs need not be positive");
}

#[test]
fn required_unit_and_id_reject_missing_nil_and_wrong_actual_types() {
    let env = fixture_env();
    env.exec(
        r#"
        RejectDisplay()
        RejectDisplay('player')
        RejectDisplay(nil, 104)
        RejectDisplay('player', nil)
        for _, value in ipairs({false, true, 104, {}, function() end}) do
            RejectDisplay(value, 104)
        end
        for _, value in ipairs({'104', false, true, {}, function() end}) do
            RejectDisplay('player', value)
            RejectDisplay('missing-unit', value)
        end
        CheckDisplay('5', 'player', 104)
        "#,
    )
    .expect("INFERRED strict actual string/number representation");
}

#[test]
fn instance_id_validation_precedes_lookup_without_lossy_conversion() {
    let env = fixture_env();
    env.exec(
        r#"
        for _, unit in ipairs({'player', 'missing-unit'}) do
            for _, id in ipairs({104.5, -1.5, 0/0, math.huge, -math.huge,
                2147483648, -2147483649}) do
                RejectDisplay(unit, id)
            end
        end
        CheckDisplay('5', 'player', 104)
        "#,
    )
    .expect("INFERRED finite integral signed-i32 ID before record lookup");
}

#[test]
fn thresholds_reject_explicit_nil_min_and_nonfinite_or_nonnumber_values_before_miss() {
    let env = fixture_env();
    env.exec(
        r#"
        for _, unit in ipairs({'player', 'missing-unit'}) do
            RejectDisplay(unit, 104, nil)
            RejectDisplay(unit, 104, nil, 5)
            for _, value in ipairs({'2', false, true, {}, function() end,
                0/0, math.huge, -math.huge}) do
                RejectDisplay(unit, 104, value)
                RejectDisplay(unit, 104, 2, value)
            end
        end
        CheckDisplay('5', 'player', 104, 2, nil)
        "#,
    )
    .expect("nonnil min, nilable max and INFERRED actual finite threshold numbers");
}

#[test]
fn untainted_caller_accepts_authentic_secret_unit_and_id_each_and_mixed() {
    let env = fixture_env();
    install_host_secrets(&env);
    env.exec(
        r#"
        assert(debug.getstacktaint() == nil)
        CheckDisplay('5', DisplaySecretUnit, 104)
        CheckDisplay('5', 'player', DisplaySecretID)
        CheckDisplay('5', DisplaySecretUnit, DisplaySecretID, 2, 5)
        CheckDisplay('*', DisplaySecretParty, DisplaySecretPartyID, 2, 5)
        CheckDisplay('', DisplaySecretUnknown, DisplaySecretID)
        RejectDisplay(DisplaySecretID, 104)
        RejectDisplay('player', DisplaySecretUnit)
        RejectDisplay(DisplaySecretUnknown, DisplaySecretFractionID)
        assert(debug.getstacktaint() == nil)
        assert(issecretvalue(DisplaySecretUnit) and issecretvalue(DisplaySecretID))
        "#,
    )
    .expect("AllowedWhenUntainted plus authenticated representation before unknown lookup");
}

#[test]
fn neversecret_thresholds_reject_each_and_mixed_even_for_secure_unknown_queries() {
    let env = fixture_env();
    install_host_secrets(&env);
    env.exec(
        r#"
        assert(debug.getstacktaint() == nil)
        for _, unit in ipairs({'player', 'missing-unit', DisplaySecretUnit, DisplaySecretUnknown}) do
            RejectDisplay(unit, 104, DisplaySecretMin)
            RejectDisplay(unit, 104, 2, DisplaySecretMax)
            RejectDisplay(unit, DisplaySecretID, DisplaySecretMin, DisplaySecretMax)
        end
        assert(issecretvalue(DisplaySecretMin) and issecretvalue(DisplaySecretMax))
        assert(secretunwrap(DisplaySecretMin) == 2 and secretunwrap(DisplaySecretMax) == 5)
        assert(debug.getstacktaint() == nil)
        CheckDisplay('5', DisplaySecretUnit, DisplaySecretID, 2, 5)
        "#,
    )
    .expect("rows367/368 NeverSecret independently of secure caller and missing record");
}

#[test]
fn tainted_caller_denies_each_secret_before_lookup_and_preserves_public_recovery() {
    let env = fixture_env();
    install_host_secrets(&env);
    env.exec(
        r#"
        local function probe()
            local before = debug.getstacktaint()
            assert(before == 'DisplayCountProbe')
            RejectDisplay(DisplaySecretUnit, 104)
            RejectDisplay('player', DisplaySecretID)
            RejectDisplay(DisplaySecretUnit, DisplaySecretID)
            RejectDisplay(DisplaySecretUnknown, 104)
            RejectDisplay('missing-unit', DisplaySecretID)
            RejectDisplay('missing-unit', 104, DisplaySecretMin)
            RejectDisplay('missing-unit', 104, 2, DisplaySecretMax)
            RejectDisplay('player', DisplaySecretID, DisplaySecretMin, DisplaySecretMax)
            for _, value in ipairs({DisplaySecretUnit, DisplaySecretID,
                DisplaySecretMin, DisplaySecretMax, DisplaySecretUnknown}) do
                assert(issecretvalue(value) and not pcall(secretunwrap, value))
            end
            CheckDisplay('5', 'player', 104, 2, 5)
            CheckDisplay('', 'missing-unit', 104)
            assert(debug.getstacktaint() == before)
        end
        debug.setobjecttaint(probe, 'DisplayCountProbe')
        probe()
        assert(debug.getstacktaint() == nil)
        CheckDisplay('5', DisplaySecretUnit, DisplaySecretID)
        "#,
    )
    .expect("row369 tainted denial without clearing caller taint or input secrecy");
}

#[test]
fn rooted_secret_gc_identity_survives_secure_tainted_public_secure_sequence() {
    let env = fixture_env();
    install_host_secrets(&env);
    env.exec(
        r#"
        local unit, id, minimum, maximum = DisplaySecretUnit, DisplaySecretID,
            DisplaySecretMin, DisplaySecretMax
        collectgarbage('collect')
        collectgarbage('collect')
        assert(rawequal(unit, DisplaySecretUnit) and rawequal(id, DisplaySecretID))
        assert(rawequal(minimum, DisplaySecretMin) and rawequal(maximum, DisplaySecretMax))
        assert(secretunwrap(unit) == 'player' and secretunwrap(id) == 104)
        CheckDisplay('5', unit, id, 2, 5)
        local function probe()
            assert(debug.getstacktaint() == 'DisplayCountGCProbe')
            RejectDisplay(unit, 104)
            RejectDisplay('player', id)
            RejectDisplay(unit, id, minimum, maximum)
            CheckDisplay('5', 'player', 104)
            assert(debug.getstacktaint() == 'DisplayCountGCProbe')
            for _, value in ipairs({unit, id, minimum, maximum}) do
                assert(issecretvalue(value) and not pcall(secretunwrap, value))
            end
        end
        debug.setobjecttaint(probe, 'DisplayCountGCProbe')
        probe()
        assert(debug.getstacktaint() == nil)
        CheckDisplay('5', unit, id)
        RejectDisplay(unit, id, minimum, maximum)
        assert(issecretvalue(unit) and issecretvalue(id))
        assert(issecretvalue(minimum) and issecretvalue(maximum))
        "#,
    )
    .expect("actual rooted VM secrets retain identity and secrecy across GC and denial");
}

fn assert_records_unchanged(actual: &[AuraInfo], expected: &[AuraInfo]) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
        assert_eq!(
            (
                (
                    &actual.name,
                    actual.spell_id,
                    actual.icon,
                    actual.applications
                ),
                (actual.duration, actual.expiration_time, &actual.source_unit),
                (actual.is_helpful, actual.is_raid, actual.is_nameplate_only),
                (actual.is_stealable, actual.can_apply_aura),
                (
                    actual.is_from_player_or_player_pet,
                    &actual.dispel_type,
                    actual.aura_instance_id
                ),
            ),
            (
                (
                    &expected.name,
                    expected.spell_id,
                    expected.icon,
                    expected.applications
                ),
                (
                    expected.duration,
                    expected.expiration_time,
                    &expected.source_unit
                ),
                (
                    expected.is_helpful,
                    expected.is_raid,
                    expected.is_nameplate_only
                ),
                (expected.is_stealable, expected.can_apply_aura),
                (
                    expected.is_from_player_or_player_pet,
                    &expected.dispel_type,
                    expected.aura_instance_id
                ),
            ),
        );
    }
}

#[test]
fn blocked_lookup_leaves_dto_store_blocklist_and_provider_state_unchanged() {
    let env = fixture_env();
    let (player, party_buffs, party_debuffs) = {
        let state = env.state().borrow();
        (
            state.player.buffs.clone(),
            state.party_members[0].buffs.clone(),
            state.party_members[0].debuffs.clone(),
        )
    };
    env.exec(
        r#"
        C_UnitAuras.AddBlockedAura('player', 104)
        C_UnitAuras.AddBlockedAura('party1', 202)
        local dto = AuraUtil.GetAuraDataByAuraInstanceID('player', 104)
        assert(dto.applications == 5 and dto.charges == 5 and dto.stackCount == 5)
        dto.applications, dto.charges, dto.stackCount = 999, 999, 999
        CheckDisplay('5', 'player', 104)
        CheckDisplay('6', 'party1', 202)
        assert(dto.applications == 999 and dto.charges == 999 and dto.stackCount == 999)
        local fresh = AuraUtil.GetAuraDataByAuraInstanceID('player', 104)
        assert(fresh.applications == 5 and fresh.charges == 5 and fresh.stackCount == 5)
        C_UnitAuras.SwitchAuraDataProvider()
        assert(AuraUtil.GetAuraDataByAuraInstanceID('player', 104) == nil)
        CheckDisplay('*', 'player', 104, 2, 4)
        CheckDisplay('6', 'party1', 202)
        CheckDisplay('', 'missing-unit', 104)
        RejectDisplay('player', 104.5)
        assert(AuraUtil.GetAuraDataByAuraInstanceID('player', 104) == nil)
        C_UnitAuras.ResetAuraDataProvider()
        local ids = {}
        for index = 1, 4 do
            ids[index] = C_UnitAuras.GetAuraDataByIndex('player', index, 'HELPFUL').auraInstanceID
        end
        assert(ids[1] == 101 and ids[2] == 102 and ids[3] == 103 and ids[4] == 105)
        assert(C_UnitAuras.GetAuraDataByIndex('player', 5, 'HELPFUL') == nil)
        assert(C_UnitAuras.GetAuraDataByIndex('party1', 1, 'HARMFUL') == nil)
        assert(AuraUtil.GetAuraDataByAuraInstanceID('player', 104).applications == 5)
        "#,
    )
    .expect("blocked-inclusive typed lookup, DTO independence and read-only provider behavior");
    let state = env.state().borrow();
    assert_records_unchanged(&state.player.buffs, &player);
    assert_records_unchanged(&state.party_members[0].buffs, &party_buffs);
    assert_records_unchanged(&state.party_members[0].debuffs, &party_debuffs);
}

#[test]
fn environments_keep_independent_counts_and_missing_records() {
    let first = fixture_env();
    let second = fixture_env();
    {
        let mut state = second.state().borrow_mut();
        state.player.buffs[3].applications = 12;
        state.party_members[0].debuffs.clear();
    }
    first
        .exec("CheckDisplay('5', 'player', 104); CheckDisplay('6', 'party1', 202)")
        .expect("first environment retains fixture counts");
    second
        .exec("CheckDisplay('12', 'player', 104); CheckDisplay('', 'party1', 202)")
        .expect("second environment reads only its own records");
    first
        .exec("CheckDisplay('5', 'player', 104); CheckDisplay('6', 'party1', 202)")
        .expect("second queries cannot change first environment");
}
