# BetterBlizzFrames no-portrait overlay

## Symptoms

`Uther` rendered over SpellBook in the supplied screenshot.

## Observed state

Saved configuration enables `BetterBlizzFramesDB.noPortraitModes`. BetterBlizzFrames creates `PlayerFrame.noPortraitMode`, parents it to `PlayerFrame`, and explicitly sets its frame strata to `HIGH`. SpellBook is an independent `MEDIUM` panel. The overlap is therefore a simulator render-group defect: raw child strata alone incorrectly let the child escape its LOW top-level owner.

## Non-causes

Mask correction `939efe88d` changed alpha sampling only. It did not change frame strata, levels, parentage, or render order.

The last known-good state is unknown. No addon-setting change was applied. The controlled native capture below establishes the needed ordering boundary; it does not make raw raised ordinals a global z-index.

## Native capture

The original retail `12.1.0.69587` / interface `120100` capture records `PlayerFrame.noPortraitMode` as `HIGH`, raw frame level `2`, raised frame level `0`; `PlayerSpellsFrame` is `MEDIUM`, raw level `1`, and its `SpellBookFrame` shares its raised state. Relevant frames report `HasFixedFrameStrata() == false`.

The controlled capture then records every control at physical 1440-pixel display height, not a 768-unit logical canvas. In all created, panel-hide-show, and panel-raise screenshots, case 1 is blue: a `HIGH` child remains in its unraised `LOW` top-level parent's render group behind independent `MEDIUM`. Cases 2–5 are red: independent `HIGH`, `DIALOG`, plain `TOOLTIP`, and actual `GameTooltip` remain above independent `MEDIUM`. The MEDIUM controls report raised `0` when created, then `33..37` after hide/show and `38..42` after explicit Raise; the LOW parent and red controls remain `0`. These ordinals show transition state, not global promotion across independent strata.

## Controlled native comparison

[UnitFrameLayerProbe](../../addons/UnitFrameLayerProbe/README.md) supplied five opaque red/blue comparisons across three phases: a HIGH child under LOW parent, independent HIGH, independent DIALOG, plain TOOLTIP, and owned `GameTooltip` versus MEDIUM. Screenshot-success gating, a 1.1-second interphase delay, cancellation/timeout cleanup, late-event isolation, and bounded control history define the capture protocol. The native run completed with `cleanupErrors=0`; development protocol coverage remains 9/9.

`f5d9ff91f` implements owner-strata grouping and the native-compatible raised-level getter boundary without changing raw strata or raw frame levels. Targeted GREEN is still pending because hit-grid input ordering must consume the flattened render buckets. No full-scene GUI or readability acceptance is claimed.

## Sources

- `/tmp/pi-unit-controls-native.lua` — authoritative controlled native capture
- `docs/local/private/probes/UnitFrameLayerProbe-controls-2026-09-08.lua` — ignored archive of that capture
- `/tmp/pi-native-control-pixels.json` — authoritative screenshot pixel classification
- [UnitFrameLayerProbe](../../addons/UnitFrameLayerProbe/README.md) — controlled capture protocol

## See Also

- [[rendering-pipeline]]
- [top-level render groups](../../specs/toplevel-render-groups.md)
- [[playerspells-runtime-load]]
