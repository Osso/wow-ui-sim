# 6.2.2 proof ledger

Scope: pinned navigation-only stub; publication/absence only. No runtime/tool changes and no native/CASC parity claim.

| Command | Revision | Exit | Evidence | Scope / invalidation |
|---|---|---|---|---|
| `cargo test --test prefork_full_ui -- patch_6_2_2` | `c30c6c1cb01ec9a3f734ddcaf9faf222fe40644c` | 0 | [p622-discovery](p622-discovery.log); SHA-256 `d2e6ba58596d6947f37b97f5ffd5356023ca8d5fa2f988a9da797c6867de2ccb` | Receipt hashes source/test/tool files at recorded revision; unchanged relevant code. No later invalidation. |
| `cargo test --test prefork_full_ui -- publication_sweep` | `40ea2f647120cbcaa2b29e6326a6da065ca1bad6` | 0 | [p622-all-sweeps](p622-all-sweeps.log); SHA-256 `0edb93e946a409fea2f367fe110635112ba4f618149698842e7d8068ca187259` | Receipt hashes source/test/tool files at recorded revision; unchanged relevant code. No later invalidation. |
| `cargo test --test prefork_full_ui -- patch_6_2_2` | `40ea2f647120cbcaa2b29e6326a6da065ca1bad6` | 1 | [p622-negative](p622-negative.log); SHA-256 `1d7e998c52071434fe4d53de86ce101bffe382132e14531f269f628b671c8abd` | One fabricated entry rejected by pinned zero-row count; expected exit 1. No publication-gap claim. |
| `cargo check --no-default-features --features sound,gui,casc,client-mists --tests` | `40ea2f647120cbcaa2b29e6326a6da065ca1bad6` | 0 | [p622-mists-check](p622-mists-check.log); SHA-256 `862e0d88bae6d2ef4b24db0758147cb1e4cf0b9405389e193eae9e4360909fd4` | Receipt hashes source/test/tool files at recorded revision; unchanged relevant code. No later invalidation. |
| `cargo fmt --check` | `40ea2f647120cbcaa2b29e6326a6da065ca1bad6` | 0 | [p622-format-check](p622-format-check.log); SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | Receipt hashes source/test/tool files at recorded revision; unchanged relevant code. No later invalidation. |
| `python3 -B tools/test_gen_patch_wikitext_register.py` | `c30c6c1cb01ec9a3f734ddcaf9faf222fe40644c` | 0 | [p622-generator-fixtures](p622-generator-fixtures.log); SHA-256 `9b45a38fb3a9cbec62423d6ebca558a9480ce5e3dcd67f50d18f19737b67be82` | Receipt hashes source/test/tool files at recorded revision; unchanged relevant code. No later invalidation. |
| `python3 -B tools/test_extract_patch_non_inventory.py` | `c30c6c1cb01ec9a3f734ddcaf9faf222fe40644c` | 0 | [p622-extractor-fixtures](p622-extractor-fixtures.log); SHA-256 `b401661f7be7591d6b385e3821e209df0abadb93f486051df18cd81f5984868b` | Receipt hashes source/test/tool files at recorded revision; unchanged relevant code. No later invalidation. |
| `python3 -B tools/test_patch_audit_validation.py` | `c30c6c1cb01ec9a3f734ddcaf9faf222fe40644c` | 0 | [p622-validator-fixtures](p622-validator-fixtures.log); SHA-256 `288c1b1a5014a9e4eba36e5547d552f1f954a7339abe224610c4cbea19a78221` | Receipt hashes source/test/tool files at recorded revision; unchanged relevant code. No later invalidation. |
| `python3 -B /home/osso/.worktrees/wow-ui-sim-p622-page/data/patch-api/evidence/6.2.2-session-2026-10-08/reproduce_sources.py` | `c30c6c1cb01ec9a3f734ddcaf9faf222fe40644c` | 0 | [p622-reproduction](p622-reproduction.log); SHA-256 `a1d4da58707021e2ec9a64cf496a10a5bc802010b288a78cc6b33dd1b6770e0c` | Receipt hashes source/test/tool files at recorded revision; unchanged relevant code. No later invalidation. |
| `python3 -B /home/osso/.worktrees/wow-ui-sim-p622-page/data/patch-api/evidence/6.2.2-session-2026-10-08/check_prior_validators.py` | `c30c6c1cb01ec9a3f734ddcaf9faf222fe40644c` | 0 | [p622-prior-validators](p622-prior-validators.log); SHA-256 `bcf75ee40de7eb61bc236aeb14516a6cd93f20bfdcf99f9afdc02f83459b8600` | Receipt hashes source/test/tool files at recorded revision; unchanged relevant code. No later invalidation. |

Source accounting: zero register entries, one metadata-only extract row, zero modeled/problematic gaps, zero retirement scan targets.

Historical source/register scope: `c30c6c1cb`. Prior-validator set: exact `git ls-tree` snapshot at `dfede62de1f48ef2af1a4d37a18d274e26df3d81`. Preservation hashes retain every original source file; current immutable source artifacts remain checked, unrelated outcomes/scheduling may evolve.

Initial acceptance-wrapper assertion expected conventional libtest exit 101; corrected to observed prefork exit 1. All actual commands completed; saved outcomes validated without rerunning Cargo. See `p622-acceptance-wrapper-correction.json`.

Own gate and relocated/future-file/tamper/restoration checks pass. No shared runtime path changed, so no addons-enabled startup comparison or unrelated caller tests required. No full integration suite run.
