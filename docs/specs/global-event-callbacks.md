# Global event callbacks

`RegisterEventCallback` and `UnregisterEventCallback` model ordinary global callback lifecycle from retail 12.0.0, independently of frame listeners. Source: `src/lua_api/globals/real/event_callbacks.rs`. See [event architecture](../event-system.md).

## What it must do

- [x] Register an ordinary callback for a known callback event and return one boolean.
- [x] Deliver `nil, ...eventPayload` through both environment event firing and synchronous Lua/model event dispatch.
- [x] Remove the same function identity with zero return values; removing another identity preserves the registration.
- [x] Isolate registrations between environments.
- [x] Accept actual `C_FunctionContainers.CreateCallback` objects without invoking callbacks during validation; reject unrelated userdata without probing its methods.
- [x] Load pinned `Blizzard_SharedXMLBase/Event.lua` unchanged and deliver its callback payload through both dispatch paths until `handle:Unregister()`.
- [x] Preserve container identity, cancellation and registry-only GC survival; invoke existing container `Invoke(nil, ...payload)` behavior.
- [ ] Preserve existing frame listeners and unit-callback APIs.

Insertion order, idempotent duplicate registration, nonempty string/function-or-modeled-container validation, accepted registration returning `true`, snapshot dispatch, global-before-frame ordering, and stopping dispatch on callback error are explicit simulator policies, not verified native behavior. Errors propagate with event context. Native consumers in pinned `Blizzard_SharedXMLBase/Event.lua` establish the nil-owner callback convention.

## How it works

- [Event architecture](../event-system.md)

## Implementation inventory

- `src/lua_api/globals/real/event_callbacks.rs`: environment-local registry, identity removal, rooted dispatch snapshot.
- `src/lua_api/globals/register.rs`: historical profile publication.
- `src/lua_api/workarounds/temporary/proxy_object_factories.rs`: internal registry bridge to existing weak backing map and `Invoke`; no new native API.
- `src/lua_api/env_events.rs`: environment event dispatch integration.
- `src/lua_api/globals/state_backed_queries.rs`: synchronous event dispatch integration.

## Tests asserting this spec

`tests/global_event_callbacks.rs`: ordinary lifecycle through both dispatch boundaries, arity and environment isolation. Tests `2d43011c2` reached RED 0/4. Container regressions `55cec0a97` load actual cached `Event.lua`: targeted RED 5/8; correction `2ff787d41` passes 8/8 focused tests. Independent proof `/tmp/verify-global-event-callbacks-corrected-ledger.json` passes 8/8 on retail 12.0.0/12.0.5/12.0.7 (12.0.0 exact-byte reuse), plus fmt/check/default binary build and startup `[]`; the original two startup errors are gone. Metadata proof `/tmp/verify-global-event-callback-metadata-ledger.json` at `cf15e178e` records two credits, 118 renewals, 28 additions, 15,098 fresh hashes, zero stale hashes, six bindings, validator exit 0 and 3,410 matching rows. Totals **2348 / 1060 / 2**; snapshot **1,086 / 282**.

## Known gaps (current cycle)

- [ ] Callback-time mutation, recursion and error recovery remain untested.

## Out of scope

Native duplicate/order/error/validation behavior, callback eligibility and restricted events, secret/taint enforcement, unit callback variants and full-LoD behavior remain unverified. No changes to frame listener storage or VM semantics.
