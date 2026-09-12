# Duration core

`LuaDurationObject` is an existing Lua table proxy with simulator-owned start, base-duration, rate, and optional manual-clock state. It makes ordinary duration queries and cooldown transfer observable while retaining native timing, modifier, and security behavior as open questions.

## Behavior

`SetTimeFromStart`, `SetTimeFromEnd`, and `SetTimeSpan` configure independent table proxies. Queries derive endpoints, total, elapsed, remaining, rate, zero, started, expired, and active state from the selected clock. Manual-clock advance and rewind update later observations without mutating configured timing.

The core stores `start`, base duration, and rate. Its rate/modifier formulas, zero-state rules, validation, default clock, reset behavior, and percent semantics are simulator choices documented in [duration core](../../specs/duration-core.md), not native WoW confirmation.

`24fe9d746` replaces the ordinary zero placeholders for `GetElapsedPercent` and `GetRemainingPercent`. Both derive fractions in `[0,1]` from the existing clamped elapsed/remaining clock state: before start and after end clamp to the boundary, and clock rewind changes later observations without mutating configured timing. Both return zero for a zero span. `RealTime`, `BaseTime`, omitted, and nil modifiers produce the same fraction because scaling cancels. These behaviors, including invalid-modifier and invalid-bound-clock errors through existing validation, are simulator policies. `/tmp/verify-duration-percent-ledger.json` records seven focused tests passing on retail 12.0.0 (reused matching hashes), 12.0.5, and 12.0.7, plus passing format, check, build, and standalone startup. It does not establish native compatibility, consumer loading, or security behavior; independent metadata follow-up remains pending.

`FrameAPICooldown.SetCooldownFromDurationObject` resolves proxy methods through Lua indexing. A zero duration preserves existing cooldown timing when `clearIfZero = false`; omitted/true clears it. This is tested simulator behavior, not native confirmation. Protected, secret, and forbidden semantics remain unresolved.

## Sources

- [Duration core spec](../../specs/duration-core.md) — modeled contract and explicit assumptions.
- [`core.rs`](../../../src/lua_api/globals/lua_duration_object/core.rs) — state and query implementation.
- [`duration_core.rs`](../../../tests/duration_core.rs) — focused ordinary behavior proof.
- [`cooldown_widget.rs`](../../../tests/cooldown_widget.rs) — bounded proxy consumer proof.

## See Also

- [[patch-12-1-5-api-audit]] — exact changed API occurrences and evidence boundary.
- [Duration text binding](../../specs/duration-text-binding.md) — separate consumer path.
