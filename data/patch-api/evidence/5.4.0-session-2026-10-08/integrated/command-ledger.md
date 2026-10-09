# 5.4.0 integrated command ledger

Base: `3c60ac0ea`. Own source/test scope: `af7edd244708325931e492a0bfb1c33606efaefa`.
All Cargo commands use `CARGO_TARGET_DIR=/home/osso/.cache/wow-ui-sim-targets/p540-page`.
Cargo commands ran sequentially in logged asynchronous drivers; prior validators and source reproduction ran independently. No poll-wait or repeated broad checks.

| Receipt | Command | Revision | Exit |
|---|---|---|---|
| own-sweep | `cargo test --test prefork_full_ui -- patch_5_4_0_publication_sweep` | `af7edd244708325931e492a0bfb1c33606efaefa` | 0 |
| all-sweeps | `cargo test --test prefork_full_ui -- publication_sweep` | `af7edd244708325931e492a0bfb1c33606efaefa` | 0 |
| prefork-patch_5_4_0 | `cargo test --test prefork_full_ui -- patch_5_4_0` | `af7edd244708325931e492a0bfb1c33606efaefa` | 0 |
| integration-patch_5_4_0 | `cargo test --test integration -- patch_5_4_0` | `af7edd244708325931e492a0bfb1c33606efaefa` | 0 |
| negative | `cargo test --test prefork_full_ui -- patch_5_4_0_publication_sweep` | `af7edd244708325931e492a0bfb1c33606efaefa` | 1 |
| reproduction | `python3 -B /home/osso/.worktrees/wow-ui-sim-p540-page/data/patch-api/evidence/5.4.0-session-2026-10-08/integrated/reproduce_sources.py` | `589116f712ad15d13f0bb4129cdf6f7f576d6447` | 0 |
| prior-validators | `python3 -B /home/osso/.worktrees/wow-ui-sim-p540-page/data/patch-api/evidence/5.4.0-session-2026-10-08/integrated/check_prior.py` | `589116f712ad15d13f0bb4129cdf6f7f576d6447` | 0 |
| checks | `python3 -B /home/osso/.worktrees/wow-ui-sim-p540-page/data/patch-api/evidence/5.4.0-session-2026-10-08/integrated/run_checks.py` | `af7edd244708325931e492a0bfb1c33606efaefa` | 0 |
| master-all-sweeps | `cargo test --test prefork_full_ui -- publication_sweep` | `3c60ac0ea839d9be4e864d82eb6c53ee3b3d7702` | 0 |
| format | `cargo fmt --check` | `af7edd244708325931e492a0bfb1c33606efaefa` | 0 |
| mists-check | `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | `af7edd244708325931e492a0bfb1c33606efaefa` | 0 |
| test_check_patch_validators | `python3 -B /home/osso/.worktrees/wow-ui-sim-p540-page/tools/test_check_patch_validators.py` | `af7edd244708325931e492a0bfb1c33606efaefa` | 0 |
| test_extract_patch_non_inventory | `python3 -B /home/osso/.worktrees/wow-ui-sim-p540-page/tools/test_extract_patch_non_inventory.py` | `af7edd244708325931e492a0bfb1c33606efaefa` | 0 |
| test_gen_patch_wikitext_register | `python3 -B /home/osso/.worktrees/wow-ui-sim-p540-page/tools/test_gen_patch_wikitext_register.py` | `af7edd244708325931e492a0bfb1c33606efaefa` | 0 |
| test_patch_audit_validation | `python3 -B /home/osso/.worktrees/wow-ui-sim-p540-page/tools/test_patch_audit_validation.py` | `af7edd244708325931e492a0bfb1c33606efaefa` | 0 |
| test_patch_warlords_register | `python3 -B /home/osso/.worktrees/wow-ui-sim-p540-page/tools/test_patch_warlords_register.py` | `af7edd244708325931e492a0bfb1c33606efaefa` | 0 |
| test_patch_mists_register | `python3 -B /home/osso/.worktrees/wow-ui-sim-p540-page/tools/test_patch_mists_register.py` | `af7edd244708325931e492a0bfb1c33606efaefa` | 0 |
| test_patch_mists_extract | `python3 -B /home/osso/.worktrees/wow-ui-sim-p540-page/tools/test_patch_mists_extract.py` | `af7edd244708325931e492a0bfb1c33606efaefa` | 0 |
| mists-all-sweeps | `cargo test --no-default-features --features sound,gui,casc,client-mists --test integration -- publication_sweep` | `0cafe27a504f0bd81624601696cb9f00c9747781` | 0 |
| master-mists-all-sweeps | `cargo test --no-default-features --features sound,gui,casc,client-mists --test integration -- publication_sweep` | `3c60ac0ea839d9be4e864d82eb6c53ee3b3d7702` | 0 |
| historical-replay | `python3 -B /home/osso/.worktrees/wow-ui-sim-p540-page/data/patch-api/evidence/5.4.0-session-2026-10-08/integrated/replay_history.py` | `589116f712ad15d13f0bb4129cdf6f7f576d6447` | 0 |
| preserve-history | `python3 -B /home/osso/.worktrees/wow-ui-sim-p540-page/data/patch-api/evidence/5.4.0-session-2026-10-08/integrated/preserve_history.py` | `589116f712ad15d13f0bb4129cdf6f7f576d6447` | 0 |

Full environment, duration, log digest and revision are in each `.proof.json`.
Negative control must fail with exactly one added gap (21 → 22). Three inherited extract failures remain unchanged.
RED own-sweep at 589116f71 is retained under supersession-red-*; its exact 5.4.2 securerandom removal changes only one gap.
Prior validators use the complete git ls-tree set at master 3c60ac0ea. Shared files are pinned; comparisons and preserved historical bytes stay in this session.
Reproduction at 589116f71 covers unchanged raw/provenance/register/text/tool inputs; the later ledger-only supersession edit does not invalidate that proof.
No runtime/vendor changes. Conditional runtime/lib-master/startup comparisons do not apply.
Later evidence/wiki/validator-only commits do not invalidate the pinned Rust/tool/source proof.
