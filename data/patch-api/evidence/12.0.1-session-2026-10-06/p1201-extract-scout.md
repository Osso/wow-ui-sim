# 12.0.1 non-inventory extract scout — 2026-10-06

## Scope and proof boundary

Retained page **659762**, revision **6747895** (2026-06-19T08:48:51Z), contains Summary, Resources, three Blue posts dated 2026-02-19/2026-02-24/2026-03-21, consolidated Enums and Structures. No standalone Deprecated API section is present; the Resources link is metadata. No external page, Discord thread, patch-note page or linked deprecated source was expanded.

The source compares 12.0.0 build 65655 to 12.0.1 build 66838 on April 3, 2026. These historical build/post dates are not the retrieval date. Capture/review took place October 6, 2026.

[Wikitext](../../sources/12.0.1-api-changes.wikitext) → [plaintext](../../sources/12.0.1-api-changes.txt) uses `tools/extract_patch_non_inventory.py --patch 12.0.1 --text-only`. Only the Global API through CVars inventories are omitted; Enums/Structures and Blue-post nested bullets remain. Template rendering keeps qualified API identity, spell IDs, comma-separated IDs, operators and typo spellings. Existing 12.0.0 plaintext regenerates byte-identically. A comparison regression prevents HTML stripping from eating `currentCharges < maxCharges and startTime > 0`; a classification regression keeps nested cooldown formulas audit-pending, not editorial metadata.

Every nonblank plaintext line has one ID with its original generated line number, blanks included. Seven enum parents plus 30 deltas; 21 structure parents plus 33 deltas. Parent rows require complete declared output/metadata proof, not only the listed changed fields. All **201 contractual rows remain audit-pending**; **51 editorial rows are metadata-only**. No new extract behavior or numeric-publication credit is awarded. Inspection of existing test bodies is discovery evidence, not a fresh runtime pass or native oracle.

[Machine scout](p1201-extract-row-scout.json) retains every statement, exact ID, batch, candidate tests and bounded next proof. [New page ledger](../../sources/12.0.1-page-coverage.json) contains 477 unique IDs: 225 inventories plus 252 extract lines. Inventory sweep credits stay publication-only: 120 partial-development-green / 38 bounded-coverage / 16 superseded metadata. Whole-page counts: 120 partial-development-green / 38 bounded-coverage / 252 audit-pending / 67 metadata-only.

## Coverage matrix

| Class | Pending | Metadata | Proof boundary |
|---|---:|---:|---|
| ENUM | 37 | 0 | Seven full parents, values, renamed/removed keys and Meta; current-vs-historical epoch reconciliation |
| STRUCT | 54 | 0 | 21 parent contracts, 33 deltas; populated producers, types, nullability, optional omissions and snapshots |
| PROSE-DATA | 74 | 0 | Spell list and declassification policy; hotfix data/caller contexts, not function existence |
| PROSE-MODELABLE | 36 | 0 | Cooldown, aura lifetime, formatting, secret identity, combat, chat and macro boundaries |
| METADATA | 0 | 51 | Headings, links, greetings, build context and design rationale |
| **Total** | **201** | **51** | **252 extract occurrences** |

## Top proof batches

One effort unit means one bounded fixture/proof pass, not a native-parity promise or time estimate. Parent closure is conditional on complete output/type assertions; existing tests do not automatically close these source IDs.

| Rank | Batch | Rows | Effort units | Next proof |
|---|---|---:|---:|---|
| 1 | B01 enums | 37 | 1 | Reuse loaded numeric publication pattern; freeze seven parent tables/Meta and old-key absence, distinguish later Tooltip enum drift |
| 2 | B02 damage-meter DTOs | 21 | 1 | Reuse actual host aggregate/detail fixtures; add exact field values/secrecy/snapshots and explicit missing-GUID control |
| 3 | B04 cooldown hotfix | 19 | 2 | Verify all boolean formulas, inactive zero-span durations, LoC replacement, aura-derived cooldowns and cached secure-delegate removal |
| 4 | B03 catalog DTOs | 9 | 1 | Reuse fully populated product/display; prove bundle quantity, disclaimer and old URL-key absence |
| 5 | B05 other DTOs | 24 | 4 | Eleven remaining parents and 13 deltas: concrete producers, optional/type and old-key assertions |

