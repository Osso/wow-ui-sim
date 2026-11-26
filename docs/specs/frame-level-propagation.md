# Frame-level propagation

`SetFrameLevel` changes frame ordering in the UI tree. The frame hierarchy implementation is in `src/lua_api/frame/methods/methods_hierarchy.rs`; see [layout system](../layout-system.md) for related frame behavior.

## What it must do

- [x] **INFERRED (R14 Q6):** When a frame's level changes by a delta, shift every reachable descendant in the same strata by that delta, including fixed-level descendants.
- [x] Do not recompute a descendant's level as its parent's level plus one or its stored offset during this change.
- [x] Skip a different-strata child and its subtree.

## How it works

- [Frame hierarchy and layout](../layout-system.md)
- [Rendering order](../rendering-pipeline.md)

## Implementation inventory

- `src/lua_api/frame/methods/methods_hierarchy.rs`: shifts same-strata descendants by the root level delta.
- `src/lua_api/frame/methods/core_state/strata_level.rs`: supplies the `SetFrameLevel` delta.
- `src/lua_api/globals/real/frame_level_helpers.rs`: supplies the `RaiseFrameLevel` and `LowerFrameLevel` deltas.

## Tests asserting this spec

- `tests/frame_level.rs::set_frame_level_shifts_same_strata_descendants_by_delta`
- `tests/xml_frame_strata.rs::test_xml_fixed_frame_level_shifts_when_parent_level_changes`
- `tests/hero_talents/rendering.rs::test_button_frame_level_change_relevels_connected_edges_on_update`

## Known gaps (current cycle)

No open verification gaps in this contract.

## Out of scope

- Frame reparenting and XML template level recalculation keep their separate behavior.
