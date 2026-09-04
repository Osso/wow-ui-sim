# FramePositionProbe

Read-only live-client evidence for [frame-position baseline drift](../../wiki/investigations/frame-position-baseline-drift.md). It observes Blizzard frames without moving, resizing, showing, hiding, or changing Edit Mode layouts.

## Capture

1. Install with `./deploy.sh` from this directory. Destination: `desktop:C:/World of Warcraft/_retail_/Interface/AddOns/FramePositionProbe`.
2. Enable **FramePositionProbe**, log in, and wait at least five seconds after entering the world.
3. While out of combat with no raid warnings or deadly debuffs, run `/fpprobe` for a manual sample. Do not change resolution or layout merely to obtain a passing comparison.
4. Run `/reload` or log out to flush SavedVariables.
5. Retrieve `_retail_/WTF/Account/<ACCOUNT>/SavedVariables/FramePositionProbe.lua` before another logout/reload replaces that capture.

The addon starts a new capture at each load. The file flushed by `/reload` contains the preceding session; its new in-memory capture does not replace that file until another flush.

## Evidence and comparison

Captures include build/interface, physical and UI screen dimensions, scale/CVars, active Edit Mode information, raw frame rectangles and anchors, parent names, warning/debuff and combat state, player class, and loaded non-Blizzard addons. Edit Mode capture includes the active layout index and selected RaidWarning/ObjectiveTracker system anchors. Samples cover login, world entry, delayed settling, and the manual command. Missing frames use `present=false`; secret values use `<secret>` with an error marker; inaccessible reads retain explicit errors. These are not real zero/nil values.

`GetRect()` uses **left/bottom/width/height in UI coordinates**, not screenshot top-left pixels. Preserve these raw values; interpret screen conversion and effective scale before comparison.

The failing simulator fixture uses **1600×1200**, observed effective scale **1**, normal default startup, no imported SavedVariables/EditMode cache, no active raid-warning message, and a hidden `DeadlyDebuffFrame`. Its cached retail source is **12.1.0.69497**. The desktop executable was **12.1.0.69587** when checked on September 4, 2026; the running client's `GetBuildInfo()` is authoritative for the capture.

A different build, active layout, scale, warning state, startup boundary, or third-party UI override is **not matching parity evidence**. Capture existing state first and resolve differences explicitly. Repeated simulator coordinates alone do not justify changing either regression assertion.
