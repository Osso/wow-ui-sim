# Wrath StatusBar value callback boundary

Commits `9ea6555f8` and `98101cda3` cover ordinary StatusBar callbacks and texture rotation; `9198d4a0` adds XML `rotatesTexture` application; `e0720b992` clips vertical fills along the bar axis. `91cb5c735` fixes the independent normal-Wrath `c_unit_auras` enum-registration gate by matching it to the `aura-instance-enumeration` module feature. At `e0720b992`, targeted normal `gui,client-wrath` source GREEN covers vertical normal/reverse 2/2, vertical rotated atlas 1/1, and Lua orientation collection 1/1; horizontal rotation controls remain GREEN 2/2 within the overlapping `rotated_statusbar_` 3/3 filter. This is source evidence only: bottom-up normal and top-down reverse are inferred, not native-verified. Independent final normal `fmt`/`check` remains pending. The stock Classic consumer establishes the callback path; Solarity build 12340 is comparison, not authority for wow-ui-sim's interface-38001 profile. Native execution was not performed.

## Evidence

`src/lua_api/frame/methods/widgets/slider.rs` now commits a changed non-interpolated StatusBar value, releases the simulator borrow, then dispatches `OnValueChanged(self, value)` through the protected script path. StatusBars omit the Slider mouse-event argument. `text_attribute_event/events.rs` admits `OnValueChanged` for both widget types.

The cached Classic `Blizzard_TextStatusBar/Classic/TextStatusBar.xml` binds `OnValueChanged` to `TextStatusBar_OnValueChanged(self, value)`. Its Lua consumer ignores the supplied value and immediately calls `TextStatusBar_UpdateTextString(self)`, which rereads `GetValue()` and `GetMinMaxValues()`. This supports committed-state synchronous delivery, not additional first-value, range, interpolation, or error semantics.

`tests/widget_slider.rs::statusbar_value_change_updates_text_synchronously` is the direct ordinary callback regression. The pre-fix artifact `/tmp/wrath-statusbar-red-statusbar_value_change_updates_text_synchronously.log` records 0/1 at the required before-return update. Final source GREEN `/tmp/wrath-statusbar-postfix-widget_slider__.log` records the callback test inside `widget_slider::` 10/10. That original run is diagnostic; `/tmp/wrath-statusbar-xml-normal-widget-slider.log` subsequently passes the same ten cases under normal Wrath at `9198d4a0`.

Rotation source GREEN is five focused cases: two rendered partial-fill UV cases in `/tmp/wrath-statusbar-postfix-rotated_statusbar_.log`, two source-replacement/atlas-coordinate cases in `/tmp/wrath-statusbar-postfix-statusbar_rotat.log`, and one custom-coordinate adoption case in `/tmp/wrath-statusbar-postfix-statusbar_adopts_existing_custom_texcoords_without_rotation.log`.

`91cb5c735` adds the matching `aura-instance-enumeration` cfg to `c_unit_auras::register_sound_trigger_enum` in `enums.rs`, committing the normal-Wrath compile-blocker fix. At `9198d4a0`, a normal `gui,client-wrath` integration build passed, then two static XML tests and one runtime-template test changed from RED 0/2 + 0/1 to GREEN 2/2 + 1/1. The static tests prove direct `true` before `OnLoad`, template `true` inheritance before `OnLoad`, and explicit `false` override; the runtime test proves `CreateFrame` template application before `OnLoad`. Subsequent normal-profile verification passes the ten callback/Slider cases and three non-XML rotation-state/adoption cases. The two renderer unit cases retain diagnostic-feature proof only. The `statusbar_rotat` filter also selects one XML case; count that overlap once.

## Stage 1: Solarity comparison

Solarity's build 12340 documentation states that `CSimpleStatusBar` keeps separate range/value validity, that `SetValue` does nothing before range initialization, that the first valid value dispatches even when zero, and that callbacks expose current state without retaining a userdata borrow across Lua. Its regression covers callbacks, range limits, and other status-bar behavior.

wow-ui-sim's `client-wrath` reports interface 38001, while Solarity targets build 12340. Stage 1 therefore records only the shared design signal—commit state before callback and release Rust-held state before Lua—not Solarity's range validity, first-zero dispatch, `OnMinMaxChanged`, callback order, precision, or rendering contracts as Wrath semantics. Those target contracts remain uncorroborated.

## Coverage matrix

| Slice | Evidence | Status |
| --- | --- | --- |
| Ordinary callback | Pre-fix 0/1 RED; normal-profile `widget_slider::` 10/10 includes synchronous label update | Normal Wrath source GREEN |
| Texture rotation | Two replacement/atlas-coordinate and one custom-coordinate adoption case pass normal Wrath; two rendered UV cases pass diagnostic features | API state/adoption normal GREEN; renderer proof remains diagnostic |
| XML rotation | Normal `gui,client-wrath` build; RED 0/2 static + 0/1 runtime, then GREEN 2/2 + 1/1 | Direct `true`, inherited `true`, and explicit `false` override apply before `OnLoad`; runtime template covered |
| Vertical fill | At `e0720b992`, normal `gui,client-wrath`: normal/reverse 2/2, rotated atlas 1/1, collector 1/1; horizontal controls remain GREEN 2/2 in overlapping `rotated_statusbar_` 3/3 | Targeted source GREEN. Bottom-up normal/top-down reverse are inferred, not native-verified; independent final `fmt`/`check` pending |
| Normal Wrath lane | `91cb5c735` aligns `c_unit_auras` enum registration; `9198d4a0` independently passed 16 unique integration cases and fresh `fmt`/`check` | Vertical source proof is later and targeted; no full UI validation |
| Stage-1 contracts | Range validity, first-zero dispatch, `OnMinMaxChanged`, callback order, and exact native vertical direction | Uncorroborated for this target; not imported from Solarity |

Stage 1 remains open: the vertical implementation has targeted normal-Wrath source GREEN, but exact native direction plus range/initialization/order contracts need separate evidence and regressions. Cached Classic `Blizzard_ActionBar/Classic/OverrideActionBar.xml` health and power bars combine `orientation="VERTICAL"` with `rotatesTexture="true"`, corroborating a concrete consumer path.

## Later four stages

2. **Tooltip lifecycle:** ownership, line registration/reuse, visibility, clearing callbacks, sizing.
3. **Scrolling and text:** scroll-child clipping, sibling positioning, FontString dimensions, font mutations.
4. **Broader native-API audit:** compare implemented APIs and verification fixtures; rank remaining differences by stock UI/addon impact and retain gaps in both projects.
5. **Integrated Wrath validation:** affected stock panels and addon interactions, startup errors, rendering regressions. Preserve source-versus-native evidence distinctions.

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
- `/tmp/wrath-statusbar-vertical-proof.md` — `e0720b992` normal-Wrath vertical RED/GREEN ledger and horizontal control results.

## See Also

- [[client-profiles]] — profile interface and vendor boundaries.
- [[partyframe-statusbar-textures]] — separate StatusBar texture ownership issue.
- [[event-system]] — script dispatch path.
