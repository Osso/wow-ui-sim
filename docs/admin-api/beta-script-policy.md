# Beta script policy (A_Admin)

`A_Admin.SetScriptsDisallowedForBeta(disallowed)` supplies the beta-realm script policy used by `C_AddOns.GetScriptsDisallowedForBeta()` on Retail and WoW Forever. This setter is simulator-only.

**VERIFIED (Retail and WoW Forever `Blizzard_APIDocumentationGenerated/AddOnsDocumentation.lua:265-270`):** `GetScriptsDisallowedForBeta()` returns one non-nil Boolean. **VERIFIED (Retail `Blizzard_ChatFrameBase/Shared/SlashCommands.lua:1173-1187`; WoW Forever `Blizzard_ChatFrameBase/Shared/SlashCommands.lua:1224-1236`; both profiles `Blizzard_GameMenu/Shared/GameMenuFrame.lua:176`):** Blizzard uses this result to gate script commands and disable the AddOns menu button. **INFERRED (R88fs (native evidence pending)):** the simulator default is `false`; the admin setter supplies either Boolean value for scenarios.

**VERIFIED (Retail and WoW Forever `Blizzard_APIDocumentationGenerated/SlashCommandDocumentation.lua:8-23`):** `AreDangerousScriptsAllowed()` returns a Boolean, and `SetAllowDangerousScripts(allowed)` accepts an optional Boolean that defaults to `false`. **VERIFIED (Retail `Blizzard_StaticPopup_Game/GameDialogDefs.lua:3154-3163`; WoW Forever `Blizzard_StaticPopup_Game/GameDialogDefs.lua:3206-3215`):** accepting the original `DANGEROUS_SCRIPTS_WARNING` dialog enables dangerous scripts. **INFERRED (R88ft (native evidence pending)):** this is a persisted per-account setting in WoW; the simulator stores it in memory only and does not persist it.

```lua
A_Admin.SetScriptsDisallowedForBeta(true)
-- Run a scenario with scripts disallowed.
A_Admin.SetScriptsDisallowedForBeta(false)
```

Each call replaces the current policy input. Kiosk session state is separate and must be disabled when a scenario needs to isolate this policy.
