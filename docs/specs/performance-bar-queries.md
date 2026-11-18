# Retail PerformanceBar network queries

Retail 12.1.5 publishes the two legacy network getters used by the original PerformanceBar tooltip. Their native contract is based on the `pr46-49-decomp-answers.md` handoff, R88ap part B, sections 7–8; the simulator keeps unknown live endpoint state as empty host input. See the [PerformanceBar query system page](../wiki/systems/performance-bar-queries.md) for implementation details.

## What it must do

### Native getter contract

- [x] **VERIFIED (Retail 12.1.5.70688, R88ap.7):** `GetAvailableBandwidth()` returns exactly one Lua number, `0`, on its normal path. The original tooltip test confirms this with nonzero `GetNetStats()` input.
- [ ] **VERIFIED (Retail 12.1.5.70688, R88ap.7):** the normal getter path does not read connection state and does not emit a bandwidth update event; these effects are not directly asserted by the simulator test.
- [x] **VERIFIED (Retail 12.1.5.70688, R88ap.8):** `GetProtocolTypes()` returns exactly two non-nil Lua numbers, in endpoint order, and each result is `0`, `1`, or `2`.
- [ ] **VERIFIED (Retail 12.1.5.70688, R88ap.8):** unsupported endpoints return `0`, and only the second result can use an alternate endpoint. The host input supplies the resulting pair; endpoint selection is not modeled.
- [x] **VERIFIED (Retail 12.1.5.70688, R88ap.8):** when neither endpoint is supported, `GetProtocolTypes()` returns `(0, 0)`, not nil, no results, or a table.

### Simulator inputs and profile scope

- [x] **VERIFIED (simulator source and Retail test):** Retail 12.1.5 stores protocol codes in an empty-default `SimState` input. An absent input returns `(0, 0)`; the typed input permits only codes `0`, `1`, and `2`.
- [x] **VERIFIED (simulator source and Retail test):** `GetAvailableBandwidth()` returns `0` even when `GetNetStats()` has nonzero bandwidth values.
- [x] **VERIFIED (PTR, Forever, Mists tests):** profiles other than Retail 12.1.5 retain the previous fallback values.
- [ ] Wrath retains its prior fallback by source and compiles, but it is compile-only and has no runtime assertion.
- [x] **VERIFIED (Retail 12.1.5.70783 consumer test):** the original `Blizzard_PerformanceBar/PerformanceBar.lua` tooltip consumes both getters. The TOC selects this file.

**INFERRED (pinned Retail 12.1.5.70783 consumer):** `PerformanceBar.lua:44–45, 81–82` displays the first code as home and the second as world; codes 1 and 2 select TCP and UDP labels. These are tooltip labels, not evidence of live endpoint values or actual transport behavior.

## How it works

- [Retail PerformanceBar query system](../wiki/systems/performance-bar-queries.md).

## Implementation inventory

- `src/c_api/performance_bar.rs` — Retail 12.1.5 producers and protocol-code input type.
- `src/lua_api/env_init/mod.rs` — registers the Retail-only producer.
- `src/lua_api/state/sim_state.rs` and `src/lua_api/state.rs` — define and initialize the empty-default input.
- `src/lua_api/workarounds/temporary/inert_global_defaults.rs` — keeps the bandwidth fallback for other profiles.
- `src/lua_api/workarounds/temporary/performance_metric_defaults.rs` — keeps the protocol fallback for other profiles.
- `tests/micro_menu.rs` — drives the original PerformanceBar tooltip with supplied inputs.

## Tests asserting this spec

- `micro_menu::performance_bar_tooltip_uses_supplied_protocols_and_zero_bandwidth` — exact return shapes, supplied `(1, 2)` and `(2, 1)` pairs, absent-input `(0, 0)`, and original-tooltip rows.
- `micro_menu::performance_bar_fallbacks_preserve_other_profile_values` — PTR, Forever, and Mists retain `(256, 1, 1)` for nonzero network stats; Wrath remains compile-only.

## Known gaps (current cycle)

- [ ] Live endpoint values, their writers, and the update event remain UNKNOWN. The simulator does not derive them from connection state.

## Out of scope

- Section 5 `UnitInSubgroup` is excluded because its public link is UNKNOWN.
- The simulator does not model connection-state transitions or bandwidth updates.
