# API contract probe

## Manual equipped item info

- `equipped-item-info <label>` is manual-only and excluded from `all`. Call `GetInventoryItemLink("player", slot)` once for each slot 1–19, forwarding only each original accessible string to one independent `C_Item.GetItemInfo` call. Recheck link access after namespace/function guards; never forward serialized or truncated copies.
- Record `equippedItemInfo.slots` with `slot`, `producer` and `itemInfo`. Preserve exact arity, nil holes and opaque errors. **Item-info results alone have an 18-position cap** for the 18 declared returns, including nil positions 16–18; set `truncated` only when `n > 18`, without inspecting further values. Producer tuples and all shared 16-position bounds remain unchanged.
- Bound 38 API calls per snapshot, ten snapshots, 256-byte output strings and 128-byte labels. Missing, invalid, restricted and failing inputs or queries must not suppress peers. Do not traverse, retain or forward returned objects.
- No ItemLocation, transmog or mutation calls; no native acceptance, normalization or historical changed-contract conclusions. Eleven actual TOC/slash fixtures prove local recorder mechanics only; matching-client and historical semantics remain unverified.

## Manual combat audio settings reads

- `combat-audio-settings-read <label>` is manual-only and excluded from `all`. Independently call `C_CombatAudioAlert.IsEnabled()` twice.
- Call `GetSpecSetting` twice each for the published `CombatAudioAlertSpecSetting` names Resource1Percent, Resource1Format, Resource1Voice, Resource1Volume, Resource2Percent, Resource2Format, Resource2Voice, Resource2Volume and SayIfTargeted. Call `GetThrottle` twice each for published `CombatAudioAlertThrottle` names Sample, PlayerHealth, TargetHealth, PlayerCast, TargetCast, PlayerResource1, PlayerResource2, PlayerHealthSamePercent, TargetHealthSamePercent, PlayerResource1SamePercent and PlayerResource2SamePercent.
- Guard every publication container/member and API lookup before inspection; forward only original accessible finite enum values, rechecked after namespace/function guards. No numeric fallback or enum iteration. Missing, invalid, restricted and failing calls must not suppress peers.
- Store `combatAudioSettingsRead.IsEnabled`, named `specSettings` and `throttles` observation pairs. Preserve raw arity, nil positions and opaque errors; returned objects remain opaque and unretained. Bound 42 calls per snapshot, ten snapshots, sixteen return positions, 256-byte scalar strings and 128-byte labels.
- Never call setters, `SpeakText`, target-list or playback APIs. No throttling enforcement, security, native defaults/ranges or CVar interpretation. Ten actual TOC/slash fixtures cover recorder behavior only; native behavior remains unverified.

## Manual full names

- `full-names <label>` is manual-only and excluded from `all`; existing `names` behavior remains unchanged.
- Call `UnitFullName` once each for `player`, `target`, `focus`, `pet`, `party1`, `nonexistent`, `invalid-unit-token`, and the empty token. Guard global lookup/function access and recheck token access after function guards.
- Record `fullNames.units` with guarded unit observations and raw `result` tuples. Preserve exact arity and nil positions; scalar guards must reject conditional-secret outputs before inspection, comparison or serialization. Never combine or compare returned name/server values; returned objects stay opaque and unretained.
- Bound eight calls per snapshot, ten snapshots, sixteen return positions, 256-byte scalar strings and 128-byte labels. No restricted-identity experiments, mutations, realm defaults, coercion, classifications or native-conformance claims.
- Ten actual TOC/slash fixtures cover recorder mechanics only. Native same-/cross-realm fixtures, restricted behavior and historical evidence remain pending.

## Manual player state queries

- `player-state-queries <label>` is manual-only and excluded from `all`. Independently call `GetCollapsingStarCost`, `ShowingCloak` and `ShowingHelm` twice each with zero arguments.
- Guard `_G` before each field lookup and check function accessibility before invocation. Fresh lookups observe replacements; missing, restricted and throwing globals must not suppress peers.
- Preserve raw arity, nil positions and opaque errors under `playerStateQueries`; retain no returned objects. Bound six calls per snapshot, ten snapshots, sixteen tuple positions, 256-byte scalar strings and 128-byte labels.
- Never call `ShowCloak`, `ShowHelm`, setters or purchases. Do not infer cost, defaults, stability or native semantics. Eleven actual TOC/slash fixtures cover recorder mechanics only; native behavior remains unverified.

## Manual outfit tooltip

- `outfit-tooltip <label>` is manual-only and excluded from `all`.
- Call `C_TransmogOutfitInfo.GetOutfitsInfo()` once. Inspect only its first returned table, positions 1–4, and each entry's guarded `outfitID`; forward only original accessible finite numeric IDs to independent `C_TooltipInfo.GetOutfit` calls. Recheck IDs after namespace/function guards.
- Preserve raw arity, nil positions and opaque errors. TooltipData remains opaque, with no line/args traversal or tooltip UI calls. Guard each list/entry/field lookup; one failure must not suppress peers. Retain no returned objects.
- Bound each snapshot to five API calls, sixteen tuple positions, 256-byte scalar strings and a 128-byte label; retain at most ten snapshots.
- No outfit selection, requests, mutations or native content conclusions. Nine actual TOC/slash fixtures cover local recorder behavior only; native behavior remains unverified.

## Manual tradeskill item quality

- `tradeskill-item-quality <label>` is manual-only, excluded from `all`.
- Read player equipment slots 1–19 with `GetInventoryItemLink`; independently forward only each original accessible string to crafted-quality and reagent-quality queries, rechecking access after namespace/function guards. Never truncate inputs.
- Preserve producer/query arity, nil positions and opaque failures. Inspect only the first returned quality object through thirteen fixed declared fields: quality, icon, iconSmall, iconInventory, iconMixed, iconAppear, iconDissolve, barFill, barBackground, barBackgroundCap, barHighlight, iconChat, iconQuestObjective. Guard every receiver/field before lookup or serialization.
- Bound captures to 57 API calls per snapshot, ten snapshots, sixteen tuple positions, 256-byte output strings and 128-byte labels. Missing/error/restricted inputs must not suppress independent slots or queries; retain no returned objects.
- No recipe queries, crafting/orders, mutations, atlas interpretation or native quality conclusions. Ten separate actual TOC/slash fixtures cover recorder behavior only; native populations and semantics remain unverified.

`docs/addons/ApiContractProbe/` prepares native investigations of scalar curve point returns, `UnitSexBase` comparison, unit name/realm returns and finite numeric formatting. It shares one manual recorder; the existing [dispel probe](aura-dispel-curve-probe.md) remains separate. See [capture protocol](../addons/ApiContractProbe/README.md).

## Manual nameplate metrics

- Manual `nameplate-metrics <label>` is excluded from `all`. Independently call `C_NamePlate.GetNamePlateSize()` twice and `C_NamePlateManager.GetNamePlateHitTestInsets(value)` twice for each published `Enum.NamePlateType.Friendly` and `Enemy`.
- Guard containers/members before inspection; require original accessible finite numeric enum values without fallback and recheck them after namespace/function guards. Store size tuples and named inset observations under `nameplateMetrics`.
- Preserve raw arity, nil positions and opaque errors up to sixteen positions, including but not enforcing the declared two/four-return signatures. Bound six calls per snapshot, ten snapshots, 256-byte strings and 128-byte labels.
- No setters, camera/3D, unit queries, geometry/default/stability conclusions or native-conformance credit. Eleven actual TOC/slash fixtures establish local mechanics only.

## Manual quest favor

- Manual `quest-favor <label>` is excluded from `all`. Call `C_NeighborhoodInitiative.GetNeighborhoodInitiativeInfo()` once; preserve the opaque producer tuple and inspect only the first object's `tasks` positions 1–4 and each entry's `rewardQuestID`.
- Original accessible finite IDs independently feed `C_QuestInfoSystem.GetQuestLogRewardFavor(id)` and `(id, true)`, exactly one versus two arguments. Store bounded IDs and query observations under `questFavor.entries`; never synthesize IDs or forward serialized substitutes.
- Guard every receiver/field read and ID inspection; recheck IDs after namespace/function lookup and function guards. Nil, restricted, malformed and opaque-error outcomes remain independent.
- Bound nine API calls per snapshot, ten shared snapshots, sixteen tuple positions, 256-byte strings and 128-byte labels. No requests, initiative refresh, quest/reward mutations, defaults, cycle-cap or native favor semantics.
- Ten actual TOC/slash fixtures establish local mechanics only; native populated reward quests and favor/cap behavior remain unverified.

## Manual empowered stages

- Manual `empowered-stages <label>` is excluded from `all`. For player, target, focus, party1, nonexistent, invalid-unit-token and the empty token, independently call `UnitEmpoweredStageDurations(unit)` and `UnitEmpoweredStagePercentages(unit, false/true)` with exact argument counts.
- Store raw query tuples under `empoweredStages.units[].queries`. Inspect only the first returned table's positions 1–8 in `entries`, preserving scalar kind/status. Only duration entries use the shared ten-method `inspectDuration` whitelist for current-only observations. Guard each receiver before lookup and after function guards. No retention, cast-state changes, comparisons or lifecycle experiments; percentages remain scalar-only and are not normalized.
- Guard globals/functions and tokens before use, recheck tokens after lookup/function guards, and guard the returned table before every index read. Missing, restricted, nil and error outcomes must not suppress peers.
- Preserve exact arity and nil positions, opaque errors, sixteen tuple positions, 256-byte strings, 128-byte labels and ten shared snapshots. Maximum 21 producer calls and 560 duration-method calls per snapshot. No cast initiation, mutation, order/completeness/default/percentage or native behavior claims.
- Fifteen actual TOC/slash fixtures (ten existing, five new) establish local mechanics only. Matching native empowered-channel fixtures remain unverified.

