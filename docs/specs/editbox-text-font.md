# EditBox XML text properties

`src/loader/editbox_font.rs` applies an EditBox's own or nearest inherited `<FontString>` text properties. This is a simulator model; the contributor's Forever client observations are not independent native verification. See [EditBox investigation](../wiki/investigations/editbox-text-font.md).

## What it must do

- [ ] Apply the selected XML text font object, explicit horizontal/vertical justification and color on ordinary XML and runtime template paths.
- [ ] Apply instance text insets or the nearest template's text-inset record on both paths.
- [ ] Keep contributor-only assertions about default LEFT justification, font-object identity and justification precedence behind `optional-tests` until independent profile evidence supports them.

## How it works

- [EditBox investigation](../wiki/investigations/editbox-text-font.md).

## Implementation inventory

- `src/loader/editbox_font.rs`: selection and application of text properties.
- `src/loader/xml_frame/finalize.rs`: ordinary XML application.
- `src/lua_api/globals/create_frame/template_chain/runtime_loader_effects.rs`: runtime font application.
- `src/lua_api/globals/template/direct.rs`: text-inset record selection.
- `src/loader/xml_frame/setup.rs` and `src/lua_api/globals/create_frame/template_chain/runtime.rs`: inset application.
- `src/widget/frame.rs`: modeled LEFT default.

## Tests asserting this spec

- `tests/editbox_text_defaults.rs`: optional synthetic-template tests for default justification, inherited font object/color, explicit justification/color and inherited left/right insets.

## Known gaps (current cycle)

- [ ] Build and run affected tests on the combined merge tree.
- [ ] Verify default LEFT and inherited font-object identity/justification rules independently for each affected profile.
- [ ] Verify the contributor's native template values and XML Instructions-region interaction. Current tests do not prove SearchBoxTemplate native parity.

## Out of scope

This change does not model the contributor-reported SearchBoxTemplate/Instructions font replacement or unnamed font-object identity. It does not establish partial-property merging, inline font file/size application, native default font/color, vertical justification or alpha parity. Existing master assertions remain unchanged.
