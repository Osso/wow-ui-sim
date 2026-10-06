# 11.2.5 non-inventory extract scout

Source page 641912, revision 6726772, retrieved 2026-10-06. [Retained plaintext](../../sources/11.2.5-api-changes.txt) is rendered from wikitext, not expanded templates. All 73 nonblank rows assigned: 58 contractual parent/member/summary candidates remain audit-pending; 15 editorial rows remain metadata-only. No behavioral, historical or native parity credit.

## Ranked bounded follow-up batches

### B01 — Socketing relocation (1 rows)

Investigate: `src/lua_api/globals/missing_surface/item_socket_info.rs`; cached deprecation aliases. Fifteen removed globals pass only exact alias identity; the broad relocation claim remains separate.
Observable proof: Nonempty existing/proposed socket fixtures; accept/cancel mutation, bound/refund flags, old/new alias identity and cached socket UI consumption. No blanket socketing parity.

Exact source IDs: `prose-undated-004`.

### B02 — Cooldown DTO and renamed flags (5 rows)

Investigate: `src/c_api/c_cooldown_viewer.rs`, enum publication and cached cooldown viewer. Current DTO has tooltip override/isKnown fields; this scout does not verify their semantics.
Observable proof: Two known/unlearned cooldown records, tooltip override selection, category filter transitions and flag numeric/alias behavior; current-epoch supersession versus historical name separated.

Exact source IDs: `enumerations-Enum-CooldownSetSpellFlags-027`, `enumerations-Enum-CooldownSetSpellFlags-028`, `structures-CooldownViewerCooldown-062`, `structures-CooldownViewerCooldown-063`, `structures-CooldownViewerCooldown-064`.

### B03 — AddOn, quest and Mythic+ DTOs (10 rows)

Investigate: `src/c_api/c_addons.rs`, quest/gossip producers, `src/c_api/c_mythic_plus_calendar.rs` and Mythic+ scenario input. Locate exact consumers/producer fields before implementation.
Observable proof: Concrete AddOnInfo output omits updateAvailable; quest ID/classification mapping; two dated runs, weekday/duration/season values, mutation and independent snapshots, no-record cases. Publication is insufficient.

Exact source IDs: `structures-AddOnInfo-060`, `structures-AddOnInfo-061`, `structures-GossipQuestUIInfo-065`, `structures-GossipQuestUIInfo-066`, `structures-MythicPlusDate-067`, `structures-MythicPlusDate-068`, `structures-MythicPlusRunInfo-069`, `structures-MythicPlusRunInfo-070`, `structures-MythicPlusRunInfo-071`, `structures-MythicPlusRunInfo-072`.

### B04 — Trait enums and increased-rank DTOs (15 rows)

Investigate: Trait registration/state under `src/lua_api/globals/missing_surface/traits/`; C_* compatibility boundary requires new modeled producers in `src/c_api/`.
Observable proof: Nonempty entryID→ranks changes, tree root selection and concrete currency/entry/node/system flag values. Purchase/refund transitions and cached consumers; do not infer native bit values or decode policy.

Exact source IDs: `enumerations-Enum-TraitCurrencyType-046`, `enumerations-Enum-TraitCurrencyType-047`, `enumerations-Enum-TraitNodeEntryType-048`, `enumerations-Enum-TraitNodeEntryType-049`, `enumerations-Enum-TraitNodeFlag-050`, `enumerations-Enum-TraitNodeFlag-051`, `enumerations-Enum-TraitNodeFlag-052`, `enumerations-Enum-TraitNodeFlag-053`, `enumerations-Enum-TraitSystemFlag-054`, `enumerations-Enum-TraitSystemFlag-055`, `structures-TraitNodeInfo-073`, `structures-TraitNodeInfo-074`, `structures-TraitNodeInfo-075`, `structures-TraitTreeInfo-076`, `structures-TraitTreeInfo-077`.

### B05 — Remaining enums and renames (27 rows)

Investigate: Current Enum table publishers and cached account/item/minimap/report/scene consumers; model enums in `src/c_api/` when their contract belongs to C_* APIs.
Observable proof: Concrete numeric values and source-versus-later aliases for all listed parents/members; populated consumer fixtures where flags affect behavior. Account/season/3D domains may retain explicit scope boundaries.

Exact source IDs: `enumerations-Enum-AccountData-019`, `enumerations-Enum-AccountData-020`, `enumerations-Enum-AccountData-021`, `enumerations-Enum-AccountData-022`, `enumerations-Enum-AccountTransType-023`, `enumerations-Enum-AccountTransType-024`, `enumerations-Enum-BonusStatIndex-025`, `enumerations-Enum-BonusStatIndex-026`, `enumerations-Enum-ItemCollectionType-029`, `enumerations-Enum-ItemCollectionType-030`, `enumerations-Enum-ItemCollectionType-031`, `enumerations-Enum-ItemCollectionType-032`, `enumerations-Enum-ItemCollectionType-033`, `enumerations-Enum-ItemConsumableSubclass-034`, `enumerations-Enum-ItemConsumableSubclass-035`, `enumerations-Enum-MinimapTrackingFilter-036`, `enumerations-Enum-MinimapTrackingFilter-037`, `enumerations-Enum-PerksVendorCategoryType-038`, `enumerations-Enum-PerksVendorCategoryType-039`, `enumerations-Enum-PhaseReason-040`, `enumerations-Enum-PhaseReason-041`, `enumerations-Enum-QuestTagType-042`, `enumerations-Enum-QuestTagType-043`, `enumerations-Enum-ReportType-044`, `enumerations-Enum-ReportType-045`, `enumerations-Enum-UIModelSceneFlags-056`, `enumerations-Enum-UIModelSceneFlags-057`.

## Metadata assignments

| Source ID | Retained editorial line |
|---|---|
| `source-context-001` | Patch 11.2.5 API changes |
| `source-context-003` | == Summary == |
| `source-context-006` | == Resources == |
| `source-context-007` | * TOC: 110205 |
| `source-context-008` | * Official patch notes: Legion Remix Content Update Notes |
| `source-context-009` | * Diffs: wow-ui-source, BlizzardInterfaceResources |
| `source-context-010` | * Deprecated APIs: |
| `source-context-011` | ** Deprecated_11_2_5.lua |
| `source-context-012` | ** Deprecated_ItemSocketInfo.lua |
| `source-context-013` | ** Deprecated_PetInfo.lua |
| `source-context-014` | ** Deprecated_WorldElapsedTimerTypes.lua |
| `source-context-016` | == Consolidated changes == |
| `source-context-017` | : 11.2.0 (62438) → 11.2.5 (63796) Oct 10 2025 |
| `source-context-018` | === Enumerations === |
| `source-context-059` | === Structures === |

## Boundaries

No Notes/Blue posts exist in this retained revision. Inventory signature annotations stay attached to publication occurrences; the sweep cannot verify them. Four inventory rows reverse later publication expectations, including two CVars removed in 11.2.7. No 11.x runtime feature exists. Forty-two namespace producer gaps and three unsupported 3D methods remain; no namespace producers owned by the concurrent 11.2.7 task were changed.

## Evidence

- [Exhaustive assignments](p1125-extract-assignments.json)
- [47-row gap review](p1125-gap-review.json): two fixed, 45 retained.
- [163 publication observations](p1125-sweep-result.json): 118 OK.
- [Coverage ledger](../../sources/11.2.5-page-coverage.json): 236 source IDs; publication-only credit.