## Manual encounter warning state

- Manual `encounter-warning-state <label>` is excluded from `all`. Independently call only `C_EncounterWarnings.IsFeatureAvailable()` and `IsFeatureEnabled()` twice each with exactly zero arguments; store observations under `encounterWarningState` keyed by API name.
- Guard namespace/function before lookup/use on every read. Missing, restricted, throwing or replaced APIs must not suppress peers. Preserve raw arity, nil positions and opaque errors; retain no raw returned objects.
- Limit calls to four per snapshot, shared snapshots to ten, result tuples to sixteen positions, scalar strings to 256 bytes and labels to 128 bytes.
- Do not toggle warnings, play sounds, create warnings, mutate state or perform security experiments. No native availability, enabled-state, default or stability claims. Twelve actual TOC/slash fixtures provide local recorder proof only.

## Manual ping enabled

- Manual `ping-enabled <label>` is excluded from `all`; call only `C_Ping.IsPingSystemEnabled()` twice independently with exactly zero arguments. Store observations under `pingEnabled.IsPingSystemEnabled`.
- Guard namespace/function before lookup/use; missing, restricted, throwing or replaced APIs must not suppress the peer read. Preserve raw arity, nil positions and opaque errors without retaining raw objects.
- Bound tuples to sixteen positions, scalar strings to 256 bytes, labels to 128 bytes and shared snapshots to ten; at most two query calls per snapshot.
- Do not call secure ping send/toggle/lifecycle APIs or read/write CVars. No native enabled-state, defaults, stability or security claims. Twelve actual TOC/slash fixtures provide local recorder proof only.

## Manual stable bonus slot

- Manual `stable-bonus-slot <label>` is excluded from `all`; call only `C_StableInfo.IsBonusPetSlotAvailable()` twice independently with exactly zero arguments, storing observations under `stableBonusSlot.IsBonusPetSlotAvailable`.
- Reuse guarded `observePublicQuery` namespace/function lookup and scalar serialization. Missing, restricted, throwing or replaced APIs must not suppress the peer observation; retain no raw objects.
- Preserve raw arity, nil positions and opaque errors. Bound each tuple to sixteen positions, scalar strings to 256 bytes, labels to 128 bytes and shared snapshots to ten, for two query calls per snapshot.
- Do not query pet IDs/lists, summon, move stable pets, rename, request or mutate state. Local proof establishes no native availability, defaults, stability or security semantics. Eleven actual TOC/slash fixtures cover recorder mechanics; matching native fixtures remain pending.

## Manual cooldown viewer reads

- Manual `cooldown-viewer-read <label>` is excluded from `all`. Read only the nine fixed published `CooldownViewerCategory` names Essential, Utility, TrackedBuff, TrackedBar, GroupBuff, SpecAgnosticEssential, SpecAgnosticTracked, EquipSlotEssential and EquipSlotTracked; never supply numeric fallbacks.
- Call `GetCooldownViewerCategorySet(category, false)` independently for each accessible finite published value. Only the first eight original accessible finite IDs may feed independent `GetCooldownViewerCooldownInfo(id)` and `GetValidAlertTypes(id)` calls. Preserve duplicates and peer failures.
- Inspect only the first info object's `cooldownID` and `category`, and the first alert list's eight scalar positions. Guard containers, members, indexes and fields before inspection; recheck original category/ID after API lookup and function guards. Never interpret flags or inspect linked fields.
- Preserve exact raw arity/nil positions and opaque errors within 16 result positions, 256-byte strings, 128-byte labels and ten captures. Maximum 153 API calls per capture; no refreshes, mutations or native default/order/completeness claims.
- Eight separate actual TOC/slash fixtures establish recorder mechanics only, not native conformance.

## Manual prey quest widgets

- Manual `prey-quest-widgets <label>` is excluded from `all`. Call `C_QuestLog.GetActivePreyQuest()` once; only its first original accessible finite quest ID may feed queries.
- Independently call `C_TaskQuest.GetQuestUIWidgetSetByType(questID, value)` for fixed published `Enum.MapIconUIWidgetSetType` names `Tooltip`, `BehindIcon`, `AdventureMapDetails`. No numeric fallback or guessed IDs. Preserve existing query tuples and add distinct widget details: each original finite set first return feeds `GetAllWidgetsBySetID` once, first four entries only.
- Guard containers and values before lookup/inspection; recheck both quest ID and enum value after API lookup/function guards. Preserve independent opaque errors, zero returns, nil positions and raw arity.
- Guard entry receivers before reading original widget ID/type. Only a type matching guarded published `Enum.UIWidgetVisualizationType.PreyHuntProgress` may feed `GetPreyHuntProgressWidgetVisualizationInfo`. Recheck original IDs, type and discriminator after API/function guards before comparison or forwarding; never dispatch wrong types.
- Inspect only the first visualization object and sixteen declared fields as guarded scalar observations; no recursive nested inspection or object retention.
- Bound nineteen calls per snapshot, ten shared snapshots, sixteen return positions, 256-byte strings and 128-byte labels. No quest acceptance, abandonment, requests, mutations, completeness or native classification/default claims.
- Sixteen cumulative actual TOC/slash fixtures, including two comparison/forwarding revocation regressions, prove recorder mechanics only; native quest/widget fixtures remain unverified.

## Manual major faction renown rewards

- Manual `major-faction-renown-rewards <label>` is excluded from `all`; record `majorFactionRenownRewards.producer` from `GetMajorFactionIDs(nil)` with exactly one explicit nil. Only first-return table positions 1–8 supply original accessible finite faction IDs.
- Each ID independently calls `GetRenownLevels(ID)`. First-return list positions 1–4 expose only `factionID`, `level`, `locked`, `isMilestone`, `isCapstone`. Original accessible finite `level` values feed `GetRenownRewardsForLevel(originalFactionID, originalLevel)`; never substitute the entry's `factionID` or deduplicate inputs.
- First-return reward positions 1–4 expose only `renownRewardID`, `uiOrder`, `isAccountUnlock`, `itemID`, `spellID`, `mountID`, `transmogID`, `transmogSetID`, `titleMaskID`, `transmogIllusionSourceID`, `icon`, `name`, `description`, `toastDescription`, `rewardType`. The pinned declaration additionally contains `isCollected`; it is outside this fifteen-field slice.
- Guard every receiver/index/field before lookup and every value before serialization. Recheck both original faction ID and level after API lookup/function guards. Preserve independent errors, zero returns, nil positions and raw arity; never forward reward IDs, recurse or retain raw objects.
- Cap 41 API calls per snapshot (1 + 8 + 32), ten shared snapshots, sixteen return positions, 256-byte strings and 128-byte labels. No unlock/claim/request/mutations or equality, ordering, completeness or native claims.
- Ten separate actual TOC/slash fixtures prove local mechanics only. Pinned retail `MajorFactionsDocumentation.lua:41–53,71–83,95–108,265–297` supplies signatures and fields; native fixtures and transitions remain unverified.

## Manual major faction journey

- Manual `major-faction-journey <label>` is excluded from `all`; capture `majorFactionJourney.producer` from `C_MajorFactions.GetMajorFactionIDs(nil)` with exactly one explicit nil argument, permitted by the pinned nilable expansion parameter.
- Only first-return table indices 1–8 supply original accessible finite faction IDs. Each entry independently invokes `ShouldDisplayMajorFactionAsJourney(ID)` and `ShouldUseJourneyRewardTrack(ID)` once, recording named `queries`; do not coerce, deduplicate or invent IDs/expansions.
- Guard namespaces/functions/list receivers before lookup and values before inspection/serialization. Recheck original IDs after each predicate namespace lookup and function guard, immediately before invocation. Preserve independent missing/restricted/invalid/error outcomes, zero returns and nil positions.
- Each ID additionally independently invokes `GetMajorFactionData(ID)` once after the same input/access rechecks. Preserve raw tuples and inspect only the first returned object: `description`, `playerCompanionID`, and opaque scalar `highlights`, plus first four highlight entries with guarded `title`, `description`, and `level`. Do not traverse other fields or query companions/rewards.
- Cap twenty-five API calls per snapshot, ten shared snapshots, sixteen return positions, 256-byte scalar strings and 128-byte labels. Retain no raw objects; no renown reward/level queries, mutations, ordering/completeness or native classification claims. Reward structures and native fixtures remain pending.
- Fifteen cumulative actual TOC/slash fixtures (ten existing, five new) establish local recorder mechanics only. Pinned retail `MajorFactionsDocumentation.lua:41–53,156–184` establishes signatures, not native behavior.

## Manual training grounds structures

- Manual `training-grounds-structures <label>` is excluded from `all`; independently call `C_PvP.GetTrainingGrounds()` and `GetRandomTrainingGroundRewards()` once each, storing `trainingGroundsStructures.grounds` and `.rewards`.
- Preserve exact tuple arity, zero returns, nil positions and opaque errors within sixteen observed positions. Only the first training return may be inspected as a table, at indices 1–8. Each entry exposes only `name`, `icon`, `gameType`, `shortDescription`, `longDescription`, `mapDescription`, `maxPlayers`, `battlegroundID`, `lfgDungeonID`, `mapID`, `isHoliday`, `isRandom`, `canEnter`, `isTrainingGround`.
- Rewards are five declared returns (`honor`, `experience`, `itemRewards`, `currencyRewards`, `roleShortageBonus`), not a structure. Nested objects remain opaque; never invent reward fields or traverse reward tables.
- Check access before every table/entry/field lookup and value inspection/serialization, including after earlier observations revoke access. Failures remain independent. Do not retain raw objects, iterate or measure source tables, queue/join/request/mutate, or infer ordering, completeness or native classifications.
- Cap two API calls per snapshot, ten shared snapshots, 256-byte scalar strings and 128-byte labels. Eleven separate actual TOC/slash fixtures prove recorder mechanics only; matching native fixtures and semantics remain unverified. Pinned retail `PvpInfoDocumentation.lua:599–610,767–774,1607–1625` supplies signatures and field declarations.

