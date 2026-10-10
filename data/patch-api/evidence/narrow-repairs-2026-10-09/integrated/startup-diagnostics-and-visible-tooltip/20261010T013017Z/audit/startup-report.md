# Startup diagnostic artifact audit

## Verdict and scope

**PASS: existing-artifact diagnostic/completion audit. NOT a clean-startup pass.** Both CLI variants exited 0 and reached their completion marker on stderr, while retaining Lua errors. No reruns, commands against the simulator, Cargo, tests, network, delegation, deployment, or source/native/other-profile verification performed. Full stdout and stderr read locally for both variants; inputs checked byte-stable before aggregate publication.

Epoch: `20261010T013017Z`. Both submissions identify normal `target/debug/wow-sim`, artifact SHA-256 `4c4730adbd8bc1c4b92988b138d6f941e325ffe7207c64ba73db264548fc8ecc`, compiled source revision `fc824cb921342c06f506eb92e8c81d52bdf28fcd`. These are recorded provenance, not a source audit or a newly performed binary verification. Both result files report unchanged artifact hash and no stream errors.

| Evidence | Debug | Control |
|---|---:|---:|
| UTC start, October 10, 2026 | 01:36:20.793715 | 01:37:37.535375 |
| UTC end, October 10, 2026 | 01:36:33.138006 | 01:37:50.499038 |
| stdout bytes / lines | 314605 / 4606 | 238178 / 4097 |
| stderr bytes / lines | 33928 / 454 | 21217 / 359 |
| Exit | 0 | 0 |
| Completion marker stderr line | 415 | 320 |
| Unique Lua signatures / runtime occurrences | 7 / 96 | 7 / 105 |
| File / handler diagnostic records | 4 / 91 | 0 / 0 |
| Reported warnings / detailed failure records | 8 / 8 | 8 / 0 |

## Privacy screen

Raw logs include private identity/cache information outside error diagnostics. No raw log excerpts, account/character/realm identifiers, cache identifiers, SavedVariables values, or arbitrary Lua arguments/payloads are copied into this report or aggregate. Exported source references are public addon-relative resource names. All eight warning details and all seven Lua signatures were screened: their retained content is resource identity, OS error class, budget exhaustion, or nil-call location, not private runtime payload. Invisible BOM in three warning filenames is rendered explicitly as `U+FEFF`. Raw log contents remain only in existing evidence files. The hash manifest fingerprints those originals without reproducing them.

## Lua signatures and exact occurrence accounting

First-error text is byte-identical and in identical order between variants after removing only the `Lua error: ` prefix, compared locally. Safe signatures below retain source and error class. File errors have `@Interface/AddOns/` source prefix and nested `Lua error:` text; handler errors have `[OnEvent]`, frame/addon/source context; L7 is a direct `Interface/AddOns/` nil-call error.

| ID | Addon-relative source | Error meaning | Debug first stderr line; occurrences | Control first stderr line; occurrences |
|---|---|---|---:|---:|
| L1 | EnhanceQoL/Settings/GroupTools.lua | instruction budget exhausted for owner EnhanceQoL | 165; 1 | 164; 1 |
| L2 | EnhanceQoL/Modules/Aura/FocusInterruptTracker.lua | instruction budget exhausted for owner EnhanceQoL | 169; 1 | 167; 1 |
| L3 | EnhanceQoL/Modules/Mouse/Init.lua | instruction budget exhausted for owner EnhanceQoL | 173; 1 | 170; 1 |
| L4 | EnhanceQoL/Modules/Food/Init.lua | instruction budget exhausted for owner EnhanceQoL | 177; 1 | 173; 1 |
| L5 | EnhanceQoL/Core/DynamicAnchors.lua:997 | OnEvent, frame #69210, owner EnhanceQoL budget exhausted | 181; 88 | 176; 96 |
| L6 | AllTheThings/lib/EventRegistration.lua:26 | OnEvent, frame #48610, owner AllTheThings budget exhausted | 335; 3 | 270; 4 |
| L7 | EllesmereUI/EllesmereUI_UICore.lua:1073 | attempt to call a nil value | 412; 1 | 317; 1 |

