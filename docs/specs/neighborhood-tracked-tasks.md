# Neighborhood tracked tasks

`C_NeighborhoodInitiative` exposes per-environment tracked-ID membership through `src/c_api/c_neighborhood_initiative.rs`, enabled at retail 12.0.0 and for Forever. Forever's generated `NeighborhoodInitiativeDocumentation.lua` declares all three reused methods and the non-nil numeric `trackedIDs` array. See the [audit inventory](../wiki/investigations/patch-api-blocker-inventory.md) for remaining initiative obligations.

## What it must do

- [x] Add and remove numeric task IDs with zero return values.
- [x] Return one fresh `{ trackedIDs = array }` table, independent of caller mutations.
- [x] Keep membership independent across environments and preserve other namespace providers.
- [ ] Start empty, ignore duplicate adds and unknown removals, and enumerate sorted IDs. These are simulator policies, not established native behavior.

## How it works

- [C API architecture](../lua-api.md)

## Implementation inventory

- `src/c_api/c_neighborhood_initiative.rs`: membership handlers using the existing namespace table.
- `src/c_api/registration.rs`: registration call; the membership module owns the single profile-capability decision.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs`: per-environment integer-ID set and initialization.
- `src/lua_api/workarounds/temporary/tracking_namespace_defaults.rs`: retains unrelated defaults, no tracking-method definitions.

## Tests asserting this spec

`tests/c_namespace_noop_replacements.rs`: three `neighborhood_tracked_tasks_*` tests committed at `3abb84e1c` reached RED 0/3 on empty membership after additions. The first runtime attempt, `d6d48d1e5`, stopped at compiler error E0308 before tests ran; corrected runtime `27223f82a` reached focused 12.0.0 GREEN 3/3. Independent proof reuses exact 12.0.0 bytes and passes 3/3 fresh on 12.0.5 and 12.0.7, plus fmt/check/build/startup/readability. Runtime ledger: `/tmp/verify-neighborhood-tracked-tasks-ledger.json`. Metadata proof at `71209241a`: 15,027 fresh hashes, zero stale, 151 renewals, 16 additions, validator exit 0 and 3,410 matching rows. Ledger: `/tmp/verify-neighborhood-tracked-tasks-metadata-ledger.json`.

## Out of scope

Forever's actual tracker initialization, empty layout, entering-world/zone events, and untracking are exercised in the grouped membership tests. Task records (including `InitiativeTaskInfo.tracked`), emitted events, persistence, hyperlinks and populated layout remain outside this slice. Native ordering, initial state, duplicate/unknown-ID semantics, ID validity, numeric coercion/ranges and errors remain unproven. Numeric arguments use existing simulator integer-ID conversion conventions; no additional validation policy is introduced.
