# Cooldown abbreviation threshold

`Cooldown:SetCountdownAbbrevThreshold` configures seconds-based countdown abbreviation; `SetMinimumCountdownDuration` and its getter use milliseconds. Public methods live in `src/lua_api/frame/methods/widgets/cooldown.rs`; countdown text is consumed in `src/iced_app/quad_builders_cooldown.rs`. See [rendering pipeline](../rendering-pipeline.md) and [attached formatters](cooldown-countdown-formatter.md).

## What it must do

### Abbreviation — bounded inferred policy

- [ ] A configured threshold in the inclusive range `60 <= threshold <= 3600` enables abbreviation. Remaining time strictly below it displays `m:ss`, without a unit suffix: remaining 91 seconds, threshold 92, yields `1:31`.
- [ ] At or above the threshold, retain ordinary countdown output: remaining 91 seconds with thresholds 91 or 90 yields `91`, not `91s`.
- [ ] Configured thresholds 59 and 3601 disable abbreviation; they remain stored literally. The range applies to configuration, not remaining time. Threshold 60 with remaining 59 yields `0:59`; threshold 3600 with remaining 3599 yields `59:59`.
- [ ] Observe threshold changes without recreating or restarting the cooldown; zero disables abbreviation and preserves existing whole-second/decimal output.
- [ ] Inference: retain existing `ceil` rounding before splitting minutes/seconds. Remaining 90.25 yields `1:31`; 59.25 yields `1:00` under threshold 100. Native rounding and exact boundaries are unknown.
- [ ] Attached custom formatter output takes precedence across threshold changes. Clearing the attachment resumes the current abbreviation policy; existing hide/minimum/expiry gates remain effective.

### Minimum duration — existing policy controls

- [ ] Preserve fractional milliseconds (91000.5, 90999.5), fractional abbreviation seconds (91.25), and independence from `SetCountdownMillisecondsThreshold(3.25)` in public getter round-trips.
- [ ] Gate against total display duration in milliseconds, not remaining time: total 91 seconds is eligible with minimum 90999.5 or 91000 ms, suppressed with 91000.5 ms.
- [ ] Inference: preserve the current equality policy `total_duration_ms >= minimum_ms`. Cached documentation says “above”; strictness is ambiguous, not native-verified.
- [ ] Eligibility persists while remaining time ticks below the minimum; an ineligible total stays suppressed across ticks. Changing minimum re-evaluates eligibility; expiry still suppresses output.

## How it works

- [Rendering pipeline](../rendering-pipeline.md)
- [Countdown formatter contract](cooldown-countdown-formatter.md)
- [Cooldown numeric methods](cooldown-numeric-methods.md)

## Implementation inventory

- `src/lua_api/frame/methods/widgets/cooldown.rs` — public setters/getters with explicit storage units.
- `src/widget/frame.rs` — remaining-time calculation and total-duration minimum gate.
- `src/iced_app/quad_builders_cooldown.rs` — countdown text selection.

## Tests asserting this spec

- `src/iced_app/quad_builders_cooldown/countdown_formatter_tests.rs` — nine new `countdown_abbreviation_*` cases, one `countdown_threshold_controls_*` default control, two `countdown_minimum_duration_*` controls. Exact boundaries use the existing renderer consumer with a deterministic supplied clock; ticking and custom formatter cases use the existing real engine tick harness.
- Same module: `configured_renderer_clear_restores_existing_default_thresholds` now expects plain `9` after clearing with invalid threshold 5; attached formatter output remains `8s`.
- `tests/cooldown_widget.rs` — `cooldown_threshold_storage_preserves_fractional_units_and_independence`, inside the existing grouped `integration` target. No new Cargo targets.

## Known gaps (current cycle)

- [ ] Parent owns GREEN compilation/runtime and final verification. Saved compiled RED at input `d656bf037`: build exit 0 in 692.14 seconds (`/tmp/patch-12.0.5-batch36-red-build-result.json`); corrected runtime filter selected 20 tests, 10 PASS / 10 FAIL (`/tmp/patch-12.0.5-batch36-red-correct-filter.{json,log}`). Original filename-based filter selected zero tests and is not proof. Storage separately PASS in original runs (parent report).
- [ ] Producer changes only the renderer abbreviation branch: configured inclusive 60–3600 seconds and strictly lower remaining time select ceil-rounded `m:ss`. Existing renderer unit control migrates the obsolete `9s` expectation to decimal `8.2` for invalid threshold 5, then checks valid abbreviation, aura precedence and custom-over-aura precedence. No build/test/check/readability/delegation performed by producer; GREEN remains pending.
- [ ] Cached retail `Blizzard_APIDocumentationGenerated/FrameAPICooldownDocumentation.lua`, inspected at lines 338–350, documents below-threshold abbreviation (example `1:31`). Exact assumption: 'If above one hour or below one minute no abbreviation' grammatical refers to configured threshold. Inclusive endpoints, strict below selection, ceil rounding and minimum equality are bounded simulator policy; no native probes establish boundaries.
- [ ] Register rows `widgets-Cooldown-GetMinimumCountdownDuration-536`, `widgets-Cooldown-SetCountdownAbbrevThreshold-538`, `widgets-Cooldown-SetMinimumCountdownDuration-540` change primitive type annotations only. These fixtures confer no behavioral-delta or whole-row completion credit from aliases alone.

Parent build requirements (not executed): existing library test target requires `gui` and `retail-12-0-5`; default features supply both through `client-retail`. Historical 12.0.5 uses `--no-default-features --features gui,profile-retail,retail-12-0-5`. The grouped storage test has no new feature gate. Build library and grouped integration artifacts separately from running them:

```text
cargo test --lib --test integration --no-run
```

Exact runtime filters on the respective emitted test binaries, with a bounded timeout:

```text
iced_app::quad_builders::cooldown::countdown_formatter_tests::countdown_abbreviation_
iced_app::quad_builders::cooldown::countdown_formatter_tests::countdown_threshold_controls_
iced_app::quad_builders::cooldown::countdown_formatter_tests::countdown_minimum_duration_
iced_app::quad_builders::cooldown::countdown_formatter_tests::configured_renderer_clear_restores_existing_default_thresholds --exact
cooldown_widget::cooldown_threshold_storage_preserves_fractional_units_and_independence --exact
```

Run the existing full `iced_app::quad_builders::cooldown::countdown_formatter_tests::` module as the formatter/gate control filter when integrating, not as an additional Cargo target. Also run `iced_app::quad_builders::cooldown::tests::` for the migrated renderer control. Paths match `/tmp/patch-12.0.5-batch36-lib-test-list.txt`.

## Out of scope

Vendor edits, new state/targets, native parity, GPU glyph rasterization, taint/security changes, audit status promotion, unrelated API behavior and final acceptance. Renderer-only production fix; source documentation supports a renderer discrepancy, not a storage type migration. Total-minimum/hide/expiry gates and aura/custom formatter selection stay unchanged.
