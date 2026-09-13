# Combat-log setting state

This slice verifies existing ordinary `C_CombatLog` setting storage. It does not add runtime behavior or model combat-log filtering/retention enforcement.

## Contract under test

- `SetFilteredEventsEnabled(bool)` roundtrips explicit true/false, including repeated writes, through `AreFilteredEventsEnabled()`.
- `SetEntryRetentionTime(number)` roundtrips explicit numeric settings through `GetEntryRetentionTime()`.
- Getters return one appropriately typed value; setters return no values.
- The two settings are independent and isolated between simulator environments.

## Evidence

Five grouped tests in `tests/c_namespace_noop_replacements.rs`, committed `7bd7f10ca`, initially pass 5/5 on retail 12.0.0. This is an existing-behavior audit, not a manufactured RED/GREEN bug fix. Values exercised include true/false and retention settings 120/240. Independent cross-profile proof remains pending.

Existing implementation: `src/lua_api/workarounds/temporary/combat_log_state.rs`. Stored settings are part of a temporary combat-log fixture, not a complete combat history model.

## Gaps

Filtering effects, pruning, clocks, defaults, invalid-input/coercion/error behavior, events, reset/persistence/lifecycle and native consumer behavior remain unproven. Restriction/secret/taint enforcement is deferred. No broader combat-log implementation or earlier-profile availability claim follows from setting-state tests.
