# Proof ledger

| Scope | Command | Revision | Result | Later invalidation |
|---|---|---|---|---|
| p548-all-sweeps | `cargo test --test prefork_full_ui -- publication_sweep` | `d67ecb8321a7816d3d5846bd3bdcc32a9af01e03` | exit 0; sealed log | None in proof scope |
| p548-cached-combat | `cargo test --test prefork_full_ui -- patch_5_4_8_cached_combat_restrictions` | `d67ecb8321a7816d3d5846bd3bdcc32a9af01e03` | exit 0; sealed log | None in proof scope |
| p548-prefork-cvar | `cargo test --test prefork_full_ui -- cvar` | `d67ecb8321a7816d3d5846bd3bdcc32a9af01e03` | exit 0; sealed log | None in proof scope |
| p548-prefork-world_map | `cargo test --test prefork_full_ui -- world_map` | `d67ecb8321a7816d3d5846bd3bdcc32a9af01e03` | exit 0; sealed log | None in proof scope |
| p548-prefork-keybindings | `cargo test --test prefork_full_ui -- keybindings` | `d67ecb8321a7816d3d5846bd3bdcc32a9af01e03` | exit 0; sealed log | None in proof scope |
| p548-integration-patch_5_4_8 | `cargo test --test integration patch_5_4_8` | `d67ecb8321a7816d3d5846bd3bdcc32a9af01e03` | exit 0; sealed log | None in proof scope |
| p548-integration-set_cvar_global | `cargo test --test integration set_cvar_global` | `d67ecb8321a7816d3d5846bd3bdcc32a9af01e03` | exit 0; sealed log | None in proof scope |
| p548-integration-cvar_bitfields | `cargo test --test integration cvar_bitfields` | `d67ecb8321a7816d3d5846bd3bdcc32a9af01e03` | exit 0; sealed log | None in proof scope |
| p548-integration-test_cvar_display_settings | `cargo test --test integration test_cvar_display_settings` | `d67ecb8321a7816d3d5846bd3bdcc32a9af01e03` | exit 0; sealed log | None in proof scope |
| p548-integration-ui_visibility_globals | `cargo test --test integration ui_visibility_globals` | `d67ecb8321a7816d3d5846bd3bdcc32a9af01e03` | exit 0; sealed log | None in proof scope |
| p548-branch-startup-driver | `python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/5.4.8-session-2026-10-08/startup_compare.py branch` | `d67ecb8321a7816d3d5846bd3bdcc32a9af01e03` | exit 0; sealed log | None in proof scope |
| p548-branch-startup-build | `cargo build --bin wow-sim` | `d67ecb8321a7816d3d5846bd3bdcc32a9af01e03` | exit 0; sealed log | None in proof scope |
| p548-format | `cargo fmt --check` | `d67ecb8321a7816d3d5846bd3bdcc32a9af01e03` | exit 0; sealed log | None in proof scope |
| p548-mists-check | `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | `d67ecb8321a7816d3d5846bd3bdcc32a9af01e03` | exit 0; sealed log | None in proof scope |
| p548-reproduction-final | `python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/5.4.8-session-2026-10-08/reproduce_sources.py` | `d67ecb8321a7816d3d5846bd3bdcc32a9af01e03` | exit 0; sealed log | None in proof scope |
| p548-test_check_patch_validators | `python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/tools/test_check_patch_validators.py` | `48860e1f80b787dadc785937038a73d28d946bf5` | exit 0; sealed log | None in proof scope |
| p548-test_extract_patch_non_inventory | `python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/tools/test_extract_patch_non_inventory.py` | `48860e1f80b787dadc785937038a73d28d946bf5` | exit 0; sealed log | None in proof scope |
| p548-test_gen_patch_wikitext_register | `python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/tools/test_gen_patch_wikitext_register.py` | `48860e1f80b787dadc785937038a73d28d946bf5` | exit 0; sealed log | None in proof scope |
| p548-test_patch_audit_validation | `python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/tools/test_patch_audit_validation.py` | `48860e1f80b787dadc785937038a73d28d946bf5` | exit 0; sealed log | None in proof scope |

Expected failures: initial combat RED; map-to-sequence fixture correction; discovery seven gaps; negative control seven → eight.
Initial discovery outputs are development evidence, not acceptance after test/fixture changes.
Final reproduction supersedes the provisional run performed before its recipe was committed.
Python fixture source stayed unchanged since its recorded revision; Rust checks cover the committed accounting revision.
Master startup executed before runtime changes; its src/crates/Cargo scope equals pinned master exactly.
No full integration suite, agent/model CLI, push or merge. No live external-cache input to validation.
Scan tool: /usr/bin/grep, whole-word -w, untruncated output files.
