# Timed signal maps

Retail, PTR and Forever provide `C_Timer.NewTimedSignalMap` as a state-backed C API object for keyed deadlines. The shared `timed-signal-maps` capability, enabled by `retail-12-1-5` and `client-wowforever`, gates the backing module, state, timer-pass dispatch, and next-deadline calculation; other profiles keep their existing timer behavior.

## What it must do

- [x] Create a `userdata` map with writable per-instance Lua fields and read-only native methods.
- [x] Schedule integer keys on the `GetTime()` clock; replacement, cancellation, count, lookup, and earliest-signal queries use map state.
- [x] Remove a due key before invoking its callback with that key.
- [x] Defer signals scheduled from a callback until the next timer pass.
- [x] Release scheduled signals and callback retention when the map is finalized, including collection from a tainted context.
- [x] Support the Retail, PTR and Forever `TimerUtil.CreateTimedSignalCallbackMap` wrappers.
- [ ] VERIFIED — R88y: Retail 12.1.5.70688 `C_Timer.NewTimedSignalMap` uses flag 3. Publish the implemented factory in separate ordinary and secure `C_Timer` tables; do not infer identity from the flag alone. Other profiles keep their current route.
- [x] Store each deadline's actual finite numeric time and authentic secrecy together in simulator state. Rust fixtures supply arbitrary VM host-secret numbers through the same scheduling API; no constant deadline or global restriction switch substitutes for stored state.
- [x] Accept authentic secret deadlines/delays in `SignalAt` and `SignalAfter` only when rilua's guarded secret access permits the caller. Preserve the input wrapper and caller taint. Rejected input leaves the existing schedule unchanged.
- [x] Reject access to secret-holding maps through `CancelAllSignals`, `CancelSignal`, `GetNextSignal`, `GetSignalCount`, `GetSignalTime`, `HasSignal`, `SignalAfter` and `SignalAt` when rilua denies secret access (tainted or revoked context). Public-only maps remain usable by addon callers.
- [x] Derive held secrecy from current deadlines. Trusted public replacement, cancellation, clearing and due removal remove only the corresponding secrecy metadata; access becomes public after the last secret deadline is gone. Finalization never requires map access permission.
- [ ] Prove builds without the capability omit the TimedSignalMap backing state while preserving existing timer behavior.

## How it works

- [Timer processing](../event-system.md)
- [C API architecture](../lua-api.md)

`src/c_api/timed_signal_map.rs` owns native userdata, the deadline records and the shared access check. `SimState::timed_signal_maps` is the single source of truth; no Lua value is retained in deadline records. Host scheduling and wakeup calculations use each record's numeric time. Existing trusted query results remain plain values; this change does not claim complete `ConstSecretAccessor` semantics.

`src/lua_api/timer_layout.rs` retains capability-gated registration and callbacks; `src/lua_api/env_runtime.rs` dispatches due signals. No vendor modification, compatibility fallback, new enum value or secret-selector widening is required.

Source basis: each affected profile's cached `Blizzard_APIDocumentationGenerated/TimedSignalMapAPIDocumentation.lua` annotates the eight guarded methods with `RequiresTimedSignalMapAccess`; `TimedSignalMapSharedDocumentation.lua` specifies an error when held secrets meet tainted execution. This is source-derived simulator behavior, not a native-client parity claim.

## Tests asserting this spec

`tests/timed_signal_map.rs` covers public methods, reentrant scheduling, finalization, cached/fixture TimerUtil consumers, authentic secure secret ingress, rejected tainted ingress preserving state, finite input validation, `SignalAfter` conversion, all eight access checks, and metadata clearing on replacement/cancel/all/due. `tests/secureenv_isolation.rs` covers Retail dual-environment factory identity. Rust supplies different secret times rather than a fixed passthrough. Due-removal tests retain another secret across the first pass and confirm public access after the last secret is removed.

Focused API tests do not establish full startup, panel parity, secret-key support, or complete revoked-context/`ConstSecretAccessor` behavior. Profile preservation and full-feature startup evidence belong to the implementation handoff proof ledger, not to broader native compatibility claims.

## Known gaps (current cycle)

- [ ] `ConditionalSecret` callback-key publication: `TimedSignalMapSharedDocumentation.lua` specifies a secret key when the map holds secrets. Callbacks still receive plain numeric keys, including when a secret deadline becomes due. This gap is explicitly excluded from the approved deadline-secrecy/access change and requires a separate follow-up.
- [ ] Native-client verification of secret-key arguments, exact query-return policy and `ConstSecretAccessor` semantics remains separate.

## Out of scope

- Callback-key secrecy, new enum values, and widening secret selectors beyond SS-P1.
- A compatibility Lua fallback or vendor-source patch. The API is registered as a native C API surface.
