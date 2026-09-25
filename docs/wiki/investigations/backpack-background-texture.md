# Backpack background and slot texture investigation

The bounded release-binary GUI run renders a ready 178×280, 16-slot Backpack. An exact 69977 `bagsitemslot2xc60` CASC asset was missing from the byte cache, fetched by its immutable source key, MD5-verified, and cached; no render fallback or substitute was added.

## Earlier source reading

Published Gethe XML still describes the panel body through `FlatPanelBackgroundTemplate`: solid `PANEL_BACKGROUND_COLOR` fill plus rounded bottom-corner atlases. That source reading remains useful for the static panel-body contract, but it was not sufficient to establish current 69977 visual completeness. It must not be used to claim a native retail texture path, an English-source match, or that a missing runtime asset is intentionally absent.

`BankFrame` separately declares the tiled `bank-frame-background`; Backpack slot chrome is distinct from that body fill. The current work did not change Blizzard/vendor Lua or panel styling.

## Exact 69977 acquisition

The missing Backpack slot asset is FDID `8187737`, cached at:

`~/.cache/wow-ui-sim/casc-extract/Interface/containerframe/bagsitemslot2xc60.blp`

`/tmp/wow-character-bug/assets-staged/backpack-promoted.json` records 17,556 bytes and content MD5 `48bf37a9463a1819f0490662a4e407da`, marked verified. It was acquired from the exact 69977 CASC key and promoted into the standard CASC byte cache. No generated file, Gethe source copy, alternate texture, or new runtime fallback was introduced.

The earlier cold fixture omitted the community listfile CSV; GUI startup changes its working directory to the binary directory, leaving the isolated resolver without a source catalog. Cached blobs therefore could not satisfy `bagsitemslot2xc60` without a path-to-FDID mapping. `d0f525630` now bundles that mapping (`8187737`) with CommonMask and FrameMetal mappings; its targeted limited-listfile regression passed 4/4. This needs no cache-case fallback. The original name is 69913 data, while this BagSlot content MD5 is independently proven for 69977; FrameMetal content remains unverified against the active root. The completed cold replay uses a real community CSV symlink at `resolver/data/community-listfile.csv`, with initially absent resolver SQLite and resolution caches; it validates this mapping in the historical fixture.

## Bounded release visual proof

`/tmp/wow-character-bug/release-mapping-build.result` is `0`; the completed release binary SHA-256 is `0dd8413215a619ce6b5a64c11ff3798931a7cb6c9afd31cce18aacd9a8009861`. In the isolated historical 69977 fixture, the observer reported `RELEASE_READY=true` for Character, PaperDoll, and Backpack; screenshot capture exited `0`, while the enclosing GUI timeout exited the expected `124`. Backpack measured 178×280 with 16 slots, portrait width 36, top corner/top edge height 95, and bottom corner height 100. Main inspected the screenshot: correct bag geometry and slot art are present.

The cold replay used a real community CSV symlink at `resolver/data/community-listfile.csv` with initially absent resolver SQLite and resolution caches. CASC built 1,441,761 resolution entries in 13.6 seconds before GUI font initialization at 30.409 seconds. First draw stalled 1.2 seconds (18.6ms quads, 12.1ms textures, 1.2s other), with no later draw stall. The intentionally excluded FDID `2447783` extracted from local CASC after `--exec-lua`. stderr has no Lua errors; only `ui-hud-minimap-frame-generic-mask` remains `Not found`. This is isolated-fixture release-binary evidence, not texture completeness, default startup, or registered-host startup acceptance. The registered installed source remains blocked by missing `wow_classic_beta` metadata.

## Sources

- `/tmp/wow-character-bug/assets-staged/backpack-promoted.json` — exact asset identity, MD5, and cache path.
- `/tmp/wow-character-bug/release-mapping-build.result` — final release build exit status.
- `d0f525630` — deterministic bundled mapping and 4/4 limited-listfile regression.
- `/tmp/wow-character-bug/release-cold-complete/{result.json,observer.json,stderr,stdout,release-character-backpack.png}` — completed cold isolated release GUI proof, cache boundary, timing, measurements, and inspected screenshot.
- `Blizzard_UIPanels_Game/Mainline/ContainerFrame.xml` and `Blizzard_SharedXML/Mainline/SharedUIPanelTemplates.xml` — static public source structure only.

## See Also

- [[forever-character-panel]] — full bounded character-panel/source-cache matrix.
- [[casc-local-index-generations]] — corrected local generation selection.
- [[casc-asset-cache]] — byte-cache and source-sync architecture.
