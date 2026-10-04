#![cfg(all(feature = "retail-12-0-5", any(feature = "profile-retail", feature = "client-ptr")))]

use wow_ui_sim::lua_api::WowLuaEnv;

fn roster_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("roster identity environment");
    {
        let mut sim = env.state().borrow_mut();
        assert!(sim.party_members.len() >= 2);
        sim.party_members.truncate(2);
        sim.party_group_active = true;
        sim.player.name = "RosterLocalIdentity".to_owned();
        let member = &mut sim.party_members[0];
        member.name = "RosterScopedIdentity".to_owned();
        member.name_cached = true;
        member.class_index = 2;
        member.level = 67;
        member.is_leader = true;
        member.dead_since = None;
        sim.party_members[1].name = "RosterPeerIdentity".to_owned();
        sim.party_members[1].name_cached = true;
    }
    env.exec(
        r#"
        OrdinaryRosterName = 'RosterScopedIdentity'
        function AssertRosterTail(...)
            assert(select('#', ...) == 12, 'exact roster tuple arity')
            local name, rank, subgroup, level, class, classFile, zone,
                online, dead, role, masterLooter, assignedRole = ...
            assert(rank == 2 and subgroup == 1 and level == 67)
            assert(class == 'Paladin' and classFile == 'PALADIN')
            assert(zone == '' and online == true and dead == false)
            assert(role == nil and masterLooter == nil and assignedRole == 'NONE')
            assert(canaccessallvalues(rank, subgroup, level, class, classFile, zone,
                online, dead, role, masterLooter, assignedRole))
        end
        "#,
    )
    .expect("retain equal ordinary literal before any roster call");
    env
}

fn classify_unit(env: &WowLuaEnv, unit: &str) -> String {
    let guid: String = env.eval(&format!("return UnitGUID('{unit}')")).unwrap();
    env.state()
        .borrow_mut()
        .identity_secret_guids
        .insert(guid.clone());
    guid
}

#[test]
fn roster_names_follow_public_state_and_instance_group_exemption() {
    let env = roster_env();
    for on_instanced_map in [false, true] {
        env.state().borrow_mut().instance_identity.on_instanced_map = on_instanced_map;
        env.exec(
            r#"
            local name = GetRaidRosterInfo(2)
            assert(not issecretvalue(name), 'unclassified roster group identity is public')
            assert(canaccessvalue(name) and name == 'RosterScopedIdentity')
            assert(name == UnitName('party1'))
            local player = GetRaidRosterInfo(1)
            assert(not issecretvalue(player), 'local player has instance exemption')
            assert(player == 'RosterLocalIdentity')
            AssertRosterTail(GetRaidRosterInfo(2))
            "#,
        )
        .expect("roster uses shared public/group/player exemption policy");
    }
}

#[test]
fn classified_roster_name_does_not_poison_equal_ordinary_strings() {
    let env = roster_env();
    classify_unit(&env, "party1");
    env.exec(
        r#"
        local name = GetRaidRosterInfo(2)
        assert(issecretvalue(name) and not canaccessvalue(name))
        assert(not issecretvalue(OrdinaryRosterName), 'retained equal literal stays public')
        assert(not issecretvalue('RosterScopedIdentity'), 'new equal literal stays public')
        assert(OrdinaryRosterName == 'RosterScopedIdentity')
        assert(issecretvalue(UnitName('party1')), 'same resolved GUID policy')
        local peer = GetRaidRosterInfo(3)
        assert(not issecretvalue(peer) and peer == 'RosterPeerIdentity')
        AssertRosterTail(GetRaidRosterInfo(2))
        "#,
    )
    .expect("trusted host wrapper restricts only the returned value");
}

#[test]
fn clearing_roster_classification_changes_new_results_not_retained_secret() {
    let env = roster_env();
    let guid = classify_unit(&env, "party1");
    env.exec(
        r#"
        RetainedRosterSecret = GetRaidRosterInfo(2)
        assert(issecretvalue(RetainedRosterSecret))
        "#,
    )
    .expect("retain classified roster output");
    env.state().borrow_mut().identity_secret_guids.remove(&guid);
    env.exec(
        r#"
        local name = GetRaidRosterInfo(2)
        assert(not issecretvalue(name), 'new result follows cleared host classification')
        assert(name == 'RosterScopedIdentity' and canaccessvalue(name))
        assert(issecretvalue(RetainedRosterSecret))
        assert(not canaccessvalue(RetainedRosterSecret))
        assert(not issecretvalue(UnitName('party1')))
        AssertRosterTail(GetRaidRosterInfo(2))
        "#,
    )
    .expect("new public result does not declassify retained output");
}

#[test]
fn explicit_player_guid_classification_overrides_roster_player_exemption() {
    let env = roster_env();
    env.exec(
        r#"
        OrdinaryPlayerName = 'RosterLocalIdentity'
        local name = GetRaidRosterInfo(1)
        assert(not issecretvalue(name), 'unclassified player roster name is public')
        assert(name == OrdinaryPlayerName)
        "#,
    )
    .expect("player roster identity starts public");
    let guid = classify_unit(&env, "player");
    env.exec(
        r#"
        local name = GetRaidRosterInfo(1)
        assert(select('#', GetRaidRosterInfo(1)) == 12)
        assert(issecretvalue(name) and not canaccessvalue(name))
        assert(issecretvalue(UnitName('player')))
        assert(not issecretvalue(OrdinaryPlayerName))
        "#,
    )
    .expect("explicit player classification matches other getters");
    env.state().borrow_mut().identity_secret_guids.remove(&guid);
    env.exec("assert(not issecretvalue(GetRaidRosterInfo(1)))")
        .expect("player roster output recovers after host clearing");
}

#[test]
fn uncached_classified_roster_name_keeps_public_unknown_and_live_tuple() {
    let env = roster_env();
    let guid = classify_unit(&env, "party1");
    env.state().borrow_mut().party_members[0].name_cached = false;
    env.exec(
        r#"
        local name = GetRaidRosterInfo(2)
        assert(name == UNKNOWN and not issecretvalue(name))
        AssertRosterTail(GetRaidRosterInfo(2))
        "#,
    )
    .expect("uncached classified identity uses existing public localized sentinel");
    env.state().borrow_mut().party_members[0].name_cached = true;
    env.exec(
        r#"
        RetainedCachedRoster = GetRaidRosterInfo(2)
        assert(issecretvalue(RetainedCachedRoster))
        AssertRosterTail(GetRaidRosterInfo(2))
        "#,
    )
    .expect("cache arrival applies current identity policy without rebuilding roster");
    env.state().borrow_mut().party_members[0].name_cached = false;
    env.exec(
        "local name = GetRaidRosterInfo(2); assert(name == UNKNOWN and not issecretvalue(name))",
    )
    .expect("uncached sentinel stays public after a classified cached read");
    {
        let mut sim = env.state().borrow_mut();
        sim.identity_secret_guids.remove(&guid);
        sim.party_members[0].name_cached = true;
    }
    env.exec(
        r#"
        local name = GetRaidRosterInfo(2)
        assert(not issecretvalue(name), 'cached new result follows public host state')
        assert(name == 'RosterScopedIdentity')
        assert(issecretvalue(RetainedCachedRoster))
        AssertRosterTail(GetRaidRosterInfo(2))
        "#,
    )
    .expect("cache and classification lifecycles stay independent");
}
