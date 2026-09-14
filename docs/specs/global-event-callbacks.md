# Global event callbacks

`RegisterEventCallback` and `UnregisterEventCallback` model ordinary global callback lifecycle from retail 12.0.0, independently of frame listeners. Source: `src/lua_api/globals/real/event_callbacks.rs`. See [event architecture](../event-system.md).

## What it must do

- [ ] Register an ordinary callback for a known callback event and return one boolean.
- [ ] Deliver `nil, ...eventPayload` through both environment event firing and synchronous Lua/model event dispatch.
- [ ] Remove the same function identity with zero return values; removing another identity preserves the registration.
- [ ] Isolate registrations between environments.
- [ ] Preserve existing frame listeners and unit-callback APIs.

Insertion order, idempotent duplicate registration, nonempty string/function validation, accepted registration returning `true`, snapshot dispatch, global-before-frame ordering, and stopping dispatch on callback error are explicit simulator policies, not verified native behavior. Errors propagate with event context. Native consumers in pinned `Blizzard_SharedXMLBase/Event.lua` establish the nil-owner callback convention.

## How it works

- [Event architecture](../event-system.md)

## Implementation inventory

- `src/lua_api/globals/real/event_callbacks.rs`: environment-local registry, identity removal, rooted dispatch snapshot.
- `src/lua_api/globals/register.rs`: historical profile publication.
- `src/lua_api/env_events.rs`: environment event dispatch integration.
- `src/lua_api/globals/state_backed_queries.rs`: synchronous event dispatch integration.

## Tests asserting this spec

`tests/global_event_callbacks.rs`: ordinary lifecycle through both dispatch boundaries, arity and environment isolation. Tests `2d43011c2` reached RED 0/4. Ledger: `/tmp/global-event-callback-red-ledger.json`. Runtime verification pending.

## Known gaps (current cycle)

- [ ] Focused runtime/profile and startup verification.
- [ ] Bounded audit evidence and provenance update.

## Out of scope

Native duplicate/order/error/validation behavior, callback eligibility and restricted events, secret/taint enforcement, unit callback variants and full-LoD behavior remain unverified. No changes to frame listener storage or VM semantics.
