# WoW Client Diff — Task Reference

Discovery data collected 2026-02-23 by running `WowDiscovery` and `WowBehaviorTest` addons against
live WoW client, then diffing against what the simulator exposes.

Source files in this directory:
- `WowDiscovery.lua` — addon that dumps live client API surface
- `WowBehaviorTest.lua` — addon that records actual values for edge-case behaviors
- `diff_summary.txt` — counts at a glance
- `diff_methods_missing.txt` / `diff_methods_extra.txt` — frame method gaps
- `diff_c_functions_missing.txt` / `diff_c_functions_extra.txt` — C_* function gaps
- `diff_c_namespaces_missing.txt` / `diff_c_namespaces_extra.txt` — C_* namespace gaps
- `diff_enums_missing.txt` / `diff_enums_missing_values.txt` / `diff_enums_extra.txt` — enum gaps
- `diff_constants_missing.txt` / `diff_constants_wrong.txt` / `diff_constants_extra.txt` — constant gaps
- `diff_global_functions_missing.txt` / `diff_global_functions_extra.txt` — global function gaps
- `generated_data_refresh_manifest.json` — tracked hash/size manifest for the two large generated Lua data files; bulk refreshes must update this in the same commit

## Current Method Snapshot Refresh (2026-10-10)

Correct complete comparison counts are **12,380 extra + 33 missing = 12,413 rows**, from original **11,985 extra + 22 missing = 12,007**. Whole saved469 extra panic lines 6871–7463 report **96 removals + 491 additions**; missing has **five removals + 16 additions**, totaling **608 changed rows**. Six retired minimap setters × 16 types account for 96 removals, not 14 types/84.

The earlier excerpt omitted 12 extra removals and all 491 extra additions. Main4528's interim reconstruction **11,901 + 33 = 11,934** and its 105-change completeness claim are incorrect. Older packet (removed evidence tree; recover with `git show c011c1d95:data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/method-full-snapshots/`) and receipts remain preserved as superseded, not full PASS. Fresh original assertions `030002`: extra **FAIL101** (12 removals + 491 additions), missing **PASS0**. Prior 21/21 documentation gate covered a bounded excerpt, not all rows; independent alpha diagnostic six-state PASS remains separate.

Corrected packet (removed evidence tree; recover with `git show c011c1d95:data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/method-full-snapshots-complete/`) retains full original/complete files, complete saved469 panic and interim FAIL receipts. Full generator `030142` completed **one named helper case, exit0**, independently audited, producing **12,380 extra + 33 missing = 12,413 rows**; original discovery and source artifact seal unchanged. Older packet is annotated superseded. Generation is not assertion proof. Corrected extra assertion `030409` passes after fifth-attempt resource admission; unchanged missing assertion reuses its `030002` PASS. Both original assertions are covered by one fresh plus one reused saved execution, not two new executions. Native discovery's **625 names / 3,522 type pairs / 16 types** is distinct from 12,413 comparison rows; native epoch/parity remains unknown. Historical suite FAIL and full handoff OPEN remain. See the [method investigation](../wiki/investigations/method-dispatch-refactor.md#current-investigation--saved469-and-alpha-receiver-2026-10-10). Historical counts below are unchanged.

### Cross-profile comparison follow-up

