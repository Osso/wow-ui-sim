# Wrath StatusBar value callback boundary

Commits `9ea6555f8` and `98101cda3` cover ordinary StatusBar callbacks and texture rotation; `9198d4a0` adds XML `rotatesTexture` application. `91cb5c735` fixes the independent normal-Wrath `c_unit_auras` enum-registration gate by matching it to the `aura-instance-enumeration` module feature. Diagnostic `gui,client-wrath,aura-instance-enumeration` source GREEN covers callbacks and five non-XML rotation cases; `9198d4a0` has separate targeted normal `gui,client-wrath` XML GREEN 3/3. Neither establishes full normal-profile acceptance: existing normal `fmt`/`check` predate the XML code and tests; fresh verification is pending. The stock Classic consumer establishes the callback path; Solarity build 12340 is comparison, not authority for wow-ui-sim's interface-38001 profile. Native execution was not performed.

## Evidence

`src/lua_api/frame/methods/widgets/slider.rs` now commits a changed non-interpolated StatusBar value, releases the simulator borrow, then dispatches `OnValueChanged(self, value)` through the protected script path. StatusBars omit the Slider mouse-event argument. `text_attribute_event/events.rs` admits `OnValueChanged` for both widget types.

The cached Classic `Blizzard_TextStatusBar/Classic/TextStatusBar.xml` binds `OnValueChanged` to `TextStatusBar_OnValueChanged(self, value)`. Its Lua consumer ignores the supplied value and immediately calls `TextStatusBar_UpdateTextString(self)`, which rereads `GetValue()` and `GetMinMaxValues()`. This supports committed-state synchronous delivery, not additional first-value, range, interpolation, or error semantics.

`tests/widget_slider.rs::statusbar_value_change_updates_text_synchronously` is the direct ordinary callback regression. The pre-fix artifact `/tmp/wrath-statusbar-red-statusbar_value_change_updates_text_synchronously.log` records 0/1 at the required before-return update. Final source GREEN `/tmp/wrath-statusbar-postfix-widget_slider__.log` records the callback test inside `widget_slider::` 10/10. That run is diagnostic, not normal-profile acceptance.

Rotation source GREEN is five focused cases: two rendered partial-fill UV cases in `/tmp/wrath-statusbar-postfix-rotated_statusbar_.log`, two source-replacement/atlas-coordinate cases in `/tmp/wrath-statusbar-postfix-statusbar_rotat.log`, and one custom-coordinate adoption case in `/tmp/wrath-statusbar-postfix-statusbar_adopts_existing_custom_texcoords_without_rotation.log`.

`91cb5c735` adds the matching `aura-instance-enumeration` cfg to `c_unit_auras::register_sound_trigger_enum` in `enums.rs`, committing the normal-Wrath compile-blocker fix. At `9198d4a0`, a normal `gui,client-wrath` integration build passed, then two static XML tests and one runtime-template test changed from RED 0/2 + 0/1 to GREEN 2/2 + 1/1. The static tests prove direct `true` before `OnLoad`, template `true` inheritance before `OnLoad`, and explicit `false` override; the runtime test proves `CreateFrame` template application before `OnLoad`. The `gui,client-wrath,aura-instance-enumeration` 15-case diagnostic proof (callback-containing `widget_slider::` 10/10 plus rotation 5/5) remains diagnostic only; it does not validate callbacks or the five non-XML rotation cases in normal Wrath.

## Stage 1: Solarity comparison

Solarity's build 12340 documentation states that `CSimpleStatusBar` keeps separate range/value validity, that `SetValue` does nothing before range initialization, that the first valid value dispatches even when zero, and that callbacks expose current state without retaining a userdata borrow across Lua. Its regression covers callbacks, range limits, and other status-bar behavior.

wow-ui-sim's `client-wrath` reports interface 38001, while Solarity targets build 12340. Stage 1 therefore records only the shared design signal—commit state before callback and release Rust-held state before Lua—not Solarity's initial-value, range, precision, ordering, or rendering contracts as Wrath semantics.

