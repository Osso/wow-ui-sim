# Special-event brawl query

## What it must do

- [x] Retail and Forever `C_PvP.GetSpecialEventBrawlInfo()` returns one value: nil when no special-event brawl record is supplied, otherwise an independent `PvpBrawlInfo` snapshot.
- [x] The supplied special-event record is independent of `GetActiveBrawlInfo`. It includes the documented brawl identity, level, queue, description, map and reward-display fields; it has no invented active flag. Minimum item level and optional time remaining are integer-typed inputs.
- [x] The original PvP honor frame uses the record to show or hide its special-event brawl button and display its supplied name.

## Best-guess basis

Retail `Blizzard_APIDocumentationGenerated/PvpInfoDocumentation.lua:775` declares the nullable `PvpBrawlInfo`; `Blizzard_PVPUI/Mainline/Blizzard_PVPUI.lua:1110` consumes it. WM23-misc build 70688 identifies the record serializer and conditionally emulates nil/present publication. The host-supplied record represents the result of the native current-event ID and catalog lookup; its absence is a simulator scenario, not a claim that every native inactive state is identical.

## Implementation inventory

- `src/c_api/c_pvp/active_brawl.rs`: Retail and Forever special-event registration and shared brawl serialization.
- `SimState.special_event_brawl`: explicit integer-typed special-event record, independent of active-brawl input.

## Tests asserting this spec

- `tests/special_event_brawl_api.rs`: API-level absent/present/absent and integer-field coverage. The exact selector failed on the missing member at `8fae33884` and passed 1/0 at `9b137ba4f`; this does not prove the original consumer.
- `tests/special_event_brawl.rs`: the original HonorFrame consumer passed absent/present/absent at `b94c49535`; producer-only RED `0b449e39d` reached the missing query at `Blizzard_PVPUI.lua:1110`.

## Known gaps

- [ ] Current-event selection, database derivation, transitions, native live publication and other profile behavior remain unverified. The simulator does not derive this record from PvpBrawl or LFG catalogs.
