# ScrollFrame script bindings

Scroll offset and range changes dispatch declared scripts through the existing binding registry. The simulator parses the unchanged Blizzard EventScrollFrame declarations rather than invoking similarly named properties as an alternate path. See [XML template system](../xml-template-system.md).

## What it must do

- [ ] Parse and register XML `OnHorizontalScroll`, `OnVerticalScroll`, and `OnScrollRangeChanged` in ordinary and runtime-template construction.
- [ ] Run registered precall, normal/hooks, and postcall handlers in order, with committed scroll values and existing payloads. Preserve unchanged-offset/range no-ops.
- [ ] Report handler errors and continue later bindings.
- [ ] Deliver unchanged EventScrollFrame callback-registry events once, before normal scripts as its XML declares; do not automatically invoke unregistered `*_Intrinsic` properties.

## How it works

- [Event dispatch](../event-system.md)
- [Scroll offsets](scrollframe-offsets.md)
- [Scroll presentation](scrollframe-presentation.md)

## Implementation inventory

- `src/xml/types_support.rs`: parsed script declarations.
- `src/loader/helpers.rs`, `src/lua_api/globals/create_frame/template_chain.rs`: ordinary/runtime script enumeration and registration.
- `src/lua_api/frame/methods/widgets/slider.rs`: offset/range mutation and registered binding dispatch.

## Tests asserting this spec

`tests/scroll_widgets/script_bindings.rs`, inside the existing grouped integration target, verifies public `GetScript` bindings before invocation and loads the real cached EventScrollFrame XML/Lua through the shared XML fixture. Existing `tests/scroll_widgets.rs` retains offset/range/presentation controls.

## Known gaps (current cycle)

Tests-only `1e4706e2a` reproduces four failures. Three stop at missing XML registrations because `ScriptsXml` drops the declarations; the fourth shows automatic invocation of unregistered properties. These are not dispatch-only RED failures. `/tmp/cross-version-scroll-script-bindings-proof.md` records commands and diagnostic follow-up `c46c6f76d`. Integrated verification is pending.

Blizzard's cached `Blizzard_SharedXML/Shared/Frame/EventScrollFrame.xml` explicitly binds the three `*_Intrinsic` methods as precalls. The corresponding Lua methods emit callback-registry events. Both files remain unchanged; this is consumer/source corroboration, not a fresh native-client probe.

## Out of scope

Mouse-wheel/size/lifecycle dispatch, new XML implicit argument names, offset clamping, layout or ownership changes, opposite-state recursive callback policy, and native/GPU/all-profile parity.
