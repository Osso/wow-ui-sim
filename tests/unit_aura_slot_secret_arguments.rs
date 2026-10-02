//! Row376 only: slot argument security, with retained store/DTO behavior.
//! Strict representations are inferred; native access/output secrecy excluded.
#![cfg(feature = "retail-12-0-5")]

use rilua::LuaApiMut;
use rilua::table_security::{wrap_host_secret_number, wrap_host_secret_string};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::AuraInfo;

fn fixture_aura(id: i32, helpful: bool, from_player: bool) -> AuraInfo {
    AuraInfo {
        name: format!("Slot fixture {id}"),
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
        dispel_type: from_player.then(|| "Magic".into()),
        aura_instance_id: id,
    }
}

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create slot environment");
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
            .expect("seeded party roster");
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
        function AssertSlotResult(expected, ...)
            assert(select('#', ...) == 1, 'one nullable AuraData result')
            local aura = ...
            if expected == nil then
                assert(aura == nil, 'slot miss')
            else
                assert(type(aura) == 'table' and aura.auraInstanceID == expected,
                    'stored slot identity, not ordinal')
            end
        end
        function CheckSlot(unit, slot, expected)
            AssertSlotResult(expected, C_UnitAuras.GetAuraDataBySlot(unit, slot))
        end
        function RejectSlot(...)
            assert(type(C_UnitAuras.GetAuraDataBySlot) == 'function', 'registered query')
            local ok, err = pcall(C_UnitAuras.GetAuraDataBySlot, ...)
            assert(not ok and type(err) == 'string' and #err > 0, 'slot argument rejection')
        end
        function AssertSlotDTO(aura, id, helpful, fromPlayer, source)
            assert(aura.name == 'Slot fixture '..id and aura.spellId == 99000 + id)
            assert(aura.icon == 134973 and aura.auraInstanceID == id)
            assert(aura.applications == 3 and aura.charges == 3 and aura.stackCount == 3)
            assert(aura.duration == 30 and aura.expirationTime == 45 and aura.timeMod == 1)
            assert(aura.sourceUnit == source)
            if fromPlayer then assert(aura.dispelName == 'Magic')
            else assert(aura.dispelName == nil) end
            assert(aura.isHelpful == helpful and aura.isHarmful == not helpful)
            assert(aura.isFromPlayerOrPlayerPet == fromPlayer and aura.canApplyAura)
            assert(not aura.isStealable and not aura.isRaid and not aura.isNameplateOnly)
            assert(not aura.nameplateShowPersonal and not aura.nameplateShowAll)
            assert(not aura.isBossAura and type(aura.points) == 'table')
            assert(next(aura.points) == nil)
        end
        function CheckSlotBatch(unit, filter, first, helpful)
            local function batch(...)
                assert(select('#', ...) == 3, 'nil continuation plus two stored slots')
                local continuation, a, b = ...
                assert(continuation == nil and a == first and b == first + 1)
                for _, id in ipairs({a, b}) do
                    CheckSlot(unit, id, id)
                    local fromPlayer = id == first
                    local source = unit == 'player' and 'player' or
                        (fromPlayer and 'pet' or 'party1')
                    AssertSlotDTO(C_UnitAuras.GetAuraDataBySlot(unit, id),
                        id, helpful, fromPlayer, source)
                end
            end
            batch(C_UnitAuras.GetAuraSlots(unit, filter, 1))
        end
        "#,
    )
    .expect("install assertions without replacing API/vendor functions");
    env
}

