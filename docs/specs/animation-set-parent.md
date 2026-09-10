# Animation SetParent

`SimpleAnim:SetParent(parent, order?)` moves an existing animation to another animation group. The pinned [PTR register](../../data/patch-api/sources/12.1.5-register.json) preserves the base `SimpleAnimGroup` parent and optional numeric order arguments; PTR adds an `AddAnimations` forbidden-aspect check. This slice models ownership transfer, not that security check.

## What it must do

- [x] Move the animation out of the old group's animation list and append it once to the new group's list; remaining animations must retain correct identity and setters after removal.
- [x] Update `GetParent`, `GetRegionParent`, target resolution, and widget hierarchy consistently. Preserve duration, alpha configuration, delays, smoothing, child key, and callbacks.
- [x] Treat optional order as animation sequence order, not insertion position. Omission preserves the existing order.
- [x] Reject non-group parents and invalid orders before changing ownership or configuration. **Simulator validation:** finite numbers from zero through `u32::MAX`; fractions truncate like the existing ordinary `SetOrder` representation.
- [x] **Simulator timing policy:** changing owners resets only the moved animation's local elapsed/progress. Neither group's playback state or timeline is restarted; subsequent destination ticks evaluate the moved animation on that group's existing timeline. A stopped destination leaves it idle until played.
- [x] **Simulator same-parent policy:** retain list position and local elapsed/progress; an explicit order changes scheduling on subsequent ticks without immediate reset.
- [x] Completion callbacks follow current group membership; finishing the old group cannot dispatch the moved animation's callback.
- [x] Preserve generic frame/region `SetParent` behavior and equivalent ordinary animation behavior on PTR and earlier retail.

## How it works

- [Animation query lifecycle](animation-query-lifecycle.md)
- [Widget system](../widget-system.md)
- [Event system](../event-system.md)

## Implementation inventory

- `src/lua_api/frame/methods/button_anchor_hierarchy/animations/parent.rs` — validates and transfers existing animation state and ownership indexes.
- `src/lua_api/frame/methods/button_anchor_hierarchy/animations.rs` — exposes the animation transfer path.
- `src/lua_api/frame/methods/button_anchor_hierarchy/hierarchy.rs` — dispatches animation parenting before the unchanged generic hierarchy path.

## Tests asserting this spec

- `tests/animation_set_parent.rs` — real groups, ticks, configuration, order, callback ownership, invalid-input atomicity, and generic parenting.
- Existing grouped animation and parenting integration tests remain regression coverage.

Focused proof at `e44ffa9a8`: seven new tests plus seventy existing animation/hierarchy regressions passed on both `client-ptr` and `client-retail` (77 each). Initial RED at `9a4fa34f6` with the new test file produced five ownership/timing/validation failures and one passing generic-parenting test.

Commands used `cargo test --test integration --offline --no-default-features --features sound,gui,client-<profile> -- <filters> --nocapture`. Filters: `animation_set_parent::`, `animation_anim::`, `animation_group::`, `animation_group_state::`, `animation_query_lifecycle::`, `methods_hierarchy::`, and the four `frame_level` reparent/same-parent regressions. Root `cargo fmt` ran before commits; check/readability/artifact gates were not run for this implementation handoff.

## Known gaps (current cycle)

- [ ] Native timing reset, same-parent scheduling, numeric coercion/error details, and transfer during already-queued callback dispatch are unverified.
- [ ] PTR `AddAnimations`, taint, secret-argument, protected, and forbidden-aspect behavior are not established by these tests.

## Out of scope

Animation-group reparenting, scheduler redesign, new ownership representations, immediate restoration of already-applied visual effects, vendor changes, and security enforcement are outside this bounded transfer correction.
