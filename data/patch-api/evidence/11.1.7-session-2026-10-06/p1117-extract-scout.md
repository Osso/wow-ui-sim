# 11.1.7 non-inventory scout

27 retained rows: 16 pending contractual candidates, 11 editorial. Each source ID appears once below. No new non-inventory behavioral credit; linked pages are not expanded. Prioritize B01 (3 summary statements), then B02 (7 enum rows), B03 (6 DTO rows). Inventory publication is separate evidence. Every batch needs concrete outputs/lifecycle proof, not source-substring tests or empty placeholders.

## B01-summary

| Source ID | Statement | Required proof / boundary |
|---|---|---|
| `prose-undated-004` | * Added a new C_AddOnProfiler.MeasureCall(func, ...) API for profiling individual function calls. | Individual-call profiler is missing: implement call metering, tick domain, errors and result preservation. Existing frame-average metrics are not call profiling. |
| `prose-undated-005` | * Added a new C_AddOns.GetAddOnLocalTable(name) API that can retrieve the namespace table of an addon that opts-in to sharing it via a new AllowAddOnTableAccess TOC directive. | Shared namespace table is passed to files but not retained for opted-in external queries. Requires real loaded-addon lifecycle, AllowAddOnTableAccess and insecure Blizzard-access denial. |
| `prose-undated-006` | * Added a new table.create(arraySizeHint[, nodeSizeHint]) API that preallocates a table with a specified internal size. | Existing utility_api test proves empty mutable tables for capacity variants, not internal preallocation or hint validation/native fidelity. Keep behavioral claim pending; no implementation-shape assertion substitutes. |

## B02-enums

| Source ID | Statement | Required proof / boundary |
|---|---|---|
| `enumerations-Enum-AccountCurrencyTransferResult-017` | Enum.AccountCurrencyTransferResult | Parent enum identity and result consumer contract need runtime audit alongside transfer lifecycle; current publication alone is insufficient. |
| `enumerations-Enum-AccountCurrencyTransferResult-018` | + CurrencyTransferDisabled | CurrencyTransferDisabled needs documented numeric identity and disabled-transfer result delivery from a modeled request; no transfer lifecycle exists. |
| `enumerations-Enum-ReportMinorCategory-019` | Enum.ReportMinorCategory | Parent enum identity/values and reporting consumer contract require targeted runtime proof; no reporting behavior credit. |
| `enumerations-Enum-ReportMinorCategory-020` | + TerroristAndViolentExtremistContent | TerroristAndViolentExtremistContent: verify exact numeric value and category acceptance; report submission/result producer remains unaudited. |
| `enumerations-Enum-ReportMinorCategory-021` | + ChildSexualExploitationAndAbuse | ChildSexualExploitationAndAbuse: verify exact numeric value and category acceptance; report submission/result producer remains unaudited. |
| `enumerations-Enum-UIWidgetVisualizationType-022` | Enum.UIWidgetVisualizationType | Parent visualization enum and producer selection need runtime proof; narrow IconAndText quest pins do not establish ButtonHeader. |
| `enumerations-Enum-UIWidgetVisualizationType-023` | + ButtonHeader | ButtonHeader enum value, typed DTO inputs and actual widget consumer require a dedicated model; missing inventory reader stays an exact gap. |

## B03-structures

| Source ID | Statement | Required proof / boundary |
|---|---|---|
| `structures-BattlemasterListInfo-026` | BattlemasterListInfo | Parent battlemaster DTO contract needs nonempty list producer and concrete current cached consumer fixtures. |
| `structures-BattlemasterListInfo-027` | # instanceType -> matchmakingType | instanceType → matchmakingType: prove new field type/value and old-field absence on real list results, not source-name presence or fabricated empty tables. |
| `structures-PlayerChoiceOptionButtonInfo-028` | PlayerChoiceOptionButtonInfo | Parent player-choice button DTO needs explicit nonempty option-button records and choice-response lifecycle. |
| `structures-PlayerChoiceOptionButtonInfo-029` | + selected | selected: prove field reflects actual option selection and reset/other-option transitions, not a constant false field. |
| `structures-TransmogCategoryAppearanceInfo-030` | TransmogCategoryAppearanceInfo | Parent category-appearance DTO needs a concrete current appearance-list producer and seeded catalog fixtures. |
| `structures-TransmogCategoryAppearanceInfo-031` | - restrictedSlotID | restrictedSlotID removal: inspect current category appearance outputs with nonempty data and prove field omission; adjacent visual/source records are different DTOs. |

## Editorial context

| Source ID | Context |
|---|---|
| `source-context-001` | Patch 11.1.7 API changes |
| `source-context-003` | == Summary == |
| `source-context-008` | == Resources == |
| `source-context-009` | * TOC: 110107 |
| `source-context-010` | * Official patch notes: Legacy of Arathor Update Notes |
| `source-context-011` | * Diffs: wow-ui-source, BlizzardInterfaceResources |
| `source-context-012` | * Deprecated API: Blizzard_Deprecated |
| `source-context-014` | == Consolidated changes == |
| `source-context-015` | : 11.1.5 (61265) → 11.1.7 (61559) Jun 17 2025 |
| `source-context-016` | === Enumerations === |
| `source-context-025` | === Structures === |
