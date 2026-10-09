# Callstack stage-trace independent audit

**Verdict: PASS for bounded diagnostic evidence; original exact library test FAIL (exit 101). Not a fix or broad acceptance.**

Audited 2026-10-09 epoch `20261009T222126Z`, diagnostic commit `be91144d2225ef1ac8b193eb758cef8e3bcdbadf`, build source revision `7130cea91dea9a051207fb84435c1beb9cbb5a08`. Later probe instrumentation is excluded. Two changed Rust files were extracted from diagnostic Git objects into `audit/source/`; both hashes match the retained source-before manifest. No build, test, artifact rerun, service operation, push, or delegation was performed. Only permitted formatter check was executed; receipts were inspected, not regenerated.

## Compilation and artifact provenance

- Retained command: `/home/osso/.worktrees/build-lock.sh /usr/bin/cargo test --offline --locked --lib --no-run --message-format=json`, default features; `CARGO_BUILD_JOBS=4`.
- `compile-service-status.json`: `ActiveState=active`, `SubState=exited`, `Result=success`, `ExecMainCode=1` (CLD_EXITED), `ExecMainStatus=0`. Submission uses `Type=oneshot`, `RemainAfterExit=yes`; submission exit 0 is not substituted for compiler exit. Compiler JSON ends with `build-finished: success=true`; stderr says finished in 59.94s. Retained service timestamps: October 9, 2026 17:21:26–17:22:26 CDT (22:21:26–22:22:26 UTC). No claim about current live service state.
- Exactly one matching library test executable compiler artifact; entire artifact record equals `run-submission.json.artifact`, not merely its basename. Target `wow_ui_sim`, kind/crate type `lib`, edition `2024`, test=true, fresh=false.
- Executable: `/home/osso/Projects/wow/wow-ui-sim/target/debug/deps/wow_ui_sim-bcad23723a7b3b0d`. Current read-only SHA-256 matches retained run receipt: `407a4cbc456dedec51b750eb36d257447ee5a9cb46e47a5f5a69f72c5a898544`.
- Exact profile: opt_level="1", debuginfo="line-tables-only", debug_assertions=true, overflow_checks=true, test=true. Exact feature list: aura-containers, aura-instance-enumeration, aura-xml-widgets, base-spell-relationships, casc, client-retail, default, forbidden-aspects, gui, native-duration-formatting, numeric-rule-formatters, on-update-modes, player-cast-durations, profile-retail, retail-12-0-0, retail-12-0-5, retail-12-0-7, retail-12-1-0, rodio, sound.

`source-before.json` and `source-after-build.json` are exactly equal: 1,656 entries, comprising 1,653 `src/` files plus Cargo.toml, Cargo.lock and build.rs. This binds the recorded source scope across compilation; it does not establish immutability of unrecorded path dependencies, generated assets, inherited environment, or excluded integration fixtures. Submission explicitly excludes two dirty integration-test files and `.code-index.db`. Compilation covers the default library test target, not other profiles or integration suites. Toolchain version is a submission observation, not independently re-executed verification.

## Exact failure and fresh line mapping

Selector receipt lists exactly one test, zero benchmarks. Retained run command is timeout 90 + the exact executable + `lua_api::workarounds::temporary::debug_environment_defaults::tests::installs_debug_environment_defaults --exact --nocapture --test-threads=1`; trace override is `WOW_SIM_TRACE_DEBUG_GETTERS=1`.

`run.stdout`: **0 passed; 1 failed; 0 ignored; 0 measured; 1977 filtered out**, 0.09s. `run-service-status.json`: `Result=exit-code`, `ExecMainStatus=101`. Submission exit 0 only means accepted service submission. Panic at diagnostic source `debug_environment_defaults.rs:244`: `(string):7: attempt to call a userdata value`.

`probe-line-map.txt` was freshly generated from the original raw Lua chunk, retaining its initial newline:

- Lua line 7: `if GetCallstackHeight() ~= 0 then return "callstack_height" end`
- Lua line 8: `if GetErrorCallstackHeight() ~= 0 then return "error_callstack_height" end`

Thus saved line 7 identifies **GetCallstackHeight**, not GetErrorCallstackHeight. The original `let result: String = env ...` probe body, error expectation and `assert_eq!(result, "ok")` are byte-equal to diagnostic commit's parent. Diagnostic commit adds only the pre-probe trace call to this test; assertions were not relaxed.

## Getter observations and inference limits

