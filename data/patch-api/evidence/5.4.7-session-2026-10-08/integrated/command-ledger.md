# 5.4.7 integrated command ledger

Base: `bebcc5830dcb21e768a2e3c365105351a400fccb`. Own source/test scope: `b14af5446ad916207e8d0a7c04469b658dfec64b`.
All Cargo commands use `CARGO_TARGET_DIR=/home/osso/.cache/wow-ui-sim-targets/p547-page`.
Commands ran sequentially in the logged asynchronous driver; no poll-wait or repeated broad checks.

| Receipt | Command | Revision | Exit |
|---|---|---|---|
| own-sweep | `cargo test --test prefork_full_ui -- patch_5_4_7_publication_sweep` | `b14af5446ad916207e8d0a7c04469b658dfec64b` | 0 |
| all-sweeps | `cargo test --test prefork_full_ui -- publication_sweep` | `b14af5446ad916207e8d0a7c04469b658dfec64b` | 0 |
| prefork-patch_5_4_7 | `cargo test --test prefork_full_ui -- patch_5_4_7` | `b14af5446ad916207e8d0a7c04469b658dfec64b` | 0 |
| integration-patch_5_4_7 | `cargo test --test integration -- patch_5_4_7` | `b14af5446ad916207e8d0a7c04469b658dfec64b` | 0 |
| negative | `cargo test --test prefork_full_ui -- patch_5_4_7_publication_sweep` | `b14af5446ad916207e8d0a7c04469b658dfec64b` | 1 |
| reproduction | `python3 -B /home/osso/.worktrees/wow-ui-sim-p547-page/data/patch-api/evidence/5.4.7-session-2026-10-08/integrated/reproduce_sources.py` | `b14af5446ad916207e8d0a7c04469b658dfec64b` | 0 |
| prior-validators | `python3 -B /home/osso/.worktrees/wow-ui-sim-p547-page/data/patch-api/evidence/5.4.7-session-2026-10-08/integrated/check_prior.py` | `43c108a28b6b80ec2922b7a9310d4c3693e8a3bf` | 0 |
| checks | `python3 -B /home/osso/.worktrees/wow-ui-sim-p547-page/data/patch-api/evidence/5.4.7-session-2026-10-08/integrated/run_checks.py` | `b14af5446ad916207e8d0a7c04469b658dfec64b` | 0 |
| master-all-sweeps | `cargo test --test prefork_full_ui -- publication_sweep` | `bebcc5830dcb21e768a2e3c365105351a400fccb` | 0 |
| format | `cargo fmt --check` | `b14af5446ad916207e8d0a7c04469b658dfec64b` | 0 |
| mists-check | `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | `b14af5446ad916207e8d0a7c04469b658dfec64b` | 0 |
| test_check_patch_validators | `python3 -B /home/osso/.worktrees/wow-ui-sim-p547-page/tools/test_check_patch_validators.py` | `b14af5446ad916207e8d0a7c04469b658dfec64b` | 0 |
| test_extract_patch_non_inventory | `python3 -B /home/osso/.worktrees/wow-ui-sim-p547-page/tools/test_extract_patch_non_inventory.py` | `b14af5446ad916207e8d0a7c04469b658dfec64b` | 0 |
| test_gen_patch_wikitext_register | `python3 -B /home/osso/.worktrees/wow-ui-sim-p547-page/tools/test_gen_patch_wikitext_register.py` | `b14af5446ad916207e8d0a7c04469b658dfec64b` | 0 |
| test_patch_audit_validation | `python3 -B /home/osso/.worktrees/wow-ui-sim-p547-page/tools/test_patch_audit_validation.py` | `b14af5446ad916207e8d0a7c04469b658dfec64b` | 0 |
| test_patch_warlords_register | `python3 -B /home/osso/.worktrees/wow-ui-sim-p547-page/tools/test_patch_warlords_register.py` | `b14af5446ad916207e8d0a7c04469b658dfec64b` | 0 |

Full environment, duration, log digest and revision are in each `.proof.json`.
Integration `patch_5_4_7` selects zero cases; prefork owns both 5.4.7 tests. No integration behavior claim.
Negative control must fail with exactly one added gap (2 → 3). Three inherited extract failures are unchanged.
No runtime, shared tool, vendor or retirement changes. No startup comparison or full integration-suite claim.
Later evidence/wiki/validator-only commits do not invalidate the pinned Rust, shared-tool or source scopes.
