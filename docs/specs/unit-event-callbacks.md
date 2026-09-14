# Unit event callbacks

`RegisterUnitEventCallback` and `UnregisterUnitEventCallback` provide ordinary unit-filtered global callback lifecycle from retail 12.0.0. Source: `src/lua_api/globals/real/event_callbacks.rs`. See [global callbacks](global-event-callbacks.md) and [event architecture](../event-system.md).

## What it must do

- [ ] Register and unregister with zero return values, matching the pinned global declarations (not the frame-method boolean contract).
- [ ] Invoke registered functions or modeled FunctionContainers with `nil, ...payload` when the first event argument matches the registered unit token.
- [ ] Deliver through both environment firing and synchronous Lua/model dispatch.
- [ ] Preserve other callback identities and other unit registrations when removing one registration.
- [ ] Isolate registrations between environments.
- [ ] Support the actual pinned `Event.RegisterUnitCallback` wrapper and its `Unregister` handle.

Exact unit-string matching, nonempty-unit validation, idempotent duplicates, snapshot dispatch, insertion order and global-before-unit-before-frame ordering are simulator policies, not verified native semantics. Existing callback error policy remains unchanged. Native token aliases, lists, validation and security are not established by these tests.

## How it works

- [Event architecture](../event-system.md)
- [Global callback contract](global-event-callbacks.md)

## Implementation inventory

- `src/lua_api/globals/real/event_callbacks.rs`: unit-keyed registry using shared identity updates and invocation.
- `src/lua_api/globals/register.rs`: historical API publication.
- `src/lua_api/workarounds/temporary/dispatcher_callback_defaults.rs`: earlier-profile compatibility only.

## Tests asserting this spec

`tests/global_event_callbacks.rs`: `unit_event_callback_` tests. Commit `c1d401ba3` reached RED 0/4 on missing initial delivery. Ledger: `/tmp/unit-event-callback-red-ledger.json`. Integrated proof pending.

## Known gaps (current cycle)

- [ ] Independent profile, regression and startup proof.
- [ ] Bounded audit provenance and inventory updates.

## Out of scope

Native aliases, list matching, duplicate/order/error/validation semantics, restricted event eligibility, secrecy/taint and full-LoD behavior. No frame listener or VM changes.
