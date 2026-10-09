# Retained Mists diagnostic audit

**Goal OPEN. Compilation JSON PASS; corrected runtime case FAIL.**

Scope: only epoch `20261009T221327Z` in `/home/osso/Projects/wow/wow-ui-sim`, plus read-only hashing of its exact compiler artifact. Followed verify skill as independent artifact verifier. No tests, builds, reruns, service queries/changes, repository edits, or delegation. Wrote only this audit directory. Initial exploratory plan was not consulted or trusted. Native contract correctness is outside this audit; runtime is not fixed.

## Compilation and identity

- `compile.stdout`: all 738 nonempty lines parsed as JSON: 663 compiler-artifacts, 74 build-script-executed, one `{"reason":"build-finished","success":true}`; zero compiler-message records.
- Exactly one integration test compiler-artifact: target `integration`, source `tests/integration.rs`, edition 2024, test profile, `fresh=false`. Exact feature list: `casc`, `client-mists`, `gui`, `rodio`, `sound`.
- Exact executable: `/home/osso/Projects/wow/wow-ui-sim/target/mists-wrapper-durable/debug/deps/integration-f15e4c1da9a8b8f4`. Current read-only SHA-256: `fa359d740be335938c91be89a6d3fb530360c4f46189ed3ab420bd49e2cce5b5`; 438004976 bytes. Matches both retained run submissions. This verifies current bytes against recorded run hashes, not independent measurement of bytes at historical execution time.
- `submission.json` documents revision `7e8acd2bd9e9a4d2f5c2885a0ec00d75d8f916bd` and Cargo/rustc **1.99.0**, not repository instruction version 1.98.1. No toolchain command was rerun; version is documentary evidence.
- Cargo JSON proves successful build completion. Compile **OS exit remains unknown**: `artifact-run-submission.json` says it was not retained after transient-unit garbage collection. `submit-result.json` exit 0 is submission success, not compile process exit. No compile service-status artifact exists in this epoch.
- `compile.stderr`: six deprecated hyphenated Clippy manifest keys in `iced-wgpu-patched/Cargo.toml`, plus one six-warning summary. `Finished test profile ... in 43.12s`. Not warning-free; no compiler-message warnings in JSON does not erase stderr manifest warnings.

## Selection and retained service outcome

| Attempt | Selected distinct cases | Passed / failed | Filtered | Proof |
|---|---:|---:|---:|---|
| Bare exact selector | 0 | 0 / 0 | 8182 | `run.stdout`; no behavioral credit |
| Module-prefixed exact selector | 1 | 0 / 1 | 8181 | `exact-run.stdout`; runtime FAIL |

`selector-list.json` exit 0 lists exactly `spell_casting::cast_bar_respects_edit_mode_lock_setting_after_startup_fix: test` (1 test, 0 benchmarks). Corrected submission uses that module-prefixed name with `--exact --nocapture --test-threads=1` under timeout 90.

The two `running 1 test` blocks are nested parent/timeout-child execution of **one case**, not two independent cases. Inner summary: 1.61s; outer: 1.63s. `exact-run.stderr:261` records assertion failure at `tests/spell_casting.rs:470:9`: actual `UIParent`, expected `BottomManagedFrameContainer`; `exact-run.stderr:267` records timeout-child exit 101 at `tests/common/timeout_reexec.rs:181:9`. Line references below/JSON use parsed log positions where available; original Rust locations are authoritative.

Exact retained `exact-run-service-status.json` stdout (query exit 0, not test exit):

```text
ActiveState=failed
SubState=failed
Result=exit-code
ExecMainStartTimestamp=Fri 2026-10-09 17:14:59 CDT
ExecMainExitTimestamp=Fri 2026-10-09 17:15:01 CDT
ExecMainCode=1
ExecMainStatus=101
```

`ExecMainCode=1` means exited; `ExecMainStatus=101` is process status, not timeout 124. Corrected submission command exit is 1. Initial retained service is active/exited, Result=success, ExecMainCode=1, ExecMainStatus=0, but selected zero tests.

## Source/cache snapshots

Before and after-build maps are exactly equal: source **3845/3845**, cache **3982/3982**, zero changed/added/removed entries in each scope. Raw snapshot SHA-256 values are in `selected-state.json` and `hashmanifest.sha256`. Equality applies only to recorded before/after-build inventories, not live files or after-run state. External path dependency contents and inherited environment were not captured; untracked `.code-index.db` excluded.

## Errors and probes

`exact-run.stderr` has 14 Lua-error headers: seven distinct messages after stripping duplicate `Lua Error:` reporting prefix; each appears twice. Categories: self-anchor creation rejection; AchievementFrameAchievements OnLoad nil call; AchievementFrameStats missing OnLoad global; AchievementFrameComparison OnLoad nil call; `next` non-table argument; CombatLogColors nil table index; AchievementFrameAchievements OnEvent nil call. These are retained diagnostics, not independently isolated root causes or native-contract findings.

Six `MISTS_CAST_PARENT` probes are retained in execution order (full selected fields in JSON): after-addon-load, after-post-load-workarounds, before/after EDIT_MODE_LAYOUTS_UPDATED, after-startup-events, before-original-assertions. Before that event: systemInfo nil/READ_ERROR, IsInitialized=false, IsInDefaultPosition=false. After: systemInfo exists, default position and initialized true, attachedToPlayerFrame=false. All six: parent.name=UIParent, layoutParent.name=UIParentBottomManagedFrameContainer, layoutParent==bottom=true, parent==bottom=false, parent==bottom.BottomManagedLayoutContainer=false, showingFrames membership nil. Probe initialization changes do not repair asserted parent relationship.

## Privacy and disclosure limits

Read/heuristically scanned all 21 candidate evidence files (2610228 bytes), limited to this epoch. Zero matches for private-key headers, credential assignments, bearer/provider tokens, or email addresses. Three IPv4-shaped matches are SDK version `1.3.268.0`, not network addresses. No broad filesystem/history/environment scan. Binary hash read was not a binary privacy scan. No privacy certification: patterns miss unknown formats. Reports omit VM pointer values, full compiler records/logs, inherited environment and external dependency contents; necessary local paths, case names, hashes, diagnostic fields remain. Do not publish raw evidence indiscriminately.

## Disposition

Evidence consistency PASS within stated scope; diagnostic runtime FAIL (0/1). No operational authority exercised. **Goal remains OPEN**; no native-contract or runtime-fix completion claim.
