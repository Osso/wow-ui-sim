# Integrated 5.2.0 proof ledger

Base: `5e4e82ef66b239da8ae23a8dbb6b276213e50c71`. Cargo commands use the dedicated external `p520-page` target. Historical receipts remain immutable. Shared parsers are master-owned (5.4.2/5.4.0/5.3.0); only bare-handler normalization is 5.2.0-specific.

Each receipt records command, exact revision, scope, exit, log hash and invalidation state. No source/test changes after the passing runtime proofs. Reproduction covers register/extractor inputs, not the subsequently updated own coverage ledger. Master receipts differ only in the three own 5.2.0 test/fixture files.

| Receipt | Command | Revision | Exit |
|---|---|---|---|
| all-sweeps | `cargo test --test prefork_full_ui -- publication_sweep` | `be12a5a778dd2f3791a330d2e3dbdabb8ceebbcd` | 0 |
| checks | `python3 -B /home/osso/.worktrees/wow-ui-sim-p520-page/data/patch-api/evidence/5.2.0-session-2026-10-08/integrated/run_checks.py` | `be12a5a778dd2f3791a330d2e3dbdabb8ceebbcd` | 0 |
| format | `cargo fmt --check` | `be12a5a778dd2f3791a330d2e3dbdabb8ceebbcd` | 0 |
| integration-patch_5_2_0 | `cargo test --test integration -- patch_5_2_0` | `be12a5a778dd2f3791a330d2e3dbdabb8ceebbcd` | 0 |
| master-all-sweeps | `cargo test --test prefork_full_ui -- publication_sweep` | `5e4e82ef66b239da8ae23a8dbb6b276213e50c71` | 0 |
| master-mists-all-sweeps | `cargo test --no-default-features --features sound,gui,casc,client-mists --test integration -- publication_sweep` | `5e4e82ef66b239da8ae23a8dbb6b276213e50c71` | 0 |
| mists-all-sweeps | `cargo test --no-default-features --features sound,gui,casc,client-mists --test integration -- publication_sweep` | `04bdddb76c4529cf3714b37bdc64d03102657952` | 0 |
| mists-check | `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | `be12a5a778dd2f3791a330d2e3dbdabb8ceebbcd` | 0 |
| negative | `cargo test --test prefork_full_ui -- patch_5_2_0_publication_sweep` | `be12a5a778dd2f3791a330d2e3dbdabb8ceebbcd` | 1 |
| own-sweep | `cargo test --test prefork_full_ui -- patch_5_2_0_publication_sweep` | `be12a5a778dd2f3791a330d2e3dbdabb8ceebbcd` | 0 |
| prefork-patch_5_2_0 | `cargo test --test prefork_full_ui -- patch_5_2_0` | `be12a5a778dd2f3791a330d2e3dbdabb8ceebbcd` | 0 |
| reproduction | `python3 -B /home/osso/.worktrees/wow-ui-sim-p520-page/data/patch-api/evidence/5.2.0-session-2026-10-08/integrated/reproduce_sources.py` | `68fccd5cfbe2c79b188993e541569c4a5a1de880` | 0 |
| test_check_patch_validators | `python3 -B /home/osso/.worktrees/wow-ui-sim-p520-page/tools/test_check_patch_validators.py` | `be12a5a778dd2f3791a330d2e3dbdabb8ceebbcd` | 0 |
| test_extract_patch_non_inventory | `python3 -B /home/osso/.worktrees/wow-ui-sim-p520-page/tools/test_extract_patch_non_inventory.py` | `be12a5a778dd2f3791a330d2e3dbdabb8ceebbcd` | 0 |
| test_gen_patch_wikitext_register | `python3 -B /home/osso/.worktrees/wow-ui-sim-p520-page/tools/test_gen_patch_wikitext_register.py` | `be12a5a778dd2f3791a330d2e3dbdabb8ceebbcd` | 0 |
| test_patch_audit_validation | `python3 -B /home/osso/.worktrees/wow-ui-sim-p520-page/tools/test_patch_audit_validation.py` | `be12a5a778dd2f3791a330d2e3dbdabb8ceebbcd` | 0 |
| test_patch_mists_520_register | `python3 -B /home/osso/.worktrees/wow-ui-sim-p520-page/tools/test_patch_mists_520_register.py` | `be12a5a778dd2f3791a330d2e3dbdabb8ceebbcd` | 0 |
| test_patch_mists_extract | `python3 -B /home/osso/.worktrees/wow-ui-sim-p520-page/tools/test_patch_mists_extract.py` | `be12a5a778dd2f3791a330d2e3dbdabb8ceebbcd` | 0 |
| test_patch_mists_register | `python3 -B /home/osso/.worktrees/wow-ui-sim-p520-page/tools/test_patch_mists_register.py` | `be12a5a778dd2f3791a330d2e3dbdabb8ceebbcd` | 0 |
| test_patch_mists_transclusion | `python3 -B /home/osso/.worktrees/wow-ui-sim-p520-page/tools/test_patch_mists_transclusion.py` | `be12a5a778dd2f3791a330d2e3dbdabb8ceebbcd` | 0 |
| test_patch_warlords_register | `python3 -B /home/osso/.worktrees/wow-ui-sim-p520-page/tools/test_patch_warlords_register.py` | `be12a5a778dd2f3791a330d2e3dbdabb8ceebbcd` | 0 |

Retail: branch 62/62, master 61/61. Mists: both 6/6. Own prefork 2/2, integration 1/1. Python 98/98. Format and Mists check pass; zero non-vendor warnings. Negative control fails exactly 55 → 56. All 65 other pages / 9,990 observations identical to pinned master.
