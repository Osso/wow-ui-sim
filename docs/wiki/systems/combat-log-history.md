# Combat-log history

This page records the Retail/Forever simulator history model. The public contract is in the [combat-log history spec](../../specs/combat-log-history-state.md); this page describes the current implementation and evidence limits.

## Current model

- **VERIFIED (simulator source):** `SimState.combat_log_history` owns records, cursor, current-entry index, retention time, filtered-event enablement, message limit and an optional `rilua::Val` handle for filter settings on Retail and WoW Forever. The Lua compatibility table also keeps the same filter-settings value for existing script consumers.
- **INFERRED (simulator mapping):** each environment starts with empty history, retention time `300`, message limit `300`, filtered events disabled and current-entry index `0`.
- **INFERRED (host input):** the host supplies an ordered vector of primitive event values. The simulator does not synthesize combat events.
- **INFERRED (fixture parity):** `ApplyFilterSettings` retains the supplied table value by identity in the compatibility state and Rust model. It does not prove native settings semantics.
- **INFERRED (fixture parity):** an empty secure current-entry/current-event query returns one `nil`; empty newest/previous seeks return `false`; `ShouldShowCurrentEntry` returns `false` when no records exist.
- **INFERRED (fixture parity):** a query with records returns the current record's values in order. An unset or invalid cursor selects the newest record. A previous seek moves toward older records; a seek at the oldest returns `false` and keeps that cursor.
- **INFERRED (consumer test setup):** the original Blizzard_CombatLogProcessor consumes the supplied values newest-to-oldest. The test replaces only its message formatter so the record traversal does not depend on unrelated spell/text catalogs.

## Profile scope

- **VERIFIED (pinned TOCs):** Retail and Forever select `Blizzard_CombatLogProcessor` for `mainline`; the addon is non-LoadOnDemand and is in Game-screen discovery.
- **VERIFIED (simulator source):** Rust history registration is limited at runtime to Retail and WoW Forever. Other profiles keep their existing temporary state implementation.
- **INFERRED (simulator scope):** this model preserves the current fixture behavior. It does not establish Retail or Forever native history behavior.

## Open questions and gaps

- **UNKNOWN (R88bt):** exact native return arity and values for empty current-entry/current-event queries; cursor boundary behavior; visibility without entries; filter-settings schema and default; filter matching and combination; event ingestion; retention units/pruning; message-limit enforcement; and reset/lifecycle behavior.
- **UNKNOWN:** restriction/taint behavior and secret-value handling.
- **VERIFIED (simulator scope):** no combat event producer is connected to this model; host input must supply records explicitly.

## Sources

- [Combat-log history contract](../../specs/combat-log-history-state.md)
- [Rust C API model](../../../src/c_api/combat_log_history.rs)
- [Runtime registration](../../../src/lua_api/globals/missing_surface.rs)
- [Per-environment state](../../../src/lua_api/state/sim_state.rs)
- [Temporary profile bootstrap](../../../src/lua_api/workarounds/temporary/combat_log_state.rs)
- [Original-consumer test](../../../tests/combat_log_history.rs)
## See Also

- [[event-system]] — event dispatch used by the original processor.
- [[lua-api]] — C API registration and the Lua boundary.
