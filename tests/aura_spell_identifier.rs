//! Bounded Retail 12.0.5 lookup contract. Seeded alias reuse, unit polarity
//! ordering and conservative secret rejection are INFERRED, not native proof.
#![cfg(feature = "retail-12-0-5")]

use rilua::LuaApiMut;
use rilua::table_security::wrap_host_secret_number;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{AuraInfo, PartyMember};

fn fixture_aura(spell_id: i32, instance_id: i32, helpful: bool) -> AuraInfo {
    AuraInfo {
        name: format!("Fixture aura {instance_id}"),
        spell_id,
        icon: 134973,
        duration: 30.0,
        expiration_time: 45.0,
        applications: 3,
        source_unit: "party1".into(),
        is_helpful: helpful,
        is_raid: false,
        is_nameplate_only: false,
        is_stealable: false,
        can_apply_aura: true,
        is_from_player_or_player_pet: false,
        dispel_type: Some("Magic".into()),
        aura_instance_id: instance_id,
    }
}

fn fixture_party() -> PartyMember {
    PartyMember {
        name: "Aura lookup fixture member".into(),
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
        buffs: vec![fixture_aura(99011, 201, true)],
        debuffs: vec![fixture_aura(99012, 202, false)],
    }
}

fn seeded_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create aura lookup environment");
    {
        let mut state = env.state().borrow_mut();
        state.player.buffs = vec![
            fixture_aura(99001, 101, true),
            fixture_aura(99002, 102, false),
        ];
        state.party_members = vec![fixture_party()];
        state.spell_id_aliases.clear();
        state
            .spell_id_aliases
            .insert("fixture player label".into(), 99001);
        state
            .spell_id_aliases
            .insert("fixture party label".into(), 99012);
    }
    env
}

#[test]
fn numeric_player_lookup_preserves_current_dto_identity_and_fields() {
    let env = seeded_env();
    env.exec(
        r#"
        local aura, extra = C_UnitAuras.GetPlayerAuraBySpellID(99001)
        assert(extra == nil and type(aura) == 'table')
        assert(aura.spellId == 99001 and aura.auraInstanceID == 101)
        assert(aura.name == 'Fixture aura 101' and aura.icon == 134973)
        assert(aura.applications == 3 and aura.charges == 3 and aura.stackCount == 3)
        assert(aura.duration == 30 and aura.expirationTime == 45)
        assert(aura.sourceUnit == 'player' and aura.dispelName == 'Magic')
        assert(aura.isHelpful == true and aura.isHarmful == false)
        assert(aura.canApplyAura == true and type(aura.points) == 'table')
        assert(C_UnitAuras.GetPlayerAuraBySpellID(99999) == nil)
        "#,
    )
    .expect("current numeric C player lookup contract");
}

#[test]
fn player_lookup_remains_helpful_only_and_ignores_existing_block_list() {
    let env = seeded_env();
    env.exec(
        r#"
        C_UnitAuras.AddBlockedAura('player', 101)
        assert(C_UnitAuras.GetPlayerAuraBySpellID(99001).auraInstanceID == 101)
        assert(C_UnitAuras.GetPlayerAuraBySpellID(99002) == nil)
        assert(GetPlayerAuraBySpellID(99001).auraInstanceID == 101)
        assert(GetPlayerAuraBySpellID(99002) == nil)
        "#,
    )
    .expect("preserve helpful-only unblocked player compatibility, not native policy");
}

#[test]
fn legacy_player_global_retains_optional_numeric_input_behavior() {
    let env = seeded_env();
    env.state()
        .borrow_mut()
        .player
        .buffs
        .push(fixture_aura(0, 103, true));
    env.exec(
        r#"
        assert(GetPlayerAuraBySpellID().auraInstanceID == 103)
        assert(GetPlayerAuraBySpellID(nil).auraInstanceID == 103)
        assert(GetPlayerAuraBySpellID(99001).auraInstanceID == 101)
        assert(GetPlayerAuraBySpellID(99001.9).auraInstanceID == 101)
        "#,
    )
    .expect("legacy global remains numeric and nil-to-zero compatible");
}