Later: B06 spell-policy **74/6 units**, B07 remaining security/token/aura/macro **17/5 units**. Total batch assignment is exactly 201, every pending occurrence once.

## Reusable inspected proof and limits

- `tests/startup_targeted_regressions/damage_meter.rs::damage_meter_explicit_aggregate_and_detail_shapes_are_distinct` checks populated session/source/spell/unit shapes. `damage_meter_explicit_meter_input_and_optional_fields_are_preserved` seeds `source_guid=None`, omits it in returned DTOs and exercises missing selectors. Presence alone does not prove every added numeric field or producer transition.
- `tests/catalog_shop_product_structures.rs::fully_populated_product_has_exact_declared_shape` includes explicit `isVCProduct=false` and `containsHousingItem=true`; display shape includes `productPMTURL`. Bundle/section changes need their own populated producers and old-key absence.
- `tests/action_loss_control_cooldown_info.rs` authenticates three VM-secret numeric payloads and ordinary `isActive`/`shouldReplaceNormalCooldown` flags. Related spell/action cooldown files contain real value/secrecy assertions. Cached `ActionButton_ApplyCooldown`, tainted frame setters and cooldown-aura interaction still need exact retained-post boundary proofs.
- `tests/secret_string_formatting.rs` verifies full secret payload retention through direct and positional precision/width formatting, with trusted host inspection. Its retained 12.0.5 contract was brought forward by this page; no new 12.0.1 proof is credited here. `Texture:IsDesatured()` is the literal source spelling: do not silently claim an `IsDesaturated` test proves that occurrence.
- `tests/patch_12_1_0_enums.rs` is the candidate full cached enum/Meta probe. Historical 12.0.0 enum tests are not a 12.0.1 oracle; only source/test-body inspection was performed for this scout.

## Data/policy blockers

The declassification list has **73 list occurrences**, **76 explicit numeric IDs** (76 unique) plus a wildcard Mythic Plus teleport occurrence without IDs; the separate policy paragraph makes 74 rows. Comma-separated Earth Shield/weapon/guard rows need every listed ID tested. No spell-data hotfix was inferred or installed. Later re-protection is announced as possible, not established here.

Aura-instance lifetime after full update, debuglocals crash behavior, status-bar secret laundering, chat lockdown and undocumented PvP restrictions need production-fidelity controls. Macro world-marker limits require rate-window input/delivery proof; encounter BNet whisper denial requires tainted macro dispatch, not publication-only tests. Private-aura changes require rejected mutation state to remain unchanged. These are independent next batches, not blockers to source/register/sweep completion.

## Every-row assignment

The machine scout is authoritative for statement text and next proof. This index covers all 252 occurrence IDs without granting behavioral credit.

### B01 — Enum values, renamed/removed keys and Meta (37)

