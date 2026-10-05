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

Default Retail sweep: GREEN against the reviewed gap set, 174 observations (150 OK, 24 non-OK; the [12.0.5 sweep](patch-12-0-5-publication-sweep.md) follow-up published `C_DelvesUI.GetTieredEntranceType`, `SecondsFormatter:FormatZero` and `GetDefaultAbbreviation`). The sweep also exposed four removed namespace members that the crawler register never listed (`C_DurationUtil.GetCurrentTime`, `C_HousingLayout.IsDraggingStairwell`, `C_Minimap.GetObjectIconTextureCoords`, `C_Scenario.GetScenarioIconInfo`); they are now retired from raw and ordinary lookup.

## Known gaps (current cycle)

Startup-surface gaps, not proof of native-client absence. Every unpublished symbol below is declared by the cached Retail 12.1.0 API documentation unless noted.

- [ ] Added but unpublished: `C_QuestInfoSystem.GetQuestHasShortExpirationWarning`, `C_ScenarioInfo.GetScenarioIconInfo`, `C_Spell.GetMawPowerRarityInfoBySpellID`, `GetBaseDifficultyID`.
- [ ] Changed but unpublished (only the namespace autostub answers): `C_Club.AssignMemberRole`, `GetAssignableRoles`, `KickMember`, `RevokeInvitation`, `SendBattleTagFriendRequest`, `SendInvitation`, `SetClubMemberNote`; `C_EncounterWarnings.GetColorForSeverity` (modeled only under the `retail-12-1-5` gate); `SimulateMouseClick/Down/Up/Wheel`.
- [ ] `HousingLayoutPinFrame` methods: the object is host-created (`HousingLayoutPinFrameAdded`) and has no Lua factory, so both rows are unprobeable.
- [ ] CVars with no registered value/default: `unlockedExpansionLandingPages` and the `{{Test-inline}}` test-realm CVars `Aftermath`, `AftermathCallstacks`, `enableMemoryTrap`.
- [ ] Console commands `fetchBleepProxies` and `MemUsageStackTrace` (test-inline) are unprobeable by design.

## Out of scope

- Behavior, signatures, secrecy annotations, payloads and CVar mutability; strict 12.0.7-epoch builds (the sweep runs only on the default Retail profile).
- The 22 crawler-register symbols absent from every checked page revision's consolidated tables (see the [12.0.7 audit](../wiki/investigations/patch-12-0-7-api-audit.md)).
