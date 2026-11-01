# C_MacOptions

`C_MacOptions` is registered on every profile whose generated API docs declare
it (all profiles except Wrath). The simulator host is never macOS.

## Behavior

- `GetGameBundleName()` returns `"World of Warcraft"`.
- `IsInputMonitoringEnabled()`, `IsMicrophoneEnabled()`,
  `IsUniversalAccessEnabled()` and `HasNewStyleInputMonitoring()` return `false`.
- `SetOSShortcutsDisabled(disable)` stores `MacDisableOsShortcuts` as `"1"` or `"0"`.
- `AreOSShortcutsDisabled()` returns the stored CVar request and `false` for
  the applied OS state.
- `OpenInputMonitoring()`, `OpenMicrophoneRequestDialogue()` and
  `OpenUniversalAccess()` do nothing and return nothing.

## Best-guess basis

best-guess, native UNKNOWN. Blizzard consumers (`Blizzard_SettingsDefinitions_Shared`,
`Blizzard_Keybindings`) gate these calls behind `IsMacClient()`, which is false on
the simulator host. No macOS permission can be granted off macOS, so permission
queries report `false`. The bundle name is the retail macOS application name.

## Known gaps

- Classic-family bundle names are not distinguished.

Test: `tests/mac_options.rs`.
