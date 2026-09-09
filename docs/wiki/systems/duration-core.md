# Duration core

`LuaDurationObject` now has simulator-owned start, base-duration, rate, and optional manual-clock state. The model makes ordinary duration queries and cooldown transfer observable while retaining native timing, modifier, and security behavior as open questions.

## Behavior

`SetTimeFromStart`, `SetTimeFromEnd`, and `SetTimeSpan` configure independent table proxies. Queries derive endpoints, total, elapsed, remaining, rate, zero, started, expired, and active state from the selected clock. Manual-clock advance and rewind update later observations without mutating configured timing.

The core stores `start`, base duration, and rate. Its rate/modifier formulas, zero-state rules, validation, default clock, and reset behavior are simulator choices documented in [duration core](../../specs/duration-core.md), not native WoW confirmation.

`FrameAPICooldown.SetCooldownFromDurationObject` now resolves proxy methods through Lua indexing, so ordinary duration transfer and reset work. Protected, secret, forbidden, and `clearIfZero = false` behavior remains unresolved.

## Sources

- [Duration core spec](../../specs/duration-core.md) — modeled contract and explicit assumptions.
- [`core.rs`](../../../src/lua_api/globals/lua_duration_object/core.rs) — state and query implementation.
- [`duration_core.rs`](../../../tests/duration_core.rs) — focused ordinary behavior proof.
- [`cooldown_widget.rs`](../../../tests/cooldown_widget.rs) — bounded proxy consumer proof.

## See Also

- [[patch-12-1-5-api-audit]] — exact changed API occurrences and evidence boundary.
- [Duration text binding](../../specs/duration-text-binding.md) — separate consumer path.
