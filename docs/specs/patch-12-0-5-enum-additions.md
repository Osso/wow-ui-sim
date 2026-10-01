# Patch 12.0.5 enum additions

Publish 33 documented enum deltas across 20 subjects from [retained patch notes](../../data/patch-api/sources/12.0.5-api-changes.txt). This contract covers Lua-visible numbers and metadata, not currency rewards, transmog eligibility, or housing suggestion behavior.

## What it must do

- [x] With `retail-12-0-5` enabled, publish `Enum.CurrencyFlagsB.CurrencyBNoBonusXP = 2048`, retaining all eleven earlier members; metadata is `MinValue=1`, `MaxValue=2048`, `NumValues=12`.
- [x] Publish `Enum.TransmogIllusionFlags.AllowedRangedShieldsHoldables = 4`, retaining values `HideUntilCollected=1` and `PlayerConditionGrantsOnLogin=2`; current retail metadata is `1/4/3` (minimum/maximum/count).
- [x] Publish `Enum.HouseFinderSuggestionReason.HomeOwner = 64`, retaining seven earlier members. For 12.0.5/12.0.7, metadata is `0/64/8`. Shared 12.1 compatibility already adds `Relinquished=65`; preserve it unchanged and refresh metadata to actual members (`0/65/9` in current retail), including after post-load restoration. Do not introduce or correct `Relinquished` through this patch.
- [x] Metadata reflects actual published numeric members rather than replacing later-epoch bounds/counts with 12.0.5 values.

Documented values: cached official `CurrencyConstantsDocumentation.lua:95–113`, `TransmogSharedDocumentation.lua:46–57`, and `PlayerHousingConstantsDocumentation.lua:52–69` under `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/`. The housing cache includes later `Relinquished=128`; that addition is excluded. Patch-note names occur at retained-source lines 573–574, 590–591, and 606–607.

## How it works

- [Lua API architecture](../lua-api.md)

## Implementation inventory

- `src/c_api/patch_12_0_5_enums.rs`: numeric publication and metadata.
- `src/c_api/mod.rs`: cumulative 12.0.5 feature gate.
- `src/lua_api/env_init/enums.rs`: registration after base missing/compat enums.
- `src/ptr/compat_bootstrap.rs`: refresh affected metadata after shared 12.1 member publication, initially and after load.

## Tests asserting this spec

- `tests/patch_12_0_5_enum_additions.rs`: 20 grouped public-value/removal/metadata tests, including 17 current-retail initialization/post-load tests and three original publication controls. Original default-retail proof predates this implementation; historical/PTR branches are not claimed as executed.
- `tests/enum_diff_coverage.rs`: representative abbreviation value uses current-retail flags while preserving the old expectation elsewhere.
- `src/loader/tests/wow_api_globals/patch_12_0_0_edit_mode_unit_frame_settings_enums.rs`: current-retail unit-frame values and metadata; historical expectations retained.
- `src/loader/tests/wow_api_globals/patch_12_1_5_transmog_illusion_flags.rs`: existing default-retail publication/post-load control passes. Numerical publication is not native-client/domain parity.

Revision-scoped commands, artifact hashes, and provenance limitations are recorded in `/tmp/patch-12.0.5-enums-ledger.md`.

## Known gaps (current cycle)

The original three additions retain their bounded default-retail proof. Historical/PTR execution and full-build provenance reconciliation remain outside that proof.

### Remaining retained-source enum deltas

- [x] Current `client-retail` publishes the other 30 retained-source deltas at numeric values from the official retail cache, with `InvalidAbbreviation`, `House`, and `PersonalOnly` absent from their respective enums. Current-retail corrections are implemented; batch7 observed GREEN is 20/20; independent bounded acceptance recorded below. Historical `profile-retail` and PTR retain their existing publications.
- [x] Each newly covered enum has coherent `NumValues`, `MinValue`, and `MaxValue` derived from actual Lua-visible numeric members, both after environment initialization and after compatibility post-load restoration. Later cumulative members are permitted; no historical exact count is asserted.

The retained section at lines 569–621 contains **33 delta rows across 20 enum subjects**, including the three already tested additions. The earlier audit's 36/24 totals are unsupported by this source; no extra rows are invented. The source/proof matrix below is the durable enum accounting source. Machine-readable cache extracts remain in `/tmp/patch-12.0.5-enum-doc-contracts.json`.

