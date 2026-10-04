
## Step 1: rebase
CommandResult(stdout='', stderr='\n\x1b[KSuccessfully rebased and updated refs/heads/p1207-r8.\n', exit_code=0, upstream_results=())

## Active goal
Integrate B08–B10 in p1207-r8 using existing provider; complete behavioral RED/GREEN, intersecting narrow controls, formatting/check and attributed commit. Exclude vendor/cache changes, coverage/wiki changes, agents, alternate profiles, startup CLI, push/merge/deploy/PR.

## Step 2: anchor/evidence inspection
E01: state; `src/c_api/duration_text_binding.rs`; OLD matches 1
E02: state; `src/c_api/duration_text_binding.rs`; OLD matches 1
E03: producer; `src/c_api/duration_text_binding.rs`; OLD matches 1
E04: producer; `src/c_api/duration_text_binding.rs`; OLD matches 1
E05: producer; `src/c_api/duration_text_binding.rs`; OLD matches 1
E06: state; `src/c_api/duration_text_binding.rs`; OLD matches 1
E07: producer; `src/c_api/duration_text_binding.rs`; OLD matches 1
E08: producer; `src/c_api/duration_text_binding.rs`; OLD matches 1
E09: producer; `src/c_api/duration_text_binding.rs`; OLD matches 1
E10: producer; `src/c_api/duration_text_binding.rs`; OLD matches 1
E11: producer; `src/c_api/duration_text_binding.rs`; OLD matches 1
E12: producer; `src/c_api/duration_text_binding.rs`; OLD matches 1
E13: state; `src/c_api/duration_text_binding.rs`; OLD matches 1
E14: test-support; `src/lua_api/globals/lua_duration_object.rs`; OLD matches 1
E15: producer; `src/lua_api/globals/lua_duration_object/core.rs`; OLD matches 1
E16: test-support; `src/loader/tests/wow_api_globals/startup_globals.rs`; OLD matches 1
E17: test-support; `src/loader/tests/wow_api_globals/startup_globals.rs`; OLD matches 1
E18: test-support; `src/loader/tests/wow_api_globals/startup_globals.rs`; OLD matches 1
E19: test-support; `src/loader/tests/wow_api_globals/startup_globals.rs`; OLD matches 1
E20: test-support; `src/loader/tests/wow_api_globals/startup_globals.rs`; OLD matches 1
E21: test-support; `src/loader/tests/wow_api_globals/startup_globals.rs`; OLD matches 1
E22: test-support; `src/c_api/duration_text_binding.rs`; OLD matches 1
HasExpired evidence: cached LuaDurationObjectAPIDocumentation.lua:338: "Returns true once the duration has reached its end time." This alone leaves zero-span unspecified. docs/specs/duration-core.md:47 explicitly states "Retail 12.0.5+ reports `HasExpired=true` regardless of start/clock" and calls this "an explicit simulator inference". Decision: correct E16 to true for the existing specified simulator contract, not native parity. No producer zero-span change.

## Step 3: RED scaffolding
Applied non-producer anchors and three new files. Producers withheld. All anchors matched; no master adaptations. Base 4ed5bb0b6b5504092a9dfd7b0afd305e5bb4302d.

RED proof invocation: CARGO_BUILD_JOBS=4 BUILD_HOST_SCRIPTS=/home/osso-test/Projects/world-of-osso/game-engine/scripts python3 scripts/build-host.py --build-host local --test --target-dir /home/osso-test/.cache/wow-ui-sim-target-b102 --test integration patch_12_0_7_duration_text_binding:: -- --test-threads=1; scope RED scaffolding working tree, no prior proof.
RED command exit 101; log `/home/osso-test/.cache/wow-ui-sim-audit/r8-RED-patch_12_0_7_duration_text_binding.log`.

## Step 4: RED observed
New module: 1 passed / 8 failed, all behavioral; compiled successfully, no warnings. Cadence case already passes existing scheduling. Actual binary target is b102 (runner banner incorrectly prints worktree/target). Applying ten producer anchors.
Producers applied; cargo fmt exit 0. Existing provider/method inventory retained.

