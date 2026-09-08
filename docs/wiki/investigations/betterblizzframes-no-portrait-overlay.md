# BetterBlizzFrames no-portrait overlay

## Symptoms

`Uther` rendered over SpellBook in the supplied screenshot.

## Observed state

Saved configuration enables `BetterBlizzFramesDB.noPortraitModes`. BetterBlizzFrames creates `PlayerFrame.noPortraitMode`, parents it to `PlayerFrame`, and explicitly sets its frame strata to `HIGH`. SpellBook is `MEDIUM`; the observed draw order follows those strata values. This identifies the overlapping frame, not the commit that introduced the reported regression or a verified native-client comparison.

## Non-causes

Mask correction `939efe88d` changed alpha sampling only. It did not change frame strata, levels, parentage, or render order.

The last known-good state is unknown. No overlap correction or addon-setting change has been applied.

## Sources

- `BetterBlizzFrames/retail/modules/noPortrait.lua:548-550`
- `/tmp/pi-unit-overlap-evidence.md`
- `/tmp/pi-clipboard-a6471921-93e0-4db8-ba3c-b3a872ce6d2b.png`

## See Also

- [[rendering-pipeline]]
- [[playerspells-runtime-load]]