Each stream has seven initial error records plus two suppression summaries. Debug stderr 449 says L5 suppressed **87 additional** times; 452 says L6 suppressed **2 additional** times. Control stderr 354 and 357 report **95** and **3 additional**, respectively. Thus totals are 96 and 105, not nine occurrences. Each error/suppression summary is followed by a two-line traceback, `(string):1: in main chunk`.

Debug stdout 4448–4451 repeats L1–L4 as XML/file warning details; these are not four additional runtime executions. Counting every line containing `Lua error` would double-count these and confuse suppression summaries with individual failures.

## All eight warning details

All details occur on debug stdout, not stderr. Control omits detailed records but retains the same five addon warning totals (1 + 1 + 1 + 1 + 4 = 8).

| Debug stdout line | Resource, relative to AddOns | Warning text meaning |
|---:|---|---|
| 3665 | EXBossData/`U+FEFF## Interface: 120100` | IO error: No such file or directory (os error 2). The log identifies a BOM-prefixed interface metadata string as a requested filename; this is not a Lua error. |
| 3746 | ExtraQuestButton/locale/enUS.lua | IO error: No such file or directory (os error 2); missing locale resource. |
| 3866 | EXBOSS-LocaleBase/`U+FEFF## Interface: 120100` | Same missing-file OS error for a BOM-prefixed metadata filename. |
| 3881 | EXBOSS-Locale/`U+FEFF## Interface: 120100` | Same missing-file OS error for a BOM-prefixed metadata filename. |
| 4448 | EnhanceQoL/Settings/SettingsUI.xml → Settings/GroupTools.lua | Lua error: instruction budget exhausted for owner EnhanceQoL; L1 restatement. |
| 4449 | EnhanceQoL/Modules/Aura/Aura.xml → Modules/Aura/FocusInterruptTracker.lua | Same owner-budget exhaustion; L2 restatement. |
| 4450 | EnhanceQoL/Modules/Mouse/Mouse.xml → Modules/Mouse/Init.lua | Same owner-budget exhaustion; L3 restatement. |
| 4451 | EnhanceQoL/Modules/Food/Food.xml → Modules/Food/Init.lua | Same owner-budget exhaustion; L4 restatement. |

The BOM/metadata filenames are observed failed requests, not a source-level root-cause finding. No source or filesystem diagnosis performed.

Both stdout summaries distinguish **Loaded: 48/78 addons**, **Failed during loading: 1**, **Load failures: 0**, **Loaded with Lua errors during loading: 1**, and **Total: 2934 Lua files, 149 XML files, 8 warnings**. Debug lines 4568–4572; control 4022–4026. Preserve these distinct counters: eight file warnings do not mean eight failed addons, and the summary does not identify its single failed-during-loading addon. Do not classify the other 30 addons from the fraction alone.

## Budget interpretation

Debug file records are on stderr 164, 168, 172, 176, immediately before L1–L4 and their tracebacks. Each belongs to EnhanceQoL and has limit 10,000,000.

| File | used_before | used_after | Remaining at entry | Consumption inside this recorded interval |
|---|---:|---:|---:|---:|
| Settings/GroupTools.lua | 9,986,831 | 10,000,000 | 13,169 | 13,169 |
| Modules/Aura/FocusInterruptTracker.lua | 10,000,000 | 10,000,000 | 0 | 0 |
| Modules/Mouse/Init.lua | 10,000,000 | 10,000,000 | 0 | 0 |
| Modules/Food/Init.lua | 10,000,000 | 10,000,000 | 0 | 0 |

