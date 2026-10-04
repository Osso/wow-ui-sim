# 12.0.7 bounded one-author batch plan

No execution authorized by this preparation. Ranking is certainty/expected scope, not measured runtime or a claim of pass. Reuse evidence first; then small explicit-state getters; then lifecycle/epoch/security work. Each batch below is one independent bounded author slice (maximum six source rows). Shared group state/producer must be integrated once; neighboring groups are not blanket authorization. Historical/native uncertainties called INFERRED in triage remain exclusions from any proposed full-conformance claim.

## Immediately account from existing evidence (proposals only)

- `source-context-001` → metadata-only; capabilities `[]`; note: "Snapshot title, URL and retrieval note identify the retained excerpt only; no runtime or implementation credit."
- `source-context-003` → metadata-only; capabilities `[]`; note: "TOC and neighboring-patch resource labels are source context only; no runtime or implementation credit."
- `source-context-006` → metadata-only; capabilities `[]`; note: "Section heading and stated inventory count organize the retained excerpt only; no runtime or implementation credit."
- `source-context-024` → metadata-only; capabilities `[]`; note: "Consolidated build range and date establish source chronology only; no runtime or implementation credit."
- `source-context-026` → metadata-only; capabilities `[]`; note: "Section heading and stated inventory count organize the retained excerpt only; no runtime or implementation credit."
- `global api-C_PartyInfo-ConfirmReadyCheck-038` → proposed bounded-coverage ONLY for `ready-check-lockdown`; note: "Inherited 12.0.5 capability ready-check-lockdown is available through retail-12-0-7 -> retail-12-0-5; bounded ready behavior only, with recorded native/strict-epoch limitations unchanged." Do not automatically close the entire statement; validate scope and existing ledger bytes before promotion.
- `global api-C_PartyInfo-DoReadyCheck-040` → proposed bounded-coverage ONLY for `ready-check-lockdown`; note: "Inherited 12.0.5 capability ready-check-lockdown is available through retail-12-0-7 -> retail-12-0-5; bounded ready behavior only, with recorded native/strict-epoch limitations unchanged." Do not automatically close the entire statement; validate scope and existing ledger bytes before promotion.
- `source-context-063` → metadata-only; capabilities `[]`; note: "Section heading and stated inventory count organize the retained excerpt only; no runtime or implementation credit."
- `source-context-082` → metadata-only; capabilities `[]`; note: "Section heading and stated inventory count organize the retained excerpt only; no runtime or implementation credit."
- `scriptobjects-DurationObject-HasExpired-089` → proposed bounded-coverage ONLY for `zero-span-charge-durations`; note: "Inherited 12.0.5 capability zero-span-charge-durations is available through retail-12-0-7 -> retail-12-0-5; bounded duration-expired behavior only, with recorded native/strict-epoch limitations unchanged." Do not automatically close the entire statement; validate scope and existing ledger bytes before promotion.
- `source-context-120` → metadata-only; capabilities `[]`; note: "Section heading and stated inventory count organize the retained excerpt only; no runtime or implementation credit."
- `source-context-128` → metadata-only; capabilities `[]`; note: "Section heading and stated inventory count organize the retained excerpt only; no runtime or implementation credit."
- `source-context-144` → metadata-only; capabilities `[]`; note: "Section heading and stated inventory count organize the retained excerpt only; no runtime or implementation credit."
- `source-context-148` → metadata-only; capabilities `[]`; note: "Section heading and stated inventory count organize the retained excerpt only; no runtime or implementation credit."
- `source-context-165` → metadata-only; capabilities `[]`; note: "Section heading and stated inventory count organize the retained excerpt only; no runtime or implementation credit."
- `deprecated-api-174` → metadata-only; capabilities `[]`; note: "Section heading and stated inventory count organize the retained excerpt only; no runtime or implementation credit."

No other row is immediately promotable from this read-only inspection. In particular existing tests/occurrence statuses alone do not create an accepted capability, and later-cache declarations alone do not establish 12.0.7 policy.

## Author batches

### B01 — asset-id (1 rows)

Rows: `global api-C_UIFileAsset-GetFileID-049`.

Shared implementation/proof boundary: Current GetFileID resolves positive numeric IDs unchanged and limited listfile paths; existing tests cover concrete numeric/path/miss behavior.

Smallest work and acceptance tests: Prove numeric ID, known path normalization, unknown path, invalid numeric boundaries and exact arity; authenticate AllowedWhenUntainted separately. Limited listfile is not all client assets.

Existing test entry points: src/c_api/c_ui_file_asset.rs::tests::ui_file_asset_uses_limited_listfile_paths; tests/ui_file_assets.rs. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B02 — bnet (2 rows)

Rows: `prose-undated-021`, `global api-C_BattleNet-InviteFriend-027`.

Shared implementation/proof boundary: Invite appends to per-environment bnet_friends, ignoring empty/duplicate inputs. Registration and happy-path social state exist; not native Battle.net service proof.

Smallest work and acceptance tests: Prove offline friend insertion/query, duplicates, empty input, independent environments, exact arity and migration wrapper under 12.0.7. Secret argument and service failure behavior remain unproven.

Existing test entry points: src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_0_7_safe_global_bridges. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B03 — bags (1 rows)

Rows: `global api-C_Container-CalculateTotalNumberOfFreeBagSlots-028`.

Shared implementation/proof boundary: Free-slot total already reads bag state; hidden-bag and family eligibility are qualified, not established by occurrence status.

Smallest work and acceptance tests: Prove two bags with populated/empty slots, capacity change, missing bag, no mutation and profile availability. INFERRED: which hidden/family bags participate.

Existing test entry points: src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_0_7_safe_global_bridges. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B04 — delves (2 rows)

