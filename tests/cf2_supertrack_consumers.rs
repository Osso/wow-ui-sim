//! Original selected consumer; runtime RED/GREEN and mutant trials are slot obligations.
#![cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use sha2::{Digest, Sha256};
use wow_ui_sim::c_api::c_quest_log::QuestAdditionalHighlights;
use wow_ui_sim::lua_api::WowLuaEnv;

fn observe(env: &WowLuaEnv) {
    env.exec(
        r#"
        cf2_notifications = 0
        EventRegistry:RegisterCallback("Supertracking.OnChanged", function(_, manager)
            cf2_manager = manager
            cf2_notifications = cf2_notifications + 1
        end, {})
    "#,
    )
    .unwrap();
}

fn original_consumer() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let toc_path = wow_ui_sim::loader::find_toc_file(&ui.join("Blizzard_FrameXMLUtil")).unwrap();
    let toc = wow_ui_sim::toc::TocFile::from_file(&toc_path).unwrap();
    let consumer = std::path::Path::new("Mainline/Blizzard_QuestSuperTracking.lua");
    assert!(
        toc.files.iter().any(|file| file == consumer),
        "consumer must be runtime-selected"
    );
    let source = std::fs::read(ui.join("Blizzard_FrameXMLUtil").join(consumer)).unwrap();
    eprintln!(
        "CF2 selected {} consumer sha256={:x}",
        toc_path.display(),
        Sha256::digest(&source)
    );
    let env = new_blizzard_addon_env(&ui);
    load_blizzard_addon_closure_into_env(
        &env,
        &ui,
        &[
            "Blizzard_SharedXMLGame",
            "Blizzard_Colors",
            "Blizzard_StaticPopup",
        ],
        &[],
    );
    observe(&env);
    let loaded = load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_FrameXMLUtil"], &[]);
    assert!(loaded.iter().any(|name| name == "Blizzard_FrameXMLUtil"));
    env.exec("assert(cf2_manager and cf2_notifications > 0, 'initial cache notification')")
        .unwrap();
    assert_consumer_errors_absent(&env);
    env
}

fn assert_consumer_errors_absent(env: &WowLuaEnv) {
    let errors = env.state().borrow().lua_errors.clone();
    eprintln!("CF2 complete error array: {errors:#?}");
    assert!(
        !errors
            .iter()
            .any(|error| error.contains("Blizzard_QuestSuperTracking.lua")),
        "original consumer must have neither :52/:56 nor a newly exposed error: {errors:#?}"
    );
}

fn supply_quests(env: &WowLuaEnv) {
    let mut state = env.state().borrow_mut();
    let mut first = state.quest_log_entries.entries[0].clone();
    first.quest_id = 70001;
    first.is_complete = true;
    first.is_failed = false;
    first.is_world_quest = false;
    let mut second = first.clone();
    second.quest_id = 70002;
    second.is_complete = false;
    state.quest_log_entries.entries.extend([first, second]);
    state.quest_log_entries.additional_highlights.extend([
        (
            70001,
            QuestAdditionalHighlights {
                ui_map_id: 84,
                world_quests: true,
                world_quests_elite: false,
                dungeons: true,
                treasures: false,
            },
        ),
        (
            70002,
            QuestAdditionalHighlights {
                ui_map_id: 85,
                world_quests: false,
                world_quests_elite: true,
                dungeons: false,
                treasures: true,
            },
        ),
    ]);
}

