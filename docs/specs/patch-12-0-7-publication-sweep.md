# Retail 12.0.7 supplemental publication sweep

A data-driven breadth probe covers every entry in the [12.0.7 wikitext register](../../data/patch-api/sources/12.0.7-wikitext-register.json), parsed from the page's collapsed consolidated tables that the plaintext extract (and the [crawler register](../../data/patch-api/sources/12.0.7-register.json)) dropped. It reuses the [12.1.0 sweep](patch-12-1-0-publication-sweep.md) classifier and proof limits: publication or absence only, never behavior.

## What it must do

- [x] Regenerate the register reproducibly from retained raw wikitext with `tools/gen_patch_wikitext_register.py` (the same tool reproduces the committed 12.1.0 register entry-for-entry). Record page header counts next to parsed counts, CVar page defaults as `page_default`, console commands as `kind: "command"`, and `{{Test-inline}}` rows as `test_inline`.
- [x] Probe all rows with the 12.1.0 classifier in one fully loaded cached Game environment and compare the non-ok ID set exactly with `tests/data/patch_12_0_7_sweep_known_gaps.json`.
- [x] Classify console commands as explicit non-ok `unprobeable` observations: no Lua surface proves a command exists.
- [x] Apply later-patch supersession (below) before computing expected publication.

## Later-patch supersession rule

The Retail profile builds the 12.1.0 surface, so a 12.0.7 row can only be judged against what 12.1.0 still publishes. For each 12.0.7 row, the latest added/removed row for the identical symbol in later wikitext registers (12.1.0 today) wins:

- 12.0.7 added/changed, later removed → expect absence (deprecation fallbacks accepted as for removed rows) and record `expected.superseded_by` with the later row ID. The ledger row becomes `superseded-by-later-patch`, never a gap.
- 12.0.7 removed, later re-added → expect publication, recorded the same way.
- A later `changed` row does not alter publication.

Matching is by exact symbol string. Today no 12.0.7 row has a later add/remove, so every expectation follows its own direction.

## Implementation inventory

- `tests/common/publication_sweep.rs` — shared classifier, supersession, results output and exact known-gap comparison.
- `tests/patch_12_0_7_publication_sweep.rs` — 12.0.7 inputs (`P1207_SWEEP_REGISTER` / `P1207_SWEEP_OUT`).
- `tools/gen_patch_wikitext_register.py` — wikitext table parser.

## Tests asserting this spec

- `tests/patch_12_0_7_publication_sweep.rs::patch_12_0_7_publication_sweep`

### Added object-kind factories

| Register owner | Factory |
|---|---|
| DurationObject | `C_DurationUtil.CreateDuration()` |
| DurationManualClock, DurationClock | `C_DurationUtil.CreateManualClock()`; the manual clock is the only Lua-constructible clock |
| Font | `CreateFont` with one fixed name |
| ModelSceneActorBase | `CreateFrame('ModelScene'):CreateActor()` |

Default Retail sweep: GREEN against the reviewed gap set, 174 observations (169 OK, 5 non-OK; the [12.0.5 sweep](patch-12-0-5-publication-sweep.md) follow-up published `C_DelvesUI.GetTieredEntranceType`, `SecondsFormatter:FormatZero` and `GetDefaultAbbreviation`; the p1207-gaps cycle closed the rows listed under Closed below). The sweep also exposed four removed namespace members that the crawler register never listed (`C_DurationUtil.GetCurrentTime`, `C_HousingLayout.IsDraggingStairwell`, `C_Minimap.GetObjectIconTextureCoords`, `C_Scenario.GetScenarioIconInfo`); they are now retired from raw and ordinary lookup.

## Known gaps (current cycle)

Startup-surface gaps, not proof of native-client absence. Every unpublished symbol below is declared by the cached Retail 12.1.0 API documentation unless noted.

- [ ] Added but unpublished: `C_ScenarioInfo.GetScenarioIconInfo`.
- [ ] `HousingLayoutPinFrame:IsConnectedToDraggingRoom` / `IsPartOfDraggingRoom`: the object is host-created (`HousingLayoutPinFrameAdded`), has no Lua factory, and the simulator has no housing layout pin/room-drag model, so both rows are unprobeable.
- [ ] Console commands `fetchBleepProxies` and `MemUsageStackTrace` (test-inline) are unprobeable by design: `C_Console.GetAllCommands` lists only CVars, and no console-command registry exists to probe.

### Closed (p1207-gaps)

- [x] Modeled: `C_QuestInfoSystem.GetQuestHasShortExpirationWarning` (host-flagged quests), `C_Spell.GetMawPowerRarityInfoBySpellID` (host rarity ID + border atlas), `C_EncounterWarnings.GetColorForSeverity` (model promoted to `retail-12-1-0`), `C_Club.SendBattleTagFriendRequest` (recorded per guild member), `SimulateMouseClick/Down/Up/Wheel` (secure callers queue input replayed by the GUI mouse handlers; insecure callers and forbidden / script-inaccessible / combat-protected foci are refused), CVars `unlockedExpansionLandingPages`, `Aftermath`, `AftermathCallstacks`, `enableMemoryTrap`.
- [x] Temporary workaround: `GetBaseDifficultyID` identity (no Difficulty.db2 variant model).
- [x] `C_Club.AssignMemberRole`, `GetAssignableRoles`, `KickMember`, `RevokeInvitation`, `SendInvitation`, `SetClubMemberNote` use the [club membership model](club-membership.md#development-proof--2026-10-05); 12 behavior tests and isolated publication sweep pass. Guild clubs still grant no community management privileges.
- [ ] SimulateMouse gaps: no gamepad limited-input event source (insecure calls always refused) and only LeftButton/RightButton dispatch.

## Out of scope

- Behavior, signatures, secrecy annotations, payloads and CVar mutability; strict 12.0.7-epoch builds (the sweep runs only on the default Retail profile).
- The 22 crawler-register symbols absent from every checked page revision's consolidated tables (see the [12.0.7 audit](../wiki/investigations/patch-12-0-7-api-audit.md)).
