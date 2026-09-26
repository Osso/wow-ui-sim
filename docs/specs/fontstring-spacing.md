# FontString line spacing

`FontString:SetSpacing` adds vertical separation between shaped text lines; `GetSpacing` returns the stored value. This is a simulator contract inferred from the API name and local consumers, not native-verified behavior.

## What it must do

- [x] Nonzero spacing moves the second glyph baseline by one spacing unit while leaving the first unchanged; zero spacing retains the existing output.
- [x] `GetStringHeight` and auto text height add spacing only between lines; single-line height stays unchanged.
- [x] Explicit newlines and wrapped lines use the same spacing policy; text scale scales measured separation.
- [x] `GetNumLines` counts layout lines rather than treating spacing as extra lines, including without an attached font system.
- [x] Segmented, colored FontString text applies spacing when wrapping to another line.

## How it works

See [rendering pipeline](../rendering-pipeline.md) and [Lua API](../lua-api.md).

## Implementation inventory

- `src/widget/frame.rs` — per-frame spacing state.
- `src/lua_api/frame/methods/widgets/editbox.rs` — setter, getter, auto-height refresh.
- `src/lua_api/frame/methods/text_attribute_event/text.rs` — FontString height and line-count queries.
- `src/render/font.rs` — shaped height and line-count metrics.
- `src/render/glyph.rs`, `src/render/glyph/text_emit.rs` — cached glyph layout, line coordinates and rendered height.
- `src/iced_app/quad_builders.rs` — regular and colored text emission.

## Tests asserting this spec

- `src/iced_app/quad_builders_tests.rs` — nonempty glyph quads and colored wrap.
- `tests/spacing_roundtrip.rs` — Lua height, auto-height, wrapping, scaling, count and getter.

## Known gaps (current cycle)

- [x] Record targeted RED/GREEN and commit with passing tests.

## Out of scope

Exact native spacing units, negative-spacing clamping, EditBox and SimpleHTML rendering, tooltip/message-frame spacing semantics, font-object inheritance, and full GUI visual parity are not established by cached API declarations or these tests.
