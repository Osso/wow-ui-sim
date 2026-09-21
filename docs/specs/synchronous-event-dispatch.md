# Synchronous event dispatch

`FireEvent` and `A_Admin.FireEvent` use `src/lua_api/globals/state_backed_queries.rs`; their frame-script delivery must agree with the existing named-event path. `A_Admin.AddBuff` and `A_Admin.RemoveBuff` do not use this route: they call `fire_named_event_state`, which already dispatches all bindings. See the [event architecture](../event-system.md) and [investigation](../wiki/investigations/synchronous-intrinsic-events.md).

## What it must do

- [x] Deliver a matching frame event to intrinsic precall, normal, and intrinsic postcall bindings, in that order, with the original receiver and payload, including nil arguments.
- [x] Apply the frame's unit filter to all three script bindings; a missing or different unit must not reach those scripts.
- [x] Preserve global callback delivery before per-frame delivery. Per-frame callbacks retain their independent unit filters and run before that frame's scripts; a mismatched frame-script filter must not suppress a matching callback.
- [x] Report handler failures through the existing error path and continue later script bindings.
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

## Proof and attribution

At `9c223f8b7`, `admin_event_api::test_fire_event_` passes 12/12. The synthetic fixture now uses explicit `Frame` templates for precall/postcall bindings; the earlier unknown-custom-frame-type failure was fixture construction, not dispatch behavior. `c6d970cf3` is therefore proven for `FireEvent`/`A_Admin.FireEvent` all-binding delivery only.

The observed GUI removal instead depended on recursive visibility dispatch: after `ReloadFrames`, the visible `pball` container lacked `UNIT_AURA` registration because normal-only `OnShow` skipped its intrinsic re-registration. `1e1dfe7c0` corrects that route. It is intentionally separate from the synchronous producer fix; AddBuff/RemoveBuff already use `fire_named_event_state`.

## Out of scope

Loader/template changes, vendor edits, new event shims, unit-alias redesign, and broader security policy. Existing intrinsic bindings are reused rather than synthesized in the event producer.
