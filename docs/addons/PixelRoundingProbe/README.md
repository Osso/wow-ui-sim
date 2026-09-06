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

The received PTR capture is stored in ignored private files `docs/local/private/probes/PixelRoundingProbe-2026-09-06.{lua,json}`. The raw SavedVariables Lua SHA-256 is `1383e92e76920adf718e0beafbeeff55fe1dc78134735f66177b73374a3a23bf`, verified equal locally and on the desktop. Staged Lua and TOC hashes remain `dd4066abad6849d99185879239754192bf405602983736b368a7e7d8640dd53a` and `cfcd6f3fcb2990c29179115c78cf3dc39e6d767c312a7f359edac59455bfc86c`.

The capture contains six samples with 12 cases each: `12.1.5` / `69594` / `120105`, physical display `3440×1440`, and no recorded operation errors. Flags default `false`; enabling and disabling them changes and restores measured geometry while raw anchor values remain unchanged. Immediate and settled samples match for the captured cases only. `InClickBindingMode` and `ToggleCollectionsJournal` are functions while their respective addons report `loaded=false`, `finished=false`. The user reports no personal addons were running; the probe does not inventory addons, so that report is context rather than capture evidence.

Do not infer behavior from an addon that reports a different build: `matchesExpectedBuild` must be true for `69594`. Preserve raw values; the probe intentionally does not calculate expected rounded geometry. The rounding point and LoadOnDemand/bootstrap ordering remain under investigation.
