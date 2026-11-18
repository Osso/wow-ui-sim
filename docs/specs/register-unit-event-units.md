# RegisterUnitEvent unit arguments (R88aj)

`Frame:RegisterUnitEvent(event, ...)` unit-argument handling lives in `src/lua_api/frame/methods/text_attribute_event/`. The native answer is `pr46-49-decomp-answers.md` (untracked decomp handoff), section `R88aj`. The scan rules are VERIFIED for Mists 5.5.4.70631, Retail 12.1.5.70688 and Forever 1.60.1.70707. Retail and Forever secret-value state and forbidden-aspect lookup inputs remain UNKNOWN and empty by default; this contract does not invent their values or error text. This supersedes the two-unit bounded model in [frame unit-event filter](frame-unit-event-filter.md).

## What it must do

- [x] **VERIFIED (R88aj aj1/aj2/aj3):** the handler reads at most four unit positions after the event and stops at the first value that is neither a string nor a number. Valid units before that value stay in the filter. A table at the first unit position gives zero units; the handler does not read table contents or later arguments.
- [x] **VERIFIED (R88aj aj1/aj2):** zero valid units register the event with an empty filter, the same as `RegisterEvent`, without an error.
- [x] **VERIFIED (R88aj aj2):** unknown unit strings are skipped without error and the scan continues; each still uses one of the four positions. Mists 5.5.4.70631 specifically rejects `boss6`–`boss8`. Retail and Forever known-token lists are UNKNOWN. Unit tokens are case-insensitive.
- [x] **VERIFIED (R88aj aj4):** each successful call returns exactly one boolean: true when the frame was not registered, or when the call changed the unit filter (position by position); false when the filter is identical, including an empty filter registered again with zero units.
- [x] **VERIFIED (R88aj aj5):** a second call replaces the filter (no merge). `RegisterEvent` after `RegisterUnitEvent` clears the filter and returns true.
- [x] **PRESERVATION:** `UnregisterEvent` removes the registration and its filter. This is the pre-existing simulator behaviour, already covered on all profiles by `tests/unit_event_second_unit.rs:61-66`. VERIFIED natively only for Mists 70631 (R88aj aj5); UNKNOWN for Retail 70688 and Forever 70707 (not decoded).
- [x] **VERIFIED preservation consumer:** the selected Mists `Blizzard_ChatFrameBase_Classic.toc` loads `Shared/CastSequenceManager.lua`, whose original consumer registers `UNIT_SPELLCAST_*` for `player` and `pet`. Its original cast sequence advances for those units and does not advance for `target` or `focus`.

## How it works

- [Event system](../event-system.md)

## Implementation inventory

- `src/lua_api/frame/methods/text_attribute_event/unit_event.rs` — the four-position scan.
- `src/lua_api/frame/methods/text_attribute_event/events.rs` — `RegisterUnitEvent` return and `RegisterEvent` filter clearing.
- `src/widget/registry/mod.rs` — filter replacement and change detection; `src/widget/frame.rs` stores a unit list per event.
- `src/lua_api/script_helpers/event_dispatch.rs` — first-payload match against any stored unit.

## Tests asserting this spec

- `tests/register_unit_event_units.rs` — return-value and filter-replacement cases using the Mists-decoded unit-token evidence.
- `tests/mists_r88aj.rs::register_unit_event_scans_native_unit_argument_boundaries` — five-position, unknown-token continuation, non-string stop and table-first boundaries; `boss6` is rejected in Mists.
- `tests/mists_r88aj.rs::cast_sequence_consumer_receives_only_registered_units` — selected original Mists `CastSequenceManager` consumer with real spellcast events.
- `tests/mists_r88aj.rs::combat_audio_manager_table_registers_unit_health_with_empty_filter` — original table-first consumer path.

## Known gaps (current cycle)

- [ ] **VERIFIED, not modelled:** the usage error for a missing or non-string event (`Usage: ("event", "unit1" [,"unit2", "unit3", "unit4"])` with the `%s:%s(): %s` prefix) and the unit-event set check; the simulator keeps its previous event validation.
- [ ] **VERIFIED (R88aj):** the full Mists known-token list, strict event-name and unit-event validation, deferred registration, tainted zero-unit special returns, secret-value conversion and Retail/Forever post-return routine remain outside this implementation slice.
- [ ] **UNKNOWN (Retail/Forever):** secret-value gate setting and forbidden-aspect lookup/source values remain empty by default; no native setting or aspect name is invented.
- [ ] **UNKNOWN (R88aj):** Retail and Forever known-token and unit-event lists; no native token-list parity is claimed for those profiles.
- [ ] **VERIFIED (R88aj aj1), not modelled:** event names are not case-sensitive natively; the simulator matches event names case-sensitively.
