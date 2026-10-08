# 7.0.1 proof ledger

Runtime scope: `2c43df7ea924009562b906689d9b1c5c85fd8e2f`; base: `25fbde058514034692df481bd568e952a2b9851e`.
Only evidence/wiki changes afterward; none invalidate Rust or fixture proof. No src/tools changes.

## p701-discovery
Revision: `2c43df7ea924009562b906689d9b1c5c85fd8e2f`; exit: 0; invalidated: False.
Command: `["cargo", "test", "--test", "prefork_full_ui", "--", "patch_7_0_1"]`
Log: `p701-discovery.txt`; SHA-256: `9ec985bb3060fdfa7cc05bfe7dcc5c7c9afbea699e73e0404abb670f59476992`.

## p701-all-sweeps
Revision: `3026d96d129a6bbb9a48d0e8db85e0cb3f44230f`; exit: 0; invalidated: False.
Command: `["cargo", "test", "--test", "prefork_full_ui", "--", "publication_sweep"]`
Log: `p701-all-sweeps.txt`; SHA-256: `07ed892153d101f8fe4280086c68c50e61fb362aec10218be19a050ffe3ceb20`.

## p701-format
Revision: `950c52aac2ca69dde4231d3b99168afdf0179986`; exit: 0; invalidated: False.
Command: `["cargo", "fmt", "--check"]`
Log: `p701-format.txt`; SHA-256: `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.

## p701-mists-check
Revision: `950c52aac2ca69dde4231d3b99168afdf0179986`; exit: 0; invalidated: False.
Command: `["cargo", "check", "--no-default-features", "--features", "sound,gui,casc,client-mists", "--tests"]`
Log: `p701-mists-check.txt`; SHA-256: `13f8d4b32afd948ff2e39badcfd42dda5477ed957d3dce56a11773e06b25bb6f`.

## p701-source-reproduction
Revision: `2c43df7ea924009562b906689d9b1c5c85fd8e2f`; exit: 0; invalidated: False.
Command: `["python3", "-B", "/home/osso/.worktrees/wow-ui-sim-p701-page/data/patch-api/evidence/7.0.1-session-2026-10-08/reproduce_sources.py"]`
Log: `p701-source-reproduction.txt`; SHA-256: `e55183daaf12cad7bd02034a3bce5bdfd54083b6deaff5f6e13f53375d406193`.

## p701-generator-fixtures
Revision: `2c43df7ea924009562b906689d9b1c5c85fd8e2f`; exit: 0; invalidated: False.
Command: `["python3", "-B", "/home/osso/.worktrees/wow-ui-sim-p701-page/tools/test_gen_patch_wikitext_register.py"]`
Log: `p701-generator-fixtures.txt`; SHA-256: `172019ae7b5319b3ef5cc3a75594922082ec53c4b66573b92209c867ce210574`.

## p701-extractor-fixtures
Revision: `2c43df7ea924009562b906689d9b1c5c85fd8e2f`; exit: 0; invalidated: False.
Command: `["python3", "-B", "/home/osso/.worktrees/wow-ui-sim-p701-page/tools/test_extract_patch_non_inventory.py"]`
Log: `p701-extractor-fixtures.txt`; SHA-256: `374bc0374f43e0b7c8dd3c8c44089677ed511cf36466830931b579933a6c8f1a`.

## p701-validator-fixtures
Revision: `2c43df7ea924009562b906689d9b1c5c85fd8e2f`; exit: 0; invalidated: False.
Command: `["python3", "-B", "/home/osso/.worktrees/wow-ui-sim-p701-page/tools/test_patch_audit_validation.py"]`
Log: `p701-validator-fixtures.txt`; SHA-256: `507f371f8e037ea497fc2c22cdb5e657197d994648cfcdfa64c7990d1c62d241`.

## p701-targeted-driver
Revision: `3026d96d129a6bbb9a48d0e8db85e0cb3f44230f`; exit: 0; invalidated: False.
Command: `["python3", "-B", "/home/osso/.worktrees/wow-ui-sim-p701-page/data/patch-api/evidence/7.0.1-session-2026-10-08/verify_targeted.py"]`
Log: `p701-targeted-driver.txt`; SHA-256: `7362982fac33fcff83b035f3d541e518235c39e4b5f09607b851cecf5d829293`.

