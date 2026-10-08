# 7.1.0 command proof ledger

Runtime/test scope: `7778521e8`; source parser/register scope unchanged since `68b72c561`. No src/ changes against master base `8b6131f36`. Source preservation/reproduction scope contains all 45 registers.

| Receipt | Command | Revision | Exit | Proof / invalidation |
|---|---|---|---|---|
| p710-all-sweeps | `cargo test --test prefork_full_ui -- publication_sweep` | `16154a1e5` | 0 | passed; no later relevant change |
| p710-behavior-discovery | `cargo test --test prefork_full_ui -- patch_7_1_0` | `16154a1e5` | 1 | retained custom-intrinsic failure |
| p710-discovery | `cargo test --test prefork_full_ui -- patch_7_1_0` | `68b72c561` | 0 | passed; no later relevant change |
| p710-extractor-fixtures | `python3 -B tools/test_extract_patch_non_inventory.py` | `16154a1e5` | 0 | passed; no later relevant change |
| p710-final-format | `cargo fmt --check` | `084021fba` | 0 | passed; no later relevant change |
| p710-format | `cargo fmt --check` | `16154a1e5` | 0 | passed; no later relevant change |
| p710-generator-fixtures | `python3 -B tools/test_gen_patch_wikitext_register.py` | `16154a1e5` | 0 | passed; no later relevant change |
| p710-intrinsic | `cargo test --test integration intrinsic_types::` | `16154a1e5` | 0 | passed; no later relevant change |
| p710-items | `cargo test --test integration c_item_api::` | `16154a1e5` | 0 | passed; no later relevant change |
| p710-mists-check | `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | `16154a1e5` | 0 | passed; no later relevant change |
| p710-negative | `cargo test --test prefork_full_ui -- patch_7_1_0_publication_sweep` | `7778521e8` | 1 | expected one-row rejection |
| p710-other-validators | `python3 -B /home/osso/.worktrees/wow-ui-sim-p710-page/data/patch-api/evidence/7.1.0-session-2026-10-08/check_other_validators.py` | `16154a1e5` | 0 | passed; no later relevant change |
| p710-own-bounded | `cargo test --test prefork_full_ui -- patch_7_1_0` | `084021fba` | 0 | passed; no later relevant change |
| p710-own-green | `cargo test --test prefork_full_ui -- patch_7_1_0` | `16154a1e5` | 1 | retained custom-intrinsic failure |
| p710-screen | `cargo test --test integration screen_mode::` | `16154a1e5` | 0 | passed; no later relevant change |
| p710-source-reproduction | `python3 -B /home/osso/.worktrees/wow-ui-sim-p710-page/data/patch-api/evidence/7.1.0-session-2026-10-08/reproduce_sources.py` | `68b72c561` | 0 | passed; no later relevant change |
| p710-targeted-driver | `python3 -B /home/osso/.worktrees/wow-ui-sim-p710-page/data/patch-api/evidence/7.1.0-session-2026-10-08/verify_targeted.py` | `16154a1e5` | 1 | retained custom-intrinsic failure |
| p710-validator-fixtures | `python3 -B tools/test_patch_audit_validation.py` | `16154a1e5` | 0 | passed; no later relevant change |
