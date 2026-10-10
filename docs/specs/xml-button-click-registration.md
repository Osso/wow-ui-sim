# Button XML click registration

Patch3.3.0 says Button `registerForClicks` can be set from XML. Current cached Retail, PTR, Mists and Forever XML contains single tokens and comma-separated tokens, including `LeftButtonUp, RightButtonUp`. This slice applies those declarations to the existing simulator click-registration behavior; it does not change mouse dispatch policy. See the [historical audit](../wiki/investigations/patch-3-3-0-api-audit.md#current-bounded-reconciliation) and [XML/template system](../xml-template-system.md).

## What it must do

Checked items record the bounded default-Retail simulator proof below, not all-profile or native acceptance.

- [x] Direct XML Button declarations initialize click registration from the observed comma-separated forms, trimming whitespace around tokens; physical input dispatches `OnClick` only for matching button/edge combinations under the existing input policy.
- [x] Inherited declarations apply to instances and Lua `CreateFrame` using XML templates. A derived or instance declaration replaces the inherited declaration; omission preserves inherited state or the existing no-declaration default.
- [x] Declarative registration is present before `OnLoad`, following the existing XML property lifecycle.
- [x] Subsequent Lua `RegisterForClicks` calls continue to replace registration normally; XML initialization adds no persistent override of Lua mutation.
- [x] No vendor source, input-dispatch semantics, public testing API or fallback behavior changes are required.

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

`src/iced_app/mouse_registration_tests.rs` contains three actual TOC/XML/template and windowless physical-input cases, plus existing Lua registration controls. Test-only `OnLoad` observations query the existing dispatch predicate; they are not native-client or physical-click receipts during `OnLoad`. Main observed all three committed cases reaching their intended behavioral RED boundaries on the source-equal sealed default-Retail library artifact at `fe04144cf` (captured source scope equals the earlier `fbb502900` label): missing right-button receipt and missing registration during instance/Lua-template `OnLoad`. Earlier wrong-selector zero-test attempts are preserved and are not proof. The shared implementation `33d62d705` now has independently read saved default-Retail module GREEN at `c6bc87c12`: three new cases plus six existing controls, actual 9/9, execution exit0; compile/fmt/default-check also exit0. Exact selectors, source/artifact seal and exclusions live in the [integrated proof SSOT](../wiki/investigations/integrated-source-and-factory-proof-2026-10-09.md#xml-click-registration--bounded-default-retail-green). This is bounded simulator proof, not native/all-profile or broad integration acceptance; parent integration/profile acceptance remains open.

## Known gaps (current cycle)

- [x] Observe behavioral RED for all three committed cases at the intended registration/dispatch boundaries.
- [x] Independently verify saved GREEN for the shared implementation at `c6bc87c12`; relevant implementation/test bytes still match the receipt at this docs audit.
- [x] Retain exact default-Retail features, source/artifact scope and module input results in the linked SSOT; broad integration evidence is not supplied by this module run.
- [ ] Complete grammar, invalid/empty token behavior and historical/native-client parity remain unverified beyond the cited declarations and existing simulator policy.

## Out of scope

New input-dispatch semantics, event ordering changes, native token-validation rules inferred without evidence, source-file dimension APIs, and unrelated profile/native catalog gaps are not part of this attribute repair.
