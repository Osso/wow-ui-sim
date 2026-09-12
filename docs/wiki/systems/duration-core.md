# Duration core

`LuaDurationObject` is an existing Lua table proxy with simulator-owned start, base-duration, rate, and optional manual-clock state. It makes ordinary duration queries and cooldown transfer observable while retaining native timing, modifier, and security behavior as open questions.

## Behavior

`SetTimeFromStart`, `SetTimeFromEnd`, and `SetTimeSpan` configure independent table proxies. Queries derive endpoints, total, elapsed, remaining, rate, zero, started, expired, and active state from the selected clock. Manual-clock advance and rewind update later observations without mutating configured timing.

The core stores `start`, base duration, and rate. Its rate/modifier formulas, zero-state rules, validation, default clock, reset behavior, and percent semantics are simulator choices documented in [duration core](../../specs/duration-core.md), not native WoW confirmation.

`24fe9d746` replaces the ordinary zero placeholders for `GetElapsedPercent` and `GetRemainingPercent`. Both derive fractions in `[0,1]` from the existing clamped elapsed/remaining clock state: before start and after end clamp to the boundary, and clock rewind changes later observations without mutating configured timing. Both return zero for a zero span. `RealTime`, `BaseTime`, omitted, and nil modifiers produce the same fraction because scaling cancels. These behaviors, including invalid-modifier and invalid-bound-clock errors through existing validation, are simulator policies. `/tmp/verify-duration-percent-ledger.json` records seven focused tests passing on retail 12.0.0 (reused matching hashes), 12.0.5, and 12.0.7, plus passing format, check, build, and standalone startup; `/tmp/verify-duration-percent-reconciled-ledger.json` completes its metadata reconciliation.

`5f7d70fe6` adds a nonzero `SetToDefaults` transition: configured timing and a bound manual clock reset to zero endpoints, rate one, and no clock binding. `/tmp/verify-duration-defaults-ledger.json` records all eight duration-core tests passing on the same three profiles, with format/readability passing; production check/build/startup proof is unchanged and reused. This does not establish native compatibility, consumer loading, or security behavior.

`19416ff84` makes `Copy` and `Assign` transfer only modeled duration timing and optional clock binding. The seven `duration_copy_` cases are pending proof: the RED record has 1/7 pass and six failures, while the first GREEN attempt failed to compile before tests ran with E0308 at `lua_duration_object.rs:339`. `acb86ceda` corrects that mutability mismatch; focused retry and independent verification remain pending. Copying a clock reference, retaining receiver identity/custom fields, and all binding policies are simulator choices; no native copy/assignment, identity, coercion, security, lifecycle/GC, or consumer behavior is inferred.

`FrameAPICooldown.SetCooldownFromDurationObject` resolves proxy methods through Lua indexing. A zero duration preserves existing cooldown timing when `clearIfZero = false`; omitted/true clears it. This is tested simulator behavior, not native confirmation. Protected, secret, and forbidden semantics remain unresolved.

## Sources

- [Duration core spec](../../specs/duration-core.md) — modeled contract and explicit assumptions.
- [`core.rs`](../../../src/lua_api/globals/lua_duration_object/core.rs) — state and query implementation.
- [`duration_core.rs`](../../../tests/duration_core.rs) — focused ordinary behavior proof.
- [`cooldown_widget.rs`](../../../tests/cooldown_widget.rs) — bounded proxy consumer proof.

## See Also

- [[patch-12-1-5-api-audit]] — exact changed API occurrences and evidence boundary.
- [Duration text binding](../../specs/duration-text-binding.md) — separate consumer path.
