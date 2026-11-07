# Button enabled-state callbacks

`Enable`, `Disable`, and `SetEnabled` dispatch registered transition scripts through the simulator's existing binding model. Source: `src/lua_api/frame/methods/button_anchor_hierarchy/buttons.rs`; see [widget system](../widget-system.md).

## What it must do

- [x] On an enabled-state change, run intrinsic precall, normal script/hooks, and intrinsic postcall for `OnEnable` or `OnDisable`, with the button as the sole argument and committed state visible to handlers.
- [x] Retain unchanged-state no-ops and normal hook-only behavior.
- [x] Report handler errors and continue later bindings without undoing the state change.
- [x] On Wrath only, `IsEnabled()` returns the numbers `1` and `0`. Every other
  profile keeps the boolean. The 3.3.5a consumer compares the result with those
  literals in both directions: enabled must satisfy `== 1`
  (`FrameXML/MainMenuBarMicroButtons.xml:13`, `FrameXML/UIPanelTemplates.xml:76,84,92`,
  `FrameXML/LootFrame.xml:71`, `FrameXML/LFDFrame.xml:59`,
  `FrameXML/KnowledgeBaseFrame.xml:258`) and disabled must satisfy `== 0`
  (`FrameXML/ChannelFrame.lua:258`, `FrameXML/MainMenuBarMicroButtons.xml:246,257`,
  `FrameXML/PaperDollFrame.xml:1440`, `FrameXML/TutorialFrame.lua:835`). The same
  consumer set compares `IsEnabled() ~= 1` for disabled styling
  (`FrameXML/UIPanelTemplates.xml:68`) and `not self:IsEnabled()`
  (`FrameXML/VehicleMenuBar.lua:1006`), so the numeric shape is the wider match and
  those two sites are consumer bugs under it. Mists-era consumers use truthiness
  instead (`mists/AddOns/Blizzard_BuffFrame/BuffFrame.lua:602,1338-1343`), so the
  numeric shape is legacy-only. Test:
  `tests/tooltip_hover.rs::test_micro_menu_hover_shows_tooltip`.

## How it works

- [XML template system](../xml-template-system.md)
- [Event dispatch](../event-system.md)

## Implementation inventory

- `src/lua_api/frame/methods/button_anchor_hierarchy/buttons.rs`: state transitions and binding dispatch.
- `src/lua_api/script_helpers.rs`: existing ordered binding lookup, reused unchanged.

## Tests asserting this spec

`tests/methods_button/enabled_bindings.rs`, within the existing grouped integration target, loads XML pre/post declarations and confirms their presence through public `GetScript` binding queries before exercising transitions.

## Known gaps (current cycle)

Tests-only `960c20b32` is RED in two dispatch cases: registered intrinsic bindings are skipped, leaving only normal/hook calls. The normal hook-only control passes; there is no normal-hook omission bug. `/tmp/cross-version-button-enabled-bindings-proof.md` records exact revisions and outputs. Independent verification of `1d989b0a3` passes 62 scoped cases: 34 Button methods, 19 texture controls, seven CheckButton controls, and two HookScript taint/restriction controls. Format/check and grouped integration compilation pass without warnings; changed-file readability reports zero issues. Three deliberate error-control diagnostics remain in the Button test log. `/tmp/cross-version-button-enabled-bindings-verification-ledger.md` records exact commands, revisions and logs.

This corrects a demonstrated mismatch between registration and dispatch in the simulator's established XML binding model; it is not a fresh native-client probe. Intrinsic `HookScript` restrictions remain unchanged.

## Out of scope

`SetButtonState` semantics, argument coercion, opposite-state recursive transitions, native all-profile parity, and vendor/UI changes.