## Manual housing catalog

- Manual `housing-catalog <label>` is excluded from `all`. Under `housingCatalog`, capture `featured` twice via no-argument `C_HousingCatalog.HasFeaturedEntries`, `products` once via `C_CatalogShop.GetNewProducts`, and `refundable` once via `GetRefundableDecors` with optional nilable `productIdFilterOpt` omitted. Preserve exact raw arity/nil positions, opaque errors and the refund producer's second return `minTimeRemainingSeconds`.
- Only first-return product list indices 1–8 may feed original accessible finite IDs to independent `GetFirstCategoryByProductID(ID)` calls. Inspect only each first category object and its declared `ID`, `displayName`, `iconTexture`, `linkTag`, `isDisabled`, `showPersistentRefundButton` fields. Refund first-return list indices 1–8 expose only declared `decorGUID`, `timeRemainingSeconds`, `name`, `price`; `standaloneDecorProductID` remains an explicit missing observation.
- Add distinct `categoryProducts` per product row: read original finite accessible `ID` from the first category object after existing field serialization, with a fresh receiver guard. Call `GetProductIDsForCategory(originalID)` once, rechecking ID access after namespace/function guards; never use the serialized copy. Preserve raw sixteen-position tuples and inspect only first-return list indices 1–8 as scalars. No deduplication, recursion or product-info queries.
- Add `currencies` keyed by `HEARTHSTEEL_VC_CURRENCY_CODE` and `TRADERS_TENDER_VC_CURRENCY_CODE`: guarded reads through `Constants.CatalogShopVirtualCurrencyConstants` supply original accessible strings to one independent `C_CatalogShop.GetVirtualCurrencyBalance` call each. Guard every container/member before lookup/inspection and recheck the string after API namespace/function guards. No hardcoded fallback, coercion or truncated-input forwarding; preserve raw arity/nils/errors and independent peer failures. No refresh or currency-state/native claims.
- Guard every table/entry/field access and value inspection/serialization. Recheck IDs after namespace/function lookup and function guards before forwarding. Peer observations continue after missing, restricted, invalid or failed inputs. Do not iterate, measure or mutate returned tables, invent IDs, retain raw objects or inspect other returned objects.
- Cap four base plus eight category, eight category-product followup and two currency calls (twenty-two total) per snapshot, ten shared snapshots, sixteen tuple positions, 256-byte strings and 128-byte labels. No requests, purchases, refunds, currency guesses or native price/default/stability/completeness claims. Twenty-four cumulative actual TOC/slash fixtures (eighteen existing, six new) establish mechanics only; native fixtures and semantics remain unverified. Pinned `CatalogShopDocumentation.lua:259–272` explicitly permits an omitted filter; `673–683,838–846` define observed fields.

## Manual neighborhood structures

- Manual `neighborhood-structures <label>` is excluded from `all`. Under `neighborhoodStructures`, independently capture `C_NeighborhoodInitiative.GetNeighborhoodInitiativeInfo`, `GetInitiativeActivityLogInfo` and `GetTrackedInitiativeTasks`, once each without arguments. Preserve raw arity, nils and opaque errors within sixteen positions; inspect only first returned objects.
- Inspect fixed fields from pinned `NeighborhoodInitiativeDocumentation.lua`: initiative scalar fields with `fields.tasks`/`fields.milestones` remaining opaque. Add distinct `taskEntries`/`milestoneEntries` observations at indices 1–4, without replacing existing fields. Tasks use the existing task-field inspector, never triggering extra API calls. Milestones expose `milestoneOrderIndex`, `requiredContributionAmount`, opaque `rewards`, and distinct `rewardEntries` at indices 1–4 with `title`, `description`, `decorID`, `decorQuantity`, `favor`, `money`, `rewardQuestID`. Preserve activity scalar fields plus `taskActivity` indices 1–8 with only `taskID`, `playerName`, `taskName`, `completionTime`, `amount`. Inspect only `trackedIDs` indices 1–4 from the tracked object; accessible finite original IDs independently feed `GetInitiativeTaskInfo(ID)` and `GetInitiativeTaskChatLink(ID)`. Task-info first returns expose declared fields, with `requirementsList`/`criteriaList` opaque. Do not use activity IDs or returned task IDs as further query inputs.
- Guard every receiver before each field/index lookup and every value before inspection or serialization. Recheck original task IDs after namespace/function lookup and function guards, immediately before invocation. Missing, restricted and failed observations must not suppress peers. Retain no raw objects; do not fabricate IDs, rebuild input structures, iterate/measure returned tables or recurse.
- Cap three producer plus eight task calls per snapshot, ten shared snapshots, sixteen tuple positions, 256-byte strings and 128-byte labels. No requests, setters, tracking mutations or native semantics claims. Sixteen cumulative actual TOC/slash fixtures establish bounded recorder mechanics only, including preservation of existing eight-log/four-tracked limits and task queries; native fixtures, transitions, ordering and completeness remain unverified.

## Manual training grounds state

- Manual `training-grounds-state <label>` is excluded from `all`. Call only `C_PvP.AreTrainingGroundsEnabled`, `CanPlayerUseTrainingGroundsUI`, and `HasRandomTrainingGroundWinToday`, twice independently with exactly zero arguments.
- Store two raw observations per function-name key under `trainingGroundsState`. Reuse guarded namespace/function lookup and scalar serialization; missing, restricted, throwing or replaced APIs must not suppress peers.
- Preserve exact arity, nil positions and opaque errors, including the eligibility failure-reason return. Cap six calls per snapshot, ten shared snapshots, sixteen result positions, 256-byte strings and 128-byte labels.
- Do not queue, join, request, mutate, or query training-ground/reward structures. Local fixtures establish recorder mechanics only, not native defaults, stability, eligibility, win-history or transition semantics.

## Manual neighborhood state

- Manual `neighborhood-state <label>` is excluded from `all`. Call only `C_NeighborhoodInitiative.GetActiveNeighborhood`, `GetRequiredLevel`, `IsInitiativeEnabled`, `IsPlayerInNeighborhoodGroup`, `IsViewingActiveNeighborhood`, `PlayerHasInitiativeAccess`, and `PlayerMeetsRequiredLevel`, twice independently with exactly zero arguments.
- Store two raw observations per function-name key under `neighborhoodState`. Reuse guarded namespace/function lookup and scalar serialization; missing, restricted, throwing or replaced APIs must not suppress peers.
- Preserve exact arity, nil positions and opaque errors. Cap fourteen calls per snapshot, ten shared snapshots, sixteen result positions, 256-byte strings including GUIDs, and 128-byte labels. GUIDs are bounded scalar observations only.
- No requests, active/viewing changes, claims, contributions or other mutations; no `GetAvailableHouseXP`, structured initiative/activity-log producers or task queries. Do not infer defaults, stability, native behavior or group identity. Native fixtures and transitions remain unverified.

## Manual sets catalog

- Manual `sets-catalog <label>` is excluded from `all`. Independently call no-argument `C_TransmogSets.GetAvailableSets()` once and `IsUsingDefaultSetsFilters()` twice, preserving raw arity, nil positions and opaque errors within sixteen return positions.
- Inspect only first-return list positions 1–8 and six fixed fields `setID`, `name`, `collected`, `favorite`, `validForCharacter`, `grantAsPrecedingVariant`. Guard every receiver before lookup and each value before serialization; no iteration, length lookup or retained raw objects. Failures must not suppress peer observations.
- Cap three API calls per snapshot, ten shared snapshots, 256-byte strings and 128-byte labels. No downstream ID queries, setters, reset or selection; no ordering, completeness, default or native claims.
- Capture `grantAsPrecedingVariant` through the same guarded scalar observation path, as declared by pinned `TransmogSetsDocumentation.lua:530`; infer no semantics. Fourteen actual TOC/slash fixtures (eleven existing, three new) prove bounded mechanics only; native observed values and behavior remain unverified.

## Manual custom-set names

- Manual `custom-set-names <label>` is excluded from `all`. Independently call `C_TransmogCollection.GetNumMaxCustomSets()` twice and `GetCustomSets()` once; preserve raw arity, nils and opaque errors within sixteen return positions.
- Inspect only first-return ID table indices 1–4, guarding each lookup. Each accessible finite original ID permits one `GetCustomSetInfo(ID)` call. Only its original accessible string first return permits one `IsValidCustomSetName(name)` call; never forward a serialized/truncated name. Recheck ID/name after namespace/function lookup and function guards, immediately before invocation.
- Keep missing/restricted/invalid/error observations independent. Cap eleven API calls per snapshot, ten shared snapshots, 256-byte output strings and 128-byte labels; retain no raw objects.
- Exclude account mutations, ItemTransmogInfo forwarding, item-list/hyperlink chains and identity/name-validity/default/native conclusions. Ten actual TOC/slash fixtures establish bounded mechanics only, not native behavior. Pinned `TransmogItemsDocumentation.lua:365–379,398–404,564–570,808–820` defines the four signatures.

## Manual outfit slots

