# Owning-after saved runtime verification

**PASS — bounded publication replay only.** Source `0d819ea3ded9904e5a69757b66e67742a78c7635`.

| Evidence | Observed |
|---|---|
| Compile JSON | All 741 records parsed; 665 artifacts, 75 build scripts, terminal build-finished success=true; 0 compiler messages |
| Compile stderr | Full 1,376 bytes read; six iced-wgpu-patched manifest Clippy kebab-case deprecations plus aggregate warning, not compiler-code warnings |
| Compile terminal | success, ExecMainStatus=0, active/exited (RemainAfterExit), MainPID=0, empty ControlGroup |
| Compile memory | MemoryPeak 4,307,513,344 < MemoryHigh 8,589,934,592 (8G); max 9,663,676,416 |
| Owning primary | 1 passed, 0 failed, exit 0; three zero-case auxiliary groups excluded |
| Publication | Complete JSON: 416 unique IDs equal archived register; 268 OK, 148 non-OK exactly equal expected 148 |

## Source and delta

Archived fixture, current committed HEAD fixture and current working fixture agree. Six selected archived source inputs match literal committed source. Full source-scope metadata retained: 707 selected archive paths; closed evidence payloads excluded. No source.tar read or full archive authentication. `tests/patch_10_0_2_publication_sweep.rs` binds register/fixture and row_count=416; `tests/common/publication_sweep.rs:306-317` writes complete results before exact-gap assertion.

Row309 `wt-global-api-C_UIWidgetManager.GetTextWithSubtextWidgetVisualizationInfo-309` is OK: raw=function; lookup=function, published, no supersession. Expected fixture removes only this ID (149→148), adds none. All 416 before/after result objects are identical: publication row309 was already OK before; only expected-gap reconciliation changed the outcome. Owning-before **FAIL** report/proof at `PRIVATE_ROOT/owning-before/runtime-gate` preserved unchanged. Only non-doc source change between source epochs is the one fixture deletion.

## Receipt — metadata continuity, not independent byte authentication

Flat receipt status PASS; receipt path equals seal sealed_path; hash_reads=1 and producer_hash_reads=1. Receipt explicitly reuses the **SINGLE producer digest** `2bc40bf8480a3b081b87244a13c4faa8f1937d616211f181f14d6f37e0aa35fa`. Seal cargo artifact matches parsed compile artifact and Retail features. Current executable stat only equals seal, receipt before/after and runtime reused stat (dev 66306, inode 152581684, size 514974296, mtime_ns 1792005024401255792); producer original before/after stats also stable. Verifier executable reads/hash reads: **0**. No independent executable-byte authentication claim.

## Runtime and diagnostic limits

Full argv, admission, stdout (323 bytes), stderr (8,782 bytes/138 lines), launcher, result and all JSON rows retained in proof. Source mounted read-only onto canonical path; only runtime output explicitly writable-bound; timeout 86s, network unshared; MemoryHigh=1536M, MemoryMax=2G. Admission allowed=true; saved launcher success, 10.408s, 1.5G peak, no swap. Normal prefork setup calls conformance then full Game preload; successful conformance output suppressed, no extra case count claimed.

All runtime stderr lines are Startup/Workarounds/EditMode; no denial/error/warning lines. Startup reaches FIRST_FRAME_RENDERED. Profile cache resolver, parent bytecode bypass and child read-only are source-confirmed. Cache bytes/settings/dependencies and historical full inherited environment/XDG_CACHE_HOME not authenticated. No WTF/font diagnostics: neither real WTF import nor font/render coverage established. WOW_SIM_NO_SOUND=1 and WOW_SIM_CASC=0 exclude sound and live CASC extraction/texture proof. Existing workaround activity is not native parity.

Compile's six manifest deprecations: large-enum-variant, map-entry, match-wildcard-for-single-variants, redundant-closure-for-method-calls, trivially-copy-pass-by-ref, type-complexity; warnings request underscore spellings. Full stderr retained, none suppressed.

**Not proven:** 27-field behavior, native parity, rendering, all-profile/broad suites, or current-parent acceptance. No commands reran compilation/tests/runtime; no network/delegation/canonical edits/executable reads/hashes/copies/archive payload reads. One compile terminal metadata read only.