| Source ID | Plaintext line | Status |
|---|---:|---|
| `enumerations-Enum-CombatAudioAlertSpecSetting-215` | 215 | audit-pending |
| `enumerations-Enum-CombatAudioAlertSpecSetting-216` | 216 | audit-pending |
| `enumerations-Enum-CombatAudioAlertSpecSetting-217` | 217 | audit-pending |
| `enumerations-Enum-CombatAudioAlertSpecSetting-218` | 218 | audit-pending |
| `enumerations-Enum-CombatAudioAlertSpecSetting-219` | 219 | audit-pending |
| `enumerations-Enum-CooldownViewerAlertEventType-220` | 220 | audit-pending |
| `enumerations-Enum-CooldownViewerAlertEventType-221` | 221 | audit-pending |
| `enumerations-Enum-CooldownViewerAlertEventType-222` | 222 | audit-pending |
| `enumerations-Enum-DamageMeterType-223` | 223 | audit-pending |
| `enumerations-Enum-DamageMeterType-224` | 224 | audit-pending |
| `enumerations-Enum-DamageMeterType-225` | 225 | audit-pending |
| `enumerations-Enum-NeighborhoodInviteResult-226` | 226 | audit-pending |
| `enumerations-Enum-NeighborhoodInviteResult-227` | 227 | audit-pending |
| `enumerations-Enum-PartyRequestJoinRelation-228` | 228 | audit-pending |
| `enumerations-Enum-PartyRequestJoinRelation-229` | 229 | audit-pending |
| `enumerations-Enum-SecretAspect-230` | 230 | audit-pending |
| `enumerations-Enum-SecretAspect-231` | 231 | audit-pending |
| `enumerations-Enum-SecretAspect-232` | 232 | audit-pending |
| `enumerations-Enum-TooltipDataLineType-233` | 233 | audit-pending |
| `enumerations-Enum-TooltipDataLineType-234` | 234 | audit-pending |
| `enumerations-Enum-TooltipDataLineType-235` | 235 | audit-pending |
| `enumerations-Enum-TooltipDataLineType-236` | 236 | audit-pending |
| `enumerations-Enum-TooltipDataLineType-237` | 237 | audit-pending |
| `enumerations-Enum-TooltipDataLineType-238` | 238 | audit-pending |
| `enumerations-Enum-TooltipDataLineType-239` | 239 | audit-pending |
| `enumerations-Enum-TooltipDataLineType-240` | 240 | audit-pending |
| `enumerations-Enum-TooltipDataLineType-241` | 241 | audit-pending |
| `enumerations-Enum-TooltipDataLineType-242` | 242 | audit-pending |
| `enumerations-Enum-TooltipDataLineType-243` | 243 | audit-pending |
| `enumerations-Enum-TooltipDataLineType-244` | 244 | audit-pending |
| `enumerations-Enum-TooltipDataLineType-245` | 245 | audit-pending |
| `enumerations-Enum-TooltipDataLineType-246` | 246 | audit-pending |
| `enumerations-Enum-TooltipDataLineType-247` | 247 | audit-pending |
| `enumerations-Enum-TooltipDataLineType-248` | 248 | audit-pending |
| `enumerations-Enum-TooltipDataLineType-249` | 249 | audit-pending |
| `enumerations-Enum-TooltipDataLineType-250` | 250 | audit-pending |
| `enumerations-Enum-TooltipDataLineType-251` | 251 | audit-pending |

### B02 — Damage-meter producer DTOs (21)

| Source ID | Plaintext line | Status |
|---|---:|---|
| `structures-DamageMeterAvailableCombatSession-265` | 265 | audit-pending |
| `structures-DamageMeterAvailableCombatSession-266` | 266 | audit-pending |
| `structures-DamageMeterCombatSession-267` | 267 | audit-pending |
| `structures-DamageMeterCombatSession-268` | 268 | audit-pending |
| `structures-DamageMeterCombatSession-269` | 269 | audit-pending |
| `structures-DamageMeterCombatSessionSource-270` | 270 | audit-pending |
| `structures-DamageMeterCombatSessionSource-271` | 271 | audit-pending |
| `structures-DamageMeterCombatSource-272` | 272 | audit-pending |
| `structures-DamageMeterCombatSource-273` | 273 | audit-pending |
| `structures-DamageMeterCombatSource-274` | 274 | audit-pending |
| `structures-DamageMeterCombatSource-275` | 275 | audit-pending |
| `structures-DamageMeterCombatSource-276` | 276 | audit-pending |
| `structures-DamageMeterCombatSource-277` | 277 | audit-pending |
| `structures-DamageMeterCombatSpell-278` | 278 | audit-pending |
| `structures-DamageMeterCombatSpell-279` | 279 | audit-pending |
| `structures-DamageMeterCombatSpell-280` | 280 | audit-pending |
| `structures-DamageMeterCombatSpell-281` | 281 | audit-pending |
| `structures-DamageMeterCombatSpellUnitDetails-282` | 282 | audit-pending |
| `structures-DamageMeterCombatSpellUnitDetails-283` | 283 | audit-pending |
| `structures-DamageMeterCombatSpellUnitDetails-284` | 284 | audit-pending |
| `structures-DamageMeterCombatSpellUnitDetails-285` | 285 | audit-pending |

### B03 — Catalog shop DTO additions/rename (9)

| Source ID | Plaintext line | Status |
|---|---:|---|
| `structures-CatalogShopBundleChildInfo-256` | 256 | audit-pending |
| `structures-CatalogShopBundleChildInfo-257` | 257 | audit-pending |
| `structures-CatalogShopProductDisplayInfo-258` | 258 | audit-pending |
| `structures-CatalogShopProductDisplayInfo-259` | 259 | audit-pending |
| `structures-CatalogShopProductInfo-260` | 260 | audit-pending |
| `structures-CatalogShopProductInfo-261` | 261 | audit-pending |
| `structures-CatalogShopProductInfo-262` | 262 | audit-pending |
| `structures-CatalogShopSectionInfo-263` | 263 | audit-pending |
| `structures-CatalogShopSectionInfo-264` | 264 | audit-pending |

