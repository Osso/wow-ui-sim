# Neighborhood tracked tasks

`C_NeighborhoodInitiative` exposes per-environment tracked-ID membership through `src/c_api/c_neighborhood_initiative.rs`, gated at retail 12.0.0. See the [audit inventory](../wiki/investigations/patch-api-blocker-inventory.md) for remaining initiative obligations.

## What it must do

- [x] Add and remove numeric task IDs with zero return values.
- [x] Return one fresh `{ trackedIDs = array }` table, independent of caller mutations.
- [x] Keep membership independent across environments and preserve other namespace providers.
- [ ] Start empty, ignore duplicate adds and unknown removals, and enumerate sorted IDs. These are simulator policies, not established native behavior.

## How it works

- [C API architecture](../lua-api.md)

## Implementation inventory

- `src/c_api/c_neighborhood_initiative.rs`: membership handlers using the existing namespace table.
- `src/c_api/mod.rs`, `src/c_api/registration.rs`: retail 12.0.0 registration.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs`: per-environment integer-ID set and initialization.
- `src/lua_api/workarounds/temporary/tracking_namespace_defaults.rs`: retains unrelated defaults, no tracking-method definitions.

## Tests asserting this spec

`tests/c_namespace_noop_replacements.rs`: three `neighborhood_tracked_tasks_*` tests committed at `3abb84e1c` reached RED 0/3 on empty membership after additions. The first runtime attempt, `d6d48d1e5`, stopped at compiler error E0308 before tests ran; corrected runtime `27223f82a` reached focused 12.0.0 GREEN 3/3. Ledger: `/tmp/neighborhood-tracked-tasks-green-fixed-ledger.json`.

## Known gaps (current cycle)

- [ ] Independent verification beyond focused development GREEN (agent 18851 pending).

## Out of scope

Task records (including `InitiativeTaskInfo.tracked`), events, persistence, hyperlinks and consumer execution are not modeled by this slice. Native ordering, initial state, duplicate/unknown-ID semantics, ID validity, numeric coercion/ranges and errors remain unproven. Numeric arguments use existing simulator integer-ID conversion conventions; no additional validation policy is introduced.
