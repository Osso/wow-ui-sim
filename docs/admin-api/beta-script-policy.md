# Beta script policy (A_Admin)

`A_Admin.SetScriptsDisallowedForBeta(disallowed)` supplies the beta-realm script policy used by `C_AddOns.GetScriptsDisallowedForBeta()` on Retail and WoW Forever. This setter is simulator-only.

**VERIFIED (Retail and WoW Forever `Blizzard_APIDocumentationGenerated/AddOnsDocumentation.lua:265-270`):** `GetScriptsDisallowedForBeta()` returns one non-nil Boolean. **VERIFIED (Retail and WoW Forever `Blizzard_ChatFrameBase/Shared/SlashCommands.lua:1173-1189` and `Blizzard_GameMenu/Shared/GameMenuFrame.lua:176`):** Blizzard uses this result to gate script commands and disable the AddOns menu button. **INFERRED (R88fs (native evidence pending)):** the simulator default is `false`; the admin setter supplies either Boolean value for scenarios.

```lua
A_Admin.SetScriptsDisallowedForBeta(true)
-- Run a scenario with scripts disallowed.
A_Admin.SetScriptsDisallowedForBeta(false)
```

Each call replaces the current policy input. Kiosk session state is separate and must be disabled when a scenario needs to isolate this policy.
