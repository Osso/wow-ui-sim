# Existing runtime evidence audit — 20261010T030623Z

**PASS: one exact ignored probe / one passed / zero failed / exit 0. Global clean startup: NOT established.**

Source epoch `9cf08121af8f0167d5ea4646ba3b4e21e6cabdcf`; 3839 before/after mappings and bytes equal. Default retail features (`client-retail`), not Mists despite build-group name. Selector `startup::tests::trusted_finite_budget_for_selected_addons` uses `--exact --ignored --nocapture --test-threads=1` under timeout 90 and recorded no-sound/verbose/nil-global/timing overrides. Recorded selection and Cargo target/features/profile match; test profile enabled. Runtime artifact `6e18e9bf5c9a357ce103c220f99351d5b0c8f80c70a0d5c64a3462c6d0542654` matches invocation, selection and independent stable read. Stream hashes and counts match result receipt. Build-end content seal remains unproven; historical source revision is submission-recorded, not independently Git-tree matched.

## Owner probe counters

| Owner | After load/GC used | Trusted test limit | After settle used | Recorded owner quota errors, all phases |
|---|---:|---:|---:|---:|
| EnhanceQoL | 12635721 | 100000000 | 0 | 0 |
| AllTheThings | 771449 | 100000000 | 0 | 0 |

12 raw safe probe records reconciled exactly with receipt. After-settle zero reflects resets, not zero startup work. Inspected probe source matches epoch snapshot: True. Probe reaches initialize/load/GC and normal headless-settle helper return; production policy unchanged. Dispatch instructions are not elapsed time, Rust work or native policy. Owner counters do not establish global error freedom or legitimate/excessive-work classification.

## Full private startup stream aggregates

stdout 314852 bytes / 4655 lines; stderr 1495476 bytes / 11706 lines consumed completely. 11378 duration records, 1 file-budget record, 1 handler-budget record. Three initial Lua error records / three unique signatures; no suppression-summary lines. Two quota errors concern EnhanceQoLSharedMedia, outside the selected owners; one nil-call remains in EllesmereUI/EllesmereUI_UICore.lua:1073. Zero stdout failure-warning details and reported zero warnings do not mean zero Lua errors.

Load summary: `{"failed_during_loading": 1, "load_failures": 0, "loaded_addons": "48/78", "loaded_with_lua_errors_during_loading": 1, "lua_files": 2974, "warning_count": 0, "xml_files": 153}`. Safe counters and error fingerprints in aggregate; no raw errors, stacks or game/provider payloads published.

## Baseline 86f272 startup comparison — limited

Compared full private baseline startup streams at `/home/osso/.local/state/wow-ui-sim/verification/toc-startup-root-fixes/20261010T021055Z` only. Baseline is normal non-test binary; probe is ignored unit test at different source epoch with two explicit trusted 100M owner limits. Baseline 7 initial signatures / 96 reported occurrences; probe 3 initial signatures, 1 exact signature shared, 6 absent and 2 new (initial signatures only, suppression restatements excluded). Budget records: file 4→1, handler 91→1. Warnings 4→0; load 48/78 unchanged, Lua 2934→2974 and XML 149→153. Not equivalent workload or causal regression/performance proof.

Four native TOCs were recorded unchanged within baseline (hashes retained in aggregate). Probe records no matching native-TOC/addon input hash boundary. Do not transfer baseline seal to this epoch or assume entire input sealed. Both streams were completely consumed; baseline receipts and aggregate hashed in manifest.

## Scope and ledger

`runtime-aggregate.json` records checks/counts/counters; `runtime-hashmanifest.json` binds streams, receipts, source snapshots, compiler metadata, baseline evidence and reports. Evidence-integrity checks all pass. Source/read/stream proof only; no execution or operational actions. External dependencies, inherited environment, cache/addon inputs, full suites, native parity and work classification excluded. No claim of original prefork proof.
