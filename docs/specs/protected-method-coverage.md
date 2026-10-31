# Protected-method combat lockdown

The pinned Retail `Blizzard_APIDocumentationGenerated` declarations identify protected script-object methods. This contract covers their frame-backed simulator implementations and the shared combat-lockdown gate. It does not establish native parity.

## What it must do

- [ ] Documented methods on a frame blocked by the shared protection model must not mutate state when called by tainted code during combat. They emit `ADDON_ACTION_BLOCKED` with the taint owner and `FrameName:Method()` and reach Blizzard's `GameEvent.HandleActionBlocked` handler.
- [ ] Secure calls in combat and insecure calls outside combat proceed. `tests/protected_method_coverage.rs` covers changed method families with observable controls; registration cases inspect simulated registration state.
- [x] `ClearAttribute` removes an existing named attribute, returns true, and dispatches the same nil change as `SetAttribute`; absent attributes return false without a callback. The absent return and callback are **best-guess, native UNKNOWN**.
- [x] `ClearPointsOffset` sets both offsets to zero on each existing anchor and retains the point, target, and relative point. This is **best-guess, native UNKNOWN**; the Warcraft Wiki ScriptRegionResizing description says the method resets the region's x/y offset to zero.

## Coverage inventory

Generated from the pinned Retail documentation by scanning Lua files for `IsProtectedFunction = true` and associating each tag with its containing method's `Name`. Rows are grouped by documented script-object API. The baseline is `cacd9477180451bd9abd62e7f3e33f48ce59ccb0`; `wired` records source inspection of the shared gate, not proof for every object type or argument. Retail and PTR targeted results cover changed cases, not all profile/type combinations. `SetAllPoints` was gated only on Wrath at baseline and is marked not wired for Retail.

