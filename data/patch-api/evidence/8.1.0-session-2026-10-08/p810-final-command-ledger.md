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