Rows: `global api-C_DelvesUI-GetDelveEntranceTitleString-029`, `global api-C_DelvesUI-GetWorldTierDifficultyForActivePlayer-030`.

Shared implementation/proof boundary: Existing seeded entrance text and world-tier state bridge; current declaration can be newer than the source.

Smallest work and acceptance tests: Prove configured title/tier, changed state, miss and exact return tuple; retain localization/active-player service limits. Do not infer full delve-state behavior from a seeded label.

Existing test entry points: src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_0_7_safe_global_bridges. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B05 — cvars (6 rows)

Rows: `cvars-assistedCombatReduceHighlights-166`, `cvars-developerLogFilterDebug-167`, `cvars-developerLogFilterError-168`, `cvars-developerLogFilterFatal-169`, `cvars-developerLogFilterNormal-170`, `cvars-developerLogFilterSpam-171`.

Shared implementation/proof boundary: The six named additions exist in explicit CVar defaults; declaration generated Lua does not normally enumerate CVars. Defaults are simulator choices, not native historical evidence.

Smallest work and acceptance tests: Prove all six through public GetCVar/GetCVarBool, non-default SetCVar round trips, reset and independent environments, plus 12.0.7 publication gate. INFERRED: default values and persistence/flags; missing 14 additions/5 removals remain separate blocked statement.

Existing test entry points: src/cvars.rs::tests; tests/set_cvar_global.rs. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B06 — clock (6 rows)

Rows: `global api-C_DurationUtil-CreateManualClock-032`, `scriptobjects-DurationClock-GetTime-083`, `scriptobjects-DurationManualClock-AdvanceTime-084`, `scriptobjects-DurationManualClock-ResetTime-085`, `scriptobjects-DurationManualClock-RewindTime-086`, `scriptobjects-DurationManualClock-SetTime-087`.

Shared implementation/proof boundary: Existing mutable clock table stores time and exposes GetTime/SetTime/AdvanceTime/RewindTime/ResetTime; cached manual clock is declared userdata with AllowedWhenUntainted inputs.

Smallest work and acceptance tests: Prove initial time, fractional forward/backward/reset, independent clocks and exact arity. Preserve explicit clock state; do not silently claim userdata identity or authenticated secret-input parity. INFERRED: negative/nonfinite/default-input policy.

Existing test entry points: src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_0_7_duration_objects_and_text_binding; tests/duration_core.rs. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B07 — duration-clock (4 rows)

Rows: `scriptobjects-DurationObject-GetClock-088`, `scriptobjects-DurationObject-HasStarted-090`, `scriptobjects-DurationObject-IsActive-091`, `scriptobjects-DurationObject-SetClock-092`.

Shared implementation/proof boundary: Duration proxy already stores clock handle and uses duration-core queries. Addition chronology is separate from existing implementation.

Smallest work and acceptance tests: Exercise same object before/after clock replacement, future/active/expired boundaries and clock movement, rooted clock identity and independent environments; authenticate secret clock/time arguments separately.

Existing test entry points: src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_0_7_duration_objects_and_text_binding; tests/duration_core.rs. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B08 — binding-storage / slice 1 (6 rows)

Rows: `scriptobjects-DurationTextBinding-CanUpdateFontString-094`, `scriptobjects-DurationTextBinding-Disable-095`, `scriptobjects-DurationTextBinding-Enable-096`, `scriptobjects-DurationTextBinding-GetDuration-097`, `scriptobjects-DurationTextBinding-GetExpiredText-098`, `scriptobjects-DurationTextBinding-GetFontString-099`.

Shared implementation/proof boundary: Binding stores duration, font string, enabled flag and configuration fields; existing setters/getters and enable/disable hooks are real bounded state, not full native formatting proof.

Smallest work and acceptance tests: Prove non-default setter/getter round trips, nil/default handling, font-string identity, object isolation and exact arity; for enable/disable verify actual tick suppression/resumption. INFERRED: validation/default policy not pinned historically.

Existing test entry points: src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_0_7_duration_objects_and_text_binding; tests/duration_text_binding_copy.rs; tests/duration_text_binding_tick.rs. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B09 — binding-storage / slice 2 (6 rows)

Rows: `scriptobjects-DurationTextBinding-GetTimeModifier-101`, `scriptobjects-DurationTextBinding-GetUpdateInterval-102`, `scriptobjects-DurationTextBinding-GetZeroDurationText-103`, `scriptobjects-DurationTextBinding-IsEnabled-104`, `scriptobjects-DurationTextBinding-SetDuration-105`, `scriptobjects-DurationTextBinding-SetExpiredText-106`.

Shared implementation/proof boundary: Binding stores duration, font string, enabled flag and configuration fields; existing setters/getters and enable/disable hooks are real bounded state, not full native formatting proof.

Smallest work and acceptance tests: Prove non-default setter/getter round trips, nil/default handling, font-string identity, object isolation and exact arity; for enable/disable verify actual tick suppression/resumption. INFERRED: validation/default policy not pinned historically.

Existing test entry points: src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_0_7_duration_objects_and_text_binding; tests/duration_text_binding_copy.rs; tests/duration_text_binding_tick.rs. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B10 — binding-storage / slice 3 (4 rows)

Rows: `scriptobjects-DurationTextBinding-SetFontString-107`, `scriptobjects-DurationTextBinding-SetTimeModifier-108`, `scriptobjects-DurationTextBinding-SetUpdateInterval-109`, `scriptobjects-DurationTextBinding-SetZeroDurationText-110`.

Shared implementation/proof boundary: Binding stores duration, font string, enabled flag and configuration fields; existing setters/getters and enable/disable hooks are real bounded state, not full native formatting proof.

Smallest work and acceptance tests: Prove non-default setter/getter round trips, nil/default handling, font-string identity, object isolation and exact arity; for enable/disable verify actual tick suppression/resumption. INFERRED: validation/default policy not pinned historically.

