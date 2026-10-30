# Issue 13 host defaults

For Retail and WoW Forever:

- `C_PlayerInteractionManager.IsReplacingUnit()` returns false. User-approved default, not native evidence. Both pinned `PlayerInteractionManagerDocumentation.lua:64-70` declare one boolean result.
- `C_VoiceChat.CanAccessSettings()` initially returns true. User-approved default, not native evidence; supplied observations still override it. See [voice settings access](voice-settings-access.md).
- `UnitPowerDisplayMod(powerType)` returns 1 while the existing `UnitPower` model leaves unmodified values unscaled. User-approved default, not native evidence. The pinned `UnitDocumentation.lua:2697` (Retail) and `:2786` (Forever) declare it. Replace this temporary default when the power model gains scaling.
- `FlashClientIcon(briefly?)` returns no values and has no host effect. User-approved default, not native evidence. Both pinned `ClientDocumentation.lua:10-16` declare it. Window attention is intentionally unsupported.