- Manual `outfit-slots <label>` is excluded from `all`. Independently call no-argument `GetAllSlotLocationInfo` and `GetSlotGroupInfo`; preserve raw arity, nil holes and opaque errors within sixteen return positions.
- Inspect first two location returns at indices 1–8, reading only guarded `slot`, `type`, `collectionType`, `slotName`, `isSecondary`. Each accessible finite original slot permits two independent exact-one-argument queries: `GetEquippedSlotOptionFromTransmogSlot(slot)` and `GetUnassignedAtlasForSlot(slot)`. Recheck input after namespace/function lookup and function guards.
- Inspect only first group return at indices 1–8: `position` and nested `appearanceSlotInfo`/`illusionSlotInfo`, each capped at eight slot entries with the same five fields. Group entries never trigger queries. Guard every receiver and field before lookup/serialization; no general recursion, length lookup, iteration or retained raw objects.
- Cap two producers plus 32 queries per snapshot, ten shared snapshots, 256-byte strings and 128-byte labels. Missing/restricted/failed observations must not suppress peers. No guessed IDs, type/option mapping, equality, ordering, completeness, atlas validity, mutations, 3D or native-conformance claims.
- Eleven actual TOC/slash fixtures prove bounded mechanics only; native behavior remains unverified.

## Manual outfit catalog

- Manual `outfit-catalog <label>` is excluded from `all`. Call `C_TransmogOutfitInfo.GetOutfitsInfo()` once; preserve raw arity, nils and opaque errors within sixteen return positions. Inspect only the first returned table at indices 1–8.
- Guard every list/entry/field lookup and scalar inspection. Read only `outfitID`, `name`, `icon`, `isEventOutfit`, `isDisabled`, `playerFacingOutfitIndex`, `situationCategories`. For categories, inspect only guarded scalar indices 1–8; no iteration or length lookup.
- Each accessible finite original outfit ID permits one independent `GetOutfitInfo(ID)` call. Recheck access after namespace/function lookup and function guards; never forward guessed or serialized/truncated IDs. Preserve its raw tuple and inspect only its first entry using the same field inspector, with no recursive follow-up queries.
- Cap nine API calls per snapshot, ten shared snapshots, 256-byte strings and 128-byte labels. Entry failures must not suppress peers; retain no raw objects. Exclude mutations, outfit selection and name queries.
- Eleven actual TOC/slash fixtures establish recorder mechanics only. No equality, identity, ordering, completeness or native-classification conclusions follow. Pinned `TransmogOutfitInfoDocumentation.lua:295–309,367–375,886–897` defines the calls and seven fields.

## Manual spell diminish categories

- Manual `spell-diminish-categories <label>` is excluded from `all`. Independently call `C_SpellDiminish.GetAllSpellDiminishCategories` for three fixed published ruleset names (`None`, `PvE`, `PvP`) and `GetSpellDiminishCategoryInfo` for eight fixed category names (`Root`, `Taunt`, `Stun`, `AoEKnockback`, `Incapacitate`, `Disorient`, `Silence`, `Disarm`).
- Accept only guarded finite numeric published values. No numeric fallback or enum iteration. Recheck enum input access after namespace/function lookup and function guards before forwarding.
- Preserve raw arity, nil positions and opaque errors within sixteen positions. Inspect only the first returned list at indices 1–8 or first returned info object; read only `category`, `name`, `icon`. Guard every list/entry/field access, without traversal, length lookup, mutation or retention. Independent failures must not suppress peers.
- Cap eleven calls per snapshot, ten shared snapshots, 256-byte strings and 128-byte labels. Missing UI prerequisites remain observed failures/empty returns, not synthesized fixtures; no UI loading.
- Exclude `ShouldTrackSpellDiminishCategory`, secret tracker events, native classification/order/completeness/default/population conclusions and 3D claims. Eleven local actual TOC/slash fixtures establish recorder mechanics only, not native behavior.

## Manual weekly progress

- Manual `weekly-progress <label>` is excluded from `all`. For fixed published `WeeklyRewardChestThresholdType` names `Raid`, `Activities`, `World`, `RankedPvP`, `Concession`, independently query `GetSortedProgressForActivity(value, false)` then `true`.
- Accept only guarded finite numeric published values; no fallback or enum iteration. Recheck value access after API lookup/function guards and before forwarding.
- Preserve sixteen raw return positions, nils and opaque errors. Inspect only the first returned table, positions 1–8, and fields `activityTierID`, `difficulty`, `numPoints`, guarding every table/entry/field read. Do not traverse, measure or mutate returned tables.
- Cap ten calls per snapshot, ten shared snapshots, 256-byte strings and 128-byte labels. Independent failures must not suppress peer observations. No guessed IDs, secret inspection, sorting/combine/completeness conclusions or native credit.

## Manual housing preview modes

- Manual `housing-preview-modes <label>` is excluded from `all`. Query only `C_HousingDecor.IsModeDisabledForPreviewState` twice independently for each fixed published `Enum.HouseEditorMode` name: `BasicDecor`, `ExpertDecor`, `Customize`, `Cleanup`, `Layout`, `ExteriorCustomization`.
- Resolve accessible finite numeric enum values only, without fallback or enum iteration. Guard tables/values/functions before lookup/use and recheck value access after function guards. Missing, invalid, restricted and error observations must not suppress peers.
- Preserve raw arity/nil positions and opaque errors within sixteen return positions, 256-byte strings, 128-byte labels, ten shared snapshots and twelve calls per snapshot.
- Do not change modes, mutate preview state, load LoD addons or infer native conditions, defaults or repeated-read stability. Eight local actual TOC/slash fixtures establish recorder mechanics only, not native behavior.

## Manual unit role predicates

- Manual `unit-role-predicates <label>` is excluded from `all`. Independently call `UnitIsLieutenant`, `UnitIsMinion`, and `UnitIsNPCAsPlayer` once each for `player`, `target`, `focus`, `pet`, `party1`, `nonexistent`, `invalid-unit-token`, and the empty string: 24 calls per snapshot, no omitted/nil-token case.
- Guard tokens/functions before inspection/use and recheck token access after lookup/function guards before invocation. Missing/restricted values and opaque errors must not suppress independent peers.
- Preserve raw arity/nil positions within sixteen return positions, 256-byte strings, 128-byte labels and ten shared snapshots. Do not infer classification, defaults, stability or native behavior.
- Exclude `UnitNameFromGUID`, threat/security queries and mutations. Ten local actual TOC/slash fixtures establish recorder mechanics only; native populated unit fixtures and outcomes remain unverified.

## Manual unit target display

- Manual `unit-target-display <label>` is excluded from `all`. Independently call only `UnitShouldDisplaySpellTargetName(unit)` twice for each of `player`, `target`, `focus`, `party1`, `nonexistent`, `invalid-unit-token`, and the empty string: fourteen calls per snapshot.
- Guard the token and function before use; recheck token access after function guards, immediately before invocation. Missing/restricted inputs or functions and opaque errors remain independent observations.
- Preserve raw arity/nils and accessible scalar results within sixteen positions, 256-byte strings, 128-byte labels and ten shared snapshots. Do not infer boolean expectations, defaults, repeated-read stability or native target semantics.
- Never query secret-return `UnitSpellTargetClass`/`UnitSpellTargetName` or add casting queries. Existing `casts` mode supplies separate manual context. Native cast/target fixtures remain unverified; eight local actual TOC/slash fixtures establish recorder mechanics only.

## Manual selected-slot spellbook durations

- Manual `spellbook-duration <actionslot> <label>` uses the existing integer parser and actual `GetActionInfo(actionslot)` producer; excluded from `all`. Only the accessible original `spell` kind and finite numeric spell ID permit `FindSpellBookSlotForSpell(ID, false, true, true, true)`, the pinned consumer's `knownSpellsOnly=false` branch, not inferred defaults.
- Record raw producer arity/nils/opaque errors. Forward only its original accessible finite numeric first pair as slotIndex/bank; no enum interpretation, coercion or guessing. Recheck kind/ID after slot lookup/function guards, and the pair before inspection and after every duration lookup/function guard.
- Independently query charge `(slot, bank)`, cooldown `(slot, bank, false)` and loss-of-control `(slot, bank)` durations. Inspect current returned objects with the existing ten read-only methods; receiver guards apply before lookup/use. Never retain objects or change cast-duration retention/counters.
- Bounds: sixteen returned positions, 256-byte strings, 128-byte labels, ten snapshots, one slot producer and three duration queries per snapshot, at most 480 method calls per snapshot. Preserve nil positions, missing/restricted outcomes and independent errors without native/default/bank/identity conclusions.
- Ten actual TOC/slash fixtures exercise forwarding, revocation, arity/errors, method guards, collectibility and bounds. Local proof establishes recorder mechanics only; native behavior remains unverified.

## Manual selected-slot spell durations

- Manual `spell-duration <slot> <label>` reuses the integer-slot parser and guarded `GetActionInfo(slot)` producer; it is excluded from `all`. Only an accessible `spell` string and finite numeric ID authorize calls with the original ID.
- Independently call `C_Spell.GetSpellChargeDuration(ID)` and `C_Spell.GetSpellLossOfControlCooldownDuration(ID)`. Protect namespace/function lookup and recheck both producer values immediately before each call. Missing/restricted/errored producers skip queries, never imply zero; one unavailable API does not gate its peer.
- Preserve raw arity, nils and opaque errors. Reuse the existing ten read-only duration methods through `inspectDuration`; no mutators, guessed IDs, spellbook or ordinary cooldown queries, or casts.
- Observe current returned objects only. Do not retain objects between captures or touch the cast-duration retention list/counter. Preserve sixteen-position, 256-byte string, 128-byte label and ten-snapshot bounds; at most two producers and 320 duration-method calls per snapshot.
- Nine separate actual TOC/slash fixtures establish recorder behavior, including interleaved cast-retention isolation. Native charge and loss-of-control timing/lifecycle and restricted-context behavior remain pending.

## Manual current unit auras