Existing test entry points: src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_0_7_duration_objects_and_text_binding; tests/duration_text_binding_copy.rs; tests/duration_text_binding_tick.rs. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B11 — party (6 rows)

Rows: `global api-C_PartyInfo-DemoteAssistant-039`, `global api-C_PartyInfo-IsGUIDInGroup-041`, `global api-C_PartyInfo-PromoteToAssistant-042`, `global api-C_PartyInfo-PromoteToLeader-043`, `global api-C_PartyInfo-SetEveryoneIsAssistant-044`, `global api-C_PartyInfo-UninviteUnit-045`.

Shared implementation/proof boundary: Current namespaced methods share live party roster/leader/assistant mutation helpers; occurrence implemented status is not a fresh acceptance gate.

Smallest work and acceptance tests: Prove before/after leader/assistant/membership/uninvite state, multiple members, duplicate/missing/invalid cases, listener-observed state where emitted, exact arity and environment isolation. INFERRED: permission, self-removal and GUID domain policies not observed natively.

Existing test entry points: src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_0_7_safe_global_bridges. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B12 — callbacks / slice 1 (6 rows)

Rows: `global api-C_PingSecure-ClearPendingPingOffScreenCallback-046`, `global api-C_PingSecure-SetPendingPingOffScreenCallback-047`, `global api-GetSecurePendingButtonCallback-055`, `global api-GetSecurePendingPingOffScreenCallback-056`, `global api-GetSecurePendingToggleRunCallback-057`, `global api-SetSecurePendingButtonCallback-059`.

Shared implementation/proof boundary: Shared PingSecure callback table implements storage/get/clear; no secure execution enforcement is inferred from its name.

Smallest work and acceptance tests: Prove callback identity replacement/clear, independent slots/environments, retained roots after collection and unchanged caller taint. Native gamepad secure-dispatch and later removed names stay outside bounded storage credit.

Existing test entry points: src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_0_7_safe_global_bridges. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B13 — callbacks / slice 2 (2 rows)

Rows: `global api-SetSecurePendingPingOffScreenCallback-060`, `global api-SetSecurePendingToggleRunCallback-061`.

Shared implementation/proof boundary: Shared PingSecure callback table implements storage/get/clear; no secure execution enforcement is inferred from its name.

Smallest work and acceptance tests: Prove callback identity replacement/clear, independent slots/environments, retained roots after collection and unchanged caller taint. Native gamepad secure-dispatch and later removed names stay outside bounded storage credit.

Existing test entry points: src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_0_7_safe_global_bridges. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B14 — tooltip (2 rows)

Rows: `prose-undated-007`, `global api-GameTooltip_AddMoneyLine-058`.

Shared implementation/proof boundary: Loaded Blizzard_GameTooltip helper, not a simulator bootstrap prefix shim, owns this Lua function. Existing loaded tests assert coin-atlas text, label order and colors with known dependency Lua errors.

Smallest work and acceptance tests: Reuse bounded loaded-helper evidence only after matching current helper/test bytes and availability under strict 12.0.7; test representative amounts, zero, label-before-money, highlight/red. No clean full-addon/layout/locale/native credit.

Existing test entry points: tests/tooltip_money_line.rs::tooltip_money_line_loaded_helper_formats_zero_and_boolean_colors; tests/tooltip_money_line.rs::tooltip_money_line_mail_enclosed_money_keeps_label_order_and_highlight; tests/tooltip_money_line.rs::tooltip_money_line_mail_unaffordable_cod_keeps_label_order_and_red. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B15 — encounter-end (2 rows)

Rows: `prose-undated-009`, `events-ENCOUNTER_END-163`.

Shared implementation/proof boundary: Current A_Admin.SimulateBossKill copies an explicitly supplied ordered encounterUnitStatus list before dispatch, or emits a fresh empty list when omitted; it does not derive all engaged bosses automatically. Source also requires non-secret fields.

Smallest work and acceptance tests: Prove multiple engaged bosses, creatureID/name/remainingHealthPercent, nonsecret fields under tainted caller, exact ENCOUNTER_END tuple, detached snapshots and empty/no-boss case. Current tests and declaration must be matched for 12.0.7 rather than accepted by occurrence status.

Existing test entry points: tests/admin_encounter_api.rs::test_boss_kill_copies_ordered_encounter_status_before_dispatch; tests/admin_encounter_api.rs::test_boss_kill_omitted_and_nil_status_are_fresh_empty_lists; tests/admin_encounter_api.rs::test_boss_kill_rejects_malformed_status_before_any_event. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B16 — colors (2 rows)

Rows: `prose-undated-010`, `global api-C_EncounterTimeline-GetEventColor-033`.

Shared implementation/proof boundary: Current encounter-event color override state and timeline delegate exist; newer timeline notification producer also exists. Existing bounded registration is not evidence for every 5-second/alpha path.

Smallest work and acceptance tests: Prove populated RGBA overrides and live changes for warnings versus timeline, one event with correct event id after commit, 5-second trigger selection and detached outputs. INFERRED: unset-color/default/trigger mapping until historical declaration established.

Existing test entry points: src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_0_7_safe_global_bridges; tests/encounter_timeline_script.rs; tests/encounter_events.rs. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B17 — event-color (1 rows)

Rows: `events-ENCOUNTER_TIMELINE_EVENT_COLOR_CHANGED-145`.

Shared implementation/proof boundary: Both valid-events gate and current timeline color-change notification producer exist; name acceptance alone is insufficient.

Smallest work and acceptance tests: Mutation -> listener -> exact eventID, state visible before dispatch, unchanged-color no-event control, warning/timeline/5-second trigger differences and alpha. Scope strict 12.0.7 separately from later timeline features.

