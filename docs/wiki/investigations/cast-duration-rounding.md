# Cast duration lost through deadline rounding

The completion-duration flake comes from reconstructing a configured span by subtracting floating-point absolute timestamps, not GC or cross-test state.

## Evidence

At base `0856b624e`, eight full parallel lib runs did not reproduce the duration assertion. A temporary diagnostic assertion captured targeted run 3 failing with `total=0.99999999999999989 start=0.25440984300000002 end=1.2544098429999999 now=0.25443996000000002`. The cast input was exactly one second. `SetCasting` stored `end = start + duration`; `UnitCastingDuration` recovered `end - start`. Rounding at the addition makes that subtraction one ULP below one. Startup scheduling changes the start value; neither expiration nor an event callback caused this failure.

`unit_cast_duration_preserves_span_across_rounded_deadline` seeds the captured start and one-second span. It fails against the old model without load, sleeps, retries, or collection. It also checks the rounded end, delay extension, and retained snapshot independence. Temporary diagnostic instrumentation was removed; the original completion test and exact one-second assertion remain unchanged.

## Fix

`CastingState` stores `start_time` and `duration` instead of `start_time` and `end_time`. `end_time()` derives the absolute deadline for casting/channel tuples and completion. Cast, specialization, crafting, channel and empower producers preserve their input spans; delay/update inputs mutate duration. Duration snapshots read the span directly and add empower hold before constructing the snapshot. There is no second mutable deadline or duration cache to synchronize.

Existing fixtures switch from absolute deadlines to equivalent spans. Zero-duration completion fixtures still finish through the real completion boundary. Overflow validation stays before mutation.

## Sources

- [Cast state](../../../src/lua_api/game_data.rs)
- [Duration queries](../../../src/lua_api/channeling/durations.rs)
- [Deterministic regression](../../../src/iced_app/casting/duration_tests.rs)
- [Duration contract](../../specs/unit-cast-durations.md)

## See Also

- [GC rooting audit](gc-rooting-audit.md) — independent GUID/event-payload lifetime fixes.
