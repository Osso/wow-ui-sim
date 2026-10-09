# Standalone startup-warning audit

**FAIL — clean startup. PASS — bounded capture completed.** No budget fix, regression fix, native parity, or broader readiness claim.

## Evidence and execution boundary

Audited existing capture `20261009T230718Z`, not a new run. Native read and Pyrun/file tools only; independent verifier role performed directly, with no recursive gate, delegation, model change, builds, tests, services, source edits, or capture reruns. Only the four authorized audit outputs were written.

Full streams consumed and classified: **stdout 314,634 bytes / 4,636 lines; stderr 21,215 bytes / 359 lines; zero unclassified lines**. Selected diagnostics retain 1-based stream line references. Input and output hashes are in `hashmanifest.sha256`; full originals remain in the capture directory, not copied into diagnostics.

Recorded invocation: controlled `target/debug/wow-sim`, `--no-saved-vars`, exec-print marker, headless `dump-tree`, no-match frame filter, **1600×1200**, 90-second bound. Third-party loading remained enabled; filtering applies to tree output, not warning streams. Public overrides recorded: `WOW_SIM_NO_SOUND=1`, `WOW_SIM_VERBOSE=1`, `WOW_SIM_DEBUG_NIL_GLOBALS=1`. Runtime sound disabled (stdout:1); sound/gui remain compiled features, not exercised GUI/audio paths.

## Lua errors: distinct signatures, not substring counts

| First stderr line | Diagnostic | Recorded occurrences | Suppressed additional |
|---|---|---:|---:|
| 164 | EnhanceQoL `Settings/GroupTools.lua`: owner instruction budget exhausted | 1 | 0 |
| 167 | EnhanceQoL `Modules/Aura/FocusInterruptTracker.lua`: same failure class | 1 | 0 |
| 170 | EnhanceQoL `Modules/Mouse/Init.lua`: same failure class | 1 | 0 |
| 173 | EnhanceQoL `Modules/Food/Init.lua`: same failure class | 1 | 0 |
| 176 | EnhanceQoL OnEvent, `Core/DynamicAnchors.lua:997`: owner instruction budget exhausted | 88 | 87 |
| 270 | AllTheThings OnEvent, `lib/EventRegistration.lua:26`: owner instruction budget exhausted | 3 | 2 |
| 317 | EllesmereUI `EllesmereUI_UICore.lua:1073`: attempt to call a nil value | 1 | 0 |

**7 distinct logged signatures; 7 initial error records + 89 suppressed additional occurrences = 96 occurrences.** Of these, 95 are budget-exhaustion occurrences and 1 is a nil call. Stderr:354 and :357 repeat existing signatures with suppression counts; they are not two new errors. The seven tracebacks and two summary tracebacks are not extra occurrences. Nested “Lua error:” inside a header is not another error. Counts describe the emitted error ledger, not unreported failures or caught Lua errors.

Compiled-revision `src/lua_errors.rs:243–255` explicitly renders suppression as `count - 1`. No suggestion that budgeting is fixed: exhaustion is observed in this capture.

## Addon loading and conditional diagnostics

Pre-startup-event summary (stdout:4560–4566): **Loaded 48/78; Failed during loading 1; Load failures 0; Loaded with Lua errors during loading 1; 2,934 Lua files; 149 XML files; 8 warnings; total 6.78s**. These are different counters, not contradictory pass/fail verdicts. Revision `src/bin/wow_sim/addon_loading.rs:453–493,693–701` counts load-failed/Lua-error addon union separately from successful loads. EnhanceQoL accounts for the observed loading-phase Lua-error addon; later AllTheThings/EllesmereUI errors occur after that summary. **The other 30 are not proven load failures**: eligibility, enabled-state, already-loaded state and bootstrap-only handling affect the denominator. Their individual dispositions are not logged here and were not inferred from private profiles.

| Addon | stdout status line | Loader warnings | Detailed stdout failure lines |
|---|---:|---:|---:|
| EXBossData | 3664 | 1 | 0 |
| ExtraQuestButton | 3744 | 1 | 1 |
| EXBOSS-LocaleBase | 3864 | 1 | 0 |
| EXBOSS-Locale | 3878 | 1 | 0 |
| EnhanceQoL | 4444 | 4 | 0 |

ExtraQuestButton's sole detailed loader failure (stdout:3745) is missing `Interface/AddOns/ExtraQuestButton/locale/enUS.lua`, IO “No such file or directory (os error 2)”. It is a loader warning, not an additional Lua-error occurrence. Four EnhanceQoL warnings coincide with its four file-execution budget errors, but the eight-warning counter must not be added to 96 as though disjoint events.

The compiled-revision `addon_loading/warning_report.rs:3–78` requires both the debug flag and membership in `VERBOSE_WARNING_ADDONS`. ExtraQuestButton is listed; EXBossData, EXBOSS-LocaleBase, EXBOSS-Locale and EnhanceQoL are not. Thus **7/8 warning details are absent on stdout despite the debug flag**. EnhanceQoL's Lua errors are independently visible on stderr; the remaining nonallowlisted warning texts cannot be reconstructed honestly. Blizzard diagnostic details use the debug flag without that addon allowlist (`addon_loading.rs:182–194`).

All 341 status rows are retained as structured counts: 293 Blizzard/plain rows, 48 third-party rows. Blizzard/plain totals: 0 warnings, 185 nil observations, 97 missing requirements. Third-party totals: 8 warnings, 165 nil observations, 125 missing requirements. Across status rows: **350 nil observations and 222 missing requirements**, compared with **328 and 210 emitted detail lines**; the remaining **22/12 details** are conditionally absent. Nil observations/missing requirements are diagnostic classifications, not actual Lua errors. Loading file/status totals do not prove each addon's runtime functionality or cover every late-demand-load result.

