# Proof ledger

Runtime scope: `43dd7ed8e15ae6a00a04818e28ad1754f4540dd6`. Later documentation/evidence-only commits do not invalidate runtime, corpus or fixture proof. Historical receipts are not new executions after 7.3.0 integration.

| Scope | Command | Revision | Exit | Evidence / invalidation |
|---|---|---|---|---|
| p725-all-sweeps | `cargo test --test prefork_full_ui -- publication_sweep` | `43dd7ed8e` | 0 | `p725-all-sweeps.txt` / not invalidated |
| p725-anima-callers | `cargo test --test integration blizzard_ui_blizzard_animadiversionui` | `43dd7ed8e` | 0 | `p725-anima-callers.txt` / not invalidated |
| p725-bare-trees | `cargo test --test integration patch_7_2_5` | `43dd7ed8e` | 0 | `p725-bare-trees.txt` / not invalidated |
| p725-base-garrison-case | `cargo test --manifest-path /tmp/p725-base-85c2acb2d/Cargo.toml --test prefork_full_ui -- blizzard_garrison_ui_loads_explicitly_via_load_addon_without_errors` | `43dd7ed8e` | 1 | `p725-base-garrison-case.txt` / base 85c2acb2d archive source; same inherited failure |
| p725-cached-anima | `cargo test --test prefork_full_ui -- animadiversion` | `43dd7ed8e` | 0 | `p725-cached-anima.txt` / zero cases; no acceptance credit, covered by integration filter |
| p725-cached-garrison | `cargo test --test prefork_full_ui -- garrison` | `43dd7ed8e` | 1 | `p725-cached-garrison.txt` / expected retained failure; same base boundary, not all-green |
| p725-discovery | `cargo test --test prefork_full_ui -- patch_7_2_5` | `8a5dd0c7b` | 1 | `p725-discovery.txt` / expected failure proving exact gap boundary |
| p725-extractor-fixtures | `python3 -B tools/test_extract_patch_non_inventory.py` | `43dd7ed8e` | 0 | `p725-extractor-fixtures.txt` / not invalidated |
| p725-format | `cargo fmt --check` | `43dd7ed8e` | 0 | `p725-format.txt` / not invalidated |
| p725-garrison-callers | `cargo test --test integration garrison` | `43dd7ed8e` | 0 | `p725-garrison-callers.txt` / not invalidated |
| p725-garrison-red | `cargo test --test prefork_full_ui -- patch_7_2_5_garrison` | `8a5dd0c7b` | 1 | `p725-garrison-red.txt` / pre-model runtime + separately hashed uncommitted harness |
| p725-generator-fixtures | `python3 -B tools/test_gen_patch_wikitext_register.py` | `43dd7ed8e` | 0 | `p725-generator-fixtures.txt` / not invalidated |
| p725-mists-check | `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | `43dd7ed8e` | 0 | `p725-mists-check.txt` / not invalidated |
| p725-negative | `cargo test --test prefork_full_ui -- patch_7_2_5_publication_sweep` | `43dd7ed8e` | 1 | `p725-negative.txt` / expected failure proving exact gap boundary |
| p725-other-validators | `python3 -B /home/osso/.worktrees/wow-ui-sim-p725-page/data/patch-api/evidence/7.2.5-session-2026-10-08/check_other_validators.py` | `43dd7ed8e` | 0 | `p725-other-validators.txt` / not invalidated |
| p725-own-green | `cargo test --test prefork_full_ui -- patch_7_2_5` | `43dd7ed8e` | 0 | `p725-own-green.txt` / not invalidated |
| p725-retail-build | `cargo build --bin wow-sim` | `43dd7ed8e` | 0 | `p725-retail-build.txt` / not invalidated |
| p725-source-reproduction | `python3 -B /home/osso/.worktrees/wow-ui-sim-p725-page/data/patch-api/evidence/7.2.5-session-2026-10-08/reproduce_sources.py` | `43dd7ed8e` | 0 | `p725-source-reproduction.txt` / not invalidated |
| p725-startup | `timeout 90 /home/osso/.cache/wow-ui-sim-targets/p725-page/debug/wow-sim --no-addons --no-saved-vars lua-errors` | `43dd7ed8e` | 0 | `p725-startup.txt` / not invalidated |
| p725-validator-fixtures | `python3 -B tools/test_patch_audit_validation.py` | `43dd7ed8e` | 0 | `p725-validator-fixtures.txt` / not invalidated |
| p725-verification | `python3 -B /home/osso/.worktrees/wow-ui-sim-p725-page/data/patch-api/evidence/7.2.5-session-2026-10-08/run_verification.py` | `43dd7ed8e` | 1 | `p725-verification.txt` / expected retained failure; same base boundary, not all-green |