| Documented object | Method | Retail declaration | Baseline | Current |
| --- | --- | --- | --- | --- |
| FrameAPICooldown | `Clear` | `FrameAPICooldownDocumentation.lua:12` | not wired | wired |
| FrameAPICooldown | `SetCooldown` | `FrameAPICooldownDocumentation.lua:283` | not wired | wired |
| FrameAPICooldown | `SetCooldownDuration` | `FrameAPICooldownDocumentation.lua:297` | not wired | wired |
| FrameAPICooldown | `SetCooldownFromDurationObject` | `FrameAPICooldownDocumentation.lua:310` | not wired | wired |
| FrameAPICooldown | `SetCooldownFromExpirationTime` | `FrameAPICooldownDocumentation.lua:322` | not wired | wired |
| FrameAPICooldown | `SetCooldownUNIX` | `FrameAPICooldownDocumentation.lua:336` | not wired | wired |
| SimpleButtonAPI | `Disable` | `SimpleButtonAPIDocumentation.lua:56` | not wired | wired |
| SimpleButtonAPI | `Enable` | `SimpleButtonAPIDocumentation.lua:65` | not wired | wired |
| SimpleButtonAPI | `RegisterForClicks` | `SimpleButtonAPIDocumentation.lua:273` | not wired | wired |
| SimpleButtonAPI | `RegisterForMouse` | `SimpleButtonAPIDocumentation.lua:284` | not wired | wired |
| SimpleButtonAPI | `SetEnabled` | `SimpleButtonAPIDocumentation.lua:338` | not wired | wired |
| SimpleFrameAPI | `AddRoleset` | `SimpleFrameAPIDocumentation.lua:20` | not wired | wired |
| SimpleFrameAPI | `ClearAttribute` | `SimpleFrameAPIDocumentation.lua:54` | not wired | wired |
| SimpleFrameAPI | `ClearAttributes` | `SimpleFrameAPIDocumentation.lua:71` | not wired | wired |
| SimpleFrameAPI | `EnableGamePadButton` | `SimpleFrameAPIDocumentation.lua:227` | not wired | wired |
| SimpleFrameAPI | `EnableGamePadStick` | `SimpleFrameAPIDocumentation.lua:238` | not wired | wired |
| SimpleFrameAPI | `EnableKeyboard` | `SimpleFrameAPIDocumentation.lua:249` | not wired | wired |
| SimpleFrameAPI | `Hide` | `SimpleFrameAPIDocumentation.lua:688` | wired | wired |
| SimpleFrameAPI | `Lower` | `SimpleFrameAPIDocumentation.lua:992` | not wired | wired |
| SimpleFrameAPI | `Raise` | `SimpleFrameAPIDocumentation.lua:1001` | not wired | wired |
| SimpleFrameAPI | `RemoveRoleset` | `SimpleFrameAPIDocumentation.lua:1102` | not wired | wired |
| SimpleFrameAPI | `ResizeToBoundsRect` | `SimpleFrameAPIDocumentation.lua:1114` | wired | wired |
| SimpleFrameAPI | `SetClampRectInsets` | `SimpleFrameAPIDocumentation.lua:1194` | not wired | wired |
| SimpleFrameAPI | `SetClampedToScreen` | `SimpleFrameAPIDocumentation.lua:1208` | not wired | wired |
| SimpleFrameAPI | `SetFixedFrameLevel` | `SimpleFrameAPIDocumentation.lua:1250` | wired | wired |
| SimpleFrameAPI | `SetFixedFrameStrata` | `SimpleFrameAPIDocumentation.lua:1261` | wired | wired |
| SimpleFrameAPI | `SetFrameLevel` | `SimpleFrameAPIDocumentation.lua:1282` | wired | wired |
| SimpleFrameAPI | `SetFrameStrata` | `SimpleFrameAPIDocumentation.lua:1294` | wired | wired |
| SimpleFrameAPI | `SetHitRectInsets` | `SimpleFrameAPIDocumentation.lua:1315` | not wired | wired |
| SimpleFrameAPI | `SetHyperlinksEnabled` | `SimpleFrameAPIDocumentation.lua:1341` | not wired | wired |
| SimpleFrameAPI | `SetID` | `SimpleFrameAPIDocumentation.lua:1352` | not wired | wired |
| SimpleFrameAPI | `SetIgnoreParentScale` | `SimpleFrameAPIDocumentation.lua:1374` | not wired | wired |
| SimpleFrameAPI | `SetRolesets` | `SimpleFrameAPIDocumentation.lua:1460` | not wired | wired |
| SimpleFrameAPI | `SetScale` | `SimpleFrameAPIDocumentation.lua:1472` | wired | wired |
| SimpleFrameAPI | `SetShown` | `SimpleFrameAPIDocumentation.lua:1484` | wired | wired |
| SimpleFrameAPI | `SetToplevel` | `SimpleFrameAPIDocumentation.lua:1496` | wired | wired |
| SimpleFrameAPI | `SetUsingParentLevel` | `SimpleFrameAPIDocumentation.lua:1518` | not wired | wired |
| SimpleFrameAPI | `Show` | `SimpleFrameAPIDocumentation.lua:1539` | wired | wired |
| SimpleFrameAPI | `StartMoving` | `SimpleFrameAPIDocumentation.lua:1548` | not wired | wired |
| SimpleFrameAPI | `StartSizing` | `SimpleFrameAPIDocumentation.lua:1559` | not wired | wired |
| SimpleFrameAPI | `StopMovingOrSizing` | `SimpleFrameAPIDocumentation.lua:1571` | not wired | wired |
| SimpleFrameScriptObjectAPI | `SetToDefaults` | `SimpleFrameScriptObjectAPIDocumentation.lua:299` | not wired | wired |
| SimpleLineAPI | `ClearAllPoints` | `SimpleLineAPIDocumentation.lua:12` | wired | wired |
| SimpleRegionAPI | `SetIgnoreParentScale` | `SimpleRegionAPIDocumentation.lua:170` | not wired | wired |
| SimpleRegionAPI | `SetScale` | `SimpleRegionAPIDocumentation.lua:181` | wired | wired |
| SimpleScriptRegionAPI | `EnableMouse` | `SimpleScriptRegionAPIDocumentation.lua:75` | not wired | wired |
| SimpleScriptRegionAPI | `EnableMouseMotion` | `SimpleScriptRegionAPIDocumentation.lua:86` | not wired | wired |
| SimpleScriptRegionAPI | `EnableMouseWheel` | `SimpleScriptRegionAPIDocumentation.lua:97` | not wired | wired |
| SimpleScriptRegionAPI | `SetCollapsesLayout` | `SimpleScriptRegionAPIDocumentation.lua:574` | not wired | wired |
| SimpleScriptRegionAPI | `SetMouseClickEnabled` | `SimpleScriptRegionAPIDocumentation.lua:585` | not wired | wired |
| SimpleScriptRegionAPI | `SetMouseMotionEnabled` | `SimpleScriptRegionAPIDocumentation.lua:596` | not wired | wired |
| SimpleScriptRegionAPI | `SetParent` | `SimpleScriptRegionAPIDocumentation.lua:607` | wired | wired |
| SimpleScriptRegionAPI | `SetPassThroughButtons` | `SimpleScriptRegionAPIDocumentation.lua:619` | not wired | wired |
| SimpleScriptRegionAPI | `SetPropagateMouseClicks` | `SimpleScriptRegionAPIDocumentation.lua:631` | not wired | wired |
| SimpleScriptRegionAPI | `SetPropagateMouseMotion` | `SimpleScriptRegionAPIDocumentation.lua:643` | not wired | wired |
| SimpleScriptRegionAPI | `SetRoundLayoutToNearestPixel` | `SimpleScriptRegionAPIDocumentation.lua:655` | not wired | wired |
| SimpleScriptRegionResizingAPI | `AdjustPointsOffset` | `SimpleScriptRegionResizingAPIDocumentation.lua:12` | wired | wired |
| SimpleScriptRegionResizingAPI | `ClearAllPoints` | `SimpleScriptRegionResizingAPIDocumentation.lua:24` | wired | wired |
| SimpleScriptRegionResizingAPI | `ClearPoint` | `SimpleScriptRegionResizingAPIDocumentation.lua:34` | wired | wired |
| SimpleScriptRegionResizingAPI | `ClearPointsOffset` | `SimpleScriptRegionResizingAPIDocumentation.lua:45` | not wired | wired |
| SimpleScriptRegionResizingAPI | `SetAllPoints` | `SimpleScriptRegionResizingAPIDocumentation.lua:113` | not wired | wired |
| SimpleScriptRegionResizingAPI | `SetHeight` | `SimpleScriptRegionResizingAPIDocumentation.lua:126` | wired | wired |
| SimpleScriptRegionResizingAPI | `SetPoint` | `SimpleScriptRegionResizingAPIDocumentation.lua:137` | wired | wired |
| SimpleScriptRegionResizingAPI | `SetPointsOffset` | `SimpleScriptRegionResizingAPIDocumentation.lua:153` | wired | wired |
| SimpleScriptRegionResizingAPI | `SetSize` | `SimpleScriptRegionResizingAPIDocumentation.lua:165` | wired | wired |
| SimpleScriptRegionResizingAPI | `SetWidth` | `SimpleScriptRegionResizingAPIDocumentation.lua:177` | wired | wired |
| SimpleScrollFrameAPI | `SetHorizontalScroll` | `SimpleScrollFrameAPIDocumentation.lua:81` | not wired | wired |
| SimpleScrollFrameAPI | `SetScrollChild` | `SimpleScrollFrameAPIDocumentation.lua:93` | not wired | wired |
| SimpleScrollFrameAPI | `SetVerticalScroll` | `SimpleScrollFrameAPIDocumentation.lua:105` | not wired | wired |

