# Patch 12.0.5 API Audit

Patch 12.0.5 work in wow-ui-sim is probe-driven rather than a single additive API-diff bridge pass. Retail `12.0.5.67823` live probes pinned core frame, event, attribute, identity, scale-event, and XML frame-level behavior; the simulator already models the safe findings with regression coverage. No dedicated `patch_12_0_5_inert_defaults` module exists.

## Content

### Source scope

The 12.0.5 audit sources are live-client probe addons under `docs/addons/` and the corresponding wiki investigation pages. Unlike the later 12.0.7 and 12.1 passes, this audit did not start from a patch-specific API-change page with a large additive namespace list.

Primary retained 12.0.5 probe sources (13 SavedVariables captures): `AnimScriptProbe`, `AttributeDispatchProbe`, `CoreBehaviorProbe`, `DevToolsDumpProbe`, `FrameIdentityProbe`, `HookScriptBindingProbe`, `IsProtectedProbe`, `JustifyProbe`, `ProtectedRetailProbe`, `ScaleEventProbe`, `SetAtlasProbe`, `StoreForbiddenProbe`, and `TextureSetTextureProbe`. `XmlFrameLevelProbe` findings are documented, but its raw capture was not retained.

The machine register is `data/patch-api/12.0.5-probes.json`, sourced from `data/patch-api/sources/12.0.5-probes.json`; [[patch-12-0-5-probe-inventory]] is its human-readable inventory. It preserves 38 probe subfindings. Current machine classification is **33 best-effort, 0 implemented, 4 evidence-required, 1 exception-requested, and 0 untriaged**: the exception is approved provenance-only, while four behavior gaps remain evidence-required—one impossible same-size input-boundary gap and three unsafe Store/security gaps.

### Itemized probe status

**Machine-classified with direct behavioral evidence:** the full Frame/AnimationGroup/nine-subtype script-handler matrix; repeated scalar/false attribute dispatch and the two-panel ShowUIPanel pulse; normal-frame forbidden behavior and absent retail forbidden constructor; valid/invalid unit-event filters; wildcard false/true/string attributes; Raise/Lower level boundaries and GUI mouse-focus ordering; frame identity slot, surrogate dispatch, duplicate-frame freshness, and DevTools frame-array dump metadata; normal HookScript chaining plus rejected explicit slots 0 and 2; absent legacy protection setters, the full plain-frame and XML-protected-frame sequences, and protected secure templates; frame-layer FontString default points, size variants, explicit anchors, implicit ButtonText anchors, EditBox backing regions/TextInsets, and MessageFrame/ScrollingMessageFrame owner-region behavior; complete observable display/UI-scale/CVAR ordering; the complete invalid-atlas argument matrix; texture path/FDID and clear behavior; and bare/fixed/parent/reparent XML frame-level semantics and flags.

**Evidence and exception state:** XML raw-capture provenance is the only approved exception because it concerns missing historical evidence while frame-level behavior is independently regression-tested. Same-size transitions remain an evidence-required impossible input-boundary gap. Secure Store behavior, Store dropdown population, and Store forbidden descendants remain evidence-required unsafe Store/security gaps; existing subsystem tests are not substituted for the missing probe behavior.

### Open probe gaps and evidence-required rows

A broad approval recorded on 2026-07-14 is superseded. The five rows below distinguish four item-specific evidence-required behavior gaps from one approved provenance-only exception-requested row. Evidence-required rows carry hashed repository evidence but need no approval, commit, or focused test; they await authoritative/live evidence or correct implementation.

1. **ProtectedRetailProbe.SecureStore — evidence-required unsafe:** Retained Store frames are forbidden, legacy setters are absent, and `IsProtected` errors; the current simulator returns normally, so exact forbidden/secret-return enforcement is unsafe to guess.
2. **ScaleEventProbe.SameSizeDuplicatePair — evidence-required impossible:** Retained live observations already establish that maximize/restore can produce another ordered display/scale pair without a dimension change. The missing boundary is the production window-transition signal, not screen-size evidence: the probe records dimensions and state, while the simulator receives only draw-time `iced::Size` and deliberately ignores equal sizes. Pinned iced 0.14.0 / winit 0.30.12 expose no maximize/restore/fullscreen transition notification. Their mode/maximize queries are not ordered transition events, so polling would be an approximation. Correct behavior remains unmodeled.
3. **XmlFrameLevelProbe.RawCaptureProvenance — approved impossible:** Behavior is regression-tested, but the raw SavedVariables capture does not exist and cannot be reconstructed locally.
4. **StoreForbiddenProbe.DropdownPopulation — evidence-required unsafe:** The retained capture has `StoreDropdown_SetDropdown == nil`, so population, reuse, text/check, callback, and protection behavior was never observed.
5. **StoreForbiddenProbe.ForbiddenDescendants — evidence-required unsafe:** The retained file lacks the `/sfp` manual descendant scan, so Store descendant forbidden/protected state is unknown; correct behavior remains unmodeled pending authoritative/live evidence.

The 38-row register is complete only for its explicit probe contract; generic fallbacks cannot be claimed as globally patch-complete without another concrete source.

### Completed modeled work

Retail `12.0.5.67823` probe results are modeled in these areas:

