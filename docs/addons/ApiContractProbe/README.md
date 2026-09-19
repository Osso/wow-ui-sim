# API Contract Probe

## Manual recraft reagent input reads

`/apicontract recraft-reagent-read <label>` is excluded from `all`. Equipment slots **1 and 2** independently feed `ItemLocation:CreateFromEquipmentSlot(slot)` and `C_Item.GetItemGUID(originalLocation)`. Only the first original accessible string GUID is eligible; never parse, clone, coerce or truncate an input GUID. Independently call `GetRecipesTracked(false)` once, then `GetRecipeSchematic(id, false, nil)` for the first two original accessible finite recipe IDs. Inspect only two slots and two original reagents per slot.

Each original reagent is independently paired with each original GUID for `IsRecraftReagentValid(GUID, reagent)`: at most **16 pair queries**. Reuse the corrected recraft-limit ancestry/field checks; authorize equipment receiver/slot/location/GUID, recipe/list/schematic/slot/reagent ancestors, and optional `itemID`/`currencyID` fields before forwarding and again after query lookup/function guards. Fields must be accessible nil or finite numbers. Bounded final GUID/field/ancestor rechecks preserve the known revocation boundaries; they do not provide atomic authorization against arbitrary guard side effects. A failed input blocks only dependent branches, not other equipment or recipes.

`recraftReagentRead` records `equipment[].producer/guid`, raw tracked `producer`, and `entries[].id/schematic/slots[].reagents[]`. Each reagent has two positional `pairs[]` with a bounded GUID observation, guarded diagnostic fields when available, and raw `result`. No raw location, reagent, list or returned object is retained. Bounds: **23 API calls per snapshot** (two constructors, two GUID queries, one tracked list, two schematics, sixteen pair queries), ten snapshots, sixteen tuple positions, 256-byte output strings and 128-byte labels. Constructor-internal Lua operations are not extra recorder API calls.

This is **input-acceptance preparation only**. Ordinary equipment GUIDs are not claimed to be recraft allocations or valid targets. No replacement arrays, synthetic reagents, transactions, allocations, `GetRecraftRemovalWarnings`, crafting, equipping or recrafting operations are performed. Pinned retail `ItemDocumentation.lua:479–491`, `TradeSkillUIDocumentation.lua:1048–1061`, and `TradeSkillUITypesDocumentation.lua` establish shapes, not native validity or historical semantics. Twelve actual TOC/slash fixtures establish local recorder mechanics only; native behavior remains unverified.

```sh
luajit docs/addons/ApiContractProbe/tests/recraft_reagent_read.lua docs/addons/ApiContractProbe
```

## Manual recraft limit reads

`/apicontract recraft-limit-read <label>` is excluded from `all`. Call `C_TradeSkillUI.GetRecipesTracked(false)` once; only positions 1–2 of its first accessible table supply original finite recipe IDs. Each ID independently feeds `GetRecipeSchematic(id, false, nil)` with exactly three arguments. Inspect only the first schematic's first two `reagentSlotSchematics` and each slot's first two `reagents`.

Each original accessible table/userdata `CraftingReagent` is passed unchanged to `RecraftLimitCategoryValid(reagent)` once. Check only its declared optional `itemID` and `currencyID`: nil or accessible finite numbers; no defaults, exclusivity rule, cloning or synthetic fields. Recheck original recipe/list/schematic/slot/reagent ancestry and current fields after API lookup/function guards and before forwarding. Reauthorize declared fields after the final ancestor pass, with bounded peer/ancestor rechecks. These sequential guards do not provide atomic authorization against arbitrary guard side effects.

`recraftLimitRead` stores raw `producer`, `entries[].id/schematic`, and `entries[].slots[].reagents[]` containing opaque `reagent`, guarded diagnostic `fields`, and raw query `result`. Missing, restricted, malformed and failed inputs remain explicit; independent paths continue. Retain no raw source objects. Bounds: **11 API calls per snapshot** (one list, two schematics, eight reagent queries), ten snapshots, sixteen tuple positions, 256-byte scalar strings and 128-byte labels.

Pinned retail `TradeSkillUIDocumentation.lua:1098–1114` and `TradeSkillUITypesDocumentation.lua:214–220` declare the exact reagent argument and fields. `ProfessionsUtil.lua:50–54,107–116` supplies the schematic chain; `Blizzard_ProfessionsCrafting.lua:608–619` forwards a reagent directly to the query. Same declared type does not establish a live allocation or valid item. No transactions, allocation construction, crafting/recraft mutation, native validity, ordering, completeness or conformance claim follows. Sixteen actual TOC/slash fixtures include 32 final-receiver field-revocation cases and 480 staged cross-input cases; they establish recorder mechanics only:

```sh
luajit docs/addons/ApiContractProbe/tests/recraft_limit_read.lua docs/addons/ApiContractProbe
```

## Manual crafting enchant items

`/apicontract crafting-enchant-items <label>` is excluded from `all`. Call `C_TradeSkillUI.GetRecipesTracked(false)` once; inspect only positions 1–4 of its first accessible returned table. Each original accessible finite recipe ID independently feeds `GetEnchantItems(originalRecipeID, nil)` with **exactly two arguments**. The nullable crafting-reagent argument is intentionally unpopulated; never build a synthetic reagent table or execute an enchant.

`craftingEnchantItems` saves the raw `producer` tuple and `entries[].id/items`. Only the first returned item list is inspected, at positions 1–8; values remain bounded scalar observations, without GUID parsing, comparison, traversal or forwarding. Guard every list receiver before indexing. Recheck the original recipe ID and explicit nil after API lookup/function guards; bounded sequential peer checks are not atomic authorization against arbitrary guard side effects. Missing, restricted, invalid or failing inputs/results do not suppress other positions.

Bounds: **five API calls per snapshot**, ten snapshots, sixteen raw tuple positions, 256-byte scalar strings and 128-byte labels. Preserve exact arity, nil positions and opaque errors; retain no raw objects. Pinned retail `TradeSkillUIDocumentation.lua:257–270,755–767` declares the signatures, not recipe suitability, list completeness, recraft behavior or native changed-contract semantics. Existing `crafting-schematic-read` is unchanged. Twelve actual TOC/slash fixtures cover recorder mechanics only:

```sh
luajit docs/addons/ApiContractProbe/tests/crafting_enchant_items.lua docs/addons/ApiContractProbe
```

## Manual abbreviation options

`/apicontract abbreviation-options <label>` is excluded from `all`. Call `C_StringUtil.GetDefaultAbbreviationBreakpoints(nil)` once with one explicit nil argument. Only its original accessible table first return supplies a fresh owned `{ breakpointData = originalTable }` options object. Never inspect, iterate, rewrite or copy breakpoint entries; do not add `locale` or `config` fields.

For each literal input `0, 999, 1000, 1234, 999999, 1000000, -1000, 1000000000`, independently call `AbbreviateNumbers(number)`, `AbbreviateNumbers(number, options)`, `AbbreviateLargeNumbers(number)` and `AbbreviateLargeNumbers(number, options)` in that order. Missing/invalid/restricted producer results skip option variants, not omitted-option controls. Recheck the original table, owned options, its guarded `breakpointData` identity and input after function lookup/access guards. A changed owned field is unavailable input, never a replacement producer. After numeric authorization, recheck both tables, then the number and both tables again to catch staged cross-input revocation. These bounded sequential checks do not provide atomic authorization against arbitrary guard side effects.

`abbreviationOptions` stores raw `producer` and `samples[].input/small/large`, with `omitted` and `options` tuples in each lane. Preserve exact arity and nil positions, opaque errors and inaccessible results. Bounds: **33 calls per snapshot**, ten snapshots, sixteen result positions, 256-byte scalar strings, 128-byte labels. Returned objects remain opaque and are not retained. Pinned `LocalizationDocumentation.lua` and `LocalizationSharedDocumentation.lua` establish call shapes and the sole supplied options field, not formatting equality, locale behavior or native semantics. No config construction, restricted setters, callbacks or locale changes. Existing `abbreviations` stays unchanged.

Seventeen targeted recorder fixtures include 64 numeric-guard revocation cases and 128 staged table/options cross-input cases:

```sh
luajit docs/addons/ApiContractProbe/tests/abbreviation_options.lua docs/addons/ApiContractProbe
```

## Manual timeline track queries

`/apicontract timeline-track-queries <label>` is excluded from `all`. Call `C_EncounterTimeline.GetEventList()` once; only the first returned table supplies positions 1–8 as original accessible finite event IDs. Independently call `GetEventTrack(id)` once per usable ID, preserving the raw track/sort-index tuple, including a nil sort index. Recheck the original ID after namespace lookup and function-access guards; never sort, deduplicate, coerce or substitute IDs.

Independently call `GetTrackList()` and `HasVisibleEvents()` once each, even when event production fails. Only the first returned track table is inspected at positions 1–5. Reuse the guarded field inspector for exactly `id`, `type`, `minimumDuration`, `maximumDuration`, `minimumEventIntroDuration`, `minimumEventGapDuration`, `maximumEventCount`, and `sortDirection`; recheck each list/entry receiver before every lookup and each value before serialization. Returned objects are not retained.

`timelineTrackQueries` stores raw `producer`, positional `entries[].id/track`, raw `trackList` with bounded `entries[].fields`, and raw `visible`. Bounds: **11 API calls per snapshot**, ten snapshots, sixteen return positions, 256-byte scalar strings, 128-byte labels. Pinned retail/PTR `EncounterTimelineDocumentation.lua:221–236,274–281,356–363,617–630` supplies call shapes and fields only. No Add/Cancel/EditMode operations, mutations, reordering, completeness, enum identity, native visibility or conformance claims. Twelve actual TOC/slash fixtures establish recorder mechanics only:

```sh
luajit docs/addons/ApiContractProbe/tests/timeline_track_queries.lua docs/addons/ApiContractProbe
```

## Manual timeline track information

`/apicontract timeline-track-info <label>` is excluded from `all`. Query `C_EncounterTimeline.GetTrackInfo` once for each fixed published `Enum.EncounterTimelineTrack` name: `Queued`, `Short`, `Medium`, `Long`, `Indeterminate`. Forward only the original accessible finite numeric value; never substitute canonical numbers, enumerate additional keys, or compare returned IDs with inputs.

`timelineTrackInfo.tracks[].name/observation` preserves raw return arity, nil positions, opaque errors and the observed input. Only the first returned object receives `info.fields`: `id`, `type`, `minimumDuration`, `maximumDuration`, `minimumEventIntroDuration`, `minimumEventGapDuration`, `maximumEventCount`, `sortDirection`. Recheck the enum after API lookup/function guards and guard the returned receiver before every field lookup and each field before serialization. Missing/restricted values and errors do not suppress peer tracks; returned objects are not retained.

Bounds: five calls per snapshot, ten snapshots, sixteen return positions, 256-byte scalar strings and 128-byte labels. Retail/PTR `EncounterTimelineDocumentation.lua:258–272,617–630` and `EncounterTimelineConstantsDocumentation.lua:63–75` establish call shape, fields and published names only. No AddEditMode/Cancel/event creation, mutation, native queue/timing/default/order or conformance claims. Twelve actual TOC/slash fixtures establish local recorder mechanics:

```sh
luajit docs/addons/ApiContractProbe/tests/timeline_track_info.lua docs/addons/ApiContractProbe
```

## Manual heal-calculator modes

`/apicontract heal-calculator-modes <label>` is excluded from `all`. Create one fresh owned `CreateUnitHealPredictionCalculator()` result. Record three baseline mode getters, then seven fixed published inputs: `UnitHealAbsorbMode.ReducedByIncomingHeals/Total`, `UnitHealAbsorbClampMode.CurrentHealth/MaximumHealth`, and `UnitDamageAbsorbClampMode.MissingHealth/MissingHealthWithoutIncomingHeals/MaximumHealth`. Each original accessible finite enum value is passed to its matching setter once, followed by one independent matching getter. Missing publication never uses numeric fallback.

Call `Reset()` and read all three getters, then `ResetPredictedValues()` and read all three again. Setter/reset failures do not suppress readbacks or peer observations. `healCalculatorModes` stores raw `constructor`, `baseline`, `modes[].enum/key/setter/getter`, and `resets[].name/result/getters` observations. Guard the calculator before type inspection, every method lookup and invocation; recheck receiver and enum after method-function guards. Returned objects remain opaque and are not retained.

Bound: **7 setters + 16 getters + 2 resets = 25 methods, plus 1 constructor = 26 calls per snapshot**, at most 260 calls across ten snapshots. Preserve exact arity and nil positions within sixteen result positions, 256-byte strings and 128-byte labels. Pinned `UnitHealPredictionCalculatorAPIDocumentation.lua` and `UnitHealPredictionCalculatorSharedDocumentation.lua` establish method shapes and seven enum names, not equality, defaults, reset equivalence or native semantics. This mutates only a fresh owned calculator; no unit binding, predicted-value population, resource mutation or security experiment is included. Existing `heal-calculator` remains unchanged.

Eleven actual TOC/slash fixtures cover local recorder mechanics only:

```sh
luajit docs/addons/ApiContractProbe/tests/heal_calculator_modes.lua docs/addons/ApiContractProbe
```

## Manual LFG title-match reads

`/apicontract lfg-title-match-read <label>` is excluded from `all`. Guard the original published `Enum.LFGListFilter.PvE` value, then follow the static vendor selection chain: `GetAvailableCategories(PvE)` → first two original category IDs → `GetAvailableActivityGroups(categoryID, PvE)` → first two original group IDs → `GetAvailableActivities(categoryID, groupID, PvE)` → first two original activity IDs per group. Missing or inaccessible publication skips the chain; no numeric fallback, synthetic IDs or activity-info objects are supplied.

For each original activity/group pair, call `DoesEntryTitleMatchPrebuiltTitle(activityID, groupID, nil, nil)` with **exactly four arguments**. `lfgTitleMatchRead.experiment` labels this `explicit-nil-playstyles`: the two optionals are declared nullable, but this is **not** a claim about native caller defaults, title context or matching semantics. No title argument, current UI read, title creation, entry/search-result query, listing, request or mutation is used.

Save `filter`, the category `producer`, and bounded `categories[].categoryID/producer/groups[].groupID/producer/activities[].activityID/match`. Guard publication, list receivers, all original ancestor IDs and the PvE value before inspection or forwarding; recheck those inputs and both nil arguments after API lookup/function guards. Preserve raw arity, nil holes, opaque errors and inaccessible values. Failures do not suppress independent peers, and no original objects are retained.

Bounds: fifteen API calls per snapshot (1 category producer + 2 group queries + 4 activity queries + 8 predicates), ten snapshots, sixteen result positions, 256-byte scalar strings and 128-byte labels. Pinned `LFGListInfoDocumentation.lua:121–136,281–295` declares the predicate and group query. `Blizzard_GroupFinder/Mainline/LFGList.lua:229,627–638,879–880,917–922,964–968` grounds the published filter and selected category/group/activity flow. Eleven actual TOC/slash fixtures prove recorder mechanics only; native title state, explicit-nil behavior, historical changes, validation and restricted contexts remain unverified.

```sh
luajit docs/addons/ApiContractProbe/tests/lfg_title_match_read.lua docs/addons/ApiContractProbe
```

## Manual LFG playstyle formatting

`/apicontract lfg-playstyle-format <label>` is excluded from `all`. Read the original published `Enum.LFGListFilter.PvE` value and call `C_LFGList.GetAvailableCategories(value)`. Only positions 1–2 of its first accessible returned table supply original finite category IDs. Each feeds `GetAvailableActivities(categoryID, 0, originalPvE)`; the literal group `0` follows the vendor's zero-group/selected-filters-zero branch, not an enum fallback. Only positions 1–2 of each first activity table supply original finite IDs to `GetActivityInfoTable(activityID)` with exactly one argument.

For each original accessible table/userdata activity result, independently call `GetPlaystyleString(Enum.LFGEntryPlaystyle.None, Enum.LFGEntryGeneralPlaystyle.None, originalActivityInfo)` using guarded published values. Missing publication skips the affected calls; no numeric substitute is supplied. Recheck every forwarded input after API lookup and function guards. Check each list receiver before indexing. Neither activity-info objects nor returned objects are traversed, reconstructed, normalized or retained.

`lfgPlaystyleFormat` saves `filter`, the category `producer`, and bounded `categories` rows with `categoryID`, activity-list `producer`, and `activities` containing `activityID`, `info` and `formatted` raw tuples. Calls fail independently; preserve exact arity, nil holes, opaque errors and restricted values. Bounds: eleven API calls per snapshot (1 + 2 + 4 + 4), ten snapshots, sixteen result positions, 256-byte scalar strings and 128-byte labels.

Pinned `LFGListInfoDocumentation.lua:184–203,368–382` supplies the info/formatter signatures; `LFGConstantsDocumentation.lua` supplies the three enum members. `Blizzard_GroupFinder/Mainline/LFGList.lua:628,881–882,4390,4404` establishes category/activity production and original-info forwarding. This mode does not call entry/search-result APIs, create listings, send requests or mutate state. Eleven actual TOC/slash fixtures prove recorder mechanics only; native listing state, non-None playstyles, formatter output and historical semantics remain unverified.