- Manual `unit-auras-current <label>` is excluded from `all`. Make three independent `C_UnitAuras.GetUnitAuras` calls with exact arguments `("player")`, `("player", "HELPFUL", 8)` and `("player", "HARMFUL", 8)`. Label the first `omittedFilterNegative`: missing required filter is a negative experiment, not valid/default semantics. Omit sort arguments; no continuation exists in the pinned signature.
- Preserve raw return arity/nils and opaque errors, with at most sixteen scalar observations. Only the first returned value may receive numeric entries 1–8, and only if it is an accessible table. Never traverse with length/pairs or mutate returned objects.
- For each accessible table/userdata entry, observe only `auraInstanceID`, `spellId` and `applications` through guarded field reads. Guard parent table/entry before every lookup and returned fields before scalar inspection; peer entries/calls continue after missing, restricted or failed observations. No coercion, downstream forwarding or retention.
- Fixed keys are consumer-grounded, not claimed schema declarations: pinned retail `Blizzard_FrameXMLUtil/AuraUtil.lua:45–54,266–267`. Signature source: `Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:452–469`, under the profile cache. No `AuraData` field declaration was located there.
- Bound output to three calls per snapshot, ten snapshots, sixteen return positions, eight entries per first table, 256-byte strings and 128-byte labels. Local tests prove recorder mechanics, not native ordering/completeness, identity, defaults, sorting, security semantics or historical contract equality.

## Manual current-only aura time

- Manual `aura-time <label>` is excluded from `all`; reuse the first-page player HELPFUL eight-slot producer without changing `aura-display-count` output or behavior. Forward only original accessible finite slots and guarded original aura instance IDs.
- Independently call `DoesAuraHaveExpirationTime`, `GetAuraBaseDuration`, `GetRefreshExtendedDuration`, and `GetAuraDuration`, each with exactly `("player", id)`. Omit optional spell IDs entirely. Recheck original inputs after namespace/function lookup and access guards, before forwarding.
- Preserve query status, return arity/nils and scalar kind/status. Reuse the ten-method duration whitelist on accessible table/userdata returns of `GetAuraDuration`, at positions 1–16, without calling the producer twice. Guard object access before inspection/lookup and recheck the receiver after function guards immediately before invocation; shared spell/cast consumers receive the same guard. Preserve independent method return arity/nils, opaque errors and bounded scalar values.
- No evaluate/set/copy methods, field traversal, retention, continuation, mutations or changes to cast-duration retained state: current-only observations, not lifecycle or native-conformance proof.
- Missing fixtures/APIs, restricted/invalid inputs and errors produce independent observations. Cap each snapshot at one page, eight data calls, 32 queries and 1,280 method calls (8 IDs × 16 returned positions × 10 methods), 16 tuple positions per producer/method, 256-byte strings and a 128-byte label; cap captures at ten.

## Manual first-page aura display counts

- Manual `aura-display-count <label>` is excluded from `all`. Call `C_UnitAuras.GetAuraSlots("player", "HELPFUL", 8)` exactly once. Preserve continuation plus vararg slots as a bounded raw tuple, never traverse continuation or claim enumeration completeness.
- At most eight original accessible finite slot values feed `GetAuraDataBySlot("player", slot)`. Preserve producer arity without traversing AuraData; access only `auraInstanceID` through guarded field lookup on an accessible table/userdata. Require an accessible finite original numeric ID, without coercion or invented values.
- Independently call `GetAuraApplicationDisplayCount("player", id)` with optional arguments omitted, `(1)`, `(2)`, `(2, 5)` and `(1, 1)`. Guard namespace/function and produced inputs before inspection/forwarding; recheck inputs after lookup and function guards. Record missing, inaccessible, invalid and error cases independently.
- Preserve raw return arity/nils and opaque errors with 16-position, 256-byte string, 128-byte label and ten-snapshot limits. At most eight data calls and forty count queries per snapshot; no gameplay mutation or native execution. Local fixtures establish recorder behavior only, not native ordering, defaults, formatting, coercion or completeness.

## Manual selected-slot spellbook metadata

- Manual `spellbook-metadata <slot> <label>` uses the existing integer-slot parser and `GetActionInfo(slot)` producer; excluded from `all`. Only an accessible `spell` kind and finite numeric original ID authorize queries.
- Independently call `C_SpellBook.FindBaseSpellByID`, `FindFlyoutSlotBySpellID` and `FindSpellOverrideByID`, each with exactly that original ID. Guard namespace/function access and recheck kind/ID after lookup and function guards before each call.
- Preserve producer/query arity, nil positions, opaque errors and restricted values without inferred identity, slot/bank validity or native semantics. Bounds: sixteen scalar positions, 256-byte strings, 128-byte labels, ten snapshots and thirty total queries.
- No spellbook slot producer, duration query, mutation, secret-specific call or object retention. Existing spell-metadata output remains unchanged. Eight actual TOC/slash fixtures establish recorder mechanics only; native classifications, transitions and security remain unverified.

## Manual selected-slot spell metadata

- Manual `spell-metadata <slot> <label>` reuses the actions integer-slot parser and is excluded from `all`. Call `GetActionInfo(slot)` with exactly one argument; preserve raw producer arity and nil positions.
- Only an accessible string first return exactly `spell` and accessible finite numeric second return authorize querying the original ID. No conversions, guessed IDs or inferred classification.
- Independently call `C_Spell.GetSpellDisplayCount`, `GetSpellMaxCumulativeAuraApplications`, `IsConsumableSpell`, `IsExternalDefensive`, `IsPriorityAura`, `IsSpellCrowdControl` and `IsSpellImportant`, passing only the ID. Omit display-count optional arguments.
- Query `GetVisibilityInfo(ID, value)` once for each fixed published `Enum.SpellAuraVisibilityType` name: RaidInCombat, RaidOutOfCombat, EnemyTarget. Accept only guarded finite numeric values, without numeric fallbacks or arbitrary iteration. Missing/restricted/invalid enum values record `unavailable-enum` without calls; base queries remain independent. Recheck original spell inputs and the enum value after lookup/function guards, before each call. Preserve zero returns as distinct from nil and opaque errors.
- Guard each namespace/function lookup and producer values before inspection; recheck input access after lookup/function checks before every downstream call. Missing APIs and opaque errors do not suppress peers. No action execution or casts.
- Independently record `auraQueries.AuraIsBigDefensive` using `C_UnitAuras.AuraIsBigDefensive(originalID)`, whose pinned declaration accepts one `SpellIdentifier` and returns a boolean. No aura instance or invented ID. Guard namespace/field/function and original kind/ID before inspection; recheck input access after potentially revoking lookups/function guards. Aura and spell-query failures must not suppress each other.
- Preserve exact arity, nils, sixteen scalar positions, 256-byte strings, 128-byte labels and ten snapshots. At most seventy base metadata, thirty visibility and ten aura-defensive calls across ten snapshots. Nineteen cumulative actual TOC/slash fixtures include five added aura-defensive cases and prove bounded recorder behavior only, with no native-conformance credit; native outputs, classifications, rank/override fixtures and security remain pending.

## Manual outfit state

- Manual `outfit-state <label>` is excluded from `all`. Independently call each of seven `C_TransmogOutfitInfo` queries twice with exactly zero arguments: `GetCurrentlyViewedOutfitID`, `GetMaxNumberOfUsableOutfits`, `GetNextOutfitCost`, `GetPendingTransmogCost`, `HasPendingOutfitSituations`, `IsEquippedGearOutfitDisplayed`, `IsEquippedGearOutfitLocked`.
- Store two raw observations per function-name key under `outfitState`. Reuse guarded namespace/function lookup and accessible scalar recording. Missing, restricted, throwing or replaced APIs must not suppress peer observations.
- Preserve exact return arity, nil positions and opaque errors, including pending-cost two-return and no-return cases. Bound captures to 14 calls per snapshot, ten snapshots, 16 return positions, 256-byte strings and 128-byte labels.
- No mutations, purchases, selection, price interpretation, defaults, stability or native-conformance claims. Matching-client state transitions and cost semantics remain unverified.

## Manual public queries

- Manual `public-queries <label>` is excluded from `all`. Independently call `C_GameRules.IsPersonalResourceDisplayEnabled()` twice and `C_DelvesUI.GetLockedTextForCompanion()` twice, each with exactly zero arguments.
- Independently call five additional no-argument queries twice each: `C_Housing.IsHousingMarketShopEnabled`, `C_EncounterTimeline.GetCurrentTime`, and `C_InstanceEncounter.IsEncounterLimitingResurrections`, `IsEncounterSuppressingRelease`, `ShouldShowTimelineForEncounter`. Preserve distinct named outputs `housingMarketShopEnabled`, `encounterTimelineCurrentTime`, `encounterLimitingResurrections`, `encounterSuppressingRelease`, `showTimelineForEncounter` alongside the original two.
- Add independent zero-argument pairs for `C_TransmogOutfitInfo.GetActiveOutfitID`, `C_HousingCustomizeMode.IsHouseExteriorDoorHovered`, and `C_SpellDiminish.IsSystemSupported`, named `activeOutfitID`, `houseExteriorDoorHovered`, and `spellDiminishSystemSupported`. Retain all seven preceding API pairs.
- Record only the omitted-companion observation. No invented companion/trait-tree IDs, trait-tree query, mutations, CVar changes, state transitions or default claims.
- Protect namespace lookup; check access before inspecting functions/results. Preserve independent missing/lookup/call errors opaquely and retain raw arity, nils and repeated observations without stability conclusions.
- Reuse sixteen-position, 256-byte string, 128-byte label and ten-snapshot bounds: ten APIs, twenty query calls per snapshot, 200 maximum. Seventeen cumulative actual TOC/slash fixtures (twelve existing, five new) cover zero arguments, repeated results, independent failures, access guards and bounds. Native ruleset/encounter/housing transitions, timeline timestamps, outfit identity, exterior-door hover, spell-diminish support, companion lock policy, trait-tree fixtures and security behavior remain unverified.

## Manual equipped-item binding

