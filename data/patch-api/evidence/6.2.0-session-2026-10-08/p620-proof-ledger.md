# 6.2.0 proof ledger

Runtime src/tests pinned at `da30fc360`; source-reproduction set at `114659452`. Later evidence/docs/validator commits do not invalidate runtime proof. Each receipt pins its actual execution revision and full log hash.

| Command | Revision | Result | Scope / invalidation |
|---|---|---|---|
| `cargo test --test prefork_full_ui -- patch_6_2_0` | `da30fc360` | exit 0; 3 cases | p620-green; valid |
| `cargo test --test prefork_full_ui -- publication_sweep` | `da30fc360` | exit 0; 50 cases | p620-all-sweeps; valid |
| `cargo test --test integration patch_6_2_0 -- --nocapture` | `da30fc360` | exit 0; 1 cases | p620-bare; valid |
| `cargo test --test integration tooltip_item_spell:: -- --nocapture` | `da30fc360` | exit 0; 88 cases | p620-tooltip-item-spell; valid |
| `cargo test --test integration tooltip_basic:: -- --nocapture` | `da30fc360` | exit 0; 62 cases | p620-tooltip-basic; valid |
| `cargo test --test integration tooltip_spell_mount_identifiers:: -- --nocapture` | `da30fc360` | exit 0; 22 cases | p620-tooltip-identifiers; valid |
| `cargo build --bin wow-sim` | `da30fc360` | exit 0 | p620-branch-build; valid |
| `timeout 90 /home/osso/.cache/wow-ui-sim-targets/p620-page/debug/wow-sim --no-saved-vars lua-errors` | `da30fc360` | exit 0 | p620-branch-startup; valid |
| `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | `b3cf1e923` | exit 0 | p620-mists-check; valid |
| `cargo fmt --check` | `b3cf1e923` | exit 0 | p620-format; valid |
| `cargo build --manifest-path /home/osso/.cache/wow-ui-sim-targets/p620-page/master-source/Cargo.toml --bin wow-sim` | `b3cf1e923` | exit 0 | p620-master-build; valid |
| `timeout 90 /home/osso/.cache/wow-ui-sim-targets/p620-page/debug/wow-sim --no-saved-vars lua-errors` | `2342e6c6f` | exit 0 | p620-master-startup; valid |
| `python3 -B $AUDIT_CHECKOUT/data/patch-api/evidence/6.2.0-session-2026-10-08/run_targeted.py` | `da30fc360` | exit 0 | p620-targeted-driver; valid |
| `python3 -B tools/test_extract_patch_non_inventory.py` | `da30fc360` | exit 0; 36 fixtures | p620-fixtures-extractor-final; valid |
| `python3 -B tools/test_gen_patch_wikitext_register.py` | `114659452` | exit 0; 31 fixtures | p620-test_gen_patch_wikitext_register; valid |
| `python3 -B tools/test_patch_audit_validation.py` | `114659452` | exit 0; 8 fixtures | p620-test_patch_audit_validation; valid |

## Additional proof

- Source reproduction: 49/49 registers, 46/49 extracts; exact inherited failures 12.0.5/12.0.7/12.1.0. Uses recorded provenance flags or prior verified flags where provenance predates fields. 249 prior source files byte-identical.
- All 30 validators named validate*.py in Git at 846a30663 pass. The 7.0.3 integrated portability repair is copied exactly from master 15b417367, not the abandoned local self-hash approach. Own validator passes.
- Whole-word /usr/bin/grep caller output is untruncated. Parent page has no removed members; retirement member set and cached-retirement scan set are empty. Fixed master/p624 Git register inventories are recorded; no missing worktree inputs required.
- Addons-enabled branch and immutable master 846a30663 startup outputs are both exactly []; neither invocation uses --no-addons. The runner clears WOW_SIM_NO_ADDONS/WOW_SIM_NO_SAVED_VARS from the environment. Builds precede timeout 90 runs.
- Changed Rust readability: reviewed cost policy, payload path, widget dispatch and behavioral tests. New policy is pure; existing builders retain shared generation. No new warning suppressions, deep nesting or duplicated bodies. Mists reports only six inherited iced vendor manifest deprecations.

## Superseded development attempts

- First Cargo capture orphaned to PID 1 after Pyrun returned no result. Owned Cargo/process tree was cancelled; exit verified. No proof credited. Missing output justifies file-backed rerun.
- Discovery at 114659452: sweep passes; linked-cost assertion fails on MANA (exit 1). This is RED evidence, not acceptance.
- Imported 6.2.4 extractor fixture used an unnormalized heading; failure retained and marked invalidated. Exact c79de9881 fixture correction passes 36/36.
- Initial generator/extract reproduction omitted legacy verified flags; corrected receipts use historical flags, with all registers and all but the three inherited extracts matching.
- The pre-repair validator matrix records the live src/c_api/mod.rs hash failure. Master repair is the accepted final code; all existing validator logs remain preserved.
