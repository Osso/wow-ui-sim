# 12.0.7 colors

Proposed bounded contract for `prose-undated-010`, `global api-C_EncounterTimeline-GetEventColor-033`. Source: [retained API excerpt](../../data/patch-api/sources/12.0.7-api-changes.txt). Later cached declarations may postdate 12.0.7; no native historical proof.

## What it must do

- [ ] Copy RGBA including alpha and reflect two distinct event IDs and live edits from secure/tainted public callers without aliasing returned/input tables.
- [ ] Legacy timeline bridge reads updated four-component RGBA tuple when retail-12-1-5 is absent; no implied parity with later ColorMixin contract.

## How it works

- [Lua API architecture](../lua-api.md)

## Implementation inventory

- src/lua_api/globals/missing_surface/encounter_events.rs:42–66,112–145,264–303 — Lua-rooted mutable color state and legacy bridge.
- src/c_api/c_encounter_timeline/visuals.rs — later severity-derived ColorMixin return; does not read configured overrides.

## Tests asserting this spec

`tests/p1207_b16_colors.rs` — Two tests predicted PASS by reading under strict 12.0.7; first also applies to later builds, legacy tuple test is excluded under retail-12-1-5. Fixed empty/white colors fail concrete RGBA values; aliasing fails input/output edits, leaked global state fails independent environment. With color producers withheld, positive overrides/legacy tuple fail. No existing expectations change, no code edits proposed for this already-implemented subset.

Source-reading predictions only; no test execution or accepted coverage.

## Known gaps (current cycle)

- [ ] Integrate and run staged tests under strict retail-12-0-7.
- [ ] Later cache GetEventColor(encounterEventID, trigger) and SetEventColor(encounterEventID, trigger, color) differ from current two-argument setter/one-argument getter. Cache timeline getter returns ColorMixin, whereas strict bridge returns four components; epoch protocol not established. Later event instance ID is not proven identical to catalog encounterEventID. Never silently select a white default or claim configured alpha survives the later static severity colors. AllowedWhenUntainted authentication and NotAllowed secret rejection are unproven, not claimed. No producer edit is proposed against an unpinned historical signature.

## Out of scope

Native parity, automatic server synchronization, production datasets, vendor/cache edits and adjacent API rows.