Current cached numbers are cumulative retail contracts, not evidence of historical 12.0.5 numbering. The new tests deliberately use `client-retail`, not historical `profile-retail` or PTR gates. Publication/metadata proof does not establish downstream restriction, housing, loot, or transmog behavior.

Actual parent RED at `eac08bda3`: 20 tests, 7 PASS / 13 FAIL; `/tmp/patch-12.0.5-batch6-enum-red.log`, exact argv in `/tmp/patch-12.0.5-batch6-runs.json`. Failure boundaries are missing members, wrong abbreviation flags, and incoherent unit-frame metadata. Batch7 post-change runtime result is 20/20 PASS. Existing grouped tests also assert all 30 current photo statuses (Disabled insertion shifts every previous status) and current unit-frame `DebuffIconSize=19`, `BigDefensiveIconSize=21`, `BuffIconSize=22`. `IconSize` absence follows the complete current cache and existing 12.1 strict removal. The legacy default assertions are updated; historical control expectations remain.

No numeric guesses were needed for the 33 current-cache rows. Historical numeric values and native/domain semantics remain unclaimed. The earlier `Relinquished=65` publication is preserved, not corrected to cached `128`.

## Retained-source / proof matrix

Paths resolve under `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/`. Each test filter below is prefixed `patch_12_0_5_enum_additions::`. The 17 new tests assert initialization and post-load publication plus actual-member metadata; the original three retain their existing control scopes. RED status is per grouped test (a failed test can stop before later assertions), not proof of every individual row or post-change behavior.

