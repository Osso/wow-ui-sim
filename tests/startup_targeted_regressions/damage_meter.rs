//! Shared explicit DamageMeter snapshots; parent owns profile compilation/runtime proof.

use super::*;
use wow_ui_sim::c_api::c_damage_meter::*;

const SESSION_ID: i64 = 47;
const DAMAGE_DONE: i32 = 0;
const HEALING_DONE: i32 = 2;
const CURRENT: i32 = 1;
const OVERALL: i32 = 0;

fn source_key() -> DamageMeterSourceKey {
    DamageMeterSourceKey {
        source_guid: Some("Creature-0-1-2-3-901-0000000047".into()),
        source_creature_id: Some(901),
    }
}

fn aggregate_source() -> DamageMeterCombatSource {
    DamageMeterCombatSource {
        source_guid: source_key().source_guid,
        source_creature_id: Some(901),
        name: "Fixture source".into(),
        class_filename: "MAGE".into(),
        spec_icon_id: 135932,
        total_amount: 840.0,
        amount_per_second: 42.0,
        is_local_player: false,
        death_recap_id: 73,
        death_time_seconds: 18.0,
        classification: "elite".into(),
        source_display_type: 2,
        faction_group: Some("Horde".into()),
    }
}

fn spell_fixture() -> DamageMeterCombatSpell {
    DamageMeterCombatSpell {
        spell_id: 133,
        total_amount: 630.0,
        amount_per_second: 31.5,
        creature_name: "Fixture target".into(),
        overkill_amount: 7.0,
        is_avoidable: true,
        is_deadly: false,
        combat_spell_details: DamageMeterCombatSpellUnitDetails {
            unit_name: "Fixture target".into(),
            unit_class_filename: "WARRIOR".into(),
            classification: "normal".into(),
            is_pet: false,
            is_mob: true,
            amount: 630.0,
            spec_icon_id: 132355,
        },
    }
}

fn install_fixture(env: &WowLuaEnv) {
    let input = DamageMeterInput {
        available: true,
        failure_reason: String::new(),
        available_sessions: vec![DamageMeterAvailableCombatSession {
            session_id: SESSION_ID,
            name: "Fixture encounter".into(),
            duration_seconds: Some(20.0),
        }],
        session_ids_by_type: [(CURRENT, SESSION_ID), (OVERALL, SESSION_ID)].into(),
        sessions: [(
            (SESSION_ID, DAMAGE_DONE),
            DamageMeterCombatSession {
                combat_sources: vec![aggregate_source()],
                max_amount: 840.0,
                total_amount: 840.0,
                duration_seconds: Some(20.0),
            },
        )]
        .into(),
        source_details: [(
            (SESSION_ID, DAMAGE_DONE, source_key()),
            DamageMeterCombatSessionSource {
                combat_spells: vec![spell_fixture()],
                max_amount: 630.0,
                total_amount: 630.0,
            },
        )]
        .into(),
    };
    env.state().borrow_mut().damage_meter = input;
}

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("DamageMeter fixture environment");
    install_fixture(&env);
    env
}

