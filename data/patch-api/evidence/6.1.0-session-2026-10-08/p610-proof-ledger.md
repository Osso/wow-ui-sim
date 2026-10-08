# Proof ledger

Revision/filter scope is authoritative; later docs/session-only changes do not invalidate these proofs. No full suite or redundant broad rerun.

- `c7aeb5dbc3e2ad78e9f7772f9908d31a6ff2be9e` / `p610-all-sweeps.proof.json`: `["cargo", "test", "--test", "prefork_full_ui", "--", "publication_sweep"]`; exit 0; log `p610-all-sweeps.txt`; invalidated=False.
- `882d8eb6c810fb88795b25724899f5965ab1c56c` / `p610-discovery-red.proof.json`: `["cargo", "test", "--test", "prefork_full_ui", "--", "patch_6_1_0"]`; exit 1; log `p610-discovery-red.txt`; invalidated=False.
- `c7aeb5dbc3e2ad78e9f7772f9908d31a6ff2be9e` / `p610-discovery.proof.json`: `["cargo", "test", "--test", "prefork_full_ui", "--", "patch_6_1_0"]`; exit 0; log `p610-discovery.txt`; invalidated=False.
- `882d8eb6c810fb88795b25724899f5965ab1c56c` / `p610-extractor-fixtures-red.proof.json`: `["python3", "-B", "/home/osso/.worktrees/wow-ui-sim-p610-page/tools/test_extract_patch_non_inventory.py"]`; exit 1; log `p610-extractor-fixtures-red.txt`; invalidated=False.
- `c7aeb5dbc3e2ad78e9f7772f9908d31a6ff2be9e` / `p610-extractor-fixtures.proof.json`: `["python3", "-B", "/home/osso/.worktrees/wow-ui-sim-p610-page/tools/test_extract_patch_non_inventory.py"]`; exit 0; log `p610-extractor-fixtures.txt`; invalidated=False.
- `c7aeb5dbc3e2ad78e9f7772f9908d31a6ff2be9e` / `p610-format.proof.json`: `["cargo", "fmt", "--check"]`; exit 0; log `p610-format.txt`; invalidated=False.
- `882d8eb6c810fb88795b25724899f5965ab1c56c` / `p610-generator-fixtures.proof.json`: `["python3", "-B", "/home/osso/.worktrees/wow-ui-sim-p610-page/tools/test_gen_patch_wikitext_register.py"]`; exit 0; log `p610-generator-fixtures.txt`; invalidated=False.
- `c7aeb5dbc3e2ad78e9f7772f9908d31a6ff2be9e` / `p610-legacy-absence.proof.json`: `["cargo", "test", "--test", "integration", "p1200_removed_plain_globals"]`; exit 0; log `p610-legacy-absence.txt`; invalidated=False.
- `c7aeb5dbc3e2ad78e9f7772f9908d31a6ff2be9e` / `p610-mists-check.proof.json`: `["cargo", "check", "--no-default-features", "--features", "sound,gui,casc,client-mists", "--tests"]`; exit 0; log `p610-mists-check.txt`; invalidated=False.
- `c7aeb5dbc3e2ad78e9f7772f9908d31a6ff2be9e` / `p610-negative.proof.json`: `["cargo", "test", "--test", "prefork_full_ui", "--", "patch_6_1_0"]`; exit 1; log `p610-negative.txt`; invalidated=False.
- `c7aeb5dbc3e2ad78e9f7772f9908d31a6ff2be9e` / `p610-prior-validator-driver.proof.json`: `["python3", "-B", "/home/osso/.worktrees/wow-ui-sim-p610-page/data/patch-api/evidence/6.1.0-session-2026-10-08/check_prior_validators.py"]`; exit 0; log `p610-prior-validator-driver.txt`; invalidated=False.
- `c7aeb5dbc3e2ad78e9f7772f9908d31a6ff2be9e` / `p610-recap-model.proof.json`: `["cargo", "test", "--test", "integration", "c_death_recap_probes"]`; exit 0; log `p610-recap-model.txt`; invalidated=False.
- `c7aeb5dbc3e2ad78e9f7772f9908d31a6ff2be9e` / `p610-recap-prefork.proof.json`: `["cargo", "test", "--test", "prefork_full_ui", "--", "death_recap"]`; exit 0; log `p610-recap-prefork.txt`; invalidated=False.
- `882d8eb6c810fb88795b25724899f5965ab1c56c` / `p610-source-reproduction.proof.json`: `["python3", "-B", "/home/osso/.worktrees/wow-ui-sim-p610-page/data/patch-api/evidence/6.1.0-session-2026-10-08/reproduce_sources.py"]`; exit 0; log `p610-source-reproduction.txt`; invalidated=False.
- `c7aeb5dbc3e2ad78e9f7772f9908d31a6ff2be9e` / `p610-targeted-driver.proof.json`: `["python3", "-B", "/home/osso/.worktrees/wow-ui-sim-p610-page/data/patch-api/evidence/6.1.0-session-2026-10-08/verify_targeted.py"]`; exit 0; log `p610-targeted-driver.txt`; invalidated=False.
- `882d8eb6c810fb88795b25724899f5965ab1c56c` / `p610-validator-fixtures.proof.json`: `["python3", "-B", "/home/osso/.worktrees/wow-ui-sim-p610-page/tools/test_patch_audit_validation.py"]`; exit 0; log `p610-validator-fixtures.txt`; invalidated=False.
