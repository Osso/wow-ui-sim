# Video capabilities

WoW Forever only. These inputs supply graphics support to the original antialiasing settings consumer. The simulator does not detect these capabilities from its renderer. Both values default to `false`.

**INFERRED (R88fy, native evidence pending):** the unsupplied defaults are false. In `Blizzard_SettingsDefinitions_Shared/Graphics.lua:1046-1050`, `MultiSampleAntiAliasingSupported()` controls `AA_MULTISAMPLE`, and both it and `AntiAliasingSupported()` must be true to add `AA_ADVANCED`.

## `A_Admin.SetMultiSampleAntiAliasingSupported(supported)`

Supplies whether the original graphics settings dropdown can offer `AA_MULTISAMPLE`. This input alone does not enable `AA_ADVANCED`.

## `A_Admin.SetAntiAliasingSupported(supported)`

Supplies the separate capability checked before the consumer adds `AA_ADVANCED`. It does not imply multisample support.

```lua
A_Admin.SetMultiSampleAntiAliasingSupported(true)
A_Admin.SetAntiAliasingSupported(true)
```