## Step 5: initial GREEN
Commit 338e6f1c5290a65a83453423e410f2bdacf962fa. Prior RED invalidated by producers. New module invocation: ['scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b102', '--test', 'integration', 'patch_12_0_7_duration_text_binding::', '--', '--test-threads=1'] with prescribed jobs/host env.
GREEN exit 101; log `/home/osso-test/.cache/wow-ui-sim-audit/r8-GREEN-patch_12_0_7_duration_text_binding.log`.

## Step 6: diagnose secret boundary
Initial GREEN 8/9; secret timing test fails unlabeled assertion. Added labels without changing assertions to locate first failing public boundary.
Diagnostic test exit 101, log /home/osso-test/.cache/wow-ui-sim-audit/r8-GREEN-secret-diagnostic.log; assertion labels only since 338e6f1c5.
Correction: prior label edit did not apply (Lua assertions omit semicolons); redundant diagnostic prior to labels did not add proof. Actual labels now applied.
Labeled diagnostic exit 101; log /home/osso-test/.cache/wow-ui-sim-audit/r8-GREEN-secret-labeled.log.

## Step 7: secret test adaptation
First failing boundary is FontString.GetText, not binding formatting/SetText. text.rs:571–606 returns plain string for secure callers; secret_origin.rs unwrap_input preserves origin separately. Adapted assertion to observe public SetText input wrapper and actual rendered text; excluded any change to widget readout/native output tagging. Other secret assertions unchanged.
Adapted GREEN new-module exit 0; log `/home/osso-test/.cache/wow-ui-sim-audit/r8-GREEN-patch_12_0_7_duration_text_binding-final.log`.

## Control duration_core::
Proof ledger checked: no earlier execution on 56fa3ff6e5a3d62823f0bfe72f8963fbb728760b; ['scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b102', '--test', 'integration', 'duration_core::', '--', '--test-threads=1'] prescribed jobs/host env.
Exit 0; `/home/osso-test/.cache/wow-ui-sim-audit/r8-GREEN-duration_core.log`; test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 10411 filtered out; finished in 4.75s

## Control patch_12_0_7_duration_clocks::
Proof ledger checked: no earlier execution on 56fa3ff6e5a3d62823f0bfe72f8963fbb728760b; ['scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b102', '--test', 'integration', 'patch_12_0_7_duration_clocks::', '--', '--test-threads=1'] prescribed jobs/host env.
Exit 0; `/home/osso-test/.cache/wow-ui-sim-audit/r8-GREEN-patch_12_0_7_duration_clocks.log`; test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 10429 filtered out; finished in 1.61s

## Control startup_globals::
Proof ledger checked: no earlier execution on 56fa3ff6e5a3d62823f0bfe72f8963fbb728760b; ['scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b102', '--lib', 'startup_globals::', '--', '--test-threads=1'] prescribed jobs/host env.
Exit 101; `/home/osso-test/.cache/wow-ui-sim-audit/r8-GREEN-startup_globals.log`; 

## Step 8: controls and lib build interruption
New module final 9/9, duration_core 27/27, clocks 9/9. startup_globals produced no test results: rustc terminated SIGTERM, not a Rust diagnostic or behavioral failure. Inspecting runner/process state before bounded retry; lib controls still required, prior narrow integration proof does not cover modified lib fixtures.
Process inspection: no remaining r8 cargo/rustc process after SIGTERM. Cause of external termination unknown. Authorized bounded retry once because required lib fixture has no evidence; no clean (no undefined symbols).
Lib retry exit 101; log `/home/osso-test/.cache/wow-ui-sim-audit/r8-GREEN-startup_globals-retry.log`; test result: FAILED. 25 passed; 1 failed; 0 ignored; 0 measured; 1949 filtered out; finished in 2.23s

## Step 9: lib fixture adaptation
startup_globals retry 25 passed/1 failed. Existing 12.1 lifetime fixture uses numeric pseudo-duration 17; strict declared DurationObject setter rejects it. Adapted only fixture to real duration, preserving GC retention/identity assertions. No provider fallback or fixture exclusion.

