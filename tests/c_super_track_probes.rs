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
        state.super_track.map_pin_saved = (0, 201);
        state.super_track.map_pin_tracking = true;
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
fn retail_supertrack_flags_follow_verified_setter_updates() {
    let env = WowLuaEnv::new().unwrap();
    {
        let mut state = env.state().borrow_mut();
        state.super_track.quest_id = Some(50);
        state.super_track.quest_tracking = true;
        state.super_track.content = Some((1, 0));
        state.super_track.content_saved = (1, 0);
        state.super_track.content_tracking = true;
        state.super_track.map_pin = Some((2, 25));
        state.super_track.map_pin_saved = (2, 25);
        state.super_track.map_pin_tracking = true;
        state.super_track.vignette = Some("Vignette-fixture-1".into());
    }

    env.exec(
        r#"
        assert(C_SuperTrack.IsSuperTrackingQuest())
        assert(C_SuperTrack.IsSuperTrackingContent())
        assert(C_SuperTrack.IsSuperTrackingMapPin())

        C_SuperTrack.SetSuperTrackedQuestID(0)
        assert(not C_SuperTrack.IsSuperTrackingQuest())
        assert(C_SuperTrack.IsSuperTrackingContent())
        assert(C_SuperTrack.IsSuperTrackingMapPin())
        assert(C_SuperTrack.GetSuperTrackedQuestID() == nil)
        local contentType, contentID = C_SuperTrack.GetSuperTrackedContent()
        assert(contentType == 1 and contentID == 0)
        local pinType, pinID = C_SuperTrack.GetSuperTrackedMapPin()
        assert(pinType == 2 and pinID == 25)

        C_SuperTrack.SetSuperTrackedQuestID(51)
        assert(C_SuperTrack.IsSuperTrackingQuest())
        assert(not C_SuperTrack.IsSuperTrackingContent())
        assert(not C_SuperTrack.IsSuperTrackingMapPin())
        assert(select('#', C_SuperTrack.GetSuperTrackedContent()) == 0)
        assert(select('#', C_SuperTrack.GetSuperTrackedMapPin()) == 0)
        assert(C_SuperTrack.GetSuperTrackedVignette() == nil)

        C_SuperTrack.SetSuperTrackedContent(0, 0)
        assert(not C_SuperTrack.IsSuperTrackingQuest())
        assert(C_SuperTrack.IsSuperTrackingContent())
        assert(not C_SuperTrack.IsSuperTrackingMapPin())
        local zeroType, zeroID = C_SuperTrack.GetSuperTrackedContent()
        assert(zeroType == 0 and zeroID == 0)
        C_SuperTrack.ClearSuperTrackedContent()
        assert(not C_SuperTrack.IsSuperTrackingContent())
        assert(select('#', C_SuperTrack.GetSuperTrackedContent()) == 0)
        "#,
    )
    .unwrap();

    assert_eq!(
        env.state().borrow().super_track.content_saved,
        (0, 0),
        "clearing a content selection must retain its saved values"
    );

    {
        let mut state = env.state().borrow_mut();
        state.super_track.map_pin = Some((3, 33));
        state.super_track.map_pin_saved = (3, 33);
        state.super_track.map_pin_tracking = false;
    }
    env.exec("C_SuperTrack.ClearSuperTrackedMapPin()").unwrap();
    {
        let state = env.state().borrow();
        assert_eq!(state.super_track.map_pin, Some((3, 33)));
        assert_eq!(state.super_track.map_pin_saved, (3, 33));
    }

    env.state().borrow_mut().super_track.map_pin_tracking = true;
    env.exec(
        r#"
        local pinType, pinID = C_SuperTrack.GetSuperTrackedMapPin()
        assert(pinType == 3 and pinID == 33)
        C_SuperTrack.ClearSuperTrackedMapPin()
        assert(not C_SuperTrack.IsSuperTrackingMapPin())
        assert(select('#', C_SuperTrack.GetSuperTrackedMapPin()) == 0)
        "#,
    )
    .unwrap();
    env.exec(
        r#"
        C_SuperTrack.ClearAllSuperTracked()
        assert(not C_SuperTrack.IsSuperTrackingQuest())
        assert(not C_SuperTrack.IsSuperTrackingContent())
        assert(not C_SuperTrack.IsSuperTrackingMapPin())
        assert(not C_SuperTrack.IsSuperTrackingCorpse())
        assert(not C_SuperTrack.IsSuperTrackingAnything())
        assert(select('#', C_SuperTrack.GetSuperTrackedContent()) == 0)
        assert(select('#', C_SuperTrack.GetSuperTrackedMapPin()) == 0)
        "#,
    )
    .unwrap();
    let state = env.state().borrow();
    assert_eq!(state.super_track.content_saved, (0, 0));
    assert_eq!(state.super_track.map_pin, None);
    assert_eq!(state.super_track.map_pin_saved, (3, 33));
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
