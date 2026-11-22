#![cfg(feature = "client-retail")]

fn assert_groupfinder_cross_faction_consumer(env: &wow_ui_sim::lua_api::WowLuaEnv) {
    env.exec(
        r#"
        assert(type(RaidFinderFrameFindRaidButton_Update) == "function", "RaidFinder consumer is loaded")
        assert(type(LFDQueueFrameFindGroupButton_Update) == "function", "LFD consumer is loaded")
        A_Admin.SetPlayerRace(0)
        A_Admin.SetPartySize(2)
        A_Admin.SetPartyMember(1, "Alliance One", 1, 80)
        A_Admin.SetPartyMemberFaction(1, "Alliance")
        A_Admin.SetPartyMember(2, "Alliance Two", 1, 80)
        A_Admin.SetPartyMemberFaction(2, "Alliance")
        RaidFinderFrameFindRaidButton_Update()
        assert(type(CROSS_FACTION_RAID_DUNGEON_FINDER_ERROR) == "string", "cross-faction warning text is available")
        assert(RaidFinderFrameFindRaidButton.disabledTooltip ~= CROSS_FACTION_RAID_DUNGEON_FINDER_ERROR, "same-faction party must not show cross-faction warning")

        A_Admin.SetPartyMemberFaction(2, "Horde")
        RaidFinderFrameFindRaidButton_Update()
        assert(not RaidFinderFrameFindRaidButton:IsEnabled(), "mixed-faction party must disable RaidFinder queue button")
        assert(RaidFinderFrameFindRaidButton.disabledTooltip == CROSS_FACTION_RAID_DUNGEON_FINDER_ERROR, "mixed-faction warning must come from original RaidFinder consumer")
        assert(C_PartyInfo.IsCrossFactionParty(LE_PARTY_CATEGORY_HOME), "home category must report supplied mixed factions")
        assert(not C_PartyInfo.IsCrossFactionParty(LE_PARTY_CATEGORY_INSTANCE), "instance category has no supplied roster")
        "#,
    )
    .expect("original RaidFinder consumer must report mixed-faction roster state");
}

#[test]
fn retail_groupfinder_raidfinder_reports_cross_faction_party() {
    crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape(
        &["Blizzard_GroupFinder"],
        &[],
        |env, _loaded| {
            assert_groupfinder_cross_faction_consumer(env);
        },
    );
}

#[test]
fn retail_unitframe_pvp_consumer_reads_supplied_party_factions() {
    crate::common::blizzard_addon_harness::with_blizzard_addon_startup_shape(
        &["Blizzard_UnitFrame"],
        &[],
        |env, _loaded| {
            env.exec(
                r#"
                assert(type(UnitFrameUtil.GetUnitPvPIndicatorDisplayInfo) == "function", "Retail UnitFrame PvP consumer is loaded")
                A_Admin.SetPlayerRace(0)
                A_Admin.SetPartySize(6)
                for index = 1, 6 do
                    A_Admin.SetPartyMember(index, "Faction Member " .. index, 1, 80)
                end
                A_Admin.SetPartyMemberFaction(1, "Alliance")
                A_Admin.SetPartyMemberFaction(2, "Horde")

                UnitFrameUtil.GetUnitPvPIndicatorDisplayInfo("party1")
                UnitFrameUtil.GetUnitPvPIndicatorDisplayInfo("party2")
                local partyFaction = UnitFactionGroup("party1")
                local raidFaction = UnitFactionGroup("raid2")
                assert(partyFaction == "Alliance", "party faction must use supplied member state")
                assert(raidFaction == "Horde", "raid faction must use supplied member state")
                assert(C_PartyInfo.IsCrossFactionParty(LE_PARTY_CATEGORY_HOME), "party faction inputs must agree with cross-faction party state")
                "#,
            )
            .expect("original UnitFrame PvP consumer must read supplied party factions");
        },
    );
}