fn install_host_secrets(env: &WowLuaEnv) {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).expect("VM security helpers");
    for (name, text) in [
        ("SlotSecretPlayer", "player"),
        ("SlotSecretParty", "party1"),
        ("SlotSecretUnknown", "missing-unit"),
    ] {
        let value = wrap_host_secret_string(lua.state_mut(), text);
        lua.state_mut().push(value);
        let inserted = lua.set_global_val(name, value);
        lua.state_mut().pop();
        inserted.expect("root authentic host-secret STRING");
    }
    for (name, number) in [
        ("SlotSecretPlayerID", 101.0),
        ("SlotSecretPlayerHarmfulID", 103.0),
        ("SlotSecretPartyID", 201.0),
        ("SlotSecretPartyHarmfulID", 203.0),
        ("SlotSecretMissingID", 99999.0),
    ] {
        let value = wrap_host_secret_number(lua.state_mut(), number);
        lua.state_mut().push(value);
        let inserted = lua.set_global_val(name, value);
        lua.state_mut().pop();
        inserted.expect("root authentic host-secret NUMBER");
    }
}

fn assert_records_unchanged(actual: &[AuraInfo], expected: &[AuraInfo]) {
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
                actual.aura_instance_id,
            ),
            (
                &expected.name,
                expected.spell_id,
                expected.icon,
                expected.duration,
                expected.expiration_time,
                expected.applications,
                &expected.source_unit,
                expected.aura_instance_id,
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
                &actual.dispel_type,
            ),
            (
                expected.is_helpful,
                expected.is_raid,
                expected.is_nameplate_only,
                expected.is_stealable,
                expected.can_apply_aura,
                expected.is_from_player_or_player_pet,
                &expected.dispel_type,
            ),
        );
    }
}

#[test]
fn player_enumerated_instance_slots_keep_full_dto_and_single_batch() {
    let env = fixture_env();
    assert_eq!(env.state().borrow().player.buffs[0].source_unit, "pet");
    env.exec(
        r#"
        CheckSlotBatch('player', 'HELPFUL', 101, true)
        CheckSlotBatch('player', 'HARMFUL', 103, false)
        CheckSlot('player', 1, nil)
        CheckSlot('player', 2, nil)
        "#,
    )
    .expect("actual enumeration, nonordinal IDs and retained player source normalization");
}

#[test]
fn party_enumerated_helpful_harmful_slots_keep_full_dto() {
    let env = fixture_env();
    env.exec(
        r#"
        CheckSlotBatch('party1', 'HELPFUL', 201, true)
        CheckSlotBatch('party1', 'HARMFUL', 203, false)
        CheckSlot('party1', 101, nil)
        CheckSlot('player', 201, nil)
        "#,
    )
    .expect("existing seeded party stores and retained source/flag/nilable dispel DTO");
}

#[test]
fn blocked_records_disappear_from_enumeration_but_remain_slot_retrievable() {
    let env = fixture_env();
    env.exec(
        r#"
        for _, row in ipairs({{'player', 101, 104}, {'party1', 201, 204}}) do
            local unit, helpful, harmful = unpack(row)
            C_UnitAuras.AddBlockedAura(unit, helpful)
            C_UnitAuras.AddBlockedAura(unit, harmful)
            local function one(expected, ...)
                assert(select('#', ...) == 2)
                local continuation, slot = ...
                assert(continuation == nil and slot == expected)
                CheckSlot(unit, slot, expected)
            end
            one(helpful + 1, C_UnitAuras.GetAuraSlots(unit, 'HELPFUL'))
            one(harmful - 1, C_UnitAuras.GetAuraSlots(unit, 'HARMFUL'))
            AssertSlotDTO(C_UnitAuras.GetAuraDataBySlot(unit, helpful),
                helpful, true, true, unit == 'player' and 'player' or 'pet')
            AssertSlotDTO(C_UnitAuras.GetAuraDataBySlot(unit, harmful),
                harmful, false, false, unit == 'player' and 'player' or 'party1')
        end
        "#,
    )
    .expect("retain blocked-inclusive instance lookup, not visible-index compaction");
}

#[test]
fn switched_aura_util_provider_does_not_disable_c_slot_lookup() {
    let env = fixture_env();
    env.exec(
        r#"
        C_UnitAuras.SwitchAuraDataProvider()
        for _, row in ipairs({{'player', 101, 103}, {'party1', 201, 203}}) do
            assert(AuraUtil.GetAuraDataByAuraInstanceID(row[1], row[2]) == nil)
            CheckSlot(row[1], row[2], row[2])
            CheckSlot(row[1], row[3], row[3])
        end
        assert(C_UnitAuras._providerSwitched == true)
        "#,
    )
    .expect("AuraUtil switch control only; C slot provider remains callable");
}