## How it works

- [Protected frame enforcement](../protected-frame-enforcement.md)
- [Protected anchor targets](protected-anchor-targets.md)

## Implementation inventory

- `src/lua_api/frame/methods/methods_helpers.rs`: shared protection decision and blocked-event dispatch.
- `src/lua_api/frame/methods/`: frame, button, region, cooldown, scroll, attribute, and input methods.

## Tests asserting this spec

- `tests/protected_method_coverage.rs`: targeted Retail and PTR tests for changed method families and Blizzard blocked-event routing.
- `tests/protected_frame_enforcement.rs` and `tests/protected_anchor_targets.rs`: prior protection and anchor contracts.

## Known gaps (current cycle)

- [ ] `ClearPointsOffset`: native per-anchor versus separately stored offset, interaction with `SetPointsOffset`/`AdjustPointsOffset`, and events remain unknown.
- [ ] `ClearAttribute`: native callback and absent-value return are unknown.
- [ ] `C_TooltipComparison.CompareItem` is a documented protected namespace function, not a frame method; no obvious protected target is established.
- [ ] Global protected functions and `ADDON_ACTION_FORBIDDEN` are outside this lane; their native list and semantics are unknown.
- [ ] The inventory is a source audit, not runtime proof for every script-object subtype or client profile.

## Out of scope

Global functions, forbidden-action policy, and vendor Lua changes.
