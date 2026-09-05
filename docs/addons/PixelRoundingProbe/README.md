# PixelRoundingProbe

Read-only evidence capture for PTR `12.1.5.69594` pixel-layout behavior. It creates only unnamed, hidden frames and regions; it does not change Blizzard frames, CVars, Edit Mode, visibility, or layout settings.

## Capture

1. Stage with `./deploy.sh` from this directory. It targets `desktop:C:/World of Warcraft/_xptr_/Interface/AddOns/PixelRoundingProbe`.
2. Enable **PixelRoundingProbe** in the PTR client, enter the world out of combat, then wait one frame.
3. Run `/pixelprobe` for a manual next-tick plus settled capture.
4. Run `/reload` or log out to flush SavedVariables.
5. Retrieve `_xptr_/WTF/Account/<ACCOUNT>/SavedVariables/PixelRoundingProbe.lua`.

The settled sample resamples the same hidden objects on the following timer tick; repeated runs reuse the bounded case pool. Each capture records build/interface identity, physical display dimensions from `GetPhysicalScreenSize`, UIParent and per-case parent geometry, `GetSize`/width/height, points/rectangles/scales, native rounding getter/setter results, and per-operation errors. Anonymous anchor targets use stable case labels. Cases distinguish fractional bottom-left and center anchors, two-anchor stretch, small negative offsets, object and parent scaling, toggling before/after geometry assignment, and texture/font-string behavior.

Each sample also passively records `C_AddOns.IsAddOnLoaded`'s `loaded` and `finished` values for `Blizzard_ClickBindingUI` and `Blizzard_Collections`, plus the types of `InClickBindingMode` and `ToggleCollectionsJournal`. It never loads either addon or calls either panel helper. This distinguishes observed bootstrap state from the PTR panel-smoke symptoms: Spellbook can encounter missing `InClickBindingMode`, while Collections can change visibility after its normal toggle path. The observations do not establish native LoadOnDemand semantics or justify a bootstrap pass, TOC reordering, or eager loading.

The staged Lua SHA-256 is `dd4066abad6849d99185879239754192bf405602983736b368a7e7d8640dd53a`; the staged TOC SHA-256 is `cfcd6f3fcb2990c29179115c78cf3dc39e6d767c312a7f359edac59455bfc86c`. No PTR SavedVariables capture exists yet.

Do not infer behavior from an addon that reports a different build: `matchesExpectedBuild` must be true for `69594`. Preserve raw values; the probe intentionally does not calculate expected rounded geometry.