#[test]
fn valid_signed_i32_misses_return_exactly_one_nil_without_positive_cap() {
    let env = fixture_env();
    env.exec(
        r#"
        for _, unit in ipairs({'player', 'party1', 'missing-unit', '', 'party99'}) do
            for _, slot in ipairs({-2147483648, -1, 0, 1, 99999, 2147483647}) do
                CheckSlot(unit, slot, nil)
            end
        end
        CheckSlot('missing-unit', 101, nil)
        local function empty(...)
            assert(select('#', ...) == 1 and (...) == nil)
        end
        empty(C_UnitAuras.GetAuraSlots('missing-unit', 'HELPFUL'))
        CheckSlot('player', 101, 101)
        "#,
    )
    .expect("INFERRED valid signed integers miss; no invented positive slot ceiling");
}

#[test]
fn required_unit_is_actual_string_without_default_or_numeric_coercion() {
    let env = fixture_env();
    env.exec(
        r#"
        RejectSlot()
        RejectSlot(nil, 101)
        for _, unit in ipairs({false, true, 0, 1, {}, function() end,
            CreateFrame('Frame')}) do
            RejectSlot(unit, 101)
        end
        CheckSlot('player', 101, 101)
        "#,
    )
    .expect("INFERRED required actual STRING; unrelated userdata is a type error");
}

#[test]
fn required_slot_is_finite_integral_signed_i32_before_unknown_lookup() {
    let env = fixture_env();
    env.exec(
        r#"
        for _, unit in ipairs({'player', 'party1', 'missing-unit'}) do
            RejectSlot(unit)
            RejectSlot(unit, nil)
            for _, slot in ipairs({false, true, '101', {}, function() end,
                101.5, -1.5, 0/0, math.huge, -math.huge, 2147483648, -2147483649}) do
                RejectSlot(unit, slot)
            end
        end
        CheckSlot('player', 101, 101)
        CheckSlot('missing-unit', 101, nil)
        "#,
    )
    .expect("INFERRED strict numeric representation authenticated before absence");
}

#[test]
fn secure_authentic_secret_slot_numbers_use_actual_stored_ids() {
    let env = fixture_env();
    install_host_secrets(&env);
    env.exec(
        r#"
        local before = debug.getstacktaint()
        assert(before == nil)
        for _, row in ipairs({{'player', SlotSecretPlayerID, 101},
            {'player', SlotSecretPlayerHarmfulID, 103},
            {'party1', SlotSecretPartyID, 201},
            {'party1', SlotSecretPartyHarmfulID, 203}}) do
            assert(issecretvalue(row[2]) and secretunwrap(row[2]) == row[3])
            CheckSlot(row[1], row[2], row[3])
            assert(issecretvalue(row[2]) and debug.getstacktaint() == before)
        end
        CheckSlot('player', SlotSecretMissingID, nil)
        CheckSlot('missing-unit', SlotSecretPlayerID, nil)
        assert(issecretvalue(SlotSecretMissingID) and debug.getstacktaint() == before)
        "#,
    )
    .expect("AllowedWhenUntainted authenticates NUMBER slot without declassifying input");
}

#[test]
fn never_secret_unit_rejects_authentic_secret_strings_even_when_secure() {
    let env = fixture_env();
    install_host_secrets(&env);
    env.exec(
        r#"
        assert(debug.getstacktaint() == nil)
        for _, row in ipairs({{SlotSecretPlayer, 'player', 101, SlotSecretPlayerID},
            {SlotSecretParty, 'party1', 201, SlotSecretPartyID},
            {SlotSecretUnknown, 'missing-unit', 101, SlotSecretPlayerID}}) do
            assert(issecretvalue(row[1]) and secretunwrap(row[1]) == row[2])
            RejectSlot(row[1], row[3])
            RejectSlot(row[1], row[4])
            assert(issecretvalue(row[1]) and issecretvalue(row[4]))
            assert(debug.getstacktaint() == nil)
        end
        CheckSlot('player', SlotSecretPlayerID, 101)
        "#,
    )
    .expect("slot unit retains NeverSecret unlike indexed-row annotation removals");
}

