# Final command ledger

All commands use explicit worktree cwd and dedicated Cargo target. Expected RED/negative failures are labeled; no-test calibration receives no behavior credit.

| Scope | Command | Result | Revision |
|---|---|---|---|
| p825-accounting | `python3 data/patch-api/evidence/8.2.5-session-2026-10-08/build_accounting.py` | PASS | 5e289fcad |
| p825-all-sweeps | `cargo test --test prefork_full_ui -- publication_sweep` | test result: ok. 38 passed; 0 failed; 38 total | 50d3c7d320581754ecdc22c9530c3b2dc1a12fe5 |
| p825-artifact-validation | `python3 data/patch-api/evidence/8.2.5-session-2026-10-08/validate.py` | validator PASS | 84442531ffb5cf7ee3670c0e6e81455a561c0648 |
| p825-bare-green | `cargo test --test integration patch_8_2_5_unused_members_stay_absent -- --nocapture` | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10815 filtered out; finished in 0.19s | 23e9056861a7217ea100c3567c70b892ca0c2b4b |
| p825-cached-green | `cargo test --test prefork_full_ui -- patch_8_2_5` | test result: ok. 2 passed; 0 failed; 2 total | 23e9056861a7217ea100c3567c70b892ca0c2b4b |
| p825-default-check | `cargo check` | PASS | 5e289fcadab4bedc5283f3fb1b3dc13109e7932c |
| p825-discovery | `cargo test --test prefork_full_ui -- patch_8_2_5` | expected RED/negative, exit 1 | 2b005f5b103f468aff05889fe7ba64b0dbf4803c |
| p825-extractor-cli-red | `python3 tools/test_extract_patch_non_inventory.py ExtractTests.test_cli_self_test_runs_without_argument_conflict` | expected RED/negative, exit 1 | 23e9056861a7217ea100c3567c70b892ca0c2b4b |
| p825-extractor-fixtures | `python3 tools/test_extract_patch_non_inventory.py` | Ran 29 tests | 50d3c7d320581754ecdc22c9530c3b2dc1a12fe5 |
| p825-format | `cargo fmt --check` | PASS | 5e289fcadab4bedc5283f3fb1b3dc13109e7932c |
| p825-generator-fixtures | `python3 tools/test_gen_patch_wikitext_register.py` | Ran 20 tests | 50d3c7d320581754ecdc22c9530c3b2dc1a12fe5 |
| p825-mists-behavior | `cargo test --no-default-features --features sound,gui,casc,client-mists --test integration patch_8_2_5_classic_members_stay_reachable -- --nocapture` | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8163 filtered out; finished in 0.10s | 50d3c7d320581754ecdc22c9530c3b2dc1a12fe5 |
| p825-mists-check | `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | PASS | 50d3c7d320581754ecdc22c9530c3b2dc1a12fe5 |
| p825-negative | `cargo test --test prefork_full_ui -- patch_8_2_5_publication_sweep` | expected RED/negative, exit 1 | 50d3c7d320581754ecdc22c9530c3b2dc1a12fe5 |
| p825-regression-blizzard_commentator_loads | `cargo test --test prefork_full_ui -- blizzard_commentator_loads` | test result: ok. 7 passed; 0 failed; 7 total | 5e289fcadab4bedc5283f3fb1b3dc13109e7932c |
| p825-regression-blizzard_communities_loads | `cargo test --test prefork_full_ui -- blizzard_communities_loads` | test result: ok. 6 passed; 0 failed; 6 total | 5e289fcadab4bedc5283f3fb1b3dc13109e7932c |
| p825-regression-blizzard_recruit_a_friend_loads | `cargo test --test prefork_full_ui -- blizzard_recruit_a_friend_loads` | test result: ok. 10 passed; 0 failed; 10 total | 5e289fcadab4bedc5283f3fb1b3dc13109e7932c |
| p825-regression-c_club_probes | `cargo test --test integration c_club_probes -- --nocapture` | test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 10795 filtered out; finished in 0.37s | 5e289fcadab4bedc5283f3fb1b3dc13109e7932c |
| p825-regression-commentator_api | `cargo test --test integration commentator_api -- --nocapture` | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10815 filtered out; finished in 0.14s | 5e289fcadab4bedc5283f3fb1b3dc13109e7932c |
| p825-regression-pvp_info | `cargo test --test integration pvp_info -- --nocapture` | test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 10808 filtered out; finished in 0.30s | 5e289fcadab4bedc5283f3fb1b3dc13109e7932c |
| p825-regression-pvp_scoreboard | `cargo test --test integration pvp_scoreboard -- --nocapture` | calibration only: zero selected; replaced by PvP info | 5e289fcadab4bedc5283f3fb1b3dc13109e7932c |
| p825-regression-recruit_a_friend_surface | `cargo test --test integration recruit_a_friend_surface -- --nocapture` | test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 10814 filtered out; finished in 0.11s | 5e289fcadab4bedc5283f3fb1b3dc13109e7932c |
| p825-retail-build | `cargo build --bin wow-sim` | PASS | 5e289fcadab4bedc5283f3fb1b3dc13109e7932c |
| p825-retirement-red | `cargo test --test prefork_full_ui -- patch_8_2_5_cached_retirement` | expected RED/negative, exit 1 | 2b005f5b103f468aff05889fe7ba64b0dbf4803c |
| p825-source-reproduction | `python3 data/patch-api/evidence/8.2.5-session-2026-10-08/reproduce_sources.py` | PASS | 5e289fcad |
| p825-startup | `timeout 90 /home/osso/.cache/wow-ui-sim-targets/p825-page/debug/wow-sim --no-addons --no-saved-vars lua-errors` | PASS | 5e289fcadab4bedc5283f3fb1b3dc13109e7932c |

All 37 individual register-generation argv commands/results: `p825-register-reproduction.json`. All extract recipes/results: `p825-saved-extract-reproduction.json`. Consumer/caller grep argv and untruncated results: `p825-removal-consumers.json`, `p825-whole-callers-after.json`. Changed-Rust manual readability: `p825-readability.md` (metric binary unavailable).
