# Independent source and receipt audit

Date: 2026-10-10
Commit: `12124ded67165a8bb97f0c2d14c7d40f0b078855`
Receipt epoch: `20261010T183127Z/` (relative to this report).

## Verdict

**Authentic diagnostic RED; behavioral fixture PASS.** Saved compilation exited 0; exactly one selected test passed; expected two successful-file records, observed zero. This is a runtime diagnostic-contract failure, not a compiler failure or a failing Rust test. No compile/test/check reruns, operations, or delegation performed by this audit. Only report file written. Receipt wait lasted 33 seconds, below the 180-second bound.

## Source audit

- Commit changes only `src/loader/lua_file.rs`: adds 60 lines entirely inside `#[cfg(test)]`, with `retail-12-0-5` gating and `#[test]` registration at lines 382–384. Current loader file equals the committed file.
- New test (lines 384–439) invokes the real `execute_compiled_lua_file` path twice, with the same tainted owner and private table. Assertions cover returns 19 and 42; final private-table `total == 42`; initial usage 0; first usage increase; exact continuity into the second chunk; unchanged limit 1000; second usage increase and usage below quota. Second chunk calls a nested Lua function. These are observable values/state, not source-shape assertions.
- Production prefix before `#[cfg(test)]` is byte-identical to parent. Existing error test and all following source are unchanged. `execution_budget.rs` and `handler_timing.rs` have identical parent/commit blob IDs. Existing quota/error behavior is unchanged by this commit; no runtime error-path rerun claimed.
- `execute_budgeted_addon_file` (`lua_file.rs:213–247`) emits only on `logging && result.is_err()`, using `format_file_budget_error`; successful results have no budget-record emission. `with_addon_budget` (`execution_budget.rs:10–19`) initializes only absent meters and otherwise retains cumulative owner accounting. `handler_timing.rs:13–15,63–75` enables diagnostics when the environment variable is present; 1000 is a duration threshold, not a disabling value. Current successful-file zero-record result matches that source.

## Readability audit

Followed verify and rust-readability skills in verifier/artifact mode. Manual changed-line audit: no readability violations identified. Approximately 55 body lines, below the 200-line test threshold; shallow flow, descriptive snapshot/chunk names, explicit mutation through `&mut` state, straightforward assertions, no warning suppressions or new TODO/FIXME/HACK/XXX markers. Fixture literals 19, 23, 42, and 1000 have directly observable meanings. Shared setup with the adjacent error test is legitimate test setup, not a required abstraction. No metrics/check command run, per requested read-only receipt scope.

## Saved proof

1. `resources.json`: admitted true, available 28.128379821777344 GiB, load1 12.18505859375, no canonical cargo processes at admission. `submission.json` names the exact commit and native `/usr/bin/cargo test --no-run --lib --offline --locked --message-format=json --timings -j 12` compilation.
2. Full saved `compile.stdout` consumed: 732 JSON records (656 compiler artifacts, 74 build-script records, one build-started, one build-finished). Final record: `{"reason":"build-finished","success":true}`. No compiler-message error records. `compile-result.json`: exit 0, source_equal true, stream_errors empty. `compile.stderr`: finished test profile in 1m 08s; six pre-existing iced_wgpu manifest deprecation warnings. Not warning-free proof.
3. Independently compared all 3853 before/after source hashes: identical. Current files also match every after hash. Independently recomputed sealed executable SHA-256: `53fa041b40c7dc6542560173a644e8d787306661489e138c8555561c17835947`, matching `artifact.json`. Cargo artifact record identifies its original executable. Feature receipt includes `retail-12-0-5`.
4. `execution-results.json` runs the sealed executable with the exact selector, `--exact --nocapture --test-threads=1`, under timeout 90; exit 0, reached_tests 1, artifact_unchanged true, optin `WOW_SIM_LOG_HANDLER_TIMINGS=1000`. Worker source explicitly sets that environment variable on the execution command.
5. Full `fixture.stdout`:

```text
running 1 test
test loader::lua_file::tests::startup_file_budget_success_preserves_meter_continuity_returns_and_effects ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1997 filtered out; finished in 0.11s
```

Full fixture stderr contains startup timing lines only. Independently counted zero lines beginning `[file-budget-success] owner="SuccessfulFileBudgetProbe" `. External worker assertion requires exactly two such lines collectively identifying `@Interface/AddOns/SuccessfulFileBudgetProbe/First.lua` and `@Interface/AddOns/SuccessfulFileBudgetProbe/Second.lua`. Saved failure message:

```text
expected two completed-file budget records; observed 0
```

## Proof boundaries

Rust test does not itself assert log emission: RED belongs to the external receipt assertion. Positive diagnostic-counter content is not validated by that external presence/count assertion. Sequential execution tests the compiled-file loader boundary, not full TOC/disk startup. Source manifests/seals do not establish external dependency provenance, inherited environment isolation, untracked-source coverage, or runtime assets outside fixture inputs (submission explicitly excludes these). No broad suite, quota-error runtime rerun, formatter/check rerun, or native quota-period parity claim.
