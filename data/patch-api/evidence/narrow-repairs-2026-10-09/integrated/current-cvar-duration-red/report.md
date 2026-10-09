# Independent bounded current CVar proof — PASS; duration boundary — reproduced FAIL

## Identity and provenance

Canonical cwd: `/home/osso/Projects/wow/wow-ui-sim`. Requested and initial observed HEAD: `1b505c7c12e4a359cb8ab01eb8bee24adf9ac031`. Final observed HEAD: `f2e1a7b4f8ac72c97e9d4630f238f1d27c40474c`. Initial status: untracked `.code-index.db`. Final status additionally contains untracked `data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/final-literal-pages/`. Concurrent HEAD advance recorded; `artifacts/concurrent-head-diff.txt` retains its docs/evidence-only changes. No runtime-source hash drift detected; no ownership or cleanup claim for concurrent changes. No tracked writes; no Cargo/build, delegation, network, operations, native probes, runtime fixes or parent-goal credit. Read/followed verify skill as assigned verifier; read both /tmp contract decisions. All CLI calls used explicit canonical cwd and argv-style Pyrun.

Executed **existing compile epoch `41f1abbeb83d508b323f51d1f371a3bc506b63b5`**, NOT a binary compiled at current HEAD. Binary: `/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/integration-1db16289b0998b2a`. On-disk SHA256 before listing, before executions and after: `3395a9d9307b65f28b7cb51f423e7d4a315c3793ecbd3e0ac214497780f8356b`; matches retained `binary-artifact.json` provenance. Original report/compiler metadata in `data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/retail-toc/` records one full integration compile, exit 0, fresh=false artifact (newly compiled), default Retail with gui and numeric-rule-formatters. Parsed full compiler JSON: 738 records, exact artifact record present and final build-finished success=true. Six receipt files including original compile streams and before/after scope snapshots matched recorded SHA256 seals. These are retained local provenance, not independent reconstruction or cryptographic attestation of toolchain/dependency origins.

## Relevant-source equivalence and exclusions

Compared 3885 retained tracked source/test/crate/iced/config/manifest/build hashes against on-disk current files. 3882 match. Exact per-file hashes: `artifacts/source-equivalence.json`. Full epoch→current tracked name diff: `artifacts/epoch-current-name-status.txt`. Relevant CVar test, globals registration, storage, YAML, environment setup, integration harness and build script unchanged. Duration test, binding, duration-object/core sources also unchanged. No later in-proof source hash drift: [].

Not whole-tree/build-input identity: Cargo.toml adds a separate client-era-required patch test; frame_collect.rs wraps assertions inside its cfg(test) unit-test module with cfg(gui); loader transmog_situation unit-test module cfg narrows to pre-12.0.5. Full exclusions diff retained in `artifacts/excluded-code-diff.txt`. The integration executable does not run these library unit tests, and build.rs generates its harness from tests/, not the new patch-tests/ target. These deltas do not change the selected Retail cases or their runtime production paths. Docs and audit/test-perf evidence changes excluded. External path-dependency source freshness, toolchain, generated artifacts other than compiler metadata, native clients and all other profiles unproved; no universal/current-SHA execution claim.

## CVar execution — 1/1 PASS

Exact filter: `set_cvar_global::register_cvar_numeric_default_preserves_existing_override_and_first_default`.

Listing once with `[binary, filter, '--exact', '--list']` selected exactly **1 test, 0 benchmarks**, exit 0. Executed once with `[binary, filter, '--exact', '--nocapture']`, timeout 90 seconds, exit 0, elapsed 0.214722s, 2026-10-09T19:46:02.645120+00:00 → 2026-10-09T19:46:02.859860+00:00. Full argv/cwd/times/exit/binary hashes and inherited environment key names retained in `artifacts/list.json` and `artifacts/test.json`; full untruncated stdout/stderr adjacent. No explicit environment overrides. Full streams read: stdout 1 passed, 0 failed, 0 ignored, 0 measured, 10697 filtered out; stderr contains startup timing only. No CVar rerun.

### Manual semantic audit and spec limits

`tests/set_cvar_global.rs:179–196` loops both global and C_CVar registration functions, uses separate names, stores override string '1.25', registers numeric 0, asserts current '1.25'/default '0', re-registers numeric 6, asserts those first values remain. Four assertions per surface (eight total) inside one Rust test. This case reads global getters only; it does not independently exercise namespace getters.

`src/lua_api/globals/set_cvar_verb.rs:187–200,343–367`: finite numbers stringify, both registration surfaces share register_cvar and SimState.cvars. `src/cvars.rs:83–110,119–128,166–185`: override reads precede defaults, registration inserts only first default and never replaces override. Observed PASS establishes this bounded simulator lifecycle on source-equivalent current paths. Spec `docs/specs/cvar-registration.md` labels numeric acceptance Datamine-inferred, not native-verified; nil policy and re-registration are simulator policy. No historical 1.1 behavior, persistence reload, namespace-getter parity, unsupported inputs, other numbers, full addon startup or native semantics established. Test storage uses per-process temp paths under deps; SetCVar may write temporary CVar files, not production persistence proof.

## Additional duration consumer execution — 0/1 FAIL (expected reproduction)

Exact filter: `numeric_rule_formatter::numeric_rule_formatter_updates_duration_binding_text`.

Listed once: exactly 1 test, 0 benchmarks, exit 0. Executed once directly with --exact --nocapture, 90-second bound, exit 101, 0.215047s, 2026-10-09T19:46:24.479988+00:00 → 2026-10-09T19:46:24.695037+00:00. Full failure streams and argv/cwd/times/exit/hash receipt: `artifacts/duration-test.*`; listing: `artifacts/duration-list.*`.

Observed stdout: 0 passed, 1 failed, 10697 filtered out. Stderr panic at `tests/numeric_rule_formatter.rs:181:6`: `expected LuaDurationObject at argument 1`. Manual inspection: fixture's first `binding:SetDuration(1.2)` is a scalar; binding SetDuration validates its value before storing. This reproduces the scalar-duration boundary, not a numeric formatter output failure. The later label/copy/custom-callback assertions never prove their outputs in this failing run. Binding custom function/table callback code receives self.duration (original input), not sampled scalar text; this is source audit only, no callback runtime proof. The supplied manual-clock object fixture correction was not applied or executed. No native callback parity or repaired test PASS claim.

## Artifacts

`artifacts/manifest.json` seals all retained files. No build performed; no reruns; no tracked edits. CVar claim bounded PASS; duration claim bounded reproduced FAIL, repair remains unverified and outside this task.
