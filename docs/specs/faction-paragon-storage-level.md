# Faction paragon storage level

`C_Reputation.GetFactionParagonInfo` exposes seeded per-faction state through `src/c_api/c_reputation.rs`. The pinned [12.0.0 contract](../../data/patch-api/sources/12.0.0-register.json) adds numeric `paragonStorageLevel` after the existing five returns.

## What it must do

- [x] Retail 12.0.0 and later return exactly six values for a seeded faction: current value, threshold, reward quest ID, pending reward flag, low-level flag, and storage level.
- [x] Preserve numeric types for returns 1–3 and 6 and boolean types for returns 4–5; preserve existing first-five values.
- [x] Return the explicitly seeded integer storage level, including nonzero values; reflect updates independently per faction without deriving storage from current reputation.
- [x] Return zero values for an unknown faction.
- [x] Profiles without the retail 12.0.0 API epoch retain exactly five values.

## How it works

- [C API architecture](../wiki/systems/c-api.md)

## Implementation inventory

- `src/c_api/c_reputation.rs` — paragon payload and state-backed getter.
- `src/c_api/mod.rs` — namespace module publication.
- `src/lua_api/globals/faction_probes.rs` — existing namespace registration delegates to the getter.
- `src/lua_api/state/support_types.rs` — preserves existing public payload import via re-export.
- `src/lua_api/state/sim_state.rs` — existing per-faction state map.

## Tests asserting this spec

- `tests/c_reputation_paragon.rs` — exact arity, no-result queries, seeded values/types, mutation, and faction isolation in the grouped integration target.
- `tests/blizzard_ui/blizzard_actionbar/behavior_paragon_tooltip.rs` and `behavior_reputation_bar_update.rs` — existing first-five consumer fixtures remain source-compatible with the added field; not sixth-return consumer proof.

## Known gaps (current cycle)

Independent proof at `9a8c05992`: ten paragon tests passed on 12.0.7; the Mists paragon/class-selection batch passed ten tests. Two first-five Blizzard consumer assertions passed, but their historical-profile loader emitted errors whose regression status remains unestablished; clean historical UI loading is not claimed. Format/check/build passed and current-retail startup emitted `[]`.

## Out of scope

Gameplay reputation earning, storage derivation, native argument validation/coercion, secure access, and native fidelity are not established by seeded-state queries. Real Blizzard consumption of the sixth return remains unproven. No other reputation subsystem behavior changes.