Existing test entry points: src/event/valid_events.rs tests; tests/encounter_timeline_script.rs. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B18 — housing-names (1 rows)

Rows: `global api-C_HousingCatalog-GetCatalogCategoryAndSubcategoryNames-034`.

Shared implementation/proof boundary: Current GetCatalogCategoryAndSubcategoryNames returns one nil unconditionally despite existing category/subcategory models.

Smallest work and acceptance tests: Read names from explicit category/subcategory catalog entries selected by documented inputs; return exact documented tuple, test distinct categories/subcategories, changed labels, miss, detached/read-only state. INFERRED: missing-entry tuple and localization.

Existing test entry points: src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_0_7_safe_global_bridges; tests/housing_category_search.rs. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B19 — merchant (1 rows)

Rows: `global api-C_MerchantFrame-GetMerchantCurrencies-037`.

Shared implementation/proof boundary: Current provider always returns an empty table; no populated merchant-currency list.

Smallest work and acceptance tests: Smallest model: ordered merchant currency records using historical output fields; test two currencies, none, live merchant change, exact table tuple and detached snapshots; deprecated wrapper unpacks this list. INFERRED: order and missing-merchant policy.

Existing test entry points: src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_0_7_safe_global_bridges. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B20 — housing-door (1 rows)

Rows: `global api-C_HousingCustomizeMode-RoomConnectionSupportsDoorType-035`.

Shared implementation/proof boundary: Current provider is constant false; no populated room/door compatibility path.

Smallest work and acceptance tests: Smallest model: explicit room-connection identifier -> supported door-type set. Test supported and unsupported types, distinct connections, live updates, invalid/missing selector and no mutation. INFERRED: missing/invalid result policy; no geometry or 3D work.

Existing test entry points: src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_0_7_safe_global_bridges. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B21 — housing-floor (1 rows)

Rows: `global api-C_HousingLayout-CanSetViewedFloor-036`.

Shared implementation/proof boundary: Current provider is constant false; false on every call is not floor permission modeling.

Smallest work and acceptance tests: Smallest model: explicit available floors and can-view set on housing state. Test permitted/denied/missing floors, permission updates and read-only queries. INFERRED: ownership/mode preconditions; do not implement a guessed service.

Existing test entry points: src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_0_7_safe_global_bridges. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B22 — calendar (1 rows)

Rows: `prose-undated-016`.

Shared implementation/proof boundary: Current Mythic+ model has run-history and best-map producers; source changes their date DTO, not merely type spelling.

Smallest work and acceptance tests: Smallest model: one explicit CalendarTime completion date on each run and shared DTO encoder for all three APIs, preserving their tuple/array shapes. Test distinct dates, unknown/missing completion, historical keys absent, fresh outputs and live update. INFERRED: weekday/month indexing/timezone and missing-date semantics; obtain pinned CalendarTime fields before acceptance.

Existing test entry points: tests/c_mythic_plus_probes.rs. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B23 — vehicle-aura (1 rows)

Rows: `prose-undated-018`.

Shared implementation/proof boundary: AuraData DTO copies explicit is_from_player_or_player_pet; 12.0.5 aura-classification-public-flags proves boolean publication, not derivation from controlled vehicle ownership.

Smallest work and acceptance tests: Smallest model: aura source/caster identity plus explicit player-controlled vehicle ownership relationship; shared classification derives true for vehicle-cast aura. Test controlled/uncontrolled vehicle, player/pet/nonplayer controls, ownership changes and plain boolean output across queries. INFERRED: classification snapshot versus live ownership timing.

Existing test entry points: tests/aura_table_shape.rs; tests/unit_aura_filter_query.rs. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B24 — sound-permission (1 rows)

Rows: `prose-undated-019`.

Shared implementation/proof boundary: 12.0.5 private-aura-sound-add-context denies insecure callers during active M+ regardless of not-in-combat; source 12.0.7 explicitly permits the out-of-combat M+ case, so inherited feature availability does NOT validate its old policy.

Smallest work and acceptance tests: Smallest model: existing sound registration state plus combat predicate in 12.0.7 active-M+ permission gate. Test insecure active-M+ out-of-combat succeeds, in-combat denied atomically, encounter/PvP controls and removal/root ownership unchanged. INFERRED: interaction with simultaneous encounter/PvP states; do not widen every restriction.

Existing test entry points: tests/private_aura_sound_add_context.rs; tests/private_aura_sound_removal.rs. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B25 — solo-group (1 rows)

Rows: `prose-undated-017`.

Shared implementation/proof boundary: GROUP_FORMED name exists, but explicit solo follower/delve entry -> event producer is not established by registration or 12.0.5 delve-instance-state.

Smallest work and acceptance tests: Smallest model: existing instance state + explicit solo-join transition emits GROUP_FORMED once on follower dungeon/delve entry. Test solo follower, solo delve, ordinary solo zone no-event, duplicate entry suppression, grouped control and state visible to listener. INFERRED: when repeated joins count as a new formation.

Existing test entry points: tests/delve_instance_state.rs; tests/c_party_info_probes.rs. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B26 — marker (1 rows)

Rows: `prose-undated-015`.

Shared implementation/proof boundary: 12.0.5 target-marker-macro-command accepts numeric /tm only and explicitly excludes !/~ prefixes; secure raidtarget set-unmarked is a separate producer contract.

Smallest work and acceptance tests: Smallest model: existing marker map plus explicit conditional set-unmarked action and /tm ~ parsing routed to the same operation. Test unmarked target set, already-marked no-op, clear/invalid marker, selected unit and cached secure-action consumer. INFERRED: collision/invalid-prefix policy; no credit from numeric-only 12.0.5 capability.

Existing test entry points: tests/mouse_tm_commands.rs; tests/wowforever_raid_marker_constants.rs. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B27 — important-filter (1 rows)

