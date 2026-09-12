# Sprite-sheet cell coordinates

`Texture:SetSpriteSheetCell` selects a normalized sprite cell through the existing texture-coordinate mutation path. Pinned API metadata declares `cell`, `numRows`, `numColumns`, and optional `cellWidth`/`cellHeight`; row-major mapping and validation below are explicit bounded simulator policies, not native conformance. See [texture/atlas architecture](../texture-atlas-system.md).

## What it must do

- [ ] Map one-based cells row-major into normalized UV rectangles, including nonsquare grids and the final cell. Replace prior quad coordinates and return zero Lua values; `GetTexCoord` exposes eight corner coordinates.
- [ ] Require finite positive integer rows/columns and an integer cell in `1..rows*columns`, without arbitrary grid-size caps. Use existing floating-point texture-coordinate precision.
- [ ] Reject malformed, missing, nonfinite, fractional, and out-of-range inputs before coordinate mutation.
- [ ] Accept absent/nil optional dimensions; reject any nonnil `cellWidth` or `cellHeight` explicitly as unmodeled before mutation.
- [ ] Use the same atlas remapping and visual invalidation path as rectangular `SetTexCoord`, without modifying vendor code.

## How it works

- [Texture and atlas system](../texture-atlas-system.md)
- [Rendering pipeline](../rendering-pipeline.md)

## Implementation inventory

- `src/lua_api/frame/methods/widgets/texture/rotation_mask.rs` — argument validation and bounded cell mapping.
- `src/lua_api/frame/methods/widgets/texture/coords.rs` — shared rectangular coordinate mutation.

## Tests asserting this spec

- `tests/texture_methods_port.rs` — cells 1/8 on 4x4, nonsquare 3x2 including the final cell, replacement, return arity, invalid-input atomicity, and optional-dimension rejection.
- `tests/targeting_verbs.rs::raid_target_icons_update_real_blizzard_target_frame_consumer` — unmodified Blizzard Lua icon consumer, visibility, coordinates, and toggle behavior; not historical XML construction.

## Known gaps (current cycle)

- [ ] Focused development proof and parent final verification.
- [ ] Optional dimension semantics lack evidence and remain unsupported.

## Out of scope

- Secret inputs/aspects and native validation/error wording: deferred security and conformance work.
- Native optional dimensions, pixel cropping, and texture sizing: no documented behavior established by retained evidence.
- Vendor changes and unrelated atlas/rendering changes.