- `CreateForbiddenFrame` is absent on current retail, and `SetForbidden(true)` on addon-created normal frames succeeds without making the frame forbidden.
- `RegisterUnitEvent("UNIT_HEALTH", "not_a_unit")` registers the event but drops the invalid unit filter; `IsEventRegistered("UNIT_HEALTH")` returns registered with no unit filter.
- Wildcard `GetAttribute` preserves an explicit `false` stored with `SetAttribute("*type1", false)`.
- `Raise()` / `Lower()` only affect same-raw-level tie ordering and do not let a lower frame level overtake a higher one.
- Frame identity dispatch uses `frame[0]` userdata tokens; surrogate tables shaped with `[0] = frame[0]` dispatch shared frame methods, while `[1]`-only surrogates do not.
- Duplicate named `CreateFrame` calls produce fresh Lua objects and fresh identity tokens rather than copying stale custom fields from the prior global binding.
- XML bare `frameLevel` is an absolute initial value, not a parent-relative offset, but remains non-fixed so later parent level changes shift the child by the captured parent delta; `fixedFrameLevel="true"` pins the level.
- `DISPLAY_SIZE_CHANGED` and `UI_SCALE_CHANGED` fire as an ordered pair for observable size/scale recalculations, with startup pairs before `PLAYER_LOGIN`.

Key implementation locations:

- `src/lua_api/frame/methods/text_attribute_event/events.rs` — invalid `RegisterUnitEvent` filter fallback and animation handler validation.
- `src/lua_api/frame/methods/text_attribute_event/attributes.rs` — retail forbidden-frame and attribute behavior.
- `src/lua_api/methods.rs`, `src/lua_bridge/table_builder.rs`, `src/lua_api/globals/create_frame/helpers_shared.rs` — frame identity token dispatch and duplicate named-frame behavior.
- `src/lua_api/globals/template/direct/frame_level.rs` — XML frame-level resolution and fixed/non-fixed propagation.
- `src/lua_api/env_runtime.rs`, `src/startup.rs`, `src/iced_app/resize_event_tests.rs` — display/scale event pair behavior.

### Verification

Regression coverage exists in:

- `tests/admin_event_api.rs` — invalid unit-filter registration fallback.
- `tests/protected_frame_enforcement.rs` — retail `SetForbidden` no-op behavior.
- `tests/protected_attribute_enforcement.rs` — wildcard explicit-false lookup and repeated-false dispatch ordering.
- `tests/frame_level.rs` — Raise/Lower and raised-frame-level ordering.
- `src/iced_app/mouse_tests.rs` — GUI hover, `GetMouseFocus`/`GetMouseFoci`, and Raise/Lower focus ordering.
- `tests/security_api.rs`, `tests/frame_table_iteration.rs`, `tests/globals_legacy.rs` — frame identity slot, surrogate dispatch, opaque identity userdata, duplicate named-frame freshness.
- `tests/xml_frame_strata.rs` — XML `frameLevel` and `fixedFrameLevel` semantics.
- `src/iced_app/resize_event_tests.rs` — display/scale ordered-pair behavior.

### Same-size transition boundary

`ScaleEventProbe` captures screen and physical dimensions, UI-scale CVars, and `UIParent` scales with each ordered event pair; it does not need another dimensions capture. `App::sync_screen_size_to_state` receives only `iced::Size` during drawing and does nothing when both dimensions match. The pinned window stack has resize, move, focus, and scale-factor notifications but no mode/maximize/restore/fullscreen transition notification. There is therefore no production input that distinguishes an equal-size maximize/restore transition from no transition. Do not fire another pair merely because the same size is observed again, and do not add an admin event as a fidelity substitute.

### Remaining inert/default surface

There is no 12.0.5-specific inert-default module. Broad compatibility defaults still live in `src/lua_api/workarounds/temporary/` and permanent unsupported C API shims, but the 12.0.5 probe-backed findings listed above have modeled behavior and tests rather than patch-scoped inert stubs.

The remaining generic defaults are intentionally outside this 12.0.5 audit unless a probe or addon failure ties one to a 12.0.5 retail behavior contract. Examples include unsupported 3D/model domains, loose/placeholder namespace defaults, and compatibility fallbacks that are tracked by their own subsystem investigations.

### Audit state

This audit remains open with 4 evidence-required rows and 1 approved provenance-only exception-requested row. The four behavior gaps are one impossible same-size input-boundary gap and three unsafe Store/security gaps; they are not exception or approval candidates. Authoritative/live evidence or correct behavior is still required before this audit can close. No 12.0.5-specific inert-default module remains, but absence of a patch shim is not proof that every retained probe result has exact regression coverage.

## Sources

- [[retail-core-behavior-probes]] — core 12.0.5 live-client behavior findings.
- [[frame-surrogate-identity-slot]] — frame `[0]` identity-token behavior.
- [[display-size-ui-scale-events]] — display/scale event pair behavior.
- [XmlFrameLevelProbe](../../../docs/addons/XmlFrameLevelProbe/README.md) — live XML frame-level probe notes.
- [CoreBehaviorProbe](../../../docs/addons/CoreBehaviorProbe/README.md) — live core behavior probe notes.
- [FrameIdentityProbe](../../../docs/addons/FrameIdentityProbe/README.md) — live frame identity probe notes.
- [ScaleEventProbe](../../../docs/addons/ScaleEventProbe/README.md) — live display/scale event probe notes.
- [ScaleEventProbe source](../../../docs/addons/ScaleEventProbe/ScaleEventProbe.lua) — captured dimensions, CVars, and event order.
- [screen-size synchronization](../../../src/iced_app/update_runtime.rs) — simulator's size-only input boundary.
- [Cargo lockfile](../../../Cargo.lock) — pinned iced 0.14.0 and winit 0.30.12.

## See Also

- [[patch-12-0-7-api-audit]] — later additive API bridge audit pattern.
- [[patch-12-1-api-audit]] — PTR API bridge audit pattern.
- [[lua-api]] — Lua runtime surface and frame method dispatch.
- [[retail-core-behavior-probes]] — retained 12.0.5 core probe evidence.
- [[event-system]] — event registration/dispatch behavior.
- [[xml-template-system]] — XML template and frame-level handling.
