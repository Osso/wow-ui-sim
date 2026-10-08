# 7.1.0 integrated command ledger

Exact master: `aa57dd8f830ec9c53e2f9c9dac5c7a69b5b042ec`. Source/runtime scope: `50c69faa3938e23b3e339d47c0ac2f7f0c7bdc6d`.
All commands run from `/home/osso/.worktrees/wow-ui-sim-p710-page`; CARGO_TARGET_DIR=/home/osso/.cache/wow-ui-sim-targets/p710-page.
The asynchronous queue loaded its driver at the runtime revision above. Receipt revision records checkout HEAD at command completion; exact per-file source hashes independently pin the tested scope. The corrected negative-only replay records its actual environment. Evidence/docs-only changes do not invalidate runtime proof.

| Receipt | Checkout revision | Command | Exit | Result |
|---|---|---|---|---|
| all-sweeps.proof.json | 18f03a97e1 | `cargo test --test prefork_full_ui -- publication_sweep` | 0 |     Finished `test` profile [optimized + debuginfo] target(s) in 2m 37s; test result: ok. 47 passed; 0 failed; 47 total |
| extend-receipts.proof.json | 50c69faa39 | `python3 -B /home/osso/.worktrees/wow-ui-sim-p710-page/tools/extend_patch_audit_receipts.py /home/osso/.worktrees/wow-ui-sim-p710-page /home/osso/.worktrees/wow-ui-sim-p710-page/data/patch-api/evidence/7.1.0-session-2026-10-08/integrated/extension p710 Merged 7.2.0 at aa57dd8f830ec9c53e2f9c9dac5c7a69b5b042ec 7.1.0` | 0 |  |
| extract_patch_non_inventory-fixtures.proof.json | e78bc91721 | `python3 -B /home/osso/.worktrees/wow-ui-sim-p710-page/tools/test_extract_patch_non_inventory.py` | 0 | Ran 34 tests in 0.033s |
| format.proof.json | e78bc91721 | `cargo fmt --check` | 0 |  |
| gen_patch_wikitext_register-fixtures.proof.json | e78bc91721 | `python3 -B /home/osso/.worktrees/wow-ui-sim-p710-page/tools/test_gen_patch_wikitext_register.py` | 0 | Ran 30 tests in 0.002s |
| integration-p710.proof.json | e78bc91721 | `cargo test --test integration -- patch_7_1_0` | 0 |     Finished `test` profile [optimized + debuginfo] target(s) in 54.11s; test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 10670 filtered out; finished in 0.00s; ZERO CASES; no coverage credited |
| intrinsic-discovery.proof.json | 44a4c402e3 | `cargo test --manifest-path /home/osso/.cache/wow-ui-sim-targets/p710-page/intrinsic-probe/Cargo.toml --test prefork_full_ui -- patch_7_1_0_clip_children_state_and_intrinsic_xml` | 1 |     Finished `test` profile [optimized + debuginfo] target(s) in 4m 46s; test result: FAILED. 0 passed; 1 failed; 1 total; ZERO CASES; no coverage credited |
| intrinsic-regression.proof.json | e78bc91721 | `cargo test --test integration -- intrinsic_types::` | 0 |     Finished `test` profile [optimized + debuginfo] target(s) in 0.14s; test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10669 filtered out; finished in 0.12s |
| items-regression.proof.json | e78bc91721 | `cargo test --test integration -- c_item_api::` | 0 |     Finished `test` profile [optimized + debuginfo] target(s) in 0.14s; test result: ok. 100 passed; 0 failed; 0 ignored; 0 measured; 10570 filtered out; finished in 1.00s; ZERO CASES; no coverage credited |
| master-all-sweeps.proof.json | f8a72ed604 | `cargo test --manifest-path /home/osso/.cache/wow-ui-sim-targets/p710-page/master-aa57dd8f8/Cargo.toml --test prefork_full_ui -- publication_sweep` | 0 |     Finished `test` profile [optimized + debuginfo] target(s) in 3m 12s; test result: ok. 46 passed; 0 failed; 46 total |
| mists-check.proof.json | e78bc91721 | `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | 0 |     Finished `dev` profile [optimized + debuginfo] target(s) in 15.53s |
| negative-setup-error.proof.json | e78bc91721 | `cargo test --test prefork_full_ui -- patch_7_1_0_publication_sweep` | 0 | INVALIDATED; no control proof credited: Copied P720 environment names did not select the P710 mutated register. |
| negative.proof.json | 65b4e93761 | `env P710_SWEEP_REGISTER=/home/osso/.worktrees/wow-ui-sim-p710-page/data/patch-api/evidence/7.1.0-session-2026-10-08/p710-negative-register.json P710_SWEEP_OUT=/home/osso/.worktrees/wow-ui-sim-p710-page/data/patch-api/evidence/7.1.0-session-2026-10-08/integrated/negative-results.json cargo test --test prefork_full_ui -- patch_7_1_0_publication_sweep` | 1 |     Finished `test` profile [optimized + debuginfo] target(s) in 0.13s; test result: FAILED. 0 passed; 1 failed; 1 total; ZERO CASES; no coverage credited |
| own-sweep.proof.json | 50c69faa39 | `cargo test --test prefork_full_ui -- patch_7_1_0_publication_sweep` | 0 |     Finished `test` profile [optimized + debuginfo] target(s) in 36.84s; test result: ok. 1 passed; 0 failed; 1 total |
| patch_audit_validation-fixtures.proof.json | e78bc91721 | `python3 -B /home/osso/.worktrees/wow-ui-sim-p710-page/tools/test_patch_audit_validation.py` | 0 | Ran 8 tests in 0.148s |
| prefork-p710.proof.json | e78bc91721 | `cargo test --test prefork_full_ui -- patch_7_1_0` | 0 |     Finished `test` profile [optimized + debuginfo] target(s) in 0.17s; test result: ok. 3 passed; 0 failed; 3 total |
| screen-regression.proof.json | e78bc91721 | `cargo test --test integration -- screen_mode::` | 0 |     Finished `test` profile [optimized + debuginfo] target(s) in 0.13s; test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 10661 filtered out; finished in 0.15s |

Source reproduction: {"registers": 46, "extracts": 43, "inherited_extract_failures": ["12.0.5", "12.0.7", "12.1.0"]}

Negative control and custom intrinsic discovery are expected failures, not passing coverage. Historical evidence is byte-preserved.
The initial negative attempt copied P720 environment names and selected the unmutated P710 register; its exit-zero receipt is explicitly invalidated. Only the corrected P710 replay supplies negative-control proof. No broad checks were repeated for this correction.