## Control startup_globals::
Revision f52a146e4cd9e9ef2bd4479e9612e89199974dc8; prior startup proof invalidated by fixture change, others not yet run. ['scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b102', '--lib', 'startup_globals::', '--', '--test-threads=1']; prescribed env.
Exit 0; `/home/osso-test/.cache/wow-ui-sim-audit/r8-GREEN-startup_globals-final.log`; test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 1949 filtered out; finished in 2.12s

## Control duration_text_binding::tests::
Revision f52a146e4cd9e9ef2bd4479e9612e89199974dc8; prior startup proof invalidated by fixture change, others not yet run. ['scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b102', '--lib', 'duration_text_binding::tests::', '--', '--test-threads=1']; prescribed env.
Exit 0; `/home/osso-test/.cache/wow-ui-sim-audit/r8-GREEN-duration_text_binding-lib.log`; test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1974 filtered out; finished in 0.13s

## Control duration_text_binding_copy::
Revision f52a146e4cd9e9ef2bd4479e9612e89199974dc8; prior startup proof invalidated by fixture change, others not yet run. ['scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b102', '--test', 'integration', 'duration_text_binding_copy::', '--', '--test-threads=1']; prescribed env.
Exit 101; `/home/osso-test/.cache/wow-ui-sim-audit/r8-GREEN-duration_text_binding_copy.log`; test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10437 filtered out; finished in 0.48s
test result: FAILED. 0 passed; 7 failed; 0 ignored; 0 measured; 10431 filtered out; finished in 1.00s

## Step 10: retained copy controls
Handoff incorrectly described copy module as entirely Forever-gated: seven cases are ungated, four secret cases gated. Control 0/7 because numeric pseudo-durations/table Format callbacks violate declared input. Adapted seven fixture groups to real duration/manual clock and NumericRuleFormatter; preserve GC/identity/copy/components/actual cached aura assertions. Empty bindings now assert unconfigured error instead of fabricated text. No cached consumers edited.

## Control duration_text_binding_copy::
Revision 226a907a7; prior copy proof invalidated by fixture changes, others new scope; argv ['scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b102', '--test', 'integration', 'duration_text_binding_copy::', '--', '--test-threads=1']; prescribed env.
Exit 101; `/home/osso-test/.cache/wow-ui-sim-audit/r8-GREEN-duration_text_binding_copy-adapted.log`; test result: FAILED. 6 passed; 1 failed; 0 ignored; 0 measured; 10431 filtered out; finished in 2.01s

## Step 11: copy control correction
Adapted copy control 6/7, including actual cached CustomAuraButton path PASS. Secure-option-copy fixture lacked formatter; automatic UpdateFontString skipped unconfigured format. Added explicit NumericRuleFormatter to fixture; assertions preserved.

## Control duration_text_binding_copy::
Scope copy fixture correction (previous copy invalidated); other controls not run yet. argv ['scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b102', '--test', 'integration', 'duration_text_binding_copy::', '--', '--test-threads=1']; prescribed env.
Exit 0; `/home/osso-test/.cache/wow-ui-sim-audit/r8-GREEN-duration_text_binding_copy-final.log`; test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 10431 filtered out; finished in 2.08s

## Control duration_text_binding_tick::
Scope copy fixture correction (previous copy invalidated); other controls not run yet. argv ['scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b102', '--test', 'integration', 'duration_text_binding_tick::', '--', '--test-threads=1']; prescribed env.
Exit 0; `/home/osso-test/.cache/wow-ui-sim-audit/r8-GREEN-duration_text_binding_tick.log`; test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 10438 filtered out; finished in 0.00s

## Control duration_numeric_formatters::
Scope copy fixture correction (previous copy invalidated); other controls not run yet. argv ['scripts/build-host.py', '--build-host', 'local', '--test', '--target-dir', '/home/osso-test/.cache/wow-ui-sim-target-b102', '--test', 'integration', 'duration_numeric_formatters::', '--', '--test-threads=1']; prescribed env.
Exit 0; `/home/osso-test/.cache/wow-ui-sim-audit/r8-GREEN-duration_numeric_formatters.log`; test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 10433 filtered out; finished in 0.93s

