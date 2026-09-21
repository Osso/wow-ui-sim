# Synchronous event dispatch

`FireEvent` and `A_Admin.FireEvent` use `src/lua_api/globals/state_backed_queries.rs`; their frame-script delivery must agree with the existing named-event path. `A_Admin.AddBuff` and `A_Admin.RemoveBuff` do not use this route: they call `fire_named_event_state`, which already dispatches all bindings. See the [event architecture](../event-system.md) and [investigation](../wiki/investigations/synchronous-intrinsic-events.md).

## What it must do

- [ ] Deliver a matching frame event to intrinsic precall, normal, and intrinsic postcall bindings, in that order, with the original receiver and payload, including nil arguments.
- [ ] Apply the frame's unit filter to all three script bindings; a missing or different unit must not reach those scripts.
- [ ] Preserve global callback delivery before per-frame delivery. Per-frame callbacks retain their independent unit filters and run before that frame's scripts; a mismatched frame-script filter must not suppress a matching callback.
- [ ] Report handler failures through the existing error path and continue later script bindings.
This preserves existing simulator dispatch ordering and exact unit-token matching; it does not establish new native ordering or alias semantics. Aura producer lifecycle is separately exercised through the existing named-event route, not as proof of this synchronous-route correction.

## How it works

- [Event architecture](../event-system.md)
- [Synchronous intrinsic-event investigation](../wiki/investigations/synchronous-intrinsic-events.md)
- [Global event callbacks](global-event-callbacks.md)

## Implementation inventory

- `src/lua_api/globals/state_backed_queries.rs` — synchronous producer entry point and preserved callback ordering.
- `src/lua_api/script_helpers/event_dispatch.rs` — shared unit-filtered, all-binding frame-script dispatch.
- `src/lua_api/script_helpers.rs` — crate-local helper export.

## Tests asserting this spec

- `tests/admin_event_api.rs` — real XML precall/postcall bindings, normal script, unit filtering, callback independence/order, and error reporting.
- `tests/forever_forbidden_consumers.rs` — native cold construction, settlement, then two add/remove cycles through the existing named-event route. This regression confirms that route's lifecycle behavior; it is not RED-to-GREEN proof that `c6d970cf3` caused or fixed the observed GUI removal.

## Known gaps (current cycle)

- No GREEN claim is current for `c6d970cf3` as a fix for the observed GUI removal. Its focused API tests establish `FireEvent`/`A_Admin.FireEvent` all-binding delivery only.
- **Diagnosis correction:** `/tmp/ellesmere-forever/aura-post-removal-state-ledger.json` shows that after `ReloadFrames`, the visible `pball` container has `UNIT_AURA` unregistered and `dynamicEvents` nil. `core_state/visibility.rs` recursively invokes only the normal `OnShow` binding, skipping the intrinsic handler that re-registers the event. This is the current removal route under implementation/proof, not an attribution to `c6d970cf3`.

## Out of scope

Loader/template changes, vendor edits, new event shims, unit-alias redesign, and broader security policy. Existing intrinsic bindings are reused rather than synthesized in the event producer.
