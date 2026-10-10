# Runtime evidence audit — 20261010T021055Z

**PASS for 11 actual selected cases; startup completion PASS, clean startup NOT established.** Existing saved proof only. No builds, tests, startup reruns, delegation or backend calls. Original evidence untouched; only audit outputs written.

## Source and artifact boundary

Compiled source `86f27272a560b66201a66bede301d4c39a40910c`. Existing compiler seal remains independently PASS for **3839** equal source mappings; source-before/source-after JSON fully consumed and byte equality confirmed here. Preserve `compiler-report.md` and `scoped-manifest.json`: seven prior scoped files plus three TOC-root additions, ten total. No compiler acceptance broadened.

Runtime receipts bind all cases to compiler-selected `wow_ui_sim-test` SHA-256 `3e1f1a61cc506193565f638d9295edd6390ac65f28f08d270526edd2020b93a6`. Startup uses normal `wow-sim` SHA-256 `ea546119f359abfe146b767eab510e69a514ddb50449263e3914ff182e079b93` with compiler record `profile.test=false`. Saved receipts report artifact hashes unchanged. No claim that later executable or source bytes are accepted; no current/future-code or native/parity claim.

## Exact-case proof

Full stdout and stderr of all twelve attempts consumed locally, matched against exact selectors, summary counts, source/artifact receipts and exit codes. Eleven selected cases each report **1 selected / 1 passed / 0 failed / exit0**. Case09 reports **0 selected / 0 passed / exit0**: wrong selector retained, no pass credit. Case12 uses corrected full qualification including `xml_basics::xml_basics_extra`; saved listing contains exactly one test and zero benchmarks. Summary.json covers only cases01–11; case12 was audited from its own receipts, not silently added to the original summary.

| Case | Exact selector | Selected | Passed | Exit | Credit |
|---|---|---:|---:|---:|---|
| case01 | `toc::tests::test_parse_text_locale_excludes_production_template` | 1 | 1 | 0 | PASS |
| case02 | `toc::tests::test_parse_text_locale_includes_exact_enus_token` | 1 | 1 | 0 | PASS |
| case03 | `toc::tests::test_parse_text_locale_ignores_enus_outside_locale_tokens` | 1 | 1 | 0 | PASS |
| case04 | `toc::tests::test_parse_text_locale_unannotated_template_remains_selected` | 1 | 1 | 0 | PASS |
| case05 | `loader::tests::lua_loading::toc_text_locale_excluded_template_has_no_missing_file_warning` | 1 | 1 | 0 | PASS |
| case06 | `loader::tests::lua_loading::toc_text_locale_included_template_loads_before_core` | 1 | 1 | 0 | PASS |
| case07 | `toc::tests::test_parse_utf8_bom_preserves_first_metadata_and_file_order` | 1 | 1 | 0 | PASS |
| case08 | `loader::tests::lua_loading::toc_utf8_bom_metadata_does_not_become_a_missing_file_warning` | 1 | 1 | 0 | PASS |
| case09 | `loader::tests::xml_basics_extra::xml_file_locale_annotations_load_only_selected_scripts_and_includes` | 0 | 0 | 0 | ZERO SELECTED — no credit |
| case10 | `loader::lua_file::tests::startup_file_budget_error_preserves_cumulative_usage_and_loader_outcome` | 1 | 1 | 0 | PASS |
| case11 | `toc::tests::test_parse_simple_toc` | 1 | 1 | 0 | PASS |
| case12 | `loader::tests::xml_basics::xml_basics_extra::xml_file_locale_annotations_load_only_selected_scripts_and_includes` | 1 | 1 | 0 | PASS |

Case stderr includes expected fixture diagnostics; successful summaries do not imply diagnostic-free streams. Raw fixture output withheld; complete stream bytes and hashes retained in manifest.

## Startup streams and privacy

Complete current stdout **315150 bytes** and stderr **1515008 bytes** consumed locally; prior stdout/stderr fully consumed too. Every line screened locally before safe aggregation. Only fixed-schema counts, static source locations, hashes and receipt metadata published; no arbitrary error text, stacks, handler arguments, game payloads, credential candidates or raw log excerpts. Candidate screening is not a guarantee raw streams are public-safe. Original private streams remain local. `runtime-aggregate.json` records screen coverage and count-only candidate results.

Saved startup exit **0**, no stream errors, completion marker once in stderr at line **11501**. Existing overrides: `WOW_SIM_NO_SOUND=1`, `WOW_SIM_VERBOSE=1`, `WOW_SIM_DEBUG_NIL_GLOBALS=1`, `WOW_SIM_LOG_HANDLER_TIMINGS=0`; `--no-saved-vars` used. No timing flag was changed or rerun here. **11381 handler-duration records**, **91 handler-budget records**, **4 file-budget records** counted directly from complete stderr and reconciled with result.json.

## Warnings: prior 8 → current 4

