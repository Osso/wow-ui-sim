# GameTooltip methods

This spec records the Retail `GameTooltip:GetMinimumWidth`, `SetMinimumWidth`, and `AddFontStrings` behavior. Native evidence is R88n.7 for Retail 12.1.5.70688. The pinned declaration is `Blizzard_APIDocumentationGenerated/FrameAPITooltipDocumentation.lua:10,47,94`.

## Profile scope

- **VERIFIED (simulator history, `b6a107994`):** only Retail uses the R88n.7 implementation. The `ClientProfile::ACTIVE` branches preserve the pre-lane getter in `line_data.rs` and `AddFontStrings` behavior in `owner.rs` for PTR, Forever, Mists, and Wrath.
- The API tests are optional and compile only with `client-retail`, `retail-12-1-5`, and `forbidden-aspects`. They do not assert native behavior for PTR, Forever, Mists, or Wrath.
- **UNKNOWN:** this spec does not certify native behavior outside Retail 12.1.5.70688.

## Minimum width

- [x] **VERIFIED (R88n.7):** Retail `GetMinimumWidth` uses `Usage: local width, forced = self:GetMinimumWidth()` and returns exactly the stored width number followed by the stored forced-state boolean.
- [x] **VERIFIED (R88n.7):** a new tooltip starts with width `0` and forced state `false`; the getter returns both values when width is zero.
- [x] **VERIFIED (simulator source):** the existing Retail `SetMinimumWidth` behavior stores both values.
- [ ] Test execution pending: the optional Retail API test asserts getter arity, defaults, and stored round trips.
- [ ] **FOLLOW-UP (R88n.7):** conditional return secrecy, diagnostic-state clearing, and return-access-state effects are not modeled. The evidence does not name the checked secret-aspect identity or the ordinary-call state predicate needed to implement the marking rule.
- **INFERRED (simulator diagnostic):** receiver conversion uses a fixed bad-self message envelope. R88n.7 says native decoration depends on the caller's runtime debug frame; tests assert the verified error reason, not the envelope.

## FontString pairs

- [x] **VERIFIED (R88n.7):** Retail `AddFontStrings` uses `Usage: self:AddFontStrings(leftFontString, rightFontString)`, reads the required tooltip and two FontStrings, and returns zero values on success.
- [x] **VERIFIED (R88n.7):** a successful call increments pair count, resizes line-state and numeric storage, initializes new numeric slots to zero, clears the new line-state marker, and resets layout progress to zero. It does not add a `NumLines` entry or assign the FontStrings' parents.
- [ ] Test execution pending: the optional Retail tests cover pair order, zero return values, unchanged parents, line-count separation, and the count-aligned reset mutation.
- [ ] Test execution pending: the Retail path checks `ChangeParent` and the six R88n.7 inherited aspects for each FontString, then checks `RemoveSecretAspects` before storage when `forbidden-aspects` is enabled.
- [ ] **FOLLOW-UP (R88n.7):** the conditional tooltip secret-state transition depends on saved Lua call state. The simulator has no corresponding saved-state input, so it does not guess this transition or its `Object did not allow secret.` path.
- **INFERRED (simulator retention):** `font_string_pairs` and its aligned arrays remain across `ClearLines`/Hide, and `CopyTooltip` copies them through `TooltipData::clone`. R88n.7 leaves later pair lifetime and copy behavior unknown.
- **UNKNOWN (R88n.7):** later layout, rendering, anchoring, and pointer lifetime use are not established.

## Original consumers

- **PRESERVATION (verified Retail selection):** `Blizzard_SharedXML.toc:154` selects `SharedTooltipTemplates.lua`; `GameTooltip_InsertFrame` uses `GetMinimumWidth()` as one scalar, so it observes only the first result. Its test preserves that consumer but is not a RED selector for getter arity.
- **VERIFIED (Retail cache search):** searching the pinned Retail `AddOns` tree found only the generated `AddFontStrings` declaration at `Blizzard_APIDocumentationGenerated/FrameAPITooltipDocumentation.lua:10`, not a Lua caller. The direct API test remains the producer test; no consumer is manufactured.

## How it works

- [Lua API: GameTooltip minimum width and FontString pairs](../wiki/systems/lua-api.md#game-tooltip-minimum-width-and-font-strings)
- [Consumer selection and RED proof](../wiki/investigations/consumer-selection-and-red-proof.md#gametooltip-minimum-width-consumer)

## Implementation inventory

- `src/lua_api/frame/methods/widgets/tooltip/line_data.rs` — profile-preserved getter, Retail getter, and argument conversion.
- `src/lua_api/frame/methods/widgets/tooltip/owner.rs` — profile-preserved legacy behavior and Retail `AddFontStrings` checks/storage.
- `src/lua_api/frame/methods/forbidden_aspects.rs` — the Retail `AddFontStrings` parent-change aspect check.
- `src/lua_api/tooltip.rs` — width fields, pair IDs, and count-aligned mutation state.
- `tests/tooltip_minimum_width.rs` — optional Retail-only API and preservation probes.
