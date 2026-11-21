#[cfg(feature = "client-retail")]
use crate::common;
#[cfg(feature = "client-retail")]
use wow_ui_sim::lua_api::WowLuaEnv;

#[cfg(feature = "client-retail")]
const SUPPLY_JOURNEY_FLAGS: &str = r#"
A_Admin.SetMajorFactionJourneyFlags(2710, true, false)
A_Admin.SetMajorFactionJourneyFlags(2696, true, true)
A_Admin.SetMajorFactionJourneyFlags(2704, false, false)
assert(C_MajorFactions.ShouldDisplayMajorFactionAsJourney(2710),
    "supplied journey display flag should be true")
assert(C_MajorFactions.ShouldDisplayMajorFactionAsJourney(2696),
    "second supplied journey display flag should be true")
assert(not C_MajorFactions.ShouldUseJourneyRewardTrack(2710),
    "first supplied faction should use the renown track")
assert(C_MajorFactions.ShouldUseJourneyRewardTrack(2696),
    "second supplied faction should use the Journey reward track")
"#;

#[cfg(feature = "client-retail")]
const ASSERT_JOURNEY_CARDS: &str = r#"
local journeys = EncounterJournal.JourneysFrame
journeys:Refresh()
local journeyIDs, renownIDs = {}, {}
for _, data in journeys.dataProvider:EnumerateEntireRange() do
    if data.factionID then
        local ids = data.isRenownJourney and renownIDs or journeyIDs
        ids[#ids + 1] = data.factionID
    end
end
table.sort(journeyIDs)
table.sort(renownIDs)
assert(#journeyIDs == 2 and journeyIDs[1] == 2696 and journeyIDs[2] == 2710,
    string.format("Journey cards must include only supplied Journey factions: count=%d ids=%s",
        #journeyIDs, table.concat(journeyIDs, ",")))
assert(#renownIDs == 2 and renownIDs[1] == 2699 and renownIDs[2] == 2704,
    string.format("non-Journey factions must remain renown cards: count=%d ids=%s",
        #renownIDs, table.concat(renownIDs, ",")))
"#;

#[cfg(feature = "client-retail")]
const ASSERT_REWARD_TRACKS: &str = r#"
local journeys = EncounterJournal.JourneysFrame
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
"#;

#[cfg(feature = "client-retail")]
#[test]
fn journeys_refresh_and_reward_track_follow_supplied_faction_flags() {
    let env = load_journeys_env();
    supply_journey_flags(&env);
    assert_journey_card_selection(&env);
    assert_reward_track_selection(&env);
}

#[cfg(feature = "client-retail")]
fn load_journeys_env() -> WowLuaEnv {
    let ui = common::blizzard_addon_harness::blizzard_ui_dir();
    let env = WowLuaEnv::new().expect("Failed to create Lua environment");
    env.set_screen_size(1024.0, 768.0);
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
    env
}

#[cfg(feature = "client-retail")]
fn supply_journey_flags(env: &WowLuaEnv) {
    env.exec(SUPPLY_JOURNEY_FLAGS)
        .expect("Journey policy inputs should be accepted");
}

#[cfg(feature = "client-retail")]
fn assert_journey_card_selection(env: &WowLuaEnv) {
    env.exec(ASSERT_JOURNEY_CARDS)
        .expect("Journey refresh should render supplied faction flags");
}

#[cfg(feature = "client-retail")]
fn assert_reward_track_selection(env: &WowLuaEnv) {
    env.exec(ASSERT_REWARD_TRACKS)
        .expect("Journey progress should select each supplied reward track");
}
