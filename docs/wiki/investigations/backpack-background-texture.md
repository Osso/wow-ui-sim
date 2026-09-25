# Backpack background and slot texture investigation

The bounded release-binary GUI run renders a ready 178×280, 16-slot Backpack. An exact 69977 `bagsitemslot2xc60` CASC asset was missing from the byte cache, fetched by its immutable source key, MD5-verified, and cached; no render fallback or substitute was added.

## Earlier source reading

Published Gethe XML still describes the panel body through `FlatPanelBackgroundTemplate`: solid `PANEL_BACKGROUND_COLOR` fill plus rounded bottom-corner atlases. That source reading remains useful for the static panel-body contract, but it was not sufficient to establish current 69977 visual completeness. It must not be used to claim a native retail texture path, an English-source match, or that a missing runtime asset is intentionally absent.

`BankFrame` separately declares the tiled `bank-frame-background`; Backpack slot chrome is distinct from that body fill. The current work did not change Blizzard/vendor Lua or panel styling.

## Exact 69977 acquisition

The missing Backpack slot asset is FDID `8187737`, cached at:

`~/.cache/wow-ui-sim/casc-extract/Interface/containerframe/bagsitemslot2xc60.blp`

`/tmp/wow-character-bug/assets-staged/backpack-promoted.json` records 17,556 bytes and content MD5 `48bf37a9463a1819f0490662a4e407da`, marked verified. It was acquired from the exact 69977 CASC key and promoted into the standard CASC byte cache. No generated file, Gethe source copy, alternate texture, or new runtime fallback was introduced.

## Bounded release visual proof

The release `wow-sim` build exited `0` with SHA-256 `0e6aceb9b05aaf984f1f85bb630b21b04e20c5dc1475a82f081de9714ba2fcc5`. In the isolated historical 69977 fixture, the observer reported `RELEASE_READY=true` for Character, PaperDoll, and Backpack; screenshot capture exited `0`, while the enclosing GUI timeout exited the expected `124`. The script opened/closed/reopened Character twice and Backpack once. Backpack measured 178×280 with 16 slots, portrait width 36, top corner/top edge height 95, and bottom corner height 100. Main inspected the screenshot: Character and Backpack were visible and Backpack was not oversized.

The cold resolver cache built 1,441,761 entries in 7.5 seconds before GUI startup. One intentionally excluded locally extractable icon, FDID `2447783`, was extracted only after `--exec-lua`; other texture and UI caches were intentionally reused. The only draw-stall line was 828.2ms (13.7ms quads, 10.0ms textures, 804.6ms other), with no later stall lines. stderr also reports `Not found` for `commonsidetabmaskc60`, `bagsitemslot2xc60`, `uiframemetal2xc60`, `ui-hud-actionbar-bag`, and `ui-hud-minimap-frame-generic-mask`; the path/case/cache cause remains under investigation. Thus this is isolated-fixture release-binary evidence, not texture-completeness, default, or registered-host startup acceptance. The registered installed source remains blocked by missing `wow_classic_beta` metadata.

## Sources

- `/tmp/wow-character-bug/assets-staged/backpack-promoted.json` — exact asset identity, MD5, and cache path.
- `/tmp/wow-character-bug/release-build.result` — release build exit status.
- `/tmp/wow-character-bug/release-cold-final/{setup.json,result.json,observer.json,stderr,stdout,release-character-backpack.png}` — isolated release GUI proof, cache boundary, timing, measurements, and inspected screenshot.
- `Blizzard_UIPanels_Game/Mainline/ContainerFrame.xml` and `Blizzard_SharedXML/Mainline/SharedUIPanelTemplates.xml` — static public source structure only.

## See Also

- [[forever-character-panel]] — full bounded character-panel/source-cache matrix.
- [[casc-local-index-generations]] — corrected local generation selection.
- [[casc-asset-cache]] — byte-cache and source-sync architecture.
