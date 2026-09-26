# Slider and StatusBar range validation

`SetMinMaxValues` validates bounds before changing widget state in `src/lua_api/frame/methods/widgets/slider.rs`. See [widget system](../widget-system.md).

## What it must do

- [x] Reject a reversed Slider range with a catchable Lua error, preserving the previous range and value.
- [x] Collapse a reversed StatusBar range to its supplied maximum and clamp the stored value to that range.
- [x] Reject NaN in either bound for both widget types with a catchable Lua error before mutation; subsequent valid setters still work.
- [x] Preserve ordinary clamping and equal-bound ranges.

These are simulator policies corroborated in part by Solarity source, not native-client verification. Solarity `crates/ui/src/script/simple_script.rs` rejects reversed Slider bounds; `crates/ui/src/script/simple_script/status_bars.rs::set_range` reduces the minimum to `minimum.min(maximum)`. NaN rejection prevents the observed Rust `f64::clamp` panic; exact native handling and error text remain unverified.

## How it works

- [Widget system](../widget-system.md)
- [Slider callback contract](slider-value-callback.md)

## Implementation inventory

- `src/lua_api/frame/methods/widgets/slider.rs`: bound validation and range application.

## Tests asserting this spec

`tests/widget_slider.rs`, existing grouped `integration` target: reversed Slider rejection, reversed StatusBar collapse, separate Slider/StatusBar NaN rejection and recovery, and equal/ordinary range controls.

## Known gaps (current cycle)

- [ ] Reversed-range interpolation clamping is retained in code but lacks a direct behavioral assertion. The existing interpolation lifecycle control passes.

RED at `0e3c8157c`: four Rust panic failures and one passing control. Independent GREEN after `c72db371d`: 15 widget tests and one interpolation control, plus `cargo check`; `/tmp/cross-version-range-validation-verification-ledger.md` records exact scope and the formatting-only follow-up.

## Out of scope

Native validation/error fidelity, infinity policies, argument coercion, range-change callback semantics, and other value setters. No vendor changes or new per-profile behavior.
