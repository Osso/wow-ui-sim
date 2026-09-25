# Forever character panel investigation

The frozen Forever 1.60.1.69977 source/cache path now supports the bounded Camelot character-panel open, close, and reopen acceptance flow. This is simulator evidence, not native Forever behavior or registered-host startup proof.

## Confirmed coverage

| Boundary | Status | Proof |
|---|---|---|
| Character panel lifecycle | passed | One unchanged full-panel acceptance case at `72d6d8b45`; opens, closes, and reopens. |
| Relic-slot query | passed | Three focused cases at `7e7ce5fb0`/`72d6d8b45`; class changes and resolved units are covered. |
| Atlas alias geometry | passed | Two focused cases at `72d6d8b45`. |
| Character/stat API surface | partial | The pre-change cached diagnostic remains 15/25 visible handlers; it does not prove all handlers after the later changes. |
| Four original panel assets | acquired | Exact 69977 encoding keys were fetched and MD5-verified into the normal CASC byte cache; no runtime fallback or substitute was added. |
| Visual lifecycle | bounded pass | Character open/close/reopen and Backpack reported ready with no Lua-error lines in the final GUI artifacts. |

Final scoped verification at `72d6d8b45efc0c012fc94cdea63a9becac534c02` passed 12/12: CDN retry/status (3), relic slot (3), atlas (2), profile identity (1), full panel (1), plus two incidental relic matches. `cargo fmt --check` and `cargo check` passed. The unchanged baseline has two Python casing failures, five dependency warnings, and eight older lib-test warnings; none is credited as fixed.

## Source and cache boundary

The source is a CASC-first, isolated historical 69977 fixture at `/tmp/wow-character-bug/frozen-69977-install` with its `Data` tree symlinked to the real data. Sync acquired all 4,398 Forever manifest files: 4,068 local, 330 CDN, one reused session, in 91.279 seconds. The resulting source cache was byte-equal when promoted to the default wowforever cache. This proves the produced 69977 UI artifact only; it does not repair or register the desktop installation.

Gethe `c6e899…` is manifest-only for this work. Its 4,397 comparable files match after CRLF normalization. `LoadLocale` differs because the selected source is `ptPT`, not `enUS`; this is an existing variant limitation, not native-English source fidelity.

The old false-absence diagnosis is superseded: for the affected local index buckets, generation `ac` is the latest numeric base entry evidenced by the 69977 fixture. Selection must use that newest generation and must not let later directory enumeration select `aa`, merge older generations, or invent a fallback. The four original records are now present in the normal byte cache:

| FDID | Cached path |
|---|---|
| 8175455 | `Interface/paperdollinfoframe/paperdollinfopart2c60.blp` |
| 8245174 | `Interface/common/commonframedividerc60.blp` |
| 8254784 | `Interface/common/commonsidetabmaskc60.blp` |
| 8197104 | `Interface/Icons/INV_SideTab_Stats_c60.blp` |

Each entry in `/tmp/wow-character-bug/assets-staged/promoted.json` records the exact cached path, byte count, and verified content MD5. No generated source, Gethe file copy, runtime fallback, or texture substitute is credited.

## Visual evidence and limits

`/tmp/wow-character-bug/final-visual/result.json` reports a ready screenshot run (the outer timeout is expected). Its observer completed in 8.14 seconds and reported `VISUAL_READY=true`; Backpack measured 178×280 with 16 slots, 36px portrait width, 95px top corner/top edge, and 100px bottom corner. Initial GPU stall was about 1.1 seconds elsewhere; initial texture work was about 13ms. This is bounded GUI evidence, not release acceptance.

The current release build is running through the main alias with Forever features. The real installed `.build.info` is still missing `wow_classic_beta`; its desktop Syncthing debug record was modified on September 24, 2026 at 19:39 CDT. The team did not edit that metadata. Registered-install startup remains blocked, so no host-startup or final-release-visual completion claim follows. An unrelated minimap mask miss remains open.

## Sources

- `/tmp/wow-character-bug/final-visual/{result.json,observer.json}` — final GUI lifecycle and Backpack metrics.
- `/tmp/wow-character-bug/assets-staged/promoted.json` — four exact 69977 asset MD5/cache records.
- `/tmp/wow-character-bug/verified-69977/cache-promotion.json` — 4,398-file byte-equal cache promotion scope.
- `/tmp/wow-character-bug/frozen-69977-install` — isolated historical CASC fixture.
- `72d6d8b45`, `7e7ce5fb0`, `9ac8d0554` — semantic atlas, relic, and published CASC-reader revisions.

## See Also

- [[casc-local-index-generations]] — newest-generation selection and source acquisition boundary.
- [[casc-asset-cache]] — runtime cache layers and sync behavior.
- [[backpack-background-texture]] — exact Backpack slot texture acquisition and visual scope.
- [Forever character-panel remaining stat rows](../../specs/forever-character-remaining-stats.md) — remaining API contract.
- [Forever atlas data](../../specs/forever-atlas-data.md) — atlas generation contract.
