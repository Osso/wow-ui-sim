use super::{load_full_game_ui_with_all_lod, missing_runtime_functions};

const PTR_METHODS: &[&str] = &[
    "CombatAudioAlertUtil.EnumerateInterruptCastInfo",
    "CombatAudioAlertUtil.EnumerateInterruptCastSuccessInfo",
    "CombatAudioAlertUtil.EnumerateSayCombatEndInfo",
    "CombatAudioAlertUtil.EnumerateSayCombatStartInfo",
    "CombatAudioAlertUtil.EnumeratetWhenTargetDiesInfo",
    "CombatAudioAlertUtil.GetInterruptCastInfo",
    "CombatAudioAlertUtil.GetInterruptCastSuccessInfo",
    "CombatAudioAlertUtil.GetSayCombatEndInfo",
    "CombatAudioAlertUtil.GetSayCombatStartInfo",
    "CombatAudioAlertUtil.GetWhenTargetDiesInfo",
];

/// Proves the pinned PTR CombatAudioAlertUtil methods are published after their LoD owners load.
#[test]
fn ptr_combat_audio_methods_are_published_by_the_loaded_owner() {
    let env = load_full_game_ui_with_all_lod();
    let missing = missing_runtime_functions(&env, PTR_METHODS);
    assert_eq!(missing, "", "pinned PTR methods must remain published");
}
