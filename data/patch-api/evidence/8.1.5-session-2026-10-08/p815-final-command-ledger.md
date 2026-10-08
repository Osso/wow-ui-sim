# Verification command ledger

All commands: cwd `/home/osso/.worktrees/wow-ui-sim-p815-page`, CARGO_TARGET_DIR `/home/osso/.cache/wow-ui-sim-targets/p815-page`. Full output and revision/environment/log hashes live in per-command receipts. No full integration suite, no agents.

| Scope | Exact command | Result | Revision |
|---|---|---|---|
| p815-all-sweeps | `cargo test --test prefork_full_ui -- publication_sweep` | exit 0; ok. 39 passed; 0 failed; 39 total | `9d3639c1b177944b2654f8b354683b76256396a8` |
| p815-bare-final | `cargo test --test integration -- patch_8_1_5` | exit 0; ok. 2 passed; 0 failed; 0 ignored; 0 measured; 10681 filtered out; finished in 0.12s | `9d3639c1b177944b2654f8b354683b76256396a8` |
| p815-retirement-cached-final | `cargo test --test prefork_full_ui -- patch_8_1_5_cached` | exit 0; ok. 1 passed; 0 failed; 1 total | `9d3639c1b177944b2654f8b354683b76256396a8` |
| p815-mists-check | `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | exit 0 | `9d3639c1b177944b2654f8b354683b76256396a8` |
| p815-mists-behavior | `cargo test --no-default-features --features sound,gui,casc,client-mists --test integration -- patch_8_1_5` | exit 0; ok. 2 passed; 0 failed; 0 ignored; 0 measured; 8164 filtered out; finished in 0.10s | `9d3639c1b177944b2654f8b354683b76256396a8` |
| p815-default-check | `cargo check` | exit 0 | `9d3639c1b177944b2654f8b354683b76256396a8` |
| p815-format | `cargo fmt --check` | exit 0 | `9d3639c1b177944b2654f8b354683b76256396a8` |
| p815-extract_patch_non_inventory-fixtures | `python3 tools/test_extract_patch_non_inventory.py` | exit 0; 30 fixtures pass | `6e3a1d16c8340ae7a9aa4099488b0cbeea24d516` |
| p815-gen_patch_wikitext_register-fixtures | `python3 tools/test_gen_patch_wikitext_register.py` | exit 0; 21 fixtures pass | `6e3a1d16c8340ae7a9aa4099488b0cbeea24d516` |
| p815-source-reproduction | `python3 /home/osso/.worktrees/wow-ui-sim-p815-page/data/patch-api/evidence/8.1.5-session-2026-10-08/reproduce_sources.py` | exit 0 | `9d3639c1b177944b2654f8b354683b76256396a8` |
| p815-extract-cli-check | `python3 tools/extract_patch_non_inventory.py --patch 8.1.5 --text-only --legacy-api-bullets --check` | exit 0 | `9d3639c1b177944b2654f8b354683b76256396a8` |
| p815-negative | `cargo test --test prefork_full_ui -- patch_8_1_5_publication_sweep` | exit 1; FAILED. 0 passed; 1 failed; 1 total; expected one-ID failure, 24 → 25 gaps | `9d3639c1b177944b2654f8b354683b76256396a8` |
| p815-retail-build | `cargo build --bin wow-sim` | exit 0 | `9d3639c1b177944b2654f8b354683b76256396a8` |
| p815-startup | `/usr/bin/timeout 90 /home/osso/.cache/wow-ui-sim-targets/p815-page/debug/wow-sim --no-addons --no-saved-vars lua-errors` | exit 0; stdout [] / zero Lua errors | `9d3639c1b177944b2654f8b354683b76256396a8` |
| p815-regression-toy | `cargo test --test integration -- toy` | exit 0; ok. 29 passed; 0 failed; 0 ignored; 0 measured; 10654 filtered out; finished in 1.88s | `9d3639c1b177944b2654f8b354683b76256396a8` |
| p815-regression-c_area_poi_probes | `cargo test --test integration -- c_area_poi_probes` | exit 0; ok. 8 passed; 0 failed; 0 ignored; 0 measured; 10675 filtered out; finished in 0.12s | `9d3639c1b177944b2654f8b354683b76256396a8` |
| p815-regression-calendar | `cargo test --test integration -- calendar` | exit 0; ok. 10 passed; 0 failed; 0 ignored; 0 measured; 10673 filtered out; finished in 0.20s | `9d3639c1b177944b2654f8b354683b76256396a8` |
| p815-regression-c_club_probes | `cargo test --test integration -- c_club_probes` | exit 0; ok. 21 passed; 0 failed; 0 ignored; 0 measured; 10662 filtered out; finished in 0.25s | `9d3639c1b177944b2654f8b354683b76256396a8` |
| p815-regression-pvp_info | `cargo test --test integration -- pvp_info` | exit 0; ok. 8 passed; 0 failed; 0 ignored; 0 measured; 10675 filtered out; finished in 0.13s | `9d3639c1b177944b2654f8b354683b76256396a8` |
| p815-regression-report_system | `cargo test --test integration -- report_system` | exit 0; ok. 2 passed; 0 failed; 0 ignored; 0 measured; 10681 filtered out; finished in 0.10s | `9d3639c1b177944b2654f8b354683b76256396a8` |
| p815-regression-c_social_probes | `cargo test --test integration -- c_social_probes` | exit 0; ok. 8 passed; 0 failed; 0 ignored; 0 measured; 10675 filtered out; finished in 0.14s | `9d3639c1b177944b2654f8b354683b76256396a8` |
| p815-regression-date_and_time_deterministic_defaults | `cargo test --test integration -- date_and_time_deterministic_defaults` | exit 0; ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10682 filtered out; finished in 0.00s | `9d3639c1b177944b2654f8b354683b76256396a8` |
| p815-regression-communities | `cargo test --test prefork_full_ui -- blizzard_communities_loads` | exit 0; ok. 6 passed; 0 failed; 6 total | `9d3639c1b177944b2654f8b354683b76256396a8` |

## Historical development proof

Discovery and behavioral RED receipts are not acceptance. Trial SetCommunityID retirement was rejected because 10.2.7 re-adds it; source/test fixtures and ledger are restored. Subsequent current-scope commands above pass.

- `p815-bare-green`: `cargo test --test integration -- patch_8_1_5` — exit 0, revision `af1efeef57442ad211a37f6986dfa2dbf1cf62d1`; historical only.
- `p815-cached-discovery`: `cargo test --test prefork_full_ui -- patch_8_1_5` — exit 1, revision `af1efeef57442ad211a37f6986dfa2dbf1cf62d1`; historical only.
- `p815-discovery`: `cargo test --test prefork_full_ui -- patch_8_1_5` — exit 1, revision `30b5e6599f72b66044826df12b1d22eee0387910`; historical only.
- `p815-fanfare-red`: `cargo test --test integration -- patch_8_1_5_toy_fanfare` — exit 101, revision `30b5e6599f72b66044826df12b1d22eee0387910`; historical only.
- `p815-retirement-cached`: `cargo test --test prefork_full_ui -- patch_8_1_5_cached` — exit 0, revision `6e3a1d16c8340ae7a9aa4099488b0cbeea24d516`; historical only.
- `p815-retirement-red`: `cargo test --test integration -- patch_8_1_5_unused` — exit 101, revision `30b5e6599f72b66044826df12b1d22eee0387910`; historical only.
- `p815-supersession-red`: `cargo test --test prefork_full_ui -- publication_sweep` — exit 1, revision `6e3a1d16c8340ae7a9aa4099488b0cbeea24d516`; historical only.

## Source recipes

All 38 exact register regeneration commands are retained in `p815-register-reproduction.json`; flags are recorded or explicitly inherited. All formerly reproducible extracts reproduce; inherited 12.0.5/12.0.7/12.1.0 failures stay unchanged. `reproduce_sources.py` records 194 prior file hashes and 74 unchanged default/example mode outcomes.

Final artifact gate: `python3 data/patch-api/evidence/8.1.5-session-2026-10-08/validate.py`; receipt recorded separately in `p815-validation-receipt.json` to avoid self-referential proof-index hashing.
