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

## Bootstrap timing capture

The ignored capture `docs/local/private/probes/BootstrapOrderProbe-2026-09-07.lua` has raw SHA-256 `7f8fe830a9937bb7d4b1989b3097661cf3c8492bd11607f2cc8d3a3b1a8d34b1`; desktop and local copies matched. Its nine records identify `12.1.5` / `69594` / `120105`, all match the expected build, and contain no probe errors.

Observed startup sequence: `A:eager`, `B:bootstrap`, `C:eager`, `D:before`, `D:bootstrap`, `D:after`, `PLAYER_LOGIN`, then two snapshots. This establishes that the tested LoD bootstrap executes during startup while B is otherwise unloaded, and that the eager D TOC preserves its literal `Before.lua → Bootstrap.lua [Bootstrap] → Normal.lua` order. In particular, this capture does not support a global bootstrap pre-pass before eager A.

B reports `loaded=true, finished=false` only while its bootstrap executes; it is `false,false` in every later record. ClickBinding and Collections are `false,false` in every record while both helper globals are functions. This confirms startup bootstrap publication for the tested LoD shape, but does not determine the full eligibility/dependency rules for Blizzard addons.

The requested `/boprobe load` calls were absent: there are no B `before`/`after` records or `load:before`/`load:after` brackets. Explicit-load file order and whether a bootstrap file re-executes remain unknown. The capture does not authorize a generic bootstrap pass or change ordinary TOC-order behavior.

## Derived bounded layout rule

PTR `PixelUtil.lua` says the native flag automatically applies the same adjustment as its deprecated `SetSize` and `SetPoint` helpers. Those helpers round each requested size or offset with quantum `768 / (physical display height × effective region scale)`.

A main-process replay checked all 66 nonempty captured `after` rectangles (264 coordinates). It rounded requested sizes and offsets at that quantum, converted relative targets with the parent/object effective-scale ratio, retained raw anchor values, and did not re-round stretch-derived dimensions. The maximum residual was `0.00002595186236931113` UI units. The captured rounded left edge `123.375` corresponds to `154.21875` physical pixels in the tested conversion, ruling out global final-edge snapping for that example.

Commit `07cce4a63` stores a per-region false-default flag, exposes the epoch-gated native methods, applies that bounded rule only during shared layout resolution, and caches the physical conversion in `WidgetRegistry`, updated by the existing display setter. It does not rewrite raw sizes or anchors. The grouped PTR replay passed all five tests: the two source-derived live-layout cases plus three existing probe-protocol cases.

Default-profile preservation verification at `fd6256059` passed 18 grouped tests (three probe, nine display, six layout) and the previously blocked client-info test (1/1). Default `cargo check` completed with zero warnings; formatting proof remains valid. Logs: `/tmp/pi-ptr125-default-regressions.{integration,client-info}.*` and `/tmp/pi-ptr125-round-layout-default-check.*`. These are bounded regressions, not whole-profile acceptance.

The capture meets the required build and flush/retrieval boundary. It does not prove visible rendering, hit testing, clipping, animation, all widget types, tie behavior, or arbitrary layout graphs.

## Sources

- [PixelRoundingProbe](../../addons/PixelRoundingProbe/README.md) — rounding capture protocol and staging target
- [BootstrapOrderProbe](../../addons/BootstrapOrderProbe_A/README.md) — pending bootstrap timing/re-execution capture protocol
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
