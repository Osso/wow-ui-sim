#![cfg(feature = "retail-12-0-5")]

use wow_ui_sim::lua_api::WowLuaEnv;

fn fixture_env() -> WowLuaEnv {
    let env = WowLuaEnv::new().expect("create roster environment");
    {
        let mut state = env.state().borrow_mut();
        assert!(state.party_members.len() >= 2, "two seeded members");
        state.party_members.truncate(2);
        state.party_group_active = true;
        state.player.name = "RosterPlayer".into();
        for member in &mut state.party_members {
            member.name_cached = true;
        }
        let member = &mut state.party_members[0];
        member.name = "CacheArrival".into();
        member.class_index = 2;
        member.level = 67;
        member.is_leader = true;
        member.connected = false;
        member.dead_since = Some(std::time::Instant::now());
        state.party_members[1].name = "Unaffected".into();
    }
    env.exec(
        r#"
        function AssertRoster(expectedName)
            assert(select('#', GetRaidRosterInfo(2)) == 12)
            local name, rank, subgroup, level, class, classFile, zone,
                online, dead, role, masterLooter, assignedRole = GetRaidRosterInfo(2)
            assert(name == expectedName and type(name) == 'string')
            assert(rank == 2 and subgroup == 1 and level == 67)
            assert(class == 'Paladin' and classFile == 'PALADIN')
            assert(zone == '' and online == true and dead == true)
            assert(role == nil and masterLooter == nil and assignedRole == 'NONE')
        end
        function AssertMissing(index)
            assert(select('#', GetRaidRosterInfo(index)) == 12)
            local values = { GetRaidRosterInfo(index) }
            for slot = 1, 12 do
                assert(values[slot] == nil)
            end
        end
        "#,
    )
    .expect("install tuple assertions");
    env
}

#[test]
fn cached_name_keeps_existing_roster_tuple() {
    let env = fixture_env();
    env.exec("AssertRoster('CacheArrival')")
        .expect("cached name and all eleven remaining returns");
}

#[test]
fn uncached_name_is_public_unknown_with_unchanged_tuple() {
    let env = fixture_env();
    env.state().borrow_mut().party_members[0].name_cached = false;
    env.exec(
        r#"
        assert(UNKNOWN == 'Unknown')
        AssertRoster(UNKNOWN)
        local name = GetRaidRosterInfo(2)
        assert(type(name) == 'string' and not issecretvalue(name))
        assert(name ~= 'CacheArrival')
        "#,
    )
    .expect("uncached existing member returns public localized Unknown");
}

#[test]
fn cache_arrival_is_live_and_does_not_change_other_members() {
    let env = fixture_env();
    env.state().borrow_mut().party_members[0].name_cached = false;
    env.exec(
        r#"
        AssertRoster(UNKNOWN)
        assert(GetRaidRosterInfo(1) == 'RosterPlayer')
        assert(GetRaidRosterInfo(3) == 'Unaffected')
        OtherRoster = { GetRaidRosterInfo(3) }
        "#,
    )
    .expect("only unresolved roster row changes");
    env.state().borrow_mut().party_members[0].name_cached = true;
    env.exec(
        r#"
        AssertRoster('CacheArrival')
        assert(GetRaidRosterInfo(1) == 'RosterPlayer')
        assert(select('#', GetRaidRosterInfo(3)) == 12)
        local other = { GetRaidRosterInfo(3) }
        for slot = 1, 12 do
            assert(other[slot] == OtherRoster[slot])
        end
        "#,
    )
    .expect("cache arrival updates live without rebuilding roster");
    env.state().borrow_mut().party_members[0].name_cached = false;
    env.exec("AssertRoster(UNKNOWN)")
        .expect("cache availability is read on every call");
}

#[test]
fn missing_indices_and_inactive_group_still_return_twelve_nils() {
    let env = fixture_env();
    env.state().borrow_mut().party_members[0].name_cached = false;
    env.exec("AssertMissing(0); AssertMissing(-1); AssertMissing(4); AssertMissing(999)")
        .expect("invalid indices are not unresolved members");
    env.state().borrow_mut().party_group_active = false;
    env.exec("AssertMissing(1); AssertMissing(2); AssertMissing(3)")
        .expect("inactive group retains existing absent-roster contract");
}
