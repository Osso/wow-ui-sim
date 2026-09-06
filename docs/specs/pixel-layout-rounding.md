# Pixel layout rounding

PTR 12.1.5 native `SetRoundLayoutToNearestPixel` and `GetRoundLayoutToNearestPixel` control per-region layout rounding. The compatibility evidence is recorded in [the live PTR investigation](../wiki/investigations/ptr-pixel-rounding-probe.md).

## What it must do

- [ ] Expose the native boolean setter/getter under the cumulative `retail-12-1-5` epoch, defaulting to false independently for Frame, Texture, and FontString objects.
- [ ] Preserve requested dimensions and anchor offsets while resolving rounded explicit dimensions and offsets using the physical display height and effective region scale. `GetPoint` retains requested offsets; disabling rounding restores fractional layout.
- [ ] Match captured bottom-left, center, two-anchor stretch, object-scale, parent-scale/reposition, Texture all-points, and explicit FontString cases. Do not snap inherited target geometry or stretch-derived dimensions a second time.
- [ ] Return coherent rounded dimensions through `GetRect`, `GetSize`, `GetWidth`, and `GetHeight`, with the same results when enabled before or after geometry assignment and on subsequent timer ticks.
- [ ] Invalidate layout after flag, scale, parent geometry, or physical display changes. Resize expectations follow the source PixelUtil conversion; resize was not exercised by the live capture.

## How it works

- [Layout coordinates and anchor resolution](../layout-system.md)
- [PTR evidence and limits](../wiki/investigations/ptr-pixel-rounding-probe.md)
- [Physical display inputs](display-metrics.md)

## Implementation inventory

- `src/widget/frame.rs`, `frame_defaults.rs` — per-region flag, initially false.
- `src/widget/registry/pixel_scale.rs`, `registry/mod.rs` — cached physical conversion for shared layout.
- `src/layout.rs` — rounded requested sizes/offsets before anchor resolution.
- `src/lua_api/frame/methods/core_state/{round_layout,mod,helpers}.rs` — epoch-gated methods and size queries.
- `src/lua_api/{state,env_runtime}.rs` — initialize/update physical conversion with display state.

## Tests asserting this spec

- `tests/pixel_rounding_probe.rs` — grouped native capture replay and display-resize regressions, plus existing read-only probe protocol tests.

## Known gaps (current cycle)

- [ ] Native methods and shared layout computation require replay and independent verification.
- [ ] PTR startup and representative panel acceptance remain open.

## Out of scope

The received capture uses hidden owned objects. It does not prove rendered pixel/glyph output, clipping, hit testing, animation, secret-value handling, protected-state restrictions, or exact half-pixel tie behavior. No such live-parity claim is made from this capture. Blizzard/vendor Lua remains unchanged.