### B04 — Cooldown hotfix contracts and secure delegation (19)

| Source ID | Plaintext line | Status |
|---|---:|---|
| `prose-2026-03-21-167` | 167 | audit-pending |
| `prose-2026-03-21-173` | 173 | audit-pending |
| `prose-2026-03-21-174` | 174 | audit-pending |
| `prose-2026-03-21-176` | 176 | audit-pending |
| `prose-2026-03-21-177` | 177 | audit-pending |
| `prose-2026-03-21-179` | 179 | audit-pending |
| `prose-2026-03-21-180` | 180 | audit-pending |
| `prose-2026-03-21-181` | 181 | audit-pending |
| `prose-2026-03-21-182` | 182 | audit-pending |
| `prose-2026-03-21-183` | 183 | audit-pending |
| `prose-2026-03-21-185` | 185 | audit-pending |
| `prose-2026-03-21-186` | 186 | audit-pending |
| `prose-2026-03-21-187` | 187 | audit-pending |
| `prose-2026-03-21-188` | 188 | audit-pending |
| `prose-2026-03-21-192` | 192 | audit-pending |
| `prose-2026-03-21-193` | 193 | audit-pending |
| `prose-2026-03-21-194` | 194 | audit-pending |
| `prose-2026-03-21-195` | 195 | audit-pending |
| `prose-2026-03-21-196` | 196 | audit-pending |

### B05 — Other structure producers and returned schema (24)

| Source ID | Plaintext line | Status |
|---|---:|---|
| `structures-CampaignInfo-254` | 254 | audit-pending |
| `structures-CampaignInfo-255` | 255 | audit-pending |
| `structures-EncounterTimelineEventInfo-286` | 286 | audit-pending |
| `structures-EncounterTimelineEventInfo-287` | 287 | audit-pending |
| `structures-EncounterWarningInfo-288` | 288 | audit-pending |
| `structures-EncounterWarningInfo-289` | 289 | audit-pending |
| `structures-HousingCatalogEntryInfo-290` | 290 | audit-pending |
| `structures-HousingCatalogEntryInfo-291` | 291 | audit-pending |
| `structures-ItemUpgradeLevelInfo-292` | 292 | audit-pending |
| `structures-ItemUpgradeLevelInfo-293` | 293 | audit-pending |
| `structures-MajorFactionData-294` | 294 | audit-pending |
| `structures-MajorFactionData-295` | 295 | audit-pending |
| `structures-MajorFactionData-296` | 296 | audit-pending |
| `structures-NumberAbbrevOptions-297` | 297 | audit-pending |
| `structures-NumberAbbrevOptions-298` | 298 | audit-pending |
| `structures-QuestTheme-299` | 299 | audit-pending |
| `structures-QuestTheme-300` | 300 | audit-pending |
| `structures-TraitNodeInfo-301` | 301 | audit-pending |
| `structures-TraitNodeInfo-302` | 302 | audit-pending |
| `structures-TraitTreeInfo-303` | 303 | audit-pending |
| `structures-TraitTreeInfo-304` | 304 | audit-pending |
| `structures-TraitTreeInfo-305` | 305 | audit-pending |
| `structures-UIMapPinInfo-306` | 306 | audit-pending |
| `structures-UIMapPinInfo-307` | 307 | audit-pending |

### B06 — Spell-ID declassification data and wildcard control (74)

