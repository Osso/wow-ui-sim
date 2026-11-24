#![cfg(feature = "client-retail")]

use crate::common::blizzard_addon_harness::with_blizzard_addon_closure;

const SCRIPTED_ANIMATION_CONSUMER_CASE: &str = r#"
    local first = ScriptedAnimationEffectsUtil.GetEffectByID(1)
    local second = ScriptedAnimationEffectsUtil.GetEffectByID(2)
    assert(first and second, 'original consumer must resolve both DB2 effect IDs')
    assert(first.id == 1 and first.visual == 166128 and first.visualScale == 1)
    assert(first.duration == 0.4 and first.animationSpeed == 1 and first.animation == 0)
    assert(second.id == 2 and second.visual == 1327007 and second.visualScale == 0.8)
    assert(second.duration == 0.3 and second.animationSpeed == 1 and second.animation == 0)
    assert(first.visual ~= second.visual, 'distinct effect IDs must retain distinct visuals')

    A_Admin.SetScriptedAnimationEffects({
        { id = 90001, visual = 91001, visualScale = 1, duration = 2, trajectory = 1,
          yawRadians = 0, pitchRadians = 0, rollRadians = 0, offsetX = 0, offsetY = 0,
          offsetZ = 0, animation = 1, animationSpeed = 1, alpha = 1, useTargetAsSource = false },
        { id = 90002, visual = 91002, visualScale = 2, duration = 3, trajectory = 2,
          yawRadians = 1, pitchRadians = 2, rollRadians = 3, offsetX = 4, offsetY = 5,
          offsetZ = 6, animation = 2, animationSpeed = 0.5, alpha = 0.5, useTargetAsSource = true },
    })
    ScriptedAnimationEffectsUtil.ReloadDB()
    local suppliedFirst = ScriptedAnimationEffectsUtil.GetEffectByID(90001)
    local suppliedSecond = ScriptedAnimationEffectsUtil.GetEffectByID(90002)
    assert(suppliedFirst.visual == 91001 and suppliedFirst.animation == 1)
    assert(suppliedSecond.visual == 91002 and suppliedSecond.animation == 2)
    assert(suppliedFirst.visual ~= suppliedSecond.visual)
"#;

#[test]
fn original_consumer_resolves_distinct_scripted_animation_effects() {
    with_blizzard_addon_closure(&["Blizzard_SharedXML"], &[], |env, loaded| {
        assert!(loaded.iter().any(|name| name == "Blizzard_SharedXML"));
        env.exec(SCRIPTED_ANIMATION_CONSUMER_CASE)
            .expect("the loaded Blizzard consumer must resolve generated and supplied effect records");
    });
}