[Method comparison SSOT](../wiki/investigations/method-dispatch-refactor.md#cross-profile-comparisons--2026-10-11-utc) links the retained `031711` packet and exact six-profile matrix. Runtime capture audit PASS is separate from 1 PASS/11 FAIL comparator outcomes; shared Retail expectations are not a native/profile oracle. Retention audit PASS; historical suite FAIL and parent OPEN.

### Compiled-configuration method baselines

The [baseline selection contract](../specs/method-surface-baselines.md) requires each build to use only its own baseline: `ACTIVE.cache_subdir`, the highest actually enabled `retail-*` epoch cfg, and the `forbidden-aspects` / `on-update-modes` flags. No enabled Retail epoch means `no-retail-epoch`, not a metadata-derived epoch or a `1207` fallback. Default execution uses one Retail baseline; it does not loop over every profile or epoch.

`retail` / `retail-12-1-0` with both flags enabled retains the root `diff_methods_extra.txt` and `diff_methods_missing.txt`. Other configurations use `method-baselines/<profile>/<epoch>/forbidden-<N>-on-update-<N>/`, with flags encoded as `0` or `1`. The existing exact comparison logic and discovery remain unchanged. The ignored refresh helper creates the selected directory and writes only its two files, including when absent. Ordinary comparison assertions fail explicitly when the selected file cannot be read, without fallback or guessed fixtures.

**Bounded proof:** at `67598f430`, default Retail and explicitly selected Wrath each pass both exact assertions (four cases); independently inspected Wrath controls detect exactly one deleted row and an absent selected file (expected exit101 each). Ignored Wrath refresh passes; all 16 protected input hashes are restored/unchanged. Selected-only writes are source-inspected, separate from unchanged-byte execution proof. Default fmt/check exit0 with existing warnings. Retained receipts (removed evidence tree; recover with `git show c011c1d95:data/patch-api/evidence/narrow-repairs-2026-10-09/integrated/method-baseline-selection/`) preserve exact streams. Other five selected configurations have byte-verified copies, not fresh GREEN; older Retail epochs lack data/runtime proof. No matrix replay, native parity, model correctness, or receiver claim. Historical suite FAIL and parent OPEN remain.

Summary of gaps:

| Category | Missing | Extra |
|---|---|---|
| Frame methods | 1382 | 3810 |
| C_* functions | 208 | 152 |
| Global functions | 107 | 16 |
| Enums | 1430 | 2 |
| Constants | 1364 (72 wrong value) | 12 |
| C_* namespaces | 6 | 11 |

## Weekly Reconciliation Cycle

Run a full `diff_*.txt` reconciliation pass once per week, and also before any broad API-surface
implementation sprint that touches more than one of these categories:

- frame methods
- C_* APIs / namespaces
- global functions
- constants
- enums

### Pass Gate

Do not start or close a reconciliation pass unless every still-open gap category has an explicit
owner recorded for that pass. "No owner yet" is itself a blocker, not a pass result.

Each pass must leave behind:

- the pass date
- the exact `diff_*.txt` files reviewed
- the owner list for every open gap class
- any tasks opened or updated from the diff review
- the unresolved gaps carried forward to the next pass, with owners

### Owner List

Use the code-area owner list below when assigning responsibility for a pass:

| Gap Class | Owner Surface |
|---|---|
| Methods | `src/lua_api/frame/methods/`, `src/lua_api/frame/method_registry/`, `src/lua_api/frame/metatable.rs` |
| C_* APIs / namespaces | `src/lua_api/globals/c_*.rs`, `src/lua_api/globals/c_stubs_api*.rs`, `src/lua_api/globals/generated_stubs.rs` |
| Globals | `src/lua_api/globals/*.rs`, `src/lua_api/globals_legacy.rs`, `src/lua_api/globals/strings/` |
| Constants | `src/lua_api/globals/constants_api.rs`, `src/lua_api/globals/c_stubs_api_extra.rs`, `src/lua_api/globals/enum_data/missing_constants.lua` |
| Enums | `src/lua_api/globals/enum_api.rs`, `src/lua_api/globals/enum_data/`, `src/lua_api/globals/enum_data/missing_enums.lua` |

### Generated Data Refresh Gate

The two large generated Lua data files:

- `src/lua_api/globals/enum_data/missing_enums.lua`
- `src/lua_api/globals/enum_data/missing_constants.lua`

must stay pinned in [`generated_data_refresh_manifest.json`](./generated_data_refresh_manifest.json).
Any bulk refresh must update that manifest in the same commit, which makes the source, hash, byte
count, and line count visible in review instead of burying a huge generated diff by itself.

---

## Section 1: Behavior Divergences

One test failed out of 161. All passing tests record the confirmed live client values below —
useful as regression anchors.

**The one failure:**

- `animation.SetToFinalAlpha` — `GetToFinalAlpha` method is nil on the sim's Alpha animation.
  Live client has it. Implement `SetToFinalAlpha` / `GetToFinalAlpha` on Alpha animation objects.

**Notable confirmed behaviors (already correct in sim, keep as-is):**

- `SetPoint('CENTER')` with no relativeTo: returns `relativeTo_name = "nil"` and `is_parent = false`
  (nil, not the parent — this is intentional in live WoW)
- `SetAllPoints` creates exactly 2 anchors: TOPLEFT and BOTTOMRIGHT (in that canonical order)
- `GetPoint(0)` returns 0 values; `GetPoint(1)` returns 5 values (1-based indexing confirmed)
- Anchor canonical order for all 9 points: TOPLEFT, TOP, TOPRIGHT, LEFT, CENTER, RIGHT, BOTTOMLEFT, BOTTOM, BOTTOMRIGHT
- `RegisterEvent` returns `true` (boolean), 1 return value
- `RegisterEvent` is idempotent — registering twice then unregistering once fully unregisters
- `SetScale(0)` and `SetScale(-1)` both produce error: `"Frame:SetScale(): Scale must be > 0"`
- Effective alpha chain: 0.5 * 0.8 * 0.5 = 0.2 (confirmed multiplicative)
- Effective scale chain: 2.0 * 3.0 = 6.0 (confirmed multiplicative)
- `Raise()` does NOT change frame level (before=1, after=1)
- `Lower()` DOES change frame level — sets to parent's level - 1 (e.g. parent=6, child goes from 7 to 5)
- `SetIgnoreParentAlpha`: effective alpha before=0.5019, after=1 (confirmed working)
- `SetIgnoreParentScale`: effective scale before=2, after=1 (confirmed working)
- `GetScript` after `HookScript`: returns a wrapper function, not the original (same_as_original=false)
- `SetScript` after `HookScript`: the hook does NOT survive — only the new SetScript handler runs
- `HookScript` with no existing handler: creates a handler, does not error (ok=true, called=true)
- Scroll: negative scroll NOT clamped — `SetVerticalScroll(-50)` returns -50.0 (not 0)
- Scroll: out-of-range scroll NOT clamped — `SetHorizontalScroll(999)` returns 999.0
- Tooltip: `NumLines` returns 0 even after `AddLine` calls (counter not updated in our sim — investigate)
- Tooltip: `AddDoubleLine` counts as 0 lines in NumLines (same issue)
- Tooltip: `GetLine` returns text via named FontString regions, not positional returns
- `RegisterAllEvents`: does NOT make `IsEventRegistered("PLAYER_LOGIN")` return true
- `Mixin(t, nil)` errors: `"Usage: local outObject = Mixin(object, ...)\nLua Taint: ..."`
- `EditBox` auto focus default is `true` (not false)
- `frame.alpha_effective` with SetAlpha(0.5): `own=0.5019..` (float precision — `0.5` stored as nearest float)
- `frame.scale_effective`: parent=2.0, child=0.5 → effective=1.0 (2*0.5=1, not 0.5)
- `Mixin` overwrites frame methods — `GetName` on a frame with a mixin that has `GetName` returns the mixin value

**Behavior to fix — `tooltip.NumLines` / `tooltip.AddLine` / `tooltip.AddDoubleLine`:**

Live client: `NumLines` returns 0 before `SetOwner`. After `ClearLines`, returns 0. After `AddLine`x2,
test result shows `count=0` — this is the test checking NumLines with no owner set. The
`multiple_SetOwner` test confirms that second `SetOwner` clears lines (count=0). The issue is the
sim's `NumLines` may not be tracking correctly.

---

## Section 2: Missing Frame Methods

Source: `diff_methods_missing.txt` (1382 entries total).

The file contains methods that exist in live WoW but not in the sim. Many are on every widget type —
that indicates the sim already has them but does NOT have them type-specifically registered
(see Section 3 for the flip side).

### Reconcile Top Type Deltas First

Start the missing-method pass with the highest-count widget types below. These still match the
current `diff_methods_missing.txt` tallies and should be treated as the first ownership queue
before broad one-off method work.

| Widget Type | Missing Methods | Primary Owner Surface |
|---|---:|---|
| `PlayerModel` | 139 | `src/lua_api/frame/methods/widget_model.rs`, `src/lua_api/frame/method_registry/model.rs` |
| `Model` | 124 | `src/lua_api/frame/methods/widget_model.rs`, `src/lua_api/frame/method_registry/model.rs` |
| `EditBox` | 109 | `src/lua_api/frame/methods/widget_editbox.rs`, `src/lua_api/frame/method_registry/edit.rs` |
| `Cooldown` | 94 | `src/lua_api/frame/methods/widget_cooldown.rs`, `src/lua_api/frame/method_registry/controls.rs` |
| `GameTooltip` | 89 | `src/lua_api/frame/methods/widget_tooltip.rs`, `src/lua_api/frame/method_registry/tooltip.rs` |
| `ColorSelect` | 87 | `src/lua_api/frame/methods/widget_misc.rs`, `src/lua_api/frame/method_registry/tooltip.rs` |
| `SimpleHTML` | 86 | `src/lua_api/frame/methods/widget_misc.rs`, `src/lua_api/frame/method_registry/misc.rs` |
| `MessageFrame` | 85 | `src/lua_api/frame/methods/widget_message_frame.rs`, `src/lua_api/frame/method_registry/misc.rs` |
| `CheckButton` | 84 | `src/lua_api/frame/methods/widget_slider.rs`, `src/lua_api/frame/method_registry/frame.rs` |
| `Button` | 83 | `src/lua_api/frame/methods/methods_button.rs`, `src/lua_api/frame/methods/widget_slider.rs`, `src/lua_api/frame/method_registry/frame.rs` |
| `StatusBar` | 81 | `src/lua_api/frame/methods/widget_slider.rs`, `src/lua_api/frame/method_registry/controls.rs` |

Use this table as the handoff point for follow-up tasks:

- first reconcile ownership in the matching `method_registry/*.rs` file
- then implement or move the method in the paired `methods_*.rs` / `widget_*.rs` file
- only after that trim the remaining long tail types

**Methods that appear on all or most widget types — genuine engine methods to add:**

All Frame-type widgets are missing this cluster of methods (present on Frame, Button, CheckButton,
EditBox, ScrollFrame, Slider, StatusBar, GameTooltip, MessageFrame, ColorSelect, Cooldown, Model,
PlayerModel, SimpleHTML):

| Method | Category |
|---|---|
| `AbortDrag` | Input |
| `CanChangeAttribute` | Attribute |
| `CanPropagateMouseClicks` / `CanPropagateMouseMotion` | Input |
| `ClearAlphaGradient` / `SetAlphaGradient` / `HasAlphaGradient` / `GetAlphaGradient` | Visual |
| `ClearAttribute` / `SetAttributeNoHandler` / `ExecuteAttribute` | Attribute |
| `ClearParentKey` | Hierarchy |
| `CollapsesLayout` / `SetCollapsesLayout` | Layout |
| `CreateLine` | Drawing |
| `DesaturateHierarchy` | Visual |
| `DisableDrawLayer` / `EnableDrawLayer` | Visual |
| `DoesHyperlinkPropagateToParent` / `SetHyperlinkPropagateToParent` | Input |
| `EnableGamePadButton` / `EnableGamePadStick` / `IsGamePadButtonEnabled` / `IsGamePadStickEnabled` | Input |
| `GetAnimationGroups` | Animation |
| `GetBoundsRect` | Layout |
| `GetClampRectInsets` / `SetClampRectInsets` | Layout |
| `GetDontSavePosition` | Persistence |
| `GetEffectivelyFlattensRenderLayers` / `GetFlattensRenderLayers` | Visual |
| `GetHighestFrameLevel` / `GetRaisedFrameLevel` | Level |
| `GetHitRectInsets` / `SetHitRectInsets` | Input |
| `GetPointByName` | Anchor |
| `GetWindow` / `SetWindow` | Window |
| `HasAnySecretAspect` / `HasSecretAspect` / `HasSecretValues` / `IsPreventingSecretValues` / `SetPreventSecretValues` | Security |
| `InterceptStartDrag` | Input |
| `IsAnchoringRestricted` / `IsAnchoringSecret` | Anchor |
| `IsCollapsed` / `IsDragging` / `IsFrameBuffer` / `IsHighlightLocked` / `IsMouseMotionFocus` / `IsProtected` | State |
| `IsIgnoringChildrenForBounds` / `SetIgnoringChildrenForBounds` | Layout |
| `IsUsingParentLevel` / `SetUsingParentLevel` | Level |
| `Lower` / `Raise` | Level |
| `RegisterEventCallback` / `RegisterUnitEventCallback` | Event |
| `RotateTextures` | Visual |
| `SetCollapsesLayout` | Layout |
| `SetHighlightLocked` | State |
| `SetIsFrameBuffer` | Visual |
| `SetPointsOffset` | Anchor |
| `SetPropagateMouseClicks` / `SetPropagateMouseMotion` | Input |
| `SetToDefaults` | Misc |
| `ShouldButtonPassThrough` | Input |
| `StartMoving` / `StartSizing` / `StopMovingOrSizing` | Layout |

**Button-specific missing methods (not on Frame):**
`SetATTTooltip`, `StartATTCoroutine` — likely addon-injected (ATT = All the Things). Skip.
`SetFormattedText` — legitimate, also on CheckButton/EditBox/FontString.
`SetDisabledAtlas`, `SetHighlightAtlas`, `SetNormalAtlas`, `SetPushedAtlas` — atlas variants for button state textures.
`ClearDisabledTexture`, `ClearHighlightTexture`, `ClearNormalTexture`, `ClearPushedTexture` — clear state texture slots.
`SetHighlightLocked` / `IsHighlightLocked` — lock highlight state.

**CheckButton-specific:**
`GetDisabledCheckedTexture` / `SetDisabledCheckedTexture` — checked state for disabled buttons.

**Cooldown-specific (Cooldown widget is largely unimplemented):**
`GetCooldownDisplayDuration`, `GetCountdownFontString`, `GetDrawBling`, `GetDrawEdge`, `GetDrawSwipe`,
`GetEdgeScale`, `GetHideCountdownNumbers`, `GetMinimumCountdownDuration`, `GetUseAuraDisplayTime`,
`IsPaused`, `Pause`, `Resume`, `SetCooldownFromDurationObject`, `SetCooldownFromExpirationTime`,
`SetCooldownUNIX`, `SetCountdownAbbrevThreshold`, `SetCountdownFont`, `SetEdgeColor`,
`SetMinimumCountdownDuration`, `SetPaused`, `SetTexCoordRange`, `SetUseAuraDisplayTime`

**FontString-specific:**
`CalculateScreenAreaFromCharacterSpan`, `ClearText`, `FindCharacterIndexAtCoordinate`,
`GetFieldSize`, `GetFontHeight`, `GetLineHeight`, `GetScaleAnimationMode`, `GetTextScale`,
`OnColorsUpdated`, `SetFixedColor`, `SetFontHeight`, `SetNonSpaceWrap`, `SetScaleAnimationMode`,
`SetTextHeight`, `SetTextScale`, `SetTextToFit`, `SetVertexColorFromBoolean`, `CanNonSpaceWrap`

**Texture-specific:**
`ClearVertexOffsets`, `GetDesaturation`, `GetTextureFileID`, `GetTextureFilePath`, `GetVertexOffset`,
`IsBlockingLoadRequested`, `ResetTexCoord`, `SetBlockingLoadsRequested`, `SetMask`, `SetSpriteSheetCell`,
`SetVertexColorFromBoolean`, `SetVertexOffset`

**GameTooltip-specific:**
`AddAtlas`, `AddFontStrings`, `AddTexture`, `ClearPadding`, `CopyTooltip`, `GetCustomLineSpacing`,
`GetLeftLine`, `GetMinimumWidth`, `GetRightLine`, `IsOwned`, `SetAllowShowWithNoLines`,
`SetAnchorType`, `SetCustomLineSpacing`, `SetCustomWordWrapMinWidth`, `SetFrameStack`,
`SetObjectTooltipPosition`, `SetShrinkToFitWrapped`

**Model-specific (Model widget largely unimplemented):**
`AdvanceTime`, `ClearFog`, `ClearTransform`, `GetCameraDistance`, `GetCameraFacing`,
`GetCameraPosition`, `GetCameraRoll`, `GetCameraTarget`, `GetDesaturation`, `GetFogColor`,
`GetFogFar`, `GetFogNear`, `GetLight`, `GetModelAlpha`, `GetModelDrawLayer`, `GetModelFileID`,
`GetPaused`, `GetPitch`, `GetRoll`, `GetShadowEffect`, `GetViewInsets`, `GetViewTranslation`,
`GetWorldScale`, `HasAttachmentPoints`, `HasCustomCamera`, `IsUsingModelCenterToTransform`,
`MakeCurrentCameraCustom`, `ReplaceIconTexture`, `SetCameraDistance` (etc.), `SetCustomCamera`,
`SetFogColor` (etc.), `SetGlow`, `SetGradientMask`, `SetModelAlpha`, `SetModelDrawLayer`,
`SetParticlesEnabled`, `SetPaused`, `SetPitch`, `SetRoll`, `SetShadowEffect`, `SetTransform`,
`SetUseGBuffer`, `SetViewInsets`, `SetViewTranslation`, `TransformCameraSpaceToModelSpace`,
`UseModelCenterToTransform`

**PlayerModel adds over Model:**
`ApplySpellVisualKit`, `CanSetUnit`, `FreezeAnimation`, `GetDisplayInfo`, `GetDoBlend`,
`GetKeepModelOnHide`, `HasAnimation`, `PlayAnimKit`, `SetBarberShopAlternateForm`,
`SetDoBlend`, `SetItem`, `SetItemAppearance`, `SetKeepModelOnHide`, `StopAnimKit`, `ZeroCachedCenterXY`

---

## Section 3: Extra Frame Methods

Source: `diff_methods_extra.txt` (3810 entries total, 350 unique method names across 17 widget types).

**Root cause**: The sim registers all methods on all widget types instead of per-type. For example,
`GetScrollChild` (a ScrollFrame method) is also registered on Button, Frame, Cooldown, etc. Live WoW
only exposes each method on the widget types that actually have it.

**Methods per type with the most extras (sim registering wrong-type methods):**

| Widget Type | Extra Methods |
|---|---|
| Minimap | 350 |
| FontString | 263 |
| Texture | 247 |
| Frame | 229 |
| ColorSelect | 225 |
| SimpleHTML | 220 |
| ScrollFrame | 220 |
| Model | 216 |
| GameTooltip | 216 |
| StatusBar | 212 |
| Slider | 212 |
| Cooldown | 210 |
| MessageFrame | 209 |
| PlayerModel | 207 |
| Button | 196 |
| CheckButton | 192 |
| EditBox | 186 |

**Examples of cross-type pollution visible in the extra list:**
- `Button:GetScrollChild`, `Button:GetVerticalScroll` — ScrollFrame methods on Button
- `Button:GetCooldownDuration`, `Button:GetFillStyle` — Cooldown/StatusBar methods on Button
- `Button:GetModel`, `Button:GetCamDistanceScale` — Model methods on Button
- `Button:AddLine`, `Button:GetNumLines` — GameTooltip/MessageFrame methods on Button
- `Button:GetChecked`, `Button:GetCheckedTexture` — CheckButton methods on Button

**Task**: Audit the Lua API registration (`src/lua_api/`) to ensure methods are only registered on
the widget types that actually have them. The `wowless/data/products/wow/uiobjects.yaml` file has
the authoritative per-type method lists.

---

## Section 4: Missing C_* Functions

Source: `diff_c_functions_missing.txt` (208 entries). Filtered for real engine functions (not addon pollution).

**High-priority namespaces with many missing functions:**

**C_TooltipInfo** (76 functions missing — entire namespace absent):
`GetAction`, `GetInventoryItem`, `GetSpellBookItem`, `GetUnitAura`, `GetUnitBuff`, `GetUnitDebuff`,
`GetMerchantItem`, `GetQuestItem`, `GetLootItem`, `GetBagItemChild`, `GetCurrencyToken`, etc.
The entire `C_TooltipInfo` namespace needs to be added as stubs.

**C_PingSecure** (14 functions — secure ping system):
`CreateFrame`, `SendPing`, `GetTargetWorldPing`, `SetPingPinFrameAddedCallback`, etc.

**C_WoWLabsMatchmaking** (16 functions) and **C_WowLabsDataManager** (6 functions):
WoW Labs / plunderstorm matchmaking system.

**C_UnitAurasPrivate** (12 functions):
`GetAllPrivateAuras`, `GetAuraDataBySlot`, `AnchorPrivateAura`, `SetPrivateAuraAnchorAddedCallback`, etc.

**C_PetBattles** (46 functions — entire namespace largely absent):
`GetAbilityInfo`, `GetHealth`, `GetLevel`, `GetName`, `GetNumPets`, `ChangePet`, etc.

**C_EncounterEvents** / **C_EncounterTimeline** / **C_EncounterWarnings** (12 functions):
Encounter event/warning system.

**C_CombatLog** (10 functions):
`AddEventFilter`, `AdvanceEntry`, `GetCurrentEntryInfo`, `GetEntryCount`, etc.

**C_CombatLogSecure** (10 functions):
Secure combat log access.

**C_MacOptions** (9 functions):
Mac-specific options. Can be stubbed as returning false/nil.

**Smaller gaps (1-3 functions each):**
- `C_ActionBar.GetActionCount`, `C_ActionBar.GetCurrentActionBarByClass`
- `C_Calendar.CloseCalendar`, `C_Calendar.GetDate`, `C_Calendar.GetMaxDate`
- `C_ChallengeMode.GetCompletionInfo`
- `C_Console.GetAllCommands`, `C_Console.GetColorFromType`
- `C_EditMode.ConvertLayoutInfoToHyperlink`
- `C_EncounterJournal.GetEncounterInfo`, `C_EncounterJournal.GetInstanceInfo`
- `C_Guild.GetGuildInfo`, `C_Guild.GetMemberInfo`, `C_Guild.GetNumMembers`, `C_Guild.IsInGuild`
- `C_LFGList.IsSquelched`
- `C_Macro.GetMacroInfo`, `C_Macro.GetNumMacros`
- `C_Mail.GetNumItems`, `C_Mail.HasNewMail`
- `C_Minimap.SetPlayerTexture`, `C_Minimap.SetTrackingFilterByFilterIndex`
- `C_MountJournal.Summon`
- `C_MythicPlus.GetOverallDungeonScore`, `C_MythicPlus.GetSeasonInfo`
- `C_NamePlate.GetNamePlateEnemySize` / `GetNamePlateFriendlySize` / `GetNamePlateSelfSize` / `GetTargetClampingInsets` / `SetNamePlate*`
- `C_Navigation.GetDestination`, `C_Navigation.IsAutoFollowEnabled`, `C_Navigation.SetAutoFollowEnabled`
- `C_PlayerChoice.GetNumPlayerChoices`, `C_PlayerChoice.GetPlayerChoiceInfo`, `C_PlayerChoice.GetPlayerChoiceOptionInfo`
- `C_PlayerInfo.GetContentDifficultyQualityForPlayer`, `C_PlayerInfo.IsExpansionLandingPageUnlockedForPlayer`
- `C_Reputation.GetFactionInfo`
- `C_Sound.PlaySoundFile`
- `C_Transmog.GetAppliedSourceID`, `C_Transmog.GetSlotInfo`
- `C_TransmogCollection.GetNumMaxOutfits`, `C_TransmogCollection.GetOutfits`, etc.
- `C_Who.GetWhoInfo`, `C_Who.SendWho`, `C_Who.SetWhoToUi`

**Missing C_* namespaces**: `0`

`diff_c_namespaces_missing.txt` is now intentionally empty. The namespace-level gaps from the
earlier snapshot were reconciled; remaining C_* work is function-level, not table-level.

---

## Section 5: Missing Enums

**Count**: 0

**Extra enums**: `0`

`diff_enums_missing.txt` and `diff_enums_missing_values.txt` are intentionally empty now. The
bulk enum import already landed through `src/lua_api/globals/enum_data/missing_enums.lua`, and the
only live runtime gap this reconciliation found was `ClientDebugAISpellReadyStatus` plus its `Meta`
table. Those are now present at runtime too.

`diff_enums_extra.txt` is intentionally empty now too. The only two lingering extras from the older
snapshot, `ExpansionLandingPageType` and `TransmogOutfitFlags`, were simulator-only inventions with
no Blizzard UI references in this tree, so they were removed from the runtime surface rather than
kept as compatibility aliases.

---

## Section 6: Missing Constants

**Count**: 164 missing constants, 0 with wrong values.

**Wrong values**: `0`

`diff_constants_wrong.txt` is intentionally empty now. The earlier 72-entry list had gone stale after
the constant data refreshes landed: `LE_AUTOCOMPLETE_PRIORITY_*`, `LE_LFG_CATEGORY_*`,
`LE_GAME_ERR_*`, `ITEM_MOD_*`, `ITEM_BNETACCOUNTBOUND`, `ITEM_COOLDOWN_TIME`, `ITEM_REQ_*`,
`RAID_BOSSES`, and `SPELL_FAILED_NOT_READY` were already corrected in the shared string/constant data.

The one live runtime bug this audit did find was deprecated alias exposure: the simulator had the
correct backing values for `ChatFrameConstants` and `Enum.WorldElapsedTimerTypes`, but it was not
exporting the legacy globals `MAX_CHARACTER_NAME_BYTES`, `MAX_COMMUNITY_NAME_LENGTH`,
`MAX_COMMUNITY_NAME_LENGTH_NO_CHANNEL`, `LE_WORLD_ELAPSED_TIMER_TYPE_CHALLENGE_MODE`, and
`LE_WORLD_ELAPSED_TIMER_TYPE_PROVING_GROUND`. That alias gap is now fixed, and the diff file is kept
empty by regression tests.

**Missing constants**: 164 total. The `diff_constants_missing.txt` file has them with their WoW values.
The large `LE_GAME_*`, `LE_AUTOCOMPLETE_*`, and `LE_LFG_*` families are no longer tracked there because
[`constants_api.rs`](../../src/lua_api/globals/constants_api.rs) already loads
[`missing_constants.lua`](../../src/lua_api/globals/enum_data/missing_constants.lua) at startup, and
runtime coverage now proves those values resolve with the expected numbers. The remaining entries are
the still-unreconciled constants such as item quality color codes, `LE_CHARACTER_UNDELETE_RESULT_*`,
`LE_CHARACTER_UPGRADE_RESULT_*`, and other smaller families.

**Task**: Keep `diff_constants_wrong.txt` at zero and then reconcile the remaining 164 missing constants.

---

## Section 7: Missing Global Functions

**Count**: 107

`diff_global_functions_missing.txt` is now intentionally filtered. It tracks only Blizzard-owned
startup/API globals that still belong in `src/lua_api/globals` or `src/lua_api/globals_legacy.rs`,
and it explicitly drops addon pollution from third-party prefixes like `Angleur_`, `Details`,
`Auctionator`, `AllTheThings`, `DejunkBindings_`, `Plumber_`, `KrowiEVU_`, and similar leak paths.

The remaining missing surface is dominated by the chat stack:

- `ChatEdit_*`
- `ChatFrame_*`
- `Chat_*`

plus the small standalone helper set Blizzard still calls globally:

- `GetCVarTableValue`
- `SetCVarTableValue`
- `GetChatTimestampFormat`
- `GetMobileEmbeddedTexture`
- `ResolvePrefixedChannelName`
- `SubstituteChatMessageBeforeSend`

**Task**: implement the remaining filtered globals without re-expanding this file back into a raw
addon-polluted discovery dump.

This section does **not** own broader Blizzard panel/data blockers that happen to surface during
startup. Those belong in the behavioral backlog, not the global diff pass. Examples already moved
out of this bucket are the old section-29 items for:

- `C_FriendList`
- `C_Macro`
- `C_VideoOptions`
- `C_CombatLog`
- `C_ZoneAbility`
- `C_LossOfControl`

Keep `diff_global_functions_missing.txt` focused on actual missing global symbols. Do not re-expand
it to track panel breakage that is really caused by C_* namespace/data gaps.

---

## Section 8: Extra Stubs to Remove or Investigate

**Extra C_* namespaces**: `0`

`diff_c_namespaces_extra.txt` is now intentionally empty. The previous 11-entry list was a stale
discovery artifact: the simulator namespaces are used by Blizzard UI locally, and several also exist
in Blizzard APIDocumentation. Any remaining drift is tracked in `diff_c_functions_extra.txt`, not as
invented top-level namespace tables.

**Extra enums**: `0`

**Extra global functions** (16 in `diff_global_functions_extra.txt`):
This file is now curated to intentional compatibility aliases only. It no longer tracks addon leak
noise, Blizzard XML/script handler globals, simulator-only helpers like `FireEvent` / `SetScreenSize`
or the internal `__*` globals, and it no longer treats stale raw-diff names as worth owning.

The remaining compatibility aliases are grouped by owner surface:

- Combat log legacy wrappers in [`src/lua_api/globals/c_stubs_api_combat.rs`](../../src/lua_api/globals/c_stubs_api_combat.rs):
  `CombatLogAddFilter`, `CombatLogAdvanceEntry`, `CombatLogGetCurrentEntry`,
  `CombatLogGetCurrentEventInfo`, `CombatLogGetNumEntries`, `CombatLogSetCurrentEntry`
- Legacy container globals in [`src/lua_api/globals/c_container_api.rs`](../../src/lua_api/globals/c_container_api.rs):
  `GetContainerItemID`, `GetContainerItemLink`, `GetContainerNumSlots`
- Legacy item globals in [`src/lua_api/globals/c_item_api_globals.rs`](../../src/lua_api/globals/c_item_api_globals.rs):
  `GetItemID`, `GetTradeSkillTexture`, `IsArtifactRelicItem`
- Old-style aura globals in [`src/lua_api/globals/aura_api.rs`](../../src/lua_api/globals/aura_api.rs):
  `GetPlayerAuraBySpellID`, `UnitAura`, `UnitBuff`, `UnitDebuff`

Anything outside those four groups should not be in `diff_global_functions_extra.txt`.

**Extra C_* functions** (119 in `diff_c_functions_extra.txt`):
This file is now curated, not a raw discovery dump. It intentionally keeps only compatibility and
legacy surfaces that the simulator still exposes on purpose:
- renamed or moved API shims such as older `C_AchievementInfo`, `C_ActionBar`, `C_Calendar`, `C_Guild`,
  `C_Macro`, `C_Mail`, `C_PlayerChoice`, `C_PlayerInfo`, `C_Tutorial`, and `C_Who` wrappers
- intentionally kept legacy/private hooks such as `C_PingSecure`, `C_UnitAurasPrivate`, `C_NamePlate`,
  `C_CombatLog`, `C_CombatLogSecure`, and `C_MacOptions`
- simulator-side Blizzard UI compatibility shims such as `C_Login`, `C_Minimap`, `C_Sound`,
  `C_Transmog*`, and `C_ScenarioInfo`

Stale false positives that are already documented in Blizzard APIDocumentation or directly referenced
by Blizzard UI were removed from the file during this audit. Examples removed from the raw dump:
`C_CinematicList.GetUICinematicList`, `C_EditMode.ConvertLayoutInfoToHyperlink`,
`C_ContributionCollector.GetContributionCollector`, `C_MountJournal.Summon`,
`C_TradeSkillUI.GetTradeSkillLine`, `C_MythicPlus.GetOverallDungeonScore`,
`C_Reputation.GetFactionInfo`, `C_Scenario.GetCriteriaInfo`, and `C_VoiceChat.IsSpeakingText`.
