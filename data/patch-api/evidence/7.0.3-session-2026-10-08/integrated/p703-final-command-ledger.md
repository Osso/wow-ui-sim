# Integrated 7.0.3 proof ledger

Runtime/source scope: `771a7d7cc` (same src/tests/tools/source bytes as final integration). Master: `25fbde058`. Commands run from the owned worktree; master uses a Git-archive manifest, not another checkout. Target: `/home/osso/.cache/wow-ui-sim-targets/p703-page`.

| Command | Exit | Result | Receipt |
|---|---:|---|---|
| `cargo test --test prefork_full_ui -- patch_7_0_3_publication_sweep` | 0 | 1 passed | [own-sweep](own-sweep.proof.json) |
| `cargo test --test prefork_full_ui -- publication_sweep` | 0 | 48 passed | [all-sweeps](all-sweeps.proof.json) |
| `cargo test --test integration -- patch_7_0_3 --nocapture` | 0 | 3 passed | [integration-patch-7-0-3](integration-patch-7-0-3.proof.json) |
| `cargo test --test integration -- professions_api:: --nocapture` | 0 | 1 passed, 35 passed | [integration-professions-api](integration-professions-api.proof.json) |
| `cargo test --test integration -- test_crafting:: --nocapture` | 0 | 23 passed | [integration-test-crafting](integration-test-crafting.proof.json) |
| `cargo test --test integration -- trade_info:: --nocapture` | 0 | 1 passed | [integration-trade-info](integration-trade-info.proof.json) |
| `cargo test --test integration -- c_collection_api:: --nocapture` | 0 | 37 passed | [integration-c-collection-api](integration-c-collection-api.proof.json) |
| `cargo test --test integration -- c_function_diff_coverage:: --nocapture` | 0 | 4 passed | [integration-c-function-diff-coverage](integration-c-function-diff-coverage.proof.json) |
| `cargo test --test integration -- test_showuipanel_professions_crafting:: --nocapture` | 0 | 1 passed, 1 passed | [integration-test-showuipanel-professions-crafting](integration-test-showuipanel-professions-crafting.proof.json) |
| `cargo test --test prefork_full_ui -- patch_7_0_3 --nocapture` | 0 | 3 passed | [prefork_full_ui-patch-7-0-3](prefork_full_ui-patch-7-0-3.proof.json) |
| `cargo test --test prefork_full_ui -- professions --nocapture` | 0 | 21 passed | [prefork_full_ui-professions](prefork_full_ui-professions.proof.json) |
| `cargo test --test prefork_full_ui -- crafting --nocapture` | 0 | 0 selected; no coverage credited | [prefork_full_ui-crafting](prefork_full_ui-crafting.proof.json) |
| `cargo test --test prefork_full_ui -- trade_skill --nocapture` | 0 | 0 selected; no coverage credited | [prefork_full_ui-trade-skill](prefork_full_ui-trade-skill.proof.json) |
| `cargo test --test prefork_full_ui -- mount --nocapture` | 0 | 1 passed | [prefork_full_ui-mount](prefork_full_ui-mount.proof.json) |
| `cargo test --test prefork_full_ui -- patch_7_0_3_publication_sweep` | 1 | expected 52→53 gaps | [negative](negative.proof.json) |
| `cargo build --bin wow-sim` | 0 | success | [build](build.proof.json) |
| `timeout 90 /home/osso/.cache/wow-ui-sim-targets/p703-page/debug/wow-sim --no-saved-vars lua-errors` | 0 | success | [startup](startup.proof.json) |
| `python3 -B /home/osso/.worktrees/wow-ui-sim-p703-page/tools/test_gen_patch_wikitext_register.py` | 0 | 31 fixtures | [gen_patch_wikitext_register-fixtures](gen_patch_wikitext_register-fixtures.proof.json) |
| `python3 -B /home/osso/.worktrees/wow-ui-sim-p703-page/tools/test_extract_patch_non_inventory.py` | 0 | 35 fixtures | [extract_patch_non_inventory-fixtures](extract_patch_non_inventory-fixtures.proof.json) |
| `python3 -B /home/osso/.worktrees/wow-ui-sim-p703-page/tools/test_patch_audit_validation.py` | 0 | 8 fixtures | [patch_audit_validation-fixtures](patch_audit_validation-fixtures.proof.json) |
| `cargo fmt --check` | 0 | success | [format](format.proof.json) |
| `python3 -B /home/osso/.worktrees/wow-ui-sim-p703-page/tools/extend_patch_audit_receipts.py /home/osso/.worktrees/wow-ui-sim-p703-page /home/osso/.worktrees/wow-ui-sim-p703-page/data/patch-api/evidence/7.0.3-session-2026-10-08/integrated/extension p703 merged 7.1.0; recorded provenance flags 7.0.3` | 0 | success | [extend-receipts](extend-receipts.proof.json) |
| `cargo test --manifest-path /home/osso/.cache/wow-ui-sim-targets/p703-page/master-25fbde058/Cargo.toml --test prefork_full_ui -- publication_sweep` | 0 | 47 passed | [master-all-sweeps](master-all-sweeps.proof.json) |
| `cargo build --manifest-path /home/osso/.cache/wow-ui-sim-targets/p703-page/master-25fbde058/Cargo.toml --bin wow-sim` | 0 | success | [master-build](master-build.proof.json) |
| `timeout 90 /home/osso/.cache/wow-ui-sim-targets/p703-page/debug/wow-sim --no-saved-vars lua-errors` | 0 | success | [master-startup](master-startup.proof.json) |
| `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | 0 | success | [mists-check](mists-check.proof.json) |
| `cargo test --no-default-features --features sound,gui,casc,client-mists --test integration -- patch_7_0_3_recipe_name_filter_changes_catalog_results --nocapture` | 0 | 1 passed | [mists-patch-7-0-3-recipe-name-filter-changes-catalog-results](mists-patch-7-0-3-recipe-name-filter-changes-catalog-results.proof.json) |
| `cargo test --no-default-features --features sound,gui,casc,client-mists --test integration -- mists_trade_skill_api:: --nocapture` | 0 | 1 passed | [mists-mists-trade-skill-api](mists-mists-trade-skill-api.proof.json) |
| `cargo test --no-default-features --features sound,gui,casc,client-mists --test integration -- professions_api:: --nocapture` | 0 | 1 passed, 35 passed | [mists-professions-api](mists-professions-api.proof.json) |
| `cargo test --no-default-features --features sound,gui,casc,client-mists --test integration -- c_collection_api::test_mount_journal --nocapture` | 0 | 12 passed | [mists-c-collection-api::test-mount-journal](mists-c-collection-api::test-mount-journal.proof.json) |

Register reproduction: 47/47 byte-identical, including recorded flags. Extracts: 44/47 identical; the exact three inherited 12.0.5/12.0.7 mismatches and 12.1.0 unknown-template failure match the integrated master evidence.

Publication: branch 48 cases, master 47; 47 pages / 9,016 observations. Own 52 exact gaps unchanged; every other page has unchanged IDs and ok/gap statuses. No supersession or LATER_AUDIT_REPLACEMENTS change required.

All scope/log hashes retained. Earlier successful commands were not rerun when the runner resumed; only the missing extension/master/Mists stages ran. Original 243 historical artifacts preserved byte-for-byte. Original Git proof history remains required and its seven audited commit mappings are explicit; five stable patch IDs match, two rebased contexts differ because 7.1.0 was added.

Six vendor iced manifest deprecations remain; zero non-vendor Mists warnings. No native/CASC visual proof claimed.
