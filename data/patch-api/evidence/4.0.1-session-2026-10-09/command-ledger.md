# 4.0.1 bounded command ledger

No broad or final gate. Receipts pin exact revision/scope/environment and log SHA-256. Compilation failure is not a behavioral RED. Uncommitted RED test snapshots are retained separately.

## extractor-regression.proof

- argv: `["python3", "-B", "tools/test_extract_patch_non_inventory.py"]`
- revision: `6c041621fd3ea0496befb12eb11d936a4e11f1cd`; targeted parser/extractor fixtures only
- exit: 0; log: `extractor-regression.log`
- environment: `{}`

## generator-regression.proof

- argv: `["python3", "-B", "tools/test_gen_patch_wikitext_register.py"]`
- revision: `6c041621fd3ea0496befb12eb11d936a4e11f1cd`; targeted parser/extractor fixtures only
- exit: 0; log: `generator-regression.log`
- environment: `{}`

## mists-factory-control.proof

- argv: `["cargo", "test", "--offline", "--no-default-features", "--features", "sound,gui,casc,client-mists", "--test", "integration", "patch_4_0_1_skill_headers_factory_surface", "--", "--nocapture"]`
- revision: `d2dfdb4a46cf8de969344a2b7d9194cb2d403688`; targeted own publication or skill-header behavior only
- exit: 0; log: `mists-factory-control.log`
- environment: `{}`

## mists-prefork-control.proof

- argv: `["cargo", "test", "--offline", "--no-default-features", "--features", "sound,gui,casc,client-mists", "--test", "prefork_full_ui", "--", "patch_4_0_1_skill_headers_cached_surface", "--nocapture"]`
- revision: `d2dfdb4a46cf8de969344a2b7d9194cb2d403688`; own negative control or Classic cached skill-header preservation only
- exit: 101; log: `mists-prefork-control.log`
- environment: `{}`

## parser-green.proof

- argv: `["python3", "-B", "tools/test_patch_cataclysm_register.py"]`
- revision: `6c041621fd3ea0496befb12eb11d936a4e11f1cd`; targeted parser/extractor fixtures only
- exit: 0; log: `parser-green.log`
- environment: `{}`

## parser-red-assertions.proof

- argv: `["python3", "-B", "tools/test_patch_cataclysm_register.py"]`
- revision: `29b56e5c18ecdb214bf0b27d46e5bebf492130b7`; uncommitted RED tests now report missing extractor option as assertion; parser/extractor unchanged
- exit: 1; log: `parser-red-assertions.log`
- environment: `{}`

## parser-red.proof

- argv: `["python3", "-B", "tools/test_patch_cataclysm_register.py"]`
- revision: `29b56e5c18ecdb214bf0b27d46e5bebf492130b7`; test_patch_cataclysm_register.py uncommitted RED tests; parser/extractor unchanged
- exit: 1; log: `parser-red.log`
- environment: `{}`

## publication-default-red.proof

- argv: `["cargo", "test", "--offline", "--test", "prefork_full_ui", "--", "patch_4_0_1_publication_sweep", "--nocapture"]`
- revision: `6c041621fd3ea0496befb12eb11d936a4e11f1cd`; own retail publication sweep; default features
- exit: 1; log: `publication-default-red.log`
- environment: `{"P401_SWEEP_OUT": "/home/osso/.worktrees/wow-ui-sim-p401-source/data/patch-api/evidence/4.0.1-session-2026-10-09/publication-default-red-results.json"}`

## publication-negative.proof

- argv: `["cargo", "test", "--offline", "--test", "prefork_full_ui", "--", "patch_4_0_1_publication_sweep", "--nocapture"]`
- revision: `d2dfdb4a46cf8de969344a2b7d9194cb2d403688`; own negative control or Classic cached skill-header preservation only
- exit: 1; log: `publication-negative.log`
- environment: `{"P401_SWEEP_REGISTER": "/home/osso/.worktrees/wow-ui-sim-p401-source/data/patch-api/evidence/4.0.1-session-2026-10-09/fabricated-register.json", "P401_SWEEP_OUT": "/home/osso/.worktrees/wow-ui-sim-p401-source/data/patch-api/evidence/4.0.1-session-2026-10-09/publication-negative-results.json"}`

## publication-red.proof

- argv: `["cargo", "test", "--offline", "--no-default-features", "--features", "client-retail", "--test", "prefork_full_ui", "--", "patch_4_0_1_publication_sweep", "--nocapture"]`
- revision: `6c041621fd3ea0496befb12eb11d936a4e11f1cd`; own retail publication sweep; headless feature profile; no runtime edits
- exit: 101; log: `publication-red.log`
- environment: `{"P401_SWEEP_OUT": "/home/osso/.worktrees/wow-ui-sim-p401-source/data/patch-api/evidence/4.0.1-session-2026-10-09/publication-red-results.json"}`

## retail-factory-green.proof

- argv: `["cargo", "test", "--offline", "--test", "integration", "patch_4_0_1_skill_headers_factory_surface", "--", "--nocapture"]`
- revision: `d2dfdb4a46cf8de969344a2b7d9194cb2d403688`; targeted own publication or skill-header behavior only
- exit: 0; log: `retail-factory-green.log`
- environment: `{}`

## retail-prefork-green.proof

- argv: `["cargo", "test", "--offline", "--test", "prefork_full_ui", "--", "patch_4_0_1", "--nocapture"]`
- revision: `d2dfdb4a46cf8de969344a2b7d9194cb2d403688`; targeted own publication or skill-header behavior only
- exit: 0; log: `retail-prefork-green.log`
- environment: `{"P401_SWEEP_OUT": "/home/osso/.worktrees/wow-ui-sim-p401-source/data/patch-api/evidence/4.0.1-session-2026-10-09/publication-green-results.json"}`

## retirement-red.proof

- argv: `["cargo", "test", "--offline", "--test", "integration", "patch_4_0_1_skill_headers_factory_surface", "--", "--nocapture"]`
- revision: `6c041621f`; uncommitted patch_4_0_1_retirements.rs RED tests; runtime unchanged
- exit: 101; log: `retirement-red.log`
- environment: `{}`

