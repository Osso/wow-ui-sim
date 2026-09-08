# BetterBlizzFrames no-portrait overlay

## Symptoms

`Uther` rendered over SpellBook in the supplied screenshot.

## Observed state

Saved configuration enables `BetterBlizzFramesDB.noPortraitModes`. BetterBlizzFrames creates `PlayerFrame.noPortraitMode`, parents it to `PlayerFrame`, and explicitly sets its frame strata to `HIGH`. SpellBook is `MEDIUM`; the observed draw order follows those strata values. This identifies the overlapping frame, not the commit that introduced the reported regression or a verified native-client comparison.

## Non-causes

Mask correction `939efe88d` changed alpha sampling only. It did not change frame strata, levels, parentage, or render order.

The last known-good state is unknown. The received raw native values are recorded below; controlled native comparisons have not yet been captured. No overlap correction or addon-setting change has been applied.

## Native capture

The received retail `12.1.0.69587` / interface `120100` capture records `PlayerFrame.noPortraitMode` as `HIGH`, raw frame level `2`, raised frame level `0`; `PlayerSpellsFrame` is `MEDIUM`, raw level `1`, raised level `18→20`, and its `SpellBookFrame` shares the raised `18→20` state. Relevant frames report `HasFixedFrameStrata() == false`. This confirms the original native `HIGH`/`0` versus `MEDIUM`/`18→20` observation, but does not establish a general render-order policy or global z-index interpretation for raised levels.

## Controlled native comparison

[UnitFrameLayerProbe](../../addons/UnitFrameLayerProbe/README.md) adds five opaque red/blue comparisons across three phases: a HIGH child under LOW parent, independent HIGH, independent DIALOG, plain TOOLTIP, and owned `GameTooltip` versus MEDIUM. Screenshot-success gating, a 1.1-second interphase delay, cancellation/timeout cleanup, late-event isolation, and bounded control history define the capture protocol; development protocol coverage is 9/9. Native screenshots—not simulator protocol tests—will determine each overlap result.

Commits `e5f4a96ae` and `e811240d0` installed the unchanged five-file control kit to desktop retail main. `/tmp/pi-unit-controls-desktop-install.json` records an exact 5/5 SHA-256 match. Native capture is pending. Independent protocol verification passed: retained 9/9 tests, formatting/syntax, read-only boundaries, and 5/5 installed hashes (`/tmp/pi-unit-controls-verification.md`). Native `cleanupErrors` still require inspection after the run. No simulator ordering change or overlap fix is claimed.

## Sources

- `BetterBlizzFrames/retail/modules/noPortrait.lua:548-550`
- `/tmp/pi-unit-overlap-evidence.md`
- `/tmp/pi-clipboard-a6471921-93e0-4db8-ba3c-b3a872ce6d2b.png`
- `docs/local/private/probes/UnitFrameLayerProbe-2026-09-08.lua` — original raw native capture
- `/tmp/pi-unit-controls-desktop-install.json` — controlled-kit desktop hash record
- [UnitFrameLayerProbe](../../addons/UnitFrameLayerProbe/README.md) — controlled capture protocol

## See Also

- [[rendering-pipeline]]
- [[playerspells-runtime-load]]
