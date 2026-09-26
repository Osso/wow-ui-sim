# Wrath StatusBar value callback boundary

Commit `9ea6555f8` adds ordinary StatusBar `OnValueChanged` dispatch, but the existing Wrath-profile regression remains RED. The implementation and stock Classic consumer establish the intended simulator path; Solarity build 12340 is a useful comparison, not authority for wow-ui-sim's interface-38001 profile. Native execution was not performed.

## Evidence

`src/lua_api/frame/methods/widgets/slider.rs` now commits a changed non-interpolated StatusBar value, releases the simulator borrow, then dispatches `OnValueChanged(self, value)` through the protected script path. StatusBars omit the Slider mouse-event argument. `text_attribute_event/events.rs` admits `OnValueChanged` for both widget types.

The cached Classic `Blizzard_TextStatusBar/Classic/TextStatusBar.xml` binds `OnValueChanged` to `TextStatusBar_OnValueChanged(self, value)`. Its Lua consumer ignores the supplied value and immediately calls `TextStatusBar_UpdateTextString(self)`, which rereads `GetValue()` and `GetMinMaxValues()`. This supports committed-state synchronous delivery, not additional first-value, range, interpolation, or error semantics.

`tests/widget_slider.rs::statusbar_value_change_updates_text_synchronously` is the direct ordinary callback regression. `/tmp/wrath-statusbar-red-statusbar_value_change_updates_text_synchronously.log` records it failing at the required before-return text update: 0 passed, 1 failed. GREEN is pending.

The normal `gui,client-wrath` compilation cannot currently run that lane because `enums.rs` references `c_unit_auras` behind an incompatible feature gate. A `gui,client-wrath,aura-instance-enumeration` build was diagnostic only; it does not validate the normal Wrath configuration or callback behavior.

## Stage 1: Solarity comparison

Solarity's build 12340 documentation states that `CSimpleStatusBar` keeps separate range/value validity, that `SetValue` does nothing before range initialization, that the first valid value dispatches even when zero, and that callbacks expose current state without retaining a userdata borrow across Lua. Its regression covers callbacks, range limits, and other status-bar behavior.

wow-ui-sim's `client-wrath` reports interface 38001, while Solarity targets build 12340. Stage 1 therefore records only the shared design signal—commit state before callback and release Rust-held state before Lua—not Solarity's initial-value, range, precision, ordering, or rendering contracts as Wrath semantics.

## Five-stage roadmap

1. **Completed: establish boundary.** Compare `9ea6555f8`, the Classic callback consumer, interface versions, and the captured ordinary-callback RED.
2. **Unblock normal profile compilation.** Resolve the independent `c_unit_auras` feature-gate failure without using the augmented diagnostic feature set as acceptance.
3. **Close ordinary dispatch.** Run the existing regression in normal `gui,client-wrath`; retain the RED artifact and record GREEN only if the label updates before `SetValue` returns.
4. **Expand only source-supported contracts.** Separately test same-value suppression, first/range initialization, min/max reclamping, callback reentry, and interpolation. Do not import Solarity behavior without matching Wrath evidence.
5. **Validate profile fidelity.** Exercise unchanged Classic TextStatusBar consumers and, if available, native 3.3.5 evidence; keep missing native execution explicit.

## Sources

- [StatusBar callback spec](../../specs/statusbar-value-callback.md) — scoped contract and current exclusions.
- [Slider/StatusBar setter](../../../src/lua_api/frame/methods/widgets/slider.rs) — commit implementation.
- [script-handler admission](../../../src/lua_api/frame/methods/text_attribute_event/events.rs) — widget eligibility.
- `/home/osso/.cache/wow-ui-sim/blizzard-ui/Blizzard_TextStatusBar/Classic/TextStatusBar.xml` — Classic binding.
- `/home/osso/.cache/wow-ui-sim/blizzard-ui/Blizzard_TextStatusBar/Classic/TextStatusBar.lua` — Classic text-update consumer.
- `/home/osso/Repos/solarityclient/docs/architecture/ui-content-loading.md` — build-12340 comparison.
- `/tmp/wrath-statusbar-red-statusbar_value_change_updates_text_synchronously.log` — recorded RED.

## See Also

- [[client-profiles]] — profile interface and vendor boundaries.
- [[partyframe-statusbar-textures]] — separate StatusBar texture ownership issue.
- [[event-system]] — script dispatch path.
