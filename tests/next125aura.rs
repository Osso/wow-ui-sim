//! Six Retail 12.0.5 index-getter argument deltas only. Representation/error
//! policies are inferred; native aura access and conditional outputs are excluded.
#![cfg(feature = "retail-12-0-5")]

use rilua::LuaApiMut;
use rilua::table_security::{wrap_host_secret_number, wrap_host_secret_string};
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::AuraInfo;

fn fixture_aura(id: i32, helpful: bool, from_player: bool) -> AuraInfo {
    AuraInfo {
        name: format!("Index fixture {id}"),
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

fn populate_existing_stores(env: &WowLuaEnv) {
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

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create index getter environment");
    populate_existing_stores(&env);
    install_assertions(&env);
    env
}

fn install_assertions(env: &WowLuaEnv) {
    env.exec(
        r#"
        IndexCases = {
            {name = 'GetAuraDataByIndex', filter = 'HELPFUL', player = 101, party = 201},
            {name = 'GetAuraDataByIndex', filter = 'HARMFUL', player = 103, party = 203},
            {name = 'GetBuffDataByIndex', filter = 'HELPFUL', player = 101, party = 201},
            {name = 'GetDebuffDataByIndex', filter = 'HARMFUL', player = 103, party = 203},
        }
        function AssertIndexResult(expected, ...)
            assert(select('#', ...) == 1, 'one nullable AuraData return')
            local aura = ...
            if expected == nil then
                assert(aura == nil, 'missing indexed aura')
            else
                assert(type(aura) == 'table' and aura.auraInstanceID == expected,
                    'stored indexed AuraData identity')
            end
        end
        function CheckIndex(case, unit, index, filter, expected)
            AssertIndexResult(expected, C_UnitAuras[case.name](unit, index, filter))
        end
        function RejectIndex(case, ...)
            local query = C_UnitAuras[case.name]
            assert(type(query) == 'function', 'real index getter must be registered')
            local ok, err = pcall(query, ...)
            assert(not ok and type(err) == 'string' and #err > 0, case.name)
        end
        function AssertIndexDTO(aura, id, helpful, fromPlayer, source)
            assert(aura.name == 'Index fixture '..id and aura.spellId == 99000 + id)
            assert(aura.icon == 134973 and aura.auraInstanceID == id)
            assert(aura.applications == 3 and aura.charges == 3 and aura.stackCount == 3)
            assert(aura.duration == 30 and aura.expirationTime == 45 and aura.timeMod == 1)
            assert(aura.sourceUnit == source and aura.dispelName == 'Magic')
            assert(aura.isHelpful == helpful and aura.isHarmful == not helpful)
            assert(aura.isFromPlayerOrPlayerPet == fromPlayer and aura.canApplyAura)
            assert(not aura.isStealable and not aura.isRaid and not aura.isNameplateOnly)
            assert(not aura.nameplateShowPersonal and not aura.nameplateShowAll)
            assert(not aura.isBossAura and type(aura.points) == 'table')
            assert(next(aura.points) == nil)
        end
        "#,
    )
    .expect("install behavioral assertions without overriding queries");
}

fn install_host_secrets(env: &WowLuaEnv) {
    let loader = env.loader_env();
    let mut lua = loader.rilua_mut();
    rilua::table_security::register_table_security(&mut lua).expect("VM security helpers");
    for (name, text) in [
        ("IndexSecretPlayer", "player"),
        ("IndexSecretParty", "party1"),
        ("IndexSecretUnknown", "missing-unit"),
        ("IndexSecretHelpful", "HELPFUL|PLAYER"),
        ("IndexSecretHarmful", "HARMFUL|PLAYER"),
    ] {
        let value = wrap_host_secret_string(lua.state_mut(), text);
        lua.state_mut().push(value);
        let inserted = lua.set_global_val(name, value);
        lua.state_mut().pop();
        inserted.expect("root authentic host-secret STRING");
    }
    let value = wrap_host_secret_number(lua.state_mut(), 1.0);
    lua.state_mut().push(value);
    let inserted = lua.set_global_val("IndexSecretNumber", value);
    lua.state_mut().pop();
    inserted.expect("root authentic host-secret NUMBER");
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

#[test]
fn player_stored_polarity_and_existing_dto_normalization() {
    let env = fixture_env();
    assert_eq!(env.state().borrow().player.buffs[0].source_unit, "pet");
    env.exec(
        r#"
        for _, case in ipairs(IndexCases) do
            local helpful = case.filter == 'HELPFUL'
            CheckIndex(case, 'player', 1, case.filter, case.player)
            CheckIndex(case, 'player', 2, case.filter, case.player + 1)
            AssertIndexDTO(C_UnitAuras[case.name]('player', 1, case.filter),
                case.player, helpful, true, 'player')
            AssertIndexDTO(C_UnitAuras[case.name]('player', 2, case.filter),
                case.player + 1, helpful, false, 'player')
        end
        "#,
    )
    .expect("existing player DTO normalizes source to player, not stored pet/party1");
}

#[test]
fn seeded_party_buffs_and_debuffs_preserve_dto_source_and_flags() {
    let env = fixture_env();
    env.exec(
        r#"
        for _, case in ipairs(IndexCases) do
            local helpful = case.filter == 'HELPFUL'
            CheckIndex(case, 'party1', 1, case.filter, case.party)
            CheckIndex(case, 'party1', 2, case.filter, case.party + 1)
            AssertIndexDTO(C_UnitAuras[case.name]('party1', 1, case.filter),
                case.party, helpful, true, 'pet')
            AssertIndexDTO(C_UnitAuras[case.name]('party1', 2, case.filter),
                case.party + 1, helpful, false, 'party1')
        end
        "#,
    )
    .expect("concrete existing party stores, both polarities and PLAYER-source flags");
}

#[test]
fn optional_filter_omission_and_nil_retain_default_polarity_and_one_return() {
    let env = fixture_env();
    env.exec(
        r#"
        for _, case in ipairs(IndexCases) do
            local expected = case.name == 'GetDebuffDataByIndex' and 103 or 101
            AssertIndexResult(expected, C_UnitAuras[case.name]('player', 1))
            AssertIndexResult(expected, C_UnitAuras[case.name]('player', 1, nil))
            AssertIndexResult(nil, C_UnitAuras[case.name]('player', 3))
            AssertIndexResult(nil, C_UnitAuras[case.name]('player', 3, nil))
        end
        "#,
    )
    .expect("documented nilable filter, current HELPFUL/default wrapper polarity");
}

#[test]
fn recognized_filter_combinations_do_not_redesign_index_selection() {
    let env = fixture_env();
    env.exec(
        r#"
        for _, case in ipairs(IndexCases) do
            for _, unit in ipairs({'player', 'party1'}) do
                local first = unit == 'player' and case.player or case.party
                CheckIndex(case, unit, 2, case.filter..'|PLAYER', first + 1)
                CheckIndex(case, unit, 2, 'PLAYER|'..case.filter, first + 1)
                CheckIndex(case, unit, 2, string.lower(case.filter)..'|player', first + 1)
            end
        end
        -- Existing index getters do not apply the instance-query PLAYER predicate.
        -- Wrappers force polarity even when the optional filter says otherwise.
        AssertIndexResult(101, C_UnitAuras.GetBuffDataByIndex('player', 1, 'HARMFUL'))
        AssertIndexResult(103, C_UnitAuras.GetDebuffDataByIndex('player', 1, 'HELPFUL'))
        AssertIndexResult(201, C_UnitAuras.GetBuffDataByIndex('party1', 1, 'HARMFUL|PLAYER'))
        AssertIndexResult(203, C_UnitAuras.GetDebuffDataByIndex('party1', 1, 'HELPFUL|PLAYER'))
        "#,
    )
    .expect("retain observed index predicates; no shared-helper PLAYER enhancement");
}

#[test]
fn blocked_records_are_excluded_before_one_based_index_selection() {
    let env = fixture_env();
    env.exec(
        r#"
        for _, case in ipairs(IndexCases) do
            C_UnitAuras.AddBlockedAura('player', case.player)
            C_UnitAuras.AddBlockedAura('party1', case.party)
        end
        for _, case in ipairs(IndexCases) do
            CheckIndex(case, 'player', 1, case.filter, case.player + 1)
            CheckIndex(case, 'party1', 1, case.filter, case.party + 1)
            CheckIndex(case, 'player', 2, case.filter, nil)
            CheckIndex(case, 'party1', 2, case.filter, nil)
        end
        "#,
    )
    .expect("existing block exclusion compacts indices, not instance-ID lookup semantics");
}

#[test]
fn valid_integer_nonpositive_past_end_and_unknown_queries_return_one_nil() {
    let env = fixture_env();
    env.exec(
        r#"
        for _, case in ipairs(IndexCases) do
            for _, unit in ipairs({'player', 'party1', 'missing-unit', 'party99', ''}) do
                for _, index in ipairs({-2147483648, -1, 0, 3, 2147483647}) do
                    CheckIndex(case, unit, index, case.filter, nil)
                end
            end
            CheckIndex(case, 'missing-unit', 1, case.filter, nil)
            CheckIndex(case, 'party99', 1, case.filter, nil)
            CheckIndex(case, '', 1, case.filter, nil)
            CheckIndex(case, 'player', 1, case.filter, case.player)
        end
        "#,
    )
    .expect("INFERRED signed-i32 representation, nonpositive is lookup miss not type error");
}

#[test]
fn required_unit_and_optional_filter_are_strict_strings_before_unknown_lookup() {
    let env = fixture_env();
    env.exec(
        r#"
        for _, case in ipairs(IndexCases) do
            RejectIndex(case)
            RejectIndex(case, nil, 1, case.filter)
            for _, value in ipairs({false, true, 0, 1, {}, function() end}) do
                RejectIndex(case, value, 1, case.filter)
                RejectIndex(case, 'player', 1, value)
                RejectIndex(case, 'missing-unit', 1, value)
            end
            CheckIndex(case, 'player', 1, case.filter, case.player)
        end
        "#,
    )
    .expect("INFERRED no coercion; optional filter nil differs from malformed representation");
}

#[test]
fn index_rejects_missing_nil_and_lossy_numeric_representations_before_lookup() {
    let env = fixture_env();
    env.exec(
        r#"
        for _, case in ipairs(IndexCases) do
            for _, unit in ipairs({'player', 'missing-unit'}) do
                RejectIndex(case, unit)
                RejectIndex(case, unit, nil, case.filter)
                for _, value in ipairs({'1', false, true, {}, function() end,
                    1.5, -1.5, 0/0, math.huge, -math.huge, 2147483648, -2147483649}) do
                    RejectIndex(case, unit, value, case.filter)
                end
            end
            CheckIndex(case, 'player', 1, case.filter, case.player)
        end
        "#,
    )
    .expect("INFERRED finite integral signed-i32; validate before unknown-unit early return");
}

#[test]
fn secure_caller_accepts_real_secret_each_argument_and_combined_on_both_stores() {
    let env = fixture_env();
    install_host_secrets(&env);
    env.exec(
        r#"
        local before = debug.getstacktaint()
        assert(before == nil)
        assert(issecretvalue(IndexSecretNumber) and secretunwrap(IndexSecretNumber) == 1)
        for _, case in ipairs(IndexCases) do
            local filter = case.filter == 'HELPFUL' and IndexSecretHelpful or IndexSecretHarmful
            assert(issecretvalue(filter) and secretunwrap(filter) == case.filter..'|PLAYER')
            for _, row in ipairs({{'player', IndexSecretPlayer, case.player},
                {'party1', IndexSecretParty, case.party}}) do
                local unit, secretUnit, expected = unpack(row)
                assert(issecretvalue(secretUnit) and secretunwrap(secretUnit) == unit)
                CheckIndex(case, secretUnit, 1, case.filter, expected)
                CheckIndex(case, unit, IndexSecretNumber, case.filter, expected)
                CheckIndex(case, unit, 1, filter, expected)
                CheckIndex(case, secretUnit, IndexSecretNumber, filter, expected)
                CheckIndex(case, secretUnit, IndexSecretNumber, nil, 
                    case.name == 'GetDebuffDataByIndex' and expected or
                    (unit == 'player' and 101 or 201))
                assert(issecretvalue(secretUnit) and issecretvalue(filter))
                assert(issecretvalue(IndexSecretNumber) and debug.getstacktaint() == before)
            end
            CheckIndex(case, IndexSecretUnknown, IndexSecretNumber, filter, nil)
            assert(issecretvalue(IndexSecretUnknown))
        end
        assert(debug.getstacktaint() == before)
        "#,
    )
    .expect("AllowedWhenUntainted uses real host-secret inputs; inputs remain secret");
}

#[test]
fn tainted_denial_each_argument_and_combined_precedes_unknown_lookup_and_recovers() {
    let env = fixture_env();
    install_host_secrets(&env);
    env.exec(
        r#"
        local function probe()
            local before = debug.getstacktaint()
            assert(before == 'IndexGetterProbe')
            for _, case in ipairs(IndexCases) do
                local filter = case.filter == 'HELPFUL' and IndexSecretHelpful or IndexSecretHarmful
                for _, row in ipairs({{'player', IndexSecretPlayer, case.player},
                    {'party1', IndexSecretParty, case.party}}) do
                    local unit, secretUnit, expected = unpack(row)
                    RejectIndex(case, secretUnit, 1, case.filter)
                    RejectIndex(case, unit, IndexSecretNumber, case.filter)
                    RejectIndex(case, unit, 1, filter)
                    RejectIndex(case, secretUnit, IndexSecretNumber, filter)
                    assert(debug.getstacktaint() == before)
                    CheckIndex(case, unit, 1, case.filter, expected)
                    assert(debug.getstacktaint() == before, 'public recovery preserves taint')
                end
                RejectIndex(case, IndexSecretUnknown, 1, case.filter)
                RejectIndex(case, 'missing-unit', IndexSecretNumber, case.filter)
                RejectIndex(case, 'missing-unit', 1, filter)
                RejectIndex(case, IndexSecretUnknown, IndexSecretNumber, filter)
                CheckIndex(case, 'missing-unit', 1, case.filter, nil)
                assert(debug.getstacktaint() == before)
            end
            for _, value in ipairs({IndexSecretPlayer, IndexSecretParty, IndexSecretUnknown,
                IndexSecretNumber, IndexSecretHelpful, IndexSecretHarmful}) do
                assert(issecretvalue(value) and not pcall(secretunwrap, value))
                assert(debug.getstacktaint() == before)
            end
        end
        debug.setobjecttaint(probe, 'IndexGetterProbe')
        probe()
        assert(debug.getstacktaint() == nil)
        for _, case in ipairs(IndexCases) do
            local filter = case.filter == 'HELPFUL' and IndexSecretHelpful or IndexSecretHarmful
            CheckIndex(case, IndexSecretPlayer, IndexSecretNumber, filter, case.player)
        end
        "#,
    )
    .expect("denial authenticates all positions without clearing taint or declassifying inputs");
}

#[test]
fn gc_rooted_secrets_keep_identity_security_and_secure_tainted_secure_roundtrip() {
    let env = fixture_env();
    install_host_secrets(&env);
    env.exec(
        r#"
        local retained = {IndexSecretPlayer, IndexSecretParty, IndexSecretUnknown,
            IndexSecretNumber, IndexSecretHelpful, IndexSecretHarmful}
        collectgarbage('collect')
        collectgarbage('collect')
        local roots = {IndexSecretPlayer, IndexSecretParty, IndexSecretUnknown,
            IndexSecretNumber, IndexSecretHelpful, IndexSecretHarmful}
        for i, value in ipairs(retained) do
            assert(rawequal(value, roots[i]) and issecretvalue(value))
        end
        assert(secretunwrap(retained[1]) == 'player')
        assert(secretunwrap(retained[2]) == 'party1')
        assert(secretunwrap(retained[3]) == 'missing-unit')
        assert(secretunwrap(retained[4]) == 1)
        assert(secretunwrap(retained[5]) == 'HELPFUL|PLAYER')
        assert(secretunwrap(retained[6]) == 'HARMFUL|PLAYER')
        local function roundtrip()
            local taint = debug.getstacktaint()
            for _, case in ipairs(IndexCases) do
                local filter = case.filter == 'HELPFUL' and retained[5] or retained[6]
                if taint == nil then
                    CheckIndex(case, retained[1], retained[4], filter, case.player)
                    CheckIndex(case, retained[2], retained[4], filter, case.party)
                else
                    assert(taint == 'IndexGetterGCProbe')
                    RejectIndex(case, retained[1], 1, case.filter)
                    RejectIndex(case, 'party1', retained[4], case.filter)
                    RejectIndex(case, 'player', 1, filter)
                    RejectIndex(case, retained[2], retained[4], filter)
                    CheckIndex(case, 'player', 1, case.filter, case.player)
                    CheckIndex(case, 'party1', 1, case.filter, case.party)
                end
                assert(debug.getstacktaint() == taint)
            end
            for _, value in ipairs(retained) do
                assert(issecretvalue(value))
                if taint ~= nil then assert(not pcall(secretunwrap, value)) end
            end
        end
        roundtrip()
        local function tainted() roundtrip() end
        debug.setobjecttaint(tainted, 'IndexGetterGCProbe')
        tainted()
        assert(debug.getstacktaint() == nil)
        collectgarbage('collect')
        roundtrip()
        "#,
    )
    .expect("real STRING/NUMBER roots survive GC, denial and public recovery");
}

#[test]
fn successes_errors_and_dto_mutation_leave_all_store_block_and_provider_state_intact() {
    let env = fixture_env();
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
        for _, case in ipairs(IndexCases) do
            local filter = case.filter == 'HELPFUL' and IndexSecretHelpful or IndexSecretHarmful
            local playerID = case.player == 101 and 102 or 103
            local partyID = case.party == 203 and 204 or 201
            CheckIndex(case, IndexSecretPlayer, IndexSecretNumber, filter, playerID)
            CheckIndex(case, IndexSecretParty, IndexSecretNumber, filter, partyID)
            CheckIndex(case, 'missing-unit', 1, case.filter, nil)
            RejectIndex(case, 'missing-unit', 1.5, case.filter)
            local aura = C_UnitAuras[case.name]('player', 1, case.filter)
            aura.name, aura.duration, aura.sourceUnit = 'changed DTO', 999, 'changed-source'
            aura.points[1] = 99
            AssertIndexDTO(C_UnitAuras[case.name]('player', 1, case.filter),
                playerID, case.filter == 'HELPFUL', playerID == 103, 'player')
        end
        local function probe()
            assert(debug.getstacktaint() == 'IndexImmutableProbe')
            for _, case in ipairs(IndexCases) do
                RejectIndex(case, IndexSecretPlayer, IndexSecretNumber, IndexSecretHelpful)
                CheckIndex(case, 'party1', 1, case.filter,
                    case.party == 203 and 204 or 201)
            end
            assert(debug.getstacktaint() == 'IndexImmutableProbe')
        end
        debug.setobjecttaint(probe, 'IndexImmutableProbe')
        probe()
        assert(debug.getstacktaint() == nil)
        assert(rawequal(blocked, C_UnitAuras._blockedAuras))
        for key, value in pairs(snapshot) do assert(blocked[key] == value) end
        for key, value in pairs(blocked) do assert(snapshot[key] == value) end
        assert(C_UnitAuras._providerSwitched == true)
        C_UnitAuras.ResetAuraDataProvider()
        AssertIndexResult(102, C_UnitAuras.GetBuffDataByIndex('player', 1))
        AssertIndexResult(204, C_UnitAuras.GetDebuffDataByIndex('party1', 1))
        "#,
    )
    .expect("read-only success/miss/error/secret paths and independent returned DTO");
    let state = env.state().borrow();
    assert_aura_records(&state.player.buffs, &player);
    assert_aura_records(&state.party_members[0].buffs, &buffs);
    assert_aura_records(&state.party_members[0].debuffs, &debuffs);
}
