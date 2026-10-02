//! Batch46, exact row382: GetAuraSlots argument authentication only.
//! Strict representations are INFERRED; retained batches are not native pagination.
#![cfg(feature = "retail-12-0-5")]

use rilua::LuaApiMut;
use rilua::table_security::{wrap_host_secret_number, wrap_host_secret_string};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::AuraInfo;

fn aura(id: i32, helpful: bool, from_player: bool) -> AuraInfo {
    AuraInfo {
        name: "Duplicate enumeration name".into(),
        spell_id: 88000 + id,
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
        dispel_type: from_player.then(|| "Magic".into()),
        aura_instance_id: id,
    }
}

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("enumeration environment");
    {
        let mut state = env.state().borrow_mut();
        state.player.buffs = vec![
            aura(307, true, true),
            aura(811, true, false),
            aura(419, false, true),
            aura(907, false, false),
        ];
        let party = state
            .party_members
            .first_mut()
            .expect("seeded party member");
        party.buffs = vec![aura(1207, true, true), aura(1801, true, false)];
        party.debuffs = vec![aura(1409, false, true), aura(1907, false, false)];
    }
    env.exec(r#"
        function AssertBatch(expected, ...)
            assert(select('#', ...) == #expected + 1, 'exact nil-plus-slot vararg arity')
            assert(select(1, ...) == nil, 'one batch, nil continuation')
            for i, id in ipairs(expected) do
                assert(select(i + 1, ...) == id, 'stored ID/order, not ordinal/name')
            end
        end
        function CheckBatch(expected, ...)
            AssertBatch(expected, C_UnitAuras.GetAuraSlots(...))
        end
        function CheckTerminated(...)
            assert(select('#', C_UnitAuras.GetAuraSlots(...)) == 0, 'no returns for supplied token')
        end
        function RejectEnumeration(...)
            local ok, err = pcall(C_UnitAuras.GetAuraSlots, ...)
            assert(not ok and type(err) == 'string' and #err > 0, 'invalid enumeration arguments')
            return err
        end
        function DenyEnumeration(...)
            local err = RejectEnumeration(...)
            assert(string.find(err, 'untainted caller', 1, true), 'caller guard before representation/lookup')
        end
        function AssertDTO(unit, id, helpful, fromPlayer)
            local dto = C_UnitAuras.GetAuraDataBySlot(unit, id)
            assert(type(dto) == 'table' and dto.auraInstanceID == id)
            assert(dto.name == 'Duplicate enumeration name' and dto.spellId == 88000 + id)
            assert(dto.icon == 134973 and dto.duration == 30 and dto.expirationTime == 45)
            assert(dto.applications == 3 and dto.charges == 3 and dto.stackCount == 3)
            assert(dto.timeMod == 1 and dto.isHelpful == helpful and dto.isHarmful == not helpful)
            assert(dto.sourceUnit == (unit == 'player' and 'player' or (fromPlayer and 'pet' or 'party1')))
            assert(dto.isFromPlayerOrPlayerPet == fromPlayer and dto.canApplyAura)
            if fromPlayer then assert(dto.dispelName == 'Magic') else assert(dto.dispelName == nil) end
            assert(not dto.isStealable and not dto.isRaid and not dto.isNameplateOnly)
            assert(not dto.nameplateShowPersonal and not dto.nameplateShowAll and not dto.isBossAura)
            assert(type(dto.points) == 'table' and next(dto.points) == nil)
            return dto
        end
        "#).expect("install assertions only, no API/vendor replacement");
    env
}

fn install_secrets(env: &WowLuaEnv) {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).expect("real VM security helpers");
    for (name, text) in [
        ("EnumPlayer", "player"),
        ("EnumParty", "party1"),
        ("EnumUnknown", "missing-unit"),
        ("EnumHelpful", "HELPFUL"),
        ("EnumHarmful", "HARMFUL"),
        ("EnumEmpty", ""),
    ] {
        let value = wrap_host_secret_string(lua.state_mut(), text);
        lua.state_mut().push(value);
        let inserted = lua.set_global_val(name, value);
        lua.state_mut().pop();
        inserted.expect("root real host-secret STRING before global insertion");
    }
    for (name, number) in [
        ("EnumMax", 1.0),
        ("EnumNegative", -2.5),
        ("EnumLarge", 1e100),
        ("EnumZero", 0.0),
        ("EnumNan", f64::NAN),
    ] {
        let value = wrap_host_secret_number(lua.state_mut(), number);
        lua.state_mut().push(value);
        let inserted = lua.set_global_val(name, value);
        lua.state_mut().pop();
        inserted.expect("root real host-secret NUMBER before global insertion");
    }
}

#[test]
fn player_batches_keep_nonordinal_duplicate_named_ids_and_dto_roundtrip() {
    let env = fixture_env();
    env.exec(
        r#"
        CheckBatch({307, 811}, 'player')
        CheckBatch({307, 811}, 'player', nil, nil, nil)
        CheckBatch({307, 811}, 'player', '')
        CheckBatch({419, 907}, 'player', 'HARMFUL', 1)
        AssertDTO('player', 307, true, true)
        AssertDTO('player', 811, true, false)
        AssertDTO('player', 419, false, true)
        AssertDTO('player', 907, false, false)
        assert(C_UnitAuras.GetAuraDataBySlot('player', 1) == nil)
        "#,
    )
    .expect("retained one-batch IDs and DTO contract");
}

#[test]
fn seeded_party_batches_retain_filter_substrings_without_player_or_raid_redesign() {
    let env = fixture_env();
    env.exec(r#"
        for _, filter in ipairs({'', 'helpful', 'HELPFUL|PLAYER', 'RAID', 'unknown-vocabulary'}) do
            CheckBatch({1207, 1801}, 'party1', filter, 1)
        end
        for _, filter in ipairs({'harmful', 'HARMFUL|PLAYER', 'prefixHARMFULsuffix'}) do
            CheckBatch({1409, 1907}, 'party1', filter)
        end
        for _, filter in ipairs({'MAW', 'maw|HARMFUL', 'EXTERNAL_DEFENSIVE', 'external_defensive|harmful'}) do
            CheckBatch({}, 'party1', filter)
            CheckBatch({}, 'player', filter)
        end
        AssertDTO('party1', 1207, true, true)
        AssertDTO('party1', 1801, true, false)
        AssertDTO('party1', 1409, false, true)
        AssertDTO('party1', 1907, false, false)
        "#).expect("actual collector ignores PLAYER/RAID, preserves uppercase substring precedence");
}

#[test]
fn blocked_ids_compact_visible_tuple_but_c_slot_getter_remains_blocked_inclusive() {
    let env = fixture_env();
    env.exec(
        r#"
        C_UnitAuras.AddBlockedAura('player', 307)
        C_UnitAuras.AddBlockedAura('party1', 1907)
        CheckBatch({811}, 'player', 'HELPFUL', 1)
        CheckBatch({1409}, 'party1', 'HARMFUL', 1)
        AssertDTO('player', 307, true, true)
        AssertDTO('party1', 1907, false, false)
        C_UnitAuras.AddBlockedAura('player', 811)
        CheckBatch({}, 'player', 'HELPFUL')
        AssertDTO('player', 811, true, false)
        "#,
    )
    .expect("visible block compaction is distinct from slot lookup");
}

#[test]
fn provider_switch_only_changes_aura_util_provider_not_c_enumeration() {
    let env = fixture_env();
    env.exec(
        r#"
        C_UnitAuras.SwitchAuraDataProvider()
        assert(C_UnitAuras._providerSwitched == true)
        assert(AuraUtil.GetAuraDataByAuraInstanceID('player', 307) == nil)
        assert(AuraUtil.GetAuraDataByAuraInstanceID('party1', 1409) == nil)
        CheckBatch({307, 811}, 'player', 'HELPFUL', 1)
        CheckBatch({1409, 1907}, 'party1', 'HARMFUL', 1)
        AssertDTO('player', 307, true, true)
        C_UnitAuras.ResetAuraDataProvider()
        assert(AuraUtil.GetAuraDataByAuraInstanceID('player', 307).auraInstanceID == 307)
        CheckBatch({307, 811}, 'player')
        "#,
    )
    .expect("real provider state control; no native AuraUtil consumer claim");
}

#[test]
fn inferred_required_utf8_unit_and_optional_actual_string_filter_validate_before_absence() {
    let env = fixture_env();
    env.exec(
        r#"
        RejectEnumeration()
        RejectEnumeration(nil)
        for _, value in ipairs({true, false, 0, 307, {}, function() end, string.char(255)}) do
            RejectEnumeration(value)
            RejectEnumeration('player', value)
            RejectEnumeration('missing-unit', value, 1, 0)
        end
        for _, unit in ipairs({'', 'missing-unit', 'party99', '未知'}) do
            CheckBatch({}, unit)
            CheckBatch({}, unit, nil, 1)
        end
        CheckBatch({307, 811}, 'player', nil)
        "#,
    )
    .expect("INFERRED strict strings; absent unit returns exactly one nil, not zero returns");
}

#[test]
fn inferred_optional_finite_f64_controls_retain_unused_max_and_any_token_termination() {
    let env = fixture_env();
    env.exec(r#"
        for _, number in ipairs({0, 1, -1, -2.5, 1.25, 2147483648, 1e100}) do
            CheckBatch({307, 811}, 'player', 'HELPFUL', number)
            CheckTerminated('player', 'HELPFUL', 1, number)
            CheckTerminated('missing-unit', nil, number, number)
        end
        for _, value in ipairs({'1', true, false, {}, function() end, 0/0, math.huge, -math.huge}) do
            RejectEnumeration('player', nil, value)
            RejectEnumeration('missing-unit', nil, value, 0)
            RejectEnumeration('player', nil, 1, value)
            RejectEnumeration('missing-unit', nil, nil, value)
        end
        CheckBatch({307, 811}, 'player', 'HELPFUL', 1, nil)
        "#).expect("INFERRED actual finite f64, no integral/positive cap; no new pagination");
}

#[test]
fn secure_authentic_secret_units_enumerate_actual_store_and_unknown_before_absence() {
    let env = fixture_env();
    install_secrets(&env);
    env.exec(
        r#"
        local before = debug.getstacktaint()
        assert(before == nil)
        CheckBatch({307, 811}, EnumPlayer, 'HELPFUL', 1)
        CheckBatch({1409, 1907}, EnumParty, 'HARMFUL', 1)
        CheckBatch({}, EnumUnknown)
        CheckBatch({}, EnumEmpty)
        for _, row in ipairs({{EnumPlayer, 'player'}, {EnumParty, 'party1'},
            {EnumUnknown, 'missing-unit'}, {EnumEmpty, ''}}) do
            assert(issecretvalue(row[1]) and secretunwrap(row[1]) == row[2])
        end
        assert(debug.getstacktaint() == before)
        "#,
    )
    .expect("row382 removes unit NeverSecret, not AllowedWhenUntainted");
}

#[test]
fn secure_optional_secret_filter_max_token_and_mixed_arguments_keep_payloads() {
    let env = fixture_env();
    install_secrets(&env);
    env.exec(
        r#"
        CheckBatch({419, 907}, 'player', EnumHarmful)
        CheckBatch({307, 811}, 'player', EnumEmpty, EnumMax)
        CheckBatch({307, 811}, 'player', nil, EnumNegative)
        CheckBatch({307, 811}, EnumPlayer, EnumHelpful, EnumLarge)
        CheckBatch({1409, 1907}, EnumParty, EnumHarmful, EnumMax, nil)
        CheckTerminated('player', nil, 1, EnumZero)
        CheckTerminated(EnumUnknown, EnumHelpful, EnumMax, EnumNegative)
        CheckTerminated(EnumPlayer, EnumHelpful, EnumMax, EnumLarge)
        RejectEnumeration('missing-unit', nil, EnumNan)
        RejectEnumeration('missing-unit', nil, nil, EnumNan)
        for _, row in ipairs({{EnumHelpful, 'HELPFUL'}, {EnumHarmful, 'HARMFUL'},
            {EnumMax, 1}, {EnumNegative, -2.5}, {EnumLarge, 1e100}, {EnumZero, 0}}) do
            assert(issecretvalue(row[1]) and secretunwrap(row[1]) == row[2])
        end
        assert(issecretvalue(EnumNan) and debug.getstacktaint() == nil)
        "#,
    )
    .expect("function policy applies to each supplied argument including ignored maxSlots");
}

#[test]
fn tainted_each_argument_and_mixed_secrets_deny_with_public_recovery_in_same_closure() {
    let env = fixture_env();
    install_secrets(&env);
    env.exec(
        r#"
        local function probe()
            local before = debug.getstacktaint()
            assert(before == 'EnumerationProbe')
            DenyEnumeration(EnumPlayer)
            DenyEnumeration(EnumUnknown)
            DenyEnumeration('player', EnumHelpful)
            DenyEnumeration('missing-unit', nil, EnumMax)
            DenyEnumeration('player', nil, nil, EnumZero)
            DenyEnumeration(EnumParty, EnumHarmful, EnumMax, EnumNegative)
            DenyEnumeration(EnumUnknown, EnumHelpful, EnumLarge, EnumZero)
            DenyEnumeration('player', nil, EnumNan)
            CheckBatch({307, 811}, 'player', 'HELPFUL', 1)
            CheckBatch({1409, 1907}, 'party1', 'HARMFUL', -2.5)
            CheckBatch({}, 'missing-unit')
            CheckTerminated('player', nil, 1, 0)
            for _, value in ipairs({EnumPlayer, EnumUnknown, EnumHelpful, EnumMax, EnumZero}) do
                assert(issecretvalue(value) and not pcall(secretunwrap, value))
            end
            assert(debug.getstacktaint() == before)
        end
        debug.setobjecttaint(probe, 'EnumerationProbe')
        probe()
        assert(debug.getstacktaint() == nil)
        CheckBatch({307, 811}, EnumPlayer, EnumHelpful, EnumMax)
        "#,
    )
    .expect("caller-taint policy before unknown lookup and token short circuit");
}

#[test]
fn all_four_authenticate_before_wrong_public_representation_or_token_short_circuit() {
    let env = fixture_env();
    install_secrets(&env);
    env.exec(
        r#"
        local function probe()
            assert(debug.getstacktaint() == 'EnumerationOrdering')
            DenyEnumeration(false, EnumHelpful)
            DenyEnumeration(false, nil, EnumMax)
            DenyEnumeration(false, nil, nil, EnumZero)
            DenyEnumeration('player', false, EnumMax)
            DenyEnumeration('missing-unit', false, nil, EnumZero)
            DenyEnumeration('player', nil, 'wrong-number', EnumZero)
            DenyEnumeration('missing-unit', nil, EnumMax, 0)
            DenyEnumeration('player', nil, nil, EnumNan)
            CheckBatch({307, 811}, 'player')
            assert(debug.getstacktaint() == 'EnumerationOrdering')
        end
        debug.setobjecttaint(probe, 'EnumerationOrdering')
        probe()
        RejectEnumeration(false, EnumHelpful)
        RejectEnumeration('player', nil, 'wrong-number', EnumZero)
        assert(debug.getstacktaint() == nil)
        "#,
    )
    .expect("mixed malformed public plus later secret must hit VM caller guard first");
}

#[test]
fn rooted_secret_identity_and_caller_taint_survive_forced_gc_roundtrips() {
    let env = fixture_env();
    install_secrets(&env);
    env.exec(
        r#"
        local retained = {EnumPlayer, EnumUnknown, EnumHarmful, EnumMax, EnumZero, EnumNegative}
        local function roots()
            local globals = {EnumPlayer, EnumUnknown, EnumHarmful, EnumMax, EnumZero, EnumNegative}
            for i, value in ipairs(retained) do
                assert(rawequal(value, globals[i]) and issecretvalue(value))
            end
        end
        local function roundtrip()
            local before = debug.getstacktaint()
            if before == nil then
                CheckBatch({419, 907}, retained[1], retained[3], retained[4])
                CheckBatch({}, retained[2], nil, retained[6])
                CheckTerminated(retained[1], retained[3], retained[4], retained[5])
                assert(secretunwrap(retained[1]) == 'player')
                assert(secretunwrap(retained[2]) == 'missing-unit')
                assert(secretunwrap(retained[3]) == 'HARMFUL')
                assert(secretunwrap(retained[4]) == 1 and secretunwrap(retained[5]) == 0)
                assert(secretunwrap(retained[6]) == -2.5)
            else
                assert(before == 'EnumerationGC')
                DenyEnumeration(retained[1], retained[3], retained[4])
                DenyEnumeration(retained[2])
                DenyEnumeration('player', nil, retained[4], 0)
                DenyEnumeration('player', nil, nil, retained[5])
                for _, value in ipairs(retained) do assert(not pcall(secretunwrap, value)) end
            end
            CheckBatch({307, 811}, 'player', nil, 1)
            collectgarbage('collect')
            roots()
            assert(debug.getstacktaint() == before)
        end
        collectgarbage('collect')
        collectgarbage('collect')
        roots()
        roundtrip()
        local function tainted() roundtrip() end
        debug.setobjecttaint(tainted, 'EnumerationGC')
        tainted()
        assert(debug.getstacktaint() == nil)
        collectgarbage('collect')
        roundtrip()
        "#,
    )
    .expect("real host STRING/NUMBER wrappers retain roots, secrecy and taint");
}

fn assert_records_unchanged(actual: &[AuraInfo], expected: &[AuraInfo]) {
    assert_eq!(actual.len(), expected.len());
    for (a, b) in actual.iter().zip(expected) {
        assert_eq!(
            (
                &a.name,
                a.spell_id,
                a.icon,
                a.duration,
                a.expiration_time,
                a.applications,
                &a.source_unit,
                a.aura_instance_id
            ),
            (
                &b.name,
                b.spell_id,
                b.icon,
                b.duration,
                b.expiration_time,
                b.applications,
                &b.source_unit,
                b.aura_instance_id
            )
        );
        assert_eq!(
            (
                a.is_helpful,
                a.is_raid,
                a.is_nameplate_only,
                a.is_stealable,
                a.can_apply_aura,
                a.is_from_player_or_player_pet,
                &a.dispel_type
            ),
            (
                b.is_helpful,
                b.is_raid,
                b.is_nameplate_only,
                b.is_stealable,
                b.can_apply_aura,
                b.is_from_player_or_player_pet,
                &b.dispel_type
            )
        );
    }
}

#[test]
fn enumeration_preserves_records_tuple_dto_block_provider_and_two_environment_isolation() {
    let env = fixture_env();
    let other = fixture_env();
    install_secrets(&env);
    let (player, buffs, debuffs) = {
        let state = env.state().borrow();
        (
            state.player.buffs.clone(),
            state.party_members[0].buffs.clone(),
            state.party_members[0].debuffs.clone(),
        )
    };
    env.exec(
        r#"
        C_UnitAuras.AddBlockedAura('player', 307)
        C_UnitAuras.AddBlockedAura('party1', 1907)
        C_UnitAuras.SwitchAuraDataProvider()
        local blocked = C_UnitAuras._blockedAuras
        local snapshot = {}
        for key, value in pairs(blocked) do snapshot[key] = value end
        local tuple = {C_UnitAuras.GetAuraSlots(EnumPlayer, EnumHelpful, EnumMax)}
        assert(tuple[1] == nil and tuple[2] == 811 and tuple[3] == nil)
        tuple[2] = 999
        CheckBatch({811}, EnumPlayer, EnumHelpful, EnumMax)
        CheckBatch({1409}, EnumParty, EnumHarmful)
        CheckBatch({}, EnumUnknown)
        CheckTerminated(EnumUnknown, nil, EnumMax, EnumZero)
        RejectEnumeration('missing-unit', nil, 'bad', 0)
        local dto = AssertDTO('player', 307, true, true)
        dto.name, dto.duration, dto.sourceUnit, dto.points[1] = 'changed', 999, 'changed', 99
        local fresh = AssertDTO('player', 307, true, true)
        assert(not rawequal(dto, fresh) and not rawequal(dto.points, fresh.points))
        local function probe()
            assert(debug.getstacktaint() == 'EnumerationImmutable')
            DenyEnumeration(EnumUnknown)
            DenyEnumeration('player', nil, EnumMax)
            CheckBatch({811}, 'player')
            CheckBatch({1409}, 'party1', 'HARMFUL')
            assert(debug.getstacktaint() == 'EnumerationImmutable')
        end
        debug.setobjecttaint(probe, 'EnumerationImmutable')
        probe()
        assert(debug.getstacktaint() == nil)
        assert(rawequal(blocked, C_UnitAuras._blockedAuras))
        for key, value in pairs(snapshot) do assert(blocked[key] == value) end
        for key, value in pairs(blocked) do assert(snapshot[key] == value) end
        assert(C_UnitAuras._providerSwitched == true)
        "#,
    )
    .expect("success/denial/miss/termination preserve all modeled state");
    {
        let state = env.state().borrow();
        assert_records_unchanged(&state.player.buffs, &player);
        assert_records_unchanged(&state.party_members[0].buffs, &buffs);
        assert_records_unchanged(&state.party_members[0].debuffs, &debuffs);
    }
    other
        .exec(
            r#"
        CheckBatch({307, 811}, 'player')
        CheckBatch({1409, 1907}, 'party1', 'HARMFUL')
        assert(next(C_UnitAuras._blockedAuras) == nil)
        assert(C_UnitAuras._providerSwitched == false and EnumPlayer == nil)
        AssertDTO('player', 307, true, true)
        "#,
        )
        .expect("block/provider/secrets/records isolated between environments");
}
