# SavedVariables and lifecycle evidence

Research dated 2026-10-31; no new native-client execution or decompiled-source
inspection was performed. No usable route to the clean-room laptop session
`aso` was available, so no request was delivered; that does not mean it has no
evidence. Profile/build-specific follow-up remains required by
[repository policy](../../AGENTS.md#clean-room).
[The persistence spec](saved-variables.md) describes current simulator behavior.

## Shutdown evidence

[Wrath build 12340 findings](reload-ui.md#wrath-12340-requirements), relayed from
aso on 2026-10-19, include conditional world departure, `PLAYER_LOGOUT`, then
serialization after handlers finish, and account-before-character managed loading.
These are existing static native findings; they do not establish Forever chronology.

For Forever, the researched best guess is conditional argless
`PLAYER_LEAVING_WORLD`, argless `PLAYER_LOGOUT`, then
`ADDONS_UNLOADING(closingClient)` before serialization; proposed `closingClient` is true
for normal close/Quit and false for ReloadUI. This is not implemented general
shutdown behavior or measured native order. Public event/loading-order documentation
supports the inference but includes incomplete shutdown sourcing:
[PLAYER_LOGOUT](https://warcraft.wiki.gg/wiki/Event:PLAYER_LOGOUT),
[ADDONS_UNLOADING](https://warcraft.wiki.gg/wiki/Event:ADDONS_UNLOADING),
[loading process](https://warcraft.wiki.gg/wiki/AddOn_loading_process).

The consumer requirement is concrete: AceDB's logout handler fires database-shutdown
callbacks and removes defaults/empty sections. Saving earlier changes persisted
data. This establishes callback significance, not every engine event's chronology.
[AceDB shutdown handler](https://github.com/WoWUIDev/Ace3/blob/ac3d1225037e0dc74e025a72342c0f7824aae147/AceDB-3.0/AceDB-3.0.lua#L391)

## Data evidence and preservation policy

- TOC declarations define owned globals and account/character scopes. Documented
  values include strings, booleans, numbers and tables; shared tables reload as
  copies and cycles may not survive. The simulator uses raw entries, duplicates
  repeated acyclic references, omits metatables and rejects cycles/unsupported
  data; its 128-table nesting bound is its own limit.
  [SavedVariables documentation](https://warcraft.wiki.gg/wiki/Saved_variables)
- Nil globals are meaningful saved state, distinct from absent files or unloaded
  addons. The simulator emits explicit nil assignments.
  [Original developer explanation](https://us.forums.blizzard.com/en/wow/t/saving-local-variables-between-sessions/380081/4)
- Lua strings contain arbitrary bytes, including zero. Binary SavedVariables are
  also real consumer data. Exact bytes and unambiguous decimal escapes are required;
  UTF-8 replacement loses information. Fresh-VM checks prove simulator read-back,
  not exhaustive native escape spelling.
  [Lua lexical rules](https://www.lua.org/manual/5.1/manual.html#2.1),
  [WTFix consumer changelog](https://github.com/eggntoast/WTFix-Public/blob/main/CHANGELOG.md)
- Sparse and explicit numeric keys occur in native output, with historical numeric
  precision regressions. The simulator preserves finite doubles and signed zero,
  string/numeric/boolean keys and sparse entries; it rejects NaN/infinity and other
  key types. Boolean-key support is a preservation choice; native signed-zero and
  exceptional-number behavior remain unconfirmed.
  [Lua types](https://www.lua.org/manual/5.1/manual.html#2.2),
  [Original numeric repro](https://www.wowinterface.com/forums/showthread.php?t=49523)
- Successful input loading is separate from addon initialization. Blizzard's 2013
  policy prevented logout saves after memory-exhaustion load failures to protect
  valid settings. The simulator distinguishes failed input from missing files;
  the historical policy does not prove every current native failure mode or
  establish a general save-suppression policy in the simulator.
  [Blizzard announcement, archived](https://www.bluetracker.gg/wow/topic/us-en/9362996672-patch-54-addon-changes-saved-variables/)

## Remaining checks

Ask aso or use disposable native fixtures with exact build/profile identification
for shutdown event order/arguments and callback effects in the next saved file;
boolean/fractional keys, nil and unsupported values, shared/cyclic tables and
remaining metatables; all byte values and numeric edges; and per-scope failure
behavior. Known character/class context must precede initialization: folder names
alone do not establish every database key.
[AceDB key construction](https://github.com/WoWUIDev/Ace3/blob/ac3d1225037e0dc74e025a72342c0f7824aae147/AceDB-3.0/AceDB-3.0.lua#L251)

Crashes and forced termination cannot establish a completed save. Serializer
fixtures establish simulator data preservation, not native multi-file atomicity
or proof that production shutdown callbacks finished. Native parity remains a
separate evidence requirement.