#[test]
fn explicitly_seeded_aliases_reuse_shared_spell_resolution_case_normalization() {
    let env = seeded_env();
    env.exec(
        r#"
        -- INFERRED resolver reuse; labels are not native names, links or locales.
        local playerID = C_Spell.GetSpellIDForSpellIdentifier('FIXTURE Player Label')
        assert(playerID == 99001)
        local player = C_UnitAuras.GetPlayerAuraBySpellID('FIXTURE Player Label')
        assert(player.spellId == playerID and player.auraInstanceID == 101)
        assert(C_UnitAuras.GetUnitAuraBySpellID('player', 'fixture player label')
            .auraInstanceID == 101)
        local partyID = C_Spell.GetSpellIDForSpellIdentifier('Fixture PARTY Label')
        local party = C_UnitAuras.GetUnitAuraBySpellID('party1', 'Fixture PARTY Label')
        assert(partyID == 99012 and party.spellId == partyID)
        assert(party.auraInstanceID == 202 and party.isHarmful == true)
        "#,
    )
    .expect("explicit model alias reuse only");
}

#[test]
fn numeric_alias_precedence_matches_shared_resolver_without_changing_legacy() {
    let env = seeded_env();
    {
        let mut state = env.state().borrow_mut();
        state.player.buffs.push(fixture_aura(99003, 104, true));
        state.spell_id_aliases.insert("99001".into(), 99003);
    }
    env.exec(
        r#"
        assert(C_Spell.GetSpellIDForSpellIdentifier(99001) == 99003)
        assert(C_UnitAuras.GetPlayerAuraBySpellID(99001).auraInstanceID == 104)
        assert(C_UnitAuras.GetUnitAuraBySpellID('player', 99001).auraInstanceID == 104)
        assert(C_UnitAuras.GetPlayerAuraBySpellID('99001').auraInstanceID == 104)
        assert(GetPlayerAuraBySpellID(99001).auraInstanceID == 101)
        "#,
    )
    .expect("C lookups honor alias-first resolution; legacy still exact numeric");
}

#[test]
fn unit_lookup_reads_modeled_party_helpful_and_harmful_aura_rows() {
    let env = seeded_env();
    env.exec(
        r#"
        local helpful = C_UnitAuras.GetUnitAuraBySpellID('party1', 99011)
        local harmful = C_UnitAuras.GetUnitAuraBySpellID('party1', 99012)
        assert(helpful.spellId == 99011 and helpful.auraInstanceID == 201)
        assert(helpful.isHelpful == true and helpful.isHarmful == false)
        assert(harmful.spellId == 99012 and harmful.auraInstanceID == 202)
        assert(harmful.isHelpful == false and harmful.isHarmful == true)
        assert(harmful.sourceUnit == 'party1' and harmful.dispelName == 'Magic')
        assert(C_UnitAuras.GetUnitAuraBySpellID('player', 99002).auraInstanceID == 102)
        assert(C_UnitAuras.GetPlayerAuraBySpellID(99002) == nil)
        "#,
    )
    .expect("unit query supports both existing modeled polarities");
}

