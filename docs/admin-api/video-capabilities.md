# Video capabilities

## `A_Admin.SetMultiSampleAntiAliasingSupported(supported)`

WoW Forever only. Supplies whether the original graphics settings dropdown can offer `AA_MULTISAMPLE` and `AA_ADVANCED`. The value is a host capability input; the simulator does not detect multisample support from its renderer. The default is `false`, matching the simulator's single-sample rendering capability.

**INFERRED (R88fy, native evidence pending):** the unsupplied default is false. The original Forever consumer calls `MultiSampleAntiAliasingSupported()` when it builds the antialiasing options in `Blizzard_SettingsDefinitions_Shared/Graphics.lua:1046-1050`.

```lua
A_Admin.SetMultiSampleAntiAliasingSupported(true)
```