Rows: `source-context-004`.

Shared implementation/proof boundary: Existing AuraUtil.AuraFilters publication and IsValidFilterString grammar omit IMPORTANT; exact 12.0.7 absence/default publication still needs bounded proof and historical epoch comparison.

Smallest work and acceptance tests: Prove published AuraUtil.AuraFilters has no IMPORTANT entry after initialization and loaded vendor consumers, with ordinary filter positive controls. Only audit parser behavior if historical evidence establishes that removal also changed token acceptance; do not infer parser requirements from a table-publication claim. Historical older-profile applicability remains unproven.

Existing test entry points: tests/aura_util_surface.rs; tests/unit_aura_filter_query.rs. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B28 — asset-known (3 rows)

Rows: `prose-undated-011`, `global api-C_UIFileAsset-IsKnownFile-050`, `global api-C_UIFileAsset-IsLooseFile-051`.

Shared implementation/proof boundary: Current shared query_asset classifies selected addon files by actual canonical filesystem existence. Current declaration says known loose files need not exist or be openable, so existing tests do not establish that declaration.

Smallest work and acceptance tests: Smallest model: loader-owned known-loose asset registry separate from shipped fileID mapping; query membership without filesystem existence check. Test registered present and absent files, unknown physically present file, selected-root scope and shipped numeric/path controls. INFERRED: historical 12.0.7 registry population; obtain pinned declaration before asserting this later-cache policy.

Existing test entry points: tests/ui_file_assets.rs::loose_sound_is_known_during_load_and_afterward_without_synthetic_id; src/c_api/c_ui_file_asset.rs::tests::ui_file_asset_uses_limited_listfile_paths. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B29 — binding-factory (2 rows)

Rows: `prose-undated-022`, `global api-C_DurationUtil-CreateDurationTextBinding-031`.

Shared implementation/proof boundary: Existing userdata configuration proxy and weak tick registry; factory existence is not complete binding formatting fidelity.

Smallest work and acceptance tests: Prove 12.0.7 factory arguments/defaults, distinct objects, retained resources, update scheduling and exact return shape; separate later-only methods. Current Copy/Assign/tick work may postdate 12.0.7.

Existing test entry points: src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_0_7_duration_objects_and_text_binding; tests/duration_text_binding_copy.rs; tests/duration_text_binding_tick.rs. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B30 — binding-format (2 rows)

Rows: `scriptobjects-DurationTextBinding-CanFormatText-093`, `scriptobjects-DurationTextBinding-GetFormattedText-100`.

Shared implementation/proof boundary: CanFormatText currently returns true unconditionally; GetFormattedText has simplified numeric/default formatting and configuration-based updates, not a complete expiration/zero-duration/options formatter.

Smallest work and acceptance tests: Smallest model: binding configuration plus explicit clock, interval and elapsed schedule; format remaining value using configured expired/zero text, time modifier and one documented formatter. Test before-start/active/zero/expired, disabled/re-enabled cadence, invalid formatter failure and real FontString text. INFERRED: rounding, modifier units, expiry precedence and CanFormatText invalid-state predicate.

Existing test entry points: src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_0_7_duration_objects_and_text_binding; tests/duration_text_binding_tick.rs; tests/duration_text_binding_copy.rs. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B31 — absent-removals / slice 1 (6 rows)

Rows: `global api-BNInviteFriend-064`, `global api-ConfirmReadyCheck-068`, `global api-DemoteAssistant-069`, `global api-DoReadyCheck-070`, `global api-GetMerchantCurrencies-071`, `global api-IsGUIDInGroup-072`.

Shared implementation/proof boundary: Current startup removal test asserts this legacy global is nil; full-load deprecated wrapper behavior and historical old-profile availability remain unproved.

Smallest work and acceptance tests: Prove strict 12.0.7 init absence, loaded wrapper on/off if historically shipped, old-profile positive control and exact forwarding tuple; no later-cache absence -> historical absence inference.

Existing test entry points: src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_0_7_removal_and_event_surface. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B32 — absent-removals / slice 2 (5 rows)

Rows: `global api-PromoteToAssistant-073`, `global api-PromoteToLeader-074`, `global api-SetEveryoneIsAssistant-075`, `global api-GetAutoCompletePresenceID-077`, `global api-IsRecognizedName-080`.

Shared implementation/proof boundary: Current startup removal test asserts this legacy global is nil; full-load deprecated wrapper behavior and historical old-profile availability remain unproved.

Smallest work and acceptance tests: Prove strict 12.0.7 init absence, loaded wrapper on/off if historically shipped, old-profile positive control and exact forwarding tuple; no later-cache absence -> historical absence inference.

Existing test entry points: src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_0_7_removal_and_event_surface. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B33 — removals / slice 1 (6 rows)

Rows: `global api-C_ClickBindings-GetStringFromModifiers-065`, `global api-C_ClickBindings-MakeModifiers-066`, `global api-C_Spell-GetMawPowerBorderAtlasBySpellID-067`, `global api-UninviteUnit-076`, `global api-GetAutoCompleteResults-078`, `global api-GetAutoCompleteRealms-079`.

Shared implementation/proof boundary: Source removes native entry point but separately records deprecated wrappers. Simulator still registers several legacy entries; absence cannot be inferred from current stub/method inventories.

Smallest work and acceptance tests: Smallest change: profile-aware native publication boundary plus existing vendor deprecated wrappers controlled by loadDeprecationFallbacks, not unconditional deletion of compatibility aliases. Test old-profile positive control, strict 12.0.7 init native absence, loaded wrapper on/off, dispatch/return tuple and no loader resurrection. INFERRED: exact removal boundary until historical native-versus-wrapper evidence. Existing 12.0.5 spell-maw-powers capability proves the OLD function, not its removal.

