# Integrated 5.4.8 command/proof ledger

Runtime `b77d0b5db0f186b75cee854b285d82e896a8a765`; master `a9d7c9566fcb33b272a193a7aae6f51d4db4973f`.
All commands run from the owned p548-page worktree; master uses immutable Git archives.
Targets: p548-page for branch, master-ref for master. No later source/test/tool changes invalidate these receipts.
Rejected multi-filter prefork invocations have no coverage; single-filter receipts supersede them.
Mists prefork is unavailable (requires client-retail); its rejection logs are not passing behavior evidence.

| Receipt | Command | Exit | Result |
|---|---|---:|---|
| all-sweeps | `cargo test --test prefork_full_ui -- publication_sweep` | 0 | test result: ok. 56 passed; 0 failed; 56 total |
| branch-build | `cargo test --test integration --test prefork_full_ui --no-run` | 0 | see sealed log |
| branch-checks | `python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/5.4.8-session-2026-10-08/integrated/run_checks.py` | 0 | see sealed log |
| branch-startup | `timeout 90 /home/osso/.cache/wow-ui-sim-targets/p548-page/debug/wow-sim --no-saved-vars lua-errors` | 0 | [] |
| format | `cargo fmt --check` | 0 | see sealed log |
| integration-list | `cargo test --test integration -- --list` | 0 | see sealed log |
| integration-patch_5_4_8 | `cargo test --test integration -- patch_5_4_8` | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10677 filtered out; finished in 0.16s |
| integration-regressions | `cargo test --test integration -- settings cvar edit_mode keybind nameplate action_bar chat combat taint secure ui_visibility screenshot hide_ui hide_user_interface patch_5_4_8` | 101 | test result: FAILED. 1076 passed; 3 failed; 3 ignored; 0 measured; 9596 filtered out; finished in 121.79s |
| lib-checks | `python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/5.4.8-session-2026-10-08/integrated/run_lib.py` | 0 | see sealed log |
| lib-mists-list | `cargo test --no-default-features --features sound,gui,casc,client-mists --lib -- --list` | 0 | see sealed log |
| lib-mists-regressions | `cargo test --no-default-features --features sound,gui,casc,client-mists --lib -- cvar set_cvar ui_visibility taint secure settings` | 0 | test result: ok. 64 passed; 0 failed; 0 ignored; 0 measured; 1766 filtered out; finished in 2.82s |
| lib-retail-list | `cargo test --lib -- --list` | 0 | see sealed log |
| lib-retail-regressions | `cargo test --lib -- cvar set_cvar ui_visibility taint secure settings` | 0 | test result: ok. 68 passed; 0 failed; 0 ignored; 0 measured; 1911 filtered out; finished in 3.17s |
| master-all-sweeps | `cargo test --manifest-path /tmp/p548-master-06uf2fon/Cargo.toml --test prefork_full_ui -- publication_sweep` | 0 | test result: ok. 55 passed; 0 failed; 55 total |
| master-checks | `python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/5.4.8-session-2026-10-08/integrated/run_master.py` | 1 | see sealed log |
| master-integration-list | `cargo test --manifest-path /tmp/p548-master-06uf2fon/Cargo.toml --test integration -- --list` | 0 | see sealed log |
| master-integration-regressions | `cargo test --manifest-path /tmp/p548-master-06uf2fon/Cargo.toml --test integration -- settings cvar edit_mode keybind nameplate action_bar chat combat taint secure ui_visibility screenshot hide_ui hide_user_interface patch_5_4_8` | 101 | test result: FAILED. 1075 passed; 3 failed; 3 ignored; 0 measured; 9596 filtered out; finished in 325.44s |
| master-lib-mists-list | `cargo test --manifest-path /tmp/p548-master-mists-mj527w7y/Cargo.toml --no-default-features --features sound,gui,casc,client-mists --lib -- --list` | 0 | see sealed log |
| master-lib-mists-regressions | `cargo test --manifest-path /tmp/p548-master-mists-mj527w7y/Cargo.toml --no-default-features --features sound,gui,casc,client-mists --lib -- cvar set_cvar ui_visibility taint secure settings` | 0 | test result: ok. 64 passed; 0 failed; 0 ignored; 0 measured; 1766 filtered out; finished in 0.96s |
| master-lib-retail-list | `cargo test --manifest-path /tmp/p548-master-mists-mj527w7y/Cargo.toml --lib -- --list` | 0 | see sealed log |
| master-lib-retail-regressions | `cargo test --manifest-path /tmp/p548-master-mists-mj527w7y/Cargo.toml --lib -- cvar set_cvar ui_visibility taint secure settings` | 0 | test result: ok. 68 passed; 0 failed; 0 ignored; 0 measured; 1911 filtered out; finished in 1.22s |
| master-missing-checks | `python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/5.4.8-session-2026-10-08/integrated/run_master_mists.py` | 0 | see sealed log |
| master-mists-integration-regressions | `cargo test --manifest-path /tmp/p548-master-mists-mj527w7y/Cargo.toml --no-default-features --features sound,gui,casc,client-mists --test integration -- cvar taint` | 101 | test result: FAILED. 61 passed; 7 failed; 0 ignored; 0 measured; 8108 filtered out; finished in 1.75s |
| master-mists-prefork-cvar | `cargo test --manifest-path /tmp/p548-master-prefork-b5qw3d92/Cargo.toml --no-default-features --features sound,gui,casc,client-mists --test prefork_full_ui -- cvar` | 101 | see sealed log |
| master-mists-prefork-taint | `cargo test --manifest-path /tmp/p548-master-prefork-b5qw3d92/Cargo.toml --no-default-features --features sound,gui,casc,client-mists --test prefork_full_ui -- taint` | 101 | see sealed log |
| master-prefork-action_bar | `cargo test --manifest-path /tmp/p548-master-prefork-b5qw3d92/Cargo.toml --test prefork_full_ui -- action_bar` | 0 | test result: ok. 13 passed; 0 failed; 13 total |
| master-prefork-chat | `cargo test --manifest-path /tmp/p548-master-prefork-b5qw3d92/Cargo.toml --test prefork_full_ui -- chat` | 0 | test result: ok. 31 passed; 0 failed; 31 total |
| master-prefork-checks | `python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/5.4.8-session-2026-10-08/integrated/run_prefork.py master` | 0 | see sealed log |
| master-prefork-combat | `cargo test --manifest-path /tmp/p548-master-prefork-b5qw3d92/Cargo.toml --test prefork_full_ui -- combat` | 0 | test result: ok. 44 passed; 0 failed; 44 total |
| master-prefork-cvar | `cargo test --manifest-path /tmp/p548-master-prefork-b5qw3d92/Cargo.toml --test prefork_full_ui -- cvar` | 0 | test result: ok. 26 passed; 0 failed; 26 total |
| master-prefork-edit_mode | `cargo test --manifest-path /tmp/p548-master-prefork-b5qw3d92/Cargo.toml --test prefork_full_ui -- edit_mode` | 0 | test result: ok. 10 passed; 0 failed; 10 total |
| master-prefork-hide_ui | `cargo test --manifest-path /tmp/p548-master-prefork-b5qw3d92/Cargo.toml --test prefork_full_ui -- hide_ui` | 0 | test result: ok. 1 passed; 0 failed; 1 total |
| master-prefork-hide_user_interface | `cargo test --manifest-path /tmp/p548-master-prefork-b5qw3d92/Cargo.toml --test prefork_full_ui -- hide_user_interface` | 0 | test result: ok. 0 passed; 0 failed; 0 total |
| master-prefork-keybind | `cargo test --manifest-path /tmp/p548-master-prefork-b5qw3d92/Cargo.toml --test prefork_full_ui -- keybind` | 0 | test result: ok. 17 passed; 0 failed; 17 total |
| master-prefork-nameplate | `cargo test --manifest-path /tmp/p548-master-prefork-b5qw3d92/Cargo.toml --test prefork_full_ui -- nameplate` | 0 | test result: ok. 7 passed; 0 failed; 7 total |
| master-prefork-patch_5_4_8 | `cargo test --manifest-path /tmp/p548-master-prefork-b5qw3d92/Cargo.toml --test prefork_full_ui -- patch_5_4_8` | 0 | test result: ok. 0 passed; 0 failed; 0 total |
| master-prefork-screenshot | `cargo test --manifest-path /tmp/p548-master-prefork-b5qw3d92/Cargo.toml --test prefork_full_ui -- screenshot` | 0 | test result: ok. 2 passed; 0 failed; 2 total |
| master-prefork-secure | `cargo test --manifest-path /tmp/p548-master-prefork-b5qw3d92/Cargo.toml --test prefork_full_ui -- secure` | 0 | test result: ok. 60 passed; 0 failed; 60 total |
| master-prefork-settings | `cargo test --manifest-path /tmp/p548-master-prefork-b5qw3d92/Cargo.toml --test prefork_full_ui -- settings` | 0 | test result: ok. 44 passed; 0 failed; 44 total |
| master-prefork-taint | `cargo test --manifest-path /tmp/p548-master-prefork-b5qw3d92/Cargo.toml --test prefork_full_ui -- taint` | 0 | test result: ok. 4 passed; 0 failed; 4 total |
| master-prefork-ui_visibility | `cargo test --manifest-path /tmp/p548-master-prefork-b5qw3d92/Cargo.toml --test prefork_full_ui -- ui_visibility` | 0 | test result: ok. 0 passed; 0 failed; 0 total |
| master-prefork_full_ui-list | `cargo test --manifest-path /tmp/p548-master-06uf2fon/Cargo.toml --test prefork_full_ui -- --list` | 0 | see sealed log |
| master-prefork_full_ui-regressions | `cargo test --manifest-path /tmp/p548-master-06uf2fon/Cargo.toml --test prefork_full_ui -- settings cvar edit_mode keybind nameplate action_bar chat combat taint secure ui_visibility screenshot hide_ui hide_user_interface patch_5_4_8` | 1 | see sealed log |
| master-retail-build | `cargo build --manifest-path /tmp/p548-master-06uf2fon/Cargo.toml --bin wow-sim` | 0 | see sealed log |
| master-startup | `timeout 90 /home/osso/.cache/wow-ui-sim-targets/master-ref/debug/wow-sim --no-saved-vars lua-errors` | 0 | [] |
| mists-check | `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | 0 | see sealed log |
| mists-integration-list | `cargo test --no-default-features --features sound,gui,casc,client-mists --test integration -- --list` | 0 | see sealed log |
| mists-integration-regressions | `cargo test --no-default-features --features sound,gui,casc,client-mists --test integration -- cvar taint` | 101 | test result: FAILED. 61 passed; 7 failed; 0 ignored; 0 measured; 8108 filtered out; finished in 5.61s |
| mists-prefork-cvar | `cargo test --no-default-features --features sound,gui,casc,client-mists --test prefork_full_ui -- cvar` | 101 | see sealed log |
| mists-prefork-taint | `cargo test --no-default-features --features sound,gui,casc,client-mists --test prefork_full_ui -- taint` | 101 | see sealed log |
| mists-prefork_full_ui-list | `cargo test --no-default-features --features sound,gui,casc,client-mists --test prefork_full_ui -- --list` | 101 | see sealed log |
| mists-prefork_full_ui-regressions | `cargo test --no-default-features --features sound,gui,casc,client-mists --test prefork_full_ui -- cvar taint` | 101 | see sealed log |
| negative | `cargo test --test prefork_full_ui -- patch_5_4_8_publication_sweep` | 1 | test result: FAILED. 0 passed; 1 failed; 1 total |
| own-sweep | `cargo test --test prefork_full_ui -- patch_5_4_8_publication_sweep` | 0 | test result: ok. 1 passed; 0 failed; 1 total |
| prefork-action_bar | `cargo test --test prefork_full_ui -- action_bar` | 0 | test result: ok. 13 passed; 0 failed; 13 total |
| prefork-chat | `cargo test --test prefork_full_ui -- chat` | 0 | test result: ok. 31 passed; 0 failed; 31 total |
| prefork-checks | `python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/5.4.8-session-2026-10-08/integrated/run_prefork.py` | 0 | see sealed log |
| prefork-combat | `cargo test --test prefork_full_ui -- combat` | 0 | test result: ok. 45 passed; 0 failed; 45 total |
| prefork-cvar | `cargo test --test prefork_full_ui -- cvar` | 0 | test result: ok. 26 passed; 0 failed; 26 total |
| prefork-edit_mode | `cargo test --test prefork_full_ui -- edit_mode` | 0 | test result: ok. 10 passed; 0 failed; 10 total |
| prefork-hide_ui | `cargo test --test prefork_full_ui -- hide_ui` | 0 | test result: ok. 1 passed; 0 failed; 1 total |
| prefork-hide_user_interface | `cargo test --test prefork_full_ui -- hide_user_interface` | 0 | test result: ok. 0 passed; 0 failed; 0 total |
| prefork-keybind | `cargo test --test prefork_full_ui -- keybind` | 0 | test result: ok. 17 passed; 0 failed; 17 total |
| prefork-nameplate | `cargo test --test prefork_full_ui -- nameplate` | 0 | test result: ok. 7 passed; 0 failed; 7 total |
| prefork-patch_5_4_8 | `cargo test --test prefork_full_ui -- patch_5_4_8` | 0 | test result: ok. 2 passed; 0 failed; 2 total |
| prefork-screenshot | `cargo test --test prefork_full_ui -- screenshot` | 0 | test result: ok. 2 passed; 0 failed; 2 total |
| prefork-secure | `cargo test --test prefork_full_ui -- secure` | 0 | test result: ok. 60 passed; 0 failed; 60 total |
| prefork-settings | `cargo test --test prefork_full_ui -- settings` | 0 | test result: ok. 44 passed; 0 failed; 44 total |
| prefork-taint | `cargo test --test prefork_full_ui -- taint` | 0 | test result: ok. 4 passed; 0 failed; 4 total |
| prefork-ui_visibility | `cargo test --test prefork_full_ui -- ui_visibility` | 0 | test result: ok. 0 passed; 0 failed; 0 total |
| prefork_full_ui-list | `cargo test --test prefork_full_ui -- --list` | 0 | see sealed log |
| prefork_full_ui-patch_5_4_8 | `cargo test --test prefork_full_ui -- patch_5_4_8` | 0 | test result: ok. 2 passed; 0 failed; 2 total |
| prefork_full_ui-regressions | `cargo test --test prefork_full_ui -- settings cvar edit_mode keybind nameplate action_bar chat combat taint secure ui_visibility screenshot hide_ui hide_user_interface patch_5_4_8` | 1 | see sealed log |
| prior-0 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/10.0.0-session-2026-10-07/validate.py` | 0 | see sealed log |
| prior-1 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/10.0.2-session-2026-10-07/validate.py` | 0 | see sealed log |
| prior-10 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/6.2.2-session-2026-10-08/validate.py` | 0 | see sealed log |
| prior-11 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/6.2.4-session-2026-10-08/integrated/validate.py` | 0 | see sealed log |
| prior-12 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/6.2.4-session-2026-10-08/validate.py` | 0 | see sealed log |
| prior-13 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/7.0.1-session-2026-10-08/validate.py` | 0 | see sealed log |
| prior-14 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/7.0.3-session-2026-10-08/integrated/validate.py` | 0 | see sealed log |
| prior-15 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/7.0.3-session-2026-10-08/validate.py` | 0 | see sealed log |
| prior-16 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/7.1.0-session-2026-10-08/validate.py` | 0 | see sealed log |
| prior-17 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/7.2.0-session-2026-10-08/validate.py` | 0 | see sealed log |
| prior-18 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/7.2.5-session-2026-10-08/validate.py` | 0 | see sealed log |
| prior-19 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/7.3.0-session-2026-10-08/validate.py` | 0 | see sealed log |
| prior-2 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/10.0.5-session-2026-10-07/validate.py` | 0 | see sealed log |
| prior-20 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/7.3.2-session-2026-10-08/validate.py` | 0 | see sealed log |
| prior-21 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/8.0.1-session-2026-10-08/validate.py` | 0 | see sealed log |
| prior-22 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/8.1.0-session-2026-10-08/validate.py` | 0 | see sealed log |
| prior-23 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/8.1.5-session-2026-10-08/validate.py` | 0 | see sealed log |
| prior-24 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/8.2.0-session-2026-10-08/validate.py` | 0 | see sealed log |
| prior-25 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/8.2.5-session-2026-10-08/validate.py` | 0 | see sealed log |
| prior-26 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/8.3.0-session-2026-10-08/validate.py` | 0 | see sealed log |
| prior-27 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/8.3.7-session-2026-10-08/validate.py` | 0 | see sealed log |
| prior-28 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/9.0.1-session-2026-10-08/validate.py` | 0 | see sealed log |
| prior-29 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/9.0.2-session-2026-10-08/validate.py` | 0 | see sealed log |
| prior-3 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/6.0.1-session-2026-10-08/validate.py` | 0 | see sealed log |
| prior-30 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/9.0.5-session-2026-10-08/validate.py` | 0 | see sealed log |
| prior-31 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/9.1.0-session-2026-10-07/validate.py` | 0 | see sealed log |
| prior-32 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/9.1.5-session-2026-10-07/validate.py` | 0 | see sealed log |
| prior-33 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/9.2.0-session-2026-10-07/validate.py` | 0 | see sealed log |
| prior-34 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/9.2.5-session-2026-10-07/validate.py` | 0 | see sealed log |
| prior-35 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/9.2.7-session-2026-10-07/validate.py` | 0 | see sealed log |
| prior-4 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/6.0.2-session-2026-10-08/integrated/validate.py` | 0 | see sealed log |
| prior-5 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/6.0.2-session-2026-10-08/validate.py` | 0 | see sealed log |
| prior-6 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/6.1.0-session-2026-10-08/integrated/validate.py` | 0 | see sealed log |
| prior-7 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/6.1.0-session-2026-10-08/validate.py` | 0 | see sealed log |
| prior-8 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/6.2.0-session-2026-10-08/integrated/validate.py` | 0 | see sealed log |
| prior-9 | `/usr/bin/python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/6.2.0-session-2026-10-08/validate.py` | 0 | see sealed log |
| prior-validators | `python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/data/patch-api/evidence/5.4.8-session-2026-10-08/integrated/check_prior.py` | 0 | see sealed log |
| retail-build | `cargo build --bin wow-sim` | 0 | see sealed log |
| test_check_patch_validators | `python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/tools/test_check_patch_validators.py` | 0 | Ran 4 tests in 0.392s |
| test_extract_patch_non_inventory | `python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/tools/test_extract_patch_non_inventory.py` | 0 | Ran 36 tests in 0.042s |
| test_gen_patch_wikitext_register | `python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/tools/test_gen_patch_wikitext_register.py` | 0 | Ran 34 tests in 0.003s |
| test_patch_audit_validation | `python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/tools/test_patch_audit_validation.py` | 0 | Ran 8 tests in 0.202s |
| test_patch_warlords_register | `python3 -B /home/osso/.worktrees/wow-ui-sim-p548-page/tools/test_patch_warlords_register.py` | 0 | Ran 3 tests in 0.001s |
