# Retail PerformanceBar network queries

The Retail 12.1.5 simulator registers the two network getters consumed by the original PerformanceBar tooltip. Native getter behavior comes from R88ap part B, sections 7–8; live endpoint values and their update path remain unknown.

## Content

- **VERIFIED (Retail 12.1.5.70688, R88ap.7):** `GetAvailableBandwidth()` returns one numeric zero on its normal path and does not read bandwidth or connection state.
- **VERIFIED (Retail 12.1.5.70688, R88ap.8):** `GetProtocolTypes()` returns two numeric endpoint codes in order. Unsupported endpoints return zero; supported results use codes 1 or 2; only the second can use an alternate endpoint.
- **VERIFIED (simulator source):** `SimState.performance_bar_protocol_types` is an optional typed pair. No input returns `(0, 0)`. The producer does not derive live values or dispatch an update event.
- **VERIFIED (simulator source):** the Retail producer replaces the former bandwidth and protocol fallbacks. Those fallback defaults remain enabled for other profiles.
- **UNKNOWN:** live protocol values, their writers, and the update event.

The original tooltip consumer is `Blizzard_PerformanceBar/PerformanceBar.lua:81–93` in the pinned Retail 12.1.5.70783 cache. The R88ap native return evidence is from build 70688; this does not claim the caches are byte-equivalent.

## Sources

- [PerformanceBar query contract](../../specs/performance-bar-queries.md) — native contract and simulator acceptance tests.
- `pr46-49-decomp-answers.md`, R88ap part B, sections 7–8 — native return facts and unknown state boundary.

## See Also

- [[lua-api]] — API surface and compatibility models.
