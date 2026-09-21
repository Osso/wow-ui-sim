# Numeric rule formatter

`C_StringUtil.CreateNumericRuleFormatter` creates a stateful userdata formatter for Retail 12.1+ and Forever through the shared `numeric-rule-formatters` capability. Rules and components follow the pinned Blizzard `NumericRuleFormatterAPIDocumentation.lua`, `NumericRuleFormatterSharedDocumentation.lua`, and `NumericFormatterAPIDocumentation.lua` contracts. Resource-bar countdowns consume it through duration text bindings.

## What it must do

- [ ] Expose the existing constructor and rounding enum on Forever so Ellesmere AuraKit can use its preferred numeric formatter without entering its seconds-formatter fallback.
- [x] Create independent userdata objects supporting `AddBreakpoint`, `ClearBreakpoints`, `Copy`, `GetBreakpoints`, `SetBreakpoints`, and `FormatNumber`.
- [x] Select the greatest threshold not exceeding the original input; apply breakpoint step rounding, then minimum/maximum clamps.
- [x] Apply each component's division, remainder and step rounding in that order; format the resulting numeric arguments with the configured format string.
- [x] Publish `Enum.NumericRuleFormatRounding` and metadata with Nearest=0, Up=1, Down=2. Omitted rounding uses the documented Nearest default.
- [x] Copy configuration on input, readback and formatter cloning; reject invalid replacement without modifying prior rules.
- [x] Feed native `FormatNumber` output into duration text bindings and FontStrings, while retaining the existing custom table `.Format` contract.

## How it works

The model keeps owned Rust rule/component data sorted by threshold. Lookup uses a binary partition point. Format validation permits numeric conversion directives only, checks their count against components, and validates printf syntax through a retained reference to the bootstrap `string.format` implementation. No runtime dependency was added.

## Implementation inventory

- `src/c_api/numeric_rule_formatter.rs` — userdata lifetime, methods, enum and constructor registration.
- `src/c_api/numeric_rule_formatter/config.rs` — configuration parsing, validation and copied readback.
- `src/c_api/numeric_rule_formatter/model.rs` — transformations and numeric format validation.
- `src/c_api/c_string_util.rs`, `src/c_api/mod.rs` — registration, including repeated utility bootstrap registration.
- `src/c_api/duration_text_binding.rs` — explicit native formatter branch in the existing duration-binding implementation; native errors propagate rather than reverting to unformatted text.

## Tests asserting this spec

`tests/numeric_rule_formatter.rs`: seven focused cases cover countdown formatting, non-tie rounding, thresholds/clamps, components, independent configuration/copies, transactional validation, duration binding, and Ellesmere AuraKit's exact duration breakpoint table. The shared `tests/fixtures/ellesmere_duration_formatter.lua` asserts `10`, `1m`, `2m`, hours and days. At 59.9 seconds the existing model returns `60`: threshold selection precedes rounding. This boundary preserves the modeled contract; native parity at that boundary has not been measured.

## Known gaps (current cycle)

Focused development verification passed 6/6 at `722573846` with zero compiler warnings (`/tmp/pi-numeric-formatter-current-green.*`). The duration test uses the documented `C_DurationUtil.CreateDurationTextBinding()` constructor and setters; no `DurationUtil` alias is added. Forever exposure reproduced RED against the existing executable: the identical Ellesmere fixture reports `native numeric-rule formatter constructor missing` (`/tmp/ellesmere-forever/numeric-rule-forever-ledger.json`). Compiled GREEN for the shared capability, broader checks and full-addon acceptance remain parent-owned.

The native behavior for empty/no-matching rule sets, duplicate thresholds and exact nearest-rounding ties is not documented by these sources. This model explicitly errors for those cases rather than inventing output or choosing an undocumented tie policy. Nonfinite numbers, nonpositive steps, zero divisors/moduli and non-UTF-8 formats are rejected. Readback is threshold-ordered; no native ordering parity claim is made.

## Out of scope

Native error wording, secret-number propagation, undocumented edge-policy parity, and a broader rewrite of the temporary duration-binding implementation. Full-addon acceptance belongs to the parent integration task.