## Completion, startup and ticks

Service record: `Result=success`, `ExecMainStatus=0`, exited; main execution **October 9, 2026, 23:07:18–23:07:34 UTC** (18:07:18–18:07:34 CDT). `ExecMainCode=1` denotes normal exited status; it is not exit code 1. Submission/status-query exits 0 are distinct from main status 0.

Startup traces cover login, world entry, post-login events and FIRST_FRAME_RENDERED through stderr:316. Login 2.36s, world entry 607.60ms, post-login events 162.33ms. `EDIT_MODE_LAYOUTS_UPDATED` was explicitly skipped (stderr:275). **Completion marker appears exactly once at stderr:320**, after the EllesmereUI error. Later layout diagnostics and the final Frame Tree header (stderr:328–352) precede the suppression summary. An empty filtered tree is expected from the chosen no-match filter; no visual/layout correctness proof follows.

At recorded revision, `src/startup.rs:250–267` settles events/timers and calls **6 OnUpdate ticks before exec**; `src/bin/wow_sim/main.rs:597–619` calls **3 further ticks after exec**, then layout/tree output. Each tick passes 0.016 (`src/startup.rs:159–180`). This is **source-path evidence for 9 tick calls**, not independently instrumented tick counts or a sustained GUI loop. The marker proves exec was reached, not that every handler succeeded; service completion/tree output provide the subsequent path evidence. No explicit tick failure line appears in either stream.

## SavedVariables and cache provenance

`SavedVariables loading disabled` (stdout:2) does **not** mean all WTF/cache reads were disabled. Stdout:3 names an EditMode cache source (identity redacted), and stdout:4 confirms **“Loaded EditMode layout cache from WTF”**. Stderr:29–34 covers its configuration/load. Recorded-revision `src/bin/wow_sim/startup.rs:91–109,133–147` creates a separate EditMode cache manager when ordinary SavedVariables are absent; `saved_var_config.rs:24–37` configures that read-only source. ServerSnapshot layout-read timing alone is not proof of imported snapshot data: source returns immediately when the main SavedVariables manager is absent (startup.rs:112–130). The 12.53µs SavedVars timing in the summary is not proof of imported SavedVariables. No private profile files inspected or private identity/layout/configuration values published; no claim about cache-write behavior beyond this evidence.

Recorded source revision **`1fac15bd0e477cd0fdf1f03dd0219dbbedfd4111`** is available in Git. Submission includes Cargo `compiler-artifact` for executable `wow-sim`, debug/dev opt-level 1, `fresh=false`; this is recorded build provenance, not a reconstructed build or independent attestation of a pristine entire build tree. Direct current binary hashing matches its captured SHA-256 **`a5e7c397129ec917ea32eb081bf5631b3cdccb8ae69c5e8c997cac981885988d`**. Relevant warning/error-source files also matched their recorded-revision contents when checked. Current/new test or source work is not validated by this earlier binary epoch.

Retail Blizzard cache provenance: product `wow`, version **12.1.0.69933**, build key `dcfc90fffd79ba00406ae46f5f657592`, source `casc-local-or-cdn`, fallback `none`, manifest SHA-256 **`aa7dfec3fb3bc9440a8c737c2274363502cb3d22c7939b8a570a58e717202edd`**. Recorded cache marker/UTF-8/Runeforge checks were true. Revision manifest hashing matches that provenance. All **4,044 pre-capture entry hashes, including completion/provenance files, match the current cache; 0 changed, 0 missing**. This compares pre-capture with audit-time state, not a continuous immutability guarantee. No post-capture cache snapshot or third-party-content hash inventory exists in this capture, so third-party exact-file provenance remains unproven. Main Blizzard bytecode cache reports 1,569/1,569 hits; third-party summary reports 3/2,957 hits, 2,954 misses/stores, zero replay/store failures (stdout:2725,4578). This is not a cold-cache run.

## Coverage verdict and exclusions

| Requested check | Coverage / verdict |
|---|---|
| Full captured stdout/stderr, actual errors, duplicates/suppression | COMPLETE; clean-startup FAIL |
| Loader counts, summary, failures, conditional allowlist | COMPLETE for emitted evidence; 7 warning texts and 22/12 nil/requirement details absent by design |
| Marker, service exit, startup events, headless ticks | COMPLETE artifact/source-path audit; completion PASS, no sustained-runtime claim |
| Binary/cache provenance and SavedVariables/EditMode behavior | COMPLETE within recorded metadata plus read-only hash comparison; private profiles excluded |
| User-requested independent Lua/startup-warning verifier check | Performed directly by this replacement verifier; no delegated subagent spawned, no fresh execution requested/performed |

Excluded: builds/tests/reruns, current test-source validation, budgeting fixes, native-client parity, other client profiles, GUI/GPU/audio behavior, visual/tree correctness, addon interactions, sustained timers/OnUpdate, SavedVariables-enabled startup, private profiles, silent caught failures, and warning details never emitted. No fix or clean-startup claim.

Privacy screening is recorded in `privacy.json`. Native read tools worked. Native grep failed twice with exactly `ripgrep (rg) is not available on PATH`; read-only Pyrun file scanning completed the search without backend fallback. One early Pyrun evaluation failed with `name 'texts' is not defined`; subsequent evaluations loaded their own inputs. Neither issue prevented full capture coverage.