- Manual `item-binding <label>` is excluded from `all`. Query `GetInventoryItemLink("player", slot)` once for each fixed slot 1–19, retaining exact producer return arity and nil positions.
- Use only the first actual producer return when it is an accessible non-secret string. Query `C_Item.IsItemBindToAccount` once with the original link, never a truncated saved string, synthetic link or numeric item ID.
- Check accessibility before type inspection and namespace lookup; recheck link accessibility immediately before calling the binding API, after function lookup/access checks. Missing, nil, nonstring or failed producers explicitly skip binding with `unavailable-input`; restricted links use `restricted-input`. Neither means false.
- Keep producer and binding errors opaque and continue independent slots. Retain sixteen scalar positions, 256-byte strings and ten snapshots. No mutations, alternate producers or native execution.
- Eight separate actual TOC/slash fixtures cover argument flow, arity, unavailable/restricted inputs, namespace failures, access revocation, bounds and manual routing. Pinned ItemDocumentation and the Journeys item-link consumer guide arguments, not native binding classifications or security conclusions.

## Manual StatusBar fill style

- Manual `statusbar-fill <label>` is excluded from `all`. Create one unnamed StatusBar under UIParent, immediately attempt Hide, then capture the raw fresh GetFillStyle default before any fill setter.
- Read only published `Enum.StatusBarFillStyle` names Standard, StandardNoRangeFill, Center and Reverse. Pass accessible finite scalar values without numeric fallbacks. Record missing/restricted/nonscalar inputs without setter attempts.
- For each eligible value, record one SetFillStyle and two subsequent independent GetFillStyle calls, preserving setter failures, raw arity/nils and opaque results rather than equality conclusions.
- Guard object, method and result access before lookup or inspection. Retain sixteen scalar positions, 256-byte strings and ten snapshots. Abort fill operations after constructor/Hide failure; retain no object reference. Hide failure explicitly leaves client visibility unconfirmed; no destruction or alternate setter is attempted.
- No Show, sizing, layout/rendering, invalid inputs, restricted-context probes or other setters. Addon code is tainted; no untainted setup assumption. Seven separate actual TOC/slash fixtures test recorder behavior, not native conformance. Native initial-state, validation and coercion captures remain pending.

## Manual scalar curve state

- [x] Manual `curve-state <label>` is excluded from `all` and leaves existing `curves` capture unchanged.
- [x] Construct empty and duplicate/unsorted scalar curves with inputs `(30,4), (10,7), (20,2), (10,9)`. Record `GetType`, `GetPointCount`, `GetPoints`, `HasSecretValues` and `Evaluate` at `-1,0,10,15,20,30,31` before and after `SetToDefaults`.
- [x] Set type only through safely discovered `Enum.LuaCurveType.Linear`, recording missing prerequisites or setter failures without guessing enum values.
- [x] Copy a separate populated curve, capture the copy before mutation, then original and copy after `AddPoint(40,11)` and `ClearPoints`. Preserve observations even if copying aliases the original; do not classify native behavior.
- [x] Call only constructed/copied curve methods; scalar-only method results remain opaque except the existing bounded point/GetXY observer for `GetPoints`. Restricted copies and constructor results fail closed; errors are opaque.

Pinned `LuaCurveObjectAPIDocumentation.lua` and `LuaCurveObjectBaseAPIDocumentation.lua` describe these argument and return shapes, not observed native defaults or copy behavior. Four point entries, eight return positions, seven evaluation inputs and the shared ten-snapshot limit bound capture. Color curves and secret semantics remain outside this experiment; point removal/replacement uses the separate experiment below. Local fake-client tests retain differing defaults, duplicate replacement/preservation, copied/aliased state and evaluation outputs; native execution remains pending.

## Manual scalar point mutation

- [x] Manual `curve-edit <label>` is excluded from `all`. Each `RemovePoint` index `-1,0,1,2,3,4` receives its own curve populated with `(30,4), (10,7), (20,2)`.
- [x] Two independent populated curves receive empty and unsorted duplicate `SetPoints` inputs `(30,4), (10,7), (20,2), (10,9)`. Construct every vector through actual accessible `CreateVector2D`; missing/throwing/restricted/invalid constructors prevent replacement, without a table-coercion fallback.
- [x] Record input definitions, count, bounded points and `Evaluate(15)` before and after each attempt; preserve mutation arity/nils and opaque errors. Retain four point entries, eight return positions and ten snapshots using existing accessibility-first observers.
- [x] Behavioral fixtures preserve differing zero-/one-based removal and SetPoints ordering, constructor failures, redaction, independent curves, manual routing and storage bounds. Do not infer index base, duplicate policy, interpolation or alias behavior from fixtures.

Pinned scalar documentation specifies `RemovePoint(index: luaIndex)` and `SetPoints(point: table<vector2>)`; pinned `Vector2D.lua` constructs an `x`/`y` object with `GetXY`. Native captures remain pending. Only numeric index controls are included; invalid-type coercion, color curves, vendor mutation and security behavior are excluded.

## Manual scalar resource scale capture

- Manual `resources <label>` is excluded from `all`; observe player/target/focus/pet/nonexistent without existence gating.
- Preserve raw health/max and power/max/type, exact arity and nil positions. Health percent curve argument 3 follows explicit false prediction; default, false/no-curve and false/nil-curve calls remain separate. Power uses default/nil power type with default calls and explicit false/true unmodified variants; curve is argument 4.
- Record scalar curve inputs `(0,0), (.5,10), (1,20), (25,30), (50,40), (100,50)` and explicitly set accessible `Enum.LuaCurveType.Linear`. Missing construction prerequisites prevent curved queries, not underlying observations. Never guess enum numbers or power IDs.
- Reuse accessibility-first scalar-only recording and existing return/storage bounds. Do not interpret normalized versus percentage scale.
- Native partial-resource fixtures, prediction, secondary powers, color curves and security remain pending; no events or gameplay mutation.

Pinned retail `UnitDocumentation.lua` declares `UnitHealthPercent(unit, usePredicted=true, curve)` and `UnitPowerPercent(unit, powerType, unmodified=false, curve)`. Declarations establish argument positions, not native scale. Simulator policy specs remain unchanged.

## Manual color-curve state

- [x] Manual `color-curves <label>` stays outside `all`; construct own curve/colors from four distinct recorded RGBA definitions at x `0,32,-16,48`, without guessed enum values.
- [x] Capture empty/populated `GetType`, `GetPointCount`, `HasSecretValues`, `GetPoints`, six indices `-1,0,1,2,3,4`, and `Evaluate`/`EvaluateUnpacked` at `-17,-16,0,16,32,48,49`.
- [x] Record Copy return arity/opaque curve shape; capture original and copy before/after adding the defined fifth point and clearing the copy. Record original before/after reset, preserving copy-alias effects rather than assuming isolation or defaults.
- [x] Inspect returned tables with accessibility-first raw `x/y/r/g/b/a` fields, four array entries and three table levels; userdata stays opaque. Never invoke returned color/point methods or metamethods. Only guarded owned-curve calls execute.
- [x] Preserve exact return arity, sixteen positions, nils, secret-channel redaction and opaque errors. Missing/invalid/restricted construction fails closed; shared ten-snapshot limit applies.
- [x] Local `tests/color_curves.lua` exercises different order/default/copy behaviors, packed/unpacked values, inaccessible channels, opaque userdata, hostile lookup, constructor failures, tuple/storage bounds and manual-only routing.

Cached `LuaColorCurveObjectAPIDocumentation.lua`, `LuaCurveObjectBaseAPIDocumentation.lua`, `CurveUtilDocumentation.lua` and `Blizzard_SharedXMLBase/Color.lua` guide construction/field names, not native proof. Actual native state/evaluation, userdata fields, security, removal/replacement and other interpolation types remain unverified.

## What it must do

- [x] Record build provenance and manual scenario labels without automatic capture or gameplay changes.
- [x] Observe scalar curve empty/populated returns, explicit index queries, fields/GetXY, repeated identity and an isolated returned-point mutation attempt without assuming native outcomes.
- [x] Independently preserve `UnitExists`, `UnitSex` and `UnitSexBase` observations for player, target, focus, pet, party1, party2, nonexistent, invalid-unit-token and empty tokens, alongside raw named enum values. Never gate sex queries on existence, convert numbering or compare outputs. Unit records contain `exists`, `legacy` and `base` rather than an aggregate status; each observation retains its own status and arity.
- [ ] Obtain native before/during/after-disguise captures with independently established stable unit identity; no fixture generation is performed.
- [x] Capture `UnitName` and `UnitNameUnmodified` with `/apicontract names <label>` and `all` for fixed tokens `player`, `party1`–`party4`, `target`, `nonexistent`, `invalid-unit-token` and the empty string. Never skip queries based on `UnitExists`.
- [x] Preserve name/realm positional returns and exact arity, including zero returns, nil slots and empty realm strings, under existing accessibility-first redaction and opaque-error rules. Same/cross-realm fixture identity comes from manual labels, not inferred return values.
- [x] Capture `/apicontract numbers <label>` and `all` numeric samples for `C_StringUtil.FloorToNearestString` and `RoundToNearestString`, with `GetLocale` provenance, build and label. Use exactly the 25 finite inputs listed in the capture protocol: signed ties and their neighbors, integers, fractions and large finite magnitudes.
- [x] Preserve numeric helper result bytes, positional nils and exact arity through the existing bounded observer without guessing rounding or grouping. Missing APIs and failed access checks fail closed; errors remain opaque.
- [x] Redact inaccessible/secret values before comparison or saving, and record opaque failures without stringifying errors.
- [x] Bound capture count, inspected results, arrays, depth and strings.
- [x] Exercise actual addon loading and slash commands through local behavioral fixtures.

## Manual raid markers

