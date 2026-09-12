# Sprite-sheet cell coordinates

`Texture:SetSpriteSheetCell` selects a normalized sprite cell through the existing texture-coordinate mutation path. Pinned API metadata declares `cell`, `numRows`, `numColumns`, and optional `cellWidth`/`cellHeight`; row-major mapping and validation below are explicit bounded simulator policies, not native conformance. See [texture/atlas architecture](../texture-atlas-system.md).

## What it must do

- [x] Map one-based cells row-major into normalized UV rectangles, including nonsquare grids and the final cell. Replace prior quad coordinates and return zero Lua values; `GetTexCoord` exposes eight corner coordinates.
- [x] Require finite positive integer rows/columns and an integer cell in `1..rows*columns`, without arbitrary grid-size caps. Use existing floating-point texture-coordinate precision.
- [x] Reject malformed, missing, nonfinite, fractional, and out-of-range inputs before coordinate mutation.
- [x] Accept absent/nil optional dimensions; reject any nonnil `cellWidth` or `cellHeight` explicitly as unmodeled before mutation.
- [x] Use the same atlas remapping and visual invalidation path as rectangular `SetTexCoord`, without modifying vendor code.

## How it works

- [Texture and atlas system](../texture-atlas-system.md)
- [Rendering pipeline](../rendering-pipeline.md)

## Implementation inventory

- `src/lua_api/frame/methods/widgets/texture/rotation_mask.rs` — argument validation and bounded cell mapping.
- `src/lua_api/frame/methods/widgets/texture/coords.rs` — shared rectangular coordinate mutation.

## Tests asserting this spec

- `tests/texture_methods_port.rs` — cells 1/8 on 4x4, nonsquare 3x2 including the final cell, replacement, return arity, invalid-input atomicity, and optional-dimension rejection.
- `tests/targeting_verbs.rs::raid_target_icons_update_real_blizzard_target_frame_consumer` — unmodified Blizzard Lua icon consumer, visibility, coordinates, and toggle behavior; not historical XML construction.

## Verification (current cycle)

- [x] Historical 12.0.0 development proof: three primitive tests and one original-Lua raid-icon consumer passed. The corrected record side effect was then proven separately by two tests.
- [x] Independent proof: corrected record plus consumer cases passed 3/3 on 12.0.7; earlier 12.0.5/12.0.7 51-case and Mists unaffected assertions are reused only where their source/test bytes were unchanged. Formatting, check, build, and current standalone startup (`[]`) freshly passed; no new readability finding was reported.
- [x] Source inspection confirms `SetSpriteSheetCell` applies the shared rectangular coordinate path, including atlas remapping and visual invalidation.

## Known gaps (current cycle)

- [ ] Optional dimension semantics lack evidence and remain unsupported.
- [ ] Historical XML construction fails on unsupported `AuraContainer`; original-Lua consumer proof is not clean addon/UI loading proof.

## Out of scope

- Secret inputs/aspects and native validation/error wording: deferred security and conformance work.
- Native optional dimensions, pixel cropping, and texture sizing: no documented behavior established by retained evidence.
- Vendor changes and unrelated atlas/rendering changes.
