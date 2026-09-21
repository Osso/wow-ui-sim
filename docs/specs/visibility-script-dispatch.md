# Visibility script dispatch

Runtime `Show`, `Hide`, and `SetShown` transitions in `src/lua_api/frame/methods/core_state/visibility.rs` must deliver installed intrinsic and normal visibility scripts. See the [event investigation](../wiki/investigations/synchronous-intrinsic-events.md) for the distinction between event delivery and the visibility-driven registration boundary.

## What it must do

- [ ] Deliver intrinsic precall, normal, and intrinsic postcall bindings for each affected frame, in that order.
- [ ] Preserve children-first traversal and exclude children whose own shown state is false. Ancestor hiding must not change a shown child's own shown state.
- [ ] Preserve existing parent transitions, unchanged-state no-ops, reentrant show/hide draining, and depth/cycle limits. A handler-selected opposite state is processed after the current frame's binding sequence completes.
- [ ] Report handler errors through the existing error handler and continue remaining bindings and parent delivery.
- [ ] A native Forever AuraContainer configured under a hidden parent must register `UNIT_AURA` when that parent is shown, unregister when hidden, and consume repeated post-construction aura add/remove cycles across repeated parent visibility transitions.

These are engine script/lifecycle requirements, not guessed aura data or a new security policy.

## How it works

- [Event system](../event-system.md)
- [Synchronous intrinsic-event investigation](../wiki/investigations/synchronous-intrinsic-events.md)

## Implementation inventory

- `src/lua_api/frame/methods/core_state/visibility.rs` — recursive transition delivery through existing ordered script binding lookup.
- `src/lua_api/script_helpers.rs` — existing precall/normal/postcall collection, reused unchanged.

## Tests asserting this spec

- `tests/frame_creation/visibility_scripts.rs` — real XML intrinsic bindings, children-first ordering, hidden-child exclusion, reentrant hide, error continuation, and existing recursion/depth controls.
- `tests/forever_forbidden_consumers.rs` — real native container configured while its parent is hidden, followed by two show/add/remove/hide cycles and registration/assignment assertions.

## Known gaps (current cycle)

- [ ] Compilation and GREEN remain with the integrating caller; this task explicitly prohibits Cargo execution.
- Runtime RED: `/tmp/ellesmere-forever/aura-post-removal-state-ledger.json` records a visible container without its required dynamic unit registration after the hidden-parent construction sequence.
- Admin aura producers already use `fire_named_event_state`, which dispatches intrinsic bindings. The separate `c6d970cf3` correction to `FireEvent`/`A_Admin.FireEvent` was valid but did not fix this visibility-driven removal failure.

## Out of scope

Vendor changes, explicit calls to native `UpdateEventRegistrations`, loader/template rewrites, and changes to taint or secret-value policy. `precompiled::fire_onshow` already iterates all installed bindings; its named intrinsic fallback is unchanged because this lifetime failure occurs during recursive runtime visibility transitions.