#[test]
fn inferred_unit_order_prefers_first_helpful_then_first_harmful_duplicate() {
    let env = seeded_env();
    {
        let mut state = env.state().borrow_mut();
        let party = &mut state.party_members[0];
        party.buffs = vec![
            fixture_aura(99021, 211, true),
            fixture_aura(99021, 212, true),
        ];
        party.debuffs = vec![
            fixture_aura(99021, 213, false),
            fixture_aura(99021, 214, false),
        ];
        // Harmful is stored first: new unit lookup still visits helpful first.
        state.player.buffs = vec![
            fixture_aura(99021, 111, false),
            fixture_aura(99021, 112, true),
        ];
    }
    env.exec(
        r#"
        assert(C_UnitAuras.GetUnitAuraBySpellID('party1', 99021).auraInstanceID == 211)
        assert(C_UnitAuras.GetUnitAuraBySpellID('player', 99021).auraInstanceID == 112)
        "#,
    )
    .expect("INFERRED helpful-first and stable first-match ordering");
    env.state().borrow_mut().party_members[0].buffs.remove(0);
    assert_eq!(
        env.eval::<i32>("return C_UnitAuras.GetUnitAuraBySpellID('party1', 99021).auraInstanceID")
            .unwrap(),
        212
    );
    env.state().borrow_mut().party_members[0].buffs.clear();
    assert_eq!(
        env.eval::<i32>("return C_UnitAuras.GetUnitAuraBySpellID('party1', 99021).auraInstanceID")
            .unwrap(),
        213
    );
}

#[test]
fn unit_target_query_uses_actual_builtin_harmful_fixture_not_invented_state() {
    let env = seeded_env();
    env.exec(
        r#"
        local aura = C_UnitAuras.GetUnitAuraBySpellID('target', 113746)
        assert(aura.spellId == 113746 and aura.auraInstanceID == 1)
        assert(aura.name == 'Weakened Armor' and aura.icon == 136039)
        assert(aura.sourceUnit == 'target' and aura.isHarmful == true)
        assert(aura.duration == 15 and aura.expirationTime == 15)
        assert(C_UnitAuras.GetUnitAuraBySpellID('target', 99001) == nil)
        assert(C_UnitAuras.GetUnitAuraBySpellID('player', 113746) == nil)
        "#,
    )
    .expect("reuse collector target fixture only; no target aura store");
}

#[test]
fn missing_units_and_unmatched_public_identifiers_do_not_fabricate_auras() {
    let env = seeded_env();
    env.exec(
        r#"
        for _, unit in ipairs({'missing-unit', 'party2', 'pet', 'raid1', ''}) do
            assert(C_UnitAuras.GetUnitAuraBySpellID(unit, 99001) == nil)
        end
        assert(C_UnitAuras.GetUnitAuraBySpellID(nil, 99001) == nil)
        for _, identifier in ipairs({99999, 'unseeded fixture label', ''}) do
            assert(C_UnitAuras.GetPlayerAuraBySpellID(identifier) == nil)
            assert(C_UnitAuras.GetUnitAuraBySpellID('player', identifier) == nil)
            assert(C_UnitAuras.GetUnitAuraBySpellID('party1', identifier) == nil)
        end
        assert(C_UnitAuras.GetUnitAuraBySpellID('party1', 99001) == nil)
        assert(C_UnitAuras.GetUnitAuraBySpellID('player', 99011) == nil)
        "#,
    )
    .expect("bounded collector units and unknown public strings return nil");
}