## Step 12: retained controls complete
Copy 7/7 including cached consumer; tick filter 0 tests (Forever-gated, no proof credit); numeric formatters 5/5. Runtime producer unchanged since 338e6f1c5; later changes only tests. Prior new-module/core/clocks/lib evidence remains valid for relevant source hashes.

## Step 13: spec/readability audit
Updated only slice spec: checked behavioral requirements supported by logs; retained unknown historical/default/identity/coercion/earlier-profile/weak-registry/overridden-query gaps. Rust readability reviewed manually: scalar helper functions bounded, no warning suppressions or untraced reference payloads added; changed registration/core branches and concrete test fixtures inspected. No agents/models invoked.

## Step 14: fmt-check
Revision 0c361c75f9837da64675fbf2e78d1177d913a740; prior targeted behavior proof retained (last change spec-only). CARGO_BUILD_JOBS=4 cargo fmt --check; explicit worktree cwd. First final invocation.
Exit 0; log `/home/osso-test/.cache/wow-ui-sim-audit/r8-GREEN-fmt-check.log`.

## Step 14: cargo-check
Revision 0c361c75f9837da64675fbf2e78d1177d913a740; prior targeted behavior proof retained (last change spec-only). CARGO_BUILD_JOBS=4 cargo check --target-dir /home/osso-test/.cache/wow-ui-sim-target-b102 --jobs 4; explicit worktree cwd. First final invocation.
Exit 0; log `/home/osso-test/.cache/wow-ui-sim-audit/r8-GREEN-cargo-check.log`.

## Step 15: final state inspection
Format check and Cargo check PASS on 0c361c75f9837da64675fbf2e78d1177d913a740, no warnings. Git diff --check PASS; clean worktree. All edits/builds/commits confined to authorized worktree and audit/target artifacts. No push, merge, deployment, PR, vendor/cache edits, agents, models or startup CLI. Master movement after initial rebase is outside this slice; final canonical merge/rebase remains main-session responsibility.

# Final integration result — 2026-10-04

Branch `p1207-r8`; worktree `/home/osso-test/.worktrees/wow-ui-sim-p1207-r8`; final HEAD `0c361c75f9837da64675fbf2e78d1177d913a740`; clean. Initial rebase succeeded onto `4ed5bb0b6b5504092a9dfd7b0afd305e5bb4302d`. Master subsequently moved to `8149b7fa6200e42dc6a46e203627a7376a3ff2c5`; no integration into that newer master was requested or performed.

## Commits

| SHA | Scope |
|---|---|
| `338e6f1c5290a65a83453423e410f2bdacf962fa` | Provider, scalar state, core authentication, nine new tests, initial spec and lib fixtures |
| `56fa3ff6e5a3d62823f0bfe72f8963fbb728760b` | Public secret SetText handoff assertion, diagnostic labels |
| `f52a146e4cd9e9ef2bd4479e9612e89199974dc8` | Retained lib duration reference fixture |
| `226a907a735860654a63e54b92fb5d91ad6530a1` | Seven retained Copy/Assign fixtures use actual duration/formatters |
| `35b2c3e63ff614b30b105868acf328e608e611e1` | Explicit secure-option-copy formatter fixture |
| `0c361c75f9837da64675fbf2e78d1177d913a740` | Bounded proof recorded in slice spec |

All commit messages end with the requested Claude Opus 5.5 co-author trailer.

## Changed files (SHA256 at final HEAD)

