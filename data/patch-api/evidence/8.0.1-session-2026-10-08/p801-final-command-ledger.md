# 8.0.1 integrated command ledger

Worktree: `/home/osso/.worktrees/wow-ui-sim-p801-page`. Target: `/home/osso/.cache/wow-ui-sim-targets/p801-page`. Explicit cwd; no agents, push, merge or vendor edits. Full logs are retained next to receipts. Source scopes in `p801-integration-proof.json` remain valid across later evidence/docs-only commits. Original `p801-proof.json` receipts are historical, not fresh acceptance.

| Receipt | Revision | Command | Exit | Result |
|---|---|---|---|---|
| p801-integration-all-sweeps-red.proof.json | 4759562bc | `cargo test --test prefork_full_ui -- publication_sweep` | 1 | Finished `test` profile [optimized + debuginfo] target(s) in 0.33s; test result: FAILED. 41 passed; 1 failed; 42 total |
| p801-integration-all-sweeps.proof.json | d14286005 | `cargo test --test prefork_full_ui -- publication_sweep` | 0 | Finished `test` profile [optimized + debuginfo] target(s) in 17.90s; test result: ok. 42 passed; 0 failed; 42 total |
| p801-integration-behavior.proof.json | 5523449f0 | `cargo test --test integration -- patch_8_0_1_` | 0 | Finished `test` profile [optimized + debuginfo] target(s) in 23.60s; test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 10663 filtered out; finished in 0.16s |
| p801-integration-cached.proof.json | 5523449f0 | `cargo test --test prefork_full_ui -- patch_8_0_1_cached_` | 0 | Finished `test` profile [optimized + debuginfo] target(s) in 0.15s; test result: ok. 1 passed; 0 failed; 1 total |
| p801-integration-check-queue.proof.json | 5523449f0 | `python3 -B /tmp/p801-check-queue.py` | 0 | exit 0 |
| p801-integration-extend-receipts.proof.json | 5523449f0 | `python3 -B /home/osso/.worktrees/wow-ui-sim-p801-page/tools/extend_patch_audit_receipts.py /home/osso/.worktrees/wow-ui-sim-p801-page /home/osso/.worktrees/wow-ui-sim-p801-page/data/patch-api/evidence/8.0.1-session-2026-10-08 p801 Integrated 8.1.0 register and recorded historical extraction flags 8.0.1` | 0 | summary 8.0.1 {'patch': '8.0.1', 'rows': 269, 'ok': 252, 'gaps': 17, 'result': 'pass'}; summary 8.1.0 {'patch': '8.1.0', 'rows': 145, 'ok': 87, 'gaps': 58, 'result': 'pass'} |
| p801-integration-extract_patch_non_inventory-fixtures.proof.json | 4759562bc | `python3 -B /home/osso/.worktrees/wow-ui-sim-p801-page/tools/test_extract_patch_non_inventory.py` | 0 | Ran 33 tests in 0.066s |
| p801-integration-format.proof.json | 5523449f0 | `cargo fmt --check` | 0 | exit 0 |
| p801-integration-gen_patch_wikitext_register-fixtures.proof.json | 4759562bc | `python3 -B /home/osso/.worktrees/wow-ui-sim-p801-page/tools/test_gen_patch_wikitext_register.py` | 0 | Ran 24 tests in 0.002s |
| p801-integration-map-api.proof.json | 5523449f0 | `cargo test --test integration -- c_map_api` | 0 | Finished `test` profile [optimized + debuginfo] target(s) in 0.13s; test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 10614 filtered out; finished in 0.50s |
| p801-integration-map-probes.proof.json | 5523449f0 | `cargo test --test integration -- c_map_probes` | 0 | Finished `test` profile [optimized + debuginfo] target(s) in 0.13s; test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 10637 filtered out; finished in 0.32s |
| p801-integration-mists-check.proof.json | 5523449f0 | `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | 0 | Finished `dev` profile [optimized + debuginfo] target(s) in 19.93s |
| p801-integration-negative.proof.json | 5523449f0 | `cargo test --test prefork_full_ui -- patch_8_0_1_publication_sweep` | 1 | Finished `test` profile [optimized + debuginfo] target(s) in 0.15s; test result: FAILED. 0 passed; 1 failed; 1 total |
| p801-integration-own-red.proof.json | 4759562bc | `cargo test --test prefork_full_ui -- patch_8_0_1_publication_sweep` | 1 | Finished `test` profile [optimized + debuginfo] target(s) in 1m 04s; test result: FAILED. 0 passed; 1 failed; 1 total |
| p801-integration-patch_audit_validation-fixtures.proof.json | 4759562bc | `python3 -B /home/osso/.worktrees/wow-ui-sim-p801-page/tools/test_patch_audit_validation.py` | 0 | Ran 8 tests in 0.328s |
| p801-integration-source-reproduction.proof.json | 4759562bc | `python3 -B /home/osso/.worktrees/wow-ui-sim-p801-page/data/patch-api/evidence/8.0.1-session-2026-10-08/reproduce_sources.py` | 0 | {"registers_reproduced": 41, "extracts_reproduced": 38, "inherited_extract_failures": ["12.0.5", "12.0.7", "12.1.0"]} |

## Exact integration outcomes

- Own closure: `wt-global-api-C_Map.GetBountySetIDForMap-21`, superseded by 8.1.0 removal `wt-global-api-C_Map.GetBountySetIDForMap-124`; 18 → 17 gaps, 252/269 inventory observations OK. Ledger: 151 partial-development-green, 104 bounded-coverage, 27 audit-pending, 13 metadata-only (295 IDs). Ten prose contracts remain pending.
- All-sweeps RED: only the stale own bounty gap fails; 41 other tests pass. GREEN: 42/42 (41 page sweeps plus factory). Later gap fixtures/ledgers unchanged: no new `LATER_AUDIT_REPLACEMENTS` entries are justified.
- Negative control: original single fabricated API mutation retained. Expected exit 1; 17 → 18, exactly one new gap and no resolved gaps.
- Source reproduction: 41/41 registers, 38/41 extracts; inherited 12.0.5/12.0.7 byte mismatches and 12.1.0 unhandled-template failure unchanged. Recorded flags are retained for 8.0.1 (`--bfa-prepatch`), 8.1.0, 8.1.5 and 8.2.0.
- `extend_patch_audit_receipts.py` refreshed own green results and added the real 8.1.0 sweep/summary (145 rows, 87 OK, 58 gaps); register rows were already reproduced with recorded flags.
- Final matrix lives in `p801-integration-validator-matrix.json`; each row retains command, revision, stdout/stderr and exit. Wiki index/log checks precede every commit; no `__pycache__` is permitted.
