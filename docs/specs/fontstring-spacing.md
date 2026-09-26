# FontString line spacing

`FontString:SetSpacing` stores per-`FontString` line spacing; `GetSpacing` returns that stored value. At `d6d7a9078`, the default-feature simulator path applies it between shaped lines in regular and segmented rendering and in `GetStringHeight`/auto-height measurement. At `4ccf92ee0` and `2c0dcb53f`, initial construction and explicit `SetFontObject` assignment snapshot an explicitly defined FontObject spacing value and refresh auto text height. This is source-backed simulator behavior, not native-verified behavior.

## What it must do

- [x] Nonzero spacing moves the second glyph baseline by one spacing unit while leaving the first unchanged; zero spacing retains the existing output.
- [x] `GetStringHeight` and auto text height add spacing only between lines; single-line height stays unchanged.
- [x] Explicit newlines and wrapped lines use the same spacing policy; text scale scales measured separation.
- [x] `GetNumLines` counts layout lines rather than treating spacing as extra lines, including without an attached font system.
- [x] Segmented, colored FontString text applies spacing when wrapping to another line.
- [x] The regular glyph-layout cache keys spacing, stores spaced line coordinates and total height, and receives spacing scaled by the FontString effective scale.
- [x] Initial FontString construction with an explicitly spaced FontObject copies that spacing into the FontString snapshot.
- [x] Explicit `SetFontObject` assignment copies explicitly defined FontObject spacing and refreshes auto text height.
- [x] The FontString's later local `SetSpacing` remains independent and does not mutate the FontObject.

## How it works

See [rendering pipeline](../rendering-pipeline.md) and [Lua API](../lua-api.md).

## Implementation inventory

- `src/widget/frame.rs` — per-frame spacing state.
- `src/lua_api/frame/methods/button_anchor_hierarchy/font_strings.rs` — reads explicit FontObject spacing into the construction/assignment snapshot and detects changes.
- `src/lua_api/frame/methods/widgets/editbox.rs` — setter, getter, auto-height refresh.
- `src/lua_api/frame/methods/text_attribute_event/text.rs` — FontString height and line-count queries.
- `src/render/font.rs` — shaped height and line-count metrics.
- `src/render/glyph.rs`, `src/render/glyph/text_emit.rs` — cached glyph layout, line coordinates and rendered height.
- `src/iced_app/quad_builders.rs` — regular text passes `text_line_spacing * effective_scale` to glyph emission; segmented colored placement adds that same scaled spacing on a new layout line.
- `src/iced_app/message_frame_render.rs` — deliberately passes `0.0` for message measurement and emission; this FontString change does not establish MessageFrame spacing.

## Source-lane proof

At `d6d7a9078`, default-feature targeted source tests were RED 0/2 for renderer glyph placement and RED 0/3 for integration height/auto-height behavior. Final targeted GREEN is **2/2 library tests** and **4/4 integration tests**; the fourth integration case covers line count without an attached font system. Logs: `/tmp/font-spacing-{lib,integration}-{red,green-current}.log`; full ledger: `/tmp/cross-version-font-spacing-proof.md`.

At `4ccf92ee0` and `2c0dcb53f`, the grouped default-feature `spacing_roundtrip` batch is GREEN **12/12**: construction and explicit assignment snapshot tests pass alongside ten existing spacing controls. The first implementation left `SetFontObject` auto text height stale; `2c0dcb53f` fixes that refresh. Actual logs: `/tmp/font-object-spacing-green-final-{build,}.log`; ledger: `/tmp/cross-version-font-object-spacing-proof.md`. Independent final verification is pending.

Independent verification of the earlier renderer/measurement scope reused the six exact-source spacing tests and passed four additional measurement/glyph controls, format, and default-feature check without warnings. See `/tmp/cross-version-text-animation-proof.md`. No native-client, full-profile, broad-suite, or GUI-parity claim follows from these counts.

## Tests asserting this spec

- `src/iced_app/quad_builders_tests.rs` — regular nonempty glyph placement and segmented colored wrap.
- `tests/spacing_roundtrip.rs` — Lua height, auto-height, wrapping, text scale, count, getter, and explicit FontObject spacing snapshot.

## Untested / out of scope

Exact native spacing units, negative-spacing clamping, EditBox and SimpleHTML rendering, tooltip/message-frame spacing semantics, later live `FontObject` mutation, FontObject graph propagation, precedence of local spacing set before a later `SetFontObject`, and full GUI visual parity are not established. `SetSpacing` is shared in the method registration path, but the documented/rendered proof here is bounded to `FontString`; it does not assign this behavior to `MessageFrame`.
