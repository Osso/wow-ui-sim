# PTR pixel-rounding probe

PTR `12.1.5.69594` exposes `SetRoundLayoutToNearestPixel` and `GetRoundLayoutToNearestPixel`. Commit `07cce4a63` models the bounded source-and-capture-backed layout rule; replay and independent verification remain pending.

## Capture status

Commits `e958786cb`, `15ca6bd2c`, and `5a5b7a187` add and correct `PixelRoundingProbe`, a read-only addon for the running desktop PTR client. It creates only unnamed hidden frames and regions; it does not change Blizzard frames, CVars, Edit Mode, visibility, or saved layout settings.

The probe records build/interface identity, physical display data, UIParent and case-parent geometry, raw points, `GetSize`/width/height/rectangles, scales, native setter/getter results, and per-operation errors. It covers fractional bottom-left and center anchors, two-anchor stretch, negative offsets, object and parent scales, setter timing, toggling, and texture/FontString regions.

The settled sample re-reads the same hidden objects on the next timer tick. Repeated runs reuse the case pool. This prevents a delayed sample from measuring newly-created frames instead of post-layout state. The protocol test reproduces the old recreation defect and verifies the correction without asserting native geometry.

Commit `5a5b7a187` replaced invented one-value physical-width/height globals with WoW's documented two-result `GetPhysicalScreenSize()`. An actual VM protocol probe first failed with only that documented API exposed, then passed after the correction; this validates the capture interface, not native rounding geometry.

The addon is staged at `C:\World of Warcraft\_xptr_\Interface\AddOns\PixelRoundingProbe`. Local and staged SHA-256 values match for `PixelRoundingProbe.lua` (`dd4066abad6849d99185879239754192bf405602983736b368a7e7d8640dd53a`) and `.toc` (`cfcd6f3fcb2990c29179115c78cf3dc39e6d767c312a7f359edac59455bfc86c`). The active desktop executable is `C:\World of Warcraft\_xptr_\WowT.exe`; its `.build.info` has `wowxptr` version `12.1.5.69594`, build key `4a9973f37906f8cfb344f8a9fe6777e0`.

## Received capture

The ignored private capture files are `docs/local/private/probes/PixelRoundingProbe-2026-09-06.lua` and `.json`. The raw SavedVariables Lua hash is `1383e92e76920adf718e0beafbeeff55fe1dc78134735f66177b73374a3a23bf`; local and desktop copies matched.

All six samples report `12.1.5` / `69594` / `120105`, `matchesExpectedBuild=true`, and physical `3440×1440`. Each has 12 cases and no captured operation errors. The getter defaults to `false`; enabling the flag changes measured geometry and disabling it restores the measured unrounded geometry. Raw point offsets remain unchanged. The immediate and settled samples match for these captured cases; this is not proof for untested layouts.

The region cases show independently queryable/settable flags on Texture and FontString objects. Their captured geometry confirms that the flag is not state-only, but does not establish behavior for all widget types or every scale/layout combination.

## Passive bootstrap observations

Every sample now reads, without loading or invoking either addon, the `loaded` and `finished` results from `C_AddOns.IsAddOnLoaded` for `Blizzard_ClickBindingUI` and `Blizzard_Collections`. It also records the types of the bootstrap-defined `InClickBindingMode` and the Collections toggle entry point `ToggleCollectionsJournal`.

In every received sample, `InClickBindingMode` and `ToggleCollectionsJournal` are functions while `Blizzard_ClickBindingUI` and `Blizzard_Collections` respectively report `loaded=false`, `finished=false`. The user reports the PTR ran without personal addons; the probe does not inventory addons, so that report is supporting context rather than a capture field. These observations do not establish native LoadOnDemand semantics or explain the earlier Spellbook/Collections behavior. In particular, they authorize neither a separate bootstrap pass nor any departure from the July 1 finding that `[Bootstrap]` entries execute in normal TOC order.

## Derived bounded layout rule

PTR `PixelUtil.lua` says the native flag automatically applies the same adjustment as its deprecated `SetSize` and `SetPoint` helpers. Those helpers round each requested size or offset with quantum `768 / (physical display height × effective region scale)`.

A main-process replay checked all 66 nonempty captured `after` rectangles (264 coordinates). It rounded requested sizes and offsets at that quantum, converted relative targets with the parent/object effective-scale ratio, retained raw anchor values, and did not re-round stretch-derived dimensions. The maximum residual was `0.00002595186236931113` UI units. The captured rounded left edge `123.375` corresponds to `154.21875` physical pixels in the tested conversion, ruling out global final-edge snapping for that example.

Commit `07cce4a63` stores a per-region false-default flag, exposes the epoch-gated native methods, applies that bounded rule only during shared layout resolution, and caches the physical conversion in `WidgetRegistry`, updated by the existing display setter. It does not rewrite raw sizes or anchors. The grouped PTR replay passed all five tests: the two source-derived live-layout cases plus three existing probe-protocol cases.

The capture meets the required build and flush/retrieval boundary. It does not prove visible rendering, hit testing, clipping, animation, all widget types, tie behavior, or arbitrary layout graphs.

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
- `docs/local/private/probes/PixelRoundingProbe-2026-09-06.{lua,json}` — ignored received raw capture and parsed representation

## See Also

- [[layout-system]] — anchor resolution and display coordinate model
- [[display-size-ui-scale-events]] — live display-scale evidence workflow
- [[frame-position-baseline-drift]] — earlier read-only live-layout capture boundary
