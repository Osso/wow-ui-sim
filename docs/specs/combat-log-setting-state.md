# Combat-log setting state

This slice verifies existing ordinary `C_CombatLog` setting storage. It does not add runtime behavior or model combat-log filtering/retention enforcement.

## Contract under test

- `SetFilteredEventsEnabled(bool)` roundtrips explicit true/false, including repeated writes, through `AreFilteredEventsEnabled()`.
- `SetEntryRetentionTime(number)` roundtrips explicit numeric settings through `GetEntryRetentionTime()`.
- Getters return one appropriately typed value; setters return no values.
- The two settings are independent and isolated between simulator environments.

## Evidence

Five grouped tests in `tests/c_namespace_noop_replacements.rs`, committed `7bd7f10ca`, initially pass 5/5 on retail 12.0.0. This is an existing-behavior audit, not a manufactured RED/GREEN bug fix. Values exercised include true/false and retention settings 120/240.

Follow-up `3d69ec6ea` writes distinct settings in both environments: first `(true, 180)`, then second `(true, 120)` and `(false, 60)`, asserting neither write leaks across environments. Independent `/tmp/verify-combatlog-settings-isolation-ledger.json` records fresh 5/5 each on retail 12.0.0/12.0.5/12.0.7, fmt/readability PASS and exact unchanged-runtime gate reuse. Metadata verification at `19eab5ef4` records 14,933 fresh hashes, zero stale, zero renewals, 16 additions, four credits and validator exit 0 with 3,410 matching rows. This is stored-setting proof only; it does not alter runtime behavior or the initial 5/5 evidence. Four bounded audit credits do not attach the earlier broad fixture tests as native proof.

Existing implementation: `src/lua_api/workarounds/temporary/combat_log_state.rs`. Stored settings are part of a temporary combat-log fixture, not a complete combat history model.

## Gaps

Filtering effects, pruning, clocks, defaults, invalid-input/coercion/error behavior, events, reset/persistence/lifecycle and native consumer behavior remain unproven. Restriction/secret/taint enforcement is deferred. No broader combat-log implementation or earlier-profile availability claim follows from setting-state tests.
