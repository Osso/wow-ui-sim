# Command proof ledger

Runtime/source/test acceptance scope: 7ba500c07. Later changes affect only evidence, accounting and docs; no relevant runtime/test/tool proof invalidated. Discovery proves its original sweep before the known-gap fixture was populated, not final acceptance. No full integration suite or startup-output claim.

| Scope | Revision | Exit | Command |
|---|---|---|---|
| p547-all-sweeps | `7ba500c07d5b52e7aed21195de0c355b99ce4082` | 0 | `["cargo", "test", "--test", "prefork_full_ui", "--", "publication_sweep"]` |
| p547-bnet-namespace | `7ba500c07d5b52e7aed21195de0c355b99ce4082` | 0 | `["cargo", "test", "--test", "integration", "p1200_rest_battle_net_outbound"]` |
| p547-cached-bnet | `7ba500c07d5b52e7aed21195de0c355b99ce4082` | 0 | `["cargo", "test", "--test", "prefork_full_ui", "--", "blizzard_deprecated_battle_net"]` |
| p547-discovery | `13f6e9f8f26087a7c79c7cf13ae749474536e0ba` | 1 | `["cargo", "test", "--test", "prefork_full_ui", "--", "patch_5_4_7"]` |
| p547-format | `84db5bf2b72f9741126dcb30d34e180144c44197` | 0 | `["cargo", "fmt", "--all", "--check"]` |
| p547-mists | `7ba500c07d5b52e7aed21195de0c355b99ce4082` | 0 | `["cargo", "check", "--no-default-features", "--features", "sound,gui,casc,client-mists", "--tests"]` |
| p547-negative | `500543bd7b220fc884ba78e2724b71e6196536a6` | 1 | `["cargo", "test", "--test", "prefork_full_ui", "--", "patch_5_4_7_publication_sweep"]` |
| p547-own-behavior | `7ba500c07d5b52e7aed21195de0c355b99ce4082` | 0 | `["cargo", "test", "--test", "prefork_full_ui", "--", "patch_5_4_7"]` |
| p547-reproduction | `13f6e9f8f26087a7c79c7cf13ae749474536e0ba` | 0 | `["python3", "-B", "/home/osso/.worktrees/wow-ui-sim-p547-page/data/patch-api/evidence/5.4.7-session-2026-10-08/reproduce_sources.py"]` |
| p547-specialization | `7ba500c07d5b52e7aed21195de0c355b99ce4082` | 0 | `["cargo", "test", "--test", "integration", "patch_11_1_0_specialization_names_use_catalog_identity"]` |
| p547-targeted-driver | `7ba500c07d5b52e7aed21195de0c355b99ce4082` | 0 | `["python3", "-B", "/home/osso/.worktrees/wow-ui-sim-p547-page/data/patch-api/evidence/5.4.7-session-2026-10-08/verify_targeted.py"]` |
| test_check_patch_validators | `13f6e9f8f26087a7c79c7cf13ae749474536e0ba` | 0 | `["python3", "-B", "/home/osso/.worktrees/wow-ui-sim-p547-page/tools/test_check_patch_validators.py"]` |
| test_extract_patch_non_inventory | `13f6e9f8f26087a7c79c7cf13ae749474536e0ba` | 0 | `["python3", "-B", "/home/osso/.worktrees/wow-ui-sim-p547-page/tools/test_extract_patch_non_inventory.py"]` |
| test_gen_patch_wikitext_register | `13f6e9f8f26087a7c79c7cf13ae749474536e0ba` | 0 | `["python3", "-B", "/home/osso/.worktrees/wow-ui-sim-p547-page/tools/test_gen_patch_wikitext_register.py"]` |
| test_patch_audit_validation | `13f6e9f8f26087a7c79c7cf13ae749474536e0ba` | 0 | `["python3", "-B", "/home/osso/.worktrees/wow-ui-sim-p547-page/tools/test_patch_audit_validation.py"]` |
| test_patch_warlords_register | `13f6e9f8f26087a7c79c7cf13ae749474536e0ba` | 0 | `["python3", "-B", "/home/osso/.worktrees/wow-ui-sim-p547-page/tools/test_patch_warlords_register.py"]` |
