# CombatText active-unit state

Pinned `data/patch-api/sources/12.0.0-register.json` declares `C_CombatText.GetActiveUnit() -> string?` and `SetActiveUnit(unitToken: UnitToken) -> no returns`. This slice models explicit selection only.

## What it must do

- [ ] Retain copied explicit `player`/`vehicle` strings, including repeated writes, independently per environment.
- [ ] Return one nil/string from the getter and zero values from the setter.
- [ ] Require a string through the existing strict string conversion helper; no guessed token validation.

Initial `None` is simulator policy, not an established native default. Current proof begins after explicit writes; initialization and invalid-input behavior remain untested.

## How it works

- [C API audit architecture](../wiki/systems/patch-api-audit-manifest.md)

## Implementation inventory

- `src/c_api/c_combat_text.rs`: merged namespace getter/setter registration.
- `src/c_api/mod.rs`, `src/c_api/registration.rs`: `retail-12-0-0` gated publication.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs`: per-environment optional string and initialization.

## Tests asserting this spec

Three `combat_text_active_unit_*` tests in `tests/c_namespace_noop_replacements.rs`, committed at `8113cc6bc`, reached RED 0/3: getter returned nil after a write and setter returned nonzero values, without establishing its exact arity. Runtime `375c71192` passes 3/3 each on retail 12.0.0 (exact-byte reuse), 12.0.5 and 12.0.7 in `/tmp/verify-combat-text-active-unit-ledger.json`. Final metadata proof at `c099a68e6` records 15,035 fresh hashes, zero stale, 95 renewals, eight additions, two credits, validator exit 0 and all 3,410 rows matching. Totals: **2339 / 1069 / 2**; snapshot: **1,095 rows / 282 plans**. Ledger: `/tmp/verify-combat-text-active-unit-metadata-ledger.json`.

## Known gaps (current cycle)

- [ ] Native initial selection, token validation, identity and lifecycle semantics.
- [ ] Loaded consumer execution and combat-text routing/events.

## Out of scope

`GetCurrentEventInfo`, security/secret behavior, token normalization and event producers are unchanged; explicit `player`/`vehicle` state proof cannot establish them.
