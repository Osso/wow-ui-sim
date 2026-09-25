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
| Release visual lifecycle | bounded pass | Release `wow-sim` opened/closed/reopened Character and opened/closed/reopened Backpack in the isolated 69977 fixture; both panels were ready. |

Final scoped verification at `72d6d8b45efc0c012fc94cdea63a9becac534c02` passed 12/12: CDN retry/status (3), relic slot (3), atlas (2), profile identity (1), full panel (1), plus two incidental relic matches. `cargo fmt --check` and `cargo check` passed. The later release build exited `0`; `wow-sim` SHA-256 is `0e6aceb9b05aaf984f1f85bb630b21b04e20c5dc1475a82f081de9714ba2fcc5`. The unchanged baseline has two Python casing failures, five dependency warnings, and eight older lib-test warnings; none is credited as fixed.

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

## Release visual evidence and limits

`/tmp/wow-character-bug/release-cold-final/result.json` records a bounded release-binary GUI smoke. The release build exited `0`; its `wow-sim` SHA-256 is `0e6aceb9b05aaf984f1f85bb630b21b04e20c5dc1475a82f081de9714ba2fcc5`. The expected outer timeout exited `124` only after the observer reported IPC readiness and `RELEASE_READY=true` for Character, PaperDoll, and Backpack; screenshot capture exited `0`. The script exercised Character twice, then Backpack open, close, and reopen. Backpack measured 178×280 with 16 slots, 36px portrait width, 95px top corner/top edge, and 100px bottom corner. Main inspected the screenshot: Character and Backpack were visible and Backpack was not oversized; that replay does not prove texture completeness.

This used the historical `/tmp/wow-character-bug/frozen-69977-install` fixture. Resolver and application caches were isolated; 1,460 existing extracts and other texture/UI caches were intentionally reused, while `ability_racial_jackofalltrades` (FDID `2447783`) was excluded and extracted from local CASC only after `--exec-lua`. Before GUI startup, CASC built 1,441,761 resolution entries in 7.5 seconds; font initialization began at 12.507 seconds. The only recorded draw-stall line was 828.2ms (13.7ms quads, 10.0ms textures, 804.6ms other), with no later stall lines. However, stderr also reports `Not found` for `commonsidetabmaskc60`, `bagsitemslot2xc60`, `uiframemetal2xc60`, `ui-hud-actionbar-bag`, and `ui-hud-minimap-frame-generic-mask`; path/case/cache cause is under investigation. This is release-binary evidence for the isolated historical fixture, not a texture-completeness, default, or registered-host startup claim.

The real installed `.build.info` is still missing `wow_classic_beta`; its desktop Syncthing debug record was modified on September 24, 2026 at 19:39 CDT. The team did not edit that metadata. Registered-install startup remains blocked. An unrelated minimap mask miss remains open.

## Sources

- `/tmp/wow-character-bug/release-build.result` — release build exit status.
- `/tmp/wow-character-bug/release-cold-final/{setup.json,result.json,observer.json,stderr,stdout,release-character-backpack.png}` — isolated release-binary GUI lifecycle, cache boundary, timing, metrics, and inspected screenshot.
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