- Manual `raid-markers <label>` is excluded from `all`; use the nine sex-control tokens: player, target, focus, pet, party1, party2, nonexistent, invalid-unit-token and empty string.
- Independently call `CanBeRaidTarget(unit)` twice per token, `IsRaidMarkerActive(index)` twice per index 1–8, and `IsRaidMarkerSystemEnabled()` twice with no arguments. Preserve repeated raw observations, not stability comparisons; missing functions never gate others.
- Guard access before every inspection; record exact arity/nils and sixteen scalar positions, bounded 256-byte strings, opaque objects/errors and explicit truncation. Shared ten-snapshot cap bounds capture; at most 36 API calls per snapshot.
- Exclude secret-return `GetRaidTargetIndex`, setters, clear/place/remove operations, permission and security claims. Native output and populated world-marker fixtures remain pending.
- Six separate actual TOC/slash fixtures prove argument lists, repeated observations, independent failure handling, guards and bounds. Pinned `RaidMarkersDocumentation.lua` establishes signatures only, not native conformance.

## Manual abbreviations

- Manual `abbreviations <label>` stays outside `all`; reuse the existing 25-number corpus, preserving `numbers` behavior.
- Call `AbbreviateLargeNumbers(number)` and `AbbreviateNumbers(number)` independently with options omitted; capture `GetLocale()` once without changing locale.
- Preserve exact scalar result arity, nil slots and accessible bytes, with sixteen positions, 256-byte strings and explicit truncation. Guard access before inspection; returned objects and errors remain opaque.
- Missing APIs and failed access checks fail closed. Bound each snapshot to fifty abbreviation calls and one locale call, under the shared ten-snapshot limit.
- Separate actual TOC/slash fixtures cover shared inputs, omitted options, localization, missing/restricted functions and results, errors, arity, opaque objects and bounds.

Pinned retail `LocalizationDocumentation.lua` supplies signatures only. No formatter, options/configuration tables, producer or mutator belongs to this slice. Native semantics and option/configuration behavior remain unverified separate gaps.

## Manual heal calculator

- Manual `heal-calculator <label>` stays outside `all`; attempt two independent no-argument `CreateUnitHealPredictionCalculator()` constructions per snapshot.
- For each accessible object, observe `GetHealAbsorbMode`, `GetHealAbsorbClampMode`, `GetDamageAbsorbClampMode`, `GetHealAbsorbs` and `GetDamageAbsorbs` twice each. Preserve repeats without comparing values or inferring stability.
- Record constructor/getter exact arity, nil slots and sixteen scalar/opaque positions with explicit truncation. Guard object access before method lookup and method/result access before inspection. Do not serialize runtime objects or errors, invoke returned-object methods or assume zero/default values.
- Missing, throwing, restricted or non-object constructor results prevent getter work for that attempt. Bound each snapshot to two constructor and twenty getter calls; retain the shared ten-snapshot cap.
- Separate actual TOC/slash fixtures cover repeated raw values, constructor/method/result access failures, opaque errors/objects, exact larger tuple arity, manual-only dispatch and storage limits.

Pinned retail `UnitDocumentation.lua` and `UnitHealPredictionCalculatorAPIDocumentation.lua` guide the constructor and five getter signatures only. Native observations, populated state, reset/default semantics and security remain unresolved. No unit contexts, setters, population, native execution or conformance credit belong to this partial experiment.

## Manual producer durations

- Manual `cast-durations <label>` is excluded from `all`. Query casting/channel durations and empowered durations with hold omitted, false and true for player/target/focus/party1/nonexistent/invalid-unit-token/empty tokens.
- Record exact arity and at most 16 scalar/opaque positions. On accessible returned objects, explicitly query only `GetTotalDuration`, `GetElapsedDuration`, `GetRemainingDuration`, `GetElapsedPercent`, `GetRemainingPercent`, `GetStartTime`, `GetEndTime`, `GetClockTime`, `GetModRate`, `HasExpired`. Guard object access before method lookup and function access before invocation; errors stay opaque. Method results are scalar-only, with no recursive object inspection.
- Keep at most 28 returned-object references from one previous duration snapshot in memory. Re-observe them on the next duration capture, then replace retention. Saved records contain local observation references, never runtime objects/functions or native identity claims. Mark excess retention explicitly; no equality/string conversion or mutation calls.
- Local fixtures distinguish live versus frozen producer objects across manual captures. Native active/completed/interrupted/absent states and time transitions require user-provided fixtures; no casts, timers, clock mutation or native security conclusions.

Pinned retail `UnitDocumentation.lua` declares optional empowered hold default true and potentially absent results; `LuaDurationObjectAPIDocumentation.lua` declares the ten read methods. These declarations guide calls, not native results. Shared ten-snapshot limit applies; retention resets when the addon reloads.

## Manual cast identities

- [x] `/apicontract casts <label>` and `all` query `UnitCastingInfo` and `UnitChannelInfo` for fixed `player`, `target`, `focus`, `party1`, `nonexistent`, `invalid-unit-token` and empty-string tokens without existence gating or gameplay actions.
- [x] Dedicated accessibility-first scalar-only protected calls preserve exact arity and up to 16 positions, including interior/trailing nils, with explicit truncation beyond 16. Accessible objects retain status/kind only; never execute returned-object methods, indexing, equality or string conversion. Errors remain opaque.
- [x] Preserve raw scalar IDs, empowerment flags and stage counts without identity/coercion assumptions; use existing build/time/label infrastructure and shared ten-capture limit. Document separate batches for fixture sequences.

Pinned `12.0.0-register.json` declarations describe casting position 10 and channel position 11 as castBarID; they are not native evidence and the executing client may differ. Matching native client, controllable casts, non-player channels and empowered/non-empowered fixtures remain pending. No recorder fixture earns native audit credit.

## Manual selected-slot action loss-of-control duration

- `action-loss-control-duration <slot> <label>` reuses `parseActionSlot`, is manual-only and excluded from `all`. Independently record `GetActionInfo(slot)` control and `C_ActionBar.GetActionLossOfControlCooldownDuration(slot)` with one original slot argument; no spell-kind or successful-control gate.
- Preserve guarded `actionLossControlDuration.slot`, raw `identity` and `duration` tuples with exact arity, nils and opaque errors. Check slot accessibility before forwarding and again after namespace/function lookup and guards. Inspect only accessible duration objects using the shared ten read-only methods and receiver guards.
- Bound to two selected-slot API calls, sixteen returned positions, 160 method calls per snapshot, ten snapshots, 256-byte scalar strings and 128-byte labels. Retain no duration objects and do not change cast-duration lifecycle state; percentages, timing, equality and native semantics are not inferred.
- Historical `GetActionLossOfControlCooldown` and `ActionBarCooldownInfo` fields remain missing. Add no adjacent cooldown calls, mutations, action execution or button registration. Native active/expired loss-of-control fixtures and restricted-context behavior remain unverified.
- Ten actual TOC/slash fixtures cover independent control/query behavior, original slots, method arity and revocation, errors/nils, bounds, collectibility and cast interleaving. Local proof earns no native credit.

## Manual selected-slot action state

- `action-state <slot> <label>` reuses `parseActionSlot`; excluded from `all`. Record `GetActionInfo(slot)` once as control without requiring a spell kind or successful control call.
- Independently call twelve `C_ActionBar` APIs with exactly the original parsed slot: `GetActionAutocast`, `GetActionText`, `GetActionUseCount`, `HasRangeRequirements`, `IsAttackAction`, `IsAutoRepeatAction`, `IsConsumableAction`, `IsEquippedAction`, `IsItemAction`, `IsStackableAction`, `IsUsableAction`, `IsActionInRange`. Omit the range target. Independently call `GetExtraBarIndex` and `GetMultiCastBarIndex` once each with zero arguments.
- Capture `actionState.slot` as a guarded scalar, `identity` as a raw tuple, and named `queries`/`bars`. Guard slot before forwarding and recheck after namespace/function lookup and function guards. Check result accessibility before inspection; inaccessible results and thrown errors remain opaque. Preserve exact arity, zero returns and nil positions; peer calls continue after errors.
- Cap 15 API calls per snapshot, ten shared snapshots, sixteen return positions, 256-byte strings and 128-byte labels. No loss-of-control APIs, button registration, action execution, mutations or native classification/default claims. Existing `actions` mode remains unchanged.
- Ten actual TOC/slash fixtures establish local mechanics only; native action, range, item, macro and restricted-state outcomes remain unverified.

## Selected action counts and charges

- [x] `/apicontract actions <slot> <label>` queries one explicitly selected signed decimal integer slot; zero and negative controls are accepted. Reject invalid syntax and integers outside the exactly representable range before any query. This is command validation, not a claim about native coercion. `all` never queries actions.
- [x] Capture optional `GetActionInfo`, default `C_ActionBar.GetActionDisplayCount`, and explicit thresholds 0, 1 and 9999 with replacement `*`, preserving differing display/charge observations.
- [x] Capture `GetActionCharges` exact arity and the five accessible raw table fields `currentCharges`, `maxCharges`, `cooldownStartTime`, `cooldownDuration`, `chargeModRate`; never invoke metatable lookup. Capture `GetActionChargeDuration` only as accessible kind/status and arity, never its values or methods.
- [x] Bound each snapshot to seven selected-slot calls, 16 stored return positions per call, existing ten-capture storage and build/time/label provenance; redact before inspection and keep errors opaque.

Native ordinary-spell, charged-spell, consumable and empty-slot fixtures plus before/use/recharge/restoration transitions remain pending. Duration-method contracts remain uncaptured. These eight plan rows are only partially prepared; no `GetActionCooldown` query belongs to this slice.

## Residual StripHyperlinks

