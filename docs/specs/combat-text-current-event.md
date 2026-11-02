# Combat text current event

Retail and PTR `C_CombatText.GetCurrentEventInfo` returns the positional payload installed for the current synchronous combat-text dispatch. Outside dispatch it returns zero values. Retail `Blizzard_APIDocumentationGenerated/CombatTextDocumentation.lua:21-29` documents the API; WM49-C establishes the positional layouts and emulates publication and scope.

`A_Admin.FireEvent("COMBAT_TEXT_UPDATE", name, ...)` supplies a simulator event name and payload. It validates the payload against the recovered WM49-C layouts, installs it before dispatch, passes only the name to listeners, and clears it after dispatch. This is a simulator input, not a gameplay event producer. The six names ENCHANTMENT_REMOVED, ENCHANTMENT_ADDED, ARENA_POINTS_GAINED, PROC_RESISTED, MISFIRE, and SPELL_MISFIRE have unknown layouts and are rejected, as are other names without recovered layouts. Nested callback behavior remains unverified.
