# XML empty script-function clearing

An XML script declaration with `function=""` clears its inherited selected handler binding during ordinary XML and runtime-template installation. Source: `src/loader/helpers.rs` and `src/lua_api/globals/create_frame/template_chain.rs`; see [XML template system](../xml-template-system.md).

## What it must do

- [ ] Treat `function=""` as a clear for the inherited selected normal, precall, postcall, or intrinsic-default binding.
- [ ] Preserve unselected bindings, `method=`, nonempty `function=`, and other binding behavior.
- [ ] Apply the clear in ordinary XML and runtime template construction.

## How it works

- [XML template system](../xml-template-system.md)
- [[xml-template-system]]
- [[xml-empty-script-clearing]]

## Implementation inventory

- `src/loader/helpers.rs`: shared empty-function classification and intrinsic binding selection.
- `src/lua_api/globals/create_frame/template_chain.rs`: optimized template installer applies the clear to its selected binding.

## Tests asserting this spec

`tests/xml_templates/inline_advanced/empty_script_overrides.rs` covers ordinary normal clearing, runtime precall/postcall selection with retained other bindings, nonempty control, and cached FauxScrollFrame inheritance.

## Known gaps (current cycle)

Tests-only `6325ae0d4` is RED: three expected failures with one nonempty-function control passing. The corrected ordinary fixture, runtime precall, and cached FauxScrollFrame cases establish the pre-fix boundary; postcall and retained-binding assertions occur after the first runtime failure and are not independently RED. `/tmp/cross-version-empty-script-proof.md` records this evidence. Independent GREEN verification of `3e67e7b6e` is pending.

## Out of scope

Native-client parity, vendor/UI changes, physical input, parser changes, and any behavior beyond empty `function=` clearing.