#[test]
fn tainted_secret_denials_precede_lookup_and_public_recovery_preserves_taint() {
    let env = fixture_env();
    install_host_secrets(&env);
    env.exec(
        r#"
        local function probe()
            local before = debug.getstacktaint()
            assert(before == 'SlotProbe')
            for _, row in ipairs({{'player', SlotSecretPlayer, SlotSecretPlayerID, 101},
                {'party1', SlotSecretParty, SlotSecretPartyID, 201}}) do
                RejectSlot(row[1], row[3])
                RejectSlot(row[2], row[4])
                RejectSlot(row[2], row[3])
                CheckSlot(row[1], row[4], row[4])
                CheckSlot(row[1], 99999, nil)
                assert(debug.getstacktaint() == before)
            end
            RejectSlot('missing-unit', SlotSecretPlayerID)
            RejectSlot('player', SlotSecretMissingID)
            RejectSlot(SlotSecretUnknown, 101)
            RejectSlot(SlotSecretUnknown, SlotSecretMissingID)
            CheckSlot('missing-unit', 101, nil)
            for _, secret in ipairs({SlotSecretPlayer, SlotSecretParty, SlotSecretUnknown,
                SlotSecretPlayerID, SlotSecretPartyID, SlotSecretMissingID}) do
                assert(issecretvalue(secret) and not pcall(secretunwrap, secret))
                assert(debug.getstacktaint() == before)
            end
        end
        debug.setobjecttaint(probe, 'SlotProbe')
        probe()
        assert(debug.getstacktaint() == nil)
        CheckSlot('player', SlotSecretPlayerID, 101)
        "#,
    )
    .expect("same tainted closure rejects secrets and recovers publicly without taint reset");
}

#[test]
fn gc_rooted_secret_identity_survives_secure_tainted_public_secure_roundtrip() {
    let env = fixture_env();
    install_host_secrets(&env);
    env.exec(
        r#"
        local retained = {SlotSecretPlayer, SlotSecretParty, SlotSecretUnknown,
            SlotSecretPlayerID, SlotSecretPartyID, SlotSecretMissingID}
        local function assertRoots()
            local roots = {SlotSecretPlayer, SlotSecretParty, SlotSecretUnknown,
                SlotSecretPlayerID, SlotSecretPartyID, SlotSecretMissingID}
            for i, value in ipairs(retained) do
                assert(rawequal(value, roots[i]) and issecretvalue(value))
            end
        end
        collectgarbage('collect')
        collectgarbage('collect')
        assertRoots()
        local function roundtrip()
            local taint = debug.getstacktaint()
            RejectSlot(retained[1], 101)
            RejectSlot(retained[2], retained[5])
            RejectSlot(retained[3], retained[6])
            if taint == nil then
                CheckSlot('player', retained[4], 101)
                CheckSlot('party1', retained[5], 201)
                CheckSlot('missing-unit', retained[6], nil)
                assert(secretunwrap(retained[1]) == 'player')
                assert(secretunwrap(retained[2]) == 'party1')
                assert(secretunwrap(retained[3]) == 'missing-unit')
                assert(secretunwrap(retained[4]) == 101)
                assert(secretunwrap(retained[5]) == 201)
                assert(secretunwrap(retained[6]) == 99999)
            else
                assert(taint == 'SlotGCProbe')
                RejectSlot('player', retained[4])
                RejectSlot('party1', retained[5])
                RejectSlot('missing-unit', retained[6])
                for _, value in ipairs(retained) do
                    assert(not pcall(secretunwrap, value))
                end
            end
            CheckSlot('player', 101, 101)
            CheckSlot('party1', 203, 203)
            CheckSlot('missing-unit', 101, nil)
            assert(debug.getstacktaint() == taint)
            assertRoots()
        end
        roundtrip()
        local function tainted() roundtrip() end
        debug.setobjecttaint(tainted, 'SlotGCProbe')
        tainted()
        assert(debug.getstacktaint() == nil)
        collectgarbage('collect')
        roundtrip()
        "#,
    )
    .expect("rooted authentic STRING/NUMBER identity and unchanged security across GC");
}

