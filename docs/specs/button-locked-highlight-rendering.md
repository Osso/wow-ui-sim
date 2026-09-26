# Button locked-highlight rendering

A Button or CheckButton with a standard HighlightTexture slot renders its highlight while locked without requiring hover. This specifies the simulator's shared rendering contract, not native-client behavior.

## What it must do

- [x] Lua-created Button `LockHighlight()` or `SetHighlightLocked(true)` emits one highlight quad in a full registry batch without hover.
- [x] Locked hover emits only one highlight quad; `UnlockHighlight()` or `SetHighlightLocked(false)` restores nonhover suppression and single hover emission.
- [x] A hidden ancestor suppresses both locked and hovered highlight emission.
- [x] Unlocked standard HighlightTexture renders on hover, not without hover.

## How it works

- [Rendering pipeline](../rendering-pipeline.md)

## Implementation inventory

- `src/iced_app/button_vis.rs` — distinguishes locked standard slot children from generic HIGHLIGHT regions and decides hover-overlay eligibility.
- `src/iced_app/strata_emit.rs` — emits registry batches and hover overlays.
- `src/iced_app/render_textures.rs` — emits live hover overlays.
- `src/iced_app/quad_builders.rs` — allows locked slot children through texture emission.

## Tests asserting this spec

- `tests/button_state_textures.rs`: `locked_button_highlight_renders_once_without_hover_and_unlock_restores_hover`, `highlight_texture_only_when_hovered`.

## Known gaps (current cycle)

- [ ] Main-owned broader verification; targeted default-feature integrated tests pass.

## Out of scope

Native-client lock/disabled interaction, changes to pressed/disabled state, and generic HIGHLIGHT-layer behavior.
