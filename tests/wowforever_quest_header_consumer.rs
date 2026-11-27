//! Whole cached header-watch consumer; simulator policy, not native/UI parity.
#![cfg(feature = "client-wowforever")]

use wow_ui_sim::c_api::QuestWatchType;
use wow_ui_sim::lua_api::WowLuaEnv;

fn load_header_consumer_env() -> WowLuaEnv {
    use crate::common::blizzard_addon_harness::{
        blizzard_ui_dir, load_blizzard_addon_closure_into_env, new_blizzard_addon_env,
    };
    // QuestMapFrame.lua:1 reads EnumUtil (Blizzard_SharedXMLBase EnumUtil.lua;
    // 096d8a1f8c retired the bootstrap copy).
    let cache = blizzard_ui_dir();
    let env = new_blizzard_addon_env(&cache);
    load_blizzard_addon_closure_into_env(&env, &cache, &["Blizzard_SharedXMLBase"], &[]);
    // Same whole-cache loading style as wowforever_quest_difficulty.rs.
    // No source slicing, rewriting, target-body copying, or C API substitution.
    // TimeUtil.lua owns SecondsFormatter, read by QuestUtils.lua at file scope;
    // b20797e67a retired the bootstrap formatter copy.
    for file in [
        "Blizzard_SharedXML/TimeUtil.lua",
        "Blizzard_FrameXMLUtil/Mainline/QuestUtils.lua",
        "Blizzard_UIPanels_Game/Mainline/QuestMapFrame.lua",
    ] {
        let path = cache.join(file);
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read whole cached {}: {error}", path.display()));
        env.exec(&source)
            .unwrap_or_else(|error| panic!("load whole cached {}: {error}", path.display()));
    }
    env.exec(include_str!("wowforever_quest_header_consumer.lua"))
        .expect("define observers and cases after loading whole vendor files");
    env
}

#[test]
fn forever_header_consumer_untracks_and_retracks_seeded_header_one() {
    let env = load_header_consumer_env();
    env.exec("WatchHeaderConsumerTests.seededRoundTrip()")
        .expect("real whole-file consumer mutates seed membership synchronously");
}

#[test]
fn forever_header_consumer_tracks_mixed_live_rows_without_promoting_world_quests() {
    let env = load_header_consumer_env();
    {
        let mut state = env.state().borrow_mut();
        let seed = state.quest_log_entries.entries.clone();
        let mut supplied = seed[0].clone();
        supplied.quest_id = 456792;
        supplied.title = "Supplied ordinary row after world rows".into();
        supplied.level = 29;
        // Rich rows have no header field: use the real synthesized header 1.
        state.quest_log_entries.entries = vec![
            seed[0].clone(),
            seed[2].clone(),
            seed[1].clone(),
            seed[3].clone(),
            supplied,
        ];
        state
            .quest_watch_types
            .insert(80000, QuestWatchType::Automatic);
    }
    env.exec("WatchHeaderConsumerTests.mixedLiveRows()")
        .expect("real membership handles world interleaving, live rows and duplicate tracking");
}

#[test]
fn forever_header_consumer_obeys_real_npe_removal_gate() {
    let env = load_header_consumer_env();
    env.state().borrow_mut().player.is_npe_restricted = true;
    env.exec("WatchHeaderConsumerTests.restrictedUntrack()")
        .expect("real QuestUtil gate prevents removal while still resetting receiver and refreshing");
    env.state().borrow_mut().player.is_npe_restricted = false;
    env.exec("WatchHeaderConsumerTests.unrestrictedUntrack()")
        .expect("same consumer removes watches after real player restriction changes");
}
