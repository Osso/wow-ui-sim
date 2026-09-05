# PTR pixel-rounding probe

PTR `12.1.5.69594` exposes `SetRoundLayoutToNearestPixel` and `GetRoundLayoutToNearestPixel`, but source alone does not establish whether native layout rounds anchor offsets, final edges, or another intermediate. The simulator does not implement these methods yet.

## Capture status

Commits `e958786cb`, `15ca6bd2c`, and `5a5b7a187` add and correct `PixelRoundingProbe`, a read-only addon for the running desktop PTR client. It creates only unnamed hidden frames and regions; it does not change Blizzard frames, CVars, Edit Mode, visibility, or saved layout settings.

The probe records build/interface identity, physical display data, UIParent and case-parent geometry, raw points, `GetSize`/width/height/rectangles, scales, native setter/getter results, and per-operation errors. It covers fractional bottom-left and center anchors, two-anchor stretch, negative offsets, object and parent scales, setter timing, toggling, and texture/FontString regions.

The settled sample re-reads the same hidden objects on the next timer tick. Repeated runs reuse the case pool. This prevents a delayed sample from measuring newly-created frames instead of post-layout state. The protocol test reproduces the old recreation defect and verifies the correction without asserting native geometry.

Commit `5a5b7a187` replaced invented one-value physical-width/height globals with WoW's documented two-result `GetPhysicalScreenSize()`. An actual VM protocol probe first failed with only that documented API exposed, then passed after the correction; this validates the capture interface, not native rounding geometry.

The addon is staged at `C:\World of Warcraft\_xptr_\Interface\AddOns\PixelRoundingProbe`. Local and staged SHA-256 values match for `PixelRoundingProbe.lua` (`dd4066abad6849d99185879239754192bf405602983736b368a7e7d8640dd53a`) and `.toc` (`cfcd6f3fcb2990c29179115c78cf3dc39e6d767c312a7f359edac59455bfc86c`). The active desktop executable is `C:\World of Warcraft\_xptr_\WowT.exe`; its `.build.info` has `wowxptr` version `12.1.5.69594`, build key `4a9973f37906f8cfb344f8a9fe6777e0`.

## Passive bootstrap observations

Every sample now reads, without loading or invoking either addon, the `loaded` and `finished` results from `C_AddOns.IsAddOnLoaded` for `Blizzard_ClickBindingUI` and `Blizzard_Collections`. It also records the types of the bootstrap-defined `InClickBindingMode` and the Collections toggle entry point `ToggleCollectionsJournal`.

This accompanies, but does not resolve, PTR panel-smoke evidence: the reported `GetBuildInfo()` identity is `12.1.5` / `69594` / `120105`; Spellbook encountered a nil `InClickBindingMode`; and Collections changed visibility through its sequential normal-toggle path. The capture must not be used to infer native LoadOnDemand behavior. In particular, it authorizes neither a separate bootstrap pass nor any departure from the July 1 finding that `[Bootstrap]` TOC entries execute in normal TOC order.

## Unresolved native behavior

PTR `PixelUtil.lua` gives the UI-unit-to-physical-pixel conversion used by its deprecated helpers. The newer native flag replaces those helpers, but its source contract only declares a boolean setter/getter. It does not establish the rounding point in anchor resolution, whether regions follow identical layout rules, or scale-change behavior.

Do not infer native geometry until a capture has `matchesExpectedBuild = true`, is flushed through `/reload` or logout, and is retrieved from PTR SavedVariables. Preserve raw values; do not turn the probe's cases into simulator expectations before comparing them.

## Sources

- [PixelRoundingProbe](../../addons/PixelRoundingProbe/README.md) — capture protocol and staging target
- [pixel_rounding_probe.rs](../../../tests/pixel_rounding_probe.rs) — same-object settled-capture and read-only-state regression coverage
- `~/.cache/wow-ui-sim/blizzard-ui/ptr/AddOns/Blizzard_SharedXML/PixelUtil.lua` — source pixel conversion and recursive native-flag caller
- `~/.cache/wow-ui-sim/blizzard-ui/ptr/AddOns/Blizzard_APIDocumentationGenerated/SimpleScriptRegionAPIDocumentation.lua` — native setter/getter declarations
- `/tmp/pi-pixel-protocol-{red,green}.*` — reproduction and corrected settled-capture protocol
- `/tmp/pi-pixel-probe-installed-hashes.*` — earlier staging hash comparison
- `/tmp/pi-pixel-physical-{red,green}.*` — documented physical-screen API RED/GREEN
- `/tmp/pi-pixel-probe-physical-installed-hashes.stdout.log` — earlier corrected staged hash comparison
- `/tmp/pi-pixel-bootstrap-installed-hashes.stdout.log` — current passive-bootstrap probe staging hashes

## See Also

- [[layout-system]] — anchor resolution and display coordinate model
- [[display-size-ui-scale-events]] — live display-scale evidence workflow
- [[frame-position-baseline-drift]] — earlier read-only live-layout capture boundary
