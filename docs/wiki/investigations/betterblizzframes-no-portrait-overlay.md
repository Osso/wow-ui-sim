# BetterBlizzFrames no-portrait overlay

## Symptoms

`Uther` rendered over SpellBook in the supplied screenshot.

## Observed state

Saved configuration enables `BetterBlizzFramesDB.noPortraitModes`. BetterBlizzFrames creates `PlayerFrame.noPortraitMode`, parents it to `PlayerFrame`, and explicitly sets its frame strata to `HIGH`. SpellBook is `MEDIUM`; the observed draw order follows those strata values. This identifies the overlapping frame, not the commit that introduced the reported regression or a verified native-client comparison.

## Non-causes

Mask correction `939efe88d` changed alpha sampling only. It did not change frame strata, levels, parentage, or render order.

The last known-good state is unknown. The user reports real WoW keeps the overlay below SpellBook, but raw native values have not yet been captured. No overlap correction or addon-setting change has been applied.

## Native capture probe

[UnitFrameLayerProbe](../../addons/UnitFrameLayerProbe/README.md) is a read-only kit for the native comparison. It appends up to 30 SavedVariables captures across reloads from login, SpellBook show/hide, and `/unitlayerprobe`; it records raw frame strata/levels/fixed flags, parent chains, geometry, visibility, display metrics, and limited relevant addon/config state. It does not open panels, move frames, alter strata, or change settings.

Local probe protocol coverage has retained GREEN evidence across five cases, including hook installation, manual/login capture, raw missing/error observations, unchanged observed frame properties, reload retention, and the capture cap. Independent verification is pending. The kit is not installed or deployed by this repository.

## Sources

- `BetterBlizzFrames/retail/modules/noPortrait.lua:548-550`
- `/tmp/pi-unit-overlap-evidence.md`
- `/tmp/pi-clipboard-a6471921-93e0-4db8-ba3c-b3a872ce6d2b.png`
- [UnitFrameLayerProbe](../../addons/UnitFrameLayerProbe/README.md) — requested native raw-value capture kit

## See Also

- [[rendering-pipeline]]
- [[playerspells-runtime-load]]
