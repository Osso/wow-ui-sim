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

## Bundled cold-fixture mappings

The earlier isolated cold fixture omitted the community listfile CSV. GUI startup changes the working directory to the binary directory in `main.rs`, so the isolated resolver had no source catalog. Cached CASC blobs alone cannot resolve a requested path without its path-to-FDID mapping; the five `Not found` lines were not evidence that those cached blobs were absent.

`d0f525630` bundles deterministic mappings for `commonsidetabmaskc60` (`8254784`), `bagsitemslot2xc60` (`8187737`), and `uiframemetal2xc60` (`8069116`) in the override and generated limited listfiles. Its targeted bundled-listfile regression passed 4/4. The original names are from 69913 data; 69977 content MD5 is independently proven for CommonMask and BagSlot only. FrameMetal's cache content is not independently verified against the active root. No cache-case fallback was added or is needed.

`/tmp/wow-character-bug/release-mapping-build.result` records build exit `0`. The completed cold replay at `/tmp/wow-character-bug/release-cold-complete` uses the real community CSV symlink at `resolver/data/community-listfile.csv`, with initially absent resolver SQLite and resolution caches. The release binary SHA-256 is `0dd8413215a619ce6b5a64c11ff3798931a7cb6c9afd31cce18aacd9a8009861`; it reached `RELEASE_READY=true` for Character, PaperDoll, and Backpack, captured a screenshot, and left only the unrelated generic minimap-mask miss. This validates the mapping fix in the historical fixture, not default or registered-host startup.

## Release visual evidence and limits

`/tmp/wow-character-bug/release-cold-complete/result.json` records the completed bounded release-binary GUI smoke. `release-mapping-build.result` is `0`; the release binary SHA-256 is `0dd8413215a619ce6b5a64c11ff3798931a7cb6c9afd31cce18aacd9a8009861`. The expected outer timeout exited `124` after the observer reported `RELEASE_READY=true` for Character, PaperDoll, and Backpack; screenshot capture exited `0`. Backpack measured 178×280 with 16 slots, 36px portrait width, 95px top corner/top edge, and 100px bottom corner. Main inspected the screenshot: correct bag geometry and slot art are present.

The historical `/tmp/wow-character-bug/frozen-69977-install` fixture used isolated application/resolver caches, a real community CSV symlink at `resolver/data/community-listfile.csv`, and initially absent resolver SQLite and resolution caches. CASC built 1,441,761 resolution entries in 13.6 seconds before GUI font initialization at 30.409 seconds. First draw stalled 1.2 seconds (18.6ms quads, 12.1ms textures, 1.2s other), with no later draw stall. The deliberately absent `ability_racial_jackofalltrades` (FDID `2447783`) extracted from local CASC after `--exec-lua`. stderr has no Lua errors; the only `Not found` line is the unrelated `ui-hud-minimap-frame-generic-mask`. This proves the corrected mappings and bounded visual behavior in the isolated historical fixture, not texture completeness, default startup, or registered-host startup.

The real installed `.product.db` contains the exact `wow_classic_beta` 69977 active-build record, so the missing `.build.info` row does not block default installed-product startup. `692e36936`/`4eb32befa` select the requested product's `activeBuildKey` through cascette's shared parser, without metadata writes or another-product fallback. Current cold-GUI evidence uses default install discovery with no `WOW_INSTALL_PATH`, `WOW_DATA_PATH`, or `WOW_PRODUCT`, isolated XDG directories, and an explicitly isolated fresh `ASSET_RESOLVER_CACHE_DIR` resolver/catalog SQLite. It links the actual community CSV and 1,460 other extracted UI assets, so only resolver/catalog state is cold. CASC built 1,441,761 entries in 7.8s before font initialization at 17.447s. IPC reported `COLD_GUI_READY=true` for Character, PaperDoll, and Backpack; the inspected PNG has c60 art and Backpack at 178×280 with 16 slots, 36px portrait, 95px top, and 100px bottom. The first draw stalled 850.4ms (14.4ms quads, 11.0ms textures, 825.0ms other); no later observed draw exceeded 500ms. Deliberately excluded local FDID `2447783` extracted after GUI without a subsequent 500ms draw stall. `.build.info` and `.product.db` hashes and mtimes were unchanged; the expected enclosing timeout was `124`; stderr has no Lua errors and only the unrelated generic minimap-mask miss. This validates simulator startup against this install, not native launcher behavior, texture completeness, fully cold all-assets startup, English-locale source fidelity, or other profiles.

## Sources

- `/tmp/wow-character-bug/product-db-release-build.{revision,result,stderr}` — `4eb32befa` release-build provenance and exit status.
- `/tmp/wow-character-bug/product-db-cold-gui/{setup.json,result.json,observer.json,stderr,metadata-before.json,metadata-after.json,character-backpack.png}` — current default-install selected cold resolver/catalog GUI readiness, cache boundary, timing, metadata non-mutation, and inspected screenshot.
- `/tmp/wow-character-bug/product-db-error-dialog/{result.json,observer.json,stderr,error-dialog.png,metadata-bracket.json}` — Linux Zenity literal-path rendering and metadata bracket through dialog/parser checks.
- `/tmp/wow-character-bug/product-db-sim-verify-report.json` — simulator verification plus pinned cascette product-db 8/8 proof.
- `/tmp/wow-character-bug/release-cold-complete/{result.json,observer.json,stderr,stdout,release-character-backpack.png}` — completed cold isolated release-binary GUI lifecycle, cache boundary, timing, metrics, and inspected screenshot.
- `/tmp/wow-character-bug/assets-staged/promoted.json` — four exact 69977 asset MD5/cache records.
- `/tmp/wow-character-bug/verified-69977/cache-promotion.json` — 4,398-file byte-equal cache promotion scope.
- `/tmp/wow-character-bug/frozen-69977-install` — isolated historical CASC fixture.
- `72d6d8b45`, `7e7ce5fb0`, `9ac8d0554` — semantic atlas, relic, and published CASC-reader revisions.
- `d0f525630` — deterministic bundled mappings and 4/4 limited-listfile regression.

## See Also

- [[casc-local-index-generations]] — newest-generation selection and source acquisition boundary.
- [[casc-asset-cache]] — runtime cache layers and sync behavior.
- [[backpack-background-texture]] — exact Backpack slot texture acquisition and visual scope.
- [Forever character-panel remaining stat rows](../../specs/forever-character-remaining-stats.md) — remaining API contract.
- [Forever atlas data](../../specs/forever-atlas-data.md) — atlas generation contract.
