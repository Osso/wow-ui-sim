//! Restored supertracking-only obligations from the deferred CF2 housing tests.
//! Supplied-state preservation, not native selection/arbitration policy.
#![cfg(any(
    feature = "client-retail",
    feature = "client-ptr",
    feature = "client-wowforever"
))]

use crate::common::blizzard_addon_harness::{
    blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
};
use wow_ui_sim::loader::find_toc_file;
use wow_ui_sim::lua_api::WowLuaEnv;
use wow_ui_sim::screen::ScreenKind;
use wow_ui_sim::toc::TocFile;

fn load_observed_consumer() -> WowLuaEnv {
    let ui = blizzard_ui_dir();
    let toc_path = find_toc_file(&ui.join("Blizzard_FrameXMLUtil")).unwrap();
    let toc = TocFile::from_file(&toc_path).unwrap();
    assert!(toc.allows_screen(ScreenKind::Game));
    assert!(!toc.is_game_type_restricted());
    assert!(
        toc.files.iter().any(|file| {
            file == std::path::Path::new("Mainline/Blizzard_QuestSuperTracking.lua")
        })
    );
    eprintln!("Selected supertracking TOC: {}", toc_path.display());
    let env = new_blizzard_addon_env(&ui);
    load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_SharedXMLGame"], &[]);
    register_cache_observer(&env);
    let loaded = load_blizzard_addon_closure_into_env(&env, &ui, &["Blizzard_FrameXMLUtil"], &[]);
    assert!(loaded.iter().any(|name| name == "Blizzard_FrameXMLUtil"));
    assert_listener_snapshot(&env, None, 1);
    env
}

fn register_cache_observer(env: &WowLuaEnv) {
    env.exec(
        r#"
        supertrack_observer = { notifications = 0 }
        EventRegistry:RegisterCallback('Supertracking.OnChanged', function(owner, tracker)
            owner.notifications = owner.notifications + 1
            owner.tracker = tracker
            owner.kind, owner.id = QuestSuperTracking_GetSuperTrackedContent(tracker)
        end, supertrack_observer)
        "#,
    )
    .expect("observe the original cache refresh without replacing the consumer");
}

fn assert_listener_snapshot(env: &WowLuaEnv, pair: Option<(i32, i32)>, count: usize) {
    let (kind, id) = pair.map_or_else(
        || ("nil".to_owned(), "nil".to_owned()),
        |(kind, id)| (kind.to_string(), id.to_string()),
    );
    env.exec(&format!(
        r#"
        local observer = supertrack_observer
        assert(observer.notifications == {count}, 'one notification per original refresh')
        local tracker = observer.tracker
        assert(type(tracker) == 'table', 'original listener exposes its tracker')
        local suffix = 'Blizzard_FrameXMLUtil/Mainline/Blizzard_QuestSuperTracking.lua'
        for _, fn in ipairs({{tracker.CacheCurrentSuperTrackInfo,
                tracker:GetScript('OnEvent'), QuestSuperTracking_GetSuperTrackedContent}}) do
            local info = debug.getinfo(fn, 'S')
            local source = info.source:gsub('\\', '/')
            assert(info.what == 'Lua' and source:sub(-#suffix) == suffix,
                'must execute original selected cache, event and helper')
            print('supertracking provenance', source, info.linedefined, info.lastlinedefined)
        end
        assert(tracker:IsEventRegistered('SUPER_TRACKING_CHANGED'))
        assert(observer.kind == {kind} and observer.id == {id},
            'listener must see the committed pair during notification')
        local cachedKind, cachedID = QuestSuperTracking_GetSuperTrackedContent(tracker)
        assert(cachedKind == {kind} and cachedID == {id}, 'original cached pair')
        assert(select('#', QuestSuperTracking_GetSuperTrackedContent(tracker)) == 2,
            'cache helper always returns its two cache fields, including nils')
        "#,
    ))
    .expect("original consumer cache and listener snapshot must match supplied state");
}

// Re-authenticated on Retail, PTR and Forever in their pinned caches:
// Blizzard_APIDocumentationGenerated/SuperTrackManagerDocumentation.lua:32-40
// (MayReturnNothing, otherwise ContentTrackingType + number), and
// ContentTrackingTypesDocumentation.lua:60-70 (Appearance=0, Mount=1,
// Achievement=2, Decor=3). IDs below are supplied fixtures, not native defaults.
#[test]
fn original_supertracking_pair_shape_restored() {
    let env = load_observed_consumer();
    env.exec(
        r#"
        local declared = {Appearance=0, Mount=1, Achievement=2, Decor=3}
        for name, value in pairs(declared) do
            assert(Enum.ContentTrackingType[name] == value, 'declared enum member')
        end
        local metadata = {NumValues=true, MinValue=true, MaxValue=true}
        for name in pairs(Enum.ContentTrackingType) do
            assert(declared[name] ~= nil or metadata[name], 'undocumented enum member '..name)
        end
        assert(select('#', QuestSuperTracking_GetSuperTrackedContent()) == 0,
            'absent binding returns no results, not one nil')
        "#,
    )
    .unwrap();
    for (index, pair) in [Some((0, 101)), Some((2, 202)), None, Some((0, 101))]
        .into_iter()
        .enumerate()
    {
        let mut state = env.state().borrow_mut();
        state.super_track.content = pair;
        if let Some(pair) = pair {
            state.super_track.content_saved = pair;
        }
        state.super_track.content_tracking = pair.is_some();
        let errors_before = env.state().borrow().lua_errors.len();
        env.fire_event("SUPER_TRACKING_CHANGED").unwrap();
        assert_eq!(env.state().borrow().lua_errors.len(), errors_before);
        assert_listener_snapshot(&env, pair, index + 2);
        assert_binding_shape(&env, pair);
    }
}

fn assert_binding_shape(env: &WowLuaEnv, pair: Option<(i32, i32)>) {
    let script = match pair {
        Some((kind, id)) => format!(
            "assert(select('#', QuestSuperTracking_GetSuperTrackedContent()) == 2); \
             local kind, id = QuestSuperTracking_GetSuperTrackedContent(); \
             assert(type(kind) == 'number' and type(id) == 'number'); \
             assert(kind == {kind} and id == {id})"
        ),
        None => "assert(select('#', QuestSuperTracking_GetSuperTrackedContent()) == 0)".to_owned(),
    };
    env.exec(&script).expect("documented optional binding pair");
}

#[test]
fn original_supertracking_refresh_publishes_cache_to_listeners_restored() {
    let env = load_observed_consumer();
    for (index, (mutation, pair)) in [
        (
            "C_SuperTrack.SetSuperTrackedContent(Enum.ContentTrackingType.Appearance, 101)",
            Some((0, 101)),
        ),
        (
            "C_SuperTrack.SetSuperTrackedContent(Enum.ContentTrackingType.Achievement, 202)",
            Some((2, 202)),
        ),
        ("C_SuperTrack.ClearSuperTrackedContent()", None),
        (
            "C_SuperTrack.SetSuperTrackedContent(Enum.ContentTrackingType.Appearance, 101)",
            Some((0, 101)),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let errors_before = env.state().borrow().lua_errors.len();
        env.exec(mutation)
            .expect("mutate supplied supertracking state");
        assert_eq!(env.state().borrow().lua_errors.len(), errors_before);
        assert_listener_snapshot(&env, pair, index + 2);
        assert_eq!(env.state().borrow().super_track.content, pair);
    }
}
