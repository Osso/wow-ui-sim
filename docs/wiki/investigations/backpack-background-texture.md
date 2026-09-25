# Backpack background and slot texture investigation

The current bounded GUI run renders a ready 178×280, 16-slot Backpack. An exact 69977 `bagsitemslot2xc60` CASC asset was missing from the byte cache, fetched by its immutable source key, MD5-verified, and cached; no render fallback or substitute was added.

## Earlier source reading

Published Gethe XML still describes the panel body through `FlatPanelBackgroundTemplate`: solid `PANEL_BACKGROUND_COLOR` fill plus rounded bottom-corner atlases. That source reading remains useful for the static panel-body contract, but it was not sufficient to establish current 69977 visual completeness. It must not be used to claim a native retail texture path, an English-source match, or that a missing runtime asset is intentionally absent.

`BankFrame` separately declares the tiled `bank-frame-background`; Backpack slot chrome is distinct from that body fill. The current work did not change Blizzard/vendor Lua or panel styling.

## Exact 69977 acquisition

The missing Backpack slot asset is FDID `8187737`, cached at:

`~/.cache/wow-ui-sim/casc-extract/Interface/containerframe/bagsitemslot2xc60.blp`

`/tmp/wow-character-bug/assets-staged/backpack-promoted.json` records 17,556 bytes and content MD5 `48bf37a9463a1819f0490662a4e407da`, marked verified. It was acquired from the exact 69977 CASC key and promoted into the standard CASC byte cache. No generated file, Gethe source copy, alternate texture, or new runtime fallback was introduced.

## Bounded visual proof

The final GUI observer reports `VISUAL_READY=true`, a 178×280 Backpack with 16 slots, portrait width 36, top corner/top edge height 95, and bottom corner height 100. It completed in 8.14 seconds; screenshot capture exited 0. The outer GUI timeout was expected and result metadata records `ready=true` and `screenshot=true`. No Lua-error lines were reported for the exercised character/open-close-reopen and Backpack flow.

This is not final release visual acceptance. The registered installed source remains blocked by missing `wow_classic_beta` metadata, and unrelated minimap mask misses remain outside this investigation.

## Sources

- `/tmp/wow-character-bug/assets-staged/backpack-promoted.json` — exact asset identity, MD5, and cache path.
- `/tmp/wow-character-bug/final-visual/{result.json,observer.json}` — bounded GUI and Backpack measurements.
- `Blizzard_UIPanels_Game/Mainline/ContainerFrame.xml` and `Blizzard_SharedXML/Mainline/SharedUIPanelTemplates.xml` — static public source structure only.

## See Also

- [[forever-character-panel]] — full bounded character-panel/source-cache matrix.
- [[casc-local-index-generations]] — corrected local generation selection.
- [[casc-asset-cache]] — byte-cache and source-sync architecture.
