# Visibility script dispatch

Runtime `Show`, `Hide`, and `SetShown` transitions in `src/lua_api/frame/methods/core_state/visibility.rs` must deliver installed intrinsic and normal visibility scripts. See the [event investigation](../wiki/investigations/synchronous-intrinsic-events.md) for the distinction between event delivery and the visibility-driven registration boundary.

## What it must do

- [x] Deliver intrinsic precall, normal, and intrinsic postcall bindings for each affected frame, in that order.
- [x] Preserve children-first traversal and exclude children whose own shown state is false. Ancestor hiding must not change a shown child's own shown state.
- [x] Preserve existing parent transitions, unchanged-state no-ops, reentrant show/hide draining, and depth/cycle limits. A handler-selected opposite state is processed after the current frame's binding sequence completes.
- [x] Report handler errors through the existing error handler and continue remaining bindings and parent delivery.
- [x] Public `SetParent` dispatches child-first `OnHide`/`OnShow` when a shown frame's effective visibility changes with its parent, after the new parent and visibility are observable; locally hidden descendants receive nothing. It preserves each frame's local shown state, ignores unchanged effective visibility (including alpha-zero parents), and bounds handler-driven reparenting without delivering stale callbacks after a redirect. Handler errors report through the existing path while parent delivery continues. Animation reparenting is unchanged.
- [x] A native Forever AuraContainer configured under a hidden parent must register `UNIT_AURA` when that parent is shown, unregister when hidden, and consume repeated post-construction aura add/remove cycles across repeated parent visibility transitions.

- [ ] For explicitly protected/forbidden frames, dispatch trusted visibility handlers across the native script boundary without inheriting the setter's taint; restore caller taint afterward. Addon-created handlers retain their own taint, and the existing combat write gate remains before dispatch.

These are engine script/lifecycle requirements, not guessed aura data. The visibility security boundary follows [protected attribute delegation](secure-attribute-delegation.md).

## How it works

- [Event system](../event-system.md)
- [Synchronous intrinsic-event investigation](../wiki/investigations/synchronous-intrinsic-events.md)

## Implementation inventory

- `src/lua_api/frame/methods/core_state/visibility.rs` — recursive transition delivery through existing ordered script binding lookup; public reparenting uses the same binding delivery and global reentry bound without altering shown state.
- `src/lua_api/frame/methods/button_anchor_hierarchy/hierarchy.rs` — public `SetParent` compares effective visibility across the validated hierarchy mutation before dispatch.
- `src/lua_api/script_helpers.rs` — existing precall/normal/postcall collection, reused unchanged.

## Tests asserting this spec

- `tests/blizzard_restricted_addon_environment_loads.rs` — unchanged secure Show/Hide snippets from addon callers, combat denial, and addon-origin callback taint preservation.
- `tests/frame_creation/visibility_scripts.rs` — real XML intrinsic bindings, children-first ordering, hidden-child exclusion, reentrant hide, error continuation, and existing recursion/depth controls.
- `tests/methods_hierarchy.rs` — pure Lua public reparenting transitions, no-op and hidden-child controls, redirect, and handler-error continuation.
- `tests/forever_forbidden_consumers.rs` — real native container configured while its parent is hidden, followed by two show/add/remove/hide cycles and registration/assignment assertions.

## Proof

At `9c223f8b7`, `frame_creation::visibility_scripts::` passes 9/9 and constructs synthetic bindings with explicit `Frame` templates after the earlier unknown-custom-frame-type fixture failure. `forever_forbidden_consumers::` passed 5/5 at `1582438c6`, including the real hidden-parent AuraContainer lifecycle; that unchanged proof remains valid after the synthetic-fixture-only correction. `/tmp/ellesmere-forever/visibility-gui-acceptance-ledger.json` records the unchanged addon completing all five existing interaction groups plus trusted aura paint, removal, and cleanup, with zero Lua-error lines during the 90-second run; exit 124 is expected teardown.

At `676cda72e`, `/tmp/cross-version-reparent-final-targeted.log` is GREEN 6/6 for public `SetParent`: child-first effective hide/show without local shown-state mutation; self-cycle rejection; descendant-cycle rejection; valid reparent/nil/same-parent child-count control; no-op/reentry final-hierarchy observation; and handler-error continuation to the parent binding. The source compiled in `/tmp/cross-version-reparent-final-build.log`. Independent verification after readability refactor `1fcdb52b2` passes six reparent cases, nine Show/Hide controls and four parent/alpha controls, plus format/check and the changed-function audit. Exact source and logs: `/tmp/cross-version-reparent-visibility-proof.md`.

The earlier RED ledger remains root-cause history: it observed a visible but unregistered container after `ReloadFrames`. Admin aura producers already used all-binding `fire_named_event_state`; `c6d970cf3` independently corrects only `FireEvent`/`A_Admin.FireEvent`.

## SetParent source reference

Wowless `wowless/modules/api.lua:106-126` compares old/new parent visibility around `DoSetParent` for a shown object and calls `UpdateVisible` on change. `wowless/modules/visibility.lua` visits shown children before the parent's script. This is source-reference evidence, not native-client verification.

## Out of scope

Vendor changes, explicit calls to native `UpdateEventRegistrations`, loader/template rewrites, and relaxing addon taint or secret-value restrictions. `precompiled::fire_onshow` already iterates all installed bindings; its named intrinsic fallback is unchanged because this lifetime failure occurs during recursive runtime visibility transitions.
