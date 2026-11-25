# Camera and tutorial inputs

Retail and WoW Forever expose simulator inputs for the Blizzard Graphics and tutorial consumers. `Blizzard_SettingsDefinitions_Shared.toc` uses `## AllowLoad: Both`; its Graphics file creates the camera-FOV proxy setting. `Blizzard_FrameXML.toc` selects `[Family]\\TutorialFrame.lua` without a game-type filter in the Retail and Forever caches.

- `A_Admin.SetCameraFOV(value)` changes the `cameraFov` CVar and dispatches its update, so the original Graphics proxy setting refreshes. `GetCameraFOVDefaults()` returns the native verified values `90, 50, 90`; it has no scenario override.
- `A_Admin.SetTutorialFlag(id, flagged?)` adds or removes one ID in the simulator tutorial set. Omitted `flagged` means `true`; the set is empty by default. IDs outside 1–65 are ignored. `IsTutorialFlagged` uses the verified native 1-based range and returns no values outside that range. Native account-versus-character scope is unknown.
- `FlagTutorial(id)` adds an in-range ID to the set. `ResetTutorials()` clears it. R88fh verifies that ResetTutorials also resets tutorial settings, but the setting identities are unknown and are not modeled.
- `A_Admin.SetTutorialsEnabled(enabled)` changes the inferred `showTutorials` setting through the CVar model. The setting identity is inferred from the Blizzard UI cache. `GetTutorialsEnabled()` uses this stored setting; the simulator defaults to enabled when no CVar value is available. Native master-gate state is not modeled.
- `A_Admin.SetPlayerNPERestricted(restricted)` supplies the New Player Experience restriction state used by its bootstrap consumer.

## Audit-only globals

- `MoveView*Start` and `MoveView*Stop` remain no-op shims. Their names occur in `Bindings_Standard.xml` and Forever's `Bindings_Camelot.xml`, but those files are not listed in either profile's `Blizzard_FrameXML.toc`. The task's no-op audit rule keeps them unchanged because the cache TOC does not establish a runtime-selected consumer.

Example:

```lua
A_Admin.SetCameraFOV(70)
A_Admin.SetTutorialFlag(10, true)
A_Admin.SetTutorialsEnabled(true)
A_Admin.SetPlayerNPERestricted(true)
```
