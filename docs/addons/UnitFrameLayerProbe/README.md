# UnitFrameLayerProbe

Read-only capture of unit-frame/SpellBook layering. It records raw getter results;
it does not decide which frame should render above another.

## Installation and capture

This directory is a probe kit, **not a deployment**. No client installation or
live capture is performed by the repository tests.

1. Copy this directory to the intended client's `Interface/AddOns/UnitFrameLayerProbe/`.
2. Enable **UnitFrameLayerProbe** on that client's AddOns screen, retaining the addon
   settings and window layout that exhibit the overlap. Enter the world.
3. Open SpellBook normally and wait one tick. Its OnShow capture is automatic.
   Run `/unitlayerprobe` for an explicit manual capture while the overlap is visible.
4. Close SpellBook normally and wait one tick for its OnHide capture. Reopen and
   capture again if comparing states; the probe never opens or closes it itself.
5. Run `/reload` (or log out) to flush `UnitFrameLayerProbeDB` to that client's
   `WTF/Account/<ACCOUNT>/SavedVariables/UnitFrameLayerProbe.lua`. Preserve that file
   before further sessions. Logging in again appends; it does not reset history.

The TOC lists retail `120100` and PTR `120105` using the comma-separated Interface
convention used by installed addons (for example Baganator) and supported by
`src/toc/tests.rs::test_multiple_interface_versions`. Select the intended client;
this does not claim compatibility with every build or other client profiles.

## Captures

- `PLAYER_LOGIN`, deferred by `C_Timer.After(0, ...)`.
- Manual `/unitlayerprobe`, captured immediately.
- `PlayerSpellsFrame.SpellBookFrame` OnShow/OnHide, deferred zero ticks to observe
  settled state. Hooks are installed once per frame when Blizzard_PlayerSpells
  loads, or when it was already loaded. Existing script handlers are retained.

SavedVariables contain schema version `1`, probe version, monotonic sequence,
server timestamp and uptime observations. Captures append until **30**; thereafter
attempts increment `skippedCaptures` and print a limit message without replacing
old captures. Reload does not reset the cap or sequence. To start a separate run,
first archive the SavedVariables file, then remove it while the client is exited.
Unsupported saved schemas are reported and left untouched, not migrated/reset.

### Observed objects

- `UIParent`, `PlayerFrame`, `PlayerFrame.noPortraitMode`, `PlayerFrame.bbfName`.
- `PlayerFrame.noPortraitMode.Texture` (no-portrait border artwork).
- PlayerFrame main health/mana `BBFPixelBorder` frames, if present.
- `PlayerSpellsFrame`, its `SpellBookFrame` child, and the global `SpellBookFrame`
  alias if present (absence is recorded, not substituted).

For each object and its parent chain (up to 16 ancestors): name/type, strata,
frame/raised levels, fixed-strata/level flags, toplevel, shown/visible state,
alpha/effective alpha, scale/effective scale, rect, draw layer and parent.
Missing frames/methods, lookup/call errors, and chain truncation/cycles are explicit.
No parent value is substituted for an unsupported child getter.

Global observations include GetBuildInfo, physical screen size, logical screen
width/height, and UIParent's own metrics above. Addon loaded/version observations
cover BetterBlizzFrames, ClickableRaidBuffs, DandersFrames, BlizzMove,
EnhanceQoLMover and Blizzard_PlayerSpells. The only settings read are boolean
`BetterBlizzFramesDB.noPortraitModes` and `noPortraitPixelBorder`; non-booleans
are marked without retaining their values. No unit text, roster or other settings
are collected.

### Result representation

Calls have `status = "ok"`, `"missing"` or `"error"`. Successful calls retain
`values.n` and one typed descriptor per return slot, including nil and false.
Primitive values are stored literally; frame/object references become session-local
identity strings, not live SavedVariables objects. Secret values are marked without
exposing them; nonfinite numbers are recorded as text. Errors retain their messages.
Object identity strings can correlate captures within one session, not across reloads.

## Read-only scope and tests

The probe creates its own event listener and adds observation hooks only. It never
forces LoadAddOn, shows/hides, moves, raises, reparents, sets strata/levels, or changes
settings on observed UI objects. It does not reset data on login or auto-overwrite
completed captures.

`tests/unit_frame_layer_probe.rs` runs in the existing grouped `integration` target.
It covers late/already-loaded hooks, manual/login capture, unchanged observed frame
properties, optional getter errors/nil values, serialized fresh-VM reload retention
and the capture cap. These are simulator protocol tests, **not native layering
measurements**. Main integration owns final checks and any later deployment.
