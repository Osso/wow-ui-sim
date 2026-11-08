# Inline XML scroll arguments

Inline scroll-related XML handlers receive named argument locals without losing their existing vararg payload. Source: `src/loader/helpers.rs`; see [XML template system](../xml-template-system.md).

## What it must do

- [x] Bind `offset` for horizontal/vertical scroll, `xrange` and `yrange` for range changes, and `delta` for mouse-wheel callbacks.
- [x] Preserve `...` forwarding and keep same-named globals unchanged; handler locals must not read those globals accidentally.
- [x] Apply the same behavior to ordinary XML and runtime templates, without altering method/function bindings.
- [x] Keep unchanged Blizzard `FauxScrollFrameTemplateLight` scrollbar and frame offsets synchronized during vertical scrolling and direct wheel-handler invocation.

## How it works

- [XML template system](../xml-template-system.md)
- [Registered scroll bindings](scrollframe-script-bindings.md)

## Implementation inventory

- `src/loader/helpers.rs`: local aliases emitted before inline bodies; existing method/function paths are unchanged.

## Tests asserting this spec

`tests/scroll_widgets/xml_arguments.rs` tests named locals, preserved varargs, global isolation and the unchanged cached FauxScrollFrame consumer. `tests/scroll_widgets/script_bindings.rs` retains the registered-binding controls.

## Known gaps (current cycle)

Historical tests-only `6b63be464` was RED in three cases: ordinary/runtime handlers read sentinel globals instead of their arguments, and the real FauxScrollFrame scrollbar does not synchronize to offset 37. The empty-function slice is independently GREEN: fresh `scroll_widgets::` output has zero prior FauxScrollFrame table-call diagnostics and one intentional sentinel diagnostic from tested error continuation. The table calls came from literal `FastHandlerRef::Function("")` resolving an empty global path to `_G` and invoking it, not shared environment or ScrollBox state. That separate empty-function proof did not verify the inline-alias slice.

Producer `0c913d2ed` implements the aliases. [Saved three-case proof](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/xml-scroll-saved-suite-2026-10-11/README.md) records ordinary XML, runtime-template and unchanged FauxScrollFrame consumer **3/3 PASS** inside suite `1dcc59c07`. The checked requirements describe those bounded assertions; function/method branches remain unchanged by source inspection. Selected loader and scroll-widget paths are unchanged through `21e1b183e`, not exhaustive runtime-input equivalence or a new current-HEAD execution. Enclosing suite remains FAIL on a separate prefork Garrison case; full-handoff proof stays open.

Cached Blizzard `SecureScrollTemplates.xml` directly uses `offset` and `delta`; the Mists `CharacterCreate.xml` range handler uses `yrange`. Vendor files remain unchanged. This is source/consumer corroboration, not a fresh native-client probe.

## Current bounded saved execution

Saved `03769bb00` independently accepts the two `d374c9520` ordinary/runtime timing-reconciled fixtures and cached FauxScrollFrame control within the broader scroll selection. Actual update tick restores range delivery without weakening exact payload/vararg/global-shadow assertions; the earlier missing-range failures remain historical REDs. [Shared receipt](editbox-text-position-selection.md#saved-xmlscroll-and-genuine-consumer-acceptance) owns totals, expected sentinel, private audit paths and remaining profile/native/parent limits. Direct wheel-handler invocation is not physical input proof.

## Out of scope

Physical wheel input/scaling, other implicit argument names, native all-profile/GPU parity, layout changes, and vendor/UI modifications.