Existing test entry points: src/loader/tests/wow_api_globals/startup_globals.rs; tests/spell_maw_powers.rs; tests/click_binding_spell_identifier.rs. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

Dependency: distinguish native deletion from vendor deprecated-wrapper load policy; public API absence and wrapper return shape need separate proof.

### B34 — removals / slice 2 (4 rows)

Rows: `deprecated-api-175`, `deprecated-api-176`, `deprecated-api-177`, `deprecated-api-178`.

Shared implementation/proof boundary: Source removes native entry point but separately records deprecated wrappers. Simulator still registers several legacy entries; absence cannot be inferred from current stub/method inventories.

Smallest work and acceptance tests: Smallest change: profile-aware native publication boundary plus existing vendor deprecated wrappers controlled by loadDeprecationFallbacks, not unconditional deletion of compatibility aliases. Test old-profile positive control, strict 12.0.7 init native absence, loaded wrapper on/off, dispatch/return tuple and no loader resurrection. INFERRED: exact removal boundary until historical native-versus-wrapper evidence. Existing 12.0.5 spell-maw-powers capability proves the OLD function, not its removal.

Existing test entry points: src/loader/tests/wow_api_globals/startup_globals.rs; tests/spell_maw_powers.rs; tests/click_binding_spell_identifier.rs. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

Dependency: distinguish native deletion from vendor deprecated-wrapper load policy; public API absence and wrapper return shape need separate proof.

### B35 — minimap-removals (6 rows)

Rows: `widgets-Minimap-SetBlipTexture-121`, `widgets-Minimap-SetCorpsePOIArrowTexture-122`, `widgets-Minimap-SetIconTexture-123`, `widgets-Minimap-SetPOIArrowTexture-124`, `widgets-Minimap-SetPlayerTexture-125`, `widgets-Minimap-SetStaticPOIArrowTexture-126`.

Shared implementation/proof boundary: Six removed methods are still in the simulator map-frame method registry; actual method-call publication and advertised metatable differ in this simulator.

Smallest work and acceptance tests: Gate these six native methods at the 12.0.7 boundary, preserving older profiles only. Test actual calls and metatable visibility before/after full load, positive old-profile controls and no unrelated minimap regression. INFERRED: historical deprecation-wrapper availability; current cache absence alone is not sufficient historical proof.

Existing test entry points: src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_0_7_widget_compatibility_surface; src/lua_api/frame/methods/map_frames.rs. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B36 — font-validation (5 rows)

Rows: `widgets-EditBox-SetFont-133`, `widgets-Font-SetFont-134`, `widgets-FontString-SetFont-135`, `widgets-MessageFrame-SetFont-136`, `widgets-SimpleHTML-SetFont-142`.

Shared implementation/proof boundary: Current font setters accept unresolved paths and unchecked heights; 12.0.5 fontstring-setfont-shape explicitly excludes asset and height validation, so it cannot cover this 12.0.7 delta.

Smallest work and acceptance tests: Smallest model: existing font asset resolver + explicit validated height before changing path/height/flags, shared across five receiver kinds without altering vendor Lua. Test known valid asset/nonintegral height, unknown asset, invalid/nonfinite height, unchanged state on failure and exact result tuple per receiver. INFERRED: allowable bounds, nil/flags/error policy and FontAsset domain require historical/native evidence.

Existing test entry points: tests/font_api.rs; tests/font_api.rs. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B37 — button-aspect (4 rows)

Rows: `widgets-Button-GetButtonState-129`, `widgets-Button-IsEnabled-130`, `widgets-Button-SetButtonState-131`, `widgets-Button-SetEnabled-132`.

Shared implementation/proof boundary: Current button getters push plain strings/bools; setters decode ordinary inputs and do not visibly attach ButtonState secret aspect. New delta is behavior, not documentation-only.

Smallest work and acceptance tests: Smallest model: existing frame button state/enabled fields plus per-frame ButtonState secret aspect; authenticate secret arguments before mutation, mark aspect on accepted input, wrap getters only when aspect is secret. Test actual VM secrets with secure/tainted callers, secret getter identity, denied atomicity, enable/disable callbacks and other-frame controls. INFERRED: aspect reset/public-overwrite lifecycle until native evidence.

Existing test entry points: src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_0_7_widget_compatibility_surface; tests/forbidden_aspect_creation.rs; tests/security_api.rs. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

Dependency: authenticate and retain actual VM secret identity; deny before lookup/mutation. Do not substitute host flags for secret-value proof.

### B38 — scroll-aspect (4 rows)

Rows: `widgets-ScrollFrame-GetHorizontalScroll-138`, `widgets-ScrollFrame-GetVerticalScroll-139`, `widgets-ScrollFrame-SetHorizontalScroll-140`, `widgets-ScrollFrame-SetVerticalScroll-141`.

Shared implementation/proof boundary: Current scroll getter emits plain numeric offset; setter coerces numeric input and commits before callbacks. New secret/aspect paths are not proven by public scrolling tests.

Smallest work and acceptance tests: Smallest model: existing offsets plus per-frame ScrollOffset aspect; VM-authenticate both setters before commit, mark accepted secret offset and secret-wrap both getters; retain callback order. Test both axes, secret secure/tainted matrix, no-mutation denial, public overwrite/reset, scripts observing post-commit state and frame isolation. INFERRED: aspect clearing lifecycle.

Existing test entry points: tests/scroll_widgets.rs::test_scrollframe_offsets_round_trip_outside_explicit_ranges_and_notify_after_commit; tests/scroll_widgets/script_bindings.rs; tests/security_api.rs. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

Dependency: authenticate and retain actual VM secret identity; deny before lookup/mutation. Do not substitute host flags for secret-value proof.

