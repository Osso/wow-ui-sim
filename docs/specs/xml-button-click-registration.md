# Button XML click registration

Patch3.3.0 says Button `registerForClicks` can be set from XML. Current cached Retail, PTR, Mists and Forever XML contains single tokens and comma-separated tokens, including `LeftButtonUp, RightButtonUp`. This slice applies those declarations to the existing simulator click-registration behavior; it does not change mouse dispatch policy. See the [historical audit](../wiki/investigations/patch-3-3-0-api-audit.md#current-bounded-reconciliation) and [XML/template system](../xml-template-system.md).

## What it must do

- [ ] Direct XML Button declarations initialize click registration from the observed comma-separated forms, trimming whitespace around tokens; physical input dispatches `OnClick` only for matching button/edge combinations under the existing input policy.
- [ ] Inherited declarations apply to instances and Lua `CreateFrame` using XML templates. A derived or instance declaration replaces the inherited declaration; omission preserves inherited state or the existing no-declaration default.
- [ ] Declarative registration is present before `OnLoad`, following the existing XML property lifecycle.
- [ ] Subsequent Lua `RegisterForClicks` calls continue to replace registration normally; XML initialization adds no persistent override of Lua mutation.
- [ ] No vendor source, input-dispatch semantics, public testing API or fallback behavior changes are required.

## How it works

- [XML/template loading and inheritance](../xml-template-system.md)
- [Frame and script data flow](../frame-data-flow.md)

## Implementation inventory

- `src/xml/types.rs` — shared `FrameXml` declarative attributes.
- `src/lua_api/globals/template/direct.rs` — inherited/instance property application.
- `src/loader/xml_frame/setup.rs` — direct XML frame initialization.
- `src/lua_api/globals/create_frame/template_chain/runtime.rs` — Lua-created template frame initialization.
- `src/iced_app/mouse.rs` — existing physical input registration policy; unchanged by this slice.

## Tests asserting this spec

`src/iced_app/mouse_registration_tests.rs` contains three actual TOC/XML/template and windowless physical-input cases, plus existing Lua registration controls. Test-only `OnLoad` observations query the existing dispatch predicate; they are not native-client or physical-click receipts during `OnLoad`. Main observed all three committed cases reaching their intended behavioral RED boundaries on the source-equal sealed default-Retail library artifact at `fbb502900`: missing right-button receipt and missing registration during instance/Lua-template `OnLoad`. Earlier wrong-selector zero-test attempts are preserved and are not proof. The shared attribute/application implementation is present; GREEN and independent acceptance remain pending, so requirements above stay unchecked.

## Known gaps (current cycle)

- [x] Observe behavioral RED for all three committed cases at the intended registration/dispatch boundaries.
- [ ] Independently verify GREEN for the shared implementation on the current relevant source.
- [ ] Retain exact profile/features, source/artifact scope, input results and applicable integration evidence.
- [ ] Complete grammar, invalid/empty token behavior and historical/native-client parity remain unverified beyond the cited declarations and existing simulator policy.

## Out of scope

New input-dispatch semantics, event ordering changes, native token-validation rules inferred without evidence, source-file dimension APIs, and unrelated profile/native catalog gaps are not part of this attribute repair.
