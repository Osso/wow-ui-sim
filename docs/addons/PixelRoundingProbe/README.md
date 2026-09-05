# PixelRoundingProbe

Read-only evidence capture for PTR `12.1.5.69594` pixel-layout behavior. It creates only unnamed, hidden frames and regions; it does not change Blizzard frames, CVars, Edit Mode, visibility, or layout settings.

## Capture

1. Stage with `./deploy.sh` from this directory. It targets `desktop:C:/World of Warcraft/_xptr_/Interface/AddOns/PixelRoundingProbe`.
2. Enable **PixelRoundingProbe** in the PTR client, enter the world out of combat, then wait one frame.
3. Run `/pixelprobe` for a manual next-tick plus settled capture.
4. Run `/reload` or log out to flush SavedVariables.
5. Retrieve `_xptr_/WTF/Account/<ACCOUNT>/SavedVariables/PixelRoundingProbe.lua`.

The settled sample resamples the same hidden objects on the following timer tick; repeated runs reuse the bounded case pool. Each capture records build/interface identity, physical display dimensions from `GetPhysicalScreenSize`, UIParent and per-case parent geometry, `GetSize`/width/height, points/rectangles/scales, native rounding getter/setter results, and per-operation errors. Anonymous anchor targets use stable case labels. Cases distinguish fractional bottom-left and center anchors, two-anchor stretch, small negative offsets, object and parent scaling, toggling before/after geometry assignment, and texture/font-string behavior.

The capture also reads Click Binding/Collections addon load state and bootstrap-function presence. It does not load either addon; these observations help diagnose the spellbook's missing `InClickBindingMode` without assuming different TOC ordering.

Do not infer behavior from an addon that reports a different build: `matchesExpectedBuild` must be true for `69594`. Preserve raw values; the probe intentionally does not calculate expected rounded geometry.
