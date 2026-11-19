#[cfg(feature = "client-wowforever")]
fn read_forever_cached_quest_colors() -> (bool, (f64, bool, bool, f64)) {
    use crate::common::blizzard_addon_harness::{
        blizzard_ui_dir, build_blizzard_addon_closure_env,
    };

    let (env, loaded) = build_blizzard_addon_closure_env(
        &blizzard_ui_dir(),
        &["Blizzard_FrameXMLBase", "Blizzard_FrameXMLUtil"],
        &[],
    );
    let provider_at = loaded
        .iter()
        .position(|name| name == "Blizzard_FrameXMLBase");
    let consumer_at = loaded
        .iter()
        .position(|name| name == "Blizzard_FrameXMLUtil");
    let ordered = provider_at.expect("cached constants provider should load")
        < consumer_at.expect("cached difficulty consumer should load");
    let observed: (f64, bool, bool, f64) = env
        .eval(
            r#"
            local impossible = QuestDifficultyColors.impossible
            local color, highlight = DifficultyUtil.GetDifficultyColor(Enum.RelativeContentDifficulty.Easy)
            return impossible.g,
                   color == QuestDifficultyColors.standard,
                   highlight == QuestDifficultyHighlightColors.standard,
                   highlight.g
            "#,
        )
        .expect("cached difficulty consumer should return source colors");
    (ordered, observed)
}

#[cfg(feature = "client-wowforever")]
#[test]
fn forever_cached_quest_difficulty_provider_drives_original_consumer() {
    let (provider_precedes_consumer, observed) = read_forever_cached_quest_colors();
    assert!(provider_precedes_consumer);
    assert_eq!(observed, (0.10, true, true, 0.93));
}
