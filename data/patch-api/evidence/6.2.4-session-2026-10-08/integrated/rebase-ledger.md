# Rebase reconciliation (2026-10-08)

Master `15b417367` supersedes the four-file `scope_input_matches` allowlist: its 7.0.3 gate removes the live runtime-scope digest loop, retaining pinned Git-scope verification. The 7.0.3 validator and seal table are unchanged from master. `test_tool_replacements.py` is retired; `p703-before-*`, `p703-validator-red/green`, `tool-replacement-red/green` and `tool-replacements.json` remain sealed historical records, not current acceptance tests.

`rebase-mapping.json` records all ten supplied old/new commit pairs, stable patch IDs, tree hashes and changed before/after blob IDs. Nine patches are identical; only the superseded allowlist commit differs. The integrated gate resolves old receipt pins to rebased commits and verifies their source scope. Historical comparisons remain against pre-rebase base `846a30663`, rather than relabeling historical runs as new-master runs. Original historical validators and receipts remain unchanged.

`context.json` pins prior-validator discovery to `15b417367`. `check_validators.py` ran each of its 30 validators and checked each executable's bytes against that Git blob; `prior-validator-matrix.json` seals executable/log SHA-256 values. The final current-worktree matrix additionally covers the original and integrated 6.2.4 validators, totaling 32.

Fresh receipts in `rebase-checks/` cover source revision `1dbc6204c`, dedicated target `/home/osso/.cache/wow-ui-sim-targets/p624-page`:

| Command | Result |
|---|---|
| `cargo test --test prefork_full_ui -- publication_sweep` | exit 0; 50 passed |
| `cargo test --test prefork_full_ui -- patch_6_2_4_publication_sweep` | exit 0; 1 passed |
| `python3 -B tools/test_gen_patch_wikitext_register.py` | exit 0; 32 tests |
| `python3 -B tools/test_extract_patch_non_inventory.py` | exit 0; 36 tests |
| `python3 -B tools/test_patch_audit_validation.py` | exit 0; 8 tests |
| `cargo fmt --check` | exit 0 |

Every receipt records exact command, revision, target, exit, log hash and elapsed time. Cargo ran asynchronously with output streamed to log files; no polling/wait loop. Inherited iced manifest warnings remain unsuppressed. Historical source reproduction, negative control and exact-master observations are retained; no runtime source change invalidates those receipts.
