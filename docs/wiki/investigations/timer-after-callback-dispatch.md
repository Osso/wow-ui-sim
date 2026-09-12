# Timer After callback dispatch

`C_Timer.After` used the ticker callback invoker, so its callback received a container proxy. Commit `1e069b03b` separates the simulator dispatch paths. Development GREEN passed; later-profile and independent verification remain pending.

## Evidence boundary

The cached retail timer API documentation distinguishes `TimerCallback` for `After` (no callback arguments) from `TickerCallback` for `NewTimer` and `NewTicker` (one callback argument).

`TimerCallbackProbe` captured `NewTicker` container acceptance, returned-container identity, and independent iteration counts. It did not call `After`. Its real-client capture therefore does not prove `After` accepts a container, returns a particular value, or shares ticker lifecycle behavior.

The RED ledger `/tmp/timer-after-red-ledger.json` ran four normal-timer-processing cases: two `After` cases deferred and returned no values but each received one injected proxy argument; the `NewTimer` and finite `NewTicker` controls passed. Result: 2/4 pass, exit 101, before `1e069b03b`.

## Simulator change

The existing shared `makeInvoker` remains for `NewTimer` and `NewTicker`; it captures the wrapped callback, suppresses cancelled containers, and injects a container proxy. `After` now uses a dedicated invoker with the same callback capture and cancellation check but calls the callback with no proxy argument.

The timer engine, queue timing, cancellation model, callback representation, and GC behavior are unchanged. This is a dispatch-argument correction only.

## Development proof

`/tmp/timer-after-green-ledger.json` records four 12.0.0 development cases passing: ordinary function and callback-container calls return no values, defer zero-delay invocation, invoke once with zero injected callback arguments, and do not repeat. `NewTimer` and finite `NewTicker` controls retain their own proxy-argument behavior; they are not native `After` capture.

## Open boundaries

Later-profile and independent verification remain pending. Native `After` callback-container acceptance, exact delay boundaries, callback identity, return behavior beyond the checked-in no-output signature, cancellation edges, scheduling, and GC/lifecycle semantics remain unproven.

## Sources

- [Timer After callback spec](../../specs/timer-after-callback.md) — scoped contract and implementation inventory.
- [TimerCallbackProbe](../../addons/TimerCallbackProbe/README.md) — live-client NewTicker capture and its explicit limits.
- `/tmp/timer-after-red-ledger.json` — pre-fix behavioral failure record.
- `/tmp/timer-after-green-ledger.json` — 12.0.0 development proof; not independent verification.

## See Also

- [[event-system]] — normal timer processing.
- [[patch-12-0-0-api-audit]] — C_Timer occurrence audit.
