# Manual simulation clock

## Contract

Real time is the default in every environment. `A_Admin.SetManualClock(seconds)` explicitly enables a separate environment-local elapsed clock once. Seconds must be finite, nonnegative and representable. A second opt-in fails without changing either clock. Enable manual time before supplying timestamped fixture state; arbitrary pre-existing numeric timestamps are not migrated.

The environment clock supplies `GetTime`, `GetTimePreciseSec`, C_Timer deadlines and modeled gameplay timestamps. Pending callbacks keep their scheduling coordinate when manual time is enabled, regardless of the requested GetTime epoch. Animation groups and EncounterTimeline retain their delta-driven clocks and receive the normal OnUpdate step. Independent duration-object manual clocks retain their own values. Supplied world elapsed timers remain separate explicit state.

`A_Admin.AdvanceTime(delta)` requires manual mode. It rejects negative, non-finite or unrepresentable deltas and coordinate overflow before mutation or dispatch. It advances both environment coordinates, runs one normal OnUpdate with the supplied delta, then drains due C_Timer callbacks by deadline and timer ID. Positive-interval tickers reschedule from their previous deadline and may fire repeatedly during one advance. Callback-created timers schedule from the already-advanced clock. Cancellation applies before another repeat. Zero-interval tickers run at most once per advance. Timers created during the drain wait until a later advance, including zero-delay timers. Timers already present at the start of the drain run in deadline and ID order through the target time; positive-interval ticker repeats are included.

Nested callback dispatch must retain the established owner, execution-budget, error, cancellation and provenance behavior, and restore the previous executing owner. No owning Lua RefCell may be borrowed again while an admin RustFn is executing.

Startup, renderer scheduling, watchdogs, diagnostic tracing and profiling clocks remain real. The autonomous party-damage random sample remains nondeterministic; its death/resurrection scheduling uses the environment clock. This feature does not make the whole host deterministic.

## Proof

Exact tests cover opt-in, environment isolation, OnUpdate delta, animation completion, pending delays, ordered deadlines, nested callbacks, finite and cancelled tickers, zero intervals, invalid deltas and independent duration clocks. Default-mode stock maps and existing timer/OnUpdate tests are required after the final relevant source change.