- `docs/specs/duration-text-binding-12-0-7-audit.md` — `cdf68b0670eb8f5dda4a3ef4eccf178a10f0c0892223479b084e1dadda3a57c7`
- `src/c_api/duration_text_binding.rs` — `873b680c58871260cff5cc4d33b18684fc6f4f6738131a4557a572e9ca887029`
- `src/c_api/duration_text_binding/state.rs` — `e01f801364bc4171fe2e039cc7b94d1f45166521a9dc563fb2ac6d0cc1b1fce9`
- `src/loader/tests/wow_api_globals/startup_globals.rs` — `689b6280c96b691c91b922dffdb8e1e1e73ab45b04c2b52d8b4e51b962f03a2e`
- `src/lua_api/globals/lua_duration_object.rs` — `91b7c19142b56b26c04613607b1e487a257e7ffe52a2cc8fc48d649d8a0ca068`
- `src/lua_api/globals/lua_duration_object/core.rs` — `34895f8ccdc2ec6a4ed934035a401af16e759550a22f7571a82e344f20fd2d91`
- `tests/duration_text_binding_copy.rs` — `bb6e6a0479a3056087152d8f23ab5f212940d208272b327fe173b5992e87aafb`
- `tests/patch_12_0_7_duration_text_binding.rs` — `87f1e23af3830921d9b8917ec51a37f765f4d525d13a2d06e89c4946bef83056`

## Proof ledger summary

Default Retail feature set includes retail-12-0-7 via retail-12-1-0; no alternate feature set executed. Host: Linux WSL2 x86_64, rustc path uses pinned 1.98.1. Observed host memory during interruption recovery: 15 GiB total, 11 GiB available, swap 4 GiB total/2.8 GiB used. Every build/test used CARGO_BUILD_JOBS=4, explicit worktree cwd, b102 target; one own build at a time. `BUILD_HOST_SCRIPTS` and `--build-host local` used for all behavioral tests. Runner banner says worktree/target but actual executable and rustc out-dir prove b102. No clean needed (no undefined symbols).

| Filter / gate | RED | Initial GREEN / failed intermediate | Final valid GREEN | Revision containing relevant proof |
|---|---|---|---|---|
| integration patch_12_0_7_duration_text_binding:: | 1 pass / 8 fail, compiled | 8 pass / 1 fail; FontString readout fixture | 9 pass / 0 fail | 56fa3ff6e; runtime unchanged subsequently |
| integration duration_core:: | not run | — | 27 pass / 0 fail | 56fa3ff6e; source unchanged subsequently |
| integration patch_12_0_7_duration_clocks:: | not run | — | 9 pass / 0 fail | 56fa3ff6e; source unchanged subsequently |
| lib startup_globals:: | not run | first build SIGTERM (0 tests); bounded retry 25 pass / 1 numeric-fixture fail | 26 pass / 0 fail | f52a146e4; lib source unchanged subsequently |
| lib duration_text_binding::tests:: | not run | — | 1 pass / 0 fail | f52a146e4; source unchanged subsequently |
| integration duration_text_binding_copy:: | not run | 0 pass / 7 incompatible-fixture fail; adapted 6 pass / 1 missing-formatter fail | 7 pass / 0 fail, including cached CustomAuraButton path | 35b2c3e63 |
| integration duration_text_binding_tick:: | not run | — | 0 selected; no proof credit (Forever-gated) | 35b2c3e63 |
| integration duration_numeric_formatters:: | not run | — | 5 pass / 0 fail | 35b2c3e63 |
| cargo fmt; cargo fmt --check | — | — | PASS | final relevant Rust / 0c361c75f |
| cargo check --target-dir b102 --jobs 4 | — | — | PASS; no warnings | 0c361c75f |
| git diff --check; git status --short | — | — | PASS; clean | 0c361c75f |

84 passing cases across seven nonempty narrow filters. Development proof only: no independent final acceptance, whole-page completion, or native parity claimed. Final source hashes preserve earlier evidence; docs/test-only follow-ups do not invalidate producer controls. Individual argv, statuses and logs appear in step entries above. Logs are `r8-RED-*.log` / `r8-GREEN-*.log` in this audit directory. Secret-only diagnostic runs were 0 pass / 1 fail; one redundant diagnostic without labels is explicitly recorded, not extra credit.

## Adaptations and exclusions

