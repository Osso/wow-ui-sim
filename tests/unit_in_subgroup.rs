#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

use crate::common::blizzard_addon_harness::with_blizzard_addon_closure;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{SubgroupMemberRecord, SubgroupOwnerRoster};

#[cfg(feature = "client-retail")]
#[test]
fn unit_position_frame_uses_raid_appearance_without_subgroup_roster() {
    assert_unit_position_frame_uses_raid_appearance_without_subgroup_roster();
}

#[cfg(feature = "client-wowforever")]
#[test]
fn wowforever_unit_position_frame_uses_raid_appearance_without_subgroup_roster() {
    assert_unit_position_frame_uses_raid_appearance_without_subgroup_roster();
}

fn assert_unit_position_frame_uses_raid_appearance_without_subgroup_roster() {
    with_blizzard_addon_closure(&["Blizzard_UIPanels_Game"], &[], |env, loaded| {
        assert!(loaded.iter().any(|name| name == "Blizzard_UIPanels_Game"));
        prepare_party(&env, 1);
        env.exec(
            r#"
            assert(type(UnitInSubgroup) == "function")
            assert(UnitInSubgroup("player") == false)
            assert(select('#', UnitInSubgroup("player")) == 1)
            local frame = {}
            Mixin(frame, UnitPositionFrameMixin, UnitPositionFrameUpdateSecureMixin)
            frame.needsFullUpdate = true
            frame.capturedTexture = nil
            function frame:ClearUnits() end
            function frame:AddUnit(_unit, texture) self.capturedTexture = texture end
            function frame:FinalizeUnits() end
            frame:SetPinTexture("party", "party-subgroup")
            frame:SetPinTexture("raid", "other-subgroup")
            frame:UpdateFull(0)
            assert(frame.capturedTexture == "other-subgroup")
        "#,
        )
        .expect("original unit-position consumer must use raid appearance without an owner");
    });
}

#[test]
fn unit_position_frame_prefers_instance_owner_without_home_retry() {
    with_blizzard_addon_closure(&["Blizzard_UIPanels_Game"], &[], |env, _| {
        prepare_party(&env, 2);
        set_rosters(&env);
        env.exec(
            r#"
            local frame = {}
            Mixin(frame, UnitPositionFrameMixin, UnitPositionFrameUpdateSecureMixin)
            frame.needsFullUpdate = true
            frame.captured = {}
            function frame:ClearUnits() end
            function frame:AddUnit(unit, texture) self.captured[unit] = texture end
            function frame:FinalizeUnits() end
            frame:SetPinTexture("party", "party-subgroup")
            frame:SetPinTexture("raid", "other-subgroup")
            frame:UpdateFull(0)
            assert(frame.captured.party1 == "other-subgroup")
            assert(frame.captured.party2 == "party-subgroup")
        "#,
        )
        .expect("original consumer must select INSTANCE without retrying HOME");
    });
}

#[test]
fn unit_in_subgroup_selects_owner_and_converts_party_index() {
    with_api_test(|env| {
        prepare_party(env, 2);
        set_rosters(env);
        env.exec(
            r#"
            assert(UnitInSubgroup("party1", 1) == true)
            assert(UnitInSubgroup("party2", 1.9) == true)
            assert(UnitInSubgroup("party1", 2) == false)
            assert(UnitInSubgroup("party2", 2.9) == true)
            assert(UnitInSubgroup("party1", 0) == false)
            assert(UnitInSubgroup("party1", 3) == false)
            assert(pcall(function() UnitInSubgroup("party1", 4294967295) end))
            assert(not pcall(function() UnitInSubgroup("party1", 4294967296) end))
            assert(UnitInSubgroup("party1") == false)
            assert(UnitInSubgroup("party2") == true)
            assert(select('#', UnitInSubgroup("party1")) == 1)
            assert(UnitInSubgroup(1) == false)
            assert(not pcall(function() UnitInSubgroup({}, 1) end))
        "#,
        )
        .expect("owner selection, conversion, and error behavior follow R88bb");
    });
}

#[test]
fn unit_in_subgroup_skips_empty_slots() {
    with_api_test(|env| {
        prepare_party(env, 1);
        let first = roster(2, &[(1, 2)]).members.remove(0);
        env.state().borrow_mut().home_subgroup_roster = Some(SubgroupOwnerRoster {
            player_subgroup: 2,
            members: vec![
                SubgroupMemberRecord {
                    identity: None,
                    subgroup: 2,
                },
                first,
            ],
        });
        env.exec("assert(UnitInSubgroup('party1', 1) == true)")
            .expect("an empty slot does not hide a later subgroup member");
    });
}

#[test]
fn unit_in_subgroup_stops_after_four_members() {
    with_api_test(|env| {
        prepare_party(env, 1);
        let members = (1..=4)
            .map(|index| SubgroupMemberRecord {
                identity: Some(format!("Other-{index}")),
                subgroup: 2,
            })
            .chain([SubgroupMemberRecord {
                identity: Some("Player-0000-00000002".into()),
                subgroup: 2,
            }])
            .collect();
        env.state().borrow_mut().home_subgroup_roster = Some(SubgroupOwnerRoster {
            player_subgroup: 2,
            members,
        });
        env.exec("assert(UnitInSubgroup('party1', 1) == false)")
            .expect("a match after the first four subgroup slots is not checked");
    });
}

#[test]
fn unit_in_subgroup_uses_home_when_instance_owner_is_absent() {
    with_api_test(|env| {
        prepare_party(env, 1);
        env.state().borrow_mut().home_subgroup_roster = Some(roster(2, &[(1, 2)]));
        env.exec("assert(UnitInSubgroup('party1') == true)")
            .expect("omitted index falls back to HOME when INSTANCE has no owner");
    });
}

fn with_api_test(assertions: impl FnOnce(&WowLuaEnv)) {
    with_blizzard_addon_closure(&["Blizzard_UIPanels_Game"], &[], |env, _| assertions(env));
}

fn prepare_party(env: &WowLuaEnv, count: usize) {
    let mut state = env.state().borrow_mut();
    state.party_group_active = true;
    let first = state
        .party_members
        .first()
        .cloned()
        .expect("seeded party member");
    state.party_members.resize(count, first);
}

fn set_rosters(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    state.home_subgroup_roster = Some(roster(2, &[(1, 2), (2, 2)]));
    state.instance_subgroup_roster = Some(roster(3, &[(2, 3)]));
}

fn roster(player_subgroup: u32, members: &[(u32, u32)]) -> SubgroupOwnerRoster {
    SubgroupOwnerRoster {
        player_subgroup,
        members: members
            .iter()
            .map(|(index, subgroup)| SubgroupMemberRecord {
                identity: Some(format!("Player-0000-000000{:02}", index + 1)),
                subgroup: *subgroup,
            })
            .collect(),
    }
}
