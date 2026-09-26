# ScrollFrame presentation offsets

ScrollFrame offsets move the designated scroll child's presented subtree, not its stored anchors or logical layout. Shared geometry lives in `src/layout.rs`; rendering and mouse hit-testing consume it. See [rendering architecture](../rendering-pipeline.md).

## What it must do

- [x] Apply horizontal and vertical offsets once to the designated scroll child and its descendants, including descendants anchored outside that subtree. Positive offsets move content left/up in screen coordinates.
- [x] Preserve sibling controls, logical `GetRect`, existing callback payloads, same-value suppression, and scroll-range state. Repeating an offset must not accumulate movement; restoring zero restores presentation.
- [x] Use the scroll child's effective scale at each crossed ScrollFrame edge. Nested scroll children accumulate ancestor offsets; each viewport inherits outer scrolling but not its own offset.
- [x] Keep quads and cache-backed hit targets aligned after offset mutation, including hit-test descent and ancestor clips.
- [x] Clip tested solid-quad content to its viewport; disjoint nested viewports must produce no visible content, not an absent clip.

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

- `tests/scroll_widgets.rs`: public setters to solid quad positions, external anchors, child scale, repeated/reset offsets, and disjoint nested clips.
- `src/iced_app/render_hit_grid_tests.rs`: public setters to cache-backed hit movement, chrome, logical geometry and reset/no-drift behavior.

## Known gaps (current cycle)

Independent final verification of `1ecc0c98c` passes 52 scoped cases: 27 scroll-widget, eight cache-backed hit, four direct hit-testing, six strata, three mask, three line, and one hover control. The four direct hit cases include both 20,000-frame chains and finish in 0.477 seconds; the initial implementation timed out at 90 seconds. Traversal-local memoization removes the introduced repeated ancestry work. Format/check pass; changed-function readability has no findings, while `strata_emit.rs` retains its pre-existing file-length-cap violation. `/tmp/cross-version-scroll-presentation-final-verification-ledger.md` records exact revisions, commands, and limitations.

`ed69bd136` reproduces the original three failures: stationary hit target, stationary quads, and unclipped disjoint nested content. `/tmp/cross-version-scroll-presentation-proof.md`.
- [ ] Direct `GetPoint` invariance, scale combined with nested input, line/mask/highlight-specific scrolling, live GPU pixels and native-client semantics remain unproven. Existing line/mask/highlight controls are not scroll-specific proof.

## Out of scope

Scroll-child ownership/replacement semantics, logical geometry query redesign, public mouse-over/intersection query semantics, automatic range refresh, new ScrollBox behavior, vendor changes, unrelated clipping/drag redesign and performance claims. This is a bounded render/input correction, not universal ScrollFrame compatibility.
