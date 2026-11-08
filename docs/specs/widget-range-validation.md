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

- [x] `statusbar_reversed_range_clamps_pending_interpolation` asserts target7/displayed1/pending, then reversed `(8,2)` synchronously collapses both bounds/values to2 and clears interpolation. [Authenticated preservation proof](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/reversed-range-interpolation-2026-10-11/README.md): 16 widget cases plus one separate interpolation lifecycle control PASS at `6a4801a40`; compile/fmt/check exit0 and manual readability clean. No producer or native-policy change; frozen broad suite remains FAIL and full handoff OPEN.

RED at `0e3c8157c`: four Rust panic failures and one passing control. Independent GREEN after `c72db371d`: 15 widget tests and one interpolation control, plus `cargo check`; `/tmp/cross-version-range-validation-verification-ledger.md` records exact scope and the formatting-only follow-up.

## Bounded Mists execution — 2026-10-16

[Shared Mists owner matrix](../baselines/README.md#mists-shared-controls--2026-10-16) owns independently accepted saved results, provenance, diagnostics and remaining profile limits. This supplements historical evidence, not native or broader contract acceptance.

[Bounded Era execution](../baselines/README.md#era-shared-controls--2026-10-16) separately accepts the corresponding matrix cases from saved artifacts only; current source and broader criteria remain unaccepted.

[Bounded Anniversary execution](../baselines/README.md#anniversary-shared-controls--2026-10-16) separately accepts the same corresponding matrix cases from saved Anniversary artifacts only; current source and broader criteria remain unaccepted.

## Out of scope

Native validation/error fidelity, infinity policies, argument coercion, range-change callback semantics, and other value setters. No vendor changes or new per-profile behavior.