fn content_transitions(env: &WowLuaEnv) {
    env.exec(r#"
        assert(cf2_manager.superTrackedContentType == nil)
        C_SuperTrack.SetSuperTrackedContent(Enum.ContentTrackingType.Appearance, 101)
        assert(cf2_manager.superTrackedContentType == 0 and cf2_manager.superTrackedContentID == 101)
        assert(C_SuperTrack.IsSuperTrackingAnything())
        C_SuperTrack.SetSuperTrackedContent(Enum.ContentTrackingType.Achievement, 202)
        local kind, id = QuestSuperTracking_GetSuperTrackedContent(cf2_manager)
        assert(kind == 2 and id == 202)
        assert(not (kind == 0 and id == 101))
        C_SuperTrack.ClearSuperTrackedContent()
        assert(cf2_manager.superTrackedContentType == nil and cf2_manager.superTrackedContentID == nil)
        assert(not C_SuperTrack.IsSuperTrackingAnything())
    "#).unwrap();
    assert_consumer_errors_absent(env);
}

#[test]
fn stock_supertrack_initialization_completes() {
    test_timeout! {
        let env = crate::spell_casting::env_with_full_blizzard_ui();
        assert_consumer_errors_absent(&env);
        observe(&env);
        env.fire_event("SUPER_TRACKING_CHANGED").unwrap();
        env.exec("assert(cf2_manager and cf2_manager.superTrackedQuestID == nil)").unwrap();
        // Stock absence alone is preservation; transitions reject constant/no-op producers.
        content_transitions(&env);
    }
}

#[test]
fn original_supertrack_content_changes_and_clears() {
    test_timeout! { content_transitions(&original_consumer()); }
}

#[test]
fn original_supertrack_quest_highlights_replace_and_reset() {
    test_timeout! {
        let env = original_consumer();
        supply_quests(&env);
        env.exec(r#"
            assert(cf2_manager.superTrackedQuestID == nil)
            C_SuperTrack.SetSuperTrackedQuestID(70001)
            assert(C_SuperTrack.GetSuperTrackedQuestID() == 70001)
            assert(cf2_manager.superTrackedQuestID == 70001 and cf2_manager.uiMapID == 84)
            assert(cf2_manager.worldQuests == true and cf2_manager.worldQuestsElite == false)
            assert(cf2_manager.dungeons == true and cf2_manager.treasures == false)
            assert(QuestSuperTracking_ShouldHighlightWorldQuests(84) == true)
            assert(QuestSuperTracking_ShouldHighlightWorldQuests(85) == false)
            C_SuperTrack.SetSuperTrackedQuestID(70002)
            assert(C_SuperTrack.GetSuperTrackedQuestID() == 70002)
            assert(cf2_manager.superTrackedQuestID == 70002 and cf2_manager.uiMapID == 85)
            assert(cf2_manager.worldQuests == false and cf2_manager.worldQuestsElite == true)
            assert(cf2_manager.dungeons == false and cf2_manager.treasures == true)
            assert(QuestSuperTracking_ShouldHighlightWorldQuestsElite(85) == true)
            assert(QuestSuperTracking_ShouldHighlightDungeons(84) == false)
            assert(QuestSuperTracking_ShouldHighlightTreasures(85) == true)
        "#).unwrap();
        assert_readiness_changes(&env);
        env.exec(r#"
            local notifications = cf2_notifications
            C_SuperTrack.SetSuperTrackedQuestID(878)
            assert(cf2_notifications > notifications, 'no-highlight quest must publish its cache')
            assert(cf2_manager.superTrackedQuestID == 878 and cf2_manager.uiMapID == 0)
            assert(cf2_manager.worldQuests == false and cf2_manager.worldQuestsElite == false)
            assert(cf2_manager.dungeons == false and cf2_manager.treasures == false)
            assert(QuestSuperTracking_ShouldHighlightWorldQuests(84) == false)
            C_SuperTrack.SetSuperTrackedQuestID(0)
            assert(C_SuperTrack.GetSuperTrackedQuestID() == nil)
            assert(cf2_manager.superTrackedQuestID == nil and cf2_manager.uiMapID == nil)
            assert(cf2_manager.worldQuests == nil and cf2_manager.worldQuestsElite == nil)
            assert(cf2_manager.dungeons == nil and cf2_manager.treasures == nil)
            assert(cf2_manager.isComplete == nil)
        "#).unwrap();
        assert_consumer_errors_absent(&env);
    }
}

fn assert_readiness_changes(env: &WowLuaEnv) {
    use wow_ui_sim::client_profile::{ACTIVE, ClientProfile};
    if ACTIVE == ClientProfile::Ptr {
        env.exec("assert(cf2_manager.isComplete == false)").unwrap();
        return;
    }
    env.exec("C_SuperTrack.SetSuperTrackedQuestID(70001); assert(cf2_manager.isComplete == true)")
        .unwrap();
    {
        let mut state = env.state().borrow_mut();
        let quest = state
            .quest_log_entries
            .entries
            .iter_mut()
            .find(|q| q.quest_id == 70001)
            .unwrap();
        quest.is_failed = true;
    }
    env.fire_event("SUPER_TRACKING_CHANGED").unwrap();
    env.exec("assert(cf2_manager.isComplete == false)").unwrap();
}

#[test]
fn original_supertrack_matching_and_nonmatching_quest_clear() {
    test_timeout! {
        let env = original_consumer();
        supply_quests(&env);
        env.exec("C_SuperTrack.SetSuperTrackedQuestID(70001)").unwrap();
        env.fire_event_with_args("QUEST_TURNED_IN", &[rilua::Val::Num(70002.0)]).unwrap();
        env.exec("assert(cf2_manager.superTrackedQuestID == 70001)").unwrap();
        env.fire_event_with_args("QUEST_TURNED_IN", &[rilua::Val::Num(70001.0)]).unwrap();
        env.exec("assert(cf2_manager.superTrackedQuestID == nil); C_SuperTrack.SetSuperTrackedQuestID(70001)").unwrap();
        for tracked in [rilua::Val::Nil, rilua::Val::Bool(true)] {
            env.fire_event_with_args("QUEST_WATCH_LIST_CHANGED", &[rilua::Val::Num(70001.0), tracked]).unwrap();
            env.exec("assert(cf2_manager.superTrackedQuestID == 70001)").unwrap();
        }
        env.fire_event_with_args("QUEST_WATCH_LIST_CHANGED", &[rilua::Val::Num(70002.0), rilua::Val::Bool(false)]).unwrap();
        env.exec("assert(cf2_manager.superTrackedQuestID == 70001)").unwrap();
        env.fire_event_with_args("QUEST_WATCH_LIST_CHANGED", &[rilua::Val::Num(70001.0), rilua::Val::Bool(false)]).unwrap();
        env.exec("assert(cf2_manager.superTrackedQuestID == nil and C_SuperTrack.GetSuperTrackedQuestID() == nil)").unwrap();
        assert_consumer_errors_absent(&env);
    }
}

#[test]
fn original_supertrack_accept_offer_and_auto_track() {
    test_timeout! {
        let env = original_consumer();
        supply_quests(&env);
        let offer: i32 = env.eval("return Enum.SuperTrackingMapPinType.QuestOffer").unwrap();
        {
            let mut state = env.state().borrow_mut();
            state.super_track.map_pin = Some((offer, 70001));
            state.super_track.map_pin_saved = (offer, 70001);
            state.super_track.map_pin_tracking = true;
        }
        env.fire_event_with_args("QUEST_ACCEPTED", &[rilua::Val::Num(70001.0)]).unwrap();
        env.exec("assert(cf2_manager.superTrackedMapPinType == nil and cf2_manager.superTrackedQuestID == 70001)").unwrap();
        {
            let mut state = env.state().borrow_mut();
            state.super_track.map_pin = Some((offer, 70002));
            state.super_track.map_pin_saved = (offer, 70002);
            state.super_track.map_pin_tracking = true;
        }
        env.fire_event_with_args("QUEST_ACCEPTED", &[rilua::Val::Num(70001.0)]).unwrap();
        env.exec(r#"
            local kind, id = C_SuperTrack.GetSuperTrackedMapPin()
            assert(kind == Enum.SuperTrackingMapPinType.QuestOffer and id == 70002)
            C_SuperTrack.ClearAllSuperTracked()
            assert(not C_SuperTrack.IsSuperTrackingAnything())
        "#).unwrap();
        env.exec("C_SuperTrack.SetSuperTrackedContent(0, 101)").unwrap();
        env.fire_event_with_args("QUEST_ACCEPTED", &[rilua::Val::Num(70002.0)]).unwrap();
        env.exec("assert(cf2_manager.superTrackedQuestID == nil); C_SuperTrack.ClearAllSuperTracked()").unwrap();
        // Existing world/task classification uses the static world-quest catalog.
        env.exec("assert(C_QuestLog.IsWorldQuest(90101) == true)").unwrap();
        env.fire_event_with_args("QUEST_ACCEPTED", &[rilua::Val::Num(90101.0)]).unwrap();
        env.exec("assert(cf2_manager.superTrackedQuestID == nil)").unwrap();
        env.fire_event_with_args("QUEST_ACCEPTED", &[rilua::Val::Num(70002.0)]).unwrap();
        env.exec("assert(cf2_manager.superTrackedQuestID == 70002)").unwrap();
        assert_consumer_errors_absent(&env);
    }
}
