//! Original selected Retail header animation, not a guessed animation clock contract.
#![cfg(all(feature = "client-retail", feature = "optional-tests"))]

use wow_ui_sim::loader::find_toc_file;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn tracker_env() -> WowLuaEnv {
    let cache = wow_ui_sim::blizzard_ui_sync::default_cache_addons_path().unwrap();
    let toc_path = find_toc_file(&cache.join("Blizzard_ObjectiveTracker")).unwrap();
    assert_eq!(
        toc_path.file_name().unwrap(),
        "Blizzard_ObjectiveTracker.toc"
    );
    let toc = TocFile::from_file(&toc_path).unwrap();
    assert!(toc.allows_screen(ScreenKind::Game));
    assert!(!toc.is_game_type_restricted());
    for selected in [
        "Blizzard_ObjectiveTrackerModule.lua",
        "Blizzard_ObjectiveTrackerModule.xml",
        "Blizzard_QuestObjectiveTracker.lua",
        "Blizzard_QuestObjectiveTracker.xml",
    ] {
        assert!(toc.files.iter().any(|file| file == selected), "{selected}");
    }
    let (env, loaded) = crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env(
        &cache,
        &["Blizzard_ObjectiveTracker"],
        &[],
    );
    assert!(
        loaded
            .iter()
            .any(|name| name == "Blizzard_ObjectiveTracker")
    );
    env.exec(
        r#"
        local header = QuestObjectiveTracker.Header
        assert(header and header:GetParent() == QuestObjectiveTracker)
        assert(header.PlayAddAnimation == ObjectiveTrackerModuleHeaderMixin.PlayAddAnimation)
        local info = debug.getinfo(header.PlayAddAnimation, 'S')
        assert(info.source:gsub('\\', '/'):find('Blizzard_ObjectiveTrackerModule.lua', 1, true))
        print('original header consumer: ' .. info.source .. ':' .. info.linedefined)
        assert(header.AddAnim and header.Background and header.Glow and header.Shine)
        -- Supply only original layout fields, not methods, API results or animation state.
        QuestObjectiveTracker.state = ObjectiveTrackerModuleState.ShownFully
        QuestObjectiveTracker.contentsHeight = 80
        QuestObjectiveTracker.collapsed = false
        assert(QuestObjectiveTracker:GetContentsHeight() == 80)
    "#,
    )
    .unwrap();
    env
}

#[test]
fn original_header_animation_survives_post_event_and_repeat_application() {
    let env = tracker_env();
    for _ in 0..2 {
        env.exec("QuestObjectiveTracker.Header:PlayAddAnimation(); assert(QuestObjectiveTracker.Header.AddAnim:IsPlaying(), 'original Restart starts XML AddAnim')")
            .unwrap();
        // The real startup hook is the producer under test. No vendor replacement,
        // time advancement, artificial animation sink or asserted final alpha.
        env.apply_post_event_workarounds();
        let playing: bool = env
            .eval("return QuestObjectiveTracker.Header.AddAnim:IsPlaying()")
            .unwrap();
        assert!(
            playing,
            "post-event workaround must not Stop the original XML AddAnim"
        );
    }
}

#[test]
fn collapsed_and_empty_header_animation_are_preserved() {
    // Preservation only: the old workaround already skips normalization here.
    let env = tracker_env();
    for layout in [
        "QuestObjectiveTracker.collapsed = true",
        "QuestObjectiveTracker.collapsed = false; QuestObjectiveTracker.contentsHeight = 0; QuestObjectiveTracker.usedBlocks = {}",
    ] {
        env.exec(layout).unwrap();
        env.exec("QuestObjectiveTracker.Header:PlayAddAnimation(); assert(QuestObjectiveTracker.Header.AddAnim:IsPlaying())").unwrap();
        env.apply_post_event_workarounds();
        let playing: bool = env
            .eval("return QuestObjectiveTracker.Header.AddAnim:IsPlaying()")
            .unwrap();
        assert!(
            playing,
            "collapsed/empty preservation must not interrupt original animation"
        );
    }
}
