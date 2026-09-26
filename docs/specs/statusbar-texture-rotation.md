# StatusBar texture rotation

`StatusBar:SetRotatesTexture` stores a boolean on the bar; its fill texture uses quarter-turn UVs. This is simulator behavior informed by the local Classic API definitions and the Solarity build-12340 source-derived assertion, **not independently verified native client behavior**. See [rendering pipeline](../rendering-pipeline.md).

## What it must do

- [x] New bars report `false`; `SetRotatesTexture(true/false)` round-trips, including when set before or across bar texture replacement.
- [x] The owned texture exposes the rotated eight-coordinate tuple `(0,1,1,1,0,0,1,0)`; replacing a raw source with an atlas retains rotation. Adopting an unrotated texture retains its existing custom texture coordinates.
- [x] Rendered fill quads rotate texture samples and clip partial fills without changing fill geometry.

## How it works

- [Rendering pipeline](../rendering-pipeline.md)
- [Texture atlas system](../texture-atlas-system.md)

## Implementation inventory

- `src/widget/frame.rs`, `src/widget/frame_defaults.rs` — bar property and default.
- `src/lua_api/frame/methods/widgets/statusbar.rs` — methods and texture source lifecycle.
- `src/iced_app/quad_builders_textures.rs` — bar-fill UV clipping and atlas remapping.

## Tests asserting this spec

- `tests/widget_methods_colorselect.rs` — source GREEN: two rotation/replacement cases plus one adopted-custom-coordinate case (3/3 total) under the diagnostic feature set.
- `src/iced_app/quad_builders_textures.rs` — source GREEN: two rendered partial-fill UV cases (2/2) under the diagnostic feature set.

The five rotation cases passed only with `gui,client-wrath,aura-instance-enumeration`; they do not accept the blocked normal `gui,client-wrath` lane.

## Known gaps (current cycle)

- [ ] Unblock and repeat under normal `gui,client-wrath`; `aura-instance-enumeration` was diagnostic only.
- [ ] Confirm exact default and rotation semantics on an actual Classic Wrath client; Solarity assertions concern original build 12340, not this target.
- [ ] Establish XML `rotatesTexture` parsing and interaction with geometry separately; schema declaration alone is insufficient.

## Out of scope

- Range and callback behavior belongs to the separate value-callback slice.
- XML `rotatesTexture` parsing is not inferred from schema presence alone.
