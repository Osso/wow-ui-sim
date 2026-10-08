# 7.3.0 integrated command ledger

Runtime/register/sweep scope: `b13975944e9b8a6d3fd25a8eb78084ad7b2a1536`. Master source: `85c2acb2d875c7945ca9e517e7989468cf238f7c`. Worktree `/home/osso/.worktrees/wow-ui-sim-p730-page`; owned target `/home/osso/.cache/wow-ui-sim-targets/p730-page`; separate master target `/home/osso/.cache/wow-ui-sim-targets/master-ref`. Source scopes are unchanged by subsequent evidence/wiki/validator-only commits. Source revision and driver-checkout revision are distinct for master receipts. Original sealed evidence remains unchanged.

| Receipt | Source | Command | Exit | Result |
|---|---|---|---|---|
| integration-p730.proof.json | b13975944 | `cargo test --test integration -- patch_7_3_0` | 0 | Finished `test` profile [optimized + debuginfo] target(s) in 1m 42s; test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10667 filtered out; finished in 0.18s |
| prefork-p730.proof.json | b13975944 | `cargo test --test prefork_full_ui -- patch_7_3_0` | 0 | Finished `test` profile [optimized + debuginfo] target(s) in 0.38s; test result: ok. 3 passed; 0 failed; 3 total |
| integration-sound.proof.json | b13975944 | `cargo test --test integration -- sound` | 0 | Finished `test` profile [optimized + debuginfo] target(s) in 0.31s; test result: ok. 55 passed; 0 failed; 0 ignored; 0 measured; 10613 filtered out; finished in 15.34s |
| prefork-sound.proof.json | b13975944 | `cargo test --test prefork_full_ui -- sound` | 0 | Finished `test` profile [optimized + debuginfo] target(s) in 0.48s; test result: ok. 5 passed; 0 failed; 5 total |
| lib-sound.proof.json | b13975944 | `cargo test --lib -- sound` | 0 | Finished `test` profile [optimized + debuginfo] target(s) in 2m 22s; test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 1971 filtered out; finished in 0.14s |
| integration-blizzard_static_popup.proof.json | b13975944 | `cargo test --test integration -- blizzard_static_popup` | 0 | Finished `test` profile [optimized + debuginfo] target(s) in 0.15s; test result: ok. 40 passed; 0 failed; 0 ignored; 0 measured; 10628 filtered out; finished in 4.22s |
| prefork_full_ui-blizzard_static_popup.proof.json | b13975944 | `cargo test --test prefork_full_ui -- blizzard_static_popup` | 0 | Finished `test` profile [optimized + debuginfo] target(s) in 0.17s; test result: ok. 33 passed; 0 failed; 33 total |
| integration-game_menu.proof.json | b13975944 | `cargo test --test integration -- game_menu` | 0 | Finished `test` profile [optimized + debuginfo] target(s) in 0.16s; test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 10644 filtered out; finished in 18.98s |
| prefork_full_ui-game_menu.proof.json | b13975944 | `cargo test --test prefork_full_ui -- game_menu` | 0 | Finished `test` profile [optimized + debuginfo] target(s) in 0.29s; test result: ok. 14 passed; 0 failed; 14 total |
| integration-ui_panel.proof.json | b13975944 | `cargo test --test integration -- ui_panel` | 0 | Finished `test` profile [optimized + debuginfo] target(s) in 0.31s; test result: ok. 71 passed; 0 failed; 0 ignored; 0 measured; 10597 filtered out; finished in 12.41s |
| prefork_full_ui-ui_panel.proof.json | b13975944 | `cargo test --test prefork_full_ui -- ui_panel` | 0 | Finished `test` profile [optimized + debuginfo] target(s) in 0.22s; test result: ok. 39 passed; 0 failed; 39 total |
| all-sweeps.proof.json | b13975944 | `cargo test --test prefork_full_ui -- publication_sweep` | 0 | Finished `test` profile [optimized + debuginfo] target(s) in 0.48s; test result: ok. 44 passed; 0 failed; 44 total |
| gen_patch_wikitext_register-fixtures.proof.json | b9ac694a6 | `python3 -B /home/osso/.worktrees/wow-ui-sim-p730-page/tools/test_gen_patch_wikitext_register.py` | 0 | Ran 26 tests in 0.002s |
| extract_patch_non_inventory-fixtures.proof.json | b9ac694a6 | `python3 -B /home/osso/.worktrees/wow-ui-sim-p730-page/tools/test_extract_patch_non_inventory.py` | 0 | Ran 34 tests in 0.069s |
| patch_audit_validation-fixtures.proof.json | b9ac694a6 | `python3 -B /home/osso/.worktrees/wow-ui-sim-p730-page/tools/test_patch_audit_validation.py` | 0 | Ran 8 tests in 0.493s |
| format.proof.json | b9ac694a6 | `cargo fmt --check` | 0 | exit 0 |
| mists-check.proof.json | b9ac694a6 | `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | 0 | Finished `dev` profile [optimized + debuginfo] target(s) in 23.57s |
| retail-build.proof.json | b9ac694a6 | `cargo build --bin wow-sim` | 0 | Finished `dev` profile [optimized + debuginfo] target(s) in 0.15s |
| startup.proof.json | b9ac694a6 | `timeout 90 /home/osso/.cache/wow-ui-sim-targets/p730-page/debug/wow-sim --no-addons --no-saved-vars lua-errors` | 0 | startup [] |
| startup-addons.proof.json | b9ac694a6 | `timeout 90 /home/osso/.cache/wow-ui-sim-targets/p730-page/debug/wow-sim --no-saved-vars lua-errors` | 0 | startup [] |
| master-build.proof.json | 85c2acb2d | `cargo build --manifest-path /home/osso/Projects/wow/wow-ui-sim/Cargo.toml --bin wow-sim` | 0 | Finished `dev` profile [optimized + debuginfo] target(s) in 3m 15s |
| master-startup-addons.proof.json | 85c2acb2d | `timeout 90 /home/osso/.cache/wow-ui-sim-targets/master-ref/debug/wow-sim --no-saved-vars lua-errors` | 0 | startup [] |
| extend-receipts.proof.json | b13975944 | `python3 -B /home/osso/.worktrees/wow-ui-sim-p730-page/tools/extend_patch_audit_receipts.py /home/osso/.worktrees/wow-ui-sim-p730-page /home/osso/.worktrees/wow-ui-sim-p730-page/data/patch-api/evidence/7.3.0-session-2026-10-08/integrated/extension p730 Merged 7.3.2 at 85c2acb2d; integrated runtime 045259220` | 0 | exit 0 |
| negative.proof.json | b9ac694a6 | `cargo test --test prefork_full_ui -- patch_7_3_0_publication_sweep` | 1 | Finished `test` profile [optimized + debuginfo] target(s) in 0.29s; test result: FAILED. 0 passed; 1 failed; 1 total |

## Source and gaps

43/43 saved registers and 40/43 extracts reproduced at b13975944; commands and flags retained per row in p730-register-reproduction.json / p730-saved-extract-reproduction.json. Three inherited extraction failures (12.0.5, 12.0.7, 12.1.0) retain their exact prior errors. The generator preserves additive --prose-api-links and --legacy-summary-tables; fixtures 26/34/8 pass. Shared extend_patch_audit_receipts.py adds the real 7.3.2 register (two rows, zero gaps). All sweeps: 43 pages plus factory, 44/44, 8,853 observations. Own gaps 0 -> 0; no gap changes in older sweeps, no supersession edits and no new LATER_AUDIT_REPLACEMENTS exceptions. Negative substitution has exactly one new failing SOUNDKIT row, no resolved/stale IDs, expected exit 1.

## Sound and startup

See sound-caller-review.md and complete scan receipts. No simulator production caller or positive test relies on old sound names; only explicit rejection tests do. All-cache legacy Mists/Cata executable string callers and comments remain distinguished from modern retail. No vendor edits or compatibility fallbacks. Branch no-addons and addons-enabled startup: []; separately built clean master 85c2acb2d addons-enabled startup: []; exact JSON lists equal, zero new errors. Mists check exits 0 with zero non-vendor warnings; six inherited iced manifest warnings plus summary remain unsuppressed.

## Validator matrix

The final post-commit matrix is recorded separately in validator-matrix.json. Every data/patch-api/evidence/*/validate.py must pass. The 7.3.0 validator checks both immutable original evidence and separately sealed integrated receipts, with fixed historical register/sweep scope rather than current globs.

All **20/20 validators pass** at `69306a0f047f3bbcfb2016d5c15c78b0dafc0688`. Final matrix changes only evidence/wiki narration, not the verified code or sealed receipts.

| Evidence directory | Exit | Result |
|---|---|---|
| `10.0.0-session-2026-10-07` | 0 | PASS |
| `10.0.2-session-2026-10-07` | 0 | PASS |
| `10.0.5-session-2026-10-07` | 0 | PASS |
| `7.3.0-session-2026-10-08` | 0 | PASS |
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
