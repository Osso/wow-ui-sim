# Synchronous event dispatch

`FireEvent`, `A_Admin.FireEvent`, and synchronous model producers use `src/lua_api/globals/state_backed_queries.rs`. Their frame-script delivery must agree with the existing named-event path. See the [event architecture](../event-system.md) and [investigation](../wiki/investigations/synchronous-intrinsic-events.md).

## What it must do

- [ ] Deliver a matching frame event to intrinsic precall, normal, and intrinsic postcall bindings, in that order, with the original receiver and payload, including nil arguments.
- [ ] Apply the frame's unit filter to all three script bindings; a missing or different unit must not reach those scripts.
- [ ] Preserve global callback delivery before per-frame delivery. Per-frame callbacks retain their independent unit filters and run before that frame's scripts; a mismatched frame-script filter must not suppress a matching callback.
- [ ] Report handler failures through the existing error path and continue later script bindings.
- [ ] An already-constructed, settled native Forever AuraContainer must consume repeated admin add/remove aura events, rearm its dirty update, and update assignments without reconstruction or manually invoking its intrinsic handler.

This preserves existing simulator dispatch ordering and exact unit-token matching; it does not establish new native ordering or alias semantics.

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
- `tests/forever_forbidden_consumers.rs` — native cold construction, settlement, then two add/remove cycles with private assignment and update-mode assertions.

## Known gaps (current cycle)

- [ ] Compilation and GREEN are deferred to the integrating caller; this slice was explicitly restricted from running Cargo.
- Existing runtime RED: `/tmp/ellesmere-forever/gui-host-observed-acceptance-ledger.json` records a painted aura remaining assigned after the producer removes it. `/tmp/ellesmere-forever/aura-event-state-trace-ledger.json` confirms a registered unit listener with no normal OnEvent handler; default `GetScript` absence does not imply missing intrinsic bindings.

## Out of scope

Loader/template changes, vendor edits, new event shims, unit-alias redesign, and broader security policy. Existing intrinsic bindings are reused rather than synthesized in the event producer.
