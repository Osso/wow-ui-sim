# 7.3.2 integrated command ledger

Runtime scope: `e5ed4121d6fa3964e7af3e664b7e4f6f86e7a3de`; rebase base `af7a101e2`, real 8.0.1 supersession register `ad188f494`.
Worktree: `/home/osso/.worktrees/wow-ui-sim-p732-page`. Target: `/home/osso/.cache/wow-ui-sim-targets/p732-page`. No agents, push, merge, cwd mutation, vendor edits or bytecode caches. Long Cargo commands logged asynchronously. Later evidence/wiki-only commits do not invalidate source scopes.

| Receipt | Revision | Command | Exit | Result |
|---|---|---|---|---|
| p732-integration-behavior.proof.json | e5ed4121d | `cargo test --test integration -- patch_7_3_2` | 0 | Finished `test` profile [optimized + debuginfo] target(s) in 22.15s; test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 10665 filtered out; finished in 0.15s |
| p732-integration-cached.proof.json | e5ed4121d | `cargo test --test prefork_full_ui -- patch_7_3_2` | 0 | Finished `test` profile [optimized + debuginfo] target(s) in 8.52s; test result: ok. 4 passed; 0 failed; 4 total |
| p732-integration-key-dispatch.proof.json | e5ed4121d | `cargo test --test integration -- key_dispatch` | 0 | Finished `test` profile [optimized + debuginfo] target(s) in 0.13s; test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 10638 filtered out; finished in 0.32s |
| p732-integration-menu-bare.proof.json | e5ed4121d | `cargo test --test integration -- game_menu` | 0 | Finished `test` profile [optimized + debuginfo] target(s) in 0.13s; test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 10643 filtered out; finished in 7.30s |
| p732-integration-menu-cached.proof.json | e5ed4121d | `cargo test --test prefork_full_ui -- game_menu` | 0 | Finished `test` profile [optimized + debuginfo] target(s) in 0.13s; test result: ok. 14 passed; 0 failed; 14 total |
| p732-integration-popup-bare.proof.json | e5ed4121d | `cargo test --test integration -- blizzard_static_popup` | 0 | Finished `test` profile [optimized + debuginfo] target(s) in 0.13s; test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 10627 filtered out; finished in 3.39s |
| p732-integration-popup-cached.proof.json | e5ed4121d | `cargo test --test prefork_full_ui -- blizzard_static_popup` | 0 | Finished `test` profile [optimized + debuginfo] target(s) in 0.13s; test result: ok. 33 passed; 0 failed; 33 total |
| p732-integration-all-sweeps.proof.json | e5ed4121d | `cargo test --test prefork_full_ui -- publication_sweep` | 0 | Finished `test` profile [optimized + debuginfo] target(s) in 0.13s; test result: ok. 43 passed; 0 failed; 43 total |
| p732-integration-negative.proof.json | e5ed4121d | `cargo test --test prefork_full_ui -- patch_7_3_2_publication_sweep` | 1 | Finished `test` profile [optimized + debuginfo] target(s) in 0.13s; test result: FAILED. 0 passed; 1 failed; 1 total |
| p732-integration-format.proof.json | e5ed4121d | `cargo fmt --check` | 0 | exit 0 |
| p732-integration-mists-session.proof.json | e5ed4121d | `cargo test --no-default-features --features sound,gui,casc,client-mists --test integration -- patch_7_3_2` | 0 | Finished `test` profile [optimized + debuginfo] target(s) in 1m 11s; test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8171 filtered out; finished in 0.16s |
| p732-integration-mists-check.proof.json | e5ed4121d | `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | 0 | Finished `dev` profile [optimized + debuginfo] target(s) in 27.17s |
| p732-integration-own.proof.json | ad188f494 | `cargo test --test prefork_full_ui -- patch_7_3_2_publication_sweep` | 0 | Finished `test` profile [optimized + debuginfo] target(s) in 1m 31s; test result: ok. 1 passed; 0 failed; 1 total |
| p732-integration-retail-build.proof.json | e5ed4121d | `cargo build --bin wow-sim` | 0 | Finished `dev` profile [optimized + debuginfo] target(s) in 0.13s |
| p732-integration-startup.proof.json | e5ed4121d | `timeout 90 /home/osso/.cache/wow-ui-sim-targets/p732-page/debug/wow-sim --no-addons --no-saved-vars lua-errors` | 0 | startup [] |
| p732-integration-gen_patch_wikitext_register-fixtures.proof.json | ad188f494 | `python3 -B /home/osso/.worktrees/wow-ui-sim-p732-page/tools/test_gen_patch_wikitext_register.py` | 0 | Ran 25 tests in 0.001s |
| p732-integration-extract_patch_non_inventory-fixtures.proof.json | ad188f494 | `python3 -B /home/osso/.worktrees/wow-ui-sim-p732-page/tools/test_extract_patch_non_inventory.py` | 0 | Ran 34 tests in 0.033s |
| p732-integration-patch_audit_validation-fixtures.proof.json | ad188f494 | `python3 -B /home/osso/.worktrees/wow-ui-sim-p732-page/tools/test_patch_audit_validation.py` | 0 | Ran 8 tests in 0.138s |
| p732-integration-source-reproduction.proof.json | ad188f494 | `python3 -B /home/osso/.worktrees/wow-ui-sim-p732-page/data/patch-api/evidence/7.3.2-session-2026-10-08/reproduce_integrated_sources.py` | 0 | 42 registers, 39/42 extracts; inherited 12.0.5/12.0.7/12.1.0 failures retained |
| p732-integration-extend-receipts.proof.json | e5ed4121d | `python3 -B /tmp/p732-extend-receipts.py` | 0 | 7.3.2: 2/2 OK; 8.0.1: 252/269 OK, 17 exact gaps; 42 registers / 39 extracts reproduced |

## Source and gap accounting

- Own publication: 0 → 0 gaps; both changed APIs remain present. No supersession closure or later-sweep gap change; no new `LATER_AUDIT_REPLACEMENTS` exception.
- All-sweeps: 42 page sweeps plus factory, 43/43 pass, 8,850 observations. Added 8.0.1 receipt: 269 rows, 252 OK, 17 exact existing gaps.
- Negative control: only Logout direction changed to removed. Expected exit 1, 0 → 1 gaps; one new gap, zero resolved. Original historical negative receipt retained.
- Registers: 42/42 byte-identical. Extracts: 39/42 byte-identical; three inherited failures remain unchanged. 8.0.1 `--bfa-prepatch` and 7.3.2 `--prose-api-links` retained.
- Original 209 input hashes/86 extraction-mode outcomes remain historical preservation evidence. Complete integrated register/sweep set pinned to runtime revision, not a moving glob or receipt-selected subset.

## Session caller coverage

Untruncated `/usr/bin/grep -rnE --exclude-dir=Wowless*` result: 109 matches (src 10, tests 12, cached Blizzard UI 87, bundled addons 0), exit 0 and empty stderr. All-profile cache retained, including documentation/context hits. No matching non-Wowless addon `run-tests` cases exist. Scan argv and complete stdout live in `p732-integration-session-callers.json`.

Retail callers include secure key Quit binding, GameMenu Logout/Quit callbacks, chat slash callbacks, inactive WoWLabs XML, class trial Logout and store `securecall("Logout")`; QUIT popup uses ForceQuit. General secure transitions and protection are exercised in integration; live menu/logout/ForceQuit/CAMP callbacks run after full cached startup. Key/menu/popup suite counts: 29, 24/14, 40/33. These tests do not claim reconstructed Legion, inactive-profile callback execution, or native countdown parity.

Supplemental slash diagnostic stopped at registration (secure stack true, mode 0, Standard 1, alias /logout exists, no callback). It never invoked Logout/Quit. Game rules/state/enum hashes match master; false default-registry precondition removed from the supplemental test. Diagnostic failure retained, unrelated default-mode bug not changed. No slash-command execution coverage claimed.

## Validator matrix

All 19/19 evidence validators pass at `996cce840a60369e36e8a253c312b8526a2e0bc0`. Full command/stdout/stderr/revision/exit rows: `p732-integration-validator-matrix.json`. Baseline matrix retained the own scope mismatch before refresh.

| Evidence directory | Exit | Result |
|---|---|---|
| `10.0.0-session-2026-10-07` | 0 | PASS |
| `10.0.2-session-2026-10-07` | 0 | PASS |
| `10.0.5-session-2026-10-07` | 0 | PASS |
| `7.3.2-session-2026-10-08` | 0 | PASS |
| `8.0.1-session-2026-10-08` | 0 | PASS |
| `8.1.0-session-2026-10-08` | 0 | PASS |
| `8.1.5-session-2026-10-08` | 0 | PASS |
| `8.2.0-session-2026-10-08` | 0 | PASS |
| `8.2.5-session-2026-10-08` | 0 | PASS |
| `8.3.0-session-2026-10-08` | 0 | PASS |
| `8.3.7-session-2026-10-08` | 0 | PASS |
| `9.0.1-session-2026-10-08` | 0 | PASS |
| `9.0.2-session-2026-10-08` | 0 | PASS |
| `9.0.5-session-2026-10-08` | 0 | PASS |
| `9.1.0-session-2026-10-07` | 0 | PASS |
| `9.1.5-session-2026-10-07` | 0 | PASS |
| `9.2.0-session-2026-10-07` | 0 | PASS |
| `9.2.5-session-2026-10-07` | 0 | PASS |
| `9.2.7-session-2026-10-07` | 0 | PASS |

## Required checks

Python fixtures: generator 25, extractor 34, shared validator 8. Format passes. Mists session preservation 1/1; requested Mists tests check exits 0 with zero non-vendor warnings (six inherited iced manifest warnings plus summary remain unsuppressed). Retail build exits 0; separate bounded startup exits 0 and prints `[]`.

Rust readability: `p732-integration-readability.md`. Current test/source scopes checked by `validate.py`; no redundant broad Cargo reruns after evidence/docs-only commits.
