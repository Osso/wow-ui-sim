# Rated PvP UI eligibility

`C_PvP.CanPlayerUseRatedPVPUI()` returns the explicit rated eligibility input and a reason string. When eligible, the reason is empty; when ineligible, it is the supplied failure reason. The default is ineligible with an empty reason. Retail, PTR and WoW Forever publish this query.

## Best-guess basis (best-guess, native UNKNOWN)

The pinned Retail, PTR and WoW Forever `Blizzard_APIDocumentationGenerated/PvpInfoDocumentation.lua:47–55` declare a boolean and non-nil string result. Retail `Blizzard_PVPUI/Mainline/Blizzard_PVPUI.lua:359–363` uses both to set the rated button state and tooltip. The disabled/empty default follows the simulator's training-ground input policy; native defaults are unknown.

## Known gaps

Native eligibility rules and failure reasons are not modeled; callers must supply the desired input state.
