# Captured EditMode input

`edit-mode-cache-account.txt` contains the account EditMode configuration used by the September 4, 2026 FramePositionProbe capture on retail 12.1.0.69587. It contains layout names and numeric settings, not account or character identifiers. The original file's SHA-256 is `44816f8b6b952ce349fa224db2ab888019b10fe08a072877f0894246774b999f`.

For a text-reviewable fixture, the terminal NUL is represented by a newline. The replay restores that NUL before passing the configuration through the existing cache decoder. `Ultrawide` is selected by name. This is a configuration input, not a frame-rectangle snapshot assigned to the simulator.

`tests/frame_position_replay.rs` supplies physical display size 3440×1440 and the captured UI-scale CVars, loads Blizzard UI, and runs startup. Expected raw rectangles/anchors come from the live probe and are asserted after startup, using the existing one-UI-unit geometry tolerance. No third-party addon is needed to reproduce the selected frames' captured values in this case; no claim is made about unrelated addon behavior or the rest of the UI.

See [the investigation](../../../docs/wiki/investigations/frame-position-baseline-drift.md) for capture provenance and limits.
