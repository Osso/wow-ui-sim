# 5.4.1 integrated command ledger

Base: `279a38f3d560e00e954d7dfb96e81f6be3668dd1`. Own source/test scope: `c394868cc0bca7b5161034c010fa292833d402ae`.
All Cargo commands use `CARGO_TARGET_DIR=/home/osso/.cache/wow-ui-sim-targets/p541-page`.
Cargo commands ran sequentially in logged asynchronous drivers; prior validators and source reproduction ran independently. No poll-wait or repeated broad checks.

| Receipt | Command | Revision | Exit |
|---|---|---|---|
| own-sweep | `cargo test --test prefork_full_ui -- patch_5_4_1_publication_sweep` | `c394868cc0bca7b5161034c010fa292833d402ae` | 0 |
| all-sweeps | `cargo test --test prefork_full_ui -- publication_sweep` | `c394868cc0bca7b5161034c010fa292833d402ae` | 0 |
| prefork-patch_5_4_1 | `cargo test --test prefork_full_ui -- patch_5_4_1` | `c394868cc0bca7b5161034c010fa292833d402ae` | 0 |
| integration-patch_5_4_1 | `cargo test --test integration -- patch_5_4_1` | `c394868cc0bca7b5161034c010fa292833d402ae` | 0 |
| negative | `cargo test --test prefork_full_ui -- patch_5_4_1_publication_sweep` | `c394868cc0bca7b5161034c010fa292833d402ae` | 1 |
| reproduction | `python3 -B /home/osso/.worktrees/wow-ui-sim-p541-page/data/patch-api/evidence/5.4.1-session-2026-10-08/integrated/reproduce_sources.py` | `c394868cc0bca7b5161034c010fa292833d402ae` | 0 |
| prior-validators | `python3 -B /home/osso/.worktrees/wow-ui-sim-p541-page/data/patch-api/evidence/5.4.1-session-2026-10-08/integrated/check_prior.py` | `c394868cc0bca7b5161034c010fa292833d402ae` | 0 |
| checks | `python3 -B /home/osso/.worktrees/wow-ui-sim-p541-page/data/patch-api/evidence/5.4.1-session-2026-10-08/integrated/run_checks.py` | `c394868cc0bca7b5161034c010fa292833d402ae` | 0 |
| master-all-sweeps | `cargo test --test prefork_full_ui -- publication_sweep` | `279a38f3d560e00e954d7dfb96e81f6be3668dd1` | 0 |
| format | `cargo fmt --check` | `c394868cc0bca7b5161034c010fa292833d402ae` | 0 |
| mists-check | `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | `c394868cc0bca7b5161034c010fa292833d402ae` | 0 |
| test_check_patch_validators | `python3 -B /home/osso/.worktrees/wow-ui-sim-p541-page/tools/test_check_patch_validators.py` | `c394868cc0bca7b5161034c010fa292833d402ae` | 0 |
| test_extract_patch_non_inventory | `python3 -B /home/osso/.worktrees/wow-ui-sim-p541-page/tools/test_extract_patch_non_inventory.py` | `c394868cc0bca7b5161034c010fa292833d402ae` | 0 |
| test_gen_patch_wikitext_register | `python3 -B /home/osso/.worktrees/wow-ui-sim-p541-page/tools/test_gen_patch_wikitext_register.py` | `c394868cc0bca7b5161034c010fa292833d402ae` | 0 |
| test_patch_audit_validation | `python3 -B /home/osso/.worktrees/wow-ui-sim-p541-page/tools/test_patch_audit_validation.py` | `c394868cc0bca7b5161034c010fa292833d402ae` | 0 |
| test_patch_warlords_register | `python3 -B /home/osso/.worktrees/wow-ui-sim-p541-page/tools/test_patch_warlords_register.py` | `c394868cc0bca7b5161034c010fa292833d402ae` | 0 |
| test_patch_mists_register | `python3 -B /home/osso/.worktrees/wow-ui-sim-p541-page/tools/test_patch_mists_register.py` | `c394868cc0bca7b5161034c010fa292833d402ae` | 0 |
| mists-all-sweeps | `cargo test --no-default-features --features sound,gui,casc,client-mists --test integration -- publication_sweep` | `d333bbe8a09d69392dbbdad4c87f0329fecee029` | 0 |
| master-mists-all-sweeps | `cargo test --no-default-features --features sound,gui,casc,client-mists --test integration -- publication_sweep` | `279a38f3d560e00e954d7dfb96e81f6be3668dd1` | 0 |
| historical-replay | `python3 -B /home/osso/.worktrees/wow-ui-sim-p541-page/data/patch-api/evidence/5.4.1-session-2026-10-08/integrated/replay_history.py` | `d856059c2e5fa2ef234a9b0599227fc772aada90` | 0 |
| history-without-original-objects | `python3 -B /home/osso/.worktrees/wow-ui-sim-p541-page/data/patch-api/evidence/5.4.1-session-2026-10-08/integrated/check_history_clone.py` | `29376f6d3c721faf372577e27a9ef1875468d4a2` | 0 |
| preserve-history | `python3 -B /home/osso/.worktrees/wow-ui-sim-p541-page/data/patch-api/evidence/5.4.1-session-2026-10-08/integrated/preserve_history.py` | `5767b8089714760a8292d58794f2602e07864e15` | 0 |
| mists-routing-correction | `python3 -B /home/osso/.worktrees/wow-ui-sim-p541-page/data/patch-api/evidence/5.4.1-session-2026-10-08/integrated/rerun_mists_sweeps.py` | `d333bbe8a09d69392dbbdad4c87f0329fecee029` | 0 |

Full environment, duration, log digest and revision are in each `.proof.json`.
Integration `patch_5_4_1` selects zero cases; prefork owns both 5.4.1 tests. No integration behavior claim.
Separate Mists integration publication sweeps cover the 5.5.2/5.5.3/5.5.4 empty inventories and SharedXML startup only, not full UI or API behavior.
Negative control must fail with exactly one added gap (2 → 3). Three inherited extract failures are unchanged.
No runtime, vendor or retirement changes; generator is byte-identical to master; extractor adds only opt-in lowercase-reflist. No startup comparison or full integration-suite claim.
Initial Classic outputs collided with the synthetic client-line control because output discovery selected its first out_env. Only the two Classic commands were rerun after selecting the unique P<number>_SWEEP_OUT; control output now has its own file. Original invalid receipts/results and context are archived, with attribution in routing-fix-invalidated.json. Original checks driver exit remains an execution receipt, not proof of the invalid Classic artifact association.
Later evidence/wiki/validator-only commits do not invalidate the pinned Rust, shared-tool or source scopes.