## Coverage matrix

| Slice | Evidence | Status |
| --- | --- | --- |
| Ordinary callback | Pre-fix 0/1 RED; post-fix `widget_slider::` 10/10 includes synchronous label update | Source GREEN under diagnostic features only |
| Texture rotation | Two renderer UV, two replacement/atlas-coordinate, and one custom-coordinate adoption case | Source GREEN 5/5 under diagnostic features only |
| XML rotation | Normal `gui,client-wrath` build; RED 0/2 static + 0/1 runtime, then GREEN 2/2 + 1/1 | Direct `true`, inherited `true`, and explicit `false` override apply before `OnLoad`; runtime template covered |
| Normal Wrath lane | `91cb5c735` aligns `c_unit_auras` enum registration; XML targeted build/tests pass | Callback and five non-XML rotation regressions, plus fresh normal `fmt`/`check`, pending |
| Stage-1 contracts | Range/first-value initialization, callback order, and vertical rendering direction | Unproven; not imported from Solarity |

Stage 1 is a bounded comparison plus diagnostic source proof, not complete profile or native validation.

## Later four stages

2. **Verify the remaining normal profile.** The normal XML build and 3/3 targeted tests pass; run fresh normal `fmt`/`check` after `9198d4a0`.
3. **Repeat ordinary and non-XML rotation regressions in normal Wrath.** Retain RED artifacts and record normal-lane GREEN only when callback delivery and all five non-XML rotation cases pass.
4. **Expand only source-supported contracts.** Separately establish same-value suppression, first/range initialization, min/max reclamping, callback order/reentry, interpolation, geometry, and vertical rendering direction. Do not import Solarity behavior without matching Wrath evidence.
5. **Validate profile fidelity.** Exercise unchanged Classic TextStatusBar consumers and, if available, native 3.3.5 evidence; keep missing native execution explicit.

## Sources

- [StatusBar callback spec](../../specs/statusbar-value-callback.md) — scoped contract and current exclusions.
- [Slider/StatusBar setter](../../../src/lua_api/frame/methods/widgets/slider.rs) — commit implementation.
- [script-handler admission](../../../src/lua_api/frame/methods/text_attribute_event/events.rs) — widget eligibility.
- [enum initialization](../../../src/lua_api/env_init/enums.rs) — `91cb5c735` feature-gate correction.
- `/home/osso/.cache/wow-ui-sim/blizzard-ui/Blizzard_TextStatusBar/Classic/TextStatusBar.xml` — Classic binding.
- `/home/osso/.cache/wow-ui-sim/blizzard-ui/Blizzard_TextStatusBar/Classic/TextStatusBar.lua` — Classic text-update consumer.
- `/home/osso/Repos/solarityclient/docs/architecture/ui-content-loading.md` — build-12340 comparison.
- `/tmp/wrath-statusbar-red-statusbar_value_change_updates_text_synchronously.log` — recorded callback RED.
- `/tmp/wrath-statusbar-postfix-widget_slider__.log` — diagnostic callback GREEN (10/10).
- `/tmp/wrath-statusbar-postfix-rotated_statusbar_.log` — diagnostic rendered rotation GREEN (2/2).
- `/tmp/wrath-statusbar-postfix-statusbar_rotat.log` — diagnostic source-replacement rotation GREEN (2/2).
- `/tmp/wrath-statusbar-postfix-statusbar_adopts_existing_custom_texcoords_without_rotation.log` — diagnostic adoption GREEN (1/1).
- `/tmp/wrath-statusbar-xml-proof.md` — normal `gui,client-wrath` XML RED/ GREEN ledger: 0/2 + 0/1 before `9198d4a0`, then 2/2 + 1/1.

## See Also

- [[client-profiles]] — profile interface and vendor boundaries.
- [[partyframe-statusbar-textures]] — separate StatusBar texture ownership issue.
- [[event-system]] — script dispatch path.
