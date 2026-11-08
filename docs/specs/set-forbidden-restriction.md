# SetForbidden restriction

`FrameScriptObject:SetForbidden` marks a script object forbidden. This contract covers which client profiles treat an insecure (addon) call as a no-op, sourced from the pinned `Blizzard_APIDocumentationGenerated/SimpleFrameScriptObjectAPIDocumentation.lua` of each profile cache and from the pinned secure environment that calls the method. Implementation notes are in [Retail core behavior probes](../wiki/investigations/retail-core-behavior-probes.md).

## What it must do

- [x] On Retail, a tainted `SetForbidden(true)` and `SetForbidden(false)` on an addon-created normal frame return without error and leave `IsForbidden()` false. Native source: live Retail `12.0.5.67823` probe (`CoreBehaviorProbe.ForbiddenState`); declaration `SimpleFrameScriptObjectAPIDocumentation.lua:288-292` carries `HasRestrictions = true`.
- [x] On PTR, the same tainted calls are a no-op. The PTR `12.1.7.70493` copy of `SimpleFrameScriptObjectAPIDocumentation.lua` is byte-identical to the Retail copy (sha256 `09c85f7ee160d6f4d39765ba92a93ccd112d9e578b48d6f99b79f65e2ec42b1d`) and declares `SetForbidden` with `HasRestrictions = true` at `:288-292`; the pinned PTR secure environment guards its own call with `issecure()` (`Blizzard_RestrictedAddOnEnvironment/RestrictedExecution.lua:397-400`). No native PTR probe exists, so the PTR case is INFERRED from that identical declaration and consumer. `tests/protected_frame_enforcement.rs` asserts the observable result on both profiles.
- [x] On WowForever, the same tainted calls are a no-op. The WowForever `1.60.1.70707` copy of `SimpleFrameScriptObjectAPIDocumentation.lua` is byte-identical to the Retail copy (same sha256; profile provenance `~/.cache/wow-ui-sim/blizzard-ui/wowforever/AddOns/.wow-ui-sim-blizzard-ui-provenance` records `product=wowf`, `version=1.60.1.70707`) and declares `SetForbidden` with `HasRestrictions = true` at `:287-295`; its `Blizzard_RestrictedAddOnEnvironment/RestrictedExecution.lua:397-400` also reaches the call only under `issecure()`. No native WowForever probe covers `SetForbidden` (`handoff/native-evidence-70707` has no match), so the wowforever case is INFERRED from those bytes, like PTR.
- [x] Secure code keeps the effect: `SetForbidden(true)` marks the frame and `IsForbidden()` reports true (`tests/protected_frame_enforcement.rs::test_secure_set_forbidden_marks_frame_forbidden`).
- [x] The no-op never changes state: a following `SetForbidden(false)` from tainted code also leaves the frame un-forbidden.

## How it works

- [Retail core behavior probes](../wiki/investigations/retail-core-behavior-probes.md)
- [Protected frame enforcement](../protected-frame-enforcement.md)

## Implementation inventory

- `src/lua_api/frame/methods/text_attribute_event/attributes.rs` — `set_forbidden`; the secure-caller gate is compiled for `profile-retail`, `client-ptr` and `client-wowforever`.
- `src/lua_api/frame/methods/text_attribute_event/mod.rs` — unconditional `SetForbidden`/`IsForbidden` registration.
- `tests/common/profile_fixtures.rs` — `classic_secur_residue_set_forbidden_restriction_fixture!` selects the profiles that run the no-op case.

## Tests asserting this spec

- `tests/protected_frame_enforcement.rs` — `test_insecure_addon_created_normal_frame_set_forbidden_is_noop` (Retail, PTR and WowForever), `test_secure_set_forbidden_marks_frame_forbidden`.

## Known gaps (current cycle)

- [ ] Wrath runs the same case (`classic_secur_residue_set_forbidden_restriction_fixture!` keeps it enabled) and fails, because the gate does not compile there and Wrath has no pinned documentation file. Wrath is outside the current goal; the case is retained, not gated off.
- [ ] Mists, Era and Anniversary declare `SetForbidden` without `HasRestrictions` (same file `:116-121`), so the no-op is not asserted for them; their unrestricted behaviour is not independently verified.

## Out of scope

`CreateForbiddenFrame` availability, forbidden-state propagation to referenced frames, access-restriction masks, and any combat-lockdown interaction. Those have separate contracts.