Compare prior `fc824cb921342c06f506eb92e8c81d52bdf28fcd` startup-debug artifact `4c4730adbd8bc1c4b92988b138d6f941e325ffe7207c64ba73db264548fc8ecc` only, not prior startup-control. Prior stdout has eight `[failure]` details and total eight warnings; current has four details at lines **4440–4443** and total four warnings. Removed: three BOM-prefixed metadata missing-file warnings (EXBossData, EXBOSS-LocaleBase, EXBOSS-Locale) and one locale-excluded missing file (ExtraQuestButton/locale/enUS.lua). These four addons now report zero warnings. Four remaining warnings are EnhanceQoL quota failures, not removed or credited as fixed.

Prior timing threshold **1000**, current **0**. Duration-record volume is not performance, native-quota, timing-parity or causal timing evidence. Both load summaries report 48/78 addons, one failed during loading, zero load failures, one loaded with Lua errors during loading, 2934 Lua files and 149 XML files. Exit0/completion does not mean no errors.

## Seven remaining Lua signatures

Initial signatures match prior exactly and in order by full local string comparison; fingerprint hashes retained in safe aggregate. Exact occurrence counts include suppression summaries, not stdout warning restatements. **7 unique; 96 runtime occurrences = 7 initial + 89 suppressed**. Six quota signatures account for **95** occurrences; one nil-call signature accounts for **1**. All seven prior occurrence counts unchanged.

| ID | Static source | Class | Initial | Suppressed additional | Current occurrences | Prior occurrences | Current stderr line |
|---|---|---|---:|---:|---:|---:|---:|
| L1 | `EnhanceQoL/Settings/GroupTools.lua` | instruction-budget-exhausted | 1 | 0 | 1 | 1 | 2373 |
| L2 | `EnhanceQoL/Modules/Aura/FocusInterruptTracker.lua` | instruction-budget-exhausted | 1 | 0 | 1 | 1 | 2377 |
| L3 | `EnhanceQoL/Modules/Mouse/Init.lua` | instruction-budget-exhausted | 1 | 0 | 1 | 1 | 2381 |
| L4 | `EnhanceQoL/Modules/Food/Init.lua` | instruction-budget-exhausted | 1 | 0 | 1 | 1 | 2385 |
| L5 | `EnhanceQoL/Core/DynamicAnchors.lua:997` | instruction-budget-exhausted | 1 | 87 | 88 | 88 | 2437 |
| L6 | `AllTheThings/lib/EventRegistration.lua:26` | instruction-budget-exhausted | 1 | 2 | 3 | 3 | 3148 |
| L7 | `EllesmereUI/EllesmereUI_UICore.lua:1073` | nil-call | 1 | 0 | 1 | 1 | 4141 |

### Remaining four file quota boundaries

| Static source | Limit | Used before | Used after | Stderr line |
|---|---:|---:|---:|---:|
| `EnhanceQoL/Settings/GroupTools.lua` | 10000000 | 9986831 | 10000000 | 2372 |
| `EnhanceQoL/Modules/Aura/FocusInterruptTracker.lua` | 10000000 | 10000000 | 10000000 | 2376 |
| `EnhanceQoL/Modules/Mouse/Init.lua` | 10000000 | 10000000 | 10000000 | 2380 |
| `EnhanceQoL/Modules/Food/Init.lua` | 10000000 | 10000000 | 10000000 | 2384 |

First file consumes remaining 13169 dispatch instructions; subsequent three enter at the same exhausted owner limit. These counters describe this simulator receipt, not native quota-period/reset/exemption semantics. No quotas or policy changed.

## Four actual native TOC input hashes

Read-only local file hashing matches all four compiler-input receipts; runtime result also records all four unchanged. No native client executed. This seals these four input bytes only, not complete addon/cache/inherited-environment inputs.

| Addon | Bytes | SHA-256 | Result |
|---|---:|---|---|
| EXBOSS-Locale | 406 | `2a5de6e4c1a3e99f22c1f907d905762277284886199cb4f7dc00853a6dac3c9b` | unchanged |
| EXBOSS-LocaleBase | 416 | `973af1c202aae181d2e9b9fa4c60f70ec3d1675c55a5e862273d4d0d52e789a8` | unchanged |
| EXBossData | 797 | `c1f31fd314f83546cf28efadae5e8cbabff41f47a4018bb58c0bba72f3ef8af8` | unchanged |
| ExtraQuestButton | 787 | `89f08ed8bc9da60d4929006ce2756fccc5a47ae25887f97fd5d5f18f38793ace` | unchanged |

## Evidence ledger and exclusions

`runtime-aggregate.json` contains safe derived counts; `runtime-hashmanifest.json` seals every case stream/receipt, listing, current/prior startup stream/receipt, source snapshots, compiler selection/seals and generated report/aggregate. Hash manifest intentionally does not hash itself. Existing artifacts retained without rewriting. Source/artifact claims remain scoped to recorded historical receipts; external dependencies, unsealed runtime inputs, other profiles, full suites and current/future code excluded. Errors remain; root-warning reduction is not clean-startup acceptance.
