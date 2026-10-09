# Integrated command proof ledger

Commands below ran against committed inputs; receipt revisions identify exact scope. No runtime source changed relative to master a82b8eb1c.

| Command | Result | Receipt |
|---|---|---|
| `cargo test --test prefork_full_ui -- publication_sweep` | 61 passed | `all-sweeps.proof.json` |
| `python3 -B /home/osso/.worktrees/wow-ui-sim-p530-page/data/patch-api/evidence/5.3.0-session-2026-10-08/integrated/run_checks.py` | PASS | `checks.proof.json` |
| `cargo fmt --check` | PASS | `format.proof.json` |
| `cargo test --test integration -- patch_5_3_0` | 1 passed | `integration-patch_5_3_0.proof.json` |
| `cargo test --test prefork_full_ui -- publication_sweep` | 60 passed | `master-all-sweeps.proof.json` |
| `cargo test --no-default-features --features sound,gui,casc,client-mists --test integration -- publication_sweep` | 6 passed | `master-mists-all-sweeps.proof.json` |
| `cargo test --no-default-features --features sound,gui,casc,client-mists --test integration -- publication_sweep` | 6 passed | `mists-all-sweeps.proof.json` |
| `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | PASS | `mists-check.proof.json` |
| `cargo test --test prefork_full_ui -- patch_5_3_0_publication_sweep` | expected negative failure | `negative.proof.json` |
| `cargo test --test prefork_full_ui -- patch_5_3_0` | 2 passed | `prefork-patch_5_3_0.proof.json` |
| `python3 -B /home/osso/.worktrees/wow-ui-sim-p530-page/data/patch-api/evidence/5.3.0-session-2026-10-08/integrated/reproduce_sources.py` | PASS | `reproduction.proof.json` |
| `python3 -B /home/osso/.worktrees/wow-ui-sim-p530-page/tools/test_check_patch_validators.py` | PASS | `test_check_patch_validators.proof.json` |
| `python3 -B /home/osso/.worktrees/wow-ui-sim-p530-page/tools/test_extract_patch_non_inventory.py` | PASS | `test_extract_patch_non_inventory.proof.json` |
| `python3 -B /home/osso/.worktrees/wow-ui-sim-p530-page/tools/test_gen_patch_wikitext_register.py` | PASS | `test_gen_patch_wikitext_register.proof.json` |
| `python3 -B /home/osso/.worktrees/wow-ui-sim-p530-page/tools/test_patch_audit_validation.py` | PASS | `test_patch_audit_validation.proof.json` |
| `python3 -B /home/osso/.worktrees/wow-ui-sim-p530-page/tools/test_patch_mists_extract.py` | PASS | `test_patch_mists_extract.proof.json` |
| `python3 -B /home/osso/.worktrees/wow-ui-sim-p530-page/tools/test_patch_mists_register.py` | PASS | `test_patch_mists_register.proof.json` |
| `python3 -B /home/osso/.worktrees/wow-ui-sim-p530-page/tools/test_patch_mists_transclusion.py` | PASS | `test_patch_mists_transclusion.proof.json` |
| `python3 -B /home/osso/.worktrees/wow-ui-sim-p530-page/tools/test_patch_warlords_register.py` | PASS | `test_patch_warlords_register.proof.json` |
| `cargo test --test prefork_full_ui -- patch_5_3_0_publication_sweep` | 1 passed | `own-sweep.proof.json` |
