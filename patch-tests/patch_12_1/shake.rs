use super::load_game_ui_without_player_choice;

/// Proves the current PTR source retains both legacy globals and utility methods.
#[test]
fn shake_helpers_keep_current_legacy_and_namespaced_publications() {
    let env = load_game_ui_without_player_choice();

    let (legacy_shake, legacy_random, namespaced_shake, namespaced_random): (
        String,
        String,
        String,
        String,
    ) = env
        .eval(
            r#"
            return type(ShakeFrame),
                type(ShakeFrameRandom),
                type(ScriptAnimationUtil.ShakeFrame),
                type(ScriptAnimationUtil.ShakeFrameRandom)
            "#,
        )
        .expect("PTR shake helper publication probe succeeds");

    assert_eq!(legacy_shake, "function");
    assert_eq!(legacy_random, "function");
    assert_eq!(namespaced_shake, "function");
    assert_eq!(namespaced_random, "function");
}
