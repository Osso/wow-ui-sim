# Saved full-suite audit: 6751 versus retained 8e

**Overall: FAIL. Parent not ready on this evidence.** Saved receipts and complete logs read privately; no commands, tests, builds, polls, delegation or operations. Only requested comparison artifacts written.

## Frozen scope

Source `6751c0f37088de11d0cc864f9313922a5b9cbdab`; comparison `8e6bc113f4c2c47ad693fa7fcb19ddf6bef48476. This is intermediate full-suite evidence, NOT current 518/2e currency diagnostics or autohide-test proof. Receipt SHA is a declared source identity, not independent executable attestation.

## Exact counts

| Revision | Component | Executed | Passed | Failed | Skipped | Exit | Seconds |
|---|---|---:|---:|---:|---:|---:|---:|
| 8e | integration | 10679 | 10677 | 2 | 19 | 100 | 856.2 |
| 8e | prefork | 2325 | 2324 | 1 | 0 | 1 | 90.9 |
| 8e | lib | 1993 | 1993 | 0 | 0 | 0 | 102.3 |
| 6751 | integration | 10679 | 10677 | 2 | 19 | 100 | 968.9 |
| 6751 | prefork | 2325 | 2324 | 1 | 0 | 1 | 108.5 |
| 6751 | lib | 1993 | 1993 | 0 | 0 | 0 | 98.6 |

Both: **14997 executed, 14994 passed, 3 failed, 19 skipped**. Prefork includes main group 2321 (2320 pass / 1 fail) and auxiliary groups 2, 1, 1, all passing. Unique execution records reconcile with terminal summaries and receipt failure selectors/exits; repeated failure-summary entries excluded.

Exact executed selector sets and statuses match across all three components: **0 added, 0 removed, 0 status changes, 0 newly failing selectors versus actual 8e**. `aggregate.json` records every executed selector, status and evidence line. Nextest supplies 19 skipped only as an aggregate, not named skip records; exact skipped identities are unavailable and not inferred.

## Preserved failures

| Exact selector | 8e | 6751 | Failure source |
|---|---|---|---|
| `method_diff_coverage::diff_methods_missing_snapshot_matches_current_metatable_surface` | FAIL, log:6822 | FAIL, log:6829 | `tests/method_diff_coverage.rs:264:5` |
| `method_diff_coverage::diff_methods_extra_snapshot_matches_current_metatable_surface` | FAIL, log:6878 | FAIL, log:6884 | `tests/method_diff_coverage.rs:264:5` |
| `blizzard_garrison_ui_loads::blizzard_garrison_ui_loads_explicitly_via_load_addon_without_errors` | FAIL, log:12032 | FAIL, log:12042 | `tests/blizzard_garrison_ui_loads.rs:286:5` |

Both method snapshot tests and explicit Garrison-load test remain failing. Lib passes; integration exit 100 and prefork exit 1 prevent full-suite readiness. No repair credit or raw assertion/Lua payload publication.

## Declared baseline versus observed comparison

Receipt new_failures is declared relative to the runner master-latest.json baseline, not the retained 8e run. Baseline bytes/SHA at 6751 execution are not archived in the receipt. Garrison fails in both actual saved runs; its declared-new entry does not establish a regression versus 8e. Current runner reference corroborates calculation, not historical installed-runner provenance.

Retained 8e report/aggregate agree with independently recounted actual 8e logs. Actual 8e receipt/log hashes and retained report/aggregate hashes match the retained manifest. Its comparison to 96 and historical source inspection are retained context, not newly re-audited 96 evidence here. No current master-latest contents were substituted for the unavailable contemporaneous baseline.

## Timing and provenance

8e receipt: 2026-10-09T21:54:05-0500 → 2026-10-09T22:11:34-0500 (1049.0 wall seconds).

6751 receipt: 2026-10-09T23:18:25-0500 → 2026-10-09T23:38:01-0500 (1176.0 wall seconds).

Offsets are -05:00. Logged command shapes match for corresponding components (nextest integration/lib, cargo test prefork); thread count 16 on nextest. No historical installed runner, compiler, binary, cache, profile/environment or deployment attestation established by this audit. No claims about current code or tests outside these selected runs. Equal selector status is not proof unchanged fixtures/assertions.

`hashmanifest.json` binds complete saved input bytes and generated outputs using SHA-256; self-hash excluded. Hashing now cannot establish historical immutability or executable linkage. Full streams processed; stream concatenation is not event chronology. Raw logs/failure payloads remain private.
