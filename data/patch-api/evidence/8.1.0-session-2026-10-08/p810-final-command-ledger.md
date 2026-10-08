# 8.1.0 proof ledger

All commands execute from p810-page with its dedicated target. Full logs and revision/scope receipts accompany each row. Later evidence/docs-only commits do not invalidate Rust proof. Historical RED failures remain labeled, not acceptance failures.

| Receipt | Revision | Command | Exit | Observable result |
|---|---|---|---|---|
| p810-behavior-final.proof.json | 71ad73fbc | `cargo test --test integration patch_8_1_0 -- --nocapture` | 0 | test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 10656 filtered out; finished in 0.10s |
| p810-cached-final.proof.json | 71ad73fbc | `cargo test --test prefork_full_ui -- patch_8_1_0` | 0 | test result: ok. 3 passed; 0 failed; 3 total |
| p810-all-sweeps.proof.json | 7113b19e9 | `cargo test --test prefork_full_ui -- publication_sweep` | 0 | test result: ok. 39 passed; 0 failed; 39 total |
| p810-negative.proof.json | 7113b19e9 | `cargo test --test prefork_full_ui -- patch_8_1_0_publication_sweep` | 1 | test result: FAILED. 0 passed; 1 failed; 1 total |
| p810-regression-c_calendar_defaults_probes.proof.json | 7113b19e9 | `cargo test --test integration c_calendar_defaults_probes` | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10657 filtered out; finished in 0.10s |
| p810-regression-c_map_probes.proof.json | 7113b19e9 | `cargo test --test integration c_map_probes` | 0 | test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 10630 filtered out; finished in 0.31s |
| p810-regression-date_and_time_defaults.proof.json | 7113b19e9 | `cargo test --lib date_and_time_defaults` | 0 | test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 1976 filtered out; finished in 0.10s |
| p810-regression-configuration_warnings_defaults.proof.json | 7113b19e9 | `cargo test --lib configuration_warnings_defaults` | 0 | test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 1974 filtered out; finished in 0.10s |
| p810-regression-blizzard_calendar_loads.proof.json | 7113b19e9 | `cargo test --test prefork_full_ui -- blizzard_calendar_loads` | 0 | test result: ok. 2 passed; 0 failed; 2 total |
| p810-regression-blizzard_world_map_loads.proof.json | 7113b19e9 | `cargo test --test prefork_full_ui -- blizzard_world_map_loads` | 0 | test result: ok. 9 passed; 0 failed; 9 total |
| p810-extract-fixtures.proof.json | 7113b19e9 | `python3 tools/test_extract_patch_non_inventory.py` | 0 | Ran 31 tests in 0.030s |
| p810-gen-fixtures.proof.json | 7113b19e9 | `python3 tools/test_gen_patch_wikitext_register.py` | 0 | Ran 22 tests in 0.001s |
| p810-source-reproduction-final.proof.json | 7113b19e9 | `python3 /home/osso/.worktrees/wow-ui-sim-p810-page/data/patch-api/evidence/8.1.0-session-2026-10-08/reproduce_sources.py` | 0 | {"registers_reproduced": 38, "input_preservation": "pinned base/audit git trees"} |
| p810-format.proof.json | 7113b19e9 | `cargo fmt --check` | 0 | exit only |
| p810-mists-check.proof.json | 7113b19e9 | `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | 0 | Finished `dev` profile [optimized + debuginfo] target(s) in 52.98s |
| p810-mists-behavior.proof.json | c5e0f15ab | `cargo test --no-default-features --features sound,gui,casc,client-mists --test integration patch_8_1_0` | 0 | test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 8164 filtered out; finished in 0.11s |
| p810-validator-portability.proof.json | fbc051693 | `python3 data/patch-api/evidence/8.1.0-session-2026-10-08/test_validator_portability.py` | 0 | one relocated/shared-drift acceptance test passes |

## Integrated post-rebase proof

Historical rows above remain historical. Current Rust scopes are pinned in `p810-proof.json`; all commands use the dedicated p810 target. The passing all-sweeps proof is not rerun for the unrelated source-name-test correction.

| Receipt | Revision | Command | Exit | Observable result |
|---|---|---|---|---|
| p810-integration-all-sweeps.proof.json | c3a3a065b | `cargo test --test prefork_full_ui -- publication_sweep` | 0 | test result: ok. 41 passed; 0 failed; 41 total |
| p810-integration-cached-final.proof.json | 9b9c78b6a | `cargo test --test prefork_full_ui -- patch_8_1_0` | 0 | test result: ok. 3 passed; 0 failed; 3 total |
| p810-integration-calendar.proof.json | 9b9c78b6a | `cargo test --test prefork_full_ui -- blizzard_calendar_loads` | 0 | test result: ok. 2 passed; 0 failed; 2 total |
| p810-integration-bare-final.proof.json | 7648b4718 | `cargo test --test integration -- patch_8_1_0 c_calendar c_map_probes date_and_time` | 0 | test result: ok. 37 passed; 0 failed; 0 ignored; 0 measured; 10626 filtered out; finished in 0.53s |
| p810-integration-lib.proof.json | 7648b4718 | `cargo test --lib -- date_and_time_defaults configuration_warnings_defaults` | 0 | test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 1971 filtered out; finished in 0.35s |
| p810-integration-negative.proof.json | 7648b4718 | `cargo test --test prefork_full_ui -- patch_8_1_0_publication_sweep` | 1 | test result: FAILED. 0 passed; 1 failed; 1 total; 58 → 59, only CompareCalendarTime mutation |
| p810-integration-extend-receipts.proof.json | aba601dc2 | `python3 -B /home/osso/.worktrees/wow-ui-sim-p810-page/tools/extend_patch_audit_receipts.py /home/osso/.worktrees/wow-ui-sim-p810-page /home/osso/.worktrees/wow-ui-sim-p810-page/data/patch-api/evidence/8.1.0-session-2026-10-08 p810 Integrated 8.1.5 and 8.2.0 recorded provenance 8.1.0` | 0 | 8.1.5 / 8.2.0 result rows added; own result refreshed |
| p810-integration-extract_patch_non_inventory-fixtures.proof.json | 7648b4718 | `python3 -B /home/osso/.worktrees/wow-ui-sim-p810-page/tools/test_extract_patch_non_inventory.py` | 0 | Ran 32 tests in 0.034s |
| p810-integration-gen_patch_wikitext_register-fixtures.proof.json | 7648b4718 | `python3 -B /home/osso/.worktrees/wow-ui-sim-p810-page/tools/test_gen_patch_wikitext_register.py` | 0 | Ran 23 tests in 0.001s |
| p810-integration-patch_audit_validation-fixtures.proof.json | 7648b4718 | `python3 -B /home/osso/.worktrees/wow-ui-sim-p810-page/tools/test_patch_audit_validation.py` | 0 | Ran 8 tests in 0.154s |
| p810-integration-format.proof.json | 7648b4718 | `cargo fmt --check` | 0 | exit 0; full log retained |
| p810-integration-mists-check.proof.json | 7648b4718 | `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | 0 |     Finished `dev` profile [optimized + debuginfo] target(s) in 21.65s; zero non-vendor warnings |

- Source reproduction: `reproduce_sources.reproduce(load_extractor())` checked all 40 registers and saved extracts with recorded/inherited historical flags; 37 saved extracts reproduce, three inherited 12.x failures unchanged. Per-command outputs/flags are retained in the reproduction JSON rows.
- Portability: `python3 -B data/patch-api/evidence/8.1.0-session-2026-10-08/test_validator_portability.py` passes relocation/new-page scope and rejects protected-input whitespace drift and missing historical receipt rows.
- Original combined cached command exits 1 because the harness accepts one positional filter; both requested filters subsequently pass separately.
- First combined integration command exits 101 on obsolete source-name assertion; replacement composition test passes in the identical 37-case selection.
