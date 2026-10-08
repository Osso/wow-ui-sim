# 5.4.7 integrated acceptance

Base: `bebcc5830dcb21e768a2e3c365105351a400fccb`. Rust/source/tool execution scope: `b14af5446`. Validator gate revision: `e65d0f2fb`.

| Command / proof | Result |
|---|---|
| `python3 -B reproduce_sources.py` | 56 byte-identical registers; 53 byte-identical extracts; inherited 12.0.5/12.0.7/12.1.0 errors unchanged with recorded flags |
| `cargo test --test prefork_full_ui -- patch_5_4_7_publication_sweep` | 1/1 pass; nine observations unchanged, two gaps |
| `cargo test --test prefork_full_ui -- publication_sweep` | 57/57 pass; all 55 other pages identical to fresh bebcc5830 observations |
| Same publication command on detached bebcc5830 | 56/56 pass; results retained under this session's `master/` |
| `cargo test --test prefork_full_ui -- patch_5_4_7` | 2/2 pass |
| `cargo test --test integration -- patch_5_4_7` | Exit 0; zero selected cases. 5.4.7 cases are prefork-only; no extra integration coverage claim |
| Own sweep with committed `negative-register.json` | Expected exit 1, exactly one changed observation, gaps 2 → 3 |
| `python3 -B tools/test_check_patch_validators.py` | 4/4 pass |
| `python3 -B tools/test_extract_patch_non_inventory.py` | 36/36 pass |
| `python3 -B tools/test_gen_patch_wikitext_register.py` | 34/34 pass |
| `python3 -B tools/test_patch_audit_validation.py` | 8/8 pass |
| `python3 -B tools/test_patch_warlords_register.py` | 3/3 pass; fixture total 85/85 |
| `cargo fmt --check` | Exit 0 |
| `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | Exit 0; zero non-vendor warnings, seven unchanged vendor-manifest warning lines |
| Prior validators inventoried with `git ls-tree` at bebcc5830 | 38/38 pass |
| `python3 -B integrated/validate.py` | PASS: 56 pages, 56 registers, 53 extracts, 16 receipts, ten own patch-equivalent rebased commits, one external pin |
| `python3 -B tools/check_patch_validators.py` | PASS: clean 40/40; synthetic later audit 41/41; zero failures |

[Command ledger](command-ledger.md), [receipts](receipts.json), [command results](command-results.json), [gap comparison](gap-comparison.json), [supersession review](supersession-review.json), [gate report](validator-gate-report.json), [gate command receipt](validator-gate.proof.json).

Historical records remain intact; only the historical validator entry point became a replay wrapper around its preserved source. Shared proof inputs resolve at recorded Git revisions, not live checkout bytes. Immutable seals cover committed session artifacts only. No new gaps, closures, replacements, runtime retirements, runtime/shared-tool/classifier changes, vendor edits, push, merge or agents. No full integration-suite or fresh startup-output claim. Later evidence/wiki-only commits leave these proof scopes valid.

Wiki index remains 2,727 lines; log grows from 471 to 475 lines. No `__pycache__` generated. Temporary master-proof and validator-gate worktrees are removed.
