//! PartyUtil consumes Retail native masks and the other profiles' simulator projection.
#![cfg(not(feature = "client-wrath"))]

use wow_ui_sim::loader::discover_blizzard_addon_closure_for_screen;
use wow_ui_sim::lua_api::WowLuaEnv;
#[cfg(feature = "client-retail")]
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
    let toc_path = wow_ui_sim::loader::find_toc_file(&ui.join("Blizzard_FrameXMLUtil"))
        .expect("FrameXMLUtil has a selected profile TOC");
    let toc = wow_ui_sim::toc::TocFile::from_file(&toc_path).expect("FrameXMLUtil TOC parses");
    assert!(toc.allows_screen(ScreenKind::Game));
    assert!(
        toc.files
            .iter()
            .any(|file| file.file_name().is_some_and(|name| name == "PartyUtil.lua"))
    );
    let source_path = ui.join("Blizzard_FrameXMLUtil/PartyUtil.lua");
    let source = std::fs::read_to_string(&source_path).expect("selected PartyUtil source");
    env.exec_named(&source, &format!("@{}", source_path.display()))
        .expect("load original PartyUtil");
}

#[cfg(feature = "client-retail")]
fn member(
    flags: u32,
    role_mask_a: u32,
    role_mask_b: u32,
    resolved_player_class_id: Option<i32>,
    cached_class_id: i32,
) -> Option<GroupMemberCountsMember> {
    Some(GroupMemberCountsMember {
        flags,
        role_mask_a,
        role_mask_b,
        resolved_player_class_id,
        cached_class_id,
    })
}

#[test]
#[cfg(feature = "client-retail")]
fn group_role_counts_retail_uses_native_roster_fields() {
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
        member(0, 2, 0, Some(-1), 99),
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
        local function pack(...) return { count = select('#', ...), ... } end
        local firstResult = pack(GetGroupMemberCounts(1))
        assert(firstResult.count == 1 and type(firstResult[1]) == 'table', 'one table return')
        local first = firstResult[1]
        assert(first.TANK == 3 and first.HEALER == 1 and first.DAMAGER == 1 and first.NOROLE == 1, 'container one role counts')
        assert(first.WARRIOR == 2 and first.PALADIN == 1 and first.HUNTER == 1 and first.ROGUE == 1 and first.PRIEST == 0, 'class tokens, skipped flag, and invalid cached class')
        assert(first.UNKNOWN == nil, 'class zero has no published class key')

        local numericString = GetGroupMemberCounts('1')
        assert(numericString.TANK == 3, 'numeric strings use the shared numeric predicate')
        local second = GetGroupMemberCounts(2)
        assert(second.TANK == 40 and second.DAMAGER == 0 and second.WARRIOR == 40, '40-member cap')
        local shown = GetGroupMemberCountsForDisplay()
        assert(shown.TANK == 40 and shown.HEALER == 0 and shown.DAMAGER == 0 and shown.NOROLE == 0, 'original PartyUtil default container')
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
fn group_role_counts_ptr_keeps_simulator_party_projection() {
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

#[test]
#[cfg(any(
    feature = "client-mists",
    feature = "client-era",
    feature = "client-anniversary",
    feature = "client-wowforever"
))]
fn group_role_counts_classic_profiles_keep_unset_roles() {
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
        local shown = GetGroupMemberCountsForDisplay()
        assert(shown.TANK == 0 and shown.HEALER == 0 and shown.DAMAGER == 3 and shown.NOROLE == 0)
        assert(shown.Warrior == 2 and shown.Paladin == 1 and shown.Hunter == 0)
        "#,
    )
    .expect("classic profiles preserve their unset-role projection");
}