| Source ID | Plaintext line | Status |
|---|---:|---|
| `prose-2026-02-24-050` | 50 | audit-pending |
| `prose-2026-02-24-057` | 57 | audit-pending |
| `prose-2026-02-24-058` | 58 | audit-pending |
| `prose-2026-02-24-059` | 59 | audit-pending |
| `prose-2026-02-24-060` | 60 | audit-pending |
| `prose-2026-02-24-061` | 61 | audit-pending |
| `prose-2026-02-24-062` | 62 | audit-pending |
| `prose-2026-02-24-063` | 63 | audit-pending |
| `prose-2026-02-24-066` | 66 | audit-pending |
| `prose-2026-02-24-067` | 67 | audit-pending |
| `prose-2026-02-24-068` | 68 | audit-pending |
| `prose-2026-02-24-069` | 69 | audit-pending |
| `prose-2026-02-24-070` | 70 | audit-pending |
| `prose-2026-02-24-071` | 71 | audit-pending |
| `prose-2026-02-24-074` | 74 | audit-pending |
| `prose-2026-02-24-075` | 75 | audit-pending |
| `prose-2026-02-24-076` | 76 | audit-pending |
| `prose-2026-02-24-077` | 77 | audit-pending |
| `prose-2026-02-24-078` | 78 | audit-pending |
| `prose-2026-02-24-081` | 81 | audit-pending |
| `prose-2026-02-24-082` | 82 | audit-pending |
| `prose-2026-02-24-083` | 83 | audit-pending |
| `prose-2026-02-24-086` | 86 | audit-pending |
| `prose-2026-02-24-087` | 87 | audit-pending |
| `prose-2026-02-24-088` | 88 | audit-pending |
| `prose-2026-02-24-091` | 91 | audit-pending |
| `prose-2026-02-24-092` | 92 | audit-pending |
| `prose-2026-02-24-093` | 93 | audit-pending |
| `prose-2026-02-24-094` | 94 | audit-pending |
| `prose-2026-02-24-097` | 97 | audit-pending |
| `prose-2026-02-24-098` | 98 | audit-pending |
| `prose-2026-02-24-101` | 101 | audit-pending |
| `prose-2026-02-24-102` | 102 | audit-pending |
| `prose-2026-02-24-103` | 103 | audit-pending |
| `prose-2026-02-24-104` | 104 | audit-pending |
| `prose-2026-02-24-108` | 108 | audit-pending |
| `prose-2026-02-24-109` | 109 | audit-pending |
| `prose-2026-02-24-110` | 110 | audit-pending |
| `prose-2026-02-24-111` | 111 | audit-pending |
| `prose-2026-02-24-112` | 112 | audit-pending |
| `prose-2026-02-24-113` | 113 | audit-pending |
| `prose-2026-02-24-114` | 114 | audit-pending |
| `prose-2026-02-24-117` | 117 | audit-pending |
| `prose-2026-02-24-118` | 118 | audit-pending |
| `prose-2026-02-24-119` | 119 | audit-pending |
| `prose-2026-02-24-120` | 120 | audit-pending |
| `prose-2026-02-24-121` | 121 | audit-pending |
| `prose-2026-02-24-122` | 122 | audit-pending |
| `prose-2026-02-24-123` | 123 | audit-pending |
| `prose-2026-02-24-124` | 124 | audit-pending |
| `prose-2026-02-24-125` | 125 | audit-pending |
| `prose-2026-02-24-126` | 126 | audit-pending |
| `prose-2026-02-24-127` | 127 | audit-pending |
| `prose-2026-02-24-128` | 128 | audit-pending |
| `prose-2026-02-24-129` | 129 | audit-pending |
| `prose-2026-02-24-133` | 133 | audit-pending |
| `prose-2026-02-24-134` | 134 | audit-pending |
| `prose-2026-02-24-137` | 137 | audit-pending |
| `prose-2026-02-24-138` | 138 | audit-pending |
| `prose-2026-02-24-139` | 139 | audit-pending |
| `prose-2026-02-24-140` | 140 | audit-pending |
| `prose-2026-02-24-141` | 141 | audit-pending |
| `prose-2026-02-24-142` | 142 | audit-pending |
| `prose-2026-02-24-143` | 143 | audit-pending |
| `prose-2026-02-24-146` | 146 | audit-pending |
| `prose-2026-02-24-147` | 147 | audit-pending |
| `prose-2026-02-24-148` | 148 | audit-pending |
| `prose-2026-02-24-149` | 149 | audit-pending |
| `prose-2026-02-24-150` | 150 | audit-pending |
| `prose-2026-02-24-153` | 153 | audit-pending |
| `prose-2026-02-24-154` | 154 | audit-pending |
| `prose-2026-02-24-157` | 157 | audit-pending |
| `prose-2026-02-24-158` | 158 | audit-pending |
| `prose-2026-02-24-159` | 159 | audit-pending |

