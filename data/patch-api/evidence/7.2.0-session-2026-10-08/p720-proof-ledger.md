# 7.2.0 proof ledger

Runtime/accounting snapshot: `8ae333df4`; base/master: `1ade15b52`. Every command has explicit owned cwd and target; complete logs and source scope hashes live in its `.proof.json`. No broad command was repeated for a milestone.

| Label | Revision | Command | Result | Validity |
|---|---|---|---|---|
| p720-all-sweeps | `8ae333df4` | `cargo test --test prefork_full_ui -- publication_sweep` | 0; test result: ok. 45 passed; 0 failed; 45 total | current-scope acceptance |
| p720-discovery | `f208af5f8` | `cargo test --test prefork_full_ui -- patch_7_2_0` | 1; test result: FAILED. 0 passed; 1 failed; 1 total | RED: real MaskTexture factory defect; invalidated by region-factory correction, not acceptance |
| p720-equipment-lib | `8ae333df4` | `cargo test --lib wow_api_equipment_set::` | 0; test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 1972 filtered out; finished in 0.14s | current-scope acceptance |
| p720-equipment-regression | `8ae333df4` | `cargo test --test integration equipment_set` | 0; test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 10662 filtered out; finished in 0.13s | current-scope acceptance |
| p720-extractor-fixtures | `f208af5f8` | `python3 -B tools/test_extract_patch_non_inventory.py` | 0; Ran 34 tests in 0.105s | current-scope acceptance |
| p720-format-check | `8ae333df4` | `cargo fmt --check` | 0 | current-scope acceptance |
| p720-format | `f208af5f8` | `cargo fmt` | 0 | mutating formatter; superseded by format-check |
| p720-generator-fixtures | `f208af5f8` | `python3 -B tools/test_gen_patch_wikitext_register.py` | 0; Ran 28 tests in 0.003s | current-scope acceptance |
| p720-mask-regression | `8ae333df4` | `cargo test --test integration methods_texture::masks_and_misc::` | 0; test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 10658 filtered out; finished in 0.17s | current-scope acceptance |
| p720-master-build | `8ae333df4` | `cargo build --manifest-path /home/osso/.cache/wow-ui-sim-targets/p720-page/master-source-1ade15b52/Cargo.toml --bin wow-sim` | 0 | current-scope acceptance |
| p720-master-queue | `8ae333df4` | `python3 -B /home/osso/.worktrees/wow-ui-sim-p720-page/data/patch-api/evidence/7.2.0-session-2026-10-08/run_master_checks.py` | 0 | orchestration receipt |
| p720-master-startup | `8ae333df4` | `timeout 90 /home/osso/.cache/wow-ui-sim-targets/p720-page/debug/wow-sim --no-saved-vars lua-errors` | 0 | current-scope acceptance |
| p720-mists-check | `8ae333df4` | `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | 0 | current-scope acceptance |
| p720-negative | `807424773` | `env P720_SWEEP_REGISTER=/home/osso/.worktrees/wow-ui-sim-p720-page/data/patch-api/evidence/7.2.0-session-2026-10-08/p720-negative-register.json cargo test --test prefork_full_ui -- patch_7_2_0_publication_sweep` | 1; test result: FAILED. 0 passed; 1 failed; 1 total | expected exit 1: exact additional failed row, 0 -> 1; zero stale/resolved gaps |
| p720-own-green | `807424773` | `cargo test --test prefork_full_ui -- patch_7_2_0` | 0; test result: ok. 4 passed; 0 failed; 4 total | current-scope acceptance |
| p720-prior-validators | `8ae333df4` | `python3 -B /home/osso/.worktrees/wow-ui-sim-p720-page/data/patch-api/evidence/7.2.0-session-2026-10-08/check_prior_validators.py` | 0 | current-scope acceptance |
| p720-retail-build | `8ae333df4` | `cargo build --bin wow-sim` | 0 | current-scope acceptance |
| p720-runner-exit | `8ae333df4` | `python3 -B -c raise SystemExit(7)` | 7 | expected exit 7: receipt runner propagates command failure; no swallowed queue failures |
| p720-source-reproduction | `f208af5f8` | `python3 -B /home/osso/.worktrees/wow-ui-sim-p720-page/data/patch-api/evidence/7.2.0-session-2026-10-08/reproduce_sources.py` | 0 | current-scope acceptance |
| p720-startup | `8ae333df4` | `timeout 90 /home/osso/.cache/wow-ui-sim-targets/p720-page/debug/wow-sim --no-saved-vars lua-errors` | 0 | current-scope acceptance |
| p720-targeted-queue | `8ae333df4` | `python3 -B /home/osso/.worktrees/wow-ui-sim-p720-page/data/patch-api/evidence/7.2.0-session-2026-10-08/run_targeted_checks.py` | 0 | orchestration receipt |
| p720-texture-regression | `8ae333df4` | `cargo test --test integration texture_methods_port::` | 0; test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 10643 filtered out; finished in 0.36s | current-scope acceptance |
| p720-validator-fixtures | `f208af5f8` | `python3 -B tools/test_patch_audit_validation.py` | 0; Ran 8 tests in 0.410s | current-scope acceptance |

Source reproduction uses recorded or inherited flags: 44/44 registers and 41/44 extracts. Three inherited failures (12.0.5, 12.0.7, 12.1.0) unchanged. No source or tool changes after these proofs; documentation/evidence-only commits do not invalidate them.

Parser RED: first fixture run produced two AttributeErrors for missing parse_legacy_widget_summaries / parse_prose_namespace_migrations, 28 tests total; next run passed 28/28. Initial unittest stderr was displayed by Pyrun, not retained by stdout-only .output; final fixture runner retains complete stdout/stderr. Discovery RED has a complete log and scope receipt.

Rust readability: changed helper adds only two region-factory entries; sweep is declarative; cached tests use concrete tuples/state transitions with shallow control flow. No new warning suppressions. Existing vendor iced manifest warnings remain unsuppressed; Mists has zero non-vendor warnings.

Master startup comparison: fixed base snapshot built separately from native git archive, in owned target only. Both addons-enabled executions return exit 0 and []; no other worktree modified.

All 20 earlier validators pass on the same source/tool scope; adding this sealed audit does not expand their fixed historical scopes. Own validation and checkout relocation evidence are recorded separately after the sealing commit.
