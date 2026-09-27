# XML empty script-function clearing

An XML script declaration with `function=""` clears its inherited selected handler binding during ordinary XML and runtime-template installation. Source: `src/loader/helpers.rs` and `src/lua_api/globals/create_frame/template_chain.rs`; see [XML template system](../xml-template-system.md).

## What it must do

- [x] Clear an inherited normal binding for ordinary XML `function=""`.
- [x] Clear only an explicitly selected intrinsic `precall` or `postcall` binding for runtime templates; preserve normal and opposite intrinsic bindings.
- [x] Preserve nonempty `function=` behavior and cached FauxScrollFrame range behavior without table-call diagnostics.
- [ ] Clear the intrinsic-default binding. Source-inspected only: no runtime test distinguishes that default slot.
- [ ] Treat whitespace-only `function=` as empty and preserve `method=` precedence. Source-inspected only; no runtime assertion covers either distinction.

## How it works

- [XML template system](../xml-template-system.md)
- [[xml-template-system]]
- [[xml-empty-script-clearing]]

## Implementation inventory

- `src/loader/helpers.rs`: shared empty-function classification and selected-binding resolution.
- `src/lua_api/globals/create_frame/template_chain.rs`: optimized template installer applies the clear to its selected binding.
- `src/lua_api/globals/create_frame/template_chain/builders.rs`: `NoOp` intrinsic installation removes the selected intrinsic slot rather than the normal slot.

## Tests asserting this spec

`tests/xml_templates/inline_advanced/empty_script_overrides.rs` proves ordinary normal clearing, explicit runtime precall/postcall selection with retained other bindings, nonempty control, and cached FauxScrollFrame inheritance. Bounded default-profile verification ran 47 test instances, `cargo fmt --check`, `cargo check`, and `cargo test --test integration --no-run` clean; `/tmp/cross-version-empty-script-verification-ledger.md` records commands and logs.

## Known gaps (current cycle)

- [ ] Add a runtime assertion that distinguishes intrinsic-default clearing from explicit precall.
- [ ] Add runtime assertions for whitespace-only `function=` and `method=` precedence.

`6325ae0d4` established RED for ordinary normal, runtime precall, and cached FauxScrollFrame cases; its postcall and retained-binding assertions followed the first runtime failure, so they were not independently RED. The later bounded GREEN proof establishes explicit precall/postcall behavior but does not change that RED-first limitation.

## Out of scope

Native-client parity, full-profile coverage, vendor/UI changes, physical input, parser changes, and behavior beyond empty `function=` clearing.
