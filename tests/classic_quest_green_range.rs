//! Classic `GetQuestGreenRange` backing the pinned difficulty-color helper.
//!
//! The classic `Blizzard_UIParent` TOCs select `Vanilla|Mists\UIParent.lua`
//! (`era` cache `Blizzard_UIParent_Vanilla.toc:14`, `mists` cache
//! `Blizzard_UIParent_Mists.toc:14`), whose `GetRelativeDifficultyColor` calls
//! `GetQuestGreenRange()` at `Vanilla/UIParent.lua:1419` and
//! `Mists/UIParent.lua:1628`. Wrath keeps its own verified 12340 table and test
//! (`tests/wrath_quest_abandon.rs`); Retail, PTR and WowForever do not publish
//! the global.

use crate::common::blizzard_addon_harness::build_blizzard_addon_closure_env;

#[cfg(any(
    feature = "client-mists",
    feature = "client-era",
    feature = "client-anniversary"
))]
fn load_classic_uiparent() -> wow_ui_sim::lua_api::WowLuaEnv {
    let ui = wow_ui_sim::paths::default_blizzard_ui_addons_path()
        .expect("Blizzard UI cache should be available");
    let (env, loaded) = build_blizzard_addon_closure_env(&ui, &["Blizzard_UIParent"], &[]);
    assert!(
        loaded.iter().any(|name| name == "Blizzard_UIParent"),
        "the classic UIParent difficulty-color consumer must load"
    );
    env
}

#[cfg(any(
    feature = "client-mists",
    feature = "client-era",
    feature = "client-anniversary"
))]
#[test]
fn classic_quest_green_range_follows_player_level_bands() {
    let env = load_classic_uiparent();
    for (level, green_range) in [(9, 4), (45, 8)] {
        env.state().borrow_mut().player.level = level;
        let (observed_range, green_is_standard, gray_is_trivial): (i32, bool, bool) = env
            .eval(&format!(
                r#"
                local range = GetQuestGreenRange()
                local green = GetRelativeDifficultyColor({level}, {level} - range)
                local gray = GetRelativeDifficultyColor({level}, {level} - range - 1)
                return range,
                       green == QuestDifficultyColors.standard,
                       gray == QuestDifficultyColors.trivial
                "#
            ))
            .expect("classic difficulty-color helper");
        assert_eq!(
            (observed_range, green_is_standard, gray_is_trivial),
            (green_range, true, true),
            "player level {level}: range drives the green/trivial boundary"
        );
    }
}
