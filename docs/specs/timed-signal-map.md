# Timed signal maps

PTR 12.1.5 provides `C_Timer.NewTimedSignalMap` as a state-backed C API object for keyed deadlines. The backing module, state, timer-pass dispatch, next-deadline calculation, and Lua app-data export compile only under `retail-12-1-5`; other profiles keep their existing timer behavior. The simulator does not emulate secret-value taint restrictions not otherwise modeled.

## What it must do

- [x] Create a `userdata` map with writable per-instance Lua fields and read-only native methods.
- [x] Schedule integer keys on the `GetTime()` clock; replacement, cancellation, count, lookup, and earliest-signal queries use map state.
- [x] Remove a due key before invoking its callback with that key.
- [x] Defer signals scheduled from a callback until the next timer pass.
- [x] Release scheduled signals and callback retention when the map is finalized.
- [x] Support the PTR `TimerUtil.CreateTimedSignalCallbackMap` wrapper.
- [ ] Prove non-PTR builds omit the TimedSignalMap backing state while preserving existing timer behavior.
- [ ] Enforce secret-value access restrictions. Rilua timer objects currently have no modeled secret-value state.

## How it works

- [Timer processing](../event-system.md)
- [C API architecture](../lua-api.md)

## Implementation inventory

- `src/c_api/timed_signal_map.rs`: native userdata, keyed deadline model, and due-signal extraction.
- `src/lua_api/timer_layout.rs`: PTR-gated `C_Timer` registration and callback retention.
- `src/lua_api/env_runtime.rs`: PTR-gated timer-pass dispatch and next-deadline integration.
- `src/lua_api/state/sim_state.rs`: PTR-gated map state owned by `SimState`.

## Tests asserting this spec

- `tests/timed_signal_map.rs`: method behavior, reentrant scheduling, finalization, and the source-derived `TimerUtil` wrapper fixture.

## Known gaps (current cycle)

- [ ] Startup and panel proof against the full PTR source cache remains separate from these focused API tests.

## Out of scope

- A compatibility Lua fallback or vendor-source patch. The PTR API is registered as a native C API surface.