### B39 — chat-events / slice 1 (6 rows)

Rows: `events-CHAT_MSG_COMBAT_FACTION_CHANGE-149`, `events-CHAT_MSG_COMBAT_HONOR_GAIN-150`, `events-CHAT_MSG_COMBAT_MISC_INFO-151`, `events-CHAT_MSG_COMBAT_XP_GAIN-152`, `events-CHAT_MSG_CURRENCY-153`, `events-CHAT_MSG_FILTERED-154`.

Shared implementation/proof boundary: Current registration accepts these names; removing SecretInChatMessagingLockdown requires auditing payload secrecy at dispatch, not merely valid_events membership.

Smallest work and acceptance tests: Smallest model: real existing chat producer/dispatch path with event-specific public payload policy for these nine names, not blanket declassification. Test active/inactive lockdown, actual incoming-secret payload fixture, addon caller taint retained, exact tuple and an unchanged restricted event control. INFERRED: producer input classification absent native samples.

Existing test entry points: tests/chat_messaging_lockdown.rs; tests/chat_lockdown_ready_checks.rs. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

Dependency: authenticate and retain actual VM secret identity; deny before lookup/mutation. Do not substitute host flags for secret-value proof.

### B40 — chat-events / slice 2 (3 rows)

Rows: `events-CHAT_MSG_LOOT-155`, `events-CHAT_MSG_MONEY-156`, `events-CHAT_MSG_RESTRICTED-157`.

Shared implementation/proof boundary: Current registration accepts these names; removing SecretInChatMessagingLockdown requires auditing payload secrecy at dispatch, not merely valid_events membership.

Smallest work and acceptance tests: Smallest model: real existing chat producer/dispatch path with event-specific public payload policy for these nine names, not blanket declassification. Test active/inactive lockdown, actual incoming-secret payload fixture, addon caller taint retained, exact tuple and an unchanged restricted event control. INFERRED: producer input classification absent native samples.

Existing test entry points: tests/chat_messaging_lockdown.rs; tests/chat_lockdown_ready_checks.rs. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

Dependency: authenticate and retain actual VM secret identity; deny before lookup/mutation. Do not substitute host flags for secret-value proof.

### B41 — profiling (4 rows)

Rows: `prose-undated-012`, `global api-GetEventCPUUsage-052`, `global api-GetFunctionCPUUsage-053`, `global api-GetScriptCPUUsage-054`.

Shared implementation/proof boundary: Shared temporary metric defaults return constant values; callable functions do not measure event/function/script CPU use.

Smallest work and acceptance tests: Smallest model: per-environment explicit cumulative CPU counters keyed by event/function/frame-script, with reset/snapshot semantics matching historical signatures; prove nonzero counters, independent keys, reset, cumulative versus last-call behavior and exact tuple. INFERRED: counter units/reset/attribution; require historical/native policy before full parity.

Existing test entry points: src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_0_7_safe_global_bridges. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B42 — event-url (1 rows)

Rows: `events-URL_TEXTURE_REQUEST_RESULT-146`.

Shared implementation/proof boundary: Event name is registerable, but no corresponding URL-texture request-result producer was established.

Smallest work and acceptance tests: Smallest model: explicit URL texture request state -> one terminal result event with documented payload; test success/failure/cancelled/no-duplicate paths and state visible in listener. INFERRED: asynchronous completion/error policies; no live network required for deterministic provider tests.

Existing test entry points: src/event/valid_events.rs::tests::url_texture_request_result_is_registerable; src/event/valid_events.rs::tests::patch_12_0_7_events_are_registerable. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B43 — identity-defaults (1 rows)

Rows: `prose-undated-008`.

Shared implementation/proof boundary: Shared identity/vitals/aura providers exist but 12.0.5 instanced-identity secrecy proof does not cover unsupported-token nil/default behavior across all these APIs.

Smallest work and acceptance tests: Smallest model: shared unit-token resolution yields explicit unsupported/not-present result; each public API returns its documented nil/default tuple instead of raising for that condition, without masking malformed arguments or secret permission denial. Test UnitGUID, all named aura/vitals entry points, supported control, unsupported PvP token and secret/taint controls. INFERRED: exact unsupported-token set and per-API default tuple.

Existing test entry points: tests/instanced_identity.rs; tests/retail_unit_queries.rs; tests/unit_aura_filter_query.rs. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

### B44 — strata (1 rows)

Rows: `prose-undated-020`.

Shared implementation/proof boundary: Current SetFrameStrata decodes a string then applies protected-state checks; public tests do not prove that authentic secrets no longer raise.

Smallest work and acceptance tests: Smallest model: existing strata enum with VM-authenticated secret selector before conversion and unchanged protected-state permission checks. Test authentic secret valid token, invalid/missing public token, clean/tainted callers, protected in combat, actual resulting strata and child propagation. INFERRED: precise AllowedWhenUntainted/Always policy; page bugfix alone does not authenticate it.

Existing test entry points: tests/xml_frame_strata.rs; tests/security_api.rs. Read/extend concrete assertions, not symbol-presence or implementation-shape tests.

Dependency: authenticate and retain actual VM secret identity; deny before lookup/mutation. Do not substitute host flags for secret-value proof.

## Evidence-blocked rows (not implementation batches)

