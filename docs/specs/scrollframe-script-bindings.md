# ScrollFrame script bindings

Scroll offset and range changes dispatch declared scripts through the existing binding registry. The simulator parses the unchanged Blizzard EventScrollFrame declarations rather than invoking similarly named properties as an alternate path. See [XML template system](../xml-template-system.md).

## What it must do

- [x] Parse and register XML `OnHorizontalScroll`, `OnVerticalScroll`, and `OnScrollRangeChanged` in ordinary and runtime-template construction.
- [x] Run registered precall, normal/hooks, and postcall handlers in order, with committed scroll values and existing payloads. Preserve unchanged-offset/range no-ops.
- [x] Report handler errors and continue later bindings.
- [x] Deliver unchanged EventScrollFrame callback-registry events once, before normal scripts as its XML declares; do not automatically invoke unregistered `*_Intrinsic` properties.
- [ ] Let empty `function=""` clear only its selected inherited script binding; preserve other bindings, nonempty functions, and methods.

## How it works

- [Event dispatch](../event-system.md)
- [Scroll offsets](scrollframe-offsets.md)
- [Scroll presentation](scrollframe-presentation.md)

## Implementation inventory

- `src/xml/types_support.rs`: parsed script declarations.
- `src/loader/helpers.rs`, `src/lua_api/globals/create_frame/template_chain.rs`: ordinary/runtime script enumeration and registration.
- `src/lua_api/frame/methods/widgets/slider.rs`: offset/range mutation and registered binding dispatch.

## Tests asserting this spec

`tests/scroll_widgets/script_bindings.rs`, inside the existing grouped integration target, proves registration and dispatch through an ordinary XML `<ScrollFrame>` and through cached EventScrollFrame XML/Lua. Both paths verify public `GetScript` bindings, committed payloads, callback order, and zero recorded Lua errors. Existing `tests/scroll_widgets.rs` retains offset/range/presentation controls.

## Known gaps (current cycle)

Tests-only `1e4706e2a` originally reproduced four failures: three missing XML registrations because `ScriptsXml` dropped the declarations, plus automatic invocation of an unregistered property. These were not dispatch-only RED failures. `c82ed91af` closes that boundary with 60 unique relevant passing cases, including ordinary XML and strengthened cached EventScrollFrame coverage; `/tmp/cross-version-scroll-script-bindings-verification-ledger.md` records full registration and runtime proof. Pre-existing readability caps remain reported there: `input_handlers_with_options` length and `template_chain.rs` file size.

Tests-only `6325ae0d4` is RED for empty `function=""` clearing: ordinary normal, runtime precall, and cached FauxScrollFrame inheritance retain the selected inherited handler; one nonempty-function control passes. Postcall and retained-binding assertions occur after the first runtime failure, so they are not independently RED. `3e67e7b6e` aligns optimized runtime-template installation with the existing general clear behavior; independent GREEN verification is pending. See [XML empty script-function clearing](xml-empty-script-clearing.md).

Blizzard's cached `Blizzard_SharedXML/Shared/Frame/EventScrollFrame.xml` explicitly binds the three `*_Intrinsic` methods as precalls. The corresponding Lua methods emit callback-registry events. Both files remain unchanged; this is consumer/source corroboration, not a fresh native-client probe.

## Out of scope

Mouse-wheel/size/lifecycle dispatch, new XML implicit argument names, offset clamping, layout or ownership changes, opposite-state recursive callback policy, and native/GPU/all-profile parity.
