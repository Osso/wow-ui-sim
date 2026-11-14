//! Public query shapes after all bootstrap passes; original-consumer tests cover events.
#![cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]
use wow_ui_sim::lua_api::WowLuaEnv;

#[test]
fn super_track_targets_round_trip_and_clear() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(C_SuperTrack.GetSuperTrackedQuestID() == nil)
        assert(select('#', C_SuperTrack.GetSuperTrackedContent()) == 0)
        assert(select('#', C_SuperTrack.GetSuperTrackedMapPin()) == 0)
        assert(C_SuperTrack.GetSuperTrackedVignette() == nil)
        assert(C_SuperTrack.GetHighestPrioritySuperTrackingType() == nil)
        assert(not C_SuperTrack.IsSuperTrackingAnything())
        C_SuperTrack.SetSuperTrackedQuestID(42)
        assert(C_SuperTrack.GetSuperTrackedQuestID() == 42)
        assert(C_SuperTrack.IsSuperTrackingAnything())
        C_SuperTrack.SetSuperTrackedQuestID(43)
        assert(C_SuperTrack.GetSuperTrackedQuestID() == 43)
        C_SuperTrack.SetSuperTrackedContent(0, 101)
        local kind, id = C_SuperTrack.GetSuperTrackedContent()
        assert(kind == 0 and id == 101)
        C_SuperTrack.ClearAllSuperTracked()
        assert(C_SuperTrack.GetSuperTrackedQuestID() == nil)
        assert(select('#', C_SuperTrack.GetSuperTrackedContent()) == 0)
        assert(not C_SuperTrack.IsSuperTrackingAnything())
    "#,
    )
    .unwrap();
}

#[test]
fn supplied_map_pin_and_vignette_are_observable_and_clearable() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.super_track.map_pin = Some((0, 201));
        state.super_track.vignette = Some("Vignette-fixture-1".into());
    }
    env.exec(
        r#"
        local kind, id = C_SuperTrack.GetSuperTrackedMapPin()
        assert(kind == 0 and id == 201)
        assert(C_SuperTrack.GetSuperTrackedVignette() == 'Vignette-fixture-1')
        assert(C_SuperTrack.IsSuperTrackingAnything())
        C_SuperTrack.ClearSuperTrackedMapPin()
        assert(select('#', C_SuperTrack.GetSuperTrackedMapPin()) == 0)
        assert(C_SuperTrack.IsSuperTrackingAnything())
        C_SuperTrack.ClearAllSuperTracked()
        assert(C_SuperTrack.GetSuperTrackedVignette() == nil)
        assert(not C_SuperTrack.IsSuperTrackingAnything())
    "#,
    )
    .unwrap();
}

#[cfg(feature = "client-retail")]
#[test]
fn retail_supertrack_flag_getters_are_published() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        assert(type(C_SuperTrack.IsSuperTrackingQuest) == "function")
        assert(type(C_SuperTrack.IsSuperTrackingContent) == "function")
        assert(type(C_SuperTrack.IsSuperTrackingMapPin) == "function")
        "#,
    )
    .unwrap();
}

#[test]
fn unsupplied_highlights_return_the_empty_set() {
    let env = WowLuaEnv::new().unwrap();
    env.exec(
        r#"
        for _, questID in ipairs({0, -1, 95774, 878, 899, 999999, 999999.9, '999999'}) do
            assert(select('#', C_QuestLog.GetQuestAdditionalHighlights(questID)) == 5)
            local map, world, elite, dungeons, treasures = C_QuestLog.GetQuestAdditionalHighlights(questID)
            assert(map == 0)
            assert(world == false and elite == false and dungeons == false and treasures == false)
        end
        for _, bad in ipairs({{}, true, 'not a number', 2147483648, -2147483649, math.huge}) do
            assert(not pcall(C_QuestLog.GetQuestAdditionalHighlights, bad))
        end
        assert(not pcall(C_QuestLog.GetQuestAdditionalHighlights))
    "#,
    )
    .unwrap();
}
