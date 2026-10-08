# Integrated 6.0.2 command/proof ledger

Runtime `89d359e658859b82bc6681dd74437d2ba61a5934`; master `d0fabed03cd6534b73341fbf569e09c6c388202e`.
Later evidence/docs commits do not invalidate unchanged src/tests/tools trees.
Every command executes from the owned worktree; Cargo target is p602-page.

| Receipt | Command | Exit | Result |
|---|---|---:|---|
| all-sweeps | `cargo test --test prefork_full_ui -- publication_sweep` | 0 | test result: ok. 54 passed; 0 failed; 54 total |
| branch-startup | `timeout 90 /home/osso/.cache/wow-ui-sim-targets/p602-page/debug/wow-sim --no-saved-vars lua-errors` | 0 | [] |
| format | `cargo fmt --check` | 0 | see sealed log |
| integration-blizzard_flight_map_loads | `cargo test --test integration -- blizzard_flight_map_loads` | 0 | test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 10674 filtered out; finished in 0.01s |
| integration-blizzard_poi_button_loads | `cargo test --test integration -- blizzard_poi_button_loads` | 0 | test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 10670 filtered out; finished in 0.02s |
| integration-blizzard_quest_navigation_loads | `cargo test --test integration -- blizzard_quest_navigation_loads` | 0 | test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 10671 filtered out; finished in 0.02s |
| integration-blizzard_shared_map_data_providers_loads | `cargo test --test integration -- blizzard_shared_map_data_providers_loads` | 0 | test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 10659 filtered out; finished in 1.45s |
| integration-c_scenario_info_probes | `cargo test --test integration -- c_scenario_info_probes` | 0 | test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 10666 filtered out; finished in 0.24s |
| integration-objective_tracker | `cargo test --test integration -- objective_tracker` | 0 | test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 10667 filtered out; finished in 4.09s |
| integration-patch_6_0_2 | `cargo test --test integration -- patch_6_0_2` | 0 | test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 10675 filtered out; finished in 0.21s |
| integration-scenario | `cargo test --test integration -- scenario --skip c_scenario_info_probes` | 101 | test result: FAILED. 9 passed; 1 failed; 0 ignored; 0 measured; 10667 filtered out; finished in 0.20s |
| master-all-sweeps | `cargo test --manifest-path /tmp/p602-master-ie7xi7c6/Cargo.toml --test prefork_full_ui -- publication_sweep` | 0 | test result: ok. 53 passed; 0 failed; 53 total |
| master-retail-build | `cargo build --manifest-path /tmp/p602-master-ie7xi7c6/Cargo.toml --bin wow-sim` | 0 | see sealed log |
| master-scenario | `cargo test --manifest-path /tmp/p602-master-scenario-ftxzz2u8/Cargo.toml --test integration -- scenario --skip c_scenario_info_probes` | 101 | test result: FAILED. 9 passed; 1 failed; 0 ignored; 0 measured; 10665 filtered out; finished in 0.42s |
| master-startup | `timeout 90 /home/osso/.cache/wow-ui-sim-targets/p602-page/debug/wow-sim --no-saved-vars lua-errors` | 0 | [] |
| mists-check | `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | 0 | see sealed log |
| negative | `cargo test --test prefork_full_ui -- patch_6_0_2_publication_sweep` | 1 | test result: FAILED. 0 passed; 1 failed; 1 total |
| own-sweep | `cargo test --test prefork_full_ui -- patch_6_0_2_publication_sweep` | 0 | test result: ok. 1 passed; 0 failed; 1 total |
| prefork_full_ui-blizzard_flight_map_loads | `cargo test --test prefork_full_ui -- blizzard_flight_map_loads` | 0 | test result: ok. 9 passed; 0 failed; 9 total |
| prefork_full_ui-blizzard_poi_button_loads | `cargo test --test prefork_full_ui -- blizzard_poi_button_loads` | 0 | test result: ok. 8 passed; 0 failed; 8 total |
| prefork_full_ui-blizzard_quest_navigation_loads | `cargo test --test prefork_full_ui -- blizzard_quest_navigation_loads` | 0 | test result: ok. 7 passed; 0 failed; 7 total |
| prefork_full_ui-blizzard_shared_map_data_providers_loads | `cargo test --test prefork_full_ui -- blizzard_shared_map_data_providers_loads` | 0 | test result: ok. 0 passed; 0 failed; 0 total |
| prefork_full_ui-objective_tracker | `cargo test --test prefork_full_ui -- objective_tracker` | 0 | test result: ok. 13 passed; 0 failed; 13 total |
| prefork_full_ui-patch_6_0_2 | `cargo test --test prefork_full_ui -- patch_6_0_2` | 0 | test result: ok. 3 passed; 0 failed; 3 total |
| prefork_full_ui-scenario | `cargo test --test prefork_full_ui -- scenario` | 0 | test result: ok. 0 passed; 0 failed; 0 total |
| retail-build | `cargo build --bin wow-sim` | 0 | see sealed log |
| test_check_patch_validators | `python3 -B /home/osso/.worktrees/wow-ui-sim-p602-page/tools/test_check_patch_validators.py` | 0 | Ran 4 tests in 0.681s |
| test_extract_patch_non_inventory | `python3 -B /home/osso/.worktrees/wow-ui-sim-p602-page/tools/test_extract_patch_non_inventory.py` | 0 | Ran 36 tests in 0.074s |
| test_gen_patch_wikitext_register | `python3 -B /home/osso/.worktrees/wow-ui-sim-p602-page/tools/test_gen_patch_wikitext_register.py` | 0 | Ran 33 tests in 0.005s |
| test_patch_audit_validation | `python3 -B /home/osso/.worktrees/wow-ui-sim-p602-page/tools/test_patch_audit_validation.py` | 0 | Ran 8 tests in 0.255s |
| test_patch_warlords_register | `python3 -B /home/osso/.worktrees/wow-ui-sim-p602-page/tools/test_patch_warlords_register.py` | 0 | Ran 3 tests in 0.001s |

Integration scenario boundary failure matches pinned master exactly (9/10).
No cached tests match the scenario/shared-map positional filters; cached scenario behavior is covered by patch_6_0_2 and objective_tracker, vignette consumers by flight-map/navigation/POI tests.
53 registers/50 extracts reproduce; three inherited extract failures remain unchanged.
37 prior validators enumerated with git ls-tree at pinned master all pass.
Historical receipts and validator invariants replay unchanged through mapped pins.
