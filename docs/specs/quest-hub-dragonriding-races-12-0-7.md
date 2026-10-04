# Quest-hub dragonriding races — Retail 12.0.7 row048

`C_QuestHub.GetDragonridingRacesForAreaPOI` reads explicit per-environment host race identifiers instead of an unconditional empty table. [Source row048](../../data/patch-api/sources/12.0.7-api-changes.txt) names the API but supplies no signature. Every policy below is **INFERRED**, authorized by the project owner, not historical/native parity.

## Evidence, in priority order

1. Cached retail `Blizzard_APIDocumentationGenerated/QuestHubInfoDocumentation.lua:10–18` declares sibling `Name = "IsAreaPOICurrentlyRelatedToHub"`, `SecretArguments = "AllowedWhenUntainted"`, and `{ Name = "areaPoiID", Type = "number", Nilable = false }`. **INFERRED**: same required numeric selector and security policy for this getter; authenticate all arguments and extras before validation.
2. Whole `~/.cache/wow-ui-sim/blizzard-ui` Lua scan found no exact getter declaration or caller. No consumer supplies field names or contradicts the inferred shape.
3. Cached retail `Blizzard_APIDocumentationGenerated/AreaPoiInfoDocumentation.lua:76–89`, `GetDragonridingRacesForMap`, says `Documentation = { "Returns all area POIInfos flagged as dragonriding races for the given map." }` and returns `{ Name = "areaPoiIDs", Type = "table", InnerType = "number", Nilable = false }`. **INFERRED**: this getter likewise returns numeric race POI identifiers, not fabricated element tables/fields. Host race records are ordered numeric IDs.
4. [Handoff row048](../../data/patch-api/evidence/12.0.7-session-2026-10-03/handoff-p1207-remaining.md) says “replace the probe with a per-environment POI/content input and live Rust getter once schema is known” and “No cached consumer of this exact getter was located.” Owner overrides that prior block by authorizing inference. Empty/unknown defaults, order, detached snapshots and isolation are **INFERRED simulator policies**, not cache declarations.

## What it must do

- [x] **INFERRED:** return exactly one nonnil array table, empty for unconfigured/unknown POIs.
- [x] **INFERRED:** read ordered numeric race POI IDs live from explicit host state; replacement/removal affects the next call without affecting previous results.
- [x] **INFERRED:** return fresh detached tables; Lua mutation cannot alter host input or subsequent results. Environments remain independent.
- [x] **INFERRED:** require one actual numeric `areaPoiID`; represent identifiers as finite integral signed-i32 values, rejecting missing/nil, other types, coercion, fractions, nonfinite and out-of-range values.
- [x] **INFERRED AllowedWhenUntainted:** authenticate every argument/extra with VM `unwrap_secret` before selector validation. Secure callers may supply secrets; tainted callers may supply public values but must be denied secret selectors/extras without changing taint. Authenticated extras are ignored.

## How it works

- [Lua API](../lua-api.md)

## Implementation inventory

- `src/c_api/c_quest_hub.rs` — gated Rust producer and authenticated selector.
- `src/lua_api/state/sim_state.rs` — per-POI ordered numeric race records.
- `src/lua_api/state.rs` — empty host map initialization.

## Tests asserting this spec

- `tests/p1207_dragonriding_races.rs` — seven behavioral cases through `WowLuaEnv`.

## Known gaps (current cycle)

- [ ] Exact historical declaration, populated native results and native security proof unavailable. All policies remain inferred even when simulator tests pass.

## Out of scope

- Automatic race discovery, race metadata, quest derivation, historical/native parity and other-profile proof.
- No cache/vendor/wiki/coverage edits; registration gate unchanged.