// Inspect keys, not sensitive values. No combat, taint clearing, or secret unwrap.
const SHAPE_HELPERS: &str = r#"
    function assertKeys(row, expected)
        assert(type(row) == "table")
        local keys = {}
        for key in pairs(row) do keys[key] = true end
        for _, key in ipairs(expected) do
            assert(keys[key], "missing field: " .. key)
            keys[key] = nil
        end
        assert(next(keys) == nil, "unexpected field")
    end
    function assertEmptySession(row)
        assertKeys(row, {"combatSources", "maxAmount", "totalAmount"})
        assert(#row.combatSources == 0)
    end
    function assertEmptyDetails(row)
        assertKeys(row, {"combatSpells", "maxAmount", "totalAmount"})
        assert(#row.combatSpells == 0)
    end
"#;

fn assert_query(env: &WowLuaEnv, code: &str) {
    env.exec(SHAPE_HELPERS).expect("DamageMeter shape helpers");
    env.exec(code).expect("DamageMeter behavioral fixture");
}

#[test]
fn damage_meter_empty_input_has_no_fabricated_records() {
    let env = WowLuaEnv::new().expect("empty DamageMeter environment");
    assert_eq!(
        env.state().borrow().damage_meter,
        DamageMeterInput::default()
    );
    assert_query(
        &env,
        r#"
        local available, reason = C_DamageMeter.IsDamageMeterAvailable()
        assert(available == false and type(reason) == "string")
        assert(#C_DamageMeter.GetAvailableCombatSessions() == 0)
        assert(C_DamageMeter.GetCurrentCombatSessionID() == nil)
        assert(C_DamageMeter.GetSessionDurationSeconds(Enum.DamageMeterSessionType.Current) == nil)
        assertEmptySession(C_DamageMeter.GetCombatSessionFromID(0, Enum.DamageMeterType.DamageDone))
        assertEmptySession(C_DamageMeter.GetCombatSessionFromType(Enum.DamageMeterSessionType.Current, Enum.DamageMeterType.DamageDone))
        assertEmptyDetails(C_DamageMeter.GetCombatSessionSourceFromID(0, Enum.DamageMeterType.DamageDone, nil, nil))
        assertEmptyDetails(C_DamageMeter.GetCombatSessionSourceFromType(Enum.DamageMeterSessionType.Current, Enum.DamageMeterType.DamageDone, nil, nil))
    "#,
    );
}

#[test]
fn damage_meter_explicit_aggregate_and_detail_shapes_are_distinct() {
    let env = fixture_env();
    assert_query(
        &env,
        r#"
        local available, reason = C_DamageMeter.IsDamageMeterAvailable()
        assert(available and reason == "")
        local list = C_DamageMeter.GetAvailableCombatSessions()
        assert(#list == 1)
        assertKeys(list[1], {"sessionID", "name", "durationSeconds"})
        assert(list[1].sessionID == 47 and list[1].name == "Fixture encounter")
        assert(list[1].durationSeconds == 20)
        assert(C_DamageMeter.GetCurrentCombatSessionID() == 47)
        assert(C_DamageMeter.GetSessionDurationSeconds(Enum.DamageMeterSessionType.Current) == 20)
        local sessions = {
            C_DamageMeter.GetCombatSessionFromID(47, Enum.DamageMeterType.DamageDone),
            C_DamageMeter.GetCombatSessionFromType(Enum.DamageMeterSessionType.Current, Enum.DamageMeterType.DamageDone),
            C_DamageMeter.GetCombatSessionFromType(Enum.DamageMeterSessionType.Overall, Enum.DamageMeterType.DamageDone),
        }
        assert(#sessions == 3)
        for index = 1, 3 do
            local session = sessions[index]
            assertKeys(session, {"combatSources", "maxAmount", "totalAmount", "durationSeconds"})
            assert(#session.combatSources == 1)
            local source = session.combatSources[1]
            assertKeys(source, {"sourceGUID", "sourceCreatureID", "name", "classFilename", "specIconID", "totalAmount", "amountPerSecond", "isLocalPlayer", "deathRecapID", "deathTimeSeconds", "classification", "sourceDisplayType", "factionGroup"})
            -- Only cached NeverSecret values are compared.
            assert(source.classFilename == "MAGE" and source.specIconID == 135932)
            assert(source.isLocalPlayer == false and source.deathRecapID == 73)
            assert(source.classification == "elite")
        end
        local sourceDetails = {
            C_DamageMeter.GetCombatSessionSourceFromID(47, Enum.DamageMeterType.DamageDone, "Creature-0-1-2-3-901-0000000047", 901),
            C_DamageMeter.GetCombatSessionSourceFromType(Enum.DamageMeterSessionType.Current, Enum.DamageMeterType.DamageDone, "Creature-0-1-2-3-901-0000000047", 901),
        }
        assert(#sourceDetails == 2)
        for index = 1, 2 do
            local details = sourceDetails[index]
            assertKeys(details, {"combatSpells", "maxAmount", "totalAmount"})
            assert(#details.combatSpells == 1)
            assertKeys(details.combatSpells[1], {"spellID", "totalAmount", "amountPerSecond", "creatureName", "overkillAmount", "isAvoidable", "isDeadly", "combatSpellDetails"})
            local unit = details.combatSpells[1].combatSpellDetails
            assertKeys(unit, {"unitName", "unitClassFilename", "classification", "isPet", "isMob", "amount", "specIconID"})
            assert(unit.unitClassFilename == "WARRIOR" and unit.classification == "normal")
            assert(unit.specIconID == 132355)
        end
    "#,
    );
}

#[test]
fn damage_meter_missing_selectors_do_not_alias_fixture_data() {
    let env = fixture_env();
    assert_query(
        &env,
        r#"
        for _, id in ipairs({0, 1, 999}) do
            assertEmptySession(C_DamageMeter.GetCombatSessionFromID(id, Enum.DamageMeterType.DamageDone))
            assertEmptyDetails(C_DamageMeter.GetCombatSessionSourceFromID(id, Enum.DamageMeterType.DamageDone, "Creature-0-1-2-3-901-0000000047", 901))
        end
        assertEmptySession(C_DamageMeter.GetCombatSessionFromType(Enum.DamageMeterSessionType.Expired, Enum.DamageMeterType.DamageDone))
        assert(C_DamageMeter.GetSessionDurationSeconds(Enum.DamageMeterSessionType.Expired) == nil)
        for _, meter in pairs(Enum.DamageMeterType) do
            if meter ~= Enum.DamageMeterType.DamageDone then
                assertEmptySession(C_DamageMeter.GetCombatSessionFromID(47, meter))
                assertEmptySession(C_DamageMeter.GetCombatSessionFromType(Enum.DamageMeterSessionType.Current, meter))
                assertEmptyDetails(C_DamageMeter.GetCombatSessionSourceFromID(47, meter, nil, nil))
                assertEmptyDetails(C_DamageMeter.GetCombatSessionSourceFromType(Enum.DamageMeterSessionType.Current, meter, nil, nil))
            end
        end
        assertEmptyDetails(C_DamageMeter.GetCombatSessionSourceFromID(47, Enum.DamageMeterType.DamageDone, "missing", nil))
        assertEmptyDetails(C_DamageMeter.GetCombatSessionSourceFromID(47, Enum.DamageMeterType.DamageDone, "Creature-0-1-2-3-901-0000000047", 902))
        assertEmptyDetails(C_DamageMeter.GetCombatSessionSourceFromID(47, Enum.DamageMeterType.DamageDone, nil, nil))
        assert(#C_DamageMeter.GetCombatSessionSourceFromID(47, Enum.DamageMeterType.DamageDone, "Creature-0-1-2-3-901-0000000047", nil).combatSpells == 1)
        assert(#C_DamageMeter.GetCombatSessionSourceFromID(47, Enum.DamageMeterType.DamageDone, nil, 901).combatSpells == 1)
    "#,
    );
    env.state()
        .borrow_mut()
        .damage_meter
        .session_ids_by_type
        .remove(&OVERALL);
    assert_query(
        &env,
        r#"
        assertEmptySession(C_DamageMeter.GetCombatSessionFromType(Enum.DamageMeterSessionType.Overall, Enum.DamageMeterType.DamageDone))
        assert(#C_DamageMeter.GetCombatSessionFromType(Enum.DamageMeterSessionType.Current, Enum.DamageMeterType.DamageDone).combatSources == 1)
    "#,
    );
}

#[test]
fn damage_meter_explicit_meter_input_and_optional_fields_are_preserved() {
    let env = fixture_env();
    let mut sparse = aggregate_source();
    sparse.source_guid = None;
    sparse.source_creature_id = None;
    sparse.faction_group = None;
    env.state().borrow_mut().damage_meter.sessions.insert(
        (SESSION_ID, HEALING_DONE),
        DamageMeterCombatSession {
            combat_sources: vec![sparse],
            max_amount: 840.0,
            total_amount: 840.0,
            duration_seconds: None,
        },
    );
    env.state().borrow_mut().damage_meter.available_sessions[0].duration_seconds = None;
    assert_query(
        &env,
        r#"
        assertKeys(C_DamageMeter.GetAvailableCombatSessions()[1], {"sessionID", "name"})
        local session = C_DamageMeter.GetCombatSessionFromID(47, Enum.DamageMeterType.HealingDone)
        assertKeys(session, {"combatSources", "maxAmount", "totalAmount"})
        assert(#session.combatSources == 1)
        assertKeys(session.combatSources[1], {"name", "classFilename", "specIconID", "totalAmount", "amountPerSecond", "isLocalPlayer", "deathRecapID", "deathTimeSeconds", "classification", "sourceDisplayType"})
        assertEmptyDetails(C_DamageMeter.GetCombatSessionSourceFromID(47, Enum.DamageMeterType.HealingDone, nil, nil))
    "#,
    );
}

#[test]
fn damage_meter_reset_clears_inputs_without_changing_availability() {
    let env = fixture_env();
    assert_query(
        &env,
        r#"
        assert(select('#', C_DamageMeter.ResetAllCombatSessions()) == 0)
        assert(#C_DamageMeter.GetAvailableCombatSessions() == 0)
        assert(C_DamageMeter.GetCurrentCombatSessionID() == nil)
        assert(C_DamageMeter.GetSessionDurationSeconds(Enum.DamageMeterSessionType.Current) == nil)
        assertEmptySession(C_DamageMeter.GetCombatSessionFromID(47, Enum.DamageMeterType.DamageDone))
        assertEmptySession(C_DamageMeter.GetCombatSessionFromType(Enum.DamageMeterSessionType.Current, Enum.DamageMeterType.DamageDone))
        assertEmptyDetails(C_DamageMeter.GetCombatSessionSourceFromID(47, Enum.DamageMeterType.DamageDone, "Creature-0-1-2-3-901-0000000047", 901))
        assertEmptyDetails(C_DamageMeter.GetCombatSessionSourceFromType(Enum.DamageMeterSessionType.Current, Enum.DamageMeterType.DamageDone, "Creature-0-1-2-3-901-0000000047", 901))
        C_DamageMeter.ResetAllCombatSessions()
        assert(C_DamageMeter.IsDamageMeterAvailable())
    "#,
    );
    let state = env.state();
    let state = state.borrow();
    assert!(state.damage_meter.available_sessions.is_empty());
    assert!(state.damage_meter.session_ids_by_type.is_empty());
    assert!(state.damage_meter.sessions.is_empty());
    assert!(state.damage_meter.source_details.is_empty());
    drop(state);
    install_fixture(&env);
    assert_query(
        &env,
        "assert(#C_DamageMeter.GetAvailableCombatSessions() == 1)",
    );
}

#[test]
fn damage_meter_queries_return_independent_nested_snapshots() {
    let env = fixture_env();
    assert_query(
        &env,
        r#"
        local list = C_DamageMeter.GetAvailableCombatSessions()
        list[1].name = "consumer edit"
        table.remove(list, 1)
        local session = C_DamageMeter.GetCombatSessionFromID(47, Enum.DamageMeterType.DamageDone)
        session.combatSources[1].classFilename = "ROGUE"
        session.combatSources[1].index = 99 -- cached BuildDataProvider decorates rows
        table.remove(session.combatSources, 1)
        local details = C_DamageMeter.GetCombatSessionSourceFromID(47, Enum.DamageMeterType.DamageDone, nil, 901)
        details.combatSpells[1].combatSpellDetails.unitClassFilename = "PRIEST"
        table.remove(details.combatSpells, 1)
        local fresh = C_DamageMeter.GetCombatSessionFromType(Enum.DamageMeterSessionType.Current, Enum.DamageMeterType.DamageDone)
        assert(#fresh.combatSources == 1)
        assert(fresh.combatSources[1].classFilename == "MAGE")
        assert(fresh.combatSources[1].index == nil)
        local freshDetails = C_DamageMeter.GetCombatSessionSourceFromType(Enum.DamageMeterSessionType.Current, Enum.DamageMeterType.DamageDone, nil, 901)
        assert(#freshDetails.combatSpells == 1)
        assert(freshDetails.combatSpells[1].combatSpellDetails.unitClassFilename == "WARRIOR")
        assert(C_DamageMeter.GetAvailableCombatSessions()[1].name == "Fixture encounter")
        C_DamageMeter.ResetAllCombatSessions()
        assert(#fresh.combatSources == 1 and #freshDetails.combatSpells == 1)
    "#,
    );
    let other = WowLuaEnv::new().expect("independent DamageMeter environment");
    assert_query(
        &other,
        "assert(#C_DamageMeter.GetAvailableCombatSessions() == 0)",
    );
}

#[test]
fn damage_meter_ambiguous_partial_selectors_return_empty_details() {
    let env = fixture_env();
    {
        let state = env.state();
        let mut state = state.borrow_mut();
        let input = &mut state.damage_meter;
        let details = input.source_details[&(SESSION_ID, DAMAGE_DONE, source_key())].clone();
        let same_creature = DamageMeterSourceKey {
            source_guid: Some("Creature-other".into()),
            source_creature_id: Some(901),
        };
        let same_guid = DamageMeterSourceKey {
            source_guid: source_key().source_guid,
            source_creature_id: Some(902),
        };
        input
            .source_details
            .insert((SESSION_ID, DAMAGE_DONE, same_creature), details.clone());
        input
            .source_details
            .insert((SESSION_ID, DAMAGE_DONE, same_guid), details);
    }
    assert_query(
        &env,
        r#"
        assertEmptyDetails(C_DamageMeter.GetCombatSessionSourceFromID(47, Enum.DamageMeterType.DamageDone, nil, 901))
        assertEmptyDetails(C_DamageMeter.GetCombatSessionSourceFromType(Enum.DamageMeterSessionType.Current, Enum.DamageMeterType.DamageDone, "Creature-0-1-2-3-901-0000000047", nil))
        local exact = C_DamageMeter.GetCombatSessionSourceFromID(47, Enum.DamageMeterType.DamageDone, "Creature-0-1-2-3-901-0000000047", 901)
        assert(#exact.combatSpells == 1)
        assert(#C_DamageMeter.GetAvailableCombatSessions() == 1)
    "#,
    );
}

#[test]
fn damage_meter_secret_selectors_reject_without_unwrapping() {
    let env = fixture_env();
    // Rejection is bounded simulator policy, not AllowedWhenUntainted parity.
    assert_query(
        &env,
        r#"
        local id = secretwrap(47)
        local kind = secretwrap(Enum.DamageMeterSessionType.Current)
        local meter = secretwrap(Enum.DamageMeterType.DamageDone)
        local guid = secretwrap("Creature-0-1-2-3-901-0000000047")
        local creature = secretwrap(901)
        assert(not pcall(C_DamageMeter.GetCombatSessionFromID, id, Enum.DamageMeterType.DamageDone))
        assert(not pcall(C_DamageMeter.GetCombatSessionFromType, kind, Enum.DamageMeterType.DamageDone))
        assert(not pcall(C_DamageMeter.GetCombatSessionFromID, 47, meter))
        assert(not pcall(C_DamageMeter.GetCombatSessionSourceFromID, 47, Enum.DamageMeterType.DamageDone, guid, nil))
        assert(not pcall(C_DamageMeter.GetCombatSessionSourceFromType, Enum.DamageMeterSessionType.Current, Enum.DamageMeterType.DamageDone, nil, creature))
        assert(not pcall(C_DamageMeter.GetSessionDurationSeconds, kind))
        assert(#C_DamageMeter.GetAvailableCombatSessions() == 1)
        assert(#C_DamageMeter.GetCombatSessionFromID(47, Enum.DamageMeterType.DamageDone).combatSources == 1)
    "#,
    );
}

#[test]
fn damage_meter_combat_publication_is_explicitly_blocked() {
    let env = fixture_env();
    env.state().borrow_mut().player.in_combat = true;
    assert_query(
        &env,
        r#"
        local queries = {
            function() return C_DamageMeter.GetCombatSessionFromID(47, Enum.DamageMeterType.DamageDone) end,
            function() return C_DamageMeter.GetCombatSessionFromType(Enum.DamageMeterSessionType.Current, Enum.DamageMeterType.DamageDone) end,
            function() return C_DamageMeter.GetCombatSessionSourceFromID(47, Enum.DamageMeterType.DamageDone, nil, 901) end,
            function() return C_DamageMeter.GetCombatSessionSourceFromType(Enum.DamageMeterSessionType.Current, Enum.DamageMeterType.DamageDone, nil, 901) end,
        }
        for _, query in ipairs(queries) do
            local ok, message = pcall(query)
            assert(not ok and string.find(message, "field secrecy is not modeled", 1, true))
        end
        assert(#C_DamageMeter.GetAvailableCombatSessions() == 1)
        C_DamageMeter.ResetAllCombatSessions()
        assert(not pcall(queries[1])) -- empty input cannot bypass the combat block
    "#,
    );
    env.state().borrow_mut().player.in_combat = false;
    assert_query(
        &env,
        "assertEmptySession(C_DamageMeter.GetCombatSessionFromID(47, Enum.DamageMeterType.DamageDone))",
    );
    install_fixture(&env);
    assert_query(
        &env,
        "assert(#C_DamageMeter.GetCombatSessionFromID(47, Enum.DamageMeterType.DamageDone).combatSources == 1)",
    );
}
