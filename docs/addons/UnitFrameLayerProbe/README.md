# UnitFrameLayerProbe

Read-only observations of existing unit-frame/SpellBook layering, plus optional
owned comparison fixtures. It records raw getter results and screenshots;
it does not decide which color should render above another.

## Installation and capture

Repository tests do not install the probe or perform native captures.
Run `python3 docs/addons/UnitFrameLayerProbe/deploy.sh` from the repository root
to install through the `desktop` SSH alias into
`C:/World of Warcraft/_retail_/Interface/AddOns/UnitFrameLayerProbe/`.
The installer does not enable the addon or change game settings.

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

## Controlled comparison: `/unitlayerprobe test`

Close SpellBook and other menus first. The probe does not close them for you.
Run `/unitlayerprobe test`; do not move or interact with the comparison rectangles
while the three screenshots are being taken. `/unitlayerprobe cancel` stops a run.

Five labeled comparisons use overlapping opaque red/blue rectangles:

1. LOW top-level parent with a HIGH red child, versus a MEDIUM blue top-level panel.
2. Independent HIGH red frame versus MEDIUM blue panel.
3. Independent DIALOG red frame versus MEDIUM blue panel.
4. Independent plain TOOLTIP red frame versus MEDIUM blue panel.
5. An owned named `GameTooltip` using `GameTooltipTemplate`, owned by a MEDIUM blue
   panel (`ANCHOR_NONE`), versus that panel. It has short text and red artwork;
   owner identity is captured, and it is shown again after panel hide/show.

Roots and labels attach directly to UIParent except the intentional LOW-parent
case. There is no shared high-strata stage. Existing GameTooltip, SpellBook,
unit frames, menus and settings are never changed.

Each comparison is observed in three phases: **created** (actual getter values,
not assumed unraised), **panel-hide-show**, and **panel-raise**. Each phase records
raw frame/raised levels, strata, fixed/toplevel flags, parent chains, geometry,
alpha/scale, build and screenshot request/completion times. The on-screen header
contains the run number, phase and server timestamp. Native screenshots, not the
protocol tests, establish which color wins the overlap.

The next phase waits for `SCREENSHOT_SUCCEEDED`, then at least 1.1 seconds to avoid
native screenshot filename collisions. `Screenshot()` receives no filename;
collect the three files from the client's normal `Screenshots` directory and
match their visible headers to `UnitFrameLayerProbeDB.controlRuns`. Run `/reload`
after completion to flush the data. Screenshot failure, a 10-second timeout,
cancel, or a setup/call error hides, clears anchors and detaches owned fixtures.
Native frames cannot be destroyed: hidden allocations remain until UI reload.

Control history is separate from original observations and limited to **three
runs** (three phases each), including failed/cancelled attempts. Once capped,
no more fixtures are allocated; archive/remove the SavedVariables file with the
client exited before starting a new series. A screenshot outstanding at cancel
or timeout blocks a new run until its completion/failure event arrives, or UI
reload; events have no request IDs, so late events must not advance another run.

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

The original observation mode creates its own event listener and observation
hooks. Neither mode forces LoadAddOn, shows/hides, moves, raises, reparents, sets
strata/levels, or changes settings on existing observed UI objects. Control mode
configures and later hides/detaches only its own fixtures. It does not reset data on login or auto-overwrite
completed captures.

`tests/unit_frame_layer_probe.rs` runs in the existing grouped `integration` target.
It covers late/already-loaded hooks, manual/login capture, unchanged observed frame
properties, optional getter errors/nil values, serialized fresh-VM reload retention
and the capture cap. Controlled tests load the actual TOC files with one shared
addon namespace and mock screenshot events/timers. They assert phase order, raw
observations, independent parenting, cleanup on success/failure/timeout/cancel,
late-event isolation and bounded/history-preserving behavior—not native color
ordering. These are simulator protocol tests, **not native layering
measurements**. Main integration owns final checks and any later deployment.