```sh
luajit docs/addons/ApiContractProbe/tests/lfg_playstyle_format.lua docs/addons/ApiContractProbe
```

## Manual crafting schematic reads

`/apicontract crafting-schematic-read <label>` is excluded from `all`. Call `C_TradeSkillUI.GetRecipesTracked(false)` once, then inspect only positions 1–4 of its first accessible returned table. Each original accessible finite recipe ID independently feeds `GetRecipeSchematic(id, false, nil)` with exactly three arguments, including the explicit nil recipe level. Missing product quality does not prevent schematic inspection; no quality API is called.

`craftingSchematicRead` records `producer` and four bounded `entries` containing an ID observation and `schematic` tuple. Inspect only the first schematic result: `recipeID`, `productQuality`, and `reagentSlotSchematics` positions 1–4. Each slot exposes `quantityRequired`, `dataSlotIndex`, `slotIndex`, `reagentType`, plus the first four `reagents` (`itemID`, `currencyID`) and first four `variableQuantities` (`quantity` and the original nested `reagent`'s `itemID`/`currencyID`). Every container, entry and field is access-checked before lookup or serialization, including after earlier observations revoke access. Recipe IDs are rechecked after namespace/function lookup and guards before forwarding. No source list length, generic iteration, recursive expansion, synthetic reagent table or downstream derived-quality call is used.

Bounds: five API calls per snapshot, ten snapshots, sixteen tuple positions, 256-byte scalar strings and 128-byte labels. Preserve raw arity, nil holes and opaque errors; do not retain raw objects. Pinned retail `TradeSkillUIDocumentation.lua:738–767` and `TradeSkillUITypesDocumentation.lua` declare the signatures/fields; `ProfessionsUtil.lua` and `Blizzard_ProfessionsRecipeTracker.lua` ground the tracked-recipe-to-schematic chain. Eleven actual TOC/slash fixtures prove local recorder mechanics only. This is **not** coverage for `CraftingReagentInfo`, `CraftingItemSlotModification`, `CraftingOrderReagentInfo` or `CraftingResourceReturnInfo`. No crafting, orders, allocations, requests, mutations, native quality/completeness/type-identity conclusions or conformance credit follow.

```sh
luajit docs/addons/ApiContractProbe/tests/crafting_schematic_read.lua docs/addons/ApiContractProbe
```

## Manual error-code publication

`/apicontract error-code-publication <label>` is excluded from `all` and does not extend the shared `publication` target list. It reads exactly the twelve fixed `LE_GAME_ERR_*` names retained by `remaining-objects-global-error-codes`, once each through guarded global lookup. No numeric values, defaults or fallbacks are supplied; looked-up functions and objects are never invoked or traversed. No error trigger is needed for these presence observations.

`errorCodePublication[]` records each `name` with a scalar observation: `missing` for an absent global, `restricted` for an inaccessible value, `lookup-error` for a denied receiver or failed lookup, `nonfinite` for nonfinite numbers, otherwise `observed` with its accessible kind/value. Functions, tables and userdata remain opaque. Recheck global receiver access for every lookup; value access is checked before type inspection or serialization. Missing access APIs retain the existing capture-level `missing-access-api` outcome.

Bounds: twelve lookup attempts per capture, ten captures, 256-byte scalar strings and 128-byte labels. `AuditTargets.lua` and existing publication behavior remain unchanged. Twelve actual TOC/slash fixtures cover exact names, noncanonical values, missing/restricted/failed lookups, receiver revocation, opaque values, limits and old/new/publication/all interleaving. This is presence-only recorder preparation, not native error-code mapping, emission or semantics; the retained plan's native numeric and load-phase evidence remains unverified.

```sh
luajit docs/addons/ApiContractProbe/tests/error_code_publication.lua docs/addons/ApiContractProbe
```

## Manual timeline source counts

`/apicontract timeline-source-counts <label>` is excluded from `all`. For each fixed published `Enum.EncounterTimelineEventSource` name `Encounter`, `Script` and `EditMode`, independently call `C_EncounterTimeline.GetEventCountBySource(value)` twice with exactly one argument. Re-read the published member for each attempt; accept only accessible finite numeric values, without numeric fallback, enum iteration or deduplication.

`timelineSourceCounts.sources[]` stores each `name` and `observations[1..2]`, including bounded scalar `input` and raw result tuples. Guard publication containers and members before lookup/inspection, then recheck the original value after API lookup and function guards before forwarding. Missing, invalid or restricted enums, missing APIs, lookup errors and opaque call errors do not suppress peer observations. Preserve exact arity, nil positions and inaccessible outputs without interpreting counts.

Bounds: six API calls per snapshot, ten snapshots, 16 result positions, 256-byte scalar strings and 128-byte labels. Pinned `EncounterTimelineDocumentation.lua:100–114` and `EncounterTimelineConstantsDocumentation.lua:18–30` establish the signature and enum names only. No event synthesis, Add/Cancel/Finish/Pause/Resume operations, EditMode events, icon mutations, native count/default/order conclusions or conformance credit. Native source populations and transitions remain unverified.

Eleven separate actual TOC/slash fixtures exercise recorder mechanics:

```text
luajit docs/addons/ApiContractProbe/tests/timeline_source_counts.lua docs/addons/ApiContractProbe
```

## Manual timeline lifecycle reads

`/apicontract timeline-lifecycle-read <label>` is excluded from `all`. Call `C_EncounterTimeline.GetEventList()` once with no arguments. Only positions 1–8 of its first returned accessible table supply original finite event IDs. Independently call `GetEventState(id)`, `GetEventTimeElapsed(id)`, `GetEventTimeRemaining(id)` and `GetEventTimer(id)` with exactly one argument per usable ID. Independently call `GetEventHighlightTime()` twice, including when the list or every ID is unavailable.

`timelineLifecycleRead` stores `producer`, `entries[].id`, API-name-keyed `entries[].queries` and `highlight[1..2]`. Guard each list index and original ID before inspection, and recheck ID accessibility after namespace/function lookup and guards. Preserve duplicate IDs, nil holes, raw arity and opaque errors. Conditional-secret values remain opaque before inspection. Only accessible `GetEventTimer` return positions receive the shared ten-method duration inspector: `GetTotalDuration`, `GetElapsedDuration`, `GetRemainingDuration`, `GetElapsedPercent`, `GetRemainingPercent`, `GetStartTime`, `GetEndTime`, `GetClockTime`, `GetModRate`, and `HasExpired`. Receiver accessibility is checked before lookup and again after method-function guards. Method results are scalar-only; timer objects are not retained, and state/time/highlight results receive no method inspection.

Bounds: 35 API calls per snapshot, ten snapshots, 16 tuple positions, 256-byte strings and 128-byte labels. The declared single duration return permits **80 method calls** per snapshot (8 events × 1 object × 10 methods); the defensive 16-return bound permits **1,280 method calls** (8 × 16 × 10), or 12,800 across ten snapshots. These method calls are additional to the unchanged 35 API-call cap. Pinned `EncounterTimelineDocumentation.lua:116–123,144–219` and `LuaDurationObjectAPIDocumentation.lua` declare call shapes, not native values. No event-count queries, guessed source enums, event mutations, retention, cast-state changes, state comparisons or timing conclusions. Seventeen cumulative actual TOC/slash fixtures (eleven existing, six new) establish local recorder mechanics only; native population, lifecycle, timing, ordering, completeness and restricted-context behavior remain unverified.

```sh
luajit docs/addons/ApiContractProbe/tests/timeline_lifecycle_read.lua docs/addons/ApiContractProbe
```

## Manual current timeline events

`/apicontract timeline-current-events <label>` is excluded from `all`. Call `C_EncounterTimeline.GetEventList()` once without arguments. Only the first returned table's positions 1–8 supply original accessible finite event IDs. For each, independently call `GetEventInfo(id)` and `GetEventColor(id)` with exactly one argument; the color override is omitted. Preserve duplicates, nil holes, raw arity and opaque errors without inventing event IDs.

`timelineCurrentEvents` contains `producer` and bounded `entries`; each entry has `id`, `info` and `color` observations. Only the first info result exposes guarded fields `id`, `source`, `spellName`, `spellID`, `iconFileID`, `duration`, `maxQueueDuration`, `icons`, `severity`, `isApproximate`. Recheck the original ID after namespace/function guards and the info receiver before every field lookup, including after tuple or earlier field observations revoke access. Conditional-secret objects and values remain opaque before inspection. Colors receive no methods or field reads; no produced objects are retained.

Bounds: 17 API calls per snapshot, ten snapshots, 16 tuple positions, 256-byte strings and 128-byte labels. Pinned `EncounterTimelineDocumentation.lua:81–97,126–152,585–600` supplies signatures and fields, not native results. Never create, cancel or finish events or set icon textures. Twelve actual TOC/slash fixtures prove recorder mechanics only; no native execution, security-context experiment, ordering, completeness, default, timing or color conclusion follows.

```sh
luajit docs/addons/ApiContractProbe/tests/timeline_current_events.lua docs/addons/ApiContractProbe
```

## Manual item interaction flags

`/apicontract item-interaction-flags <label>` is excluded from `all`. Call `C_ItemInteraction.GetItemInteractionInfo()` twice independently with zero arguments. Missing APIs, nil/unavailable results and opaque errors remain observations; no interaction is opened or populated.

`itemInteractionFlags[1..2]` preserves raw return arity and nil positions. Only the first accessible table/userdata result exposes `values[1].fields.flags`, using guarded receiver and value reads. Other fields and returned objects are not traversed. Recheck the root after tuple serialization; inaccessible flags stay opaque. No enum interpretation, historical equivalence or native flags claim follows.

Bounds: two calls per snapshot, ten snapshots, 16 result positions, 256-byte strings and 128-byte labels. Never call initialization, pending-item, open, perform or conversion APIs. The pinned `ItemInteractionUIDocumentation.lua` declares the no-argument optional root and `flags`; populated native behavior remains unverified. Eleven actual TOC/slash fixtures prove recorder mechanics only:

```sh
luajit docs/addons/ApiContractProbe/tests/item_interaction_flags.lua docs/addons/ApiContractProbe
```

## Manual house exterior options

`/apicontract house-exterior-options <label>` is excluded from `all`. Independently call `C_HouseExterior.GetCurrentHouseExteriorType()`, `GetHouseExteriorSizeOptions()` and `GetHouseExteriorTypeOptions()` once each, without arguments. Preserve raw return arity, nil positions and opaque errors; failures do not suppress peer calls.

`houseExteriorOptions` contains observations keyed by API name. Only the first size/type result exposes `selectedSize`/`selectedExteriorType` and an opaque scalar `options` observation under `values[1].fields`. Separate `values[1].optionEntries.entries` captures positions 1–4: size options read `size`, `name`, `isLocked`; type options read `houseExteriorTypeID`, `name`, `isLocked`, `isInvalid`, `reasonString`. Guard every receiver before lookup and every value before serialization, including after earlier observations revoke access. No generic iteration, recursion or object retention.

Bounds: three API calls per snapshot, ten snapshots, 16 tuple positions, 256-byte strings and 128-byte labels. Pinned `HouseExteriorUIDocumentation.lua:40–66` and `HouseExteriorConstantsDocumentation.lua:6–43` declare the current producers and fields. Current `reasonString` is not historical `lockReasonString`; this recorder does not establish that historical contract. Ten actual TOC/slash fixtures prove local mechanics only. No setters, entering a house, fixture-debug calls, 3D/camera behavior, native lock/default meanings, ordering or completeness claims.

```sh
luajit docs/addons/ApiContractProbe/tests/house_exterior_options.lua docs/addons/ApiContractProbe
```

## Manual expansion audio fields

`/apicontract expansion-audio-fields <label>` is excluded from `all`. Read only the published globals `LE_EXPANSION_CLASSIC` and `LE_EXPANSION_LEVEL_CURRENT`, requiring accessible finite numeric values without fallback. Independently call the global `GetExpansionDisplayInfo(value)` twice per name, omitting `desiredReleaseType`. Recheck the original value after API lookup/function guards. The generated `Expansion` system has no `Namespace`; no `C_Expansion` substitute is used.

`expansionAudioFields.expansions` records each name and two observations, preserving raw arity, nil positions and opaque errors. Only the first returned table/userdata exposes guarded `glueAmbianceSoundKit`, `glueCreditsSoundKit` and `glueMusicSoundKit` scalar fields. Recheck the receiver before every field lookup, including after prior observations revoke access. Never traverse other fields, retain objects or forward sound-kit values.

Bounds: four API calls per snapshot, ten snapshots, 16 tuple positions, 256-byte strings, 128-byte labels. Pinned `ExpansionDocumentation.lua:55–68,297–311` declares the producer and fields; glue UI consumers reference the two global input names. Eleven actual TOC/slash fixtures establish recorder mechanics only. No playback, requests, mutation, asset validity, defaults, stability, release-type or native-conformance claims.

```sh
luajit docs/addons/ApiContractProbe/tests/expansion_audio_fields.lua docs/addons/ApiContractProbe
```

## Manual Perks criteria

`/apicontract perks-criteria <label>` is excluded from `all`. It calls `C_PerksActivities.GetPerksActivitiesInfo()` twice independently with no arguments. Only the first returned root's declared `activities` list is inspected, at positions 1–8. Each activity records only `ID`, `criteriaList` positions 1–4 (`criteriaID`, `requiredValue`), and `requirementsList` positions 1–4 (`completed`, `requirementText`). Every container, entry and field is access-guarded before lookup or serialization. No generic traversal or recursive expansion occurs.

`perksCriteria[1..2]` preserves raw status, arity and nil positions; nested observations live under `values[1].fields.activities.entries`. Bounds: two API calls per snapshot, ten snapshots, 16 tuple positions, 256-byte strings, 128-byte labels. Missing, restricted and failing observations do not suppress the second call or peer entries. No refresh, claim, tracking or other mutations are performed.

Pinned retail/PTR `PerksActivitiesDocumentation.lua` declares the root and activity lists; `PerksVendorConstantsDocumentation.lua` declares `CriteriaRequiredValue` and `CriteriaRequirement`. The root is not an activity array. Removed `PerksActivityCriteria`/`PerksActivityRequirement` names are not substituted for current types. Ten actual TOC/slash fixtures prove local mechanics only; native criteria values, ordering, completeness, changed-contract and historical behavior remain unverified.

```sh
luajit docs/addons/ApiContractProbe/tests/perks_criteria.lua docs/addons/ApiContractProbe
```

## Manual transmog slot visual info

`/apicontract transmog-slot-visual-info <label>` is excluded from `all`. Four cases use the literal string descriptors `HEADSLOT` and `SHOULDERSLOT`, each with `false` and `true` secondary input, and the guarded published `Enum.TransmogType.Appearance` value. Call the original `TransmogUtil.CreateTransmogLocation(descriptor, type, secondary)` **without a receiver argument**, then original `location:GetData()`, then `C_Transmog.GetSlotVisualInfo(originalData)`. No replacement location/data tables, numeric enum fallbacks, field coercion or slot/type remapping are used.

`transmogSlotVisualInfo.cases` stores descriptor/secondary scalar observations and `factory`, `data`, `visual` tuples. Guard factory inputs and location receivers after lookup/function guards. Before forwarding the first GetData table, guard its exact `slotID`, `type`, `modification` fields as finite numbers; recheck fields after later field lookups and again after query lookup/function guards. Missing, invalid, restricted and failed cases remain independent. Only the first visual object exposes nine guarded fields: `baseSourceID`, `baseVisualID`, `appliedSourceID`, `appliedVisualID`, `pendingSourceID`, `pendingVisualID`, `hasUndo`, `isHideVisual`, `itemSubclass`. Other returned objects remain opaque; nothing is retained between captures.

Each case additionally records `sources.baseSourceID` and `sources.appliedSourceID` independently. Reread these fields from the original first visual object after its field observations, never from the serialized copies; require accessible finite numbers and recheck each ID after namespace lookup/function guards. Call `C_TransmogCollection.GetAppearanceSourceInfo(originalID)` once per usable field. Preserve each raw tuple and inspect only its first object's ten declared fields: `category`, `itemAppearanceID`, `canHaveIllusion`, `icon`, `isCollected`, `itemLink`, `transmoglink`, `sourceType`, `itemSubclass`, `ignoreModelAttachmentChecksForIllusion`. Every receiver and value is guarded; the last field is a raw observation, not a 3D instruction. No fallback source, deduplication, ID invention, comparison or recursive forwarding occurs.

Bounds: **20 recorder-level invocations per snapshot** (four factories, four GetData methods, four visual queries, up to eight source queries), ten snapshots, 16 tuple positions, 256-byte strings and 128-byte labels. Vendor factory-internal calls are explicitly excluded from that invocation count. Native visual/source state, collection semantics, historical contracts and restricted-context behavior remain unverified; no transmog application, outfit selection, equipment/account mutation or 3D behavior is performed.

Source: retail `Blizzard_TransmogShared.lua:61–121,149–189,532–540,604–611`, `Blizzard_Wardrobe.lua:1073`, `TransmogDocumentation.lua:86–101,303–315`, `TransmogSharedDocumentation.lua:105–114`, and `TransmogItemsDocumentation.lua:196–208,1163–1179`. The vendor's equipped-slot path passes base/applied source IDs through to `GetAppearanceSourceInfo`; the recorder does not reproduce its fallback or model logic. Eighteen cumulative actual TOC/slash fixtures (eleven existing, seven new) establish local recorder mechanics only, including bounded execution of unmodified pinned vendor factory/Set/GetData source sections with local dependencies, not a native client run.

```sh
luajit docs/addons/ApiContractProbe/tests/transmog_slot_visual_info.lua docs/addons/ApiContractProbe
```

## Manual equipped transmog eligibility

`/apicontract equipped-transmog-eligibility <label>` is excluded from `all`. For slots 1–19, call the original `ItemLocation:CreateFromEquipmentSlot(slot)` method, then independently pass its first accessible table/userdata result once to `C_Item.CanItemTransmogAppearance`. Recheck the constructor receiver after method guards and the original location after namespace/function guards. Never clone locations, inspect their fields, substitute links, or retain objects between captures.

`equippedTransmogEligibility.slots` records `slot`, `producer` and `eligibility`. Preserve exact arity, nil positions and opaque errors, including both eligibility outputs. Each tuple is bounded to 16 positions, strings to 256 bytes, labels to 128 bytes, and captures to ten snapshots. Maximum: 19 constructor and 19 query calls per snapshot. Missing or failing slots do not suppress peers.

Pinned `Blizzard_ObjectAPI/Mainline/ItemLocation.lua:15–18` establishes the constructor, and `ItemDocumentation.lua:34–47` establishes the eligibility signature. Nine actual TOC/slash fixtures prove recorder mechanics only; native eligibility/error meanings and historical changed-contract behavior remain unverified. No equip, transmog apply, purchase or other item-state mutation is performed.

```sh
luajit docs/addons/ApiContractProbe/tests/equipped_transmog_eligibility.lua docs/addons/ApiContractProbe
```

## Manual equipped item info

`/apicontract equipped-item-info <label>` is excluded from `all`. For equipment slots 1–19, it calls `GetInventoryItemLink("player", slot)` once and independently forwards each original accessible string link to `C_Item.GetItemInfo` once. Missing, invalid, restricted or failing producers do not suppress other slots. Links are rechecked after namespace/function guards and never truncated before invocation.

`equippedItemInfo.slots` stores `slot`, `producer` and `itemInfo`. **Only this mode's item-info result captures up to 18 return positions**, matching the pinned declaration; producer tuples and shared observers retain their 16-position bounds. Exact `n` and nil holes are preserved, including positions 16–18; results exceeding 18 positions set `truncated` and never inspect position 19. Scalar output strings remain capped at 256 bytes, labels at 128 bytes, and captures at ten snapshots. Maximum: 19 producer calls plus 19 item-info calls per snapshot. Outputs remain opaque where necessary, unretained, and are never forwarded.

Pinned `ItemDocumentation.lua:604–636` declares all 18 returns. `Blizzard_AuctionHouseUI/Shared/Blizzard_AuctionHouseUtil.lua:340–345` passes an item link to `C_Item.GetItemInfo`. These sources ground call shape, not native acceptance or historical changed-contract semantics. No ItemLocation, transmog, item mutation or native-conformance claims. Eleven actual TOC/slash fixtures cover recorder mechanics only:

```sh
luajit docs/addons/ApiContractProbe/tests/equipped_item_info.lua docs/addons/ApiContractProbe
```

## Manual combat audio settings reads

`/apicontract combat-audio-settings-read <label>` is excluded from `all`. It calls `C_CombatAudioAlert.IsEnabled()` twice, `GetSpecSetting(value)` twice for nine fixed published `Enum.CombatAudioAlertSpecSetting` names, and `GetThrottle(value)` twice for eleven fixed published `Enum.CombatAudioAlertThrottle` names. No enum iteration or numeric fallback occurs.

`combatAudioSettingsRead` stores `IsEnabled` observations and named `specSettings`/`throttles` rows. Every publication container/member and API lookup is guarded; original accessible finite enum values are rechecked after namespace/function guards before invocation. Calls are independent, preserving raw arity, nil positions and opaque errors. Bounds: 42 calls per snapshot, ten snapshots, sixteen return positions, 256-byte scalar strings and 128-byte labels. Returned objects remain opaque and unretained.

Pinned `CombatAudioAlertDocumentation.lua` declares the three read signatures; `CombatAudioAlertSharedDocumentation.lua` declares the fixed names. `Blizzard_SettingsDefinitions_Shared/AudioAssist.lua` uses the published enum paths. No setters, `SpeakText`, targeting-list operations or playback occur; no throttling enforcement, security, native defaults/ranges or CVar interpretation is established. Ten actual TOC/slash fixtures prove recorder mechanics only; native behavior remains unverified.

```sh
luajit docs/addons/ApiContractProbe/tests/combat_audio_settings_read.lua docs/addons/ApiContractProbe
```

## Manual threat lead read

`/apicontract threat-lead-read <label>` independently calls `UnitThreatLeadSituation` once for each exact pair: `("player", "target")`, `("player", "focus")`, `("player", "party1")`, and `("player", "nonexistent")`. It is excluded from `all`. Both arguments are original unit tokens: the second parameter is named `mobGUID` but declared `UnitToken`; no GUID producer or conversion is used.

`threatLeadRead.pairs` records guarded `unit`/`mob` observations and raw `result` tuples. Both tokens are checked before lookup and rechecked after global lookup/function guards. Conditional-secret results remain opaque before inspection or serialization. Bounds: four calls per snapshot, ten snapshots, sixteen return positions, 256-byte scalar strings and 128-byte labels. Nil positions, zero returns and opaque errors remain distinct; failures do not suppress peer pairs.

Pinned `UnitDocumentation.lua:3134–3153` and `Blizzard_UnitFrame/Shared/CompactUnitFrame.lua:622` establish the signature and consumer shape, not native results. No combat/threat mutation, restricted-context experiment, classification, scale, default or native-conformance claim follows. Nine actual TOC/slash fixtures cover recorder mechanics only:

```text
luajit docs/addons/ApiContractProbe/tests/threat_lead_read.lua docs/addons/ApiContractProbe
```

## Manual GUID identity

`/apicontract guid-identity <label>` calls `UnitGUID(unit)` once each for `player`, `target`, and `party1`. Only an accessible string in the first return position is forwarded, unchanged, to independent `UnitClassFromGUID(guid)` and `UnitNameFromGUID(guid)` calls. No GUID parsing, coercion, invented IDs, or truncated observation copies are used as inputs. The mode is excluded from `all`; existing `names` and `full-names` modes are unchanged.

`guidIdentity.units` stores the guarded unit, raw `producer` tuple and named `queries` tuples. Token/GUID accessibility is rechecked after global lookup and function guards. Conditional-secret outputs remain opaque before inspection or serialization. Class/name/server values are not combined, compared or interpreted; exact arity and nil positions are preserved, including three class and two name returns. Bounds: nine calls per snapshot, ten snapshots, sixteen return positions, 256-byte scalar strings and 128-byte labels. Returned objects are not traversed or retained.

Pinned `UnitDocumentation.lua:937–953`, `1213–1227`, and `2417–2432` establish the argument/return shapes and conditional identity secrecy, not native results. No restricted-context/security experiment, mutation, identity/class/realm inference or native-conformance credit follows. Native fixtures, transitions and historical semantics remain unverified. Eleven actual TOC/slash fixtures cover recorder mechanics only:

```sh
luajit docs/addons/ApiContractProbe/tests/guid_identity.lua docs/addons/ApiContractProbe
```

## Manual full names

`/apicontract full-names <label>` calls `UnitFullName` once for each fixed token: `player`, `target`, `focus`, `pet`, `party1`, `nonexistent`, `invalid-unit-token`, and the empty string. It is excluded from `all`; existing `names` observations remain unchanged.

`fullNames.units` records guarded unit observations and raw `result` tuples. Global lookup/function guards precede invocation, and token access is rechecked after function guards. Every returned value passes the existing scalar guards before inspection or serialization; conditional-secret results stay opaque. Preserve exact arity and nil positions, including both name/server returns, without joining or comparing them. Bounds: eight calls per snapshot, ten snapshots, sixteen return positions, 256-byte scalar strings and 128-byte labels. Returned objects are not retained.

Pinned `UnitDocumentation.lua:1196–1211` declares `SecretWhenUnitIdentityRestricted=true` and `SecretArguments=AllowedWhenUntainted`. This mode observes ordinary accessible results only, never restricted-identity experiments, realm defaults, coercion or native classifications. Native same-/cross-realm fixtures and historical behavior remain unverified. Ten actual TOC/slash fixtures prove recorder mechanics only:

```sh
luajit docs/addons/ApiContractProbe/tests/full_names.lua docs/addons/ApiContractProbe
```

## Manual cloak/helm transition — changes appearance

**`/apicontract cloak-helm-transition <label>` changes cloak/helm appearance when later manually run. Restoration is attempted but can fail. No native execution was performed during preparation.** This explicit mode is excluded from `all` and `player-state-queries`; existing read modes never invoke its setters.

Each independent cloak/helm lane reads `ShowingCloak()`/`ShowingHelm()` once. Only an original accessible boolean baseline permits any setters. An eligible lane attempts setter `false` → read, setter `true` → read, then setter **original baseline** → final read. Baseline access is checked before every experimental setter and again after lookups/function/argument guards. Native calls are protected; setter errors do not bypass cleanup. A skipped setter has a skipped paired read. Bounds: seven appearance API calls per lane, fourteen total per snapshot, ten snapshots, sixteen tuple positions, 256-byte scalar strings and 128-byte labels.

`cloakHelmTransition.cloak` / `.helm` retain only safe observations: `baseline`, two `steps` (each `setter` and `read`), and `restoration`. Cleanup records `restoration-skipped`, `restoration-error` or `restoration-unconfirmed` on failure; no setter attempts means `not-needed`. `confirmed-by-observation` requires an actually invoked, nonthrowing restore call and an accessible boolean final getter equal to the still-accessible original baseline. This label is an observation, not a native semantic guarantee. No inaccessible value is substituted or forwarded.

After any attempted mutation with unconfirmed restoration, the addon-session lock rejects subsequent transition captures (`blocked-restoration-unconfirmed`) without further appearance calls, including after SavedVariables are cleared. The other lane in the current capture still runs independently. There are no automatic retries; read-only modes remain usable. No security, rendering, equipment or CVar coupling is claimed. Pinned `PlayerScriptDocumentation.lua:1530–1566` supplies setter/getter signatures only; native transitions remain unverified.

Fourteen actual TOC/slash local modeled-setter fixtures cover preparation mechanics only:

```sh
luajit docs/addons/ApiContractProbe/tests/cloak_helm_transition.lua docs/addons/ApiContractProbe
```

## Manual player state queries

`/apicontract player-state-queries <label>` is excluded from `all`. It independently calls the globals `GetCollapsingStarCost`, `ShowingCloak` and `ShowingHelm` twice each with zero arguments. Every call performs a fresh guarded `_G` field lookup and function-access check; missing, restricted or throwing globals do not suppress peers.

`playerStateQueries` stores named pairs of raw observations, preserving zero returns, nil positions and opaque errors. Bounds: six calls per snapshot, ten snapshots, sixteen tuple positions, 256-byte scalar strings and 128-byte labels. Returned objects stay opaque and are not retained. No `ShowCloak`, `ShowHelm`, setters or purchases occur; no cost, default, stability or native behavior is inferred.

Pinned `PlayerScriptDocumentation.lua:275–282,1550–1566` declares the three no-argument signatures without secret annotations. Eleven actual TOC/slash fixtures prove local recorder mechanics only:

```sh
luajit docs/addons/ApiContractProbe/tests/player_state_queries.lua docs/addons/ApiContractProbe
```

## Manual outfit tooltip

`/apicontract outfit-tooltip <label>` is excluded from `all`. One `C_TransmogOutfitInfo.GetOutfitsInfo()` call supplies the first four list entries; only guarded original finite `outfitID` values feed independent `C_TooltipInfo.GetOutfit(id)` calls. Access is rechecked after namespace/function guards. No synthesized IDs or serialized input substitutes are used.

`outfitTooltip` preserves the producer tuple; its first table observation has four bounded `entries`, each with `id` and `query` observations. TooltipData stays opaque: no lines, args, fields, tooltip UI or recursive traversal. Errors and missing/restricted entries do not suppress peers; returned objects are not retained.

Bounds: five API calls per snapshot, ten snapshots, sixteen tuple positions, 256-byte scalar strings and 128-byte labels. No outfit selection, requests, mutations or native tooltip-content claims. Pinned `TooltipInfoDocumentation.lua:646–662` declares `GetOutfit(number) -> TooltipData`, `MayReturnNothing` and `SecretArguments = AllowedWhenUntainted`; `TransmogOutfitInfoDocumentation.lua:367–375` supplies the list producer. Nine actual TOC/slash fixtures prove recorder mechanics only:

```sh
luajit docs/addons/ApiContractProbe/tests/outfit_tooltip.lua docs/addons/ApiContractProbe
```

## Manual tradeskill item quality

`/apicontract tradeskill-item-quality <label>` is excluded from `all`. For equipment slots 1–19, `GetInventoryItemLink("player", slot)` supplies the original accessible string to independent `C_TradeSkillUI.GetItemCraftedQualityInfo` and `GetItemReagentQualityInfo` calls. Inputs are never replaced by serialized/truncated links; access is rechecked after namespace/function guards.

`tradeskillItemQuality.slots[]` preserves producer and query tuples; each query's `info.fields` observes only the first returned object's thirteen declared `CraftingQualityInfo` fields: quality, icon, iconSmall, iconInventory, iconMixed, iconAppear, iconDissolve, barFill, barBackground, barBackgroundCap, barHighlight, iconChat and iconQuestObjective. Every field lookup rechecks its receiver. Missing links, inaccessible values and errors remain independent observations.

Bounds: 19 producers plus 38 queries per snapshot, ten snapshots, sixteen tuple positions, 256-byte output strings and 128-byte labels. No recipe queries, crafting/orders, mutations, atlas interpretation or native quality claims. Pinned `TradeSkillUIDocumentation.lua:334–376`, `TradeSkillUITypesDocumentation.lua:194–211` and `Blizzard_ItemButton/Mainline/ItemButtonTemplate.lua:330–334` ground signatures, fields and item-link inputs. Ten actual TOC/slash fixtures prove recorder mechanics only:

```sh
luajit docs/addons/ApiContractProbe/tests/tradeskill_item_quality.lua docs/addons/ApiContractProbe
```

## Manual nameplate metrics

`/apicontract nameplate-metrics <label>` is excluded from `all`. It independently calls `C_NamePlate.GetNamePlateSize()` twice and `C_NamePlateManager.GetNamePlateHitTestInsets(value)` twice for each fixed published `Enum.NamePlateType.Friendly` and `Enemy` value. Only accessible finite published numbers are forwarded, without fallback; values are rechecked after namespace/function guards.

`nameplateMetrics.size` and `nameplateMetrics.modes[].observations` preserve raw arity, nil positions and opaque errors. Pinned size/inset signatures declare two/four returns, but captures do not enforce that shape or infer geometry, defaults or stability. Bounds: six calls per snapshot, ten snapshots, sixteen tuple positions, 256-byte strings and 128-byte labels. No setters, camera/3D or unit queries; native behavior remains unverified.

Pinned sources: `NamePlateDocumentation.lua:27–34`, `NamePlateManagerDocumentation.lua:11–28`, and `NamePlateConstantsDocumentation.lua:152–162`. Eleven actual TOC/slash fixtures test recorder mechanics:

```text
luajit docs/addons/ApiContractProbe/tests/nameplate_metrics.lua docs/addons/ApiContractProbe
```

## Manual quest favor

`/apicontract quest-favor <label>` is excluded from `all`. It calls `C_NeighborhoodInitiative.GetNeighborhoodInitiativeInfo()` once, retaining its bounded opaque tuple. Only the first returned object's `tasks` positions 1–4 supply original accessible finite `rewardQuestID` values. Each independently feeds `C_QuestInfoSystem.GetQuestLogRewardFavor(id)` and `GetQuestLogRewardFavor(id, true)` with exact one/two-argument calls.

After the unchanged produced-ID matrix, `questFavor.omissions.omitted` records `GetQuestLogRewardFavor()` and `questFavor.omissions.unclamped` records `GetQuestLogRewardFavor(nil, false)` independently, even when the producer or task inputs are unavailable. These preserve exact zero/two-argument arity; explicit nil/false inputs are checked before lookup and again after function guards.

Every receiver/field lookup is guarded; IDs are rechecked after API lookup/function guards. Missing, restricted, malformed and error outcomes do not suppress peers. Maximum eleven API calls per snapshot, ten snapshots, sixteen tuple positions, 256-byte output strings and 128-byte labels. No guessed IDs, requests, refreshes, quest/reward mutations or default/cap/equality/favor conclusions. Native behavior remains unverified.

Pinned `QuestInfoSystemDocumentation.lua:42–55` declares optional quest/clamp arguments; `NeighborhoodInitiativeDocumentation.lua:94–101,308–350` supplies the task producer and `rewardQuestID` field. Seventeen cumulative actual TOC/slash fixtures (ten existing, seven new) prove recorder mechanics only:

```text
luajit docs/addons/ApiContractProbe/tests/quest_favor.lua docs/addons/ApiContractProbe
```

## Manual empowered stages

`/apicontract empowered-stages <label>` independently calls `UnitEmpoweredStageDurations(unit)`, `UnitEmpoweredStagePercentages(unit, false)` and `UnitEmpoweredStagePercentages(unit, true)` for player, target, focus, party1, nonexistent, invalid-unit-token and the empty token. Results live under `empoweredStages.units[].queries` as `durations`, `percentagesWithoutHold` and `percentagesWithHold`.

Only the first returned table exposes positions 1–8 in `entries`, preserving scalar kind/status. Duration entries additionally use the shared `inspectDuration` ten-method whitelist for current-only observations; percentage entries remain scalar-only. Receivers are guarded before method lookup and rechecked after function guards. No objects are retained and cast-duration state is unchanged. Function and token access are guarded, tokens rechecked after function guards, and the returned table rechecked before every index lookup. Peer failures remain independent. Raw arity, nil positions and opaque errors are preserved within sixteen result positions, 256-byte strings, 128-byte labels and ten shared snapshots (21 producer calls plus at most 560 duration-method calls per snapshot). Excluded from `all`.

Pinned retail `UnitDocumentation.lua:1129–1165` declares both stage-list APIs with `MayReturnNothing` and `SecretArguments=AllowedWhenUntainted`, without secret-return annotations. Explicit false/true observations do not establish default or percentage semantics. No cast initiation, mutation, stage count/order, lifecycle or native-conformance claims. Fifteen actual TOC/slash fixtures (ten existing, five new) prove local recorder mechanics only:

```text
luajit docs/addons/ApiContractProbe/tests/empowered_stages.lua docs/addons/ApiContractProbe
```

## Manual encounter warning state

`/apicontract encounter-warning-state <label>` independently calls only `C_EncounterWarnings.IsFeatureAvailable()` and `IsFeatureEnabled()` twice each with zero arguments. Results live under `encounterWarningState.IsFeatureAvailable` and `encounterWarningState.IsFeatureEnabled`. Each read performs fresh guarded namespace/function lookup; missing, restricted, throwing or replaced APIs do not suppress peer observations.

Excluded from `all`; four calls per snapshot, ten shared snapshots, sixteen result positions, 256-byte scalar strings and 128-byte labels. Raw arity, nil positions and opaque errors are preserved without retaining returned objects. No toggles, sounds, warning creation, mutations or security experiments. Repeated reads establish no native availability, enabled-state, default or stability semantics.

Pinned retail `EncounterWarningsDocumentation.lua:76–94` declares both no-argument boolean queries without secret annotations; it establishes call shape, not native results. Twelve actual TOC/slash fixtures exercise recorder mechanics only:

```text
luajit docs/addons/ApiContractProbe/tests/encounter_warning_state.lua docs/addons/ApiContractProbe
```

## Manual ping enabled

`/apicontract ping-enabled <label>` calls only `C_Ping.IsPingSystemEnabled()` twice independently with zero arguments. Observations are stored under `pingEnabled.IsPingSystemEnabled`. Fresh guarded namespace/function lookup keeps missing, restricted, throwing or replaced APIs independent.

Excluded from `all`; raw arity, nil positions and opaque errors are preserved within sixteen result positions, 256-byte scalar strings, 128-byte labels and ten shared snapshots (two calls per snapshot). No raw objects are retained. No secure ping send/toggle/lifecycle APIs or CVars are queried or changed. Repeated observations establish no native enabled-state, default, stability or security semantics.

Pinned retail `PingManagerDocumentation.lua:44–52` declares the no-argument boolean query without secret/restriction annotations; this establishes call shape, not native results. Twelve actual TOC/slash fixtures cover recorder mechanics:

```text
luajit docs/addons/ApiContractProbe/tests/ping_enabled.lua docs/addons/ApiContractProbe
```

## Manual stable bonus slot

`/apicontract stable-bonus-slot <label>` independently calls only `C_StableInfo.IsBonusPetSlotAvailable()` twice with no arguments. Results are stored under `stableBonusSlot.IsBonusPetSlotAvailable`. Each call freshly uses guarded namespace/function lookup; missing, restricted, throwing or replaced APIs do not suppress the other observation.

The mode is excluded from `all`. It preserves raw arity, nil positions and opaque errors, bounded to sixteen result positions, 256-byte scalar strings, 128-byte labels and ten shared snapshots (twenty query calls maximum). No raw objects are retained. It performs no pet-ID/list queries, summon, stable move, rename, request or mutation operations. Repeated values establish neither stability nor native availability/default/security semantics.

Pinned retail `StableInfoDocumentation.lua:99–105` declares the no-argument boolean result; `Blizzard_StableUI.lua:749–753` consumes it for the secondary-pet button. These sources establish the call shape, not native results. Eleven actual TOC/slash fixtures test recorder mechanics only:

```text
luajit docs/addons/ApiContractProbe/tests/stable_bonus_slot.lua docs/addons/ApiContractProbe
```

## Manual cooldown viewer reads

`/apicontract cooldown-viewer-read <label>` queries the nine fixed published `Enum.CooldownViewerCategory` names: Essential, Utility, TrackedBuff, TrackedBar, GroupBuff, SpecAgnosticEssential, SpecAgnosticTracked, EquipSlotEssential and EquipSlotTracked. No numeric fallback is used. Each category feeds `GetCooldownViewerCategorySet(value, false)`; the first eight original accessible finite IDs independently feed `GetCooldownViewerCooldownInfo(id)` and `GetValidAlertTypes(id)`. Duplicate IDs remain independent observations.

Only `cooldownID` and `category` are inspected on the first info object; only the first eight scalar entries of the first alert list are inspected. Every receiver is guarded before lookup; category/ID access is rechecked after API lookup and function guards. Raw tuples retain arity/nil positions and opaque errors, capped at 16 positions and 256-byte strings. Labels cap at 128 bytes, captures at ten, API calls at 153 per capture. Excluded from `all`; no linked-field traversal, flag interpretation, refreshes or mutations.

Pinned `CooldownViewerDocumentation.lua` and `CooldownViewerConstantsDocumentation.lua` establish call shapes and publication names, not native defaults, membership, ordering, completeness or alert semantics. Eight actual TOC/slash fixtures prove recorder mechanics only; native behavior remains unverified.

```sh
luajit docs/addons/ApiContractProbe/tests/cooldown_viewer_read.lua docs/addons/ApiContractProbe
```

## Manual prey quest widgets

`/apicontract prey-quest-widgets <label>` calls `C_QuestLog.GetActivePreyQuest()` once, then uses only its original accessible finite first quest ID for three independent `C_TaskQuest.GetQuestUIWidgetSetByType` calls. Inputs use published `Enum.MapIconUIWidgetSetType.Tooltip`, `BehindIcon`, and `AdventureMapDetails` values, never numeric fallbacks. Both inputs are rechecked after API lookup/function guards.

Distinct `details` preserve existing query tuples. Each original finite widget-set first return feeds `GetAllWidgetsBySetID` once; only its first four entries supply guarded original `widgetID`/`widgetType`. Only a match with accessible published `Enum.UIWidgetVisualizationType.PreyHuntProgress` permits visualization lookup, with all dispatch inputs rechecked after lookup/function guards. The first visualization object exposes sixteen declared fields as scalar observations, without nested traversal.

Manual-only, excluded from `all`: at most nineteen API calls per snapshot, ten shared snapshots, sixteen return positions, 256-byte output strings and 128-byte labels. Raw arity, nil positions and opaque errors remain distinct. No wrong-type calls, quest acceptance/abandonment, requests, mutations, completeness or native classification/default claims.

Pinned retail `QuestLogDocumentation.lua:96–103`, `QuestTaskInfoDocumentation.lua:111–125`, and `UIWidgetManagerSharedDocumentation.lua:6–15` supply call shapes and enum names, not native outputs. `UIWidgetManagerDocumentation.lua:11–24,278–290,1626–1646,2073–2081` and the published visualization enum ground the extension. Sixteen cumulative actual TOC/slash fixtures (eight original, six widget-chain cases, two revocation regressions) prove local mechanics only:

```text
luajit docs/addons/ApiContractProbe/tests/prey_quest_widgets.lua docs/addons/ApiContractProbe
```

## Manual major faction renown rewards

`/apicontract major-faction-renown-rewards <label>` is manual-only, excluded from `all`. It records one explicit-nil `GetMajorFactionIDs(nil)` call, then up to eight original finite faction IDs through `GetRenownLevels`. First four level entries expose `factionID`, `level`, `locked`, `isMilestone`, `isCapstone`; each original accessible finite level independently feeds `GetRenownRewardsForLevel(originalFactionID, originalLevel)`. Entry `factionID` never replaces the producer ID.

First four rewards expose fifteen fixed fields: `renownRewardID`, `uiOrder`, `isAccountUnlock`, `itemID`, `spellID`, `mountID`, `transmogID`, `transmogSetID`, `titleMaskID`, `transmogIllusionSourceID`, `icon`, `name`, `description`, `toastDescription`, `rewardType`. Pinned `MajorFactionsDocumentation.lua:265–297` also declares `isCollected`; that field is outside this slice. Both original call inputs are rechecked after API lookup/function guards. Every list/entry/field access is guarded; unavailable inputs and errors remain independent.

Bounds: 41 calls per snapshot, ten snapshots, sixteen raw return positions, 256-byte strings, 128-byte labels. No reward-ID forwarding, recursion, deduplication, unlock/claim/request/mutations, retained objects or inferred ordering/completeness/equality/native semantics. Native reward fixtures remain unverified. Ten actual TOC/slash fixtures prove recorder mechanics only:

```sh
luajit docs/addons/ApiContractProbe/tests/major_faction_renown_rewards.lua docs/addons/ApiContractProbe
```

## Manual major faction journey

`/apicontract major-faction-journey <label>` is excluded from `all`. Call `C_MajorFactions.GetMajorFactionIDs(nil)` once with **one explicit nil** expansion argument. Only first-return table indices 1–8 supply original accessible finite faction IDs to two independent predicates: `ShouldDisplayMajorFactionAsJourney(ID)` and `ShouldUseJourneyRewardTrack(ID)`. Preserve duplicates and fractional inputs without interpreting them; never invent IDs or expansion values.

Guard namespace/function/list before lookup or inspection and recheck each original ID after predicate lookup/function guards. Record raw arity, nil holes and opaque errors; failures do not suppress peer predicates or entries. Bounds: twenty-five API calls per snapshot, ten snapshots, sixteen return positions, 256-byte scalar strings and 128-byte labels. Raw objects are not retained. No renown reward/level queries, mutations, ordering/completeness or native classification claims; reward structures and matching native fixtures remain pending.

Each original ID also independently feeds `GetMajorFactionData(ID)` once. Preserve its raw tuple and inspect only the first returned data object: `description`, `playerCompanionID`, and an opaque scalar `highlights` observation, plus the first four highlights with `title`, `description`, and `level`. Every container/index/field read is guarded; no companion or reward followups. Data inspection uses pinned declarations at `MajorFactionsDocumentation.lua:26–39,228–253,300–307`.

Pinned retail `MajorFactionsDocumentation.lua:41–53,156–184` supplies the nilable producer argument and predicate signatures, not native outputs. Fifteen cumulative actual TOC/slash fixtures (ten existing, five new) prove recorder mechanics only:

```sh
luajit docs/addons/ApiContractProbe/tests/major_faction_journey.lua docs/addons/ApiContractProbe
```

## Manual training grounds structures

`/apicontract training-grounds-structures <label>` is excluded from `all`. Independently call `C_PvP.GetTrainingGrounds()` and `C_PvP.GetRandomTrainingGroundRewards()` once each. `trainingGroundsStructures.grounds` preserves raw arity and inspects only the first returned table at indices 1–8. Each entry exposes the fourteen declared fields: `name`, `icon`, `gameType`, `shortDescription`, `longDescription`, `mapDescription`, `maxPlayers`, `battlegroundID`, `lfgDungeonID`, `mapID`, `isHoliday`, `isRandom`, `canEnter`, `isTrainingGround`.

Pinned retail `PvpInfoDocumentation.lua:599–610` declares **five reward returns**, not one structure: `honor`, `experience`, `itemRewards`, `currencyRewards`, `roleShortageBonus`. `rewards` records that raw tuple; nested objects remain opaque. Lines 767–774 and 1607–1625 define the training producer and fields. Guard every table, entry and field before lookup and every value before serialization. Preserve nil holes, zero returns, opaque errors and independent failures; retain no raw objects.

Bounds: two calls per snapshot, ten snapshots, sixteen return positions, 256-byte scalar strings, 128-byte labels. No queue/join/request/mutation, nested reward inspection, ordering, completeness or native classification claims. Eleven actual TOC/slash fixtures establish recorder mechanics only; native behavior remains unverified.

```sh
luajit docs/addons/ApiContractProbe/tests/training_grounds_structures.lua docs/addons/ApiContractProbe
```

## Manual housing catalog

`/apicontract housing-catalog <label>` is excluded from `all`. Call `C_HousingCatalog.HasFeaturedEntries()` twice and independently call `C_CatalogShop.GetNewProducts()` once. Only the first returned product list at indices 1–8 supplies original accessible finite product IDs. Each ID permits one independent `GetFirstCategoryByProductID(ID)` call; inspect only its first returned object through six fixed fields: `ID`, `displayName`, `iconTexture`, `linkTag`, `isDisabled`, `showPersistentRefundButton`.

Each product row additionally records `categoryProducts`: reread the original first category object's accessible finite `ID` after existing field capture, then call `GetProductIDsForCategory(originalID)` once. Recheck the receiver before reading `ID` and the ID after namespace/function guards. Preserve raw tuples and inspect only first-return list indices 1–8 as scalars. No deduplication, recursive queries or product details.

Independently call `GetRefundableDecors()` once with its nilable `productIdFilterOpt` **omitted**, not passed explicitly as nil. Inspect only the first returned list at indices 1–8 through `decorGUID`, `timeRemainingSeconds`, `name`, `price`; preserve the raw second return `minTimeRemainingSeconds`. `standaloneDecorProductID` is not declared in the pinned current `RefundableDecorInfo` and remains missing, not inferred from another field.

Guard every receiver before each index/field lookup and every value before inspection/serialization. Recheck original product IDs after namespace/function lookup and function guards, immediately before forwarding. Missing, restricted, invalid and failed observations do not suppress peers. No iteration/length lookup, fabricated IDs, retained raw objects, requests, purchases, refunds, mutations, currency guesses, native price/default/stability or completeness claims.

`currencies` independently reads `Constants.CatalogShopVirtualCurrencyConstants.HEARTHSTEEL_VC_CURRENCY_CODE` and `TRADERS_TENDER_VC_CURRENCY_CODE`, then calls `GetVirtualCurrencyBalance` once per original accessible string. Guard each container/member and recheck the string after API lookup/function guards. Never substitute literals or forward the truncated observation; long original strings remain intact. Missing publications and peer failures remain independent. Pinned `CatalogShopConstantsDocumentation.lua:1–16` and `CatalogShopDocumentation.lua:314–326` establish publication and signature only; no refresh or currency-state claims.

Bounds: four base plus eight category, eight category-product followup and two currency calls (twenty-two total) per snapshot, ten shared snapshots, sixteen tuple positions, 256-byte scalar strings and 128-byte labels. Pinned `HousingCatalogUIDocumentation.lua:287–293` and `CatalogShopDocumentation.lua:141–162,259–272,673–683,838–846` define the signatures and fields, not native results. Twenty-four cumulative actual TOC/slash fixtures (eighteen existing, six new) prove recorder mechanics only. `CatalogShopDocumentation.lua:195–207` declares the followup signature, not native ordering, completeness or equality. Run `luajit docs/addons/ApiContractProbe/tests/housing_catalog.lua docs/addons/ApiContractProbe`.

## Manual neighborhood structures

`/apicontract neighborhood-structures <label>` is excluded from `all`. Independently call `C_NeighborhoodInitiative.GetNeighborhoodInitiativeInfo()`, `GetInitiativeActivityLogInfo()` and `GetTrackedInitiativeTasks()` once. Preserve raw arity, nil positions and opaque errors within sixteen return positions; inspect only each first returned object. Initiative fields are `isLoaded`, `neighborhoodGUID`, `initiativeID`, `currentCycleID`, `progressRequired`, `currentProgress`, `playerTotalContribution`, `duration`, `tasks`, `milestones`, `title`, `description`. Keep the scalar `fields.tasks` and `fields.milestones` observations opaque and unchanged. Add separate `taskEntries` and `milestoneEntries` observations, each inspecting indices 1–4 only. Initiative tasks use the same task-field inspector as tracked task responses, without additional task API calls. Milestones expose `milestoneOrderIndex`, `requiredContributionAmount` and opaque `rewards`; separate `rewardEntries` inspects reward indices 1–4 with only `title`, `description`, `decorID`, `decorQuantity`, `favor`, `money`, `rewardQuestID`.

Activity fields are `isLoaded`, `neighborhoodGUID`, `nextUpdateTime`, `taskActivity`; inspect only activity indices 1–8 and their `taskID`, `playerName`, `taskName`, `completionTime`, `amount` fields. Only `trackedIDs` indices 1–4 from the tracked object supply original accessible finite task IDs. Independently query `GetInitiativeTaskInfo(ID)` and `GetInitiativeTaskChatLink(ID)` with exactly one original argument. Inspect only the first task-info return: `ID`, `taskName`, `description`, `progressContributionAmount`, `tracked`, `supersedes`, `timesCompleted`, `completed`, `inProgress`, `taskType`, `sortOrder`, `rewardQuestID`, `requirementsList`, `criteriaList`; keep the last two opaque.

Guard each receiver before every lookup and each value before inspection/serialization; recheck task IDs after namespace lookup and function guards. Failures do not suppress peers. No general recursion, table iteration/length lookup, fabricated IDs, retained raw objects, requests, setters or tracking mutations. Bounds: three producers plus eight task calls per snapshot, ten shared snapshots, 256-byte strings and 128-byte labels. Native fixtures, ordering, completeness, defaults and behavior remain unverified.

Pinned `NeighborhoodInitiativeDocumentation.lua:39–79,94–119,259–353` defines the calls and fields, not native results. Sixteen cumulative actual TOC/slash fixtures (eleven existing, five nested-capture additions) prove recorder mechanics only, including unchanged eight-log/four-tracked limits and eleven API calls per snapshot. Run `luajit docs/addons/ApiContractProbe/tests/neighborhood_structures.lua docs/addons/ApiContractProbe`.

## Manual training grounds state

`/apicontract training-grounds-state <label>` is excluded from `all`. Independently call `C_PvP.AreTrainingGroundsEnabled`, `CanPlayerUseTrainingGroundsUI`, and `HasRandomTrainingGroundWinToday` twice each with zero arguments. Reuse guarded namespace/function lookup and scalar serialization; missing, restricted, throwing or replaced APIs do not suppress peers. Preserve exact arity, nil positions and opaque errors, including the eligibility query's failure-reason return.

Bounds: six calls per snapshot, ten shared snapshots, sixteen return positions, 256-byte strings and 128-byte labels. No queue/join operations, requests, mutations, or training-ground/reward structure queries. Repeated observations establish neither defaults nor stability nor native behavior. Pinned `PvpInfoDocumentation.lua:20–26,57–64,846–852` establishes call shapes only.

Ten actual TOC/slash fixtures test recorder mechanics:

```sh
luajit docs/addons/ApiContractProbe/tests/training_grounds_state.lua docs/addons/ApiContractProbe
```

## Manual neighborhood state

`/apicontract neighborhood-state <label>` is excluded from `all`. Independently call seven `C_NeighborhoodInitiative` queries twice each with zero arguments: `GetActiveNeighborhood`, `GetRequiredLevel`, `IsInitiativeEnabled`, `IsPlayerInNeighborhoodGroup`, `IsViewingActiveNeighborhood`, `PlayerHasInitiativeAccess`, `PlayerMeetsRequiredLevel`. Guard namespace/function lookup and scalar serialization; preserve raw arity, nil positions and opaque errors. Missing, restricted, throwing or replaced APIs do not suppress peers.

Bounds: fourteen calls per snapshot, ten shared snapshots, sixteen return positions, 256-byte strings (including GUIDs) and 128-byte labels. No GUID parsing or group identity conclusions, requests, active/viewing changes, claims, contributions or other mutations. Do not call `GetAvailableHouseXP`, structured initiative/activity-log producers or task queries. Repeated observations establish neither defaults nor stability nor native behavior.

Pinned `NeighborhoodInitiativeDocumentation.lua:21–28,103–110,121–164` supplies the seven no-argument signatures, not native results. Ten actual TOC/slash fixtures exercise recorder mechanics only. Run `luajit docs/addons/ApiContractProbe/tests/neighborhood_state.lua docs/addons/ApiContractProbe`.

## Manual sets catalog

`/apicontract sets-catalog <label>` is excluded from `all`. Call `C_TransmogSets.GetAvailableSets()` once and independently call `IsUsingDefaultSetsFilters()` twice, all without arguments. Preserve raw arity, nil positions and opaque errors within sixteen return positions. Inspect only the first returned list at indices 1–8, reading only six guarded fields: `setID`, `name`, `collected`, `favorite`, `validForCharacter`, `grantAsPrecedingVariant`. Recheck each list/entry receiver before every lookup and each field before serialization; never iterate, measure, mutate or retain returned objects.

Bounds: three API calls per snapshot, ten shared snapshots, 256-byte scalar strings and 128-byte labels. Missing, restricted and failed observations remain independent. No downstream set-ID queries, setters, reset, selection, ordering, completeness, defaults or native-conformance claims. The sixth field, `grantAsPrecedingVariant`, is declared as a boolean in pinned `TransmogSetsDocumentation.lua:530`. Its native observed values and semantics remain unverified.

Pinned `TransmogSetsDocumentation.lua:61–68,412–419,511–530` defines the calls and fields. Actual consumers: `Blizzard_TransmogShared.lua:1024–1033` calls the list producer; `Blizzard_TransmogTemplates.lua:1456–1459,1498,1525–1531` reads collected/favorite/setID/name; `Blizzard_Wardrobe_Sets.lua:648–651` reads validity. These consumers do not establish native results. Fourteen actual TOC/slash fixtures (eleven existing, three new) prove recorder mechanics only. Run `luajit docs/addons/ApiContractProbe/tests/sets_catalog.lua docs/addons/ApiContractProbe`.

## Manual transmog source validity

`/apicontract transmog-source-validity <label>` is excluded from `all`. Call `C_TransmogCollection.GetNumTransmogSources()` once and preserve its raw tuple. Only its original accessible finite nonnegative **integer** first return permits enumeration of `1..min(count, 8)`. Call `IsValidTransmogSource(index)` independently once per derived index. These are `TransmogSource` filter indices, **not** appearance IDs or visual `baseSourceID`/`appliedSourceID` values.

`transmogSourceValidity` stores `producer`, count-validation `status`, and `sources` query observations. Guard the original count before arithmetic, after producer serialization, and again after each query lookup/function guard and derived-index access check. Do not replace the count with the capped loop bound or re-read it from serialized output. Missing, invalid, restricted and error outcomes stay explicit; one query failure does not suppress later indices. No continuation, filtering, reconstruction or fallback IDs.

Bounds: nine API calls per snapshot, ten shared snapshots, sixteen return positions, 256-byte output strings and 128-byte labels. Returned objects are never inspected or retained. No setters, filters, native validity, ordering or completeness claims. Pinned `TransmogItemsDocumentation.lua:573–580,823–836` defines the signatures; `Blizzard_Collections/Mainline/Blizzard_Wardrobe.lua:67–70` supplies the count/index loop. These sources establish inputs, not native outputs.

Thirteen actual TOC/slash fixtures (eleven existing, two count-guard revocation regressions) prove local recorder mechanics only. Run `luajit docs/addons/ApiContractProbe/tests/transmog_source_validity.lua docs/addons/ApiContractProbe`. Native behavior remains unverified.

## Manual custom-set names

`/apicontract custom-set-names <label>` is excluded from `all`. Independently call `C_TransmogCollection.GetNumMaxCustomSets()` twice and `GetCustomSets()` once. Preserve raw return arity, nil positions and opaque errors within sixteen positions. Inspect only the first returned ID table at indices 1–4, guarding the table before every lookup. Each accessible finite original ID permits one `GetCustomSetInfo(ID)` call; its accessible string first return permits one `IsValidCustomSetName(name)` call. Forward the original name, never its serialized 256-byte prefix. Recheck IDs and names after namespace/function lookup and function guards, immediately before use.

Missing, restricted, invalid and failed observations do not suppress peers. Bounds: eleven API calls per snapshot, ten shared snapshots, 256-byte output strings and 128-byte labels. No returned objects are retained. No account mutations, name-validity/identity/default/native conclusions, ItemTransmogInfo forwarding, item-list or hyperlink chains. Pinned `TransmogItemsDocumentation.lua:365–379,398–404,564–570,808–820` supplies the four signatures, not observed client behavior.

Ten actual TOC/slash fixtures prove recorder mechanics only; native behavior remains unverified. Run `luajit docs/addons/ApiContractProbe/tests/custom_set_names.lua docs/addons/ApiContractProbe`.

## Manual outfit slots

`/apicontract outfit-slots <label>` is excluded from `all`. Independently call `GetAllSlotLocationInfo()` and `GetSlotGroupInfo()` without arguments, preserving sixteen raw return positions, nil holes and opaque errors. Inspect only the first two location lists, eight entries each, and five guarded fields: `slot`, `type`, `collectionType`, `slotName`, `isSecondary`. Only these original location entries may feed `GetEquippedSlotOptionFromTransmogSlot(slot)` and `GetUnassignedAtlasForSlot(slot)`, each with exactly one original accessible finite slot argument. Recheck slot access after API lookup/function guards; failures remain independent.

Inspect only the first group return, eight entries, with `position` and nested `appearanceSlotInfo`/`illusionSlotInfo` lists capped at eight slot entries each. Group entries never trigger queries. Every receiver and field lookup/serialization is guarded; no general recursion, length lookup, iteration or raw-object retention. Cap two producers plus 32 queries per snapshot, ten shared snapshots, 256-byte strings and 128-byte labels. No mutations, 3D, guessed IDs, type/option mapping, equality, ordering, completeness, atlas validity or native claims.

Pinned `TransmogOutfitInfoDocumentation.lua:130–138,184–198,420–427,489–503,918–937` supplies exact signatures and structs. Eleven actual TOC/slash fixtures prove recorder mechanics only. Run `luajit docs/addons/ApiContractProbe/tests/outfit_slots.lua docs/addons/ApiContractProbe`.

## Manual outfit catalog

`/apicontract outfit-catalog <label>` is excluded from `all`. Call `C_TransmogOutfitInfo.GetOutfitsInfo()` once, preserving raw arity/nils/opaque errors within sixteen return positions. Inspect only the first returned table at indices 1–8. Each accessible entry exposes only guarded `outfitID`, `name`, `icon`, `isEventOutfit`, `isDisabled`, `playerFacingOutfitIndex`, and `situationCategories`; the latter exposes only guarded scalar indices 1–8, without iteration or length lookup.

Each accessible finite original outfit ID permits exactly one independent `GetOutfitInfo(ID)` call. Recheck ID access after namespace/function lookup and function guards. Inspect only its first returned entry with the same field bounds, without recursive queries. Never forward serialized/truncated substitutes. Entry failures do not suppress peers; no returned objects are retained.

Bounds: nine API calls per snapshot, ten shared snapshots, 256-byte strings and 128-byte labels. No mutations, outfit selection, name queries, equality/identity, ordering/completeness or native-classification claims. Pinned `TransmogOutfitInfoDocumentation.lua:295–309,367–375,886–897` supplies signatures and seven fields. Eleven actual TOC/slash fixtures prove recorder mechanics only; native behavior remains unverified. Run `luajit docs/addons/ApiContractProbe/tests/outfit_catalog.lua docs/addons/ApiContractProbe`.

## Manual spell diminish categories

`/apicontract spell-diminish-categories <label>` is excluded from `all`. Query `GetAllSpellDiminishCategories` once for each fixed published `Enum.SpellDiminishRuleset` name `None`, `PvE`, `PvP`; independently query `GetSpellDiminishCategoryInfo` once for each published `Enum.SpellDiminishCategory` name `Root`, `Taunt`, `Stun`, `AoEKnockback`, `Incapacitate`, `Disorient`, `Silence`, `Disarm`. Require accessible finite numeric values, without numeric fallback or enum iteration. Recheck inputs after API lookup/function guards.

Preserve raw arity, nils and opaque errors within sixteen return positions. Inspect only the first returned list at positions 1–8, or the first returned category-info object, reading only guarded `category`, `name`, `icon` fields. Every list, entry and field access is guarded; do not traverse, measure or retain returned objects. Eleven independent calls per snapshot, ten shared snapshots, 256-byte strings and 128-byte labels bound capture size. Missing/restricted/failed observations do not suppress peers.

Pinned `SpellDiminishUIDocumentation.lua` and `SpellDiminishConstantsDocumentation.lua` declare calls, fields and enum names. `RequiresSpellDiminishUI` may return nothing; the recorder does not load UI or manufacture fixtures. Never call secret-return `ShouldTrackSpellDiminishCategory` or access secret tracker events. No mutation, native classification, order, completeness, defaults, population or 3D claims. Eleven actual TOC/slash fixtures prove recorder mechanics only; native behavior remains unverified. Run `luajit docs/addons/ApiContractProbe/tests/spell_diminish_categories.lua docs/addons/ApiContractProbe`.

## Manual weekly progress

`/apicontract weekly-progress <label>` is excluded from `all`. Resolve only the five fixed published `Enum.WeeklyRewardChestThresholdType` names `Raid`, `Activities`, `World`, `RankedPvP`, `Concession`; call `C_WeeklyRewards.GetSortedProgressForActivity(value, false)` then independently with `true`. Values must be accessible finite numbers, with no numeric fallback or enum iteration. Recheck inputs after API lookup/function guards.

Preserve raw tuples up to sixteen positions. Only the first returned table is inspected at indices 1–8, reading only `activityTierID`, `difficulty`, `numPoints`; guard each table, entry and field read. No returned-table iteration, length lookup or mutation. Bounds: ten calls per snapshot, ten shared snapshots, 256-byte strings and 128-byte labels. Missing, restricted and failed observations do not suppress peers. No sorting, combine, completeness or native-result claims.

Pinned `WeeklyRewardsDocumentation.lua:180-193,339-346` supplies signature and fields; `Blizzard_WeeklyRewards.lua:35-63` and `Blizzard_WeeklyRewards.xml:499` supply enum-name consumers. Eleven local actual TOC/slash fixtures prove recorder mechanics only. Run `luajit docs/addons/ApiContractProbe/tests/weekly_progress.lua docs/addons/ApiContractProbe`.

## Manual housing preview modes

`/apicontract housing-preview-modes <label>` is excluded from `all`. For each fixed published `Enum.HouseEditorMode` name (`BasicDecor`, `ExpertDecor`, `Customize`, `Cleanup`, `Layout`, `ExteriorCustomization`), independently resolve an accessible finite numeric value and call `C_HousingDecor.IsModeDisabledForPreviewState(value)` twice. No enum iteration or numeric fallback. Guard enum tables, values and functions before lookup/use; recheck the value after function guards. Missing, invalid, restricted and error outcomes remain explicit and do not suppress peers.

Bounds: twelve calls per snapshot, sixteen return positions, 256-byte strings, 128-byte labels and ten shared snapshots. Preserve raw arity/nils and opaque errors without asserting preview conditions, defaults or repeated-read stability. No mode changes, preview mutations or LoD loads. Pinned `HousingDecorUIDocumentation.lua:311-323` declares the call; `Blizzard_HouseEditorModeButtons.lua:117` and XML lines 230/243/256/269/283/298 supply the consumer and six names. Eight local actual TOC/slash fixtures prove recorder mechanics only; native behavior remains unverified.

## Manual unit role predicates

`/apicontract unit-role-predicates <label>` is excluded from `all`. For each fixed token (`player`, `target`, `focus`, `pet`, `party1`, `nonexistent`, `invalid-unit-token`, and the empty string), call `UnitIsLieutenant`, `UnitIsMinion`, and `UnitIsNPCAsPlayer` once independently. The 24 fixed-token calls remain unchanged. Separate `omittedInput.omitted` and `omittedInput.explicitNil` observations call only `UnitIsNPCAsPlayer()` and `UnitIsNPCAsPlayer(nil)` independently, preserving zero versus one argument. Guard tokens and functions before inspection/use and recheck tokens after lookup/function guards; guard explicit nil before forwarding and again after function guards.

Preserve raw arity, nil positions, restricted results and opaque errors: 26 calls per snapshot, sixteen return positions, 256-byte strings, 128-byte labels and ten shared snapshots. No `UnitNameFromGUID`, threat/security queries or mutations. Pinned retail `Blizzard_APIDocumentationGenerated/UnitDocumentation.lua:2028–2088` declares one unit argument and boolean returns without secret-return annotations; it does not establish native results. No classification, default, equality, identity or stability claims.

Sixteen cumulative actual TOC/slash fixtures (ten existing, six new) prove recorder mechanics only:

```text
luajit docs/addons/ApiContractProbe/tests/unit_role_predicates.lua docs/addons/ApiContractProbe
```

## Manual unit target display

`/apicontract unit-target-display <label>` is excluded from `all`. It independently calls only `UnitShouldDisplaySpellTargetName(unit)` twice for each fixed token: `player`, `target`, `focus`, `party1`, `nonexistent`, `invalid-unit-token`, and the empty string. Guard the token and function before use and recheck token access after function guards. Missing/restricted APIs, tokens and opaque call errors do not suppress peer observations.

Preserve raw arity/nils and accessible scalar results without asserting boolean values, defaults or stability. Bounds: fourteen calls per snapshot, sixteen return positions, 256-byte strings, 128-byte labels and ten shared snapshots. Never call `UnitSpellTargetClass`, `UnitSpellTargetName` or additional cast queries; use the separate `casts` mode for manual context. Native targeted-cast transitions and semantics remain unverified. Pinned retail `Blizzard_APIDocumentationGenerated/UnitDocumentation.lua:3007–3068` declares the ordinary predicate and explicitly secret class/name returns.

Eight actual TOC/slash fixtures prove recorder mechanics only:

```text
luajit docs/addons/ApiContractProbe/tests/unit_target_display.lua docs/addons/ApiContractProbe
```

## Manual selected-slot spellbook durations

`/apicontract spellbook-duration <actionslot> <label>` is manual only, excluded from `all`. The original accessible `"spell"` kind and finite spell ID from `GetActionInfo(actionslot)` authorize `C_SpellBook.FindSpellBookSlotForSpell(ID, false, true, true, true)`. This is the `knownSpellsOnly=false` branch used by the pinned `Blizzard_SpellBookFrame.lua` consumer, not a claim about defaults. Preserve producer arity, nils and opaque errors. Only the original accessible finite numeric first pair is forwarded, without interpreting or guessing bank/slot identity.

Independently call `GetSpellBookItemChargeDuration(slot, bank)`, `GetSpellBookItemCooldownDuration(slot, bank, false)` and `GetSpellBookItemLossOfControlCooldownDuration(slot, bank)`. Recheck kind/ID after lookup/function guards before the slot producer, and slot/bank after every duration API lookup/function guard. Missing or restricted inputs remain unavailable observations, not zero durations. Current objects receive the existing ten read-only duration methods, with receiver access rechecked before invocation; no objects are retained and cast-duration state is untouched.

Bounds: sixteen return positions, 256-byte scalar strings, 128-byte labels, ten snapshots, one slot lookup and three duration queries per snapshot; at most 480 method calls per snapshot. Ten actual TOC/slash fixtures prove recorder mechanics only. No native bank, identity, cooldown, default, lifecycle or security semantics are established.

## Manual selected-slot spell durations

`/apicontract spell-duration <slot> <label>` is excluded from `all`. It uses the existing integer-slot parser and `GetActionInfo(slot)` producer. Only an accessible `"spell"` first return and finite numeric second return permit queries with that original ID. Recheck both inputs after each protected namespace lookup and function guard, immediately before calling `C_Spell.GetSpellChargeDuration(ID)` or `C_Spell.GetSpellLossOfControlCooldownDuration(ID)`. Missing, restricted or failed producers are unavailable inputs, not zero durations; one failed API never suppresses its peer.

Preserve raw return arity, nils and opaque errors. Reuse the ten read-only methods listed under [producer cast durations](#producer-cast-durations) through the existing duration inspector; no setters, guessed IDs, spellbook queries, ordinary cooldown queries, casts or mutations. This mode observes **current objects only**: it retains no objects between snapshots and never reads or changes the cast-duration retention list or capture counter. Repeated manual captures do not prove object identity or lifecycle behavior.

Bounds remain sixteen returned positions and sixteen scalar positions per method, 256-byte strings, 128-byte labels and ten shared snapshots. At most two duration producers and 320 method calls occur per snapshot. Pinned `SpellDocumentation.lua:233–247,426–440` declares both producers with one spell identifier and possible zero returns; declarations and local fixtures are not native evidence. Charge/recharge and loss-of-control transitions, native timing and restricted-context semantics remain unverified.

Nine separate actual TOC/slash fixtures, including interleaved cast-duration retention, run with:

```text
luajit docs/addons/ApiContractProbe/tests/spell_duration.lua docs/addons/ApiContractProbe
```

## Manual current unit auras

`/apicontract unit-auras-current <label>` is excluded from `all`. Three independent `C_UnitAuras.GetUnitAuras` calls use exactly `("player")`, `("player", "HELPFUL", 8)`, and `("player", "HARMFUL", 8)`. The `omittedFilterNegative` result is an intentional missing-required-filter negative case, **not** a valid default-filter experiment. Sort arguments are omitted. This API has no continuation argument.

Each result preserves raw arity, nil positions, opaque errors and at most sixteen scalar observations. Only the first returned value, if an accessible table, receives numeric entry observations for indices 1–8. Accessible table/userdata entries receive only `auraInstanceID`, `spellId`, and `applications` field observations. Access is checked before every table/entry lookup and before scalar inspection; lookup failures remain independent. No length/pairs traversal, mutation, coercion or returned-object retention occurs.

The pinned `Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:452–469` establishes the signature and conditional contents. An `AuraData` field declaration was not found in this cache: the fixed keys are grounded in actual consumers, `Blizzard_FrameXMLUtil/AuraUtil.lua:45–54` (`applications`, `spellId`) and `:266–267` (`auraInstanceID`), not a claimed declared schema. Paths are relative to `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/`.

Bounds: three API calls per snapshot, ten snapshots, sixteen return positions, eight entry positions per first table, 256-byte strings and 128-byte labels. Nine local fixtures prove recorder mechanics only. No native ordering, completeness, identity, sorting, defaults, security semantics or historical-contract equivalence is established.

```text
luajit docs/addons/ApiContractProbe/tests/unit_auras_current.lua docs/addons/ApiContractProbe
```

## Manual current-only aura time

`/apicontract aura-time <label>` is excluded from `all`. It reuses the first-page player HELPFUL eight-slot producer described below, without changing `aura-display-count`. Each guarded original aura instance ID receives four independent calls with exactly `("player", id)`: `DoesAuraHaveExpirationTime`, `GetAuraBaseDuration`, `GetRefreshExtendedDuration`, and `GetAuraDuration`. Optional spell IDs are omitted, not supplied as nil.

These are current-only observations. Preserve query status, arity, nil positions and scalar kind/status. For each of the first sixteen `GetAuraDuration` returns, accessible table/userdata observations additionally contain the existing ten [read-only duration methods](#producer-cast-durations). Method lookup checks object access; invocation rechecks receiver access after function guards. Method results preserve arity/nils, opaque errors and bounded scalars independently. No evaluate/set/copy methods, field traversal, object retention or changes to cast-duration state. Missing fixtures/APIs, restricted inputs and errors remain independent observations. Original slots/IDs are checked before inspection and again after namespace/function lookups and access guards.

Bounds: one page, eight data calls, 32 time queries and at most 1,280 method calls (8 IDs × 16 returned positions × 10 methods) per snapshot; 16 tuple positions per producer/method, 256-byte strings, 128-byte labels, ten snapshots. No continuation, mutation, native execution or claims about expiration, refresh, defaults, lifecycle or security semantics.

```text
luajit docs/addons/ApiContractProbe/tests/aura_time.lua docs/addons/ApiContractProbe
```

## Manual first-page aura display counts

`/apicontract aura-display-count <label>` is excluded from `all`. It calls `C_UnitAuras.GetAuraSlots("player", "HELPFUL", 8)` once, preserving the continuation and vararg slot tuple. Continuation is recorded but never followed: this is explicitly first-page coverage, not complete enumeration.

At most eight original accessible finite slots feed `GetAuraDataBySlot("player", slot)`. Only guarded `auraInstanceID` lookup on an accessible table/userdata is permitted; AuraData is otherwise opaque. Each accessible finite original ID receives five independent `GetAuraApplicationDisplayCount` calls: omitted min/max, min 1, min 2, min 2/max 5, min 1/max 1. Namespace/function guards and input rechecks precede forwarding. Missing, restricted, invalid and error observations do not become fabricated IDs or counts.

Results preserve arity/nils and opaque errors, capped at 16 positions, 256-byte strings, 128-byte labels and ten snapshots. Per snapshot: one page call, at most eight data calls and forty count queries. No gameplay mutation, native execution or inference about ordering, completeness, defaults, coercion or formatting semantics.

```text
luajit docs/addons/ApiContractProbe/tests/aura_display_count.lua docs/addons/ApiContractProbe
```

## Manual selected-slot spell metadata

`/apicontract spellbook-metadata <slot> <label>` uses the same integer-slot parser and observed `GetActionInfo(slot)` identity, and is excluded from `all`. An accessible `spell` kind plus finite numeric original ID permits three independent one-argument calls: `C_SpellBook.FindBaseSpellByID`, `FindFlyoutSlotBySpellID`, and `FindSpellOverrideByID`. Namespace/function guards precede calls; original kind/ID access is rechecked after lookup and function guards. Raw arity, nils and opaque errors remain observations, not inferred spell identity or slot/bank semantics. Limits: sixteen result positions, 256-byte strings, 128-byte labels, ten snapshots and thirty queries. No duration or spellbook-slot producer, mutation, secret-specific API or retained objects. Eight separate `tests/spellbook_metadata.lua` fixtures prove recorder mechanics only; native behavior remains unverified.

`/apicontract spell-metadata <slot> <label>` reuses the integer-slot parser and stays outside `all`. Call `GetActionInfo(slot)` with exactly one argument and preserve producer arity. Only an accessible first return exactly equal to `"spell"` and an accessible finite numeric second return permit downstream queries. Use that original ID without conversion, guessed IDs or classification.

Independently query `C_Spell.GetSpellDisplayCount`, `GetSpellMaxCumulativeAuraApplications`, `IsConsumableSpell`, `IsExternalDefensive`, `IsPriorityAura`, `IsSpellCrowdControl` and `IsSpellImportant`, each with only the ID. Display-count optional arguments remain omitted. Guard each namespace/function lookup and recheck producer-value access after lookup and function checks, immediately before each call. Errors never suppress peer queries. No actions or casts execute.

Then query `GetVisibilityInfo(ID, value)` once for each fixed `Enum.SpellAuraVisibilityType` name: `RaidInCombat`, `RaidOutOfCombat`, `EnemyTarget`. Use only accessible published finite numbers, never numeric fallbacks or arbitrary enum iteration. Missing, restricted or invalid enum members record `unavailable-enum` without a call; the seven base queries remain independent. Recheck the original spell inputs and enum value after lookup/function guards before every call. Preserve zero returns, nil positions, guarded scalar results and opaque errors without inferring combat/spec state.

Separately record `auraQueries.AuraIsBigDefensive` from `C_UnitAuras.AuraIsBigDefensive(originalID)`. Pinned `UnitAuraDocumentation.lua` declares one `SpellIdentifier` argument and one boolean return; no aura instance is required. Guard namespace, field, function and original kind/ID access before inspection, then recheck inputs after lookup/function guards. Its absence or failure does not suppress spell queries, nor do spell-query failures suppress this observation.

Retain exact arity/nils, sixteen scalar positions, 256-byte strings, 128-byte labels and opaque errors under the ten-snapshot cap: at most ten producer, seventy base metadata, thirty visibility and ten aura-defensive calls. Pinned `SpellDocumentation.lua` and `SpellConstantsDocumentation.lua` supply signatures and enum names; `AuraUtil.GetCachedVisibilityInfo` consumes the raid visibility values. These are not classifications or native outputs. Native spell fixtures, rank/override transitions and restricted-context behavior remain unverified. Nineteen cumulative actual TOC/slash fixtures (five added aura-defensive cases) test recorder mechanics only, with no native-conformance credit:

```text
luajit docs/addons/ApiContractProbe/tests/spell_metadata.lua docs/addons/ApiContractProbe
```

## Manual outfit state

`/apicontract outfit-state <label>` is excluded from `all`. It independently calls seven `C_TransmogOutfitInfo` functions twice each, with no arguments: `GetCurrentlyViewedOutfitID`, `GetMaxNumberOfUsableOutfits`, `GetNextOutfitCost`, `GetPendingTransmogCost`, `HasPendingOutfitSituations`, `IsEquippedGearOutfitDisplayed`, and `IsEquippedGearOutfitLocked`.

Each `outfitState` function-name key contains two guarded raw observations. Zero returns, explicit nil positions, opaque errors, and the pending-cost two-result tuple remain distinct. No state changes, purchases, selection, price interpretation, defaults, stability or native behavior are inferred. Bounds: 14 calls per snapshot, 10 snapshots, 16 return positions, 256-byte strings and 128-byte labels.

Pinned `TransmogOutfitInfoDocumentation.lua:175–182,262–278,377–385,556–563,583–599` establishes these call shapes only. Ten separate actual TOC/slash fixtures test recorder mechanics:

```text
luajit docs/addons/ApiContractProbe/tests/outfit_state.lua docs/addons/ApiContractProbe
```

## Manual public queries

`/apicontract public-queries <label>` stays outside `all`. Independently call `C_GameRules.IsPersonalResourceDisplayEnabled()` twice and `C_DelvesUI.GetLockedTextForCompanion()` twice with exactly zero arguments. The latter records only the omitted-companion case; no companion or trait-tree IDs are invented and `IsTraitTreeForCompanion` is not called.

Also independently call `C_Housing.IsHousingMarketShopEnabled()`, `C_EncounterTimeline.GetCurrentTime()`, and `C_InstanceEncounter.IsEncounterLimitingResurrections()`, `IsEncounterSuppressingRelease()`, and `ShouldShowTimelineForEncounter()` twice each. Their separate output keys are `housingMarketShopEnabled`, `encounterTimelineCurrentTime`, `encounterLimitingResurrections`, `encounterSuppressingRelease`, and `showTimelineForEncounter`.

Three further independent pairs call `C_TransmogOutfitInfo.GetActiveOutfitID()`, `C_HousingCustomizeMode.IsHouseExteriorDoorHovered()`, and `C_SpellDiminish.IsSystemSupported()`. Their named outputs are `activeOutfitID`, `houseExteriorDoorHovered`, and `spellDiminishSystemSupported`.

Protected namespace lookup and access guards precede inspection. Missing APIs, lookup failures and opaque call errors do not prevent the other observations. Preserve raw repeated results, exact arity/nils, sixteen scalar positions, 256-byte strings and 128-byte labels under the shared ten-snapshot cap (ten APIs, twenty query calls per snapshot; 200 maximum). Objects remain opaque. No mutations, CVar changes, state transitions, default or stability claims are made.

Pinned `GameRulesDocumentation.lua:231–238` and `DelvesUIDocumentation.lua:229–245` supply the argument shapes, not native outputs. Native ruleset/state transitions, companion lock policy, trait-tree fixtures and restricted-context behavior remain pending. Pinned `TransmogOutfitInfoDocumentation.lua:121–128`, `HousingCustomizeModeUIDocumentation.lua:257–265`, and `SpellDiminishUIDocumentation.lua:43–50` declare the additional zero-argument number/bool/bool returns, not native outfit, hover or support behavior. Seventeen cumulative actual TOC/slash fixtures (twelve existing, five new) prove recorder mechanics only:

```text
luajit docs/addons/ApiContractProbe/tests/public_queries.lua docs/addons/ApiContractProbe
```

## Manual equipped-item binding

`/apicontract item-binding <label>` stays outside `all`. For fixed equipment slots 1–19, call `GetInventoryItemLink("player", slot)` once and retain its exact return arity. Only the first returned value, if an accessible non-secret string, is passed once to `C_Item.IsItemBindToAccount(link)`. The original link is passed, never its truncated saved representation. No synthetic links, numeric item IDs, alternate producers or mutations are used.

Producer and binding results retain sixteen scalar positions, nils and opaque errors; saved strings stop at 256 bytes. Missing, nil, nonstring or failed producers yield `unavailable-input`, not false. Restricted links yield `restricted-input`. Namespace lookup and function access are guarded; link access is rechecked immediately before passing it onward. Each slot proceeds independently after earlier errors. Shared ten-snapshot limit bounds producer calls to 190 and binding calls to at most 190 per load.

Pinned `ItemDocumentation.lua:1362–1374` declares the ItemInfo argument and boolean return. `Blizzard_EncounterJournal/Mainline/Blizzard_Journeys.lua:161–174` passes an actual item link to this API; it does not establish binding classifications for equipped fixtures. Native results, known/unknown binding fixtures and security behavior remain unverified. Eight separate actual TOC/slash fixtures prove recorder mechanics only:

```text
luajit docs/addons/ApiContractProbe/tests/item_binding.lua docs/addons/ApiContractProbe
```

## Manual StatusBar fill style

`/apicontract statusbar-fill <label>` stays outside `all`. It creates one unnamed `CreateFrame("StatusBar", nil, UIParent)` and immediately attempts `Hide`, before recording the fresh `GetFillStyle()` default. For each published `Enum.StatusBarFillStyle` name `Standard`, `StandardNoRangeFill`, `Center`, `Reverse`, an accessible finite scalar value permits one `SetFillStyle(value)` followed by two independent `GetFillStyle()` observations. No enum numbers are substituted. Missing or throwing setters do not suppress those getters. Missing, restricted or nonscalar enum entries are recorded but not passed to setters.

Object, method and result access is guarded before inspection. Exact arity/nil slots and up to sixteen scalar positions are retained; objects and errors remain opaque. Strings retain at most 256 bytes; ten snapshots cap frame creation. Constructor or Hide failure aborts fill operations and retains no frame reference. A failed Hide is `visibility-unconfirmed`: the addon cannot guarantee disposal or invisibility of a client-owned frame when hiding fails. No Show, sizing, layout, rendering, destruction or other setters are used.

Pinned `SimpleStatusBarAPIDocumentation.lua` declares the getter and enum setter, not native defaults or successful roundtrips. Addon execution is tainted; this recorder assumes no untainted setup and performs no restricted-context or invalid-input experiments. Native validation, coercion and default behavior remain pending. Seven separate local actual TOC/slash fixtures are recorder proof only:

```text
luajit docs/addons/ApiContractProbe/tests/statusbar_fill.lua docs/addons/ApiContractProbe
```

## Manual raid-marker observations

`/apicontract raid-markers <label>` stays outside `all`. It calls `CanBeRaidTarget(unit)` twice for each of `player`, `target`, `focus`, `pet`, `party1`, `party2`, `nonexistent`, `invalid-unit-token` and the empty token; `IsRaidMarkerActive(index)` twice for each index 1–8; and `IsRaidMarkerSystemEnabled()` twice with no arguments. All 36 calls are independent: missing APIs never gate other observations. Repeated results remain raw, not stability comparisons.

Pinned `RaidMarkersDocumentation.lua` supplies these argument shapes, not native results. `GetRaidTargetIndex` is excluded because it declares secret returns. No setter, clear, place or remove API runs; no permission or security conclusions are recorded. Capture naturally available states with manual labels; native output and populated world-marker fixtures remain pending.

Accessibility precedes inspection on every call. Exact arity, nil slots and up to sixteen scalar positions are retained; strings stop at 256 bytes, with truncation explicit. Objects and errors remain opaque. Shared ten-snapshot limit applies. Six separate local TOC/slash fixtures test raw repeated results, independent failures, redaction, arity and bounds—not native conformance:

```text
luajit docs/addons/ApiContractProbe/tests/raid_markers.lua docs/addons/ApiContractProbe
```

## Manual color curve state

`/apicontract color-curves <label>` is excluded from `all`. Own `CreateColorCurve` and `CreateColor` inputs, in insertion order, are `(x; r,g,b,a)`: `(0; 0,1,.25,1)`, `(32; 1,0,.25,.75)`, `(-16; 0,.25,1,.5)`, `(48; 1,.75,0,.25)`. No enum type is assigned or guessed.

Empty and populated snapshots record type/count/secret flag/points, indices `-1,0,1,2,3,4`, and both evaluations at `-17,-16,0,16,32,48,49`. Copy observations surround adding `(16; .5,.25,.75,.5)` to the copy and clearing it; both original and copy are captured after each operation. The original is then captured before/after `SetToDefaults`, even if copying aliased it. Results are observations, not assertions of defaults, order, interpolation or isolation.

Returned tables expose only raw `x/y/r/g/b/a` fields and four array entries, with three levels of table traversal; userdata stays opaque. No returned `GetRGBA`, `__index`, equality or string methods run. Guarded methods on owned curves are allowed. Exact arity and 16 positions are recorded; larger tuples are marked truncated. Ten snapshots maximum; errors stay opaque and missing constructors fail closed.

Cached color-curve/CurveUtil/base documentation and `Blizzard_SharedXMLBase/Color.lua` guide these calls, not native expectations. Userdata field representation, native captures, security semantics, removal/replacement and alternate interpolation types remain gaps. Local fixture command:

```text
luajit docs/addons/ApiContractProbe/tests/color_curves.lua docs/addons/ApiContractProbe
```

## Manual scalar resource scale observations

`/apicontract resources <label>` captures player, target, focus, pet and nonexistent units; it is excluded from `all`. Records raw health/max, power/max/type, omitted-curve and explicit-nil percent calls. Health curve argument 3 follows explicit `usePredicted=false`; the default call is separate. Power uses default power type (`nil`), with separate explicit false/true `unmodified` calls and curve argument 4. No observed power ID is reused or guessed.

One scalar curve records inputs `(0,0), (.5,10), (1,20), (25,30), (50,40), (100,50)`. Construction explicitly selects accessible `Enum.LuaCurveType.Linear`; missing enum/factory/SetType fails closed for curved queries while underlying observations remain available. Results use scalar-only recording, exact arity and the existing 16-position bound, never scale classification or returned-object methods.

Capture naturally available partial-resource states with labels and matching client build. Prediction, secondary powers, color curves, secret contracts and actual native scale remain pending. No events or gameplay changes are triggered. Ten-snapshot limit applies. Local fixtures distinguish fake normalized/percentage results; they are not native evidence.

Manual native observations for scalar curve points, `UnitSexBase`, unit names/realms, finite numeric formatting, publication/CVars and event payloads. Complements [AuraDispelCurveProbe](../AuraDispelCurveProbe/README.md). Neither recorder infers native contracts from fixture outputs.

## Scalar curve state

`/apicontract curve-state <label>` is manual-only, excluded from `all`; existing `curves` is unchanged. It creates empty and unsorted duplicate-point scalar curves using `(30,4), (10,7), (20,2), (10,9)`, attempts the discovered `Enum.LuaCurveType.Linear`, and records type/count/points/secret-flag plus evaluation at `-1,0,10,15,20,30,31` before and after reset. Setter failures and missing enum values are explicit observations.

A separate populated curve is copied; both original and copy are observed after adding `(40,11)` and after clearing the original. No expected defaults, duplicate policy, interpolation or copy isolation is assumed. Method results are scalar-only except existing bounded point/GetXY inspection; errors remain opaque. Four returned points and eight result positions are retained. Ten snapshots maximum. Color curves and secret semantics remain pending; `SetPoints` and `RemovePoint` have the separate manual experiment below. No native execution is claimed.

## Scalar point removal and replacement

`/apicontract curve-edit <label>` is manual-only and excluded from `all`. Each of six `RemovePoint` inputs (`-1,0,1,2,3,4`) gets a fresh scalar curve populated with `(30,4), (10,7), (20,2)`. Two more fresh curves receive `SetPoints`: an empty array, and the unsorted duplicate sequence `(30,4), (10,7), (20,2), (10,9)`. Every replacement point comes from the actual `CreateVector2D(x,y)` constructor; missing, throwing, inaccessible or invalid constructor results prevent that replacement. No plain-table coercion is assumed.

Each case records definitions, `GetPointCount`, bounded `GetPoints` and `Evaluate(15)` before and after the attempt, plus mutation return arity and opaque errors. Existing bounds retain four points, eight return positions and ten snapshots. Numeric out-of-range controls are observations, not expected rejection rules; default curve interpolation is not inferred. Curves and replacement vectors are independently constructed for every case; no returned-point alias or copy semantics are assumed.

Pinned `LuaCurveObjectAPIDocumentation.lua` declares `RemovePoint(index: luaIndex)` and `SetPoints(point: table<vector2>)`; `Blizzard_SharedXML/Vector2D.lua` implements `CreateVector2D` with `x`, `y` and `GetXY`. These sources guide construction, not native indexing, duplicate/order or mutation results. Local fixtures retain distinct zero-/one-based removal and replacement ordering. Native execution, invalid-type coercion, color curves and security semantics remain pending.

## Target

Interface `120100` follows the locally pinned retail `12.1.0.69497` source, not a newly observed desktop build. Install only on a matching client. No installation or native execution has been performed by this task.

## Capture

1. Copy this directory to the matching client's `Interface/AddOns/ApiContractProbe`, then enable it.
2. Run `/apicontract all normal` to capture curves, sex, names, numbers, casts and publication. Use `/apicontract curves`, `/apicontract sex <label>` or `/apicontract names <label>` to capture separately.
3. For sex comparisons, select real targets or repeat during an independently obtained transformation, labeling the scenario. The addon does not alter units or gameplay.
4. Run `/reload` or log out to flush SavedVariables.
5. Retain `WTF/Account/<ACCOUNT>/SavedVariables/ApiContractProbe.lua` with scenario notes. Keep the raw artifact unchanged.

Ten snapshots maximum; further calls increment `dropped`. No automatic captures. `all` includes publication but does not start event listeners.

Additional commands:

- `/apicontract publication after-login` records configured symbol and CVar observations. Repeat with labeled phases after manually loading relevant addons; no addon is loaded by this recorder.
- `/apicontract events-start before-cast` registers the configured events and records real subsequent deliveries.
- `/apicontract events-stop` unregisters listeners. Event recording is capped at 256 entries; excess deliveries increment `droppedEvents`.

`AuditTargets.lua` contains 199 publication/CVar targets and 88 event-registration targets tied to the blocker snapshot hash. Registration errors, missing parents and missing members are distinct observations. Event payloads retain arity and nil slots, with at most eight summarized values; tables remain bounded observations, not full structure captures. No native actions, purchases, transitions or account mutations are triggered. A symbol missing at one phase does not prove removal; a listener with no deliveries does not prove event absence. These shared recorders do **not** complete all associated behavioral plans. See the [full preparation inventory](../../baselines/native-probe-preparation.json). Start a new capture batch by moving the saved file aside while logged out.

## Manual abbreviations

`/apicontract abbreviations <label>` is excluded from `all`. It reuses the same 25 finite inputs as `numbers`, calling `AbbreviateLargeNumbers(number)` and `AbbreviateNumbers(number)` with options omitted, plus one `GetLocale()` observation. No locale changes or formatting assumptions are made.

Scalar-only capture preserves exact return arity, nil positions and accessible string bytes, with sixteen retained positions and 256 bytes per string; larger results are marked truncated. Returned objects and errors stay opaque. Access checks precede inspection; missing functions or failed guards fail closed. Each snapshot allows at most fifty abbreviation calls and one locale call; the shared ten-snapshot cap applies.

Pinned retail `LocalizationDocumentation.lua` declares each number argument and optional `NumberAbbrevOptions`. This slice does not create options/configuration tables or formatters. Native output, breakpoint/options/configuration behavior and table producers remain separate gaps.

Separate local fixtures (not native evidence):

```text
luajit docs/addons/ApiContractProbe/tests/abbreviations.lua docs/addons/ApiContractProbe
```

## Manual heal calculator

`/apicontract heal-calculator <label>` is manual-only, excluded from `all`. Each snapshot attempts two fresh `CreateUnitHealPredictionCalculator()` calls with no arguments. On each accessible object, query `GetHealAbsorbMode`, `GetHealAbsorbClampMode`, `GetDamageAbsorbClampMode`, `GetHealAbsorbs`, and `GetDamageAbsorbs` twice, in that order. Repeated observations remain separate; no equality or stability inference is performed.

Record constructor and getter exact arity, nil positions and at most sixteen scalar/opaque positions, marking truncation. Object access precedes method lookup; function and result access precede inspection. Returned objects and errors remain opaque. Missing, restricted or invalid constructors prevent getter calls. At most two constructor and twenty getter calls per snapshot; the shared ten-snapshot limit applies.

Pinned retail `UnitDocumentation.lua` declares the no-argument constructor; `UnitHealPredictionCalculatorAPIDocumentation.lua` declares three mode getters and two `(amount, clamped)` getters. These signatures guide calls, not expected values. No unit variants, population, setters, reset/default transitions, security experiments or native execution. Fresh default values and populated behavior remain unverified.

Separate local behavioral fixtures:

```text
luajit docs/addons/ApiContractProbe/tests/heal_calculator.lua docs/addons/ApiContractProbe
```

## Producer cast durations

`/apicontract cast-durations <label>` is manual-only, excluded from `all`. Queries `UnitCastingDuration`, `UnitChannelDuration`, and `UnitEmpoweredChannelDuration` with hold omitted/false/true for player, target, focus, party1, nonexistent, invalid-unit-token and empty tokens. Record exact arity, nils and at most 16 positions; larger tuples are marked truncated.

For accessible returned duration objects, this controlled experiment calls only `GetTotalDuration`, `GetElapsedDuration`, `GetRemainingDuration`, `GetElapsedPercent`, `GetRemainingPercent`, `GetStartTime`, `GetEndTime`, `GetClockTime`, `GetModRate`, and `HasExpired`. Object and method access are checked before lookup/call; method results stay scalar/opaque, errors are not stringified. No setters, Assign, Reset, object equality or native identity inference.

Capture `active`, then a later `completed` or `interrupted` observation using actual externally produced states. Each capture re-queries up to 28 objects from the previous duration capture and labels them with that capture's `observationRef`; these are observation labels, not native object IDs. Only the new snapshot's first 28 accessible object occurrences are retained, with excess marked `retention = "limit"` and counted. Seven units × five calls can exceed this limit. Runtime references never enter SavedVariables, and reload clears retention. Ten total snapshots per batch; no timers or synthetic casts. Native tracking versus frozen behavior remains unknown until these observations are collected on the matching client.

## Plain-function callbacks

Run `/apicontract callbacks-start <label>`, produce a real `UNIT_HEALTH` transition externally, then `/apicontract callbacks-stop`. This manual mode is excluded from `all`; it never fires events or changes gameplay. It registers only its own global callback and its own `player` unit callback, preserving their identities for removal.

Each of at most ten sessions records build/time/label, separate registration/removal results and up to 128 deliveries shared across the two callbacks. Payloads retain exact arity and up to 16 scalar-only positions (including nils); objects remain opaque, inaccessible values are redacted, strings retain the existing 256-byte bound. Excess deliveries increment `dropped`; stop still attempts cleanup. Callbacks themselves never enter SavedVariables.

`registration-incomplete` preserves independent failures/refusals. A throwing registration may already have installed its callback, so stop still attempts removal. Missing, throwing, refused or inaccessible cleanup results retain the identity for another stop attempt and report `cleanup-incomplete`, not success. A successful stop disables recording even if an old callback is invoked later. Repeated start while a session needs cleanup is rejected; stop then start creates one new session. Missing access APIs prevent registration entirely. A nonthrowing zero-return call is recorded as accepted; this is not independent evidence that native registration took effect. Inspect real deliveries and raw results.

### Duplicate registration inputs

`/apicontract callbacks-duplicate-start <label>` uses the same single-session limit and the same two owned lanes, registering each lane's exact closure twice (four calls). `callbacks-stop` disables receiving before attempting at most two removals per duplicate lane. Global and unit cleanup proceed independently. Normal `callbacks-start` retains its singular registration/removal records and behavior.

Duplicate lanes store `registrations[1..2]` separately. Each available registration call reserves a cleanup slot before invocation, including calls that throw or return false. Each pending slot gets one removal attempt per explicit stop; `removals[slot]` holds its latest raw result and `removalAttempts[slot]` its attempt count. These fixed-size records replace prior removal results on an explicit retry rather than accumulating an unbounded history. Accepted cleanup retires only that slot. False, errors, inaccessible or truncated outcomes retain the exact closure and report `cleanup-incomplete`; another start remains blocked. No automatic retries occur. An API that deduplicates registrations but returns false on the second removal can therefore leave cleanup unconfirmed; the recorder never guesses deduplication or reference-count semantics.

Event, unit and callback accessibility are checked before forwarding and again after function guards in duplicate mode. Existing ten-session, one-active-session, 128-delivery, 16-position and 128-byte label limits apply. Local fixture deliveries are driven only by tests, never by the recorder. Run the focused suite with `luajit docs/addons/ApiContractProbe/tests/callback_duplicates.lua docs/addons/ApiContractProbe`.

Native execution remains pending. The duplicate mode prepares same-closure registration/removal inputs only; it establishes no native duplicate, delivery or ordering semantics. FunctionContainer wrappers, aliases, callback-time mutation, security and GC claims remain outside this experiment. This is partial preparation, not native conformance proof.

## Observations

**Curves:** creates an empty curve and another with insertion order `(30,4), (10,7), (20,2)`. Records `GetPoints`, `GetPoint` at `0,1,2,3,-1,4`, return arity, accessible `x`/`y` and `GetXY`, and repeated-point equality. On a separate curve, attempts `GetPoint(1).x = 91` and records another lookup. Index 1 may be invalid; rejection is evidence, not failure of the experiment. Array inspection is bounded to four entries, return inspection to eight values, and object nesting to two levels.

**Names:** `/apicontract names same-realm-party1-cross-realm-party2` records `UnitName` (`name`) and `UnitNameUnmodified` (`unmodified`) under `names.units` for `player`, `party1`–`party4`, `target`, `nonexistent`, `invalid-unit-token` and the empty string. Every token is queried without an existence check. Exact return counts and positional nils distinguish zero returns, nil realms, empty realms and explicit realm text; results are not normalized. Establish actual same/cross-realm membership independently, then label each manual snapshot with the relevant slots and scenario. Unknown/invalid token spellings are fixed test inputs, not assumptions about native results. No party, target or gameplay changes are made. Name/realm captures contain character identities; retain them privately.

**Numbers:** `/apicontract numbers <label>` records `GetLocale` under `numbers.locale` and 25 ordered `numbers.samples` with `input`, `floor` (`C_StringUtil.FloorToNearestString`) and `round` (`RoundToNearestString`). Corpus: `-2.5,-1.5,-0.5,0,0.5,1.5,2.5`; `-0.500001,-0.499999,0.499999,0.500001`; `-1,1,-100,100`; `-0.1,0.1,-0.9,0.9`; `-1234.5678,1234.5678,-1e6,1e6,-1e12,1e12`. Retains accessible result bytes and exact arity using the existing bounded observer, including nil slots and zero returns; never derives expected output from helper names. Repeat manually on each matching build/locale, retaining labels and build provenance. Missing APIs/access checks fail closed and errors remain opaque. Native output corpora remain pending; nonfinite inputs, coercion, security behavior and values outside this corpus are not probed.

**Casts:** `/apicontract casts <label>` records `UnitCastingInfo` (`casting`) and `UnitChannelInfo` (`channel`) under `casts.units` for exactly `player`, `target`, `focus`, `party1`, `nonexistent`, `invalid-unit-token` and the empty string. No existence gating, casting or gameplay actions occur. A dedicated accessibility-first scalar-only observer retains exact arity and up to 16 positions, including interior/trailing nils; larger tuples set `truncated = true`. Accessible objects retain only status/kind, never fields, methods, equality or string conversion. IDs are recorded without coercion or assumptions that castID and castBarID match. Existing string bounds apply.

The pinned declarations in `data/patch-api/sources/12.0.0-register.json` list ten casting results (castID at 7, castBarID at 10) and eleven channel results (empowered flag at 9, stage count at 10, castBarID at 11). These declarations guide observation, **not native proof**; the current client may differ. Matching native builds, controllable player casts, non-player channels and empowered/non-empowered fixtures remain pending.

Use separately labeled manual snapshots for idle, active, repeated same cast, consecutive/replaced casts, completed and cancelled states. Capture channels and empowerment scenarios in separate batches: the shared ten-snapshot cap also counts `all` and other modes. Retain each flushed artifact with scenario notes, then move the saved file aside while logged out to begin another batch. Do not interpret timing or identity stability from unrelated captures or fixture values.

**Sex:** independently records `UnitExists`, `UnitSex` and `UnitSexBase` for player, target, focus, pet, party1, party2, nonexistent, invalid-unit-token and the empty token, plus named `Enum.UnitSex` values. Absent, restricted or failed existence queries never suppress either sex query. Each unit now stores separate `exists`, `legacy` and `base` observations instead of an aggregate `status`; their individual statuses and arities preserve errors, nils and redaction. No numbering conversion or output comparisons. Native before/during/after-disguise captures with independently established stable unit identity remain pending; the recorder generates no fixtures.

Restricted values are redacted before comparison or serialization. API errors are opaque status labels, not stringified error objects. Missing access APIs fail closed. Rejected addon-tainted calls are inconclusive; do not bypass restrictions. String observations are truncated to 256 bytes with `truncated = true` when shortened.

## Residual hyperlink observations

Run `/apicontract hyperlinks <label>` explicitly; `all` excludes this mode. Each snapshot makes 144 calls: 16 literal strings × nine variants. Inputs cover ASCII, `é漢字🙂`, empty text, literal `|n`, an actual newline, balanced item/color/atlas/texture markup and their combination, escaped `||`, unclosed item/color/atlas/texture markup and orphan `|h`.

Variants are omitted optional arguments; five false flags; each single true flag in order `maintainColor`, `maintainBrackets`, `stripNewlines`, `maintainAtlases`, `maintainTextures`; consumer `(false,true,false,true,true)`; and all true. Each row saves exact `input`, `variant`, `flags`, `argumentCount` and raw `result`. Omission remains one argument, not six false arguments. No expected stripping or malformed-text normalization is applied.

Existing observer bounds retain exact return arity and at most eight positions. Strings retain up to 256 bytes and set `truncated = true` when shortened; this may split UTF-8. Missing APIs/access checks fail closed, secret values are redacted and errors remain opaque. Ten captures maximum; rejected captures make no calls. Native execution remains pending; nonboolean truthiness/coercion, arbitrary-byte behavior outside the corpus, string-view lifetime and security semantics remain unverified. Local fixtures prove recording only; see [StripHyperlinks contract](../../specs/strip-hyperlinks.md).

## Pure mapvalues observations

Run `/apicontract mapvalues <label>` manually; `all` excludes it. The pinned retail `Blizzard_APIDocumentationGenerated/FrameScriptDocumentation.lua` declares `mapvalues(func, ...)`, not a table argument. Actual `TooltipDataHandler.lua` forwards `mapvalues(SanitizeTooltipDataArgument, ...)`; `Blizzard_AuraContainerUtil.lua` applies validation callbacks through it and discards the return. These sources guide the experiment, not native packing or traversal conclusions.

Each snapshot runs nine cases: zero inputs; `(17)`; `(17,"two",false)`; `(17,nil,"tail")`; `(17,nil,nil)`; then `(17,"two")` with multiple, nil and zero callback returns; finally `(17)` with an opaque thrown object. The first five callbacks return literal `"mapped"`; multiple returns are `"first",nil,"third"`. Callback results never depend on observed arguments. No callbacks recurse into `mapvalues`.

Each case stores input arity/positions, callback mode, ordered invocation observations and outer output. Exact arity is retained with at most 16 scalar positions per tuple; objects remain opaque and inaccessible values are redacted before inspection. Existing 256-byte string and ten-snapshot limits apply. Errors retain only `call-error`, never the thrown object. There is one shared budget of 32 recorded callback invocations per snapshot. An excess invocation throws an opaque object and sets `invocation-limit`; remaining cases are skipped even if the mapper catches that error. A callback invoked after its case returns is rejected without recording. This bounds recorder work/storage, not an arbitrary broken mapper that loops while swallowing errors.

Native order, nil handling, multi-return packing, callback failure propagation and security remain unverified. Contrasting fake mappers test raw recording, not native expectations. Local fixture:

```text
luajit docs/addons/ApiContractProbe/tests/mapvalues.lua docs/addons/ApiContractProbe
```

## Manual selected-slot action loss-of-control duration

`/apicontract action-loss-control-duration <slot> <label>` reuses the signed-integer parser and is excluded from `all`. It independently records `GetActionInfo(slot)` as control and `C_ActionBar.GetActionLossOfControlCooldownDuration(slot)` with exactly one original parsed slot argument. Item/macro identities, missing controls and control errors do not gate the duration query. Guard the slot before use and recheck after namespace/function lookup and access guards.

`actionLossControlDuration` contains guarded `slot`, raw `identity`, and `duration`. Preserve exact arity, nil positions and opaque errors. Accessible duration objects use the shared ten-method read-only inspector with receiver checks before lookup and after function guards. Objects are current-only, never retained; cast-duration retained objects and capture counters remain untouched. Bounds: two selected-slot API calls and at most 160 duration-method calls per snapshot, ten snapshots, sixteen return positions, 256-byte scalar strings and 128-byte labels.

Pinned retail `ActionBarFrameDocumentation.lua:210–224` declares one `actionID:luaIndex`, `RequiresValidActionSlot`, `SecretArguments=AllowedWhenUntainted` and a `LuaDurationObject` return. This does not establish native slot validation, timing or loss-of-control behavior. Historical `GetActionLossOfControlCooldown` and `ActionBarCooldownInfo` fields remain missing. No adjacent cooldown APIs, button registration, action execution or mutations are added. Ten actual TOC/slash fixtures prove recorder mechanics only:

```sh
luajit docs/addons/ApiContractProbe/tests/action_loss_control_duration.lua docs/addons/ApiContractProbe
```

## Manual selected-slot action state

`/apicontract action-state <slot> <label>` reuses the signed-integer slot parser and is excluded from `all`. `GetActionInfo(slot)` is an observed control, not an authorization gate: item, macro, missing, and error results do not suppress independent queries. The original parsed slot feeds exactly one argument to `C_ActionBar.GetActionAutocast`, `GetActionText`, `GetActionUseCount`, `HasRangeRequirements`, `IsAttackAction`, `IsAutoRepeatAction`, `IsConsumableAction`, `IsEquippedAction`, `IsItemAction`, `IsStackableAction`, `IsUsableAction`, and `IsActionInRange` (target omitted). `GetExtraBarIndex()` and `GetMultiCastBarIndex()` are independent no-argument namespace calls.

`actionState` stores guarded `slot`, raw `identity`, named `queries`, and named `bars`. Recheck slot accessibility after namespace/function lookup and guards before forwarding. Restricted results remain opaque before inspection; preserve zero returns, nil positions and opaque errors. Bounds: 15 API calls per snapshot, ten snapshots, sixteen return positions, 256-byte scalar strings, 128-byte labels. No loss-of-control queries, button registration, action execution, mutation, native classification or range/default claims.

Pinned retail `ActionBarFrameDocumentation.lua` establishes these call shapes, including two returns for autocast/usability and the optional range target, not native outputs. Ten actual TOC/slash fixtures prove recorder mechanics only:

```sh
luajit docs/addons/ApiContractProbe/tests/action_state.lua docs/addons/ApiContractProbe
```

## Selected action slots

Run `/apicontract actions 7 before-use` with a slot you independently identify, then repeat with labels after manual use, during recharge and after restoration. No slot roles are inferred and no action is executed. Use separate labeled batches for ordinary spells, charged spells, consumables and empty slots; `0` and `-1` are explicit input controls. Syntax accepts signed decimal integers within ±9007199254740991 only, rejecting malformed input before queries. This does not establish native slot validation. `all` excludes this selected-slot mode.

Each snapshot records optional `GetActionInfo`, default display count and thresholds `0`, `1`, `9999` with replacement `*`, plus `GetActionCharges` arity and five raw table fields: `currentCharges`, `maxCharges`, `cooldownStartTime`, `cooldownDuration`, `chargeModRate`. Display counts are not assumed to equal charges. `GetActionChargeDuration` records only arity and accessible kind/status: no methods or value inspection. No `GetActionCooldown` call is made. Charge fields use `rawget`, never metatable lookup; restricted values and errors remain opaque.

At most seven selected-slot calls per snapshot, 16 saved return positions per call (exact arity and explicit truncation), and ten captures per batch. Build/time and bounded labels are retained. Native slot fixtures/transitions and duration-method contracts remain pending; recorder fixtures do not establish native parity or full preparation of this plan.

## Local fixture

```text
luajit docs/addons/ApiContractProbe/tests/harness.lua docs/addons/ApiContractProbe
```

Cast fixtures exercise distinct tenth/eleventh IDs, empowerment, nil/zero arity, repeated/consecutive captures, 16-position truncation, hostile objects and inaccessible/error returns without asserting native semantics.

Numeric fixtures deliberately return differing locale bytes, non-rounding strings, multiple/nil/zero returns and restricted/error results; these prove literal recording, not native rounding. Fixtures test the recorder under different fake client behaviors, including differing modified/unmodified names, realm forms, nil arity, unknown/invalid tokens and restricted/error returns. They do not establish native indexing, ordering, copy/identity, sex numbering, transformation or security semantics. See [spec](../../specs/api-contract-probe.md).

## Manual death recap current observations

`/apicontract death-recap-current <label>` is excluded from `all`. It independently calls `C_DeathRecap.GetRecapEvents()`, `GetRecapLink()`, and `HasRecapEvents()` twice each with **zero arguments**, not an explicit nil or invented recap ID. Each lookup uses the guarded namespace query helper. Missing, restricted and error outcomes do not suppress peer observations.

Pinned `DeathRecapDocumentation.lua` declares nilable recap IDs and an empty `DeathRecapEventInfo` structure. `GameDialogDefs.lua:203` calls `C_DeathRecap.HasRecapEvents()` without an ID. These sources ground call shapes only; the mode name does not establish native current/default/stability semantics.

Return arity and nil positions are preserved, with at most 16 recorded positions, 256 bytes per scalar string, 128 bytes per label, ten snapshots and six calls per snapshot. Tables and userdata stay opaque and are not retained. No event-field traversal, identity comparison, link opening, request, death trigger, mutation or security experiment is performed. Native recap population, non-nil IDs, transitions and event fields remain unverified.

Eleven separate actual-TOC/slash fixtures establish recorder mechanics only:

```text
luajit docs/addons/ApiContractProbe/tests/death_recap_current.lua docs/addons/ApiContractProbe
```

## Manual hyperlink residual observations

`/apicontract hyperlinks-residual <label>` is excluded from `all`; existing `hyperlinks` corpus and behavior remain unchanged. Thirty-five calls vary each of five optional positions through explicit nil, 0, 1, empty string, `x`, an owned empty table and an owned function, keeping other flags false and all six argument positions. Eight additional one-argument literals cover an unclosed link header, nested links/colors, crossed atlas/texture markers, stray closers, embedded NUL, control bytes and high bytes absent from the original corpus.

Every forwarded value is access-checked after API lookup/function guards. Owned table/function inputs are observed opaquely, never executed or retained by the recorder. Calls preserve raw arity, nil positions and opaque failures; outputs are capped at 16 positions and 256-byte strings, labels at 128 bytes, captures at ten (43 calls each). Seven actual TOC/slash fixtures prove recording mechanics only. No normalization, native coercion/output semantics, security behavior or historical fifth-flag claim follows.

## Manual resource color-curve inputs

`/apicontract resource-color-input <label>` is excluded from `all`. It creates a fresh `C_CurveUtil.CreateColorCurve()` and two original `CreateColor` objects, then adds the literal points `(0, red RGBA 1,0,0,1)` and `(100, blue RGBA 0,0,1,1)`. These are controlled inputs, not evidence of native resource scaling.

Every setup call retains bounded raw outcomes. Missing, invalid, inaccessible or erroring setup leaves both resource observations `unavailable-input`. Successful setup permits independent `UnitHealthPercent("player", false, originalCurve)` and `UnitPowerPercent("player", nil, false, originalCurve)` calls. Curve, colors and arguments are guarded before lookup and after function guards; the AddPoint receiver is checked again before invocation. Original objects are forwarded, not reconstructed or serialized substitutes.

At most seven calls occur per snapshot: one curve constructor, two color constructors, two AddPoint calls and two resource queries. Ten snapshots, 16 return positions, 256-byte scalar strings and 128-byte labels are retained. Resource outputs remain scalar/opaque observations: no result fields or methods, secret inspection, object retention, resource mutation or combat transition. Existing `resources` and `color-curves` behavior is unchanged.

Pinned `CurveUtilDocumentation.lua`, `LuaColorCurveObjectAPIDocumentation.lua`, `Blizzard_SharedXMLBase/Color.lua` and `UnitDocumentation.lua` establish construction/call shapes only. Native input scale, color/resource results and restricted-context behavior remain unverified. Thirteen local actual-TOC/slash fixtures prove recorder mechanics only:

```text
luajit docs/addons/ApiContractProbe/tests/resource_color_input.lua docs/addons/ApiContractProbe
```

## Manual explicit power observations

`/apicontract explicit-power <label>` is excluded from `all`. For `player` and `target`, it reads only published `Enum.PowerType.Mana`, `Rage` and `Energy`, without numeric fallback. Each unit/type receives independent `UnitPower` and `UnitPowerMax` calls with `unmodified=false/true`, and `UnitPowerPercent` calls with exactly four arguments, including an explicit nil curve.

The recorder makes at most 36 calls per snapshot and retains ten snapshots, 16 return positions, 256-byte scalar strings and 128-byte labels. Unit/type accessibility is rechecked after global lookup and function guards; inaccessible results remain opaque. Existing `resources` behavior is unchanged. No resource mutation, curve experiment, power-scale conclusion, restricted-context experiment or native-conformance credit follows. Pinned `UnitDocumentation.lua:2643–2767` and `PowerTypeConstantsDocumentation.lua:18–42` establish the call shapes and publication names only.

Eight actual TOC/slash fixtures cover recorder mechanics:

```text
luajit docs/addons/ApiContractProbe/tests/explicit_power.lua docs/addons/ApiContractProbe
```
