# Camera and tutorial inputs

Retail and WoW Forever expose simulator inputs for the Blizzard Graphics and tutorial consumers. Retail and Forever `Blizzard_SettingsDefinitions_Shared.toc` uses `## AllowLoad: Both`; the graphics Lua loads the FOV consumer. `Blizzard_FrameXML.toc` selects `[Family]\\TutorialFrame.lua` without an `AllowLoadGameType` filter for Retail mainline and Forever mainline.

- `A_Admin.SetCameraFOV(value)` supplies the current `cameraFov` CVar value read by the Graphics setting. `GetCameraFOVDefaults()` returns the native verified values `90, 50, 90` and has no scenario override.
- `A_Admin.SetTutorialFlag(id, flagged?)` adds or removes one ID in the simulator's supplied tutorial set. The omitted `flagged` value means `true`; the set is empty by default. IDs outside 1–65 are ignored. `IsTutorialFlagged` reads the set with native 1-based ID and range semantics; native account-versus-character scope is unknown.
- `FlagTutorial(id)` adds an in-range ID to the same set. `ResetTutorials()` clears that set. The specific native tutorial settings that ResetTutorials also resets are unknown and are not modeled.
- `GetTutorialsEnabled()` stays a compatibility default. Runtime-selected consumers exist in Retail/Forever `Blizzard_NewPlayerExperience_Bootstrap.lua:4` and `Blizzard_SettingsDefinitions_Frame/Mainline/InterfaceOverrides.lua` (Retail line 258; Forever line 285). R88fh proves the native master gate and stored setting, but their simulator backing is not modeled here. The `showTutorials` CVar mapping is inferred from the cache. The shim stays active on Retail and Forever until the missing setting and master-gate state are modeled.

## Audit-only globals

- `MoveView*Start` and `MoveView*Stop` stay no-op shims. Retail `Bindings_Standard.xml:1509,1516` calls the in/out start verbs and Forever includes those calls in its Standard and Camelot binding XML. These are runtime-selected binding actions, not evidence for a simulator camera-move model or consumer-backed output; the task's no-op audit requirement keeps them unchanged.

Example:

```lua
A_Admin.SetCameraFOV(70)
A_Admin.SetTutorialFlag(10, true)
```
