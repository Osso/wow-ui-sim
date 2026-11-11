# Selected model-scene catalogue

The simulator supplies scene, actor and camera IDs for scenes 596 (dressing room), 641 (Perks celebration) and 727 (Allied Races) from the pinned client catalogue. Actor script tags are the exported values. Both model-scene transition paths use these values; no 3D rendering is provided. `ModelSceneActor:SetAnimationBlendOperation` accepts documented `ModelBlendOperation` values 0 and 1 but does not render; other frame types do not expose it. Retail, PTR and WoW Forever `FrameAPIModelSceneFrameActorBaseDocumentation.lua:354–360`, and Mists, Era and Anniversary versions at `:349`, declare the method. Retail `ModelAnimationSharedDocumentation.lua:6–14` declares the enum. Wrath does not expose this method.

`python3 tools/gen_model_scenes.py` reads wago.tools DB2 CSV exports for build 12.1.5.70391, locale enUS: `UiModelScene.csv` SHA-256 `a7a6fb76876ba22ea2e63c42643b65e741c13c1af0d5fef6da784fac7b88a9c0`, `UiModelSceneActor.csv` `65e91ad5a7e5f8689395f9ed0ea03e44a3f57a055a0cec988de88b902088a58b`, and `UiModelSceneCamera.csv` `054cd99c25c6daf1b75866b8395d9cdf47ae2384d9710b92e85eed817391c585`.

The previous scene-596 seed invented `human-male`, `human` and `player-rider`; these tags are removed. The original `ModelSceneMixin:GetPlayerActor` tries those race labels and uses the exported `player` tag when absent. Exported race-specific tags, such as `tauren-male`, remain available.

## Known gaps

Only these three scenes are supplied. Camera and actor display properties and 3D rendering remain unsupported; this catalogue does not claim full `C_ModelInfo` parity. Character-create APIs may store camera-profile and zoom state for Lua readback, but blur, camera-profile, model-dress and preview-gear visuals remain part of this permanent 3D gap; see [character-create blur](character-create-blur.md) and [character-create camera and preview state](character-create-visual-state.md).
