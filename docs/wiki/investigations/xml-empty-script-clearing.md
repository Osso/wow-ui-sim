# XML Empty Script-Function Clearing

`function=""` clears its selected inherited script binding. The default-profile module has nine development-test passes; independent verification is pending. It does not claim native-client or full-profile parity.

## Content

### Root cause

The optimized path did **not** classify `function=""` as `FastHandlerRef::NoOp`. It represented it literally as `FastHandlerRef::Function("")`; `resolve_global_path("")` returns the globals table (`_G`), not `_G[""]`, and the installer then attempted to invoke that table.

A separate existing `NoOp` installation bug ignored an intrinsic slot and always cleared the normal binding. It is distinct from the empty-function table invocation: intrinsic `NoOp` installation must remove the selected precall or postcall slot, while normal `NoOp` installation removes the normal script.

### Scoped contract

Ordinary XML `function=""` clears an inherited normal binding. Runtime templates clear explicitly selected intrinsic precall/postcall bindings and preserve normal and opposite intrinsic bindings; absent `intrinsicOrder` selects precall. Ordinary XML and runtime templates treat whitespace-only `function=` plus empty or whitespace-only bodies as clears. In both construction paths, `method=` with empty `function=` remains callable. Nonempty `function=` and other bindings remain unchanged.

### Evidence status

`6325ae0d4` established RED for ordinary normal, runtime precall, and cached FauxScrollFrame inherited-range cases; its nonempty-function control passed. Postcall and retained-binding assertions followed the first runtime failure, so they were not independently RED.

Tests-only `267f1fea4` adds five behavior cases to the existing module: intrinsic-default precall clearing; ordinary and runtime whitespace/empty-body clearing; and ordinary and runtime `method=` plus empty `function=`. Together with four existing cases, the bounded default-profile module passes 9/9. This closes the prior source-only branches, but independent verification remains pending. `/tmp/cross-version-empty-script-edge-proof.md` records the scoped test run. No native-client, full-profile, vendor/UI, or clean-all-diagnostics claim is made.

### Scroll diagnostic correction

The prior five FauxScrollFrame `attempt to call upvalue 'func' (a table value)` diagnostics came from the literal empty `FastHandlerRef::Function("")` path resolving to `_G` and invoking it. Fresh `scroll_widgets::` output has zero occurrences of that table-call error. It retains exactly one intentional `scroll precall sentinel` diagnostic from the tested error-continuation path; that test passes and asserts the report while later bindings continue.

## Sources

- [XML empty script-function clearing](../../specs/xml-empty-script-clearing.md) — contract and proof boundary
- [helpers.rs](../../../src/loader/helpers.rs) — shared classification and binding selection
- [template_chain.rs](../../../src/lua_api/globals/create_frame/template_chain.rs) — optimized handler construction and installation
- [builders.rs](../../../src/lua_api/globals/create_frame/template_chain/builders.rs) — selected-slot `NoOp` removal
- [empty_script_overrides.rs](../../../tests/xml_templates/inline_advanced/empty_script_overrides.rs) — RED and bounded GREEN coverage

## See Also

- [[xml-template-system]] — XML script installation paths
- [[widget-system]] — ScrollFrame consumer boundary
- [[lua-api]] — public script bindings