The first file consumed the remaining **13,169 dispatch instructions**, not the entire ten-million-owner allowance. Its preceding 9,986,831 usage is cumulative owner state. The next three entered already exhausted; zero counter delta does not establish successful execution or zero attempted work. These counters are not elapsed time, Rust work, or a whole-file cost measurement.

There are **91 handler-budget-error records**, all on debug stderr 180–391: 88 EnhanceQoL, 3 AllTheThings. Ninety show 10,000,000 → 10,000,000. One, AllTheThings frame #48610 / PLAYER_LOGIN at line 334, shows 770,564 → 10,000,000: remaining/consumed interval 9,229,436. Events: ADDON_LOADED 74, DISPLAY_SIZE_CHANGED 2, UI_SCALE_CHANGED 2, PLAYER_LOGIN 8, PLAYER_ENTERING_WORLD 4, BAG_UPDATE_DELAYED 1.

`handler-budget-error` labels a failed owner-budgeted callback, **not necessarily an exhaustion-caused error**. Counter state alone is not causal evidence. Here the separately printed deduplicated L5/L6 messages explicitly say exhaustion; their occurrence totals align with owner counts, but that does not convert the diagnostic category into a universal causal label. No native quota/reset/exemption/parity conclusions.

## Control comparison and log boundaries

Debug submission enables `WOW_SIM_VERBOSE=1`, `WOW_SIM_DEBUG_NIL_GLOBALS=1`, and `WOW_SIM_LOG_HANDLER_TIMINGS=1000`; control retains verbosity but explicitly unsets both diagnostic variables. Both disable sound and use `--no-saved-vars`; matching dimensions are 1600×1200. These are separate invocations with distinct deliberately nonmatching tree filters and completion labels, not one invocation toggled in place.

| Added detail/tag | Debug stdout / stderr | Control stdout / stderr |
|---|---:|---:|
| nil-observation | 328 / 0 | 0 / 0 |
| missing-requirement | 210 / 0 | 0 / 0 |
| failure | 8 / 0 | 0 / 0 |
| file-budget-error | 0 / 4 | 0 / 0 |
| handler-budget-error | 0 / 91 | 0 / 0 |
| handler-timing / handler-timings duration records | 0 / 0 | 0 / 0 |

Control absence of added logs is established for these captured artifacts. Debug absence of duration records at its 1000 ms threshold does not prove timing emission for a slow handler. Existing addon duration summaries remain in both variants and are not added handler-duration records.

Within stderr, file errors precede the first EnhanceQoL handler error, then AllTheThings, then the nil-call, then completion. Debug completion is `[StartupBudgetCaptureComplete]` at line 415; control is `[StartupBudgetControlComplete]` at 320. Completion is not the end of output: post-marker frame diagnostics/tree heading follow, then suppression summaries at 449/452 or 354/357, with tracebacks through final lines 454/359. There are no initial Lua error records after either marker; deferred suppression summaries are not new post-marker failures.

Stdout carries file-loading/addon summaries, warning/nil/missing-requirement details when enabled, and tree-related output. Stderr carries startup/event diagnostics, Lua errors, budget records when enabled, completion markers, and deferred suppression summaries. Separate files cannot establish a total cross-stream ordering; line references establish order only within each stream.

**Unchanged:** exact ordered initial error signatures, exit/completion success, and the reported load/warning counters. **Changed:** emitted diagnostic detail and repeated-error frequencies (L5 88→96; L6 3→4). Therefore no claim of identical error multiplicity, execution timing, or full behavior invariance.

## Deliverables

`startup-aggregate.json` contains sanitized counts, public source references, line references, and all 95 numeric budget records. `startup-hashmanifest.json` records SHA-256/byte sizes for both variants' original streams/submissions/results, artifact-selection metadata, report, and aggregate. It excludes itself to avoid recursive hashing. Existing compiler audit remains untouched. Source/native/all-profile/visible-tooltip proof is outside this startup audit.
