# Profile frame-position baselines

`tests/frame_positions.rs` checks the profile-selected Blizzard UI on a 1600×1200 default canvas. Native-source derivations and remaining model boundaries belong in [frame-position baseline drift](../wiki/investigations/frame-position-baseline-drift.md).

## What it must do

- [x] Keep all 28 case names and the one-pixel geometry and 0.02 alpha tolerances.
- [x] Use profile-native minimap dimensions, header extents and preset anchors, not the Retail rectangle for every profile.
- [x] Assert the default minimap parent, dimensions, scale and anchors before comparing rectangles.
- [x] Use native frame names and test absence when the selected profile source excludes a frame.
- [x] Change a baseline only with a native XML/Lua derivation; retain failures caused by missing or incorrect simulator inputs.
- [x] Keep Retail clean and reduce failure names on PTR, Forever, Mists, Era and Anniversary relative to the lane's `origin/master` baseline.

## How it works

- [Source derivations and evidence boundary](../wiki/investigations/frame-position-baseline-drift.md)

## Implementation inventory

- `tests/frame_positions.rs`: shared startup fixture, named geometry/alpha assertions and profile-native input checks.

## Tests asserting this spec

- `tests/frame_positions.rs`: custom `frame_positions` harness; parse `test NAME ... FAILED`, not a libtest failures block.

## Known gaps (current cycle)

- [x] Reconcile source-proven stale expectations without accepting zero-anchor or placeholder geometry as native output.
- [x] Record residual per-profile failures and native-input questions in the untracked lane report.
- [ ] Keep Classic bags and chat rectangles failing until native startup inputs are established; do not use simulator-observed rectangles.

## Out of scope

Wrath, tolerance increases, vendor edits, guessed Classic startup inputs, unrelated integration failures and claims of whole-client native parity.
