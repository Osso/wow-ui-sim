# Special-event brawl query

## What it must do

- [ ] Retail `C_PvP.GetSpecialEventBrawlInfo()` returns one value: nil when no special-event brawl record is supplied, otherwise an independent `PvpBrawlInfo` snapshot.
- [ ] The supplied special-event record is independent of `GetActiveBrawlInfo`. It includes the documented brawl identity, level, queue, description, map and reward-display fields; it has no invented active flag.
- [ ] The original PvP honor frame uses the record to show or hide its special-event brawl button and display its supplied name.

## Best-guess basis

Retail `Blizzard_APIDocumentationGenerated/PvpInfoDocumentation.lua:775` declares the nullable `PvpBrawlInfo`; `Blizzard_PVPUI/Mainline/Blizzard_PVPUI.lua:1110` consumes it. WM23-misc build 70688 identifies the record serializer and conditionally emulates nil/present publication. The host-supplied record represents the result of the native current-event ID and catalog lookup; its absence is a simulator scenario, not a claim that every native inactive state is identical.

## Implementation inventory

- `src/c_api/c_pvp/active_brawl.rs`: Retail special-event registration and shared brawl serialization.
- `SimState.special_event_brawl`: explicit record, independent of active-brawl input.

## Tests asserting this spec

- `tests/special_event_brawl.rs`: selected Retail PvP addon consumer and supplied absent/present/absent records. Runtime reachability and RED/GREEN are pending.

## Known gaps

- [ ] Current-event selection, database derivation, transitions, native live publication and other profile behavior remain unverified. The simulator does not derive this record from PvpBrawl or LFG catalogs.
