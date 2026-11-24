# Scripted animation effects

`A_Admin.SetScriptedAnimationEffects(effects)` supplies Retail scripted-animation records for `C_ScriptedAnimations.GetAllScriptedAnimationEffects()`.

- `effects` is an ordered array of records with the required fields `id`, `visual`, `visualScale`, `duration`, `trajectory`, `yawRadians`, `pitchRadians`, `rollRadians`, `offsetX`, `offsetY`, `offsetZ`, `animation`, `animationSpeed`, `alpha`, and `useTargetAsSource`.
- Optional fields are `startBehavior`, `startSoundKitID`, `finishEffectID`, `finishBehavior`, `finishSoundKitID`, `startAlphaFade`, `startAlphaFadeDuration`, `endAlphaFade`, `endAlphaFadeDuration`, `animationStartOffset`, `loopingSoundKitID`, and `particleOverrideScale`.
- Passing `nil` clears the supplied records and restores the generated Retail catalog.

This setter is simulator-only. It does not change Forever or PTR behavior.