- All E01–E22 OLD anchors matched exactly once after rebase. No master anchor adaptation. Applied state/support before producers; eight original RED failures were behavioral, not compilation failures. Automatic-cadence case already passed old scheduler behavior.
- HasExpired adjudication follows the cached declaration plus explicit existing simulator spec, not a newly inferred native exception. E16 changed to true; zero-span producer unchanged. Evidence and quote above.
- Staged secret test expected FontString.GetText to return a wrapper. Existing widget contract instead returns plain text to secure callers. Test now observes wrapped public SetText input and concrete secure readout; widget code unchanged.
- Additional retained lib lifetime fixture and seven ungated Copy/Assign groups used numeric pseudo-durations or table Format callbacks. Converted fixtures to actual DurationObject/manual clock/NumericRuleFormatter, retaining identity, GC, mutation, component-copy and cached-consumer assertions. Handoff's claim that all copy controls were Forever-gated was inaccurate.
- No incompatible cached-consumer candidate remains observed: real cached CustomAuraButton Copy/Assign/SetDurationText path passes. No cached Lua modified. Historical defaults/coercions/formatting, B31 component/color semantics, native identity, method-override dispatch, older profiles, weak scheduler collection, exhaustive scheduler errors and native FontString secret-return tagging are excluded from coverage claims. Retained existing methods/provider; no second provider or runtime fallback added.
- Known unrelated c_api_surface temporary-shim checks and c_system_api console failure not selected or altered. No unfiltered suites, startup CLI, alternate profiles, native probes, agents/models, pushes, merges, deploys, PRs, coverage JSON or forbidden wiki edits.

## Per-source-row recommendation

IDs read from `data/patch-api/sources/12.0.7-page-coverage.json`, which remains untouched. **partial-development-green** means bounded public-API simulator behavior observed, not independent acceptance. Common unproved scope for every row: historical build 68182/native execution and earlier-profile regression.

