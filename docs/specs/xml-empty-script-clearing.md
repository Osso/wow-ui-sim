# XML empty script-function clearing

An XML script declaration with `function=""` clears its inherited selected handler binding during ordinary XML and runtime-template installation. Source: `src/loader/helpers.rs` and `src/lua_api/globals/create_frame/template_chain.rs`; see [XML template system](../xml-template-system.md).

## What it must do

- [x] Clear an inherited normal binding for ordinary XML `function=""`.
- [x] Clear only an explicitly selected intrinsic `precall` or `postcall` binding for runtime templates; preserve normal and opposite intrinsic bindings.
- [x] Preserve nonempty `function=` behavior and cached FauxScrollFrame range behavior without table-call diagnostics.
- [x] Clear the runtime intrinsic-default binding: absent `intrinsicOrder` selects precall while retaining normal and postcall bindings.
- [x] Treat whitespace-only `function=` and empty/whitespace-only bodies as empty in ordinary XML and runtime templates.
- [x] Preserve a callable `method=` binding when paired with empty `function=` in ordinary XML and runtime templates.

## How it works

- [XML template system](../xml-template-system.md)
- [[xml-template-system]]
- [[xml-empty-script-clearing]]

## Implementation inventory

- `src/loader/helpers.rs`: shared empty-function classification and selected-binding resolution.
- `src/lua_api/globals/create_frame/template_chain.rs`: optimized template installer applies the clear to its selected binding.
- `src/lua_api/globals/create_frame/template_chain/builders.rs`: `NoOp` intrinsic installation removes the selected intrinsic slot rather than the normal slot.

## Tests asserting this spec

`tests/xml_templates/inline_advanced/empty_script_overrides.rs` now has nine passing default-profile module cases: four existing cases cover ordinary normal clearing, explicit runtime precall/postcall selection and retention, nonempty control, and cached FauxScrollFrame inheritance; five added cases cover runtime intrinsic-default selection, ordinary and runtime whitespace/empty-body clearing, and ordinary and runtime `method=` with empty `function=`. The five additions are development-test passes at `267f1fea4`; independent verification remains pending. `/tmp/cross-version-empty-script-edge-proof.md` records their bounded scope.

## Proof status

`6325ae0d4` established RED for ordinary normal, runtime precall, and cached FauxScrollFrame cases; its postcall and retained-binding assertions followed the first runtime failure, so they were not independently RED. The later source fix did not itself prove the remaining branches. `267f1fea4` closes those source-only branches with behavior tests, but does not establish native-client parity.

## Out of scope

Native-client parity, full-profile coverage, vendor/UI changes, physical input, parser changes, and behavior beyond empty `function=` clearing.
