# PTR I4 GameRules supplied inputs

This bounded PTR 12.1.7.70493 packet exposes six GameRules queries/mutations from supplied state and finite source-published enums. It does not establish native capability policy. See the [I4 investigation](../wiki/investigations/patch-12-1-7-api-audit.md).

## What it must do

- [x] Publish `IsHardcoreActive`, `IsSelfFoundAllowed`, `IsSDHDToggleEnabled` and `AccountHasSDEnabled` only on PTR in this packet. Each returns exactly one boolean from its distinct supplied field. Initial false means no positive input was supplied, not a native account default.
- [x] PTR `IsWoWHack` returns exactly one boolean, true only when existing supplied `active_game_mode` equals the existing `GameMode.WoWHack` enum declaration. Original MicroMenu OnLoad assigns mode-specific button parents and layout indices; original UIErrorsFrame OnLoad enables energy errors and adjusts its anchor only in that mode. Independent environments do not share state.
- [x] `SetSDHDToggleValue(setToSD, usageLocation)` updates supplied account SD selection and records the supplied origin, returning no values. The original Barbershop click mutates state before the additional return-arity assertion.
- [x] Original selected Barbershop, CharacterCreate and popup consumers react to changed supplied inputs. Retail publication remains unchanged.
- [x] Publish RecentAlliesInteractionCategoryFilter and metadata exactly as pinned documentation declares. The original six labels and unknown-value result remain reachable. Publish the documented SdToggleUsageLocationUI argument enum and metadata without extrapolating values.

## How it works

- [I4 source and runtime investigation](../wiki/investigations/patch-12-1-7-api-audit.md#i4-gamerules-and-recentallies-prerequisites)

## Implementation inventory

- `src/lua_api/sim_substates/mod.rs`: supplied capability and SD-origin fields; existing active-mode state.
- `src/c_api/ptr_i4_game_rules.rs`: PTR declarations, queries and supplied SD mutation.
- `src/c_api/mod.rs`: module declaration.
- `src/lua_api/globals/game_rules.rs`: registration calls the C API packet.

## Tests asserting this spec

- `tests/ptr_i4_batch2.rs`: original selected game/glue consumers, transitions, return shapes, environment isolation and Retail absence.

## Known gaps (current cycle)

- [x] Final merged-source GREEN and producer-only RED pass their respective exact eight-consumer boundaries with identical test bytes. Retail absence passes. The fresh stock comparison is 296→272 raw keys and 410→378 occurrences, with all ten added keys traced individually. Retail/PTR all-target checks meet zero/known-three warning limits; formatting passes.
- [ ] Native capability, eligibility, restrictions and mode-selection policy remain unknown. Existing RecentAllies support-query, Edit Mode initialization and chat-docking gaps remain visible in stock startup.

## Out of scope

GraphicsOverrides preset support is a separate follow-up. Its original callback was not reached: missing CHARACTER_MODELS prevented valid proxy registration. No preset producer, enum, backing field or deliberately failing preset test is included here. Aggregate I4 tests remain on the investigation lane, not in the normal landing suite. Native capability derivation, mode selection, HD/SD eligibility, telemetry, persistence, argument restrictions, secret handling, SmartNavigation and additional RecentAllies APIs are not established.
