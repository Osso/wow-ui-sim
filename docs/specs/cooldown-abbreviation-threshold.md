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

## Historical pre-independent status (superseded by acceptance below)

- [ ] Parent saved bounded GREEN compilation/runtime; independent299 and final verification remain pending. Saved compiled RED at input `d656bf037`: build exit 0 in 692.14 seconds (`/tmp/patch-12.0.5-batch36-red-build-result.json`); corrected runtime filter selected 20 tests, 10 PASS / 10 FAIL (`/tmp/patch-12.0.5-batch36-red-correct-filter.{json,log}`). Original filename-based filter selected zero tests and is not proof. Storage separately PASS in original runs (parent report).
- [ ] Producer changes only the renderer abbreviation branch: configured inclusive 60–3600 seconds and strictly lower remaining time select ceil-rounded `m:ss`. Existing renderer unit control migrates the obsolete `9s` expectation to decimal `8.2` for invalid threshold 5, then checks valid abbreviation, aura precedence and custom-over-aura precedence. No build/test/check/readability/delegation performed by producer; parent GREEN is recorded below, independent299 remains pending.
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

## Reconciled batch36 parent proof — 2026-10-01

Saved parent RED `d656bf037`: compile exit0/692.14s; corrected runtime-red-correct-filter selects **20 = 10 PASS / 10 FAIL**, exit101; separate storage **1 PASS**. Wrong filename-derived filter selected **0 tests**, not proof. Artifacts: `/tmp/patch-12.0.5-batch36-red-{build-result,runs}.json` and `/tmp/patch-12.0.5-batch36-red-correct-filter.{json,log}`.

Parent GREEN compilation `053d7ed4d` includes producer `69c454146`: exit0/**447.06s**. `/tmp/patch-12.0.5-batch36-green-build-result.json` binds executable hashes; `/tmp/patch-12.0.5-batch36-green-runs.json` binds revision, hashes, exact argv and three logs. **40 PASS: 20 formatter + 8 renderer + 12 widget**, all exit0. Covers renderer text, threshold/minimum/ticking/precedence controls and fractional storage, not GPU/native parity.

**Historical pre-independent report: Independent299 pending (superseded below).** No startup rerun: registration unchanged; historical startup is not current-binary proof. No fresh fmt/check/readability or final acceptance claimed. Cached below-threshold `m:ss` example `1:31` is explicit; configured range interpretation, inclusive endpoints, strict boundaries, ceil rounding and minimum equality remain inferred. Exact rows536/538/540 stay pending; primitive aliases alone confer no behavioral-delta credit. **264 pending / 84 bounded / 14 partial = 362**, source IDs/hash unchanged until independent review.

Earlier accidental PLAN tracking corrected by `053d7ed4d`; `PLAN.md` remains ignored local accounting, never staged or force-added.

## Independent bounded acceptance — 2026-10-01

Full report: `/tmp/patch-12.0.5-cooldown-abbreviation-independent-proof.md`; unique artifacts `/tmp/patch-12.0.5-cooldown-abbreviation-independent-06a481b4cf*`. Saved GREEN at `053d7ed4d`: **40 PASS = 20 formatter + 8 renderer + 12 widget**. Corrected RED 10 PASS/10 FAIL; zero-selected filter excluded. Source/binary identity confirmed; fresh fmt/check **exit0** at clean `053d7ed4d`, source unchanged through docs-only `b685cdf178`. No reruns here.

Parent accepts only exact numeric-unit/consumer slices for `widgets-Cooldown-GetMinimumCountdownDuration-536`, `widgets-Cooldown-SetCountdownAbbrevThreshold-538`, `widgets-Cooldown-SetMinimumCountdownDuration-540`: fractional milliseconds getter/default/reset, fractional seconds storage/live abbreviation, fractional milliseconds total-duration gate/live updates respectively. Only these three statuses become bounded-coverage: **261 pending / 87 bounded / 14 partial = 362**. IDs, register, source plaintext SHA and unrelated rows unchanged. Primitive aliases change type annotations only, not classes or behavior; renderer discrepancy fixed separately. Minimum/storage/security fixtures preserve existing behavior.

No whole-method/native parity: range interpretation/inclusive endpoints, ceil rounding, strict threshold/minimum equality and precedence remain inferred. Near-threshold fractional carry lacks dedicated fixture. Native comparison, exhaustive secret/AllowedWhenUntainted/error/NaN/infinity/negative-input policy, GPU visuals and all-profile acceptance remain unproven. One nonblocking naming suggestion for range/minute constants deferred; no code change authorized.

## Out of scope

Vendor edits, new state/targets, native parity, GPU glyph rasterization, taint/security changes, whole-method audit promotion, unrelated API behavior and broader acceptance. Renderer-only production fix; source documentation supports a renderer discrepancy, not a storage type migration. Total-minimum/hide/expiry gates and aura/custom formatter selection stay unchanged.