- `prose-undated-013`: Missing evidence: pinned 12.0.7 limited-input budget consumption, authentic gamepad-action context and locked/focus predicates, and unchanged caller-taint behavior. Later declarations identify four entry points but cannot prove the historical policy. Then model explicit action context and focus state, deny in each forbidden/protected/combat case, prove budget accounting, unchanged taint and real input side effects. No speculative generic input API.
- `prose-undated-014`: Missing evidence: native definition of accessed-secret state (current function versus caller, scope/lifetime/reset) and VM exposure of that history for debugstack/debuglocals. Do not replace it with global taint; future proof requires nested-frame access/no-access controls and rooted secret results.
- `global api-C_QuestHub-GetDragonridingRacesForAreaPOI-048`: Missing historical GetDragonridingRacesForAreaPOI argument and result declaration/content schema. Obtain 12.0.7 declaration or native populated return before modeling areaPOI -> ordered race records; empty array alone gives no populated-state evidence.
- `scriptobjects-DurationTextFormattingOptions-GetAddRemainingText-111`: Missing evidence: historical 12.0.7 type/factory reachability, receiver identity, defaults and method declarations. Do not invent CreateDurationTextFormattingOptions/CreateDurationTextRawValue. If authenticated, smallest model would store duration-type/addRemaining flags or one seconds scalar with ms conversion; test factory reachability, non-default round trips and object independence.
- `scriptobjects-DurationTextFormattingOptions-GetDurationType-112`: Missing evidence: historical 12.0.7 type/factory reachability, receiver identity, defaults and method declarations. Do not invent CreateDurationTextFormattingOptions/CreateDurationTextRawValue. If authenticated, smallest model would store duration-type/addRemaining flags or one seconds scalar with ms conversion; test factory reachability, non-default round trips and object independence.
- `scriptobjects-DurationTextFormattingOptions-SetAddRemainingText-113`: Missing evidence: historical 12.0.7 type/factory reachability, receiver identity, defaults and method declarations. Do not invent CreateDurationTextFormattingOptions/CreateDurationTextRawValue. If authenticated, smallest model would store duration-type/addRemaining flags or one seconds scalar with ms conversion; test factory reachability, non-default round trips and object independence.
- `scriptobjects-DurationTextFormattingOptions-SetDurationType-114`: Missing evidence: historical 12.0.7 type/factory reachability, receiver identity, defaults and method declarations. Do not invent CreateDurationTextFormattingOptions/CreateDurationTextRawValue. If authenticated, smallest model would store duration-type/addRemaining flags or one seconds scalar with ms conversion; test factory reachability, non-default round trips and object independence.
- `scriptobjects-DurationTextRawValue-GetMilliseconds-115`: Missing evidence: historical 12.0.7 type/factory reachability, receiver identity, defaults and method declarations. Do not invent CreateDurationTextFormattingOptions/CreateDurationTextRawValue. If authenticated, smallest model would store duration-type/addRemaining flags or one seconds scalar with ms conversion; test factory reachability, non-default round trips and object independence.
- `scriptobjects-DurationTextRawValue-GetSeconds-116`: Missing evidence: historical 12.0.7 type/factory reachability, receiver identity, defaults and method declarations. Do not invent CreateDurationTextFormattingOptions/CreateDurationTextRawValue. If authenticated, smallest model would store duration-type/addRemaining flags or one seconds scalar with ms conversion; test factory reachability, non-default round trips and object independence.
- `scriptobjects-DurationTextRawValue-SetMilliseconds-117`: Missing evidence: historical 12.0.7 type/factory reachability, receiver identity, defaults and method declarations. Do not invent CreateDurationTextFormattingOptions/CreateDurationTextRawValue. If authenticated, smallest model would store duration-type/addRemaining flags or one seconds scalar with ms conversion; test factory reachability, non-default round trips and object independence.
- `scriptobjects-DurationTextRawValue-SetSeconds-118`: Missing evidence: historical 12.0.7 type/factory reachability, receiver identity, defaults and method declarations. Do not invent CreateDurationTextFormattingOptions/CreateDurationTextRawValue. If authenticated, smallest model would store duration-type/addRemaining flags or one seconds scalar with ms conversion; test factory reachability, non-default round trips and object independence.
- `widgets-ModelSceneActorBase-GetModelUnitGUID-137`: Missing evidence: historical receiver binding/identity query contract (unbound/nil, unit ownership and GUID access). Once established, a binding token/GUID-only state model may implement this without 3D; test public GUID in classified/unclassified contexts, unbound actor and no rendering dependency. Do not authorize any 3D work.
- `events-CLUB_MEMBER_ADDED-158`: Missing evidence: 12.0.7 opaque-member-ID representation, identity equality/lifetime and concrete membership-event payloads. If proven numeric alias only, use metadata-only note; otherwise model explicit club/member opaque identity and drive real mutations -> event tuple.
- `events-CLUB_MEMBER_PRESENCE_UPDATED-159`: Missing evidence: 12.0.7 opaque-member-ID representation, identity equality/lifetime and concrete membership-event payloads. If proven numeric alias only, use metadata-only note; otherwise model explicit club/member opaque identity and drive real mutations -> event tuple.
- `events-CLUB_MEMBER_REMOVED-160`: Missing evidence: 12.0.7 opaque-member-ID representation, identity equality/lifetime and concrete membership-event payloads. If proven numeric alias only, use metadata-only note; otherwise model explicit club/member opaque identity and drive real mutations -> event tuple.
- `events-CLUB_MEMBER_ROLE_UPDATED-161`: Missing evidence: 12.0.7 opaque-member-ID representation, identity equality/lifetime and concrete membership-event payloads. If proven numeric alias only, use metadata-only note; otherwise model explicit club/member opaque identity and drive real mutations -> event tuple.
- `events-CLUB_MEMBER_UPDATED-162`: Missing evidence: 12.0.7 opaque-member-ID representation, identity equality/lifetime and concrete membership-event payloads. If proven numeric alias only, use metadata-only note; otherwise model explicit club/member opaque identity and drive real mutations -> event tuple.
- `source-context-172`: Missing evidence: full 12.0.7 page/extract or authenticated endpoint CVar diff with all 19 missing symbols. Keep aggregate claim as one blocked row; do not invent names or count 19 fabricated rows.
