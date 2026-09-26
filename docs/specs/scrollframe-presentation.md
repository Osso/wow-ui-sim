# ScrollFrame presentation offsets

ScrollFrame offsets move the designated scroll child's presented subtree, not its stored anchors or logical layout. Shared geometry lives in `src/layout.rs`; rendering and mouse hit-testing consume it. See [rendering architecture](../rendering-pipeline.md).

## What it must do

- [ ] Apply horizontal and vertical offsets once to the designated scroll child and its descendants, including descendants anchored outside that subtree. Positive offsets move content left/up in screen coordinates.
- [ ] Preserve sibling controls, logical `GetRect`/`GetPoint`, existing callback payloads, same-value suppression, and scroll-range state. Repeating an offset must not accumulate movement; restoring zero restores presentation.
- [ ] Use the scroll child's effective scale at each crossed ScrollFrame edge. Nested scroll children accumulate ancestor offsets; each viewport inherits outer scrolling but not its own offset.
- [ ] Keep quads and cache-backed hit targets aligned after offset mutation, including hit-test descent and ancestor clips.
- [ ] Clip content to its viewport; disjoint nested viewports must produce no visible content, not an absent clip.

Solarity's `crates/ui/src/render/c_simple_render.rs` separates immutable geometry from ScrollFrame presentation transforms and scales offsets by child effective scale. Its render-coordinate vertical sign differs from this simulator's Y-down screen convention. This corroborates the modeled approach, not native `GetRect`, scale, or nested-scroll semantics. No vendor code is copied or modified.

## How it works

- [Layout architecture](../layout-system.md)
- [Rendering architecture](../rendering-pipeline.md)

## Implementation inventory

- `src/layout.rs`: shared scroll-ancestry presentation translation.
- `src/lua_api/frame/methods/widget_scroll.rs`, `widgets/slider.rs`: scroll offset mutation and subtree presentation invalidation.
- `src/iced_app/strata_emit.rs`, `quad_builders_line.rs`, `masking.rs`: presented render bounds, clips, line vertices and masks.
- `src/iced_app/update_helpers.rs`, `view/hit_testing.rs`: incremental hit geometry and transformed ancestor clipping.

## Tests asserting this spec

- `tests/scroll_widgets.rs`: public setters to solid quad positions, external anchors, sibling chrome, logical geometry, repeated/reset offsets, and disjoint nested clips.
- `src/iced_app/render_hit_grid_tests.rs`: public setters to cache-backed hit movement, chrome, logical geometry and reset/no-drift behavior.

## Known gaps (current cycle)

- [ ] Final verification pending after repairing repeated ancestry traversal. Initial independent verification passed 27 scroll-widget cases, eight cache-backed hit cases, six strata, three mask, three line, and one hover control, but two existing 20,000-frame hit-test chains timed out. Traversal-local offset memoization and clipping short-circuiting address that regression; the deep-chain module must pass before completion. `/tmp/cross-version-scroll-presentation-verification-ledger.md`.

`ed69bd136` reproduces the original three failures: stationary hit target, stationary quads, and unclipped disjoint nested content. `/tmp/cross-version-scroll-presentation-proof.md`.
- [ ] Scaled/nested hit-test fixtures, line/mask/highlight-specific scrolling, live GPU pixels and native-client semantics are not proven by the initial three cases.

## Out of scope

Scroll-child ownership/replacement semantics, logical geometry query redesign, public mouse-over/intersection query semantics, automatic range refresh, new ScrollBox behavior, vendor changes, unrelated clipping/drag redesign and performance claims. This is a bounded render/input correction, not universal ScrollFrame compatibility.
