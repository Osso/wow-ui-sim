# 11.2.0 non-inventory extract scout

Page 636685, revision 6726773, retrieved 2026-10-06. All 82 nonblank extract rows assigned: 65 contractual candidates pending, 17 editorial rows metadata-only. No behavior/historical/native credit. [Plaintext](../../sources/11.2.0-api-changes.txt).

## Ranked bounded batches

### B01 — TOC locale directives, mixins and security documentation (4 rows)

Investigate: src/toc/mod.rs; src/loader; mixin bootstrap and API security metadata. Current TOC code hardcodes enUS; metadata/native implementation identity needs separate evidence.
Observable proof: Load concrete enUS/deDE TOCs with actual file execution, compare directives/expansion; mutate two mixin targets and inspect native callable identity; exact security annotations are metadata, not enforcement.

Exact source IDs: `prose-undated-005`, `prose-undated-006`, `prose-undated-007`, `prose-undated-008`.

### B02 — Font scaling, rect invalidation and cached StaticPopup contracts (8 rows)

Investigate: src/lua_api/frame/methods; src/layout; font-state DTO publishers; unmodified cached StaticPopup. Gradient readback does not prove font scaling or ClearAllPoints rect semantics.
Observable proof: Two configurable text scales; measurable font/script metadata; measure rect immediately before/after clear before layout tick; show two real cached dialogs, iterate shown frames and exercise button/editbox accessors/renamed keys.

Exact source IDs: `prose-undated-004`, `prose-undated-009`, `prose-undated-012`, `prose-undated-013`, `prose-undated-014`, `prose-undated-015`, `structures-FontScriptInfo-081`, `structures-FontScriptInfo-082`.

### B03 — Bank/tab enum migrations and BankTabData (34 rows)

Investigate: src/c_api/item_spell/c_currency.rs; src/c_api enum publishers; container state and cached bank/tab UI. Existing bank placeholders cannot establish view/purchase behavior.
Observable proof: Nonempty character/account bank tabs; assert numeric/alias values and removed members across all retained later epochs, capacity/sorting/tab cleanup/edit header outputs and cached bank consumers.

Exact source IDs: `enumerations-Enum-BagFlag-031`, `enumerations-Enum-BagFlag-032`, `enumerations-Enum-BagFlag-033`, `enumerations-Enum-BagIndex-034`, `enumerations-Enum-BagIndex-035`, `enumerations-Enum-BagIndex-036`, `enumerations-Enum-BagIndex-037`, `enumerations-Enum-BagIndex-038`, `enumerations-Enum-BagIndex-039`, `enumerations-Enum-BagIndex-040`, `enumerations-Enum-BagIndex-041`, `enumerations-Enum-BagIndex-042`, `enumerations-Enum-BagIndex-043`, `enumerations-Enum-BagIndex-044`, `enumerations-Enum-BagIndex-045`, `enumerations-Enum-BagIndex-046`, `enumerations-Enum-BagIndex-047`, `enumerations-Enum-BagIndex-048`, `enumerations-Enum-BagIndex-049`, `enumerations-Enum-BagIndex-050`, `enumerations-Enum-BagIndex-051`, `enumerations-Enum-BagIndex-052`, `enumerations-Enum-SlotRegion-063`, `enumerations-Enum-SlotRegion-064`, `enumerations-Enum-SlotRegion-065`, `enumerations-Enum-SlotRegionMask-066`, `enumerations-Enum-SlotRegionMask-067`, `enumerations-Enum-SlotRegionMask-068`, `enumerations-Enum-SubcontainerType-069`, `enumerations-Enum-SubcontainerType-070`, `enumerations-Enum-SubcontainerType-071`, `structures-BankTabData-074`, `structures-BankTabData-075`, `structures-BankTabData-076`.

### B04 — Crafting, expansion, LFG leaver and random BG DTOs (9 rows)

Investigate: src/c_api/crafting_tables.rs; expansion display defaults; src/c_api/c_lfg_info.rs and LFG search producers. Existing hiddenInCraftingForm field needs behavior/input verification.
Observable proof: Two concrete hidden/visible reagent schematics; populated expansion texture kits; leaver/non-leaver player records; two BG rows with exact index/name, host mutations and independent output snapshots.

Exact source IDs: `structures-CraftingReagentSlotSchematic-077`, `structures-CraftingReagentSlotSchematic-078`, `structures-ExpansionDisplayInfo-079`, `structures-ExpansionDisplayInfo-080`, `structures-LfgSearchResultPlayerInfo-083`, `structures-LfgSearchResultPlayerInfo-084`, `structures-RandomBGInfo-085`, `structures-RandomBGInfo-086`, `structures-RandomBGInfo-087`.

### B05 — Cooldown, gem/socket, quest and scene enum additions (10 rows)

Investigate: Current Enum publishers in src/c_api and item/socket/quest/scene consumers. PlaceHolder1 and Fiber renames require later-register/native numeric evidence, not guessed values.
Observable proof: Assert source and later numeric values/aliases with populated cooldown/item/quest consumers; distinguish 2D scene metadata from unsupported 3D rendering.

Exact source IDs: `enumerations-Enum-CooldownSetSpellFlags-053`, `enumerations-Enum-CooldownSetSpellFlags-054`, `enumerations-Enum-ItemGemColor-055`, `enumerations-Enum-ItemGemColor-056`, `enumerations-Enum-ItemSocketType-057`, `enumerations-Enum-ItemSocketType-058`, `enumerations-Enum-QuestCompleteSpellType-059`, `enumerations-Enum-QuestCompleteSpellType-060`, `enumerations-Enum-WarbandScenePlacementType-061`, `enumerations-Enum-WarbandScenePlacementType-062`.

## Editorial assignments

| Source ID | Retained line |
|---|---|
| `source-context-001` | Patch 11.2.0 API changes |
| `source-context-003` | == Summary == |
| `source-context-011` | == Breaking changes == |
| `source-context-017` | == Resources == |
| `source-context-018` | * TOC: 110200 |
| `source-context-019` | * Official patch notes: Ghosts of K'aresh Content Update Notes |
| `source-context-020` | * Diffs: wow-ui-source, BlizzardInterfaceResources |
| `source-context-021` | * Deprecated APIs: |
| `source-context-022` | ** Deprecated_11_2_0.lua |
| `source-context-023` | ** Deprecated_ChatInfo.lua |
| `source-context-024` | ** Blizzard_DeprecatedSpecialization |
| `source-context-025` | ** Deprecated_SpellBook.lua |
| `source-context-026` | ** Deprecated_UnitScript.lua |
| `source-context-028` | == Consolidated changes == |
| `source-context-029` | : 11.1.7 (61559) → 11.2.0 (62438) Aug  5 2025 |
| `source-context-030` | === Enumerations === |
| `source-context-073` | === Structures === |

## Boundaries

No Notes/Blue posts in retained revision. Inventory annotations stay attached to publication occurrences, not behavior proofs. Missing bank/challenge/group/pet/inspection/talent/spell producers and two lifecycle/global retirements stay explicit. Default retail is 12.1.0, not historical 11.2.0. Catalog shop and 11.2.7 fixtures untouched.

## Evidence

- [Exhaustive assignment map](p1120-extract-assignments.json)
- [38-row gap review](p1120-gap-review.json): 12 fixed, 26 retained.
- [162 observations](p1120-sweep-result.json): 136 OK.
- [Coverage ledger](../../sources/11.2.0-page-coverage.json): 244 unique IDs.
