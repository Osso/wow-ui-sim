# Unit event callbacks

`RegisterUnitEventCallback` and `UnregisterUnitEventCallback` provide ordinary unit-filtered global callback lifecycle from retail 12.0.0. Source: `src/lua_api/globals/real/event_callbacks.rs`. See [global callbacks](global-event-callbacks.md) and [event architecture](../event-system.md).

## What it must do

- [x] Register and unregister with zero return values, matching the pinned global declarations (not the frame-method boolean contract).
- [x] Invoke registered functions or modeled FunctionContainers with `nil, ...payload` when the first event argument matches the registered unit token.
- [x] Deliver through both environment firing and synchronous Lua/model dispatch.
- [x] Preserve other callback identities and other unit registrations when removing one registration.
- [x] Isolate registrations between environments.
- [x] Support the actual pinned `Event.RegisterUnitCallback` wrapper and its `Unregister` handle.

Exact unit-string matching, nonempty-unit validation, idempotent duplicates, snapshot dispatch, insertion order and global-before-unit-before-frame ordering are simulator policies, not verified native semantics. Existing callback error policy remains unchanged. Native token aliases, lists, validation and security are not established by these tests.

## How it works

- [Event architecture](../event-system.md)
- [Global callback contract](global-event-callbacks.md)

## Implementation inventory

- `src/lua_api/globals/real/event_callbacks.rs`: unit-keyed registry using shared identity updates and invocation.
- `src/lua_api/globals/register.rs`: historical API publication.
- `src/lua_api/workarounds/temporary/dispatcher_callback_defaults.rs`: earlier-profile compatibility only.

## Tests asserting this spec

`tests/global_event_callbacks.rs`: `unit_event_callback_` tests. Commit `c1d401ba3` reached RED 0/4 on missing initial delivery. Runtime `0a0ac416f` plus publication `0835b3c94` passes 12/12 each on retail 12.0.0/12.0.5/12.0.7: four unit-callback and eight global-callback regressions. Fallback tests pass 2/2 retail and 3/3 Mists. Fmt/check/default builds and startup `[]` pass. Metadata proof `539c44c57`: two credits, 63 renewals, 20 additions, 15,124 fresh / zero stale, validator exit 0 / 3,410 rows; totals **2351 / 1057 / 2**, snapshot **1,083 / 282**. Ledgers: `/tmp/verify-unit-event-callbacks-ledger.json`, `/tmp/verify-unit-event-callback-metadata-ledger.json`.

## Known gaps (current cycle)

No further modeled-cycle gaps.

## Out of scope

Native aliases, list matching, duplicate/order/error/validation semantics, restricted event eligibility, secrecy/taint and full-LoD behavior. No frame listener or VM changes.
