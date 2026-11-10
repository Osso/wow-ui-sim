# Profile frame-position baselines

`tests/frame_positions.rs` checks the profile-selected Blizzard UI on a 1600×1200 default canvas. Native-source derivations and remaining model boundaries belong in [frame-position baseline drift](../wiki/investigations/frame-position-baseline-drift.md).

## What it must do

- [x] Keep all 28 case names and the one-pixel geometry and 0.02 alpha tolerances.
- [x] Use profile-native minimap dimensions, header extents and preset anchors, not the Retail rectangle for every profile.
- [x] Assert the default minimap parent, dimensions, scale and anchors before comparing rectangles.
- [x] Use native frame names and test absence when a profile's selected TOC excludes the Retail-only frame.
- [x] Change a baseline only with a native XML/Lua derivation; retain failures caused by missing or incorrect simulator inputs.
- [x] Assert Mists player/target/focus Edit Mode anchor inputs and source-derived Party, CompactParty and PaladinPowerBar rectangles.
- [x] Derive Mists ChatFrame1's managed bottom anchor from the selected Edit Mode anchor and visible managed-frame inputs.
- [x] Keep Retail clean and reduce failure names on PTR, Forever, Mists, Era and Anniversary relative to master `3ca96ba58`.
- [x] Reconcile Forever bag, status-manager and managed-right geometry from pinned Camelot inputs; assert bag registration/dimensions/preset, keyboard-manager anchor, and the visible managed-child width bound before geometry. Do not treat the status manager as its Edit Mode child bars.

## How it works

- [Source derivations and evidence boundary](../wiki/investigations/frame-position-baseline-drift.md)

## Implementation inventory

- `tests/frame_positions.rs`: shared startup fixture, named geometry/alpha assertions and profile-native input checks.

## Tests asserting this spec

- `tests/frame_positions.rs`: custom `frame_positions` harness; parse `test NAME ... FAILED`, not a libtest failures block.

## Known gaps (current cycle)

- [x] Reconcile source-proven stale expectations without accepting zero-anchor or placeholder geometry as native output.
- [x] Record residual per-profile failures and native-input questions in the untracked lane report.

## Out of scope

Wrath, tolerance increases, vendor edits, unrelated integration failures and claims of whole-client native parity.
