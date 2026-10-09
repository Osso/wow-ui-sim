# 5.4.2 integrated command ledger

Base: `896086537a2b3c1ead5886d5ae3e430d56e7ef20`. Own source/test scope: `feb7b155b86821baa8f312aaea8bad0e8f3b5189`.
All Cargo commands use `CARGO_TARGET_DIR=/home/osso/.cache/wow-ui-sim-targets/p542-page`.
Cargo commands ran sequentially in logged asynchronous drivers; prior validators and source reproduction ran independently. No poll-wait or repeated broad checks.

| Receipt | Command | Revision | Exit |
|---|---|---|---|
| own-sweep | `cargo test --test prefork_full_ui -- patch_5_4_2_publication_sweep` | `feb7b155b86821baa8f312aaea8bad0e8f3b5189` | 0 |
| all-sweeps | `cargo test --test prefork_full_ui -- publication_sweep` | `feb7b155b86821baa8f312aaea8bad0e8f3b5189` | 0 |
| prefork-patch_5_4_2 | `cargo test --test prefork_full_ui -- patch_5_4_2` | `feb7b155b86821baa8f312aaea8bad0e8f3b5189` | 0 |
| integration-patch_5_4_2 | `cargo test --test integration -- patch_5_4_2` | `feb7b155b86821baa8f312aaea8bad0e8f3b5189` | 0 |
| negative | `cargo test --test prefork_full_ui -- patch_5_4_2_publication_sweep` | `feb7b155b86821baa8f312aaea8bad0e8f3b5189` | 1 |
| reproduction | `python3 -B /home/osso/.worktrees/wow-ui-sim-p542-page/data/patch-api/evidence/5.4.2-session-2026-10-08/integrated/reproduce_sources.py` | `feb7b155b86821baa8f312aaea8bad0e8f3b5189` | 0 |
| prior-validators | `python3 -B /home/osso/.worktrees/wow-ui-sim-p542-page/data/patch-api/evidence/5.4.2-session-2026-10-08/integrated/check_prior.py` | `93f0be2a04b76b9b5a7760b23756063499559f78` | 0 |
| checks | `python3 -B /home/osso/.worktrees/wow-ui-sim-p542-page/data/patch-api/evidence/5.4.2-session-2026-10-08/integrated/run_checks.py` | `feb7b155b86821baa8f312aaea8bad0e8f3b5189` | 0 |
| master-all-sweeps | `cargo test --test prefork_full_ui -- publication_sweep` | `896086537a2b3c1ead5886d5ae3e430d56e7ef20` | 0 |
| format | `cargo fmt --check` | `feb7b155b86821baa8f312aaea8bad0e8f3b5189` | 0 |
| mists-check | `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | `feb7b155b86821baa8f312aaea8bad0e8f3b5189` | 0 |
| test_check_patch_validators | `python3 -B /home/osso/.worktrees/wow-ui-sim-p542-page/tools/test_check_patch_validators.py` | `feb7b155b86821baa8f312aaea8bad0e8f3b5189` | 0 |
| test_extract_patch_non_inventory | `python3 -B /home/osso/.worktrees/wow-ui-sim-p542-page/tools/test_extract_patch_non_inventory.py` | `feb7b155b86821baa8f312aaea8bad0e8f3b5189` | 0 |
| test_gen_patch_wikitext_register | `python3 -B /home/osso/.worktrees/wow-ui-sim-p542-page/tools/test_gen_patch_wikitext_register.py` | `feb7b155b86821baa8f312aaea8bad0e8f3b5189` | 0 |
| test_patch_audit_validation | `python3 -B /home/osso/.worktrees/wow-ui-sim-p542-page/tools/test_patch_audit_validation.py` | `feb7b155b86821baa8f312aaea8bad0e8f3b5189` | 0 |
| test_patch_warlords_register | `python3 -B /home/osso/.worktrees/wow-ui-sim-p542-page/tools/test_patch_warlords_register.py` | `feb7b155b86821baa8f312aaea8bad0e8f3b5189` | 0 |
| test_patch_mists_register | `python3 -B /home/osso/.worktrees/wow-ui-sim-p542-page/tools/test_patch_mists_register.py` | `feb7b155b86821baa8f312aaea8bad0e8f3b5189` | 0 |
| mists-all-sweeps | `cargo test --no-default-features --features sound,gui,casc,client-mists --test integration -- publication_sweep` | `aff59aa37b20c433810d18842feabc7c017cd9fa` | 0 |
| master-mists-all-sweeps | `cargo test --no-default-features --features sound,gui,casc,client-mists --test integration -- publication_sweep` | `896086537a2b3c1ead5886d5ae3e430d56e7ef20` | 0 |
| mists-sweeps-driver | `python3 -B /home/osso/.worktrees/wow-ui-sim-p542-page/data/patch-api/evidence/5.4.2-session-2026-10-08/integrated/run_mists_sweeps.py 1789639` | `0041b518cdca9379bc5c66c971578ac4df92a9f4` | 0 |
| historical-replay | `python3 -B /home/osso/.worktrees/wow-ui-sim-p542-page/data/patch-api/evidence/5.4.2-session-2026-10-08/validate.py` | `d47f90834a4f8501bbffe11917e1038c4409307a` | 0 |
| history-without-original-objects | `python3 -B /home/osso/.worktrees/wow-ui-sim-p542-history-proof/data/patch-api/evidence/5.4.2-session-2026-10-08/validate.py` | `d47f90834a4f8501bbffe11917e1038c4409307a` | 0 |

Full environment, duration, log digest and revision are in each `.proof.json`.
Integration `patch_5_4_2` selects zero cases; prefork owns all three 5.4.2 tests. No integration behavior claim.
Separate Mists integration publication sweeps cover the 5.5.3/5.5.4 empty inventories and SharedXML startup only, not full UI or API behavior.
Negative control must fail with exactly one added gap (39 → 40). Three inherited extract failures are unchanged.
No runtime, vendor or retirement changes; parser flags coexist with merged master flags. No startup comparison or full integration-suite claim.
Later evidence/wiki/validator-only commits do not invalidate the pinned Rust, shared-tool or source scopes.
