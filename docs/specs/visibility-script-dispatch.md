# Visibility script dispatch

Runtime `Show`, `Hide`, and `SetShown` transitions in `src/lua_api/frame/methods/core_state/visibility.rs` must deliver installed intrinsic and normal visibility scripts. See the [event investigation](../wiki/investigations/synchronous-intrinsic-events.md) for the distinction between event delivery and the visibility-driven registration boundary.

## What it must do

- [x] Deliver intrinsic precall, normal, and intrinsic postcall bindings for each affected frame, in that order.
- [x] Preserve children-first traversal and exclude children whose own shown state is false. Ancestor hiding must not change a shown child's own shown state.
- [x] Preserve existing parent transitions, unchanged-state no-ops, reentrant show/hide draining, and depth/cycle limits. A handler-selected opposite state is processed after the current frame's binding sequence completes.
- [x] Report handler errors through the existing error handler and continue remaining bindings and parent delivery.
- [x] A native Forever AuraContainer configured under a hidden parent must register `UNIT_AURA` when that parent is shown, unregister when hidden, and consume repeated post-construction aura add/remove cycles across repeated parent visibility transitions.

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

## Proof

At `9c223f8b7`, `frame_creation::visibility_scripts::` passes 9/9 and constructs synthetic bindings with explicit `Frame` templates after the earlier unknown-custom-frame-type fixture failure. `forever_forbidden_consumers::` passes 5/5, including the real hidden-parent AuraContainer lifecycle. `/tmp/ellesmere-forever/visibility-gui-acceptance-ledger.json` records the unchanged addon completing all five existing interaction groups plus trusted aura paint, removal, and cleanup, with zero Lua-error lines during the 90-second run; exit 124 is expected teardown.

The earlier RED ledger remains root-cause history: it observed a visible but unregistered container after `ReloadFrames`. Admin aura producers already used all-binding `fire_named_event_state`; `c6d970cf3` independently corrects only `FireEvent`/`A_Admin.FireEvent`.

## Out of scope

Vendor changes, explicit calls to native `UpdateEventRegistrations`, loader/template rewrites, and changes to taint or secret-value policy. `precompiled::fire_onshow` already iterates all installed bindings; its named intrinsic fallback is unchanged because this lifetime failure occurs during recursive runtime visibility transitions.
