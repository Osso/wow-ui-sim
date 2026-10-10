//! Matching Mists source fade sequencing, not native-client timing parity.
#![cfg(feature = "client-mists")]

#[test]
fn mists_idle_cast_bar_hides_after_source_fade_finishes() {
    crate::common::with_timeout(90, || {
        wow_ui_sim::loader::enter_bytecode_cache_parent_bypass_mode();
        let env = crate::spell_casting::env_with_full_blizzard_ui();
        let released = wow_ui_sim::loader::release_prefork_parent_bytecode_cache_memory()
            .expect("seal the original constructor's source-only cache");
        assert_eq!(released, 0);
        wow_ui_sim::loader::enter_bytecode_cache_read_only_mode();

        let initial: (bool, bool, bool) = env
            .eval(
                "local cast = PlayerCastingBarFrame; return cast:IsShown(), \
                 cast.FadeOutAnim:IsPlaying(), \
                 UnitCastingInfo('player') == nil and UnitChannelInfo('player') == nil",
            )
            .expect("read actual idle cast and source animation state");
        eprintln!("MISTS_CAST_FADE initial: {initial:?}");
        assert!(initial.2, "backing player state must be idle");
        assert!(initial.1, "source FinishSpell must start FadeOutAnim");
        assert!(initial.0, "source fade has not completed before any tick");

        // Actual XML: 0.2-second delay, then 0.3-second alpha fade.
        env.fire_on_update(0.25)
            .expect("advance normal frame/animation pipeline before fade completion");
        let midway: (bool, bool) = env
            .eval("return PlayerCastingBarFrame:IsShown(), PlayerCastingBarFrame.FadeOutAnim:IsPlaying()")
            .expect("read unfinished source fade");
        eprintln!("MISTS_CAST_FADE after 0.25s: {midway:?}");
        assert_eq!(midway, (true, true));

        env.fire_on_update(0.30)
            .expect("advance normal pipeline through source OnFinished");
        let finished: (bool, bool, bool) = env
            .eval(
                "local cast = PlayerCastingBarFrame; return cast:IsShown(), \
                 cast.FadeOutAnim:IsPlaying(), \
                 UIParentBottomManagedFrameContainer.showingFrames[cast] == nil",
            )
            .expect("read actual source fade and managed-frame removal");
        eprintln!("MISTS_CAST_FADE after 0.55s: {finished:?}");
        assert_eq!(finished, (false, false, true));
    });
}
