# Timer After callback dispatch

`C_Timer.After` used the ticker callback invoker, so its callback received a container proxy. Commit `1e069b03b` separates the simulator dispatch paths. Final verification passed at `62b9eb70c`.

## Evidence boundary

The cached retail timer API documentation distinguishes `TimerCallback` for `After` (no callback arguments) from `TickerCallback` for `NewTimer` and `NewTicker` (one callback argument).

`TimerCallbackProbe` captured `NewTicker` container acceptance, returned-container identity, and independent iteration counts. It did not call `After`. Its real-client capture therefore does not prove `After` accepts a container, returns a particular value, or shares ticker lifecycle behavior.

The RED ledger `/tmp/timer-after-red-ledger.json` ran four normal-timer-processing cases: two `After` cases deferred and returned no values but each received one injected proxy argument; the `NewTimer` and finite `NewTicker` controls passed. Result: 2/4 pass, exit 101, before `1e069b03b`.

## Simulator change

The existing shared `makeInvoker` remains for `NewTimer` and `NewTicker`; it captures the wrapped callback, suppresses cancelled containers, and injects a container proxy. `After` now uses a dedicated invoker with the same callback capture and cancellation check but calls the callback with no proxy argument.

The timer engine, queue timing, cancellation model, callback representation, and GC behavior are unchanged. This is a dispatch-argument correction only.

## Verification

`/tmp/verify-timer-after-ledger.json` reuses hash-matched 12.0.0 GREEN (4/4) and records fresh 4/4 focused cases on 12.0.5, 12.0.7, and Mists. The cases cover ordinary function and callback-container calls returning no values; deferred zero-delay invocation; one zero-argument callback invocation without repetition; and NewTimer/finite NewTicker proxy controls.

At `62b9eb70c`, `cargo fmt --check`, default `cargo check`, both default binaries, default startup `lua-errors` (zero errors), and the canonical 12.0.0 validator exit 0. All manifest evidence hashes are fresh. Later-profile test builds emitted 6 warnings on 12.0.5, 1 on 12.0.7, and 6 on Mists; default verification emitted none.

## Open boundaries

This closes the scoped simulator proof only. Native `After` callback-container acceptance and identity, historical callback arguments, exact delay boundaries, coercion/errors, cancellation edges, scheduling, GC/lifecycle semantics, and real Blizzard consumer behavior remain unproven.

## Sources

- [Timer After callback spec](../../specs/timer-after-callback.md) — scoped contract and implementation inventory.
- [TimerCallbackProbe](../../addons/TimerCallbackProbe/README.md) — live-client NewTicker capture and its explicit limits.
- `/tmp/timer-after-red-ledger.json` — pre-fix behavioral failure record.
- `/tmp/verify-timer-after-ledger.json` — final cross-profile, default-runtime, validator, and manifest-freshness proof.

## See Also

- [[event-system]] — normal timer processing.
- [[patch-12-0-0-api-audit]] — C_Timer occurrence audit.
