# Combat-log history state

Retail and WoW Forever expose a shared Rust-backed history model in `src/c_api/combat_log_history.rs`. The model preserves the prior temporary fixture's Lua-visible behavior as **INFERRED** simulator behavior; it does not claim native parity. See the [combat-log history system](../wiki/systems/combat-log-history.md).

## What it must do

- [ ] Start with no history. Do not create combat events automatically; accept records only through explicit host input.
- [ ] Preserve the prior fixture's empty results: count `0`, one `nil` from secure current-entry/current-event queries, `false` from empty seeks, and `false` from `ShouldShowCurrentEntry`.
- [ ] Return each supplied record's values in order. Start traversal at the newest record and move to the previous record when the secure processor requests it.
- [ ] Preserve the prior fixture's behavior when a previous seek reaches the oldest record: return `false` and leave the cursor on that record.
- [ ] Keep retention time, message limit, filtered-event enablement and current-entry state isolated per simulator environment. Defaults `300`, `300`, `false` and `0` are **INFERRED** from the previous fixture.
- [ ] Run the selected original Blizzard_CombatLogProcessor against empty history and at least two distinct host records on Retail and Forever.
- [ ] Keep other profiles on their existing compatibility shims.

## How it works

- [Combat-log history system](../wiki/systems/combat-log-history.md)
- [Combat-log event filter records](combat-log-event-filters.md)

## Implementation inventory

- `src/c_api/combat_log_history.rs`: Rust-owned Retail/Forever history records, cursor, settings and C API publication.
- `src/lua_api/globals/missing_surface.rs`: registers the model during the runtime-surface bootstrap.
- `src/lua_api/state/sim_state.rs`: per-environment history ownership.
- `src/lua_api/workarounds/temporary/combat_log_state.rs`: non-history compatibility helpers and other-profile shims.
- `tests/combat_log_history.rs`: original processor test for empty and supplied history.

## Tests asserting this spec

`tests/combat_log_history.rs::original_processor_refilters_empty_and_host_supplied_history` covers the empty result shapes and two distinct records through the original mainline processor. Exact Retail and Forever results are recorded in the lane report.

## Known gaps (current cycle)

- [ ] Native empty-query arity, cursor boundary behavior and filter matching remain unproved; the implemented contract is fixture parity only.
- [ ] Native event ingestion is not modeled. The simulator uses explicit host-supplied records.
- [ ] Retention pruning, message-limit enforcement, event-filter matching, restrictions, taint and secret-value behavior are not modeled by this history slice.

## Out of scope

Automatic combat events and real-client parity for the inferred fixture behavior are out of scope. Open R88bt questions are listed in the lane report.