### B07 — Other security, token, aura and macro contracts (17)

| Source ID | Plaintext line | Status |
|---|---:|---|
| `prose-2026-02-19-026` | 26 | audit-pending |
| `prose-2026-02-19-027` | 27 | audit-pending |
| `prose-2026-02-19-028` | 28 | audit-pending |
| `prose-2026-02-19-029` | 29 | audit-pending |
| `prose-2026-02-19-030` | 30 | audit-pending |
| `prose-2026-02-19-031` | 31 | audit-pending |
| `prose-2026-02-19-032` | 32 | audit-pending |
| `prose-2026-02-19-034` | 34 | audit-pending |
| `prose-2026-02-19-035` | 35 | audit-pending |
| `prose-2026-03-21-200` | 200 | audit-pending |
| `prose-2026-03-21-201` | 201 | audit-pending |
| `prose-2026-03-21-203` | 203 | audit-pending |
| `prose-2026-03-21-204` | 204 | audit-pending |
| `prose-2026-03-21-205` | 205 | audit-pending |
| `prose-2026-03-21-206` | 206 | audit-pending |
| `prose-2026-03-21-207` | 207 | audit-pending |
| `prose-2026-03-21-208` | 208 | audit-pending |

### Metadata (51)

| Source ID | Plaintext line | Status |
|---|---:|---|
| `source-context-001` | 1 | metadata-only |
| `source-context-003` | 3 | metadata-only |
| `source-context-004` | 4 | metadata-only |
| `source-context-006` | 6 | metadata-only |
| `source-context-007` | 7 | metadata-only |
| `source-context-009` | 9 | metadata-only |
| `source-context-010` | 10 | metadata-only |
| `source-context-011` | 11 | metadata-only |
| `source-context-014` | 14 | metadata-only |
| `source-context-015` | 15 | metadata-only |
| `source-context-016` | 16 | metadata-only |
| `source-context-018` | 18 | metadata-only |
| `source-context-020` | 20 | metadata-only |
| `source-context-022` | 22 | metadata-only |
| `source-context-025` | 25 | metadata-only |
| `source-context-033` | 33 | metadata-only |
| `source-context-038` | 38 | metadata-only |
| `source-context-039` | 39 | metadata-only |
| `source-context-041` | 41 | metadata-only |
| `source-context-043` | 43 | metadata-only |
| `source-context-045` | 45 | metadata-only |
| `source-context-047` | 47 | metadata-only |
| `source-context-048` | 48 | metadata-only |
| `source-context-052` | 52 | metadata-only |
| `source-context-055` | 55 | metadata-only |
| `source-context-056` | 56 | metadata-only |
| `source-context-065` | 65 | metadata-only |
| `source-context-073` | 73 | metadata-only |
| `source-context-080` | 80 | metadata-only |
| `source-context-085` | 85 | metadata-only |
| `source-context-090` | 90 | metadata-only |
| `source-context-096` | 96 | metadata-only |
| `source-context-100` | 100 | metadata-only |
| `source-context-107` | 107 | metadata-only |
| `source-context-116` | 116 | metadata-only |
| `source-context-132` | 132 | metadata-only |
| `source-context-136` | 136 | metadata-only |
| `source-context-145` | 145 | metadata-only |
| `source-context-152` | 152 | metadata-only |
| `source-context-156` | 156 | metadata-only |
| `source-context-162` | 162 | metadata-only |
| `source-context-163` | 163 | metadata-only |
| `source-context-165` | 165 | metadata-only |
| `source-context-169` | 169 | metadata-only |
| `source-context-172` | 172 | metadata-only |
| `source-context-191` | 191 | metadata-only |
| `source-context-199` | 199 | metadata-only |
| `source-context-211` | 211 | metadata-only |
| `source-context-212` | 212 | metadata-only |
| `source-context-214` | 214 | metadata-only |
| `source-context-253` | 253 | metadata-only |

## Sources

- [Retained source/provenance](../../sources/12.0.1-api-changes.provenance.json).
- [Publication contract](../../../../docs/specs/patch-12-0-1-publication-sweep.md) — separate inventory-only proof.
- [12.0.0 scout](../12.0.0-session-2026-10-05/p1200-extract-scout.md) — ID/proof conventions, not reused acceptance.