| Source line | Enum subject | Delta | Expected public value | Exact official cached doc/member line | Test in `tests/patch_12_0_5_enum_additions.rs` | Proof level |
|---|---|---|---|---|---|---|
| 570 | `AbbreviationDataError` | `- InvalidAbbreviation` | absent (no numeric value assigned) | `LocalizationSharedDocumentation.lua:6–17 (complete enumeration)` | `abbreviation_errors_remove_invalid_abbreviation_and_publish_current_flags` | RED FAIL; bounded independent PASS (current Retail) |
| 572 | `AddOnRestrictionType` | `+ Chat` | 5 | `RestrictedActionsConstantsDocumentation.lua:31` | `addon_restrictions_publish_chat` | RED FAIL; bounded independent PASS (current Retail) |
| 574 | `CurrencyFlagsB` | `+ CurrencyBNoBonusXP` | 2048 | `CurrencyConstantsDocumentation.lua:113` | `currency_flags_publish_no_bonus_xp_and_retain_old_members` | RED PASS; bounded independent PASS (current Retail) |
| 576 | `EditModeAccountSetting` | `+ ShowTotemActionBar` | 34 | `EditModeManagerConstantsDocumentation.lua:243` | `edit_mode_account_settings_publish_totem_action_bar` | RED PASS; bounded independent PASS (current Retail) |
| 578 | `EditModeStatusTrackingBarSetting` | `+ Size` | 3 | `EditModeManagerConstantsDocumentation.lua:620` | `edit_mode_status_tracking_settings_publish_size` | RED PASS; bounded independent PASS (current Retail) |
| 580 | `EditModeSystem` | `+ TotemActionBar` | 25 | `EditModeManagerConstantsDocumentation.lua:668` | `edit_mode_systems_publish_totem_action_bar` | RED PASS; bounded independent PASS (current Retail) |
| 582 | `EditModeUnitFrameSetting` | `+ BigDefensiveIconSize` | 21 | `EditModeManagerConstantsDocumentation.lua:712` | `edit_mode_unit_frame_settings_publish_big_defensive_icon_size` | RED FAIL; bounded independent PASS (current Retail) |
| 584 | `FragmentID` | `+ FPathingDynamicLinks` | 40 | `WowCSConstantsDocumentation.lua:53` | `fragments_publish_dynamic_pathing_links_and_housing_decor_proxy_tag` | RED FAIL; bounded independent PASS (current Retail) |
| 585 | `FragmentID` | `+ TagHousingDecorProxyGameObject` | 226 | `WowCSConstantsDocumentation.lua:83` | `fragments_publish_dynamic_pathing_links_and_housing_decor_proxy_tag` | RED FAIL; bounded independent PASS (current Retail) |
| 587 | `FrameTutorialAccount` | `+ HousingEndeavorsTabSeen` | 48 | `TutorialDocumentation.lua:137` | `account_tutorials_publish_housing_endeavors_tab_seen` | RED FAIL; bounded independent PASS (current Retail) |
| 589 | `HouseExteriorWMODataFlags` | `+ HiddenUnlessOwned` | 8 | `PlayerHousingConstantsDocumentation.lua:48` | `house_exterior_flags_publish_hidden_unless_owned` | RED FAIL; bounded independent PASS (current Retail) |
| 591 | `HouseFinderSuggestionReason` | `+ HomeOwner` | 64 | `PlayerHousingConstantsDocumentation.lua:66` | `house_finder_publishes_home_owner_without_adding_later_reasons` | RED PASS; bounded independent PASS (current Retail) |
| 593 | `HousingDecorPlacementRestriction` | `+ InvalidLightOverlap` | 64 | `HousingDecorSharedDocumentation.lua:19` | `housing_decor_restrictions_publish_invalid_light_overlap` | RED FAIL; bounded independent PASS (current Retail) |
| 595 | `HousingItemToastType` | `# House -> HouseType` | 4; House absent | `HousingUIDocumentation.lua:956; 945–957 (old name absent)` | `housing_item_toasts_rename_house_to_house_type_without_old_alias` | RED FAIL; bounded independent PASS (current Retail) |
| 597 | `HousingResult` | `+ BoundToStartingArea` | 19 | `PlayerHousingConstantsDocumentation.lua:286` | `housing_results_publish_starting_area_binding_and_invalid_light_overlap` | RED PASS; bounded independent PASS (current Retail) |
| 598 | `HousingResult` | `+ InvalidLightOverlap` | 60 | `PlayerHousingConstantsDocumentation.lua:327` | `housing_results_publish_starting_area_binding_and_invalid_light_overlap` | RED PASS; bounded independent PASS (current Retail) |
| 600 | `LootMethodStyles` | `+ Mainline` | 0 | `LootConstantsDocumentation.lua:29` | `loot_method_styles_publish_mainline_without_personal_only_alias` | RED FAIL; bounded independent PASS (current Retail) |
| 601 | `LootMethodStyles` | `- PersonalOnly` | absent (no numeric value assigned) | `LootConstantsDocumentation.lua:22–31 (complete enumeration)` | `loot_method_styles_publish_mainline_without_personal_only_alias` | RED FAIL; bounded independent PASS (current Retail) |
| 603 | `PhotoSharingUploadStatus` | `+ Disabled` | 0 | `ImageSharingConstantsDocumentation.lua:57` | `photo_sharing_upload_statuses_publish_disabled` | RED FAIL; bounded independent PASS (current Retail) |
| 605 | `SurveyDeliveryMoment` | `+ EncounterEnd` | 5 | `WowSurveyConstantsDocumentation.lua:30` | `survey_delivery_moments_publish_encounter_end` | RED FAIL; bounded independent PASS (current Retail) |
| 607 | `TransmogIllusionFlags` | `+ AllowedRangedShieldsHoldables` | 4 | `TransmogSharedDocumentation.lua:55` | `illusion_flags_publish_ranged_permission_and_retain_old_members` | RED PASS; bounded independent PASS (current Retail) |
| 609 | `TransmogSituation` | `+ AllWeather` | 22 | `TransmogOutfitConstantsDocumentation.lua:320` | `transmog_situations_publish_weather_and_time_categories` | RED FAIL; bounded independent PASS (current Retail) |
| 610 | `TransmogSituation` | `+ WeatherClear` | 23 | `TransmogOutfitConstantsDocumentation.lua:321` | `transmog_situations_publish_weather_and_time_categories` | RED FAIL; bounded independent PASS (current Retail) |
| 611 | `TransmogSituation` | `+ WeatherRain` | 24 | `TransmogOutfitConstantsDocumentation.lua:322` | `transmog_situations_publish_weather_and_time_categories` | RED FAIL; bounded independent PASS (current Retail) |
| 612 | `TransmogSituation` | `+ WeatherSnow` | 25 | `TransmogOutfitConstantsDocumentation.lua:323` | `transmog_situations_publish_weather_and_time_categories` | RED FAIL; bounded independent PASS (current Retail) |
| 613 | `TransmogSituation` | `+ WeatherSand` | 26 | `TransmogOutfitConstantsDocumentation.lua:324` | `transmog_situations_publish_weather_and_time_categories` | RED FAIL; bounded independent PASS (current Retail) |
| 614 | `TransmogSituation` | `+ AllTime` | 27 | `TransmogOutfitConstantsDocumentation.lua:325` | `transmog_situations_publish_weather_and_time_categories` | RED FAIL; bounded independent PASS (current Retail) |
| 615 | `TransmogSituation` | `+ TimeMorning` | 28 | `TransmogOutfitConstantsDocumentation.lua:326` | `transmog_situations_publish_weather_and_time_categories` | RED FAIL; bounded independent PASS (current Retail) |
| 616 | `TransmogSituation` | `+ TimeDay` | 29 | `TransmogOutfitConstantsDocumentation.lua:327` | `transmog_situations_publish_weather_and_time_categories` | RED FAIL; bounded independent PASS (current Retail) |
| 617 | `TransmogSituation` | `+ TimeEvening` | 30 | `TransmogOutfitConstantsDocumentation.lua:328` | `transmog_situations_publish_weather_and_time_categories` | RED FAIL; bounded independent PASS (current Retail) |
| 618 | `TransmogSituation` | `+ TimeNight` | 31 | `TransmogOutfitConstantsDocumentation.lua:329` | `transmog_situations_publish_weather_and_time_categories` | RED FAIL; bounded independent PASS (current Retail) |
| 620 | `TransmogSituationTrigger` | `+ Weather` | 9 | `TransmogOutfitConstantsDocumentation.lua:377` | `transmog_situation_triggers_publish_weather_and_time_of_day` | RED FAIL; bounded independent PASS (current Retail) |
| 621 | `TransmogSituationTrigger` | `+ TimeOfDay` | 10 | `TransmogOutfitConstantsDocumentation.lua:378` | `transmog_situation_triggers_publish_weather_and_time_of_day` | RED FAIL; bounded independent PASS (current Retail) |

