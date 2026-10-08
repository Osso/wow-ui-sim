# 7.0.3 proof ledger

Commands are argv, complete logs retained; scope hashes and exact revisions live in receipts. Retail proof is pinned at 69625973b. The sole subsequent source change moves a cfg attribute between the same two modules, leaving default retail declarations unchanged; Mists is checked/tested at 5168fb2d4. validate.py proves this exact equivalence, not a broad drift allowance. No broad suite or successful scope is rerun for an evidence/docs milestone.

| Label | Revision | Exit / selected tests | Command | Status |
|---|---|---|---|---|
| p703-acceptance | 69625973b | 1 | `python3 -B /home/osso/.worktrees/wow-ui-sim-p703-page/data/patch-api/evidence/7.0.3-session-2026-10-08/run_acceptance.py` | Aggregate reports initial Mists failure; accepted child proofs remain valid, repaired profile has separate fresh receipts. |
| p703-all-sweeps | 69625973b | 0; test result: ok. 47 passed; 0 failed; 47 total | `cargo test --test prefork_full_ui -- publication_sweep` | PASS |
| p703-bare-fixed | 69625973b | 0; test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 10670 filtered out; finished in 0.11s | `cargo test --test integration patch_7_0_3 -- --nocapture` | PASS |
| p703-behavior-green | 7554b0eaf | 101 | `cargo test --test integration patch_7_0_3 -- --nocapture` | Compile failure: Vec<i32> has no IntoStack; fixed using existing Lua array helper. |
| p703-collection-integration | 69625973b | 0; test result: ok. 37 passed; 0 failed; 0 ignored; 0 measured; 10636 filtered out; finished in 0.45s | `cargo test --test integration c_collection_api:: -- --nocapture` | PASS |
| p703-crafting-panel-integration | 5168fb2d4 | 0; test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10672 filtered out; finished in 1.85s; test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10672 filtered out; finished in 28.35s | `cargo test --test integration test_showuipanel_professions_crafting:: -- --nocapture` | PASS |
| p703-crafting-panel-prefork | 69625973b | 0; test result: ok. 0 passed; 0 failed; 0 total | `cargo test --test prefork_full_ui -- test_showuipanel_professions_crafting` | Zero selected cases; no coverage credit. Native integration selection supplies real pipeline coverage. |
| p703-discovery | ffe934adf | 1; test result: FAILED. 0 passed; 2 failed; 2 total | `cargo test --test prefork_full_ui -- patch_7_0_3` | RED: 57 publication gaps and cached name-filter no-op; superseded by committed model/retirements/factory correction. |
| p703-extractor-fixtures | ffe934adf | 0 | `python3 -B /home/osso/.worktrees/wow-ui-sim-p703-page/tools/test_extract_patch_non_inventory.py` | PASS |
| p703-filter-red | ffe934adf | 101; test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10670 filtered out; finished in 0.29s | `cargo test --test integration patch_7_0_3_recipe_name_filter_changes_catalog_results -- --nocapture` | RED: concrete bare recipe search assertion fails before implementation. |
| p703-format-check | fcb63eeb1 | 0 | `cargo fmt --check` | PASS |
| p703-format-final | 5168fb2d4 | 0 | `cargo fmt --check` | PASS |
| p703-function-diff-integration | 69625973b | 0; test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 10669 filtered out; finished in 0.10s | `cargo test --test integration c_function_diff_coverage:: -- --nocapture` | PASS |
| p703-generator-fixtures | ffe934adf | 0 | `python3 -B /home/osso/.worktrees/wow-ui-sim-p703-page/tools/test_gen_patch_wikitext_register.py` | PASS |
| p703-master-baseline | ffe934adf | 0 | `python3 -B /home/osso/.worktrees/wow-ui-sim-p703-page/data/patch-api/evidence/7.0.3-session-2026-10-08/run_master_baseline.py` | Control-plane receipt; immutable archived master startup has its own revision receipt. |
| p703-master-startup | aa57dd8f8 | 0 | `timeout 90 /home/osso/.cache/wow-ui-sim-targets/p703-page/master-wow-sim --no-saved-vars lua-errors` | PASS |
| p703-mists-check | 69625973b | 101 | `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | Compile failure: module insertion attached retail quality gate to new filter; repaired at 5168fb2d4. |
| p703-mists-fixed | 5168fb2d4 | 0 | `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | PASS |
| p703-mists-recipe | 5168fb2d4 | 0; test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8175 filtered out; finished in 0.21s | `cargo test --no-default-features --features sound,gui,casc,client-mists --test integration patch_7_0_3_recipe_name_filter_changes_catalog_results -- --nocapture` | PASS |
| p703-negative | 69625973b | 1; test result: FAILED. 0 passed; 1 failed; 1 total | `cargo test --test prefork_full_ui -- patch_7_0_3_publication_sweep` | Expected failure: exact 52 -> 53 gaps, no resolved/stale IDs. |
| p703-own-fixed | 69625973b | 0; test result: ok. 3 passed; 0 failed; 3 total | `cargo test --test prefork_full_ui -- patch_7_0_3` | PASS |
| p703-own-green | 21cf3430e | 101 | `cargo test --test prefork_full_ui -- patch_7_0_3` | Same compile failure; not accepted as proof. |
| p703-profession-integration | 69625973b | 0; test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10672 filtered out; finished in 0.21s; test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 10638 filtered out; finished in 0.42s | `cargo test --test integration professions_api:: -- --nocapture` | PASS |
| p703-profession-prefork | 69625973b | 0; test result: ok. 21 passed; 0 failed; 21 total | `cargo test --test prefork_full_ui -- professions` | PASS |
| p703-retail-build | 69625973b | 0 | `cargo build --bin wow-sim` | PASS |
| p703-startup | 69625973b | 0 | `timeout 90 /home/osso/.cache/wow-ui-sim-targets/p703-page/debug/wow-sim --no-saved-vars lua-errors` | PASS |
| p703-validator-fixtures | ffe934adf | 0 | `python3 -B /home/osso/.worktrees/wow-ui-sim-p703-page/tools/test_patch_audit_validation.py` | PASS |

## Preservation

All 46 registers reproduce; 43 extracts reproduce, with exactly the inherited 12.0.5/12.0.7/12.1.0 errors. Python fixtures 35/30/8 pass. All 25 prior validators passed after adding this page register; later runtime-only changes do not alter their protected historical inputs. All 45 older sweep ID sets and ok-statuses match retained master snapshots.

## Exclusions

No native WoW install/CASC visual proof, historical Legion server parity, 3D renderer, full integration suite, push, merge, model CLI or agent. 7.1.0 first-position placeholder remains for integration; read-only merged-register review finds no intersection.

## Isolated crafting diagnostics

Branch and exact archived master each pass 1/1 pipeline case. Complete Lua Error signature/count sets are identical; retained MerchantFrame:934/CanMerchantRepair diagnostics are pre-existing, not silently treated as a clean fixture. See p703-crafting-diagnostic-comparison.json and p703-master-crafting-snapshot.json.
