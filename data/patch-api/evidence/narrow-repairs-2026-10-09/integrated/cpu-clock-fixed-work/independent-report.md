# Independent CPU clock verification

Date: 2026-10-10
Status: SOURCE AUDIT PASS; SEALED TARGETED RUNTIME PASS (1/1, exit 0). Formatting / default cargo check PENDING until the next main-owned job. No full-suite passing claim.

## Scope and identity

Read `<agent-skills>/verify/SKILL.md` first, both CPU-clock handoffs, and the complete rust-readability skill. Read-only Git inspection established repair commit `b8b6be5384ef519e6a2462fab40711905dcc3245` changes only `<repository>/src/iced_app/update_tests.rs` (9 insertions, 8 deletions). Earlier source-audit HEAD and saved submission revision: `d80ee1b7f5fc0aed8e868c593dc84c71ca3ef817`. Runtime-artifact audit observed current HEAD: `3d6c8a6c8bf6660506bdca1f5537f053ca0c256b`.

Git object identity at repair commit, current HEAD, and current working file matches for both inspected files:

- `src/iced_app/update_tests.rs`: `72824eb84322212f2359581fb4b02979b500a891`.
- `src/iced_app/update.rs`: `605b23c2c989af2ce3fc943bbef23188c6a65506`.

`git diff b8b6be538 -- src/iced_app/update_tests.rs src/iced_app/update.rs` produced no output. Identity is scoped to these two files, not the entire current checkout. Read-only Git commands exited 0; no builds, tests, formatter, lint, checks, runtime operations, delegation, or repository edits performed. This report is the only written artifact.

## Native-clock contract and reachability

`<repository>/src/iced_app/update_tests.rs:861-877` retains `#[cfg(unix)]` and `#[test]`. `<repository>/src/iced_app/update.rs:790-792` includes this file as the test module. The test samples the real `main_thread_cpu_time()` before and after exactly `WORK_ITERATIONS = 1_000_000` iterations of the original wrapping multiply/add recurrence. Each result passes through `std::hint::black_box`. Both samples explicitly require `Some`; the assertion is strictly `after > before`.

The production helper at `<repository>/src/iced_app/update.rs:770-783` calls `libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, &mut spec)` and returns `None` on syscall or conversion failure. Repair commit does not change it. The test exercises the calling thread's native CPU clock, not a mocked clock or elapsed wall time.

Removed: 30 ms wall deadline and arbitrary >=10 ms thread-CPU threshold. This deliberately drops the stronger scheduling-share assumption, not the stated native-clock advancement contract. A frozen clock, decreasing samples, or unavailable clock still fails. No retry, spin-until-pass, ignored test, platform-gate broadening, fallback, or weakened availability requirement was added. Fixed iteration count bounds work count; it does not promise bounded elapsed time under OS scheduling.

## Manual Rust readability audit

Changed-line audit: no violations. Named iteration/multiplier constants; one accumulator; one loop level; effect-revealing clock helper; explicit sample failure context; direct positive comparison. The retained recurrence is concrete test workload rather than domain logic requiring clock injection. No new TODO/FIXME/HACK/XXX, suppressions, commented-out code, duplicate helpers, parameter overload, or excess branching.

## Historical failure preserved

Read existing, unmodified artifacts:

- `<saved-suite-results>/b048251f38b2d53e33b3365590cd227d40fc07d0.json`: matching `ref`/`sha`, library exit 100, CPU-clock test recorded in failures and new_failures.
- `<saved-suite-results>/b048251f38b2d53e33b3365590cd227d40fc07d0.log:14852-14869`: exact target failed; `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2000 filtered out; finished in 0.03s`.
- Panic at `src/iced_app/update_tests.rs:872:5`: `30ms of spinning should add CPU time: 100.287µs -> 4.773533ms`.
- Log summary at lines 16510-16512: `2001 tests run: 2000 passed, 1 failed, 0 skipped`, naming the CPU-clock failure.

Read historical source from b048 Git object: original exact 30 ms deadline and >=10 ms CPU assertion are present. Observed samples advance by 4.673246 ms; this failure does not demonstrate a frozen/incorrect native clock. Historical suite failure remains a failure; current source audit cannot rewrite it as passing. No historical artifact was changed or rerun.