#[test]
fn query_paths_preserve_records_dto_block_provider_and_environment_isolation() {
    let env = fixture_env();
    let other = fixture_env();
    install_host_secrets(&env);
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
        C_UnitAuras.AddBlockedAura('player', 101)
        C_UnitAuras.AddBlockedAura('party1', 203)
        C_UnitAuras.SwitchAuraDataProvider()
        local blocked = C_UnitAuras._blockedAuras
        local snapshot = {}
        for key, value in pairs(blocked) do snapshot[key] = value end
        CheckSlot('player', SlotSecretPlayerID, 101)
        CheckSlot('party1', SlotSecretPartyHarmfulID, 203)
        CheckSlot('missing-unit', 101, nil)
        RejectSlot('missing-unit', 101.5)
        RejectSlot(SlotSecretPlayer, 101)
        for _, row in ipairs({{'player', 101, true, true, 'player'},
            {'party1', 203, false, true, 'pet'}}) do
            local dto = C_UnitAuras.GetAuraDataBySlot(row[1], row[2])
            dto.name, dto.duration, dto.sourceUnit = 'changed DTO', 999, 'changed-source'
            dto.points[1] = 99
            local fresh = C_UnitAuras.GetAuraDataBySlot(row[1], row[2])
            assert(not rawequal(dto, fresh) and not rawequal(dto.points, fresh.points))
            AssertSlotDTO(fresh, row[2], row[3], row[4], row[5])
        end
        local function probe()
            assert(debug.getstacktaint() == 'SlotImmutableProbe')
            RejectSlot('player', SlotSecretPlayerID)
            RejectSlot(SlotSecretParty, SlotSecretPartyID)
            CheckSlot('player', 101, 101)
            CheckSlot('party1', 203, 203)
            assert(debug.getstacktaint() == 'SlotImmutableProbe')
        end
        debug.setobjecttaint(probe, 'SlotImmutableProbe')
        probe()
        assert(debug.getstacktaint() == nil)
        assert(rawequal(blocked, C_UnitAuras._blockedAuras))
        for key, value in pairs(snapshot) do assert(blocked[key] == value) end
        for key, value in pairs(blocked) do assert(snapshot[key] == value) end
        assert(C_UnitAuras._providerSwitched == true)
        "#,
    )
    .expect("all query outcomes preserve state; DTO tables are independent");
    {
        let state = env.state().borrow();
        assert_records_unchanged(&state.player.buffs, &player);
        assert_records_unchanged(&state.party_members[0].buffs, &buffs);
        assert_records_unchanged(&state.party_members[0].debuffs, &debuffs);
    }
    other
        .exec(
            r#"
        assert(next(C_UnitAuras._blockedAuras) == nil)
        assert(C_UnitAuras._providerSwitched == false)
        assert(SlotSecretPlayer == nil and SlotSecretPlayerID == nil)
        CheckSlotBatch('player', 'HELPFUL', 101, true)
        CheckSlotBatch('party1', 'HARMFUL', 203, false)
        assert(AuraUtil.GetAuraDataByAuraInstanceID('party1', 203).auraInstanceID == 203)
        "#,
        )
        .expect("block/provider/global security roots do not leak to another environment");
    let state = other.state().borrow();
    assert_records_unchanged(&state.player.buffs, &player);
    assert_records_unchanged(&state.party_members[0].buffs, &buffs);
    assert_records_unchanged(&state.party_members[0].debuffs, &debuffs);
}
