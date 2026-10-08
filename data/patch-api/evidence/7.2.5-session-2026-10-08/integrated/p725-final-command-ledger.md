# 7.2.5 integrated command ledger

Runtime/register/sweep scope: `80bb6f64edb56e257bc6e22cdb163defd86197e1`. Exact archived master source: `1ade15b526f897e33c588409b64d8799d6942fa1`. Worktree `/home/osso/.worktrees/wow-ui-sim-p725-page`; owned target `/home/osso/.cache/wow-ui-sim-targets/p725-page`. Actual driver revisions may differ only in evidence/validator/wiki files. Historical receipts remain unchanged.

| Receipt | Driver revision | Source revision | Command | Exit | Result |
|---|---|---|---|---|---|
| master-garrison.proof.json | 80bb6f64e | 1ade15b52 | `cargo test --manifest-path /tmp/p725-master-1ade15b52/Cargo.toml --test prefork_full_ui -- blizzard_garrison_ui_loads_explicitly_via_load_addon_without_errors` | 1 |     Finished `test` profile [optimized + debuginfo] target(s) in 1m 43s; test result: FAILED. 0 passed; 1 failed; 1 total |
| master-build.proof.json | 01a266edc | 1ade15b52 | `cargo build --manifest-path /tmp/p725-master-1ade15b52/Cargo.toml --bin wow-sim` | 0 |     Finished `dev` profile [optimized + debuginfo] target(s) in 0.13s |
| master-startup-addons.proof.json | 01a266edc | 1ade15b52 | `timeout 90 /home/osso/.cache/wow-ui-sim-targets/p725-page/debug/wow-sim --no-saved-vars lua-errors` | 0 | [] |
| all-sweeps.proof.json | 01a266edc | 80bb6f64e | `cargo test --test prefork_full_ui -- publication_sweep` | 0 |     Finished `test` profile [optimized + debuginfo] target(s) in 1m 42s; test result: ok. 45 passed; 0 failed; 45 total |
| integration-p725.proof.json | 01a266edc | 80bb6f64e | `cargo test --test integration -- patch_7_2_5` | 0 |     Finished `test` profile [optimized + debuginfo] target(s) in 44.14s; test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 10668 filtered out; finished in 0.11s |
| prefork-p725.proof.json | ba25334e1 | 80bb6f64e | `cargo test --test prefork_full_ui -- patch_7_2_5` | 0 |     Finished `test` profile [optimized + debuginfo] target(s) in 0.14s; test result: ok. 5 passed; 0 failed; 5 total |
| integration-garrison.proof.json | ba25334e1 | 80bb6f64e | `cargo test --test integration -- garrison` | 0 |     Finished `test` profile [optimized + debuginfo] target(s) in 0.13s; test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 10642 filtered out; finished in 1.31s |
| integration-order_hall.proof.json | ba25334e1 | 80bb6f64e | `cargo test --test integration -- order_hall` | 0 |     Finished `test` profile [optimized + debuginfo] target(s) in 0.13s; test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 10664 filtered out; finished in 0.02s |
| integration-anima.proof.json | ba25334e1 | 80bb6f64e | `cargo test --test integration -- anima` | 0 |     Finished `test` profile [optimized + debuginfo] target(s) in 0.13s; test result: ok. 163 passed; 0 failed; 0 ignored; 0 measured; 10507 filtered out; finished in 17.65s |
| prefork_full_ui-garrison.proof.json | ba25334e1 | 80bb6f64e | `cargo test --test prefork_full_ui -- garrison` | 1 |     Finished `test` profile [optimized + debuginfo] target(s) in 0.14s; test result: FAILED. 37 passed; 1 failed; 38 total |
| prefork_full_ui-order_hall.proof.json | ba25334e1 | 80bb6f64e | `cargo test --test prefork_full_ui -- order_hall` | 0 |     Finished `test` profile [optimized + debuginfo] target(s) in 0.13s; test result: ok. 6 passed; 0 failed; 6 total |
| prefork_full_ui-anima.proof.json | ba25334e1 | 80bb6f64e | `cargo test --test prefork_full_ui -- anima` | 0 |     Finished `test` profile [optimized + debuginfo] target(s) in 0.13s; test result: ok. 9 passed; 0 failed; 9 total |
| gen_patch_wikitext_register-fixtures.proof.json | ba25334e1 | 80bb6f64e | `python3 -B /home/osso/.worktrees/wow-ui-sim-p725-page/tools/test_gen_patch_wikitext_register.py` | 0 | Ran 27 tests in 0.002s |
| extract_patch_non_inventory-fixtures.proof.json | ba25334e1 | 80bb6f64e | `python3 -B /home/osso/.worktrees/wow-ui-sim-p725-page/tools/test_extract_patch_non_inventory.py` | 0 | Ran 34 tests in 0.032s |
| patch_audit_validation-fixtures.proof.json | ba25334e1 | 80bb6f64e | `python3 -B /home/osso/.worktrees/wow-ui-sim-p725-page/tools/test_patch_audit_validation.py` | 0 | Ran 8 tests in 0.140s |
| format.proof.json | ba25334e1 | 80bb6f64e | `cargo fmt --check` | 0 | exit 0 |
| mists-check.proof.json | ba25334e1 | 80bb6f64e | `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | 0 |     Finished `dev` profile [optimized + debuginfo] target(s) in 17.95s |
| retail-build.proof.json | ba25334e1 | 80bb6f64e | `cargo build --bin wow-sim` | 0 |     Finished `dev` profile [optimized + debuginfo] target(s) in 0.13s |
| startup-addons.proof.json | ba25334e1 | 80bb6f64e | `timeout 90 /home/osso/.cache/wow-ui-sim-targets/p725-page/debug/wow-sim --no-saved-vars lua-errors` | 0 | [] |
| extend-receipts.proof.json | ba25334e1 | 80bb6f64e | `python3 -B /home/osso/.worktrees/wow-ui-sim-p725-page/tools/extend_patch_audit_receipts.py /home/osso/.worktrees/wow-ui-sim-p725-page /home/osso/.worktrees/wow-ui-sim-p725-page/data/patch-api/evidence/7.2.5-session-2026-10-08/integrated/extension p725 Merged 7.3.0 at 1ade15b526f897e33c588409b64d8799d6942fa1 7.2.5` | 0 | exit 0 |
| negative.proof.json | ba25334e1 | 80bb6f64e | `cargo test --test prefork_full_ui -- patch_7_2_5_publication_sweep` | 1 |     Finished `test` profile [optimized + debuginfo] target(s) in 0.13s; test result: FAILED. 0 passed; 1 failed; 1 total |
| prefork-diversion.proof.json | ba25334e1 | 80bb6f64e | `cargo test --test prefork_full_ui -- anima_diversion` | 0 |     Finished `test` profile [optimized + debuginfo] target(s) in 0.13s; test result: ok. 0 passed; 0 failed; 0 total; zero cases, NO coverage credited |

## Source and gaps

44/44 registers and 41/44 extracts reproduce with retained flags. Three inherited failures remain exact: 12.0.5 and 12.0.7 output mismatches; 12.1.0 unsupported #description2 template. Saved hashes/flags/outcomes are in source-reproduction-summary.json and per-row reproduction receipts. Shared extender adds the real 7.3.0 register and sweep rows (3 rows, 0 gaps). Its copied template revision is retained separately; the new row is stamped with the actual extension receipt revision.

44 pages plus factory: 45/45 passing cases, 8,869 observations. Own gaps 3 -> 3, no earlier/later gap changes, no new supersession edits or LATER_AUDIT_REPLACEMENTS exceptions. Negative mutation added -> removed changes exactly one ChatBubbles row: 3 -> 4 gaps, expected exit 1, no resolved/stale IDs.

## Garrison, order hall and diversion

Garrison integration 28/28; prefork 37/38. Only cached explicit-load test fails: identical to exact archived master 1ade15b52, ipairs(nil) at Blizzard_AdventuresCombatLog.lua:90. Unchanged assertion/caller/backing implementation checked against both revisions. Order hall integration and prefork each 6/6. Own integration 2/2 and prefork 5/5.

The broader anima integration filter passes 163/163 including all 42 AnimaDiversion cases. Broader prefork anima passes 9/9 animation cases, not diversion cases. Exact prefork anima_diversion filter selects 0 cases; no coverage credited. Existing diversion tests use #[test], not prefork_full_ui_case!. No tests/assertions/vendor behavior were changed to turn this green.

## Checks and startup

Fixtures 27/34/8 pass. Format and requested Mists check pass, zero non-vendor warnings; six inherited iced manifest deprecations remain unsuppressed. Retail build passes. Exact master and branch addons-enabled bounded startup both print []; zero new Lua errors. No __pycache__, vendor edits, agents, push or merge.

## Validator matrix

Final matrix recorded in validator-matrix.json; all evidence/*/validate.py scripts must pass. Original historical scope remains fixed at its recorded revision; separately sealed integrated scope contains 44 registers at the recorded runtime revision.
