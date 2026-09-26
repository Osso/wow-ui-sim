# Button locked-highlight rendering

A Button or CheckButton with a standard HighlightTexture slot renders its highlight while locked without requiring hover. This specifies the simulator's shared rendering contract, not native-client behavior.

## What it must do

- [x] Lua-created Button `LockHighlight()` or `SetHighlightLocked(true)` emits one highlight quad in a full registry batch without hover.
- [x] Locked hover emits only one highlight quad; `UnlockHighlight()` or `SetHighlightLocked(false)` restores nonhover suppression and single hover emission.
- [x] A hidden ancestor suppresses both locked and hovered highlight emission.
- [x] Unlocked standard HighlightTexture renders on hover, not without hover.

## How it works

The early texture-visibility decision had unconditionally culled `HighlightTexture` and HIGHLIGHT-layer children before the later locked-child allowance in quad building could run. `button_vis.rs` now permits only a locked standard `HighlightTexture` slot through that early decision. The registry path and live hover overlay share an unlocked-and-visible predicate, so an unlocked hovered slot emits once rather than through both paths. Ancestor visibility remains a prerequisite for both locked regular emission and live hover emission.

- [Rendering pipeline](../rendering-pipeline.md)

## Implementation inventory

- `src/iced_app/button_vis.rs` — distinguishes a locked standard slot child from generic HIGHLIGHT regions, repairs the early cull, and supplies the shared hover predicate.
- `src/iced_app/strata_emit.rs` — emits registry batches and hover overlays.
- `src/iced_app/render_textures.rs` — emits live hover overlays.
- `src/iced_app/quad_builders.rs` — emits the surviving locked slot child.

## Tests asserting this spec

- `tests/button_state_textures.rs`: `locked_button_highlight_renders_once_without_hover_and_unlock_restores_hover`, `highlight_texture_only_when_hovered`.
- `/tmp/cross-version-locked-highlight-proof.md`: RED is 0/1 (locked nonhover emitted zero quads); committed `204f52235` GREEN is the full `button_state_textures::` group, 9/9.

## Known gaps (current cycle)

- [ ] Main-owned final gate remains pending.
- [ ] Native-client behavior remains unverified.

## Out of scope

Pressed and disabled state policy, generic HIGHLIGHT-layer behavior, and native lock/disabled interaction.
