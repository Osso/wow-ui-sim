# Full third-party startup — 2026-10-10

**Completion PASS; errorless full-third-party startup FAIL.** Existing receipt `20261010T173544Z` records exit 0, completion marker seen and unchanged sealed artifact. This docs-only record retains sanitized evidence; no rerun, build, source change, operation, delegation or commit occurred. SSOT, index and log remain untouched for main integration.

## Recorded coverage

| Observation | Evidence / boundary |
|---|---|
| Completion | Exit 0 and `[FullAddonStartupComplete]`; not errorless acceptance |
| Third-party loading | 79 addon attempts, 49 loaded; not all 79 failed |
| Lua errors | 105 occurrences across seven displayed signatures, including suppression summaries |
| Owner budget | 104 occurrences: EnhanceQoL 100, AllTheThings 4 |
| Nil call | One EllesmereUI occurrence at UICore line 1224 |
| Runtime warnings | Four reported; individual details unavailable |

Four file-load budget signatures contribute one occurrence each; EnhanceQoL OnEvent contributes 96 and AllTheThings OnEvent four. Seven first-occurrence headers plus 98 suppressed occurrences yield 105. No handler/file budget diagnostic counters were present: entry exhaustion versus callback consumption and native quota parity remain unestablished.

Loading separately reports one failed during loading, zero load failures and one loaded with Lua errors. Those categories do not explain every nonloaded addon. Sound and SavedVariables were disabled; cache hits demonstrate warm-cache execution, not runtime-input provenance. Inherited `WOW_SIM_NO_ADDONS` was not recorded/unset, but actual third-party loading is demonstrated by the saved streams.

## Nil-call metadata, not input semantics

The marker reports `nil`, `table`, `nil` for `type(IsUsingGamepad)`, `type(C_GamePad)` and the type of raw `IsEnabled`. Current local EllesmereUI source corroborates **IsUsingGamepad as callee one only**. It is not a receipt-hashed runtime asset. The first nil call prevents establishing that the second call was reached.

The raw-member probe does not establish metamethod-resolved lookup, native active-device semantics or gamepad enablement behavior. Native semantics remain unknown. No guessed default or repair is proposed; no vendor source is retained.

## Artifact association and separate stock proof

The full-addon submission revision is `d6f767b11b11a072a3bc3e8a1dd3395e7c4d97df`; it reused the seal from original submission `9635468d994be53d1f3571a8005115a49864c18a`. Independently verified seal SHA-256: `c9dd3209fbc3c10e8c40dc930cdf3890e8725b4bbe7a0ff8704fe0168956c552`. The independent report checked 3,853 recorded source/input paths: zero changed or missing. This is bounded source-byte association, not external dependency or all-runtime-input provenance.

Two separate clean-stock results remain separate evidence; neither establishes full-third-party acceptance. The inspected report specifically describes the original `--no-addons` completion receipt. This page does not elevate that receipt into third-party success or independently attest the other stock result.

## Retention

Only sanitized report, small outcome, marker summary and derivative SHA-256 manifest are retained. Original stream hashes are metadata, not retained raw streams. No raw vendor stderr, addon payload, private character/realm or cache identity is published. Original build warnings are separate from the four runtime warnings.

## Sources

- /home/osso/.local/state/wow-ui-sim/verification/full-addon-startup-current/independent-report.md — existing independent artifact inspection.
- /home/osso/.local/state/wow-ui-sim/verification/full-addon-startup-current/20261010T173544Z/outcome.json — exit and completion outcome.
- /home/osso/Projects/wow/wow-ui-sim/data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/full-addon-startup-current/independent-report.md — sanitized retained report.
- /home/osso/Projects/wow/wow-ui-sim/data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/full-addon-startup-current/marker-summary.json — bounded counts, shapes and original artifact hashes.
- /home/osso/Projects/wow/wow-ui-sim/data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/full-addon-startup-current/SHA256SUMS — retained derivative hashes.