#[test]
fn missing_and_invalid_identifiers_error_even_when_unit_has_no_aura_store() {
    let env = seeded_env();
    env.exec(
        r#"
        local function reject(fn, ...)
            local ok, err = pcall(fn, ...)
            assert(not ok, 'invalid identifier must error, not yield a DTO or nil')
            assert(type(err) == 'string' and #err > 0)
        end
        reject(C_UnitAuras.GetPlayerAuraBySpellID)
        reject(C_UnitAuras.GetPlayerAuraBySpellID, nil)
        reject(C_UnitAuras.GetUnitAuraBySpellID, 'player')
        reject(C_UnitAuras.GetUnitAuraBySpellID, 'missing-unit', nil)
        for _, identifier in ipairs({false, true, {}, function() end, 0/0, math.huge, -math.huge}) do
            reject(C_UnitAuras.GetPlayerAuraBySpellID, identifier)
            reject(C_UnitAuras.GetUnitAuraBySpellID, 'player', identifier)
            reject(C_UnitAuras.GetUnitAuraBySpellID, 'missing-unit', identifier)
        end
        assert(C_UnitAuras.GetPlayerAuraBySpellID(99001).auraInstanceID == 101)
        "#,
    )
    .expect("INFERRED strict representation validation before absent-unit short circuit");
}

#[test]
fn dto_snapshots_and_alias_state_are_environment_isolated_across_gc() {
    let first = seeded_env();
    let second = seeded_env();
    first
        .exec(
            r#"
        retainedAura = C_UnitAuras.GetUnitAuraBySpellID('party1', 'fixture party label')
        retainedAura.name = 'mutated Lua DTO'
        retainedAura.points[1] = 42
        collectgarbage('collect')
        local fresh = C_UnitAuras.GetUnitAuraBySpellID('party1', 99012)
        assert(fresh ~= retainedAura and fresh.name == 'Fixture aura 202')
        assert(fresh.points[1] == nil and fresh.auraInstanceID == 202)
        "#,
        )
        .expect("DTO mutation and full GC do not modify backing aura");
    {
        let mut state = first.state().borrow_mut();
        state.party_members[0].debuffs.clear();
        state
            .spell_id_aliases
            .insert("fixture player label".into(), 99999);
    }
    first
        .exec(
            r#"
        collectgarbage('collect')
        assert(retainedAura.spellId == 99012 and retainedAura.auraInstanceID == 202)
        assert(C_UnitAuras.GetUnitAuraBySpellID('party1', 99012) == nil)
        assert(C_UnitAuras.GetPlayerAuraBySpellID('fixture player label') == nil)
        "#,
        )
        .expect("removed backing rows disappear; returned DTO is a snapshot");
    second
        .exec(
            r#"
        assert(C_UnitAuras.GetUnitAuraBySpellID('party1', 99012).auraInstanceID == 202)
        assert(C_UnitAuras.GetPlayerAuraBySpellID('fixture player label').auraInstanceID == 101)
        "#,
        )
        .expect("one environment's aura and alias mutations do not leak");
}

#[test]
fn actual_secret_identifier_rejection_survives_gc_and_preserves_caller_taint() {
    let env = seeded_env();
    {
        let loader = env.loader_env();
        let mut lua = loader.rilua_mut();
        let secret = wrap_host_secret_number(lua.state_mut(), 99001.0);
        lua.state_mut().push(secret);
        let inserted = lua.set_global_val("SecretAuraIdentifier", secret);
        lua.state_mut().pop();
        inserted.expect("install actual host-secret spell identifier");
    }
    env.exec(
        r#"
        collectgarbage('collect')
        assert(issecretvalue(SecretAuraIdentifier))
        local function probe()
            local before = debug.getstacktaint()
            for _, call in ipairs({
                function() return C_UnitAuras.GetPlayerAuraBySpellID(SecretAuraIdentifier) end,
                function() return C_UnitAuras.GetUnitAuraBySpellID('player', SecretAuraIdentifier) end,
                function() return C_UnitAuras.GetUnitAuraBySpellID('missing-unit', SecretAuraIdentifier) end,
            }) do
                local ok, err = pcall(call)
                assert(not ok, 'conservative policy rejects secrets before DTO/nil result')
                assert(type(err) == 'string' and #err > 0)
                assert(debug.getstacktaint() == before, 'rejection preserves caller taint')
                assert(issecretvalue(SecretAuraIdentifier), 'input must remain secret')
            end
            local aura = C_UnitAuras.GetPlayerAuraBySpellID(99001)
            assert(aura.auraInstanceID == 101)
            assert(C_UnitAuras.GetUnitAuraBySpellID('player', 99001).auraInstanceID == 101)
            assert(debug.getstacktaint() == before, 'public success preserves caller taint')
        end
        assert(issecure())
        probe()
        assert(issecure())
        local function addon()
            assert(debug.getstacktaint() == 'AuraIdentifierFixture')
            probe()
            assert(debug.getstacktaint() == 'AuraIdentifierFixture')
        end
        debug.setobjecttaint(addon, 'AuraIdentifierFixture')
        addon()
        assert(issecure())
        "#,
    )
    .expect("INFERRED conservative actual-secret rejection in secure and tainted callers");
}