28 stderr trace records: two getters at 14 phases. Both are Nil at init_entry, before/after globals registration, before/after runtime bootstrap, before/after permanent bootstrap, and before temporary bootstrap. Both become Function immediately after temporary bootstrap: GetCallstackHeight=`GcRef(6627, gen=0)`, GetErrorCallstackHeight=`GcRef(6631, gen=0)`. These identities persist before/after profile bootstrap, before/after final GC, and at the test's final pre-probe checkpoint.

Observed sequence: correctly typed Function at pre-probe, then a non-callable userdata call error at GetCallstackHeight during probe execution. No intermediate value capture exists in this epoch. It does **not** prove a first writer, direct global replacement, a particular VM corruption mechanism, or GC as cause. Earlier probe steps include delegate invocation and Button/EditBox creation; the boundary remains unresolved. GetErrorCallstackHeight's call on line 8 is not reached by this failing probe.

## Actual code, default semantics and readability

`env_init/mod.rs:40–51` gates tracing on environment value exactly `"1"`. Unset, other values, or environment-read error return before global lookup/output. Flag-off path introduces environment checks (and test-side temporary RefCell borrow), not Lua global writes, getter calls, Lua execution, metatable operations, or GC requests. Existing initialization order and getter compatibility defaults (nil-guarded functions returning 0) remain unchanged.

Flag-on path uses `LuaApiMut::get_global_val` then Rust type/Debug formatting to stderr. Pinned compiled rilua `842e4d3...`, `src/api.rs:70–76`, interns the key and directly reads the global table. It invokes no Lua getter/metamethod, performs no global assignment and does not push VM stack values or request GC. **Qualification:** key interning can mutate internal string-arena state/allocation accounting; "nonmutating" is valid for observed Lua globals/metatables, not a promise of zero internal VM effects or zero diagnostic timing perturbation. No protective redesign is required by this audit.

Manual Rust readability audit of all 31 changed Rust lines: helper is short with explicit opt-in guard, two-name loop, effect-revealing trace name and visible stderr emission; added call sites name each boundary. No new suppression, TODO/FIXME/HACK/XXX, complex boolean chain, excessive parameters or nested control flow. Existing long initialization function is not a new diagnostic defect. No in-scope readability violation found.

`rustfmt --check --edition 2024 --config skip_children=true` on exactly the two diagnostic Git-object Rust snapshots: **exit 0**, empty stdout/stderr (`rustfmt.json`). Child module traversal is disabled; no workspace cargo fmt/check was run. Snapshot hashes bind formatting evidence to the earlier diagnostic source, not later edits.

## Streams, warnings and privacy

Full retained streams inspected/parsed: compile.stdout 565,216 bytes / 731 lines; compile.stderr 1,304 / 10; run.stdout 369 / 11; run.stderr 4,153 / 43. Compiler stdout is Cargo JSON metadata/artifacts (0 compiler-message records); test stdout is only test/failure summaries. All 28 trace records and runtime panic are on stderr.

Compilation stderr has **six distinct manifest deprecation warnings** from iced-wgpu-patched/Cargo.toml: large-enum-variant, map-entry, match-wildcard-for-single-variants, redundant-closure-for-method-calls, trivially-copy-pass-by-ref, type-complexity. Its aggregate `generated 6 warnings` line is not a seventh warning. Runtime stderr has **0 warning lines**; its panic and `RUST_BACKTRACE` note are failure evidence, not manifest warnings. No Rust source compiler warning record was emitted.

Privacy candidate scan, each of four streams: secret/password/token/api-key assignments=0; Bearer credentials=0; private-key headers=0; credential-bearing URLs=0; email addresses=0 (patterns and counts in `privacy-candidates.json`). Manual trace review sees only static phase/getter labels, Nil and Function GcRef IDs; no frame fields, account data or environment dump. Logs still expose local paths, dependency/build metadata, PID and internal reference IDs. Heuristic zero candidates is not a universal privacy guarantee; arbitrary future non-function values formatted via Debug are not proven privacy-safe by this nil/function run.

## Proof boundary

[EXIST] PASS — two hash-bound source snapshots, retained receipts, executable hash match.
[SUBSTANTIVE] PASS — opt-in observer implementation, not placeholder code.
[WIRED] PASS — 13 initialization checkpoints plus test pre-probe call; 28 matching output records.
[ANTI-PATTERN] PASS — no new scanned markers/suppressions in diagnostic diff.

**Diagnostic proof only. Original behavior remains failing. No first writer identified, fix claimed, broad test acceptance, or later-instrumentation attribution.**
