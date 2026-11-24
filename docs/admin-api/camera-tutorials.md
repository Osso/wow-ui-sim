# Camera and tutorial inputs

Retail and WoW Forever expose simulator inputs for the Blizzard Graphics and tutorial consumers.

- `A_Admin.SetCameraFOV(value)` supplies the current `cameraFov` CVar value read by the Graphics setting. `GetCameraFOVDefaults()` returns the native verified values `90, 50, 90` and has no scenario override.
- `A_Admin.SetTutorialFlag(id, flagged?)` adds or removes one ID in the simulator's supplied tutorial set. The omitted `flagged` value means `true`; the set is empty by default. Native account-versus-character scope is unknown.
- `A_Admin.SetTutorialsEnabled(enabled)` supplies the `showTutorials` CVar value used by `GetTutorialsEnabled()`. The CVar mapping is inferred from the Blizzard UI cache; the native setting identity is unknown.
- `A_Admin.SetPlayerNPERestricted(restricted)` supplies the player gate for the New Player Experience bootstrap consumer.

Example:

```lua
A_Admin.SetCameraFOV(70)
A_Admin.SetTutorialFlag(10, true)
A_Admin.SetTutorialsEnabled(true)
A_Admin.SetPlayerNPERestricted(true)
```
