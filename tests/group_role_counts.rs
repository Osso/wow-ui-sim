//! Retail PartyUtil consumes native role masks and class-token counts.
#![cfg(any(feature = "client-retail", feature = "client-ptr"))]

use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::lua_api::state::{GroupMemberCountsMember, GroupMemberCountsState};
use wow_ui_sim::screen::ScreenKind;

fn load_party_util(env: &WowLuaEnv) {
    let ui = wow_ui_sim::client_profile::blizzard_ui_addons_dir();
    let addons = discover_blizzard_addon_closure_for_screen(
        &ui,
        ScreenKind::Game,
        &["Blizzard_FrameXMLUtil"],
    );
    assert!(
        addons
            .iter()
            .any(|(name, _)| name == "Blizzard_FrameXMLUtil")
    );
    let source_path = ui.join("Blizzard_FrameXMLUtil/PartyUtil.lua");
    let source = std::fs::read_to_string(&source_path).expect("selected PartyUtil source");
    env.exec_named(&source, &format!("@{}", source_path.display()))
        .expect("load original PartyUtil");
}

fn member(
    flags: u32,
    role_mask_a: u32,
    role_mask_b: u32,
    resolved_class_id: Option<i32>,
    cached_class_id: i32,
) -> Option<GroupMemberCountsMember> {
    Some(GroupMemberCountsMember {
        flags,
        role_mask_a,
        role_mask_b,
        resolved_class_id,
        cached_class_id,
    })
}

#[test]
#[cfg(feature = "client-retail")]
fn group_member_counts_retail_uses_native_roster_fields() {
    let env = WowLuaEnv::new().expect("environment");
    load_party_util(&env);

    let container_one = vec![
        member(0, 2, 0, Some(1), 99),
        member(0, 12, 0, Some(2), 99),
        member(0, 8, 10, Some(3), 99),
        member(0, 0, 0, None, 4),
        member(16, 2, 0, Some(5), 99),
        member(0, 8, 2, None, 99),
        member(0, 2, 0, Some(0), 99),
        member(0, 8, 0, Some(99), 99),
        None,
        member(0, 2, 0, Some(2), 99),
    ];
    let container_two = (0..40)
        .map(|_| member(0, 2, 0, Some(1), 99))
        .chain([member(0, 8, 10, Some(2), 99)])
        .collect();
    env.state().borrow_mut().group_member_counts = GroupMemberCountsState {
        container_one: Some(container_one),
        container_two: Some(container_two),
    };

    env.exec(
        r#"
        assert(debug.getinfo(GetGroupMemberCountsForDisplay, 'S').source:find('PartyUtil.lua', 1, true))
        local first = GetGroupMemberCounts(1)
        assert(first.TANK == 3 and first.HEALER == 1 and first.DAMAGER == 1 and first.NOROLE == 1)
        assert(first.WARRIOR == 2 and first.PALADIN == 1 and first.HUNTER == 1 and first.ROGUE == 1)
        assert(first.UNKNOWN == nil, 'class zero has no published class key')
        local shown = GetGroupMemberCountsForDisplay()
        assert(shown.TANK == 3 and shown.HEALER == 1 and shown.DAMAGER == 2 and shown.NOROLE == 0)

        local second = GetGroupMemberCounts(2)
        assert(second.TANK == 40 and second.DAMAGER == 0 and second.WARRIOR == 40)
        local default = GetGroupMemberCounts()
        local invalid = GetGroupMemberCounts(7)
        local nonnumeric = GetGroupMemberCounts('not a number')
        assert(default.TANK == 40 and invalid.TANK == 40 and nonnumeric.TANK == 40)

        local supplied = { sentinel = 'preserved', TANK = -1 }
        local returned = GetGroupMemberCounts(supplied)
        assert(returned == supplied and returned.sentinel == 'preserved' and returned.TANK == 40)
        "#,
    )
    .expect("original PartyUtil consumes native role and class counts");

    env.state().borrow_mut().group_member_counts = GroupMemberCountsState::default();
    env.exec(
        r#"
        local empty = GetGroupMemberCounts()
        assert(empty.TANK == 0 and empty.HEALER == 0 and empty.DAMAGER == 0 and empty.NOROLE == 0)
        assert(empty.WARRIOR == 0 and empty.PALADIN == 0)
        "#,
    )
    .expect("no roster returns zero role and class counts");
}

#[test]
#[cfg(feature = "client-ptr")]
fn group_role_counts_ptr_preserves_existing_party_role_projection() {
    let env = WowLuaEnv::new().expect("environment");
    load_party_util(&env);
    {
        let mut state = env.state().borrow_mut();
        state.player.class_index = 1;
        state.party_members[0].class_index = 2;
        state.party_members[1].class_index = 99;
        state.party_members.truncate(2);
        state.party_group_active = true;
        state.party_leader_index = None;
    }
    env.exec(
        r#"
        assert(UnitSetRole('player', 'TANK'))
        assert(UnitSetRole('party1', 'HEALER'))
        local shown = GetGroupMemberCountsForDisplay()
        assert(shown.TANK == 1 and shown.HEALER == 1 and shown.DAMAGER == 1 and shown.NOROLE == 0)
        assert(shown.Warrior == 2 and shown.Paladin == 1 and shown.Hunter == 0)
        "#,
    )
    .expect("PTR keeps its existing simulator group projection");
}

#[cfg(any(feature = "client-retail", feature = "client-ptr"))]
use wow_ui_sim::loader::discover_blizzard_addon_closure_for_screen;