## Out of scope

- Domain semantic parity: publishing a flag does not implement its downstream domain behavior.
- Earlier profiles/epochs without `retail-12-0-5`; later `Relinquished` publication and PTR housing compatibility repairs.
- Vendor changes, broad checks, native-client probes, deployment, and push.

## Batch7 observed proof — 2026-10-01

Observed batch7 default build snapshot `c5ba89ae3d35a951cd77ca8b773b4bfc56ad9ebd`, rilua `6044544b960cd68b4b0c58bb3373412757c2caee`, compiled successfully in 34m51s. Exact argv, artifact SHA256 and referenced outputs: `/tmp/patch-12.0.5-batch7-integration-runs.json` and `/tmp/patch-12.0.5-batch7-lib-runs.json`. Independent verifier 104 report `/tmp/patch-12.0.5-batch7-independent-proof.md` was not yet available when recording these logs; no independently validated final acceptance, native parity or whole-page completion is claimed.

`patch_12_0_5_enum_additions::` PASS 20/20 (`/tmp/patch-12.0.5-batch7-integration-5.log`), representative enum control PASS 1/1 (`/tmp/patch-12.0.5-batch7-integration-6.log`); current 12.0.7 EditMode library control PASS 1/1 (`/tmp/patch-12.0.5-batch7-lib-edit-mode-current.log`). Historical 12.0.0 filter selected zero tests (`/tmp/patch-12.0.5-batch7-lib-3.log`): NOT proof. Numerical publication/removal/metadata only, not downstream semantics.

## Reconciled bounded proof — 2026-10-01

Independent report `/tmp/patch-12.0.5-enum-accounting-independent-proof.md` accepts **30/30 previously pending rows** mapped by `/tmp/patch-12.0.5-enum-accounting-refresh.md`: IDs 570, 572, 576, 578, 580, 582, 584, 585, 587, 589, 593, 595, 597, 598, 600, 601, 603, 605, 609–618, 620, 621. These are external enum data contracts, not requirements to implement downstream consumers. Seventeen grouped tests assert publication/removal/rename and actual-member metadata after initialization and compatibility post-load; three original controls retain their narrower existing scopes. Saved batch7 **20/20 PASS** is reused, not rerun.

Snapshot `c5ba89ae3d35a951cd77ca8b773b4bfc56ad9ebd`; independently bound historical binary SHA256 `159bc2f24ff76ddd9f1a29664c17b8a6972b8c09cfa8b7fe4b420f68d0143d02`. Producer/test/enum initialization/compatibility/retained-source hashes match the snapshot (exact receipts in the independent report). Today's target executable has been replaced; this is unchanged-source accounting backed by historical execution, not current-binary execution or current-HEAD build/check/fmt proof.

Current Retail only. Historical 12.0.5 numbering, PTR/all-profile/native runtime and downstream domain behavior remain unproved; no whole-page completion. The historical zero-test filter remains no proof. Earlier batch7 pending wording describes the original checkpoint, superseded only for this bounded enum contract.