## Saved native runtime proof — PASS, bounded scope

Receipt directory: `<private-verification>/map-display-security-red-current/20261010T193717Z`.

Read complete `cpu-clock-result.json`, `cpu-clock.stdout`, `lib-artifact.json`, `submission.json`, `compile-result.json`, and `cargo-result.json`; parsed both complete source manifests. `cpu-clock.stderr` is zero bytes. Saved compilation exited 0, reported `source_equal: true`, and Cargo reported exit 0 with no stream errors. Submitted command was `<system-bin>/cargo test --no-run --test integration --lib --offline --locked --message-format=json --timings -j 12`, at the saved submission revision above. Library feature receipt includes default, gui, sound, casc, and client-retail. This is library-test compilation evidence, not default cargo-check or formatting evidence. Receipts inspected here do not establish a particular rustc version or external dependency provenance; submission explicitly excludes external dependencies/source-to-artifact provenance, untracked files/index, inherited environment, and runtime assets outside fixture inputs.

Saved execution argv: `<system-bin>/timeout 90 <private-verification>/map-display-security-red-current/20261010T193717Z/lib-sealed iced_app::update::update_tests::main_thread_cpu_time_advances_with_busy_work --exact --nocapture --test-threads=1`.

Exact complete stdout:

```text
running 1 test
test iced_app::update::update_tests::main_thread_cpu_time_advances_with_busy_work ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2000 filtered out; finished in 0.00s
```

Execution receipt: `exit: 0`, `expected_tests: 1`, `reached_tests: 1`, `artifact_unchanged: true`. This proves an actual single targeted execution, not a zero-test success. Saved execution was bounded by 90 seconds; workload is independently bounded by one million iterations, not spin-until-pass.

Independently streamed and SHA-256 hashed `lib-sealed`: `643d6f44267f92eb8a29f133905ee6f0500c6375b9b9230a70d85de4bccf5cf2`, matching `lib-artifact.json`. Original executable recorded there: `<repository>/target/debug/deps/wow_ui_sim-ea7724947ed01504`. Proof uses the sealed executable; no current mutable target-directory executable was run.

Both source manifests contain 3853 entries and compare equal. Relevant before/after/current SHA-256 values match:

- `<repository>/src/iced_app/update_tests.rs`: `6eda0b1c4fe45080ac54672ed4af5a6da40834535a50c3c2d5fe8225acf5cd77`.
- `<repository>/src/iced_app/update.rs`: `7d4418a54226a0631fd9a98ca87bc0854ed6484db2802427fa0a863c7dd72bf4`.

Independently computed current Git blobs match the repair identities above. Read-only `git diff b8b6be538 -- src/iced_app/update_tests.rs src/iced_app/update.rs` remains empty. Original b048 helper blob is also `605b23c2c989af2ce3fc943bbef23188c6a65506`. Re-read actual current test/helper and repair diff: real CLOCK_THREAD_CPUTIME_ID, fixed one-million-work recurrence with black_box, explicit availability checks, strict increase; no mock or retry. Manual changed-line Rust readability audit found no violations.

Re-read authentic b048 failure log and JSON: exact historical test still FAILED, 0 passed / 1 failed, and remains listed in lib failures/new_failures. Neither historical receipts nor original failure were modified. Targeted repaired-source PASS does not convert that full suite to PASS.

Current ongoing C_Map security work is outside this specific clock-test/helper proof scope. It does not alone invalidate the saved targeted result or justify a milestone-only rerun. No whole-current-checkout identity or C_Map correctness claim is made.

## Remaining gates and actions

Formatting (`cargo fmt --check`) and default `cargo check`: PENDING until the next main-owned job. No new test, build, check, formatter, operational action, delegation, cwd change, Bash, or repository edit performed. Only read-only artifact/source/Git inspection and this report update. One read-only Pyrun evaluation failed before inspection commands because its imports were not present; the corrected evaluation performed those inspections without executing any test. Targeted CPU runtime proof is complete; broader main-owned gates remain open.
