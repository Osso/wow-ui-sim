# Timed signal maps

PTR 12.1.5 and Forever provide `C_Timer.NewTimedSignalMap` as a state-backed C API object for keyed deadlines. The shared `timed-signal-maps` capability, enabled by `retail-12-1-5` and `client-wowforever`, gates the backing module, state, timer-pass dispatch, and next-deadline calculation; other profiles keep their existing timer behavior. The simulator does not emulate secret-value taint restrictions not otherwise modeled.

## What it must do

- [x] Create a `userdata` map with writable per-instance Lua fields and read-only native methods.
- [x] Schedule integer keys on the `GetTime()` clock; replacement, cancellation, count, lookup, and earliest-signal queries use map state.
- [x] Remove a due key before invoking its callback with that key.
- [x] Defer signals scheduled from a callback until the next timer pass.
- [x] Release scheduled signals and callback retention when the map is finalized.
- [x] Support the PTR and Forever `TimerUtil.CreateTimedSignalCallbackMap` wrappers.
- [ ] Prove builds without the capability omit the TimedSignalMap backing state while preserving existing timer behavior.
- [ ] Enforce secret-value access restrictions. Rilua timer objects currently have no modeled secret-value state.

## How it works

- [Timer processing](../event-system.md)
- [C API architecture](../lua-api.md)

## Implementation inventory

- `src/c_api/timed_signal_map.rs`: native userdata, keyed deadline model, and due-signal extraction.
- `src/lua_api/timer_layout.rs`: capability-gated `C_Timer` registration and callback retention.
- `src/lua_api/env_runtime.rs`: capability-gated timer-pass dispatch and next-deadline integration.
- `src/lua_api/state/sim_state.rs`: capability-gated map state owned by `SimState`.

## Tests asserting this spec

- `tests/timed_signal_map.rs`: method behavior, reentrant scheduling, finalization, the PTR source-derived wrapper fixture, and the authenticated Forever cache's `TimerUtil.lua` consumer.

## Known gaps (current cycle)

- [ ] Startup and panel proof against the full PTR source cache remains separate from these focused API tests.

## Out of scope

- A compatibility Lua fallback or vendor-source patch. The PTR API is registered as a native C API surface.
