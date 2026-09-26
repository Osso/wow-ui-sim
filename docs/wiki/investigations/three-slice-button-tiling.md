# Three-Slice Button Highlight Stripes

Escape menu red buttons showed vertical stripe artifacts because their standard
`HighlightTexture` overlays were rendered while the buttons were not hovered.
The highlight atlas itself contains pale vertical columns, so drawing it on all
buttons made inactive buttons look striped. The remaining active-highlight
stripe came from applying the simulator's dark-texture brightness boost to
additive overlays, which made very low-alpha atlas edge pixels too visible.

## Content

`ThreeSliceButtonMixin` sets Left and Right with `useAtlasSize=true`, scales them
from `buttonHeight / leftAtlasInfo.height`, and sets Center with
`SetAtlas(self:GetCenterAtlasName())`. The Center texture is anchored between
the scaled Left and Right textures and has `horizTile=true`.

For `BigRedThreeSliceButtonTemplate`, the pieces are:

- `128-RedButton-Left`: `114x128`, not tiled
- `_128-RedButton-Center`: `64x128`, `tiles_horizontally=true`
- `128-RedButton-Right`: `292x128`, not tiled
- `128-RedButton-Highlight`: full-button additive hover overlay

In a 200x36 GameMenu button, Left resolves to about `32x36`, Right to about
`82x36`, and Center to about `85x36`. The Center source tile repeats at its
authored atlas size. The dense stripe report was not from that Center texture:
runtime dumps showed each button's `.HighlightTexture` child visible at alpha
1.0 even with no hovered frame.

The original fix established the intended child policy, but `204f52235` found a later contradiction: `button_vis.rs` returned false for `HighlightTexture` and generic HIGHLIGHT children before the later quad-builder locked-child allowance could execute. The repair permits only a locked standard `HighlightTexture` slot through the early cull. The registry generic emitter and live hover overlay now share the same unlocked-and-visible predicate, so unlocked hover emits one quad, not two. A hidden ancestor suppresses both locked regular emission and live hover emission. The full `button_state_textures::` group is GREEN 9/9; the preceding locked nonhover case was RED 0/1 (zero rather than one quad). Additive overlays still bypass the shader brightness boost.

This does not redefine pressed or disabled selection, or generic HIGHLIGHT-layer policy. The evidence is simulator-only; native-client behavior and the main-owned final gate remain unverified.

## Sources

- [ThreeSliceButtonTemplate.lua](../../Interface/BlizzardUI/Blizzard_SharedXML/Shared/Button/ThreeSliceButtonTemplate.lua) — Blizzard mixin scale and atlas setup
- [ThreeSliceButtonTemplate.xml](../../Interface/BlizzardUI/Blizzard_SharedXML/Shared/Button/ThreeSliceButtonTemplate.xml) — Center texture `horizTile=true`
- [data/atlas.rs](../../data/atlas.rs) — RedButton atlas dimensions and tiling metadata
- [quad_builders.rs](../../src/iced_app/quad_builders.rs) — button highlight child gating
- [quad.wgsl](../../src/render/shader/quad.wgsl) — additive overlays bypass brightness boost
- [tiling.rs](../../src/iced_app/tiling.rs) — simulator tile-size computation and regression test
- [Button locked-highlight rendering](../../specs/button-locked-highlight-rendering.md) — verified shared emission boundary and limits
- `/tmp/cross-version-locked-highlight-proof.md` — actual RED/GREEN commands and logs for `204f52235`

## See Also

- [[texture-atlas]] — atlas metadata and UV remapping
- [[rendering-pipeline]] — quad emission and texture request flow
- [[animated-status-bar-atlas-visibility]] — recognized Button slot visibility boundary