| Source ID | Proven scope | Unproved scope (in addition to common gaps) | Recommended status |
|---|---|---|---|
| `prose-undated-022` | Distinct factory userdata; default configuration, reset, Copy/Assign, host scalars and cached aura initialization. | Whole ScriptObject/native identity and historical publication/defaults. | partial-development-green |
| `global api-C_DurationUtil-CreateDurationTextBinding-031` | Factory creates independent handles and nil duration/font configuration; retains sole provider and resource identity. | Undocumented constructor extensions, exact historical defaults/native identity. | partial-development-green |
| `scriptobjects-DurationObject-HasExpired-089` | Inherited zero-span true; before/end-time observations on live manual clock; single return; untainted wrapped modifier/extras; tainted rejection precedes receiver validation. | Native zero-span semantics, exact coercions/errors, exhaustive secret timing/clock combinations. | partial-development-green |
| `scriptobjects-DurationTextBinding-CanFormatText-093` | Unconfigured false; configured zero text true; duration plus formatter eligibility; eligibility used by real live formatting. | Native eligibility/default/expired-text precedence and all configuration combinations. | partial-development-green |
| `scriptobjects-DurationTextBinding-CanUpdateFontString-094` | False without configured FontString; configured eligible bindings update concrete text; formatting eligibility included. | Native handle validation and all incomplete/forbidden combinations. | partial-development-green |
| `scriptobjects-DurationTextBinding-Disable-095` | Disable returns no values; host enabled=false; engine ticks stop changing concrete FontString text. | Native scheduling/GC/error routing and other profiles. | partial-development-green |
| `scriptobjects-DurationTextBinding-Enable-096` | Enable returns no values; host enabled=true; next eligible engine tick resumes text updates. | Native invalidation timing/GC/error routing and other profiles. | partial-development-green |
| `scriptobjects-DurationTextBinding-GetDuration-097` | Single nil default; duration reference identity survives Copy/Assign/GC and environment isolation; live duration changes affect output. | Native handle identity/custom fields/secret-reference provenance policies. | partial-development-green |
| `scriptobjects-DurationTextBinding-GetExpiredText-098` | Single nil default; plain and wrapped text readback; Copy/Assign/reset and per-environment independence. | Native defaults, coercions and exact secret getter tagging. | partial-development-green |
| `scriptobjects-DurationTextBinding-GetFontString-099` | Single nil default; configured FontString identity preserved through Copy/Assign/resource collection. | Native handle identity/forbidden-frame combinations. | partial-development-green |
| `scriptobjects-DurationTextBinding-GetFormattedText-100` | Single concrete remaining text from host clock/rate and host modifier; zero/expired selection; rewind/reconfiguration; VM-secret formatting and wrapped SetText handoff; errors propagate without fallback. | Native strings/rounding/precedence, B31 component/color semantics, method-override dispatch and exhaustive secret combinations. | partial-development-green |
| `scriptobjects-DurationTextBinding-GetTimeModifier-101` | Single RealTime default; authenticated BaseTime setting read live by Rust sampling; copied/assigned scalars independent. | Exact native default/coercion and profiles outside default Retail. | partial-development-green |
| `scriptobjects-DurationTextBinding-GetUpdateInterval-102` | Single interval=1 default; finite interval roundtrip/Copy/Assign/isolation and cadence changes reflected in engine updates. | Native defaults/ranges/precision and exhaustive scheduling. | partial-development-green |
| `scriptobjects-DurationTextBinding-GetZeroDurationText-103` | Single nil default; concrete plain zero text and opaque wrapped zero text; resets clear configuration. | Exact native defaults and secret getter/zero-text selection parity. | partial-development-green |
| `scriptobjects-DurationTextBinding-IsEnabled-104` | Single true default; host-state Disable/Enable/SetEnabled transitions; copied bindings independent. | Native lifecycle/scheduling and other profiles. | partial-development-green |
| `scriptobjects-DurationTextBinding-SetDuration-105` | Actual duration identity accepted; wrapped reference authenticated; missing/nil/table/numeric pseudo-duration rejected atomically; tainted wrapped args/extras rejected before validation. | Native exact proxy identity/coercions/errors and constructor input compatibility. | partial-development-green |
| `scriptobjects-DurationTextBinding-SetExpiredText-106` | String/nil and secure wrapped text accepted; wrapped string stays opaque; invalid numeric input atomic; tainted args/extras rejected first. | Native coercions/error text/secret-output provenance parity. | partial-development-green |
| `scriptobjects-DurationTextBinding-SetFontString-107` | Actual FontString and secure wrapped reference accepted; missing/nil/table rejected atomically; tainted args/extras rejected first. | Unforgeable native handle identity/forbidden-object validation; protocol test is not full native type proof. | partial-development-green |
| `scriptobjects-DurationTextBinding-SetTimeModifier-108` | RealTime/BaseTime secure wrapped/plain values accepted; 1.5/table/nil rejected atomically; extras authenticated first; Rust live state changes concrete rate-scaled output. | Native coercion/range/error policies and profiles outside default Retail. | partial-development-green |
| `scriptobjects-DurationTextBinding-SetUpdateInterval-109` | Finite nonnegative host interval; secure wrapped/plain accepted; negative/nonfinite/string/table/nil rejected atomically; tainted args/extras rejected first; zero cadence updates every tick. | Native range/default/precision and exhaustive scheduler-error policy. | partial-development-green |
| `scriptobjects-DurationTextBinding-SetZeroDurationText-110` | String/nil and secure wrapped text accepted; wrapped nil normalizes to nil; false rejected atomically; tainted args/extras rejected first; zero text remains opaque to addon. | Native coercions/error text/defaults/secret-output parity. | partial-development-green |

## Merge risk

**Moderate.** Default Retail targeted behavior and actual cached aura consumer are green; strict declared input types intentionally stop old numeric/table pseudo-inputs, so external addons using simulator extensions can break. Defaults/eligibility/zero-expired precedence and secret text tagging remain INFERRED. Scalar storage is shared with earlier profiles, which were not executed. Sampling still resolves mutable proxy getters, so hostile method replacement is unproved. Weak scheduler GC/error paths lack Retail acceptance coverage. Master advanced since initial rebase; main integrator must reconcile new master and owns independent acceptance/ledger promotion. No failing selected tests remain after fixture corrections; no merge or deployment performed.
