# Retail 12.0.5 supplemental publication sweep

A data-driven breadth probe covers every entry in the [12.0.5 wikitext register](../../data/patch-api/sources/12.0.5-wikitext-register.json). The register comes from the page's collapsed consolidated tables (raw wikitext revid 6747894). The plaintext extract and the [crawler register](../../data/patch-api/sources/12.0.5-register.json) kept only the changed entries of those tables and dropped all 216 added/removed rows. The sweep reuses the [12.0.7 sweep](patch-12-0-7-publication-sweep.md) classifier and proof limits: publication or absence only, never behavior.

## What it must do

- [x] Regenerate the register reproducibly from retained raw wikitext with `tools/gen_patch_wikitext_register.py`. 363 rows; header counts equal parsed counts in every section.
- [x] Parse changed entries that carry a documentation-system label (` PlayerScript {{api|GetHaste}}`, `Unit`, `Localization`): 53 of the 134 changed globals. Parse bare colored rename lines (`   <font>A -> B</font>`) as annotations. Neither change alters the committed 12.0.7 or 12.1.0 register entries.
- [x] Probe all rows in one fully loaded cached Game environment and compare the non-ok ID set exactly with `tests/data/patch_12_0_5_sweep_known_gaps.json`.
- [x] Apply later-patch supersession against the 12.0.7 then 12.1.0 wikitext registers, in that order. The latest add/remove row wins. The rule is in the [12.0.7 spec](patch-12-0-7-publication-sweep.md#later-patch-supersession-rule).

Five rows are superseded: `lastLockedDelvesCompanionAbilities` (removed again in 12.1.0), `nameplateShowFriends` (re-added in 12.1.0), `C_Spell.GetMawPowerBorderAtlasBySpellID` (removed again in 12.0.7; its deprecation fallback is accepted), `unlockedExpansionLandingPages` (re-added in 12.0.7) and `C_UnitAuras.RemovePrivateAuraAppliedSound` (removed in 12.1.0).

## Implementation inventory

- `tests/common/publication_sweep.rs`: shared classifier, supersession, results output and exact known-gap comparison.
- `tests/patch_12_0_5_publication_sweep.rs`: 12.0.5 inputs (`P1205_SWEEP_REGISTER` / `P1205_SWEEP_OUT`).
- `src/c_api/patch_retired_members.rs`: 12.0.5 and 12.0.7 removed namespace members, plus removed whole namespaces, marked at registration (`retail-12-0-5` / `retail-12-0-7`).
- `src/cvars.rs` (`PATCH_12_0_5_CVARS`, `PATCH_12_0_5_REMOVED_CVARS`) and `src/event/valid_events.rs` (12.0.5 added/removed registerable event), gated `retail-12-0-5`.

### Added object-kind factories

| Register owner | Factory |
|---|---|
| ModelSceneActor | `CreateFrame('ModelScene'):CreateActor()` |
| AbbreviateConfig | `CreateAbbreviateConfig()` |
| AbbreviatedNumberFormatter, NumericFormatter | `C_StringUtil.CreateAbbreviatedNumberFormatter()`. `NumericFormatter` is the interface this formatter implements. |
| NumericRuleFormatter | `C_StringUtil.CreateNumericRuleFormatter()` |
| HousingCatalogSearcher | `C_HousingCatalog.CreateCatalogSearcher()` |

## Fixes the sweep drove

- The namespace autostub fabricated removed members. `C_GossipInfo.GetActiveDelveGossip`, `C_GossipInfo.GetGossipDelveMapID` and `C_NamePlateManager.SetNamePlateHitTestFrame` are now retired. The whole `C_HousingPhotoSharing` namespace is now absent, because its members moved to `C_PhotoSharing`.
- CVars: the five `secret*RestrictionsForced` were renamed to `addon*RestrictionsForced` (default `0`). `cameraDistanceFixedValue` and `endeavorInitiativesLastPoints` were removed. `AllowSpectateMode`, `houseExterior_Hide_Decor`, `transmogPreviewedWeaponToggle` and `endeavorInitiativesLastPointsMap` were added. The last one has no default on the page, so the simulator uses an INFERRED empty-string default.
- Events: `HOUSE_EXTERIOR_DECOR_HIDDEN_CHANGED` is now registerable and `CATALOG_SHOP_PMT_IMAGE_DOWNLOADED` is no longer registerable.
- `HousingCatalogSearcher:ToggleStoredOnly` and `ToggleBaseVariantOnly` are now published.
- Added globals backed by simulator state: `C_AutoComplete.IsRecognizedName` and `GetAutoCompletePresenceID` (social candidates, Battle.net friends); `C_PartyInfo.GetLootMethodStyle` (Mainline ruleset); `C_StringUtil.GetDefaultAbbreviationBreakpoints` (abbreviated-formatter defaults); `C_DelvesUI.GetTieredEntranceType` (`tiered_entrance_type`, INFERRED Delve default); `C_ScenarioInfo.GetDisplayInfo` and `GetTieredEntranceActiveSpells` (scenario theme color and challenge spells); the `C_HousingInspectMode` namespace (inspect mode and hovered decor); `C_PhotoSharing.GetStatus` and `ClearAuthorization` (account-link flags).
- Added globals as workarounds: the seven `C_PhotoSharing` service members (OAuth flow, capture, upload) are permanent shims, because the simulator has no external service. `GetCurrentCinematicSummary` (no cinematic playback model) and `C_Commentator.SendAddonMessageLogged` (no commentator comms model) are temporary inert defaults.
- Methods: the 16 `SecondsFormatter` getters, curve setters, `Reset` and `FormatZero`; promotion and lowercase settings drive `Format`. `DurationObject:EvaluateTotalDuration` and `FontString:GetUnboundedStringWidthForText` (real text measurement in the font string's font) are also published. `ModelSceneActor:SetSheathedCategory`/`UseUnitSheatheCategory` and `ModelSceneActorBase:SetGradientMaskWithDyes` are permanent 3D no-ops.

Default Retail sweep: GREEN against the reviewed gap set. 363 observations: 351 OK, 12 non-OK. Results are in [the sweep result file](../../data/patch-api/evidence/12.0.5-session-2026-10-03/p1205-wikitext-sweep-result.json).

## Known gaps (current cycle)

These are startup-surface gaps, not proof of native-client absence. The cached Retail 12.1.0 API documentation declares every unpublished symbol below.

- [ ] `NamePlate` hit-test methods (5): nameplates are host-created per unit and have no Lua factory, so these rows are unprobeable.
- [ ] Removed methods that deprecation files republish as aliases of current native methods, with no Deprecated source to attribute: `HousingCatalogSearcher:IsOwnedOnlyActive`, `SetOwnedOnly`, `ToggleOwnedOnly` (`Deprecated_12_0_5.lua`), and `C_UnitAuras.RemovePrivateAuraAppliedSound` (the same gap exists in the 12.1.0 sweep).
- [ ] `CLASS_TALENTS_SWITCH_TO_LOADOUT_BY_INDEX`, `..._SPECIALIZATION_BY_INDEX` and `..._SPECIALIZATION_BY_NAME` are `callback: true, noscript: true` events in Wowless `events.yaml`. `RegisterEvent` rejects them, and the classifier has no callback-event probe. `..._LOADOUT_BY_NAME` registers through `RegisterEvent` because `src/event/valid_events_a.rs` lists it in the registerable `EVENTS_A` table, out of sort order, while Wowless and the cached `ClassTalentsDocumentation.lua` (`CallbackEvent = true`) treat all four alike. That inconsistency is unreviewed.
- [x] CVar `unlockedExpansionLandingPages` (re-added in 12.0.7) is published with an INFERRED `0` default.

## Out of scope

- Behavior, signatures, secrecy annotations, payloads and CVar mutability. Strict 12.0.5-epoch builds are also out of scope: the sweep runs only on the default Retail profile.
- The Enums and Structures sections, which the generator does not parse. Their crawler-register subjects all appear in the raw wikitext.
