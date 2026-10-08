# Final command ledger

Source/code proof revision: `25934cc4ea714a08b5e5a323d747840b661d6da3`. Historical register/sweep scope: `c5f05d05d`. Later documentation/evidence commits do not invalidate these source-scoped commands.

| Command | Revision | Exit | Scope/result |
|---|---|---:|---|
| `cargo test --test prefork_full_ui -- patch_7_3_2` | `25934cc4e` | 0 | p732-cached-active-stack; test result: ok. 2 passed; 0 failed; 2 total |
| `cargo test --test integration patch_7_3_2` | `25934cc4e` | 0 | p732-bare-final; test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 10663 filtered out; finished in 0.16s |
| `cargo test --test integration key_dispatch` | `25934cc4e` | 0 | p732-key-dispatch; test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 10636 filtered out; finished in 0.33s |
| `cargo test --test integration blizzard_game_menu` | `25934cc4e` | 0 | p732-menu-bare; test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 10661 filtered out; finished in 0.01s |
| `cargo test --test prefork_full_ui -- blizzard_game_menu` | `25934cc4e` | 0 | p732-menu-cached; test result: ok. 10 passed; 0 failed; 10 total |
| `cargo test --test prefork_full_ui -- publication_sweep` | `25934cc4e` | 0 | p732-all-sweeps; test result: ok. 42 passed; 0 failed; 42 total |
| `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | `25934cc4e` | 0 | p732-mists-check; See full log |
| `cargo test --no-default-features --features sound,gui,casc,client-mists --test integration patch_7_3_2` | `ac94f5a4a` | 0 | p732-mists-session; test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8169 filtered out; finished in 0.16s |
| `cargo fmt --all -- --check` | `ac94f5a4a` | 0 | p732-format; See full log |
| `cargo test --test prefork_full_ui -- patch_7_3_2_publication_sweep` | `25934cc4e` | 1 | p732-negative; test result: FAILED. 0 passed; 1 failed; 1 total |
| `python3 -B /home/osso/.worktrees/wow-ui-sim-p732-page/data/patch-api/evidence/7.3.2-session-2026-10-08/reproduce_sources.py` | `c5f05d05d` | 0 | p732-source-reproduction; See full log |
| `python3 -B tools/test_gen_patch_wikitext_register.py` | `cca5a744e` | 0 | tool fixtures; Ran 24 tests in 0.001s |
| `python3 -B tools/test_extract_patch_non_inventory.py` | `cca5a744e` | 0 | tool fixtures; Ran 33 tests in 0.032s |
| `python3 -B tools/test_patch_audit_validation.py` | `56167088a` | 0 | p732-validation-fixtures; Ran 8 tests in 0.341s |
| `cargo test --test prefork_full_ui -- patch_7_3_2` | `c5f05d05d` | 1 | p732-discovery; Historical RED: cached publication passes; insecure actions mutate state. |
| `cargo test --test integration patch_7_3_2_bare` | `56167088a` | 101 | p732-bare-red; Historical RED: explicit taint capture and session state mutation. |

Every log/receipt is retained by basename in `p732-proof.json`; SHA-256 binds full output. No command was repeated merely to recover logs. RED commands prove historical failure, not final acceptance.

Additional proof: own validator PASS; all 18 local validators PASS; relocation accepts a later out-of-scope register and rejects preserved-input whitespace tampering and missing reproduction rows. Mists check has zero non-vendor warnings; six inherited iced manifest warnings are unsuppressed.

Retirements: none. Complete grep-qualified/bare cached scans, whole-word source/test caller scans, the p801 register snapshot and merged-master 8.0.1 recheck are retained.

Reproduction: 41/41 registers, 38/41 saved extracts. Inherited 12.0.5/12.0.7 byte mismatches and 12.1.0 description-template failure remain unchanged. All 209 original source inputs / 86 old extractor-mode outcomes preserved.

Superseded development attempts: `p732-cached-green` failed compilation because the temporary module is private; `de193c97f` wires its existing facade. `p732-cached-final` failed behavioral protection because the native frame is untainted; `25934cc4e` matches the VM active-stack rule. Neither is acceptance evidence.

Outstanding integration: 8.0.1 merged while proof ran; placeholder intentionally retained for main-thread rebase/integration. Preserve referenced proof revisions or refresh proofs if rebasing. Native error/block-event, hardware-event, countdown/cancellation and historical/secret/restricted-context parity remain recorded, not modeled.
