# PTR pixel-rounding probe

PTR `12.1.5.69594` exposes `SetRoundLayoutToNearestPixel` and `GetRoundLayoutToNearestPixel`, but source alone does not establish whether native layout rounds anchor offsets, final edges, or another intermediate. The simulator does not implement these methods yet.

## Capture status

Commits `e958786cb`, `15ca6bd2c`, and `5a5b7a187` add and correct `PixelRoundingProbe`, a read-only addon for the running desktop PTR client. It creates only unnamed hidden frames and regions; it does not change Blizzard frames, CVars, Edit Mode, visibility, or saved layout settings.

The probe records build/interface identity, physical display data, UIParent and case-parent geometry, raw points, `GetSize`/width/height/rectangles, scales, native setter/getter results, and per-operation errors. It covers fractional bottom-left and center anchors, two-anchor stretch, negative offsets, object and parent scales, setter timing, toggling, and texture/FontString regions.

The settled sample re-reads the same hidden objects on the next timer tick. Repeated runs reuse the case pool. This prevents a delayed sample from measuring newly-created frames instead of post-layout state. The protocol test reproduces the old recreation defect and verifies the correction without asserting native geometry.

Commit `5a5b7a187` replaced invented one-value physical-width/height globals with WoW's documented two-result `GetPhysicalScreenSize()`. An actual VM protocol probe first failed with only that documented API exposed, then passed after the correction; this validates the capture interface, not native rounding geometry.

The addon is staged at `C:\World of Warcraft\_xptr_\Interface\AddOns\PixelRoundingProbe`. Local and staged SHA-256 values match for `PixelRoundingProbe.lua` (`e9cb08416dfc4b859c67ab31f0c938dd5016c5f9e1b06cebc9679d09c408e44b`) and `.toc` (`cfcd6f3fcb2990c29179115c78cf3dc39e6d767c312a7f359edac59455bfc86c`). The active desktop executable is `C:\World of Warcraft\_xptr_\WowT.exe`; its `.build.info` has `wowxptr` version `12.1.5.69594`, build key `4a9973f37906f8cfb344f8a9fe6777e0`.

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
- `/tmp/pi-pixel-probe-physical-installed-hashes.stdout.log` — corrected staged hash comparison

## See Also

- [[layout-system]] — anchor resolution and display coordinate model
- [[display-size-ui-scale-events]] — live display-scale evidence workflow
- [[frame-position-baseline-drift]] — earlier read-only live-layout capture boundary
