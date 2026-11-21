#![cfg(any(feature = "client-retail", feature = "client-wowforever"))]

mod common;

fn assert_groupfinder_cross_faction_consumer(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    env.exec(
        r#"
        assert(type(RaidFinderFrameFindRaidButton_Update) == "function")
        A_Admin.SetPlayerRace(1)
        A_Admin.SetPartySize(2)
        A_Admin.SetPartyMember(1, "Alliance One", 1, 80)
        A_Admin.SetPartyMemberFaction(1, "Alliance")
        A_Admin.SetPartyMember(2, "Alliance Two", 1, 80)
        A_Admin.SetPartyMemberFaction(2, "Alliance")
        RaidFinderFrameFindRaidButton_Update()
        assert(RaidFinderFrameFindRaidButton.disabledTooltip ~= CROSS_FACTION_RAID_DUNGEON_FINDER_ERROR)

        A_Admin.SetPartyMemberFaction(2, "Horde")
        RaidFinderFrameFindRaidButton_Update()
        assert(not RaidFinderFrameFindRaidButton:IsEnabled())
        assert(RaidFinderFrameFindRaidButton.disabledTooltip == CROSS_FACTION_RAID_DUNGEON_FINDER_ERROR)
        assert(C_PartyInfo.IsCrossFactionParty(LE_PARTY_CATEGORY_HOME))
        assert(not C_PartyInfo.IsCrossFactionParty(LE_PARTY_CATEGORY_INSTANCE))
        "#,
    )
    .expect("original RaidFinder consumer must report mixed-faction roster state");
}

#[cfg(feature = "client-retail")]
#[test]
fn retail_groupfinder_raidfinder_reports_cross_faction_party() {
    common::blizzard_addon_harness::with_blizzard_addon_startup_shape(
        &["Blizzard_GroupFinder"],
        &[],
        |env, loaded| {
            assert!(loaded.iter().any(|name| name == "Blizzard_GroupFinder"));
            assert_groupfinder_cross_faction_consumer(env);
        },
    );
}

#[cfg(feature = "client-wowforever")]
#[test]
fn wowforever_groupfinder_raidfinder_reports_cross_faction_party() {
    common::blizzard_addon_harness::with_blizzard_addon_startup_shape(
        &["Blizzard_GroupFinder"],
        &[],
        |env, loaded| {
            assert!(loaded.iter().any(|name| name == "Blizzard_GroupFinder"));
            assert_groupfinder_cross_faction_consumer(env);
        },
    );
}
