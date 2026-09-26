# Slider and StatusBar range validation

`SetMinMaxValues` validates bounds before changing widget state in `src/lua_api/frame/methods/widgets/slider.rs`. See [widget system](../widget-system.md).

## What it must do

- [ ] Reject a reversed Slider range with a catchable Lua error, preserving the previous range and value.
- [ ] Collapse a reversed StatusBar range to its supplied maximum and clamp stored/interpolated values to that range.
- [ ] Reject NaN in either bound for both widget types with a catchable Lua error before mutation; subsequent valid setters still work.
- [ ] Preserve ordinary clamping and equal-bound ranges.

These are simulator policies corroborated in part by Solarity source, not native-client verification. Solarity `crates/ui/src/script/simple_script.rs` rejects reversed Slider bounds; `crates/ui/src/script/simple_script/status_bars.rs::set_range` reduces the minimum to `minimum.min(maximum)`. NaN rejection prevents the observed Rust `f64::clamp` panic; exact native handling and error text remain unverified.

## How it works

- [Widget system](../widget-system.md)
- [Slider callback contract](slider-value-callback.md)

## Implementation inventory

- `src/lua_api/frame/methods/widgets/slider.rs`: bound validation and range application.

## Tests asserting this spec

`tests/widget_slider.rs`, existing grouped `integration` target: reversed Slider rejection, reversed StatusBar collapse, separate Slider/StatusBar NaN rejection and recovery, and equal/ordinary range controls.

## Known gaps (current cycle)

- [ ] GREEN and independent verification pending. RED at `0e3c8157c`: four Rust panic failures, one valid-range control passes; `/tmp/cross-version-range-validation-proof.md`.

## Out of scope

Native validation/error fidelity, infinity policies, argument coercion, range-change callback semantics, and other value setters. No vendor changes or new per-profile behavior.
