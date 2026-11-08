# Issue 13 host defaults

For Retail and WoW Forever:

- `C_PlayerInteractionManager.IsReplacingUnit()` returns false. User-approved default, not native evidence. Both pinned `PlayerInteractionManagerDocumentation.lua:64-70` declare one boolean result.
- `C_VoiceChat.CanAccessSettings()` initially returns false. VERIFIED against PTR 12.1.7, Retail 12.1.5 and Forever 1.60.1 binaries: it is the AND of voice-service readiness state and starts false (see `native-evidence-12.1.7/C_VoiceChat.CanAccessSettings.md`). Supplied observations still override it. See [voice settings access](voice-settings-access.md).
- `UnitPowerDisplayMod(powerType)` returns 1 while the existing `UnitPower` model leaves unmodified values unscaled. User-approved default, not native evidence. The pinned `UnitDocumentation.lua:2697` (Retail) and `:2786` (Forever) declare it. Replace this temporary default when the power model gains scaling.
- `FlashClientIcon(briefly?)` returns no values and has no host effect. User-approved default, not native evidence. Both pinned `ClientDocumentation.lua:10-16` declare it. Window attention is intentionally unsupported.
