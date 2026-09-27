# Inline XML scroll arguments

Inline scroll-related XML handlers receive named argument locals without losing their existing vararg payload. Source: `src/loader/helpers.rs`; see [XML template system](../xml-template-system.md).

## What it must do

- [ ] Bind `offset` for horizontal/vertical scroll, `xrange` and `yrange` for range changes, and `delta` for mouse-wheel callbacks.
- [ ] Preserve `...` forwarding and keep same-named globals unchanged; handler locals must not read those globals accidentally.
- [ ] Apply the same behavior to ordinary XML and runtime templates, without altering method/function bindings.
- [ ] Keep unchanged Blizzard `FauxScrollFrameTemplateLight` scrollbar and frame offsets synchronized during vertical scrolling and direct wheel-handler invocation.

## How it works

- [XML template system](../xml-template-system.md)
- [Registered scroll bindings](scrollframe-script-bindings.md)

## Implementation inventory

- `src/loader/helpers.rs`: local aliases emitted before inline bodies; existing method/function paths are unchanged.

## Tests asserting this spec

`tests/scroll_widgets/xml_arguments.rs` tests named locals, preserved varargs, global isolation and the unchanged cached FauxScrollFrame consumer. `tests/scroll_widgets/script_bindings.rs` retains the registered-binding controls.

## Known gaps (current cycle)

Tests-only `6b63be464` remains RED in three cases: ordinary/runtime handlers read sentinel globals instead of their arguments, and the real FauxScrollFrame scrollbar does not synchronize to offset 37. The empty-function slice is independently GREEN: fresh `scroll_widgets::` output has zero prior FauxScrollFrame table-call diagnostics and one intentional sentinel diagnostic from tested error continuation. The table calls came from literal `FastHandlerRef::Function("")` resolving an empty global path to `_G` and invoking it, not shared environment or ScrollBox state. This does not make the inline-alias slice error-clean or verified.

Cached Blizzard `SecureScrollTemplates.xml` directly uses `offset` and `delta`; the Mists `CharacterCreate.xml` range handler uses `yrange`. Vendor files remain unchanged. This is source/consumer corroboration, not a fresh native-client probe.

## Out of scope

Physical wheel input/scaling, other implicit argument names, native all-profile/GPU parity, layout changes, and vendor/UI modifications.
