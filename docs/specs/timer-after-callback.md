# Timer After callback dispatch

`C_Timer.After` dispatches a one-shot callback without arguments. Its simulator wrapper lives in `src/lua_api/workarounds/temporary/proxy_object_factories.rs`; scheduling remains in the existing timer engine.

## What it must do

- [x] Accept the existing ordinary function and callback-container forms and return no values.
- [x] Defer zero-delay callbacks until normal timer processing, invoke them once with zero arguments, and not repeat on a later processing pass.
- [x] Preserve NewTimer/NewTicker callbacks' one container-proxy argument, handle equality, shared fields, and finite ticker iteration behavior.
- [ ] Retain After/NewTimer/NewTicker timers registered inside a callback for subsequent processing passes, without firing them in the registering pass.
- [ ] Preserve already-pending timers, finite ticker repeats, and cancellation of both callback-created timers and timers already taken for processing.

The cached retail `UITimerDocumentation.lua` declares `After` with a `TimerCallback` (no callback arguments), while `NewTimer`/`NewTicker` use `TickerCallback` (one callback argument). The checked-in 12.0.0 occurrence records the callback-type change to `LuaFunctionContainer`, seconds input, and no return values; it does not independently establish callback arguments. Historical simulator-profile tests are not native historical-client evidence.

## How it works

- [Event system](../event-system.md)
- [Timer callback probe and captured limits](../addons/TimerCallbackProbe/README.md)

## Implementation inventory

- `src/lua_api/workarounds/temporary/proxy_object_factories.rs`: callback-container acceptance and distinct After/ticker invocation.
- `src/lua_api/timer_layout.rs`: existing scheduling and registry-held callbacks.
- `src/lua_api/env_runtime.rs`: normal timer processing.

## Tests asserting this spec

`tests/missing_apis.rs` contains four `timer_after_` cases: ordinary function and container After callbacks, plus NewTimer and finite NewTicker controls. Two `timer_nested_callbacks_` cases additionally cover callback-created timers, finite repeats, retained pending timers and cancellation. The nested fixture lives in `tests/fixtures/nested_timers.lua`. Tests use `WowLuaEnv::process_timers`, not direct callback invocation or mocked scheduling.

## Known gaps (current cycle)

- [ ] Nested-timer compiled GREEN remains pending. Before the production change, the existing simulator binary ran the shared fixture through normal headless update/timer passes and reported `nested timer pass 2 after: expected 1, got 0`. Evidence: `/tmp/ellesmere-forever/nested-timers-ledger.json`.

- [ ] Native After container identity, historical callback arguments, exact delay boundaries, coercion/errors, cancellation edge cases and GC/lifecycle behavior are not established by this slice.
- [ ] The real-client NewTimer/NewTicker capture does not prove After behavior.

## Out of scope

Timer-engine redesign, cancellation implementation, callback representation, GC changes, unrelated callback APIs, vendor code, and secrets/security enforcement are out of scope. Queue preservation retains callback-created entries after existing pending/repeating entries. Zero-delay processing observations describe the simulator; no native timing guarantee is inferred.