- [x] Manual-only `/apicontract hyperlinks <label>` records 16 literal inputs × nine flag variants (144 calls); `all` excludes it. Exact corpus and ordered flags are documented in the capture protocol.
- [x] Preserve omitted optional arguments separately from five explicit false flags; record exact input, variant, flags, argument count and raw accessible result arity without guessed stripping/coercion or malformed-text normalization.
- [x] Use existing eight-position observer and ten-capture bounds. Strings retain at most 256 bytes with explicit truncation; inaccessible results are redacted, missing APIs fail closed and errors are opaque.
- [x] Behavioral fixtures discriminate all argument positions/omission, UTF-8 and embedded NUL output bytes, malformed output, restrictions/errors, truncation, manual-only routing and overflow without extra calls.

Native execution, nonboolean truthiness, arbitrary-byte contracts beyond the fixed corpus, string-view lifetime and security remain unverified. This recorder does not change the [simulator StripHyperlinks contract](strip-hyperlinks.md) or earn native audit credit.

## Residual plain-function callbacks

- [x] Manual `callbacks-start <label>` and `callbacks-stop`, excluded from `all`, register only owned plain functions for global `UNIT_HEALTH` and unit `UNIT_HEALTH`/`player`; cleanup uses those exact identities.
- [x] Preserve independent registration/removal arity and scalar-only results with opaque errors. Refused or uncertain registration is not success; a throwing call retains possible registration ownership for cleanup. Failed/refused/uncertain cleanup retains identity and reports incomplete cleanup until a successful retry.
- [x] Capture real deliveries only: 128 entries per session, exact arity and 16 scalar-only positions, nil preservation, accessibility-first redaction, label/build/time provenance. Overflow does not prevent manual cleanup; stale callbacks cannot record after stop.
- [x] Reject repeated starts while cleanup is outstanding, create one new session after stop, and bound saved sessions to ten. Missing access APIs prevent registration. Callback function objects remain private, outside SavedVariables.

The four pinned global APIs take event name and callback, with a third unit argument for unit registration/removal; current retail declarations list no returns. Record observed arity instead of importing the simulator global registration boolean policy. Zero-return protected-call success is an accepted call, not proof of a native delivery. External `UNIT_HEALTH` fixtures remain pending. FunctionContainer wrappers, native duplicate/ordering semantics, aliases, callback-time mutation, recursion, GC and security claims are excluded; this is not complete callback preparation.

Behavioral fixtures cover owned registrations, hostile/secret/nil payloads, identity removal, no after-stop records, partial registration errors, cleanup retries/refusals, repeated start, new sessions, payload/session limits and `all` exclusion.

### Duplicate callback inputs

- [x] Manual `callbacks-duplicate-start <label>` reuses one active session with global and player-unit lanes, registering each exact owned closure twice. Four registration calls maximum; no extra callback identity or synthesized delivery.
- [x] Preserve both raw registration results per lane. Reserve each available call's cleanup slot before invocation, including throw-after-side-effect and false results. Guard event/unit/callback inputs before forwarding and after function guards.
- [x] `callbacks-stop` disables receiving first and attempts each pending duplicate slot once, at most four removals total. Peer lanes continue independently. Accepted outcomes retire only their corresponding slot; unconfirmed outcomes retain identity and `cleanup-incomplete`, blocking new starts.
- [x] No automatic retries beyond those slots. A later explicit stop retries only still-pending slots. Store latest raw removal result and attempt count per slot in fixed-size records; do not infer deduplication/reference-count semantics from false or error outcomes.
- [x] Preserve normal `callbacks-start`/`callbacks-stop` output and behavior, ten saved sessions, one active session, 128 payloads, 16 result positions and 128-byte labels. Fixture-driven local deliveries prove recorder bounds only; native execution and callback semantics remain unverified.

## Pure mapvalues capture

- [x] Manual `/apicontract mapvalues <label>` is excluded from `all`; query the actual accessible global with a callback followed by a fixed vararg tuple, never a guessed table signature.
- [x] Nine cases cover zero/one/multiple inputs, interior/trailing nils, fixed one/multiple/nil/zero callback returns and an opaque callback throw. Preserve callback invocation sequence, each invocation's arity/positions and outer return arity/scalars without asserting native traversal or packing.
- [x] Check accessibility before inspecting callback arguments or outer results; objects stay opaque, errors are not stringified, and callback outputs are ordinary fixed literals independent of observed arguments. Missing/restricted APIs fail closed.
- [x] Retain at most 16 positions per tuple with explicit truncation and 32 callback invocations shared across a snapshot. Excess invocations throw opaquely, mark `invocation-limit` and prevent later cases even when the mapper swallows the error. Reject late callbacks after case completion; callbacks never recurse. Preserve existing string/storage bounds. No recorder can terminate an arbitrary mapper that loops while swallowing callback errors.
- [x] `tests/mapvalues.lua` loads the actual TOC/slash handler and contrasts individual-value, whole-tuple and reversed fake mappers, nil/multiple returns, hostile/restricted values, errors, tuple/invocation/storage limits and manual-only routing.

Pinned retail `FrameScriptDocumentation.lua` declares `mapvalues(func, values...)` with strided input/output values. `Blizzard_SharedXMLGame/Tooltip/TooltipDataHandler.lua` forwards mapped tooltip arguments, while `Blizzard_AuraContainer/Blizzard_AuraContainerUtil.lua` discards validation-callback results. Native execution, nil/order/packing/error contracts and security semantics remain pending; fixture behavior earns no native credit.

## Publication and event recording

- [x] Capture raw and ordinary lookup observations for configured enum, constant and API paths, distinguishing missing parents from missing members.
- [x] Capture configured CVars through `C_CVar.GetCVar` and `GetCVarDefault` without writes.
- [x] Register configured events only on explicit `events-start`; retain registration errors and bounded positional payloads with nil slots.
- [x] Stop listeners with `events-stop`; retain build/source provenance, scenario labels and overflow counts.

`AuditTargets.lua` contains 199 publication/CVar targets and 88 event-registration targets from the blocker snapshot. These are recording capabilities, not 287 complete behavioral probes. Native transitions, earlier builds, load phases, producers and restricted payloads remain unresolved. Configuration processing is bounded to 256 targets per category; events to 256 and payloads to eight values. Passive event payloads use raw table inspection without invoking their methods or `__index`.

Runtime `d959372a3` passes 10/10 fixtures (exact-hash reuse) plus 21/21 independent supplemental checks. Proof: `/tmp/verify-all-probes-recorders-corrected-ledger.json`. The initial passive-inspection bug is preserved by regression `5870cfa46`; no native API credits follow from recorder tests.

## How it works

- [Capture protocol](../addons/ApiContractProbe/README.md)

## Implementation inventory

- `docs/addons/ApiContractProbe/ApiContractProbe.lua`: shared observation and manual curve, sex, name, numeric, cast, publication and event captures.
- `docs/addons/ApiContractProbe/ApiContractProbe.toc`: manual addon and SavedVariables registration.
- `docs/addons/ApiContractProbe/AuditTargets.lua`: exact snapshot-derived capture targets and source hash.

## Tests asserting this spec

`docs/addons/ApiContractProbe/tests/harness.lua`. Initial RED at `e2adbb57d`: addon not yet present; exit 1. Logs `/tmp/api-contract-probe-red.stdout` and `/tmp/api-contract-probe-red.stderr`. Runtime `a055b98c9` passes 8/8 local fixtures. Independent `/tmp/verify-api-contract-probe-ledger.json` reuses exact-hash fixture proof and passes 14/14 supplemental checks, including invalid indices, read-only points, raw values, access filtering and real addon wiring. This proves the recorder, not native API semantics.

Name fixtures exercise differing modified/unmodified names, nil/empty/explicit realms, positional nils and zero returns, all fixed tokens without existence gating, redacted secrets/inaccessible values, opaque errors, missing APIs and `all` inclusion. These are recorder tests, not native name/realm expectations.

Numeric fixtures assert the literal 25-input corpus against deliberately different fake clients, locale bytes, multiple/nil/zero returns, redacted results, opaque errors and missing/failed access APIs. They do not establish native rounding rules.

Cast fixtures cover distinct tenth/eleventh IDs, interior/trailing nils, empowerment flags/stage counts, repeated and consecutive manual captures, zero-result idle states, restricted/opaque errors, hostile objects, missing access/APIs and 16-position bounds. Tests assert recording behavior only.

## Known gaps (current cycle)

- [ ] Native numeric formatting corpora on matching builds/locales; nonfinite/coercion/security behavior and values outside the finite corpus remain outside this bounded recorder.
- [ ] Native captures on a matching client, including independently identified same-realm and cross-realm party fixtures for names.

## Out of scope

Installation, native execution, inferred numeric mappings or vector contracts, secret access bypass, arbitrary object serialization and gameplay manipulation. No API audit credit from local recorder fixtures.

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

## Manual explicit power observations

`/apicontract explicit-power <label>` is excluded from `all`. For `player` and `target`, it reads only published `Enum.PowerType.Mana`, `Rage` and `Energy`, without numeric fallback. Each unit/type receives independent `UnitPower` and `UnitPowerMax` calls with `unmodified=false/true`, and `UnitPowerPercent` calls with exactly four arguments, including an explicit nil curve.

The recorder makes at most 36 calls per snapshot and retains ten snapshots, 16 return positions, 256-byte scalar strings and 128-byte labels. Unit/type accessibility is rechecked after global lookup and function guards; inaccessible results remain opaque. Existing `resources` behavior is unchanged. No resource mutation, curve experiment, power-scale conclusion, restricted-context experiment or native-conformance credit follows. Pinned `UnitDocumentation.lua:2643–2767` and `PowerTypeConstantsDocumentation.lua:18–42` establish the call shapes and publication names only.

Eight actual TOC/slash fixtures cover recorder mechanics:

```text
luajit docs/addons/ApiContractProbe/tests/explicit_power.lua docs/addons/ApiContractProbe
```
