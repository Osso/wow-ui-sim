use crate::common;

#[cfg(any(feature = "client-retail", feature = "client-wowforever"))]
#[test]
fn journeys_refresh_and_reward_track_follow_supplied_faction_flags() {
    use wow_ui_sim::lua_api::WowLuaEnv;

    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
    let ui = common::blizzard_addon_harness::blizzard_ui_dir();
    common::panel_fixtures::seed_addon_search_paths(&env);
    common::panel_fixtures::load_panel_addons(&env);
    let loaded = common::blizzard_addon_harness::load_blizzard_addon_closure_into_env(
        &env,
        &ui,
        &["Blizzard_EncounterJournal"],
        &[],
    );
    assert!(
        loaded
            .iter()
            .any(|name| name == "Blizzard_EncounterJournal")
    );
    env.apply_post_load_workarounds();
    common::panel_fixtures::fire_startup_events(&env);

    env.exec(
        r#"
        A_Admin.SetMajorFactionJourneyFlags(2710, true, false)
        A_Admin.SetMajorFactionJourneyFlags(2696, true, true)
        A_Admin.SetMajorFactionJourneyFlags(2704, false, false)
        local journeys = EncounterJournal.JourneysFrame
        journeys:Refresh()
        local ids = {}
        for _, data in journeys.dataProvider:EnumerateEntireRange() do
            if data.factionID then ids[#ids + 1] = data.factionID end
        end
        table.sort(ids)
        assert(#ids == 2 and ids[1] == 2696 and ids[2] == 2710,
            string.format("Journeys list must render only supplied journey factions: count=%d ids=%s expansion=%s current=%s",
                #ids, table.concat(ids, ","), tostring(journeys.expansionFilter),
                tostring(LE_EXPANSION_LEVEL_CURRENT)))

        local progress = journeys.JourneyProgress
        progress.majorFactionData = C_MajorFactions.GetMajorFactionData(2710)
        progress:SetupRewardTrack()
        local renownTrack = progress.track
        progress.majorFactionData = C_MajorFactions.GetMajorFactionData(2696)
        progress:SetupRewardTrack()
        assert(renownTrack == progress.RenownTrackFrame,
            "first faction should use the renown track")
        assert(progress.track == progress.EncounterRewardProgressFrame,
            "second faction should use the journey reward track")
        assert(renownTrack ~= progress.track,
            "supplied factions must select different reward-track branches")
        "#,
    )
    .expect("Journey refresh should render supplied faction flags");
}
