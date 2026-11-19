#[cfg(feature = "client-wowforever")]
#[test]
fn forever_cached_quest_difficulty_provider_drives_original_consumer() {
    use crate::common::blizzard_addon_harness::{
        blizzard_ui_dir, build_blizzard_addon_closure_env,
    };

    let (env, loaded) = build_blizzard_addon_closure_env(
        &blizzard_ui_dir(),
        &["Blizzard_FrameXMLBase", "Blizzard_FrameXMLUtil"],
        &[],
    );
    let colors_at = loaded
        .iter()
        .position(|name| name == "Blizzard_FrameXMLBase")
        .expect("cached constants provider should load");
    let consumer_at = loaded
        .iter()
        .position(|name| name == "Blizzard_FrameXMLUtil")
        .expect("cached difficulty consumer should load");
    assert!(colors_at < consumer_at);

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
    assert_eq!(observed, (0.10, true, true, 0.93));
}
