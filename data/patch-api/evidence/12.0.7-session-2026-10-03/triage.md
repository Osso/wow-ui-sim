# Retail 12.0.7 page-audit preparation

Read-only snapshot: 2026-10-03. No repository writes, builds, tests, Git mutation, agents or model CLIs. This is a preparation ledger, not completed conformance proof.

## Source scope and stable identities

12.0.5 is exhaustive for its retained full plaintext: **362 rows = 118 Blue posts + 155 Global API + 5 ScriptObjects + 10 Widgets + 6 Events + 33 Enums + 35 Structures**. Its consolidated rows correspond to individual annotation lines, not just callable symbols. IDs are lowercase section + `-` + subject with `.`/`:` replaced by `-` + `-` + one-based source line padded to at least three digits; spaces in `global api` are preserved. Example: `global api-C_ActionBar-FindSpellActionButtons-229`. Prose IDs are `prose-YYYY-MM-DD-NNN`; the date is inherited from the actual dated source heading, not retrieval date.

12.0.7 register has **no row IDs/source lines**, only 131 `direction/category/symbol/detail` occurrences. Preserve every occurrence in register order via mapping, then order seed rows by source line. Named row IDs follow the 12.0.5 rule above. The manifest's `direction:symbol` IDs are recorded in the mapping, not substituted for per-line page identities. Supplemental prose is genuinely undated, so IDs use `prose-undated-NNN` rather than inventing a post date. Extra editorial context is `source-context-NNN`; deprecated-wrapper summaries are `deprecated-api-NNN`. `source-row-map.json` records exact line/text, category and occurrence linkage. All 131 named rows have exactly one unique source-line match.

**Exhaustive retained excerpt, NOT authenticated full Wiki page.** The text identifies itself as a crawled snapshot/excerpt and admits incomplete CVar lists. Unlike 12.0.5, no separate full-extract provenance was supplied. Capture complete historical/full plaintext and reconcile IDs before declaring the actual page exhausted. No missing structure/enum rows are invented: this excerpt has no consolidated enum/structure sections; EncounterUnitStatus and CalendarTime claims remain explicit prose rows. Combined annotations in one excerpt line remain one row with multiple required semantics, not falsely fully satisfied by one link.

Seed `source_sha256` binds raw **register** bytes (the 12.0.5 coverage convention), not text bytes. Text SHA-256 is recorded below. All source rows remain audit-pending with empty capability arrays, even metadata and suggested reuse.

## Cache and proof limits

Cached declaration root: `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/`. It is a later retail cache, not pinned build 68182. Every declaration cited below **may postdate 12.0.7**; absent declarations do not prove historical absence. In particular duration binding Copy/Assign/current scheduling, UID asset nonexistence policy and current timeline producers need epoch reconciliation. No historical build is inferred from cache file mtimes. Generated declarations provide signatures/annotations, not native policy/behavior proofs. Exact declaration snippets and simulator anchors are retained in `provider-declaration-evidence.md`; current provider bytes override stale wiki statements.

Cargo.toml:118–121 explicitly has `retail-12-0-7 = ["retail-12-0-5"]`. Gate inclusion makes 12.0.5 capability implementations available under 12.0.7; it does NOT prove a changed 12.0.7 policy. Default client-retail enables 12.1.0 (line 149), so a default-profile test is not strict-epoch availability proof. Only the rows explicitly labeled ALREADY-COVERED below propose bounded inherited credit, not whole-line/native conformance. `fontstring-setfont-shape`, `aura-classification-public-flags`, `target-marker-macro-command`, `spell-maw-powers`, and `private-aura-sound-add-context` are deliberately not treated as covering their changed deltas.

Occurrence statuses 29 implemented/101 best-effort/1 exception-requested are historical inputs only. Named tests below were located/read, **not run**; no new passing or independent-proof claims. Candidate models are plans, not implementation authorization. INFERRED policies must remain qualified until historical/native evidence. No 3D implementation is proposed.

Text SHA-256: `014f7d51eca1b2fc5d76071978e09c537efd66d14069e21d163e58ccd04a561e`. Register SHA-256: `389e3b19174bf77c3646028f764cf186ccfe1b7dddaca2a3b3fcba75e3bdec60`.

## Counts

- Source context: 14
- Blue posts: 16
- Global API: 52
- ScriptObjects: 36
- Widgets: 20
- Events: 17
- CVars: 6
- Deprecated API: 5

- METADATA-ONLY: 13
- ALREADY-IMPLEMENTED-NEEDS-PROOF: 73
- MODELABLE: 59
- BLOCKED: 18
- ALREADY-COVERED-BY-12.0.5-CAPABILITY: 3

## Exact lines absent from the occurrence register

- L1 `source-context-001`: Patch 12.0.7/API changes - Warcraft Wiki source snapshot from https://warcraft.wiki.gg/wiki/Patch_12.0.7/API_changes (searched 2026-07-06).
- L3 `source-context-003`: Resources: TOC 120007. Previous patch 12.0.5. Next patch 12.1.0.
- L4 `source-context-004`: Undocumented: Removed IMPORTANT from AuraFilters.
- L6 `source-context-006`: Blue posts / notes:
- L7 `prose-undated-007`: - Added GameTooltip_AddMoneyLine API using embedded atlases/MoneyFormatter; Blizzard removed usages of SetTooltipMoney.
- L8 `prose-undated-008`: - Unit identity: APIs restricting unit token types (UnitGUID, UnitAura, health/power APIs when called with PvP-restricted tokens) no longer raise Lua errors for unsupported tokens; return nil/default.
- L9 `prose-undated-009`: - ENCOUNTER_END includes additional payload: list of EncounterUnitStatus tables for all boss units engaged; fields creatureID, creatureName, remainingHealthPercent as non-secrets.
- L10 `prose-undated-010`: - C_EncounterEvents allows configuration of different colors for text warnings/timeline events; custom color for 5 seconds remaining signaled via ENCOUNTER_TIMELINE_EVENT_COLOR_CHANGED; color APIs accept alpha.
- L11 `prose-undated-011`: - Added C_UIFileAsset namespace: GetFileID(asset), IsKnownFile(asset), IsLooseFile(asset).
- L12 `prose-undated-012`: - Profiling APIs available to addons again: GetEventCPUUsage, GetFunctionCPUUsage, GetScriptCPUUsage.
- L13 `prose-undated-013`: - SimulateMouse APIs no longer carry taint when used, restricted to gamepad action and disallowed with forbidden/locked/script-inaccessible/protected mouse foci in combat.
- L14 `prose-undated-014`: - debugstack and debuglocals return secret values if current function or caller accessed secret value.
- L15 `prose-undated-015`: - Added secure action raidtarget option "set-unmarked" and /tm ~marker syntax.
- L16 `prose-undated-016`: - C_MythicPlus.GetRunHistory, GetWeeklyBestForMap, GetSeasonBestForMap now return CalendarTime structs instead of MythicPlusDate structs.
- L17 `prose-undated-017`: - GROUP_FORMED sent when player joins a follower dungeon or delve alone.
- L18 `prose-undated-018`: - AuraData.isFromPlayerOrPlayerPet true if aura came from player-controlled vehicle.
- L19 `prose-undated-019`: - Addons allowed to call C_UnitAuras.AddPrivateAuraAppliedSound during active M+ if player not in combat.
- L20 `prose-undated-020`: - Fixed secret value errors in SetFrameStrata.
- L21 `prose-undated-021`: - BNInviteFriend migrated to C_BattleNet.InviteFriend.
- L22 `prose-undated-022`: - Added DurationTextBinding script object type.
- L24 `source-context-024`: Consolidated changes 12.0.5 (67602) -> 12.0.7 (68182) Jun 12 2026.
- L26 `source-context-026`: Global API Added (35):
- L63 `source-context-063`: Global API Removed (17):
- L82 `source-context-082`: ScriptObjects Added (36):
- L120 `source-context-120`: Widgets Removed (6):
- L128 `source-context-128`: Widget Changes:
- L144 `source-context-144`: Events Added (2):
- L148 `source-context-148`: Event Changes:
- L165 `source-context-165`: CVars Added examples from crawled page:
- L172 `source-context-172`: (20 added, 5 removed; crawler excerpt did not include full list.)
- L174 `deprecated-api-174`: Deprecated API added:
- L175 `deprecated-api-175`: 12.0.7 Deprecated_12_0_7.lua: C_ClickBindings.MakeModifiers -> MakeModifiers; C_ClickBindings.GetStringFromModifiers -> GetStringFromModifiers; C_Spell.GetMawPowerBorderAtlasBySpellID -> C_Spell.GetMawPowerRarityInfoBySpellID; GetMerchantCurrencies -> C_MerchantFrame.GetMerchantCurrencies.
- L176 `deprecated-api-176`: Deprecated_BattleNet.lua: BNInviteFriend -> C_BattleNet.InviteFriend.
- L177 `deprecated-api-177`: Deprecated_PartyInfo.lua: ConfirmReadyCheck/DemoteAssistant/DoReadyCheck/PromoteToAssistant/PromoteToLeader/SetEveryoneIsAssistant/UninviteUnit/IsGUIDInGroup -> C_PartyInfo namespace.
- L178 `deprecated-api-178`: 12.0.5 Deprecated_AutoComplete.lua wrappers: GetAutoCompletePresenceID/GetAutoCompleteResults/GetAutoCompleteRealms/IsRecognizedName -> C_AutoComplete namespace.

Blank lines contain no claims; every nonblank excerpt line has a row, including section headings as explicitly editorial metadata.

## Per-row triage

### L1 `source-context-001` — METADATA-ONLY

Source: `Patch 12.0.7/API changes - Warcraft Wiki source snapshot from https://warcraft.wiki.gg/wiki/Patch_12.0.7/API_changes (searched 2026-07-06).`

Assessment / exact one-sentence metadata note: Snapshot title, URL and retrieval note identify the retained excerpt only; no runtime or implementation credit.

Current provider: No exact provider located for this row; no provider is required for editorial metadata..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

### L3 `source-context-003` — METADATA-ONLY

Source: `Resources: TOC 120007. Previous patch 12.0.5. Next patch 12.1.0.`

Assessment / exact one-sentence metadata note: TOC and neighboring-patch resource labels are source context only; no runtime or implementation credit.

Current provider: No exact provider located for this row; no provider is required for editorial metadata..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

### L4 `source-context-004` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `Undocumented: Removed IMPORTANT from AuraFilters.`

Assessment: Existing AuraUtil.AuraFilters publication and IsValidFilterString grammar omit IMPORTANT; exact 12.0.7 absence/default publication still needs bounded proof and historical epoch comparison.

Current provider: `src/lua_api/globals/auras.rs:156`.

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: tests/aura_util_surface.rs; tests/unit_aura_filter_query.rs. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove published AuraUtil.AuraFilters has no IMPORTANT entry after initialization and loaded vendor consumers, with ordinary filter positive controls. Only audit parser behavior if historical evidence establishes that removal also changed token acceptance; do not infer parser requirements from a table-publication claim. Historical older-profile applicability remains unproven.

### L6 `source-context-006` — METADATA-ONLY

Source: `Blue posts / notes:`

Assessment / exact one-sentence metadata note: Section heading and stated inventory count organize the retained excerpt only; no runtime or implementation credit.

Current provider: No exact provider located for this row; no provider is required for editorial metadata..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

### L7 `prose-undated-007` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `- Added GameTooltip_AddMoneyLine API using embedded atlases/MoneyFormatter; Blizzard removed usages of SetTooltipMoney.`

Assessment: Loaded Blizzard_GameTooltip helper, not a simulator bootstrap prefix shim, owns this Lua function. Existing loaded tests assert coin-atlas text, label order and colors with known dependency Lua errors.

Current provider: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_GameTooltip/Mainline/GameTooltip.lua:320`.

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: tests/tooltip_money_line.rs::tooltip_money_line_loaded_helper_formats_zero_and_boolean_colors; tests/tooltip_money_line.rs::tooltip_money_line_mail_enclosed_money_keeps_label_order_and_highlight; tests/tooltip_money_line.rs::tooltip_money_line_mail_unaffordable_cod_keeps_label_order_and_red. These are located source, not fresh execution.

Model / required proof / missing evidence: Reuse bounded loaded-helper evidence only after matching current helper/test bytes and availability under strict 12.0.7; test representative amounts, zero, label-before-money, highlight/red. No clean full-addon/layout/locale/native credit.

### L8 `prose-undated-008` — MODELABLE

Source: `- Unit identity: APIs restricting unit token types (UnitGUID, UnitAura, health/power APIs when called with PvP-restricted tokens) no longer raise Lua errors for unsupported tokens; return nil/default.`

Assessment: Shared identity/vitals/aura providers exist but 12.0.5 instanced-identity secrecy proof does not cover unsupported-token nil/default behavior across all these APIs.

Current provider: `src/lua_api/globals/unit_misc.rs:144`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/UnitDocumentation.lua:1213`; **later cache: may postdate 12.0.7**.

Existing tests: tests/instanced_identity.rs; tests/retail_unit_queries.rs; tests/unit_aura_filter_query.rs. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: shared unit-token resolution yields explicit unsupported/not-present result; each public API returns its documented nil/default tuple instead of raising for that condition, without masking malformed arguments or secret permission denial. Test UnitGUID, all named aura/vitals entry points, supported control, unsupported PvP token and secret/taint controls. INFERRED: exact unsupported-token set and per-API default tuple.

### L9 `prose-undated-009` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `- ENCOUNTER_END includes additional payload: list of EncounterUnitStatus tables for all boss units engaged; fields creatureID, creatureName, remainingHealthPercent as non-secrets.`

Assessment: Current A_Admin.SimulateBossKill copies an explicitly supplied ordered encounterUnitStatus list before dispatch, or emits a fresh empty list when omitted; it does not derive all engaged bosses automatically. Source also requires non-secret fields.

Current provider: `src/lua_api/globals/admin_encounter.rs:20`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/EncounterInfoDocumentation.lua:40`; **later cache: may postdate 12.0.7**.

Existing tests: tests/admin_encounter_api.rs::test_boss_kill_copies_ordered_encounter_status_before_dispatch; tests/admin_encounter_api.rs::test_boss_kill_omitted_and_nil_status_are_fresh_empty_lists; tests/admin_encounter_api.rs::test_boss_kill_rejects_malformed_status_before_any_event. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove multiple engaged bosses, creatureID/name/remainingHealthPercent, nonsecret fields under tainted caller, exact ENCOUNTER_END tuple, detached snapshots and empty/no-boss case. Current tests and declaration must be matched for 12.0.7 rather than accepted by occurrence status.

### L10 `prose-undated-010` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `- C_EncounterEvents allows configuration of different colors for text warnings/timeline events; custom color for 5 seconds remaining signaled via ENCOUNTER_TIMELINE_EVENT_COLOR_CHANGED; color APIs accept alpha.`

Assessment: Current encounter-event color override state and timeline delegate exist; newer timeline notification producer also exists. Existing bounded registration is not evidence for every 5-second/alpha path.

Current provider: `src/lua_api/globals/missing_surface/encounter_events.rs:112`; `src/lua_api/globals/missing_surface/encounter_events.rs:125`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/EncounterEventsDocumentation.lua:11`; `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/EncounterEventsDocumentation.lua:106`; **later cache: may postdate 12.0.7**.

Existing tests: src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_0_7_safe_global_bridges; tests/encounter_timeline_script.rs; tests/encounter_events.rs. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove populated RGBA overrides and live changes for warnings versus timeline, one event with correct event id after commit, 5-second trigger selection and detached outputs. INFERRED: unset-color/default/trigger mapping until historical declaration established.

### L11 `prose-undated-011` — MODELABLE

Source: `- Added C_UIFileAsset namespace: GetFileID(asset), IsKnownFile(asset), IsLooseFile(asset).`

Assessment: Current shared query_asset classifies selected addon files by actual canonical filesystem existence. Current declaration says known loose files need not exist or be openable, so existing tests do not establish that declaration.

Current provider: `src/c_api/c_ui_file_asset.rs:55`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/UIFileAssetAPIDocumentation.lua:27`; **later cache: may postdate 12.0.7**.

Existing tests: tests/ui_file_assets.rs::loose_sound_is_known_during_load_and_afterward_without_synthetic_id; src/c_api/c_ui_file_asset.rs::tests::ui_file_asset_uses_limited_listfile_paths. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: loader-owned known-loose asset registry separate from shipped fileID mapping; query membership without filesystem existence check. Test registered present and absent files, unknown physically present file, selected-root scope and shipped numeric/path controls. INFERRED: historical 12.0.7 registry population; obtain pinned declaration before asserting this later-cache policy.

### L12 `prose-undated-012` — MODELABLE

Source: `- Profiling APIs available to addons again: GetEventCPUUsage, GetFunctionCPUUsage, GetScriptCPUUsage.`

Assessment: Shared temporary metric defaults return constant values; callable functions do not measure event/function/script CPU use.

Current provider: `src/lua_api/workarounds/temporary/performance_metric_defaults.rs:50`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:40`; **later cache: may postdate 12.0.7**.

Existing tests: src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_0_7_safe_global_bridges. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: per-environment explicit cumulative CPU counters keyed by event/function/frame-script, with reset/snapshot semantics matching historical signatures; prove nonzero counters, independent keys, reset, cumulative versus last-call behavior and exact tuple. INFERRED: counter units/reset/attribution; require historical/native policy before full parity.

### L13 `prose-undated-013` — BLOCKED

Source: `- SimulateMouse APIs no longer carry taint when used, restricted to gamepad action and disallowed with forbidden/locked/script-inaccessible/protected mouse foci in combat.`

Assessment: No exact SimulateMouse provider located in src/tests. Later InputDocumentation.lua names SimulateMouseClick/Down/Up/Wheel with RequiresLimitedInput, MouseFocusValidForLimitedInput and AllowedWhenUntainted; it does not authenticate historical 12.0.7 gamepad/taint semantics.

Current provider: No exact receiver/symbol provider located; neighboring subsystem only, no callable credit..

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/InputDocumentation.lua:266`; `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/InputDocumentation.lua:279`; `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/InputDocumentation.lua:292`; `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/InputDocumentation.lua:305`; **later cache: may postdate 12.0.7**.

Existing tests: tests/forbidden_frames.rs; tests/forbidden_aspect_creation.rs. These are located source, not fresh execution.

Model / required proof / missing evidence: Missing evidence: pinned 12.0.7 limited-input budget consumption, authentic gamepad-action context and locked/focus predicates, and unchanged caller-taint behavior. Later declarations identify four entry points but cannot prove the historical policy. Then model explicit action context and focus state, deny in each forbidden/protected/combat case, prove budget accounting, unchanged taint and real input side effects. No speculative generic input API.

### L14 `prose-undated-014` — BLOCKED

Source: `- debugstack and debuglocals return secret values if current function or caller accessed secret value.`

Assessment: Debug stack/locals surfaces exist, but statement requires current/caller secret-access history rather than ordinary output secrecy or stack taint.

Current provider: `src/lua_api/workarounds/temporary/debug_environment_defaults.rs:121`; `src/lua_api/workarounds/temporary/debug_environment_defaults.rs:153`.

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: tests/security_api.rs. These are located source, not fresh execution.

Model / required proof / missing evidence: Missing evidence: native definition of accessed-secret state (current function versus caller, scope/lifetime/reset) and VM exposure of that history for debugstack/debuglocals. Do not replace it with global taint; future proof requires nested-frame access/no-access controls and rooted secret results.

### L15 `prose-undated-015` — MODELABLE

Source: `- Added secure action raidtarget option "set-unmarked" and /tm ~marker syntax.`

Assessment: 12.0.5 target-marker-macro-command accepts numeric /tm only and explicitly excludes !/~ prefixes; secure raidtarget set-unmarked is a separate producer contract.

Current provider: `src/lua_api/globals/spell_macro_verbs.rs:189`.

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: tests/mouse_tm_commands.rs; tests/wowforever_raid_marker_constants.rs. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: existing marker map plus explicit conditional set-unmarked action and /tm ~ parsing routed to the same operation. Test unmarked target set, already-marked no-op, clear/invalid marker, selected unit and cached secure-action consumer. INFERRED: collision/invalid-prefix policy; no credit from numeric-only 12.0.5 capability.

### L16 `prose-undated-016` — MODELABLE

Source: `- C_MythicPlus.GetRunHistory, GetWeeklyBestForMap, GetSeasonBestForMap now return CalendarTime structs instead of MythicPlusDate structs.`

Assessment: Current Mythic+ model has run-history and best-map producers; source changes their date DTO, not merely type spelling.

Current provider: `src/lua_api/globals/missing_surface/mythic_plus.rs:266`; `src/lua_api/globals/missing_surface/mythic_plus.rs:325`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/MythicPlusInfoDocumentation.lua:137`; `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/TimeDocumentation.lua:6`; **later cache: may postdate 12.0.7**.

Existing tests: tests/c_mythic_plus_probes.rs. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: one explicit CalendarTime completion date on each run and shared DTO encoder for all three APIs, preserving their tuple/array shapes. Test distinct dates, unknown/missing completion, historical keys absent, fresh outputs and live update. INFERRED: weekday/month indexing/timezone and missing-date semantics; obtain pinned CalendarTime fields before acceptance.

### L17 `prose-undated-017` — MODELABLE

Source: `- GROUP_FORMED sent when player joins a follower dungeon or delve alone.`

Assessment: GROUP_FORMED name exists, but explicit solo follower/delve entry -> event producer is not established by registration or 12.0.5 delve-instance-state.

Current provider: `src/event/valid_events_a.rs:718`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:641`; **later cache: may postdate 12.0.7**.

Existing tests: tests/delve_instance_state.rs; tests/c_party_info_probes.rs. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: existing instance state + explicit solo-join transition emits GROUP_FORMED once on follower dungeon/delve entry. Test solo follower, solo delve, ordinary solo zone no-event, duplicate entry suppression, grouped control and state visible to listener. INFERRED: when repeated joins count as a new formation.

### L18 `prose-undated-018` — MODELABLE

Source: `- AuraData.isFromPlayerOrPlayerPet true if aura came from player-controlled vehicle.`

Assessment: AuraData DTO copies explicit is_from_player_or_player_pet; 12.0.5 aura-classification-public-flags proves boolean publication, not derivation from controlled vehicle ownership.

Current provider: `src/lua_api/globals/auras.rs:817`.

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: tests/aura_table_shape.rs; tests/unit_aura_filter_query.rs. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: aura source/caster identity plus explicit player-controlled vehicle ownership relationship; shared classification derives true for vehicle-cast aura. Test controlled/uncontrolled vehicle, player/pet/nonplayer controls, ownership changes and plain boolean output across queries. INFERRED: classification snapshot versus live ownership timing.

### L19 `prose-undated-019` — MODELABLE

Source: `- Addons allowed to call C_UnitAuras.AddPrivateAuraAppliedSound during active M+ if player not in combat.`

Assessment: 12.0.5 private-aura-sound-add-context denies insecure callers during active M+ regardless of not-in-combat; source 12.0.7 explicitly permits the out-of-combat M+ case, so inherited feature availability does NOT validate its old policy.

Current provider: `src/c_api/private_aura_sounds/add.rs:167`.

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: tests/private_aura_sound_add_context.rs; tests/private_aura_sound_removal.rs. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: existing sound registration state plus combat predicate in 12.0.7 active-M+ permission gate. Test insecure active-M+ out-of-combat succeeds, in-combat denied atomically, encounter/PvP controls and removal/root ownership unchanged. INFERRED: interaction with simultaneous encounter/PvP states; do not widen every restriction.

### L20 `prose-undated-020` — MODELABLE

Source: `- Fixed secret value errors in SetFrameStrata.`

Assessment: Current SetFrameStrata decodes a string then applies protected-state checks; public tests do not prove that authentic secrets no longer raise.

Current provider: `src/lua_api/frame/methods/core_state/strata_level.rs:13`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleFrameAPIDocumentation.lua:1292`; **later cache: may postdate 12.0.7**.

Existing tests: tests/xml_frame_strata.rs; tests/security_api.rs. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: existing strata enum with VM-authenticated secret selector before conversion and unchanged protected-state permission checks. Test authentic secret valid token, invalid/missing public token, clean/tainted callers, protected in combat, actual resulting strata and child propagation. INFERRED: precise AllowedWhenUntainted/Always policy; page bugfix alone does not authenticate it.

### L21 `prose-undated-021` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `- BNInviteFriend migrated to C_BattleNet.InviteFriend.`

Assessment: Invite appends to per-environment bnet_friends, ignoring empty/duplicate inputs. Registration and happy-path social state exist; not native Battle.net service proof.

Current provider: `src/c_api/c_battle_net.rs:70`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/BattleNetDocumentation.lua:211`; **later cache: may postdate 12.0.7**.

Existing tests: src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_0_7_safe_global_bridges. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove offline friend insertion/query, duplicates, empty input, independent environments, exact arity and migration wrapper under 12.0.7. Secret argument and service failure behavior remain unproven.

### L22 `prose-undated-022` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `- Added DurationTextBinding script object type.`

Assessment: Existing userdata configuration proxy and weak tick registry; factory existence is not complete binding formatting fidelity.

Current provider: `src/c_api/duration_text_binding.rs:118`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationUtilDocumentation.lua:21`; **later cache: may postdate 12.0.7**.

Existing tests: src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_0_7_duration_objects_and_text_binding; tests/duration_text_binding_copy.rs; tests/duration_text_binding_tick.rs. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove 12.0.7 factory arguments/defaults, distinct objects, retained resources, update scheduling and exact return shape; separate later-only methods. Current Copy/Assign/tick work may postdate 12.0.7.

### L24 `source-context-024` — METADATA-ONLY

Source: `Consolidated changes 12.0.5 (67602) -> 12.0.7 (68182) Jun 12 2026.`

Assessment / exact one-sentence metadata note: Consolidated build range and date establish source chronology only; no runtime or implementation credit.

Current provider: No exact provider located for this row; no provider is required for editorial metadata..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

### L26 `source-context-026` — METADATA-ONLY

Source: `Global API Added (35):`

Assessment / exact one-sentence metadata note: Section heading and stated inventory count organize the retained excerpt only; no runtime or implementation credit.

Current provider: No exact provider located for this row; no provider is required for editorial metadata..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

### L27 `global api-C_BattleNet-InviteFriend-027` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `C_BattleNet.InviteFriend`

Occurrence input: `added:C_BattleNet.InviteFriend`, historical status `implemented` (not conclusion).

Assessment: Invite appends to per-environment bnet_friends, ignoring empty/duplicate inputs. Registration and happy-path social state exist; not native Battle.net service proof.

Current provider: `src/c_api/c_battle_net.rs:70`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/BattleNetDocumentation.lua:211`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`; `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove offline friend insertion/query, duplicates, empty input, independent environments, exact arity and migration wrapper under 12.0.7. Secret argument and service failure behavior remain unproven.

### L28 `global api-C_Container-CalculateTotalNumberOfFreeBagSlots-028` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `C_Container.CalculateTotalNumberOfFreeBagSlots`

Occurrence input: `added:C_Container.CalculateTotalNumberOfFreeBagSlots`, historical status `best-effort` (not conclusion).

Assessment: Free-slot total already reads bag state; hidden-bag and family eligibility are qualified, not established by occurrence status.

Current provider: `src/c_api/item_spell/c_container.rs:63`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ContainerDocumentation.lua:11`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`; `tests/mists_action_micro_bag_status.rs:18::mists_action_micro_bag_and_status_bars_are_interactive`; `tests/blizzard_main_menu_bar_bag_buttons_loads.rs:249::blizzard_main_menu_bar_bag_buttons_auto_discovered_on_game_screen_only`; `tests/inventory_counts.rs:183::total_free_slots_use_captured_capacities_including_bag_five`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove two bags with populated/empty slots, capacity change, missing bag, no mutation and profile availability. INFERRED: which hidden/family bags participate.

### L29 `global api-C_DelvesUI-GetDelveEntranceTitleString-029` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `C_DelvesUI.GetDelveEntranceTitleString`

Occurrence input: `added:C_DelvesUI.GetDelveEntranceTitleString`, historical status `best-effort` (not conclusion).

Assessment: Existing seeded entrance text and world-tier state bridge; current declaration can be newer than the source.

Current provider: `src/lua_api/globals/missing_surface/delves_ui.rs:81`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DelvesUIDocumentation.lua:147`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove configured title/tier, changed state, miss and exact return tuple; retain localization/active-player service limits. Do not infer full delve-state behavior from a seeded label.

### L30 `global api-C_DelvesUI-GetWorldTierDifficultyForActivePlayer-030` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `C_DelvesUI.GetWorldTierDifficultyForActivePlayer`

Occurrence input: `added:C_DelvesUI.GetWorldTierDifficultyForActivePlayer`, historical status `best-effort` (not conclusion).

Assessment: Existing seeded entrance text and world-tier state bridge; current declaration can be newer than the source.

Current provider: `src/lua_api/globals/missing_surface/delves_ui.rs:104`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DelvesUIDocumentation.lua:364`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove configured title/tier, changed state, miss and exact return tuple; retain localization/active-player service limits. Do not infer full delve-state behavior from a seeded label.

### L31 `global api-C_DurationUtil-CreateDurationTextBinding-031` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `C_DurationUtil.CreateDurationTextBinding`

Occurrence input: `added:C_DurationUtil.CreateDurationTextBinding`, historical status `best-effort` (not conclusion).

Assessment: Existing userdata configuration proxy and weak tick registry; factory existence is not complete binding formatting fidelity.

Current provider: `src/c_api/duration_text_binding.rs:237`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationUtilDocumentation.lua:21`; **later cache: may postdate 12.0.7**.

Existing tests: `src/c_api/duration_text_binding.rs:353::duration_binding_availability_preserves_client_versions`; `src/loader/tests/wow_api_globals/startup_globals.rs:84::test_patch_12_0_7_duration_objects_and_text_binding`; `src/loader/tests/wow_api_globals/startup_globals.rs:184::test_patch_12_1_duration_binding_reference_lifetime_and_identity`; `src/loader/tests/wow_api_globals/startup_globals.rs:772::test_patch_12_1_safe_global_bridges`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove 12.0.7 factory arguments/defaults, distinct objects, retained resources, update scheduling and exact return shape; separate later-only methods. Current Copy/Assign/tick work may postdate 12.0.7.

### L32 `global api-C_DurationUtil-CreateManualClock-032` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `C_DurationUtil.CreateManualClock`

Occurrence input: `added:C_DurationUtil.CreateManualClock`, historical status `implemented` (not conclusion).

Assessment: Existing mutable clock table stores time and exposes GetTime/SetTime/AdvanceTime/RewindTime/ResetTime; cached manual clock is declared userdata with AllowedWhenUntainted inputs.

Current provider: `src/lua_api/globals/lua_duration_object.rs:104`; `src/lua_api/globals/lua_duration_object.rs:111`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationUtilDocumentation.lua:31`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:84::test_patch_12_0_7_duration_objects_and_text_binding`; `tests/duration_core.rs:98::duration_core_secret_inputs_store_wrappers_and_preserve_lifecycle`; `tests/duration_core.rs:147::duration_core_secret_tainted_queries_and_mutations_reject_atomically`; `tests/duration_core.rs:245::duration_percent_tracks_clock_boundaries_and_rewind`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove initial time, fractional forward/backward/reset, independent clocks and exact arity. Preserve explicit clock state; do not silently claim userdata identity or authenticated secret-input parity. INFERRED: negative/nonfinite/default-input policy.

### L33 `global api-C_EncounterTimeline-GetEventColor-033` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `C_EncounterTimeline.GetEventColor`

Occurrence input: `added:C_EncounterTimeline.GetEventColor`, historical status `best-effort` (not conclusion).

Assessment: Current encounter-event color override state and timeline delegate exist; newer timeline notification producer also exists. Existing bounded registration is not evidence for every 5-second/alpha path.

Current provider: `src/lua_api/globals/missing_surface/encounter_events.rs:24`; `src/lua_api/globals/missing_surface/encounter_events.rs:42`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/EncounterTimelineDocumentation.lua:81`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`; `tests/encounter_timeline_view.rs:14::encounter_tracks_real_blizzard_track_layout_and_view`; `tests/encounter_events.rs:36::encounter_events_color_override_round_trip_and_clear`; `tests/encounter_events.rs:101::encounter_events_invalid_ids_are_ignored`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove populated RGBA overrides and live changes for warnings versus timeline, one event with correct event id after commit, 5-second trigger selection and detached outputs. INFERRED: unset-color/default/trigger mapping until historical declaration established.

### L34 `global api-C_HousingCatalog-GetCatalogCategoryAndSubcategoryNames-034` — MODELABLE

Source: `C_HousingCatalog.GetCatalogCategoryAndSubcategoryNames`

Occurrence input: `added:C_HousingCatalog.GetCatalogCategoryAndSubcategoryNames`, historical status `best-effort` (not conclusion).

Assessment: Current GetCatalogCategoryAndSubcategoryNames returns one nil unconditionally despite existing category/subcategory models.

Current provider: `src/c_api/c_housing.rs:348`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/HousingCatalogUIDocumentation.lua:92`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`. These are located source, not fresh execution.

Model / required proof / missing evidence: Read names from explicit category/subcategory catalog entries selected by documented inputs; return exact documented tuple, test distinct categories/subcategories, changed labels, miss, detached/read-only state. INFERRED: missing-entry tuple and localization.

### L35 `global api-C_HousingCustomizeMode-RoomConnectionSupportsDoorType-035` — MODELABLE

Source: `C_HousingCustomizeMode.RoomConnectionSupportsDoorType`

Occurrence input: `added:C_HousingCustomizeMode.RoomConnectionSupportsDoorType`, historical status `best-effort` (not conclusion).

Assessment: Current provider is constant false; no populated room/door compatibility path.

Current provider: `src/c_api/c_housing.rs:354`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/HousingCustomizeModeUIDocumentation.lua:314`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: explicit room-connection identifier -> supported door-type set. Test supported and unsupported types, distinct connections, live updates, invalid/missing selector and no mutation. INFERRED: missing/invalid result policy; no geometry or 3D work.

### L36 `global api-C_HousingLayout-CanSetViewedFloor-036` — MODELABLE

Source: `C_HousingLayout.CanSetViewedFloor`

Occurrence input: `added:C_HousingLayout.CanSetViewedFloor`, historical status `best-effort` (not conclusion).

Assessment: Current provider is constant false; false on every call is not floor permission modeling.

Current provider: `src/c_api/c_housing.rs:645`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/HousingLayoutUIDocumentation.lua:26`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: explicit available floors and can-view set on housing state. Test permitted/denied/missing floors, permission updates and read-only queries. INFERRED: ownership/mode preconditions; do not implement a guessed service.

### L37 `global api-C_MerchantFrame-GetMerchantCurrencies-037` — MODELABLE

Source: `C_MerchantFrame.GetMerchantCurrencies`

Occurrence input: `added:C_MerchantFrame.GetMerchantCurrencies`, historical status `best-effort` (not conclusion).

Assessment: Current provider always returns an empty table; no populated merchant-currency list.

Current provider: `src/c_api/c_merchant_frame.rs:52`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/MerchantFrameDocumentation.lua:43`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`; `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: ordered merchant currency records using historical output fields; test two currencies, none, live merchant change, exact table tuple and detached snapshots; deprecated wrapper unpacks this list. INFERRED: order and missing-merchant policy.

### L38 `global api-C_PartyInfo-ConfirmReadyCheck-038` — ALREADY-COVERED-BY-12.0.5-CAPABILITY

Source: `C_PartyInfo.ConfirmReadyCheck`

Occurrence input: `added:C_PartyInfo.ConfirmReadyCheck`, historical status `implemented` (not conclusion).

Assessment: 12.0.5 ready-check-lockdown capability already exercises modeled ready-check state and named C_PartyInfo method with caller/lockdown controls; addition page row can only inherit that bounded scope.

Current provider: `src/c_api/c_party_info.rs:100`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:103`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`; `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`; `tests/chat_lockdown_ready_checks.rs:127::public_confirm_blocks_inactive_state_for_both_responses_and_combat_axes`; `tests/chat_lockdown_ready_checks.rs:159::public_confirm_preserves_active_response_then_recovers_after_unlock`. These are located source, not fresh execution.

Model / required proof / missing evidence: Retain capability limitations and add strict-12.0.7 epoch/migration checks if claiming historical availability; no full service/native security credit.

Inherited capability: `ready-check-lockdown`, recorded proof `bounded-independent-pass`, tests `tests/chat_lockdown_ready_checks.rs`. Gate validity: **yes, 12.0.7 includes retail-12-0-5 (Cargo.toml:120)**; only the recorded scope carries forward, not later policy or missing native/strict-profile proof. Recorded scope: Explicit chat-lockdown versus combat-only predicate; atomic rejection and unlock recovery in bounded simulator operations. Native producer/security/error/permissions, exhaustive API inventory and all-profile parity unknown; no C_Ping action or ping delivery claim.

### L39 `global api-C_PartyInfo-DemoteAssistant-039` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `C_PartyInfo.DemoteAssistant`

Occurrence input: `added:C_PartyInfo.DemoteAssistant`, historical status `implemented` (not conclusion).

Assessment: Current namespaced methods share live party roster/leader/assistant mutation helpers; occurrence implemented status is not a fresh acceptance gate.

Current provider: `src/c_api/c_party_info.rs:70`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:144`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`; `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove before/after leader/assistant/membership/uninvite state, multiple members, duplicate/missing/invalid cases, listener-observed state where emitted, exact arity and environment isolation. INFERRED: permission, self-removal and GUID domain policies not observed natively.

### L40 `global api-C_PartyInfo-DoReadyCheck-040` — ALREADY-COVERED-BY-12.0.5-CAPABILITY

Source: `C_PartyInfo.DoReadyCheck`

Occurrence input: `added:C_PartyInfo.DoReadyCheck`, historical status `implemented` (not conclusion).

Assessment: 12.0.5 ready-check-lockdown capability already exercises modeled ready-check state and named C_PartyInfo method with caller/lockdown controls; addition page row can only inherit that bounded scope.

Current provider: `src/c_api/c_party_info.rs:94`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:172`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`; `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`; `tests/chat_lockdown_ready_checks.rs:118::public_start_blocks_fresh_state_on_both_combat_axes`; `tests/chat_lockdown_ready_checks.rs:143::public_start_preserves_active_response_then_recovers_after_unlock`. These are located source, not fresh execution.

Model / required proof / missing evidence: Retain capability limitations and add strict-12.0.7 epoch/migration checks if claiming historical availability; no full service/native security credit.

Inherited capability: `ready-check-lockdown`, recorded proof `bounded-independent-pass`, tests `tests/chat_lockdown_ready_checks.rs`. Gate validity: **yes, 12.0.7 includes retail-12-0-5 (Cargo.toml:120)**; only the recorded scope carries forward, not later policy or missing native/strict-profile proof. Recorded scope: Explicit chat-lockdown versus combat-only predicate; atomic rejection and unlock recovery in bounded simulator operations. Native producer/security/error/permissions, exhaustive API inventory and all-profile parity unknown; no C_Ping action or ping delivery claim.

### L41 `global api-C_PartyInfo-IsGUIDInGroup-041` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `C_PartyInfo.IsGUIDInGroup`

Occurrence input: `added:C_PartyInfo.IsGUIDInGroup`, historical status `implemented` (not conclusion).

Assessment: Current namespaced methods share live party roster/leader/assistant mutation helpers; occurrence implemented status is not a fresh acceptance gate.

Current provider: `src/c_api/c_party_info.rs:62`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:420`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`; `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove before/after leader/assistant/membership/uninvite state, multiple members, duplicate/missing/invalid cases, listener-observed state where emitted, exact arity and environment isolation. INFERRED: permission, self-removal and GUID domain policies not observed natively.

### L42 `global api-C_PartyInfo-PromoteToAssistant-042` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `C_PartyInfo.PromoteToAssistant`

Occurrence input: `added:C_PartyInfo.PromoteToAssistant`, historical status `implemented` (not conclusion).

Assessment: Current namespaced methods share live party roster/leader/assistant mutation helpers; occurrence implemented status is not a fresh acceptance gate.

Current provider: `src/c_api/c_party_info.rs:76`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:495`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`; `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove before/after leader/assistant/membership/uninvite state, multiple members, duplicate/missing/invalid cases, listener-observed state where emitted, exact arity and environment isolation. INFERRED: permission, self-removal and GUID domain policies not observed natively.

### L43 `global api-C_PartyInfo-PromoteToLeader-043` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `C_PartyInfo.PromoteToLeader`

Occurrence input: `added:C_PartyInfo.PromoteToLeader`, historical status `implemented` (not conclusion).

Assessment: Current namespaced methods share live party roster/leader/assistant mutation helpers; occurrence implemented status is not a fresh acceptance gate.

Current provider: `src/c_api/c_party_info.rs:82`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:507`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`; `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove before/after leader/assistant/membership/uninvite state, multiple members, duplicate/missing/invalid cases, listener-observed state where emitted, exact arity and environment isolation. INFERRED: permission, self-removal and GUID domain policies not observed natively.

### L44 `global api-C_PartyInfo-SetEveryoneIsAssistant-044` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `C_PartyInfo.SetEveryoneIsAssistant`

Occurrence input: `added:C_PartyInfo.SetEveryoneIsAssistant`, historical status `implemented` (not conclusion).

Assessment: Current namespaced methods share live party roster/leader/assistant mutation helpers; occurrence implemented status is not a fresh acceptance gate.

Current provider: `src/c_api/c_party_info.rs:88`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:534`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`; `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove before/after leader/assistant/membership/uninvite state, multiple members, duplicate/missing/invalid cases, listener-observed state where emitted, exact arity and environment isolation. INFERRED: permission, self-removal and GUID domain policies not observed natively.

### L45 `global api-C_PartyInfo-UninviteUnit-045` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `C_PartyInfo.UninviteUnit`

Occurrence input: `added:C_PartyInfo.UninviteUnit`, historical status `implemented` (not conclusion).

Assessment: Current namespaced methods share live party roster/leader/assistant mutation helpers; occurrence implemented status is not a fresh acceptance gate.

Current provider: `src/c_api/c_party_info.rs:66`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PartyInfoDocumentation.lua:596`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`; `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`; `tests/group_verbs.rs:116::remove_from_party_unknown_name_is_noop`; `tests/group_verbs.rs:126::uninvite_unit_by_party_token_removes_indexed_member`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove before/after leader/assistant/membership/uninvite state, multiple members, duplicate/missing/invalid cases, listener-observed state where emitted, exact arity and environment isolation. INFERRED: permission, self-removal and GUID domain policies not observed natively.

### L46 `global api-C_PingSecure-ClearPendingPingOffScreenCallback-046` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `C_PingSecure.ClearPendingPingOffScreenCallback`

Occurrence input: `added:C_PingSecure.ClearPendingPingOffScreenCallback`, historical status `implemented` (not conclusion).

Assessment: Shared PingSecure callback table implements storage/get/clear; no secure execution enforcement is inferred from its name.

Current provider: `src/c_api/c_ping_secure.rs:76`.

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `src/c_api/c_ping_secure.rs:223::registers_ping_secure_api`; `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove callback identity replacement/clear, independent slots/environments, retained roots after collection and unchanged caller taint. Native gamepad secure-dispatch and later removed names stay outside bounded storage credit.

### L47 `global api-C_PingSecure-SetPendingPingOffScreenCallback-047` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `C_PingSecure.SetPendingPingOffScreenCallback`

Occurrence input: `added:C_PingSecure.SetPendingPingOffScreenCallback`, historical status `implemented` (not conclusion).

Assessment: Shared PingSecure callback table implements storage/get/clear; no secure execution enforcement is inferred from its name.

Current provider: `src/c_api/c_ping_secure.rs:83`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PingManagerSecureDocumentation.lua:159`; **later cache: may postdate 12.0.7**.

Existing tests: `src/c_api/c_ping_secure.rs:223::registers_ping_secure_api`; `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`; `tests/blizzard_ping_ui_loads.rs:127::blizzard_ping_ui_toc_declares_eager_secure_mainline_only_with_sharedxml_dep`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove callback identity replacement/clear, independent slots/environments, retained roots after collection and unchanged caller taint. Native gamepad secure-dispatch and later removed names stay outside bounded storage credit.

### L48 `global api-C_QuestHub-GetDragonridingRacesForAreaPOI-048` — BLOCKED

Source: `C_QuestHub.GetDragonridingRacesForAreaPOI`

Occurrence input: `added:C_QuestHub.GetDragonridingRacesForAreaPOI`, historical status `best-effort` (not conclusion).

Assessment: Current provider returns an empty table; exact method is absent from current generated cache.

Current provider: `src/c_api/c_quest_hub.rs:27`.

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`. These are located source, not fresh execution.

Model / required proof / missing evidence: Missing historical GetDragonridingRacesForAreaPOI argument and result declaration/content schema. Obtain 12.0.7 declaration or native populated return before modeling areaPOI -> ordered race records; empty array alone gives no populated-state evidence.

### L49 `global api-C_UIFileAsset-GetFileID-049` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `C_UIFileAsset.GetFileID`

Occurrence input: `added:C_UIFileAsset.GetFileID`, historical status `best-effort` (not conclusion).

Assessment: Current GetFileID resolves positive numeric IDs unchanged and limited listfile paths; existing tests cover concrete numeric/path/miss behavior.

Current provider: `src/c_api/c_ui_file_asset.rs:16`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/UIFileAssetAPIDocumentation.lua:11`; **later cache: may postdate 12.0.7**.

Existing tests: `src/c_api/c_ui_file_asset.rs:152::ui_file_asset_uses_limited_listfile_paths`; `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`; `tests/ui_file_assets.rs:24::loose_sound_is_known_during_load_and_afterward_without_synthetic_id`; `tests/ui_file_assets.rs:64::only_selected_addon_root_supplies_loose_files_and_texture_extensions`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove numeric ID, known path normalization, unknown path, invalid numeric boundaries and exact arity; authenticate AllowedWhenUntainted separately. Limited listfile is not all client assets.

### L50 `global api-C_UIFileAsset-IsKnownFile-050` — MODELABLE

Source: `C_UIFileAsset.IsKnownFile`

Occurrence input: `added:C_UIFileAsset.IsKnownFile`, historical status `best-effort` (not conclusion).

Assessment: Current shared query_asset classifies selected addon files by actual canonical filesystem existence. Current declaration says known loose files need not exist or be openable, so existing tests do not establish that declaration.

Current provider: `src/c_api/c_ui_file_asset.rs:20`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/UIFileAssetAPIDocumentation.lua:27`; **later cache: may postdate 12.0.7**.

Existing tests: `src/c_api/c_ui_file_asset.rs:152::ui_file_asset_uses_limited_listfile_paths`; `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`; `tests/ui_file_assets.rs:24::loose_sound_is_known_during_load_and_afterward_without_synthetic_id`; `tests/ui_file_assets.rs:64::only_selected_addon_root_supplies_loose_files_and_texture_extensions`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: loader-owned known-loose asset registry separate from shipped fileID mapping; query membership without filesystem existence check. Test registered present and absent files, unknown physically present file, selected-root scope and shipped numeric/path controls. INFERRED: historical 12.0.7 registry population; obtain pinned declaration before asserting this later-cache policy.

### L51 `global api-C_UIFileAsset-IsLooseFile-051` — MODELABLE

Source: `C_UIFileAsset.IsLooseFile`

Occurrence input: `added:C_UIFileAsset.IsLooseFile`, historical status `best-effort` (not conclusion).

Assessment: Current shared query_asset classifies selected addon files by actual canonical filesystem existence. Current declaration says known loose files need not exist or be openable, so existing tests do not establish that declaration.

Current provider: `src/c_api/c_ui_file_asset.rs:26`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/UIFileAssetAPIDocumentation.lua:43`; **later cache: may postdate 12.0.7**.

Existing tests: `src/c_api/c_ui_file_asset.rs:152::ui_file_asset_uses_limited_listfile_paths`; `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`; `tests/ui_file_assets.rs:24::loose_sound_is_known_during_load_and_afterward_without_synthetic_id`; `tests/ui_file_assets.rs:64::only_selected_addon_root_supplies_loose_files_and_texture_extensions`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: loader-owned known-loose asset registry separate from shipped fileID mapping; query membership without filesystem existence check. Test registered present and absent files, unknown physically present file, selected-root scope and shipped numeric/path controls. INFERRED: historical 12.0.7 registry population; obtain pinned declaration before asserting this later-cache policy.

### L52 `global api-GetEventCPUUsage-052` — MODELABLE

Source: `GetEventCPUUsage`

Occurrence input: `added:GetEventCPUUsage`, historical status `best-effort` (not conclusion).

Assessment: Shared temporary metric defaults return constant values; callable functions do not measure event/function/script CPU use.

Current provider: `src/lua_api/workarounds/temporary/performance_metric_defaults.rs:49`; `src/lua_api/workarounds/temporary/performance_metric_defaults.rs:50`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:40`; **later cache: may postdate 12.0.7**.

Existing tests: `src/lua_api/workarounds/temporary/performance_metric_defaults.rs:98::installs_performance_metric_defaults`; `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: per-environment explicit cumulative CPU counters keyed by event/function/frame-script, with reset/snapshot semantics matching historical signatures; prove nonzero counters, independent keys, reset, cumulative versus last-call behavior and exact tuple. INFERRED: counter units/reset/attribution; require historical/native policy before full parity.

### L53 `global api-GetFunctionCPUUsage-053` — MODELABLE

Source: `GetFunctionCPUUsage`

Occurrence input: `added:GetFunctionCPUUsage`, historical status `best-effort` (not conclusion).

Assessment: Shared temporary metric defaults return constant values; callable functions do not measure event/function/script CPU use.

Current provider: `src/lua_api/workarounds/temporary/performance_metric_defaults.rs:55`; `src/lua_api/workarounds/temporary/performance_metric_defaults.rs:56`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:67`; **later cache: may postdate 12.0.7**.

Existing tests: `src/lua_api/workarounds/temporary/performance_metric_defaults.rs:98::installs_performance_metric_defaults`; `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: per-environment explicit cumulative CPU counters keyed by event/function/frame-script, with reset/snapshot semantics matching historical signatures; prove nonzero counters, independent keys, reset, cumulative versus last-call behavior and exact tuple. INFERRED: counter units/reset/attribution; require historical/native policy before full parity.

### L54 `global api-GetScriptCPUUsage-054` — MODELABLE

Source: `GetScriptCPUUsage`

Occurrence input: `added:GetScriptCPUUsage`, historical status `best-effort` (not conclusion).

Assessment: Shared temporary metric defaults return constant values; callable functions do not measure event/function/script CPU use.

Current provider: `src/lua_api/workarounds/temporary/performance_metric_defaults.rs:61`; `src/lua_api/workarounds/temporary/performance_metric_defaults.rs:62`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/PerformanceDocumentation.lua:77`; **later cache: may postdate 12.0.7**.

Existing tests: `src/lua_api/workarounds/temporary/performance_metric_defaults.rs:98::installs_performance_metric_defaults`; `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: per-environment explicit cumulative CPU counters keyed by event/function/frame-script, with reset/snapshot semantics matching historical signatures; prove nonzero counters, independent keys, reset, cumulative versus last-call behavior and exact tuple. INFERRED: counter units/reset/attribution; require historical/native policy before full parity.

### L55 `global api-GetSecurePendingButtonCallback-055` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `GetSecurePendingButtonCallback`

Occurrence input: `added:GetSecurePendingButtonCallback`, historical status `implemented` (not conclusion).

Assessment: Shared PingSecure callback table implements storage/get/clear; no secure execution enforcement is inferred from its name.

Current provider: `src/c_api/c_ping_secure.rs:30`.

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`; `tests/wowforever_profile.rs:202::wowforever_profile_reports_build_identity_and_finite_event_validation`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove callback identity replacement/clear, independent slots/environments, retained roots after collection and unchanged caller taint. Native gamepad secure-dispatch and later removed names stay outside bounded storage credit.

### L56 `global api-GetSecurePendingPingOffScreenCallback-056` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `GetSecurePendingPingOffScreenCallback`

Occurrence input: `added:GetSecurePendingPingOffScreenCallback`, historical status `implemented` (not conclusion).

Assessment: Shared PingSecure callback table implements storage/get/clear; no secure execution enforcement is inferred from its name.

Current provider: `src/c_api/c_ping_secure.rs:36`.

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove callback identity replacement/clear, independent slots/environments, retained roots after collection and unchanged caller taint. Native gamepad secure-dispatch and later removed names stay outside bounded storage credit.

### L57 `global api-GetSecurePendingToggleRunCallback-057` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `GetSecurePendingToggleRunCallback`

Occurrence input: `added:GetSecurePendingToggleRunCallback`, historical status `implemented` (not conclusion).

Assessment: Shared PingSecure callback table implements storage/get/clear; no secure execution enforcement is inferred from its name.

Current provider: `src/c_api/c_ping_secure.rs:42`.

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove callback identity replacement/clear, independent slots/environments, retained roots after collection and unchanged caller taint. Native gamepad secure-dispatch and later removed names stay outside bounded storage credit.

### L58 `global api-GameTooltip_AddMoneyLine-058` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `GameTooltip_AddMoneyLine`

Occurrence input: `added:GameTooltip_AddMoneyLine`, historical status `best-effort` (not conclusion).

Assessment: Loaded Blizzard_GameTooltip helper, not a simulator bootstrap prefix shim, owns this Lua function. Existing loaded tests assert coin-atlas text, label order and colors with known dependency Lua errors.

Current provider: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_GameTooltip/Mainline/GameTooltip.lua:320`.

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `tests/tooltip_money_line.rs:86::tooltip_money_line_loaded_helper_formats_zero_and_boolean_colors`. These are located source, not fresh execution.

Model / required proof / missing evidence: Reuse bounded loaded-helper evidence only after matching current helper/test bytes and availability under strict 12.0.7; test representative amounts, zero, label-before-money, highlight/red. No clean full-addon/layout/locale/native credit.

### L59 `global api-SetSecurePendingButtonCallback-059` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `SetSecurePendingButtonCallback`

Occurrence input: `added:SetSecurePendingButtonCallback`, historical status `implemented` (not conclusion).

Assessment: Shared PingSecure callback table implements storage/get/clear; no secure execution enforcement is inferred from its name.

Current provider: `src/c_api/c_ping_secure.rs:48`.

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove callback identity replacement/clear, independent slots/environments, retained roots after collection and unchanged caller taint. Native gamepad secure-dispatch and later removed names stay outside bounded storage credit.

### L60 `global api-SetSecurePendingPingOffScreenCallback-060` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `SetSecurePendingPingOffScreenCallback`

Occurrence input: `added:SetSecurePendingPingOffScreenCallback`, historical status `implemented` (not conclusion).

Assessment: Shared PingSecure callback table implements storage/get/clear; no secure execution enforcement is inferred from its name.

Current provider: `src/c_api/c_ping_secure.rs:54`.

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove callback identity replacement/clear, independent slots/environments, retained roots after collection and unchanged caller taint. Native gamepad secure-dispatch and later removed names stay outside bounded storage credit.

### L61 `global api-SetSecurePendingToggleRunCallback-061` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `SetSecurePendingToggleRunCallback`

Occurrence input: `added:SetSecurePendingToggleRunCallback`, historical status `implemented` (not conclusion).

Assessment: Shared PingSecure callback table implements storage/get/clear; no secure execution enforcement is inferred from its name.

Current provider: `src/c_api/c_ping_secure.rs:60`.

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove callback identity replacement/clear, independent slots/environments, retained roots after collection and unchanged caller taint. Native gamepad secure-dispatch and later removed names stay outside bounded storage credit.

### L63 `source-context-063` — METADATA-ONLY

Source: `Global API Removed (17):`

Assessment / exact one-sentence metadata note: Section heading and stated inventory count organize the retained excerpt only; no runtime or implementation credit.

Current provider: No exact provider located for this row; no provider is required for editorial metadata..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

### L64 `global api-BNInviteFriend-064` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `BNInviteFriend`

Occurrence input: `removed:BNInviteFriend`, historical status `best-effort` (not conclusion).

Assessment: Current startup removal test asserts this legacy global is nil; full-load deprecated wrapper behavior and historical old-profile availability remain unproved.

Current provider: No exact receiver/symbol provider located; neighboring subsystem only, no callable credit..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove strict 12.0.7 init absence, loaded wrapper on/off if historically shipped, old-profile positive control and exact forwarding tuple; no later-cache absence -> historical absence inference.

### L65 `global api-C_ClickBindings-GetStringFromModifiers-065` — MODELABLE

Source: `C_ClickBindings.GetStringFromModifiers`

Occurrence input: `removed:C_ClickBindings.GetStringFromModifiers`, historical status `best-effort` (not conclusion).

Assessment: Source removes native entry point but separately records deprecated wrappers. Simulator still registers several legacy entries; absence cannot be inferred from current stub/method inventories.

Current provider: `src/lua_api/workarounds/temporary/click_bindings_defaults.rs:27`.

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest change: profile-aware native publication boundary plus existing vendor deprecated wrappers controlled by loadDeprecationFallbacks, not unconditional deletion of compatibility aliases. Test old-profile positive control, strict 12.0.7 init native absence, loaded wrapper on/off, dispatch/return tuple and no loader resurrection. INFERRED: exact removal boundary until historical native-versus-wrapper evidence. Existing 12.0.5 spell-maw-powers capability proves the OLD function, not its removal.

### L66 `global api-C_ClickBindings-MakeModifiers-066` — MODELABLE

Source: `C_ClickBindings.MakeModifiers`

Occurrence input: `removed:C_ClickBindings.MakeModifiers`, historical status `best-effort` (not conclusion).

Assessment: Source removes native entry point but separately records deprecated wrappers. Simulator still registers several legacy entries; absence cannot be inferred from current stub/method inventories.

Current provider: `src/lua_api/workarounds/temporary/click_bindings_defaults.rs:55`.

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `src/lua_api/workarounds/temporary/click_bindings_defaults.rs:77::default_profile_reports_interaction_and_execute_remains_inert`; `src/lua_api/workarounds/temporary/click_bindings_defaults.rs:96::make_modifiers_reads_modeled_modifier_keys`; `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`; `tests/click_targeting.rs:257::click_bindings_default_profile_reports_interaction_but_execute_remains_inert`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest change: profile-aware native publication boundary plus existing vendor deprecated wrappers controlled by loadDeprecationFallbacks, not unconditional deletion of compatibility aliases. Test old-profile positive control, strict 12.0.7 init native absence, loaded wrapper on/off, dispatch/return tuple and no loader resurrection. INFERRED: exact removal boundary until historical native-versus-wrapper evidence. Existing 12.0.5 spell-maw-powers capability proves the OLD function, not its removal.

### L67 `global api-C_Spell-GetMawPowerBorderAtlasBySpellID-067` — MODELABLE

Source: `C_Spell.GetMawPowerBorderAtlasBySpellID`

Occurrence input: `removed:C_Spell.GetMawPowerBorderAtlasBySpellID`, historical status `best-effort` (not conclusion).

Assessment: Source removes native entry point but separately records deprecated wrappers. Simulator still registers several legacy entries; absence cannot be inferred from current stub/method inventories.

Current provider: `src/c_api/c_spell_maw_powers.rs:25`.

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `src/lua_api/workarounds/temporary/spell_static_defaults.rs:40::installs_spell_static_defaults`; `src/lua_api/workarounds/temporary/spell_static_defaults.rs:60::preserves_existing_spell_static_provider`; `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`; `tests/blizzard_maw_buffs_loads.rs:240::blizzard_maw_buffs_excluded_from_all_glue_screen_auto_discovery_passes`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest change: profile-aware native publication boundary plus existing vendor deprecated wrappers controlled by loadDeprecationFallbacks, not unconditional deletion of compatibility aliases. Test old-profile positive control, strict 12.0.7 init native absence, loaded wrapper on/off, dispatch/return tuple and no loader resurrection. INFERRED: exact removal boundary until historical native-versus-wrapper evidence. Existing 12.0.5 spell-maw-powers capability proves the OLD function, not its removal.

### L68 `global api-ConfirmReadyCheck-068` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `ConfirmReadyCheck`

Occurrence input: `removed:ConfirmReadyCheck`, historical status `best-effort` (not conclusion).

Assessment: Current startup removal test asserts this legacy global is nil; full-load deprecated wrapper behavior and historical old-profile availability remain unproved.

Current provider: No exact receiver/symbol provider located; neighboring subsystem only, no callable credit..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`; `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`; `tests/chat_lockdown_ready_checks.rs:127::public_confirm_blocks_inactive_state_for_both_responses_and_combat_axes`; `tests/chat_lockdown_ready_checks.rs:159::public_confirm_preserves_active_response_then_recovers_after_unlock`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove strict 12.0.7 init absence, loaded wrapper on/off if historically shipped, old-profile positive control and exact forwarding tuple; no later-cache absence -> historical absence inference.

### L69 `global api-DemoteAssistant-069` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `DemoteAssistant`

Occurrence input: `removed:DemoteAssistant`, historical status `best-effort` (not conclusion).

Assessment: Current startup removal test asserts this legacy global is nil; full-load deprecated wrapper behavior and historical old-profile availability remain unproved.

Current provider: No exact receiver/symbol provider located; neighboring subsystem only, no callable credit..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`; `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove strict 12.0.7 init absence, loaded wrapper on/off if historically shipped, old-profile positive control and exact forwarding tuple; no later-cache absence -> historical absence inference.

### L70 `global api-DoReadyCheck-070` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `DoReadyCheck`

Occurrence input: `removed:DoReadyCheck`, historical status `best-effort` (not conclusion).

Assessment: Current startup removal test asserts this legacy global is nil; full-load deprecated wrapper behavior and historical old-profile availability remain unproved.

Current provider: No exact receiver/symbol provider located; neighboring subsystem only, no callable credit..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`; `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`; `tests/chat_lockdown_ready_checks.rs:118::public_start_blocks_fresh_state_on_both_combat_axes`; `tests/chat_lockdown_ready_checks.rs:143::public_start_preserves_active_response_then_recovers_after_unlock`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove strict 12.0.7 init absence, loaded wrapper on/off if historically shipped, old-profile positive control and exact forwarding tuple; no later-cache absence -> historical absence inference.

### L71 `global api-GetMerchantCurrencies-071` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `GetMerchantCurrencies`

Occurrence input: `removed:GetMerchantCurrencies`, historical status `best-effort` (not conclusion).

Assessment: Current startup removal test asserts this legacy global is nil; full-load deprecated wrapper behavior and historical old-profile availability remain unproved.

Current provider: No exact receiver/symbol provider located; neighboring subsystem only, no callable credit..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`; `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove strict 12.0.7 init absence, loaded wrapper on/off if historically shipped, old-profile positive control and exact forwarding tuple; no later-cache absence -> historical absence inference.

### L72 `global api-IsGUIDInGroup-072` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `IsGUIDInGroup`

Occurrence input: `removed:IsGUIDInGroup`, historical status `best-effort` (not conclusion).

Assessment: Current startup removal test asserts this legacy global is nil; full-load deprecated wrapper behavior and historical old-profile availability remain unproved.

Current provider: No exact receiver/symbol provider located; neighboring subsystem only, no callable credit..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`; `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove strict 12.0.7 init absence, loaded wrapper on/off if historically shipped, old-profile positive control and exact forwarding tuple; no later-cache absence -> historical absence inference.

### L73 `global api-PromoteToAssistant-073` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `PromoteToAssistant`

Occurrence input: `removed:PromoteToAssistant`, historical status `best-effort` (not conclusion).

Assessment: Current startup removal test asserts this legacy global is nil; full-load deprecated wrapper behavior and historical old-profile availability remain unproved.

Current provider: No exact receiver/symbol provider located; neighboring subsystem only, no callable credit..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`; `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove strict 12.0.7 init absence, loaded wrapper on/off if historically shipped, old-profile positive control and exact forwarding tuple; no later-cache absence -> historical absence inference.

### L74 `global api-PromoteToLeader-074` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `PromoteToLeader`

Occurrence input: `removed:PromoteToLeader`, historical status `best-effort` (not conclusion).

Assessment: Current startup removal test asserts this legacy global is nil; full-load deprecated wrapper behavior and historical old-profile availability remain unproved.

Current provider: No exact receiver/symbol provider located; neighboring subsystem only, no callable credit..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`; `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove strict 12.0.7 init absence, loaded wrapper on/off if historically shipped, old-profile positive control and exact forwarding tuple; no later-cache absence -> historical absence inference.

### L75 `global api-SetEveryoneIsAssistant-075` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `SetEveryoneIsAssistant`

Occurrence input: `removed:SetEveryoneIsAssistant`, historical status `best-effort` (not conclusion).

Assessment: Current startup removal test asserts this legacy global is nil; full-load deprecated wrapper behavior and historical old-profile availability remain unproved.

Current provider: No exact receiver/symbol provider located; neighboring subsystem only, no callable credit..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`; `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove strict 12.0.7 init absence, loaded wrapper on/off if historically shipped, old-profile positive control and exact forwarding tuple; no later-cache absence -> historical absence inference.

### L76 `global api-UninviteUnit-076` — MODELABLE

Source: `UninviteUnit`

Occurrence input: `removed:UninviteUnit`, historical status `best-effort` (not conclusion).

Assessment: Source removes native entry point but separately records deprecated wrappers. Simulator still registers several legacy entries; absence cannot be inferred from current stub/method inventories.

Current provider: `src/lua_api/globals/group_verbs.rs:234`.

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:211::test_patch_12_0_7_safe_global_bridges`; `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`; `tests/group_verbs.rs:116::remove_from_party_unknown_name_is_noop`; `tests/group_verbs.rs:126::uninvite_unit_by_party_token_removes_indexed_member`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest change: profile-aware native publication boundary plus existing vendor deprecated wrappers controlled by loadDeprecationFallbacks, not unconditional deletion of compatibility aliases. Test old-profile positive control, strict 12.0.7 init native absence, loaded wrapper on/off, dispatch/return tuple and no loader resurrection. INFERRED: exact removal boundary until historical native-versus-wrapper evidence. Existing 12.0.5 spell-maw-powers capability proves the OLD function, not its removal.

### L77 `global api-GetAutoCompletePresenceID-077` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `GetAutoCompletePresenceID`

Occurrence input: `removed:GetAutoCompletePresenceID`, historical status `best-effort` (not conclusion).

Assessment: Current startup removal test asserts this legacy global is nil; full-load deprecated wrapper behavior and historical old-profile availability remain unproved.

Current provider: No exact receiver/symbol provider located; neighboring subsystem only, no callable credit..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`; `tests/blizzard_deprecated_auto_complete_loads.rs:79::blizzard_deprecated_auto_complete_appears_in_game_discovery_only`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove strict 12.0.7 init absence, loaded wrapper on/off if historically shipped, old-profile positive control and exact forwarding tuple; no later-cache absence -> historical absence inference.

### L78 `global api-GetAutoCompleteResults-078` — MODELABLE

Source: `GetAutoCompleteResults`

Occurrence input: `removed:GetAutoCompleteResults`, historical status `best-effort` (not conclusion).

Assessment: Source removes native entry point but separately records deprecated wrappers. Simulator still registers several legacy entries; absence cannot be inferred from current stub/method inventories.

Current provider: `src/c_api/c_auto_complete.rs:41`.

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `src/lua_api/workarounds/temporary/auto_complete_defaults.rs:55::installs_legacy_results_forwarder`; `src/lua_api/workarounds/temporary/auto_complete_defaults.rs:81::preserves_existing_legacy_results_forwarder`; `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`; `tests/blizzard_deprecated_auto_complete_loads.rs:45::blizzard_deprecated_auto_complete_toc_is_minimal_with_no_flags_or_deps`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest change: profile-aware native publication boundary plus existing vendor deprecated wrappers controlled by loadDeprecationFallbacks, not unconditional deletion of compatibility aliases. Test old-profile positive control, strict 12.0.7 init native absence, loaded wrapper on/off, dispatch/return tuple and no loader resurrection. INFERRED: exact removal boundary until historical native-versus-wrapper evidence. Existing 12.0.5 spell-maw-powers capability proves the OLD function, not its removal.

### L79 `global api-GetAutoCompleteRealms-079` — MODELABLE

Source: `GetAutoCompleteRealms`

Occurrence input: `removed:GetAutoCompleteRealms`, historical status `best-effort` (not conclusion).

Assessment: Source removes native entry point but separately records deprecated wrappers. Simulator still registers several legacy entries; absence cannot be inferred from current stub/method inventories.

Current provider: `src/lua_api/workarounds/temporary/auto_complete_defaults.rs:10`.

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `src/lua_api/workarounds/temporary/auto_complete_defaults.rs:37::installs_empty_realm_defaults`; `src/lua_api/workarounds/temporary/auto_complete_defaults.rs:106::preserves_existing_autocomplete_realms`; `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`; `tests/blizzard_deprecated_auto_complete_loads.rs:79::blizzard_deprecated_auto_complete_appears_in_game_discovery_only`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest change: profile-aware native publication boundary plus existing vendor deprecated wrappers controlled by loadDeprecationFallbacks, not unconditional deletion of compatibility aliases. Test old-profile positive control, strict 12.0.7 init native absence, loaded wrapper on/off, dispatch/return tuple and no loader resurrection. INFERRED: exact removal boundary until historical native-versus-wrapper evidence. Existing 12.0.5 spell-maw-powers capability proves the OLD function, not its removal.

### L80 `global api-IsRecognizedName-080` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `IsRecognizedName`

Occurrence input: `removed:IsRecognizedName`, historical status `best-effort` (not conclusion).

Assessment: Current startup removal test asserts this legacy global is nil; full-load deprecated wrapper behavior and historical old-profile availability remain unproved.

Current provider: No exact receiver/symbol provider located; neighboring subsystem only, no callable credit..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`; `tests/blizzard_deprecated_auto_complete_loads.rs:45::blizzard_deprecated_auto_complete_toc_is_minimal_with_no_flags_or_deps`; `tests/blizzard_deprecated_auto_complete_loads.rs:79::blizzard_deprecated_auto_complete_appears_in_game_discovery_only`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove strict 12.0.7 init absence, loaded wrapper on/off if historically shipped, old-profile positive control and exact forwarding tuple; no later-cache absence -> historical absence inference.

### L82 `source-context-082` — METADATA-ONLY

Source: `ScriptObjects Added (36):`

Assessment / exact one-sentence metadata note: Section heading and stated inventory count organize the retained excerpt only; no runtime or implementation credit.

Current provider: No exact provider located for this row; no provider is required for editorial metadata..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

### L83 `scriptobjects-DurationClock-GetTime-083` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `DurationClock:GetTime`

Occurrence input: `added:DurationClock.GetTime`, historical status `implemented` (not conclusion).

Assessment: Existing mutable clock table stores time and exposes GetTime/SetTime/AdvanceTime/RewindTime/ResetTime; cached manual clock is declared userdata with AllowedWhenUntainted inputs.

Current provider: `src/lua_api/globals/lua_duration_object.rs:176`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/LuaDurationClockAPIDocumentation.lua:11`; **later cache: may postdate 12.0.7**.

Existing tests: `src/lua_api/workarounds/temporary/game_time_calendar_invites.rs:93::installs_missing_game_time_clock_default`; `src/lua_api/workarounds/temporary/game_time_calendar_invites.rs:139::preserves_existing_game_time_clock_default`; `src/loader/tests/wow_api_globals/startup_globals.rs:84::test_patch_12_0_7_duration_objects_and_text_binding`; `src/loader/tests/wow_api_globals/startup_globals.rs:1023::test_old_stack_startup_globals_exist_on_rilua_path`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove initial time, fractional forward/backward/reset, independent clocks and exact arity. Preserve explicit clock state; do not silently claim userdata identity or authenticated secret-input parity. INFERRED: negative/nonfinite/default-input policy.

### L84 `scriptobjects-DurationManualClock-AdvanceTime-084` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `DurationManualClock:AdvanceTime`

Occurrence input: `added:DurationManualClock.AdvanceTime`, historical status `implemented` (not conclusion).

Assessment: Existing mutable clock table stores time and exposes GetTime/SetTime/AdvanceTime/RewindTime/ResetTime; cached manual clock is declared userdata with AllowedWhenUntainted inputs.

Current provider: `src/lua_api/globals/lua_duration_object.rs:190`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/LuaDurationManualClockAPIDocumentation.lua:11`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:84::test_patch_12_0_7_duration_objects_and_text_binding`; `tests/duration_core.rs:376::duration_core_manual_progress_rate_and_rewind`; `tests/duration_core.rs:582::duration_copy_shares_clock_reference_but_rebinds_independently`; `tests/duration_core.rs:608::duration_copy_assign_shares_clock_reference_but_rebinds_independently`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove initial time, fractional forward/backward/reset, independent clocks and exact arity. Preserve explicit clock state; do not silently claim userdata identity or authenticated secret-input parity. INFERRED: negative/nonfinite/default-input policy.

### L85 `scriptobjects-DurationManualClock-ResetTime-085` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `DurationManualClock:ResetTime`

Occurrence input: `added:DurationManualClock.ResetTime`, historical status `implemented` (not conclusion).

Assessment: Existing mutable clock table stores time and exposes GetTime/SetTime/AdvanceTime/RewindTime/ResetTime; cached manual clock is declared userdata with AllowedWhenUntainted inputs.

Current provider: `src/lua_api/globals/lua_duration_object.rs:204`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/LuaDurationManualClockAPIDocumentation.lua:22`; **later cache: may postdate 12.0.7**.

Existing tests: `src/lua_api/workarounds/temporary/game_time_calendar_invites.rs:111::installs_missing_game_time_globals`; `src/loader/tests/wow_api_globals/startup_globals.rs:84::test_patch_12_0_7_duration_objects_and_text_binding`; `tests/startup_api_stubs.rs:452::startup_quest_link_and_date_helpers_are_callable`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove initial time, fractional forward/backward/reset, independent clocks and exact arity. Preserve explicit clock state; do not silently claim userdata identity or authenticated secret-input parity. INFERRED: negative/nonfinite/default-input policy.

### L86 `scriptobjects-DurationManualClock-RewindTime-086` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `DurationManualClock:RewindTime`

Occurrence input: `added:DurationManualClock.RewindTime`, historical status `implemented` (not conclusion).

Assessment: Existing mutable clock table stores time and exposes GetTime/SetTime/AdvanceTime/RewindTime/ResetTime; cached manual clock is declared userdata with AllowedWhenUntainted inputs.

Current provider: `src/lua_api/globals/lua_duration_object.rs:197`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/LuaDurationManualClockAPIDocumentation.lua:31`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:84::test_patch_12_0_7_duration_objects_and_text_binding`; `tests/duration_core.rs:376::duration_core_manual_progress_rate_and_rewind`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove initial time, fractional forward/backward/reset, independent clocks and exact arity. Preserve explicit clock state; do not silently claim userdata identity or authenticated secret-input parity. INFERRED: negative/nonfinite/default-input policy.

### L87 `scriptobjects-DurationManualClock-SetTime-087` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `DurationManualClock:SetTime`

Occurrence input: `added:DurationManualClock.SetTime`, historical status `implemented` (not conclusion).

Assessment: Existing mutable clock table stores time and exposes GetTime/SetTime/AdvanceTime/RewindTime/ResetTime; cached manual clock is declared userdata with AllowedWhenUntainted inputs.

Current provider: `src/lua_api/globals/lua_duration_object.rs:183`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/LuaDurationManualClockAPIDocumentation.lua:42`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:84::test_patch_12_0_7_duration_objects_and_text_binding`; `tests/encounter_timeline_script.rs:57::encounter_script_clock_pause_resume_and_retained_timer`; `tests/duration_core.rs:5::retail_native_aura_button_preserves_wrapped_duration_arguments`; `tests/duration_core.rs:98::duration_core_secret_inputs_store_wrappers_and_preserve_lifecycle`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove initial time, fractional forward/backward/reset, independent clocks and exact arity. Preserve explicit clock state; do not silently claim userdata identity or authenticated secret-input parity. INFERRED: negative/nonfinite/default-input policy.

### L88 `scriptobjects-DurationObject-GetClock-088` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `DurationObject:GetClock`

Occurrence input: `added:DurationObject.GetClock`, historical status `best-effort` (not conclusion).

Assessment: Duration proxy already stores clock handle and uses duration-core queries. Addition chronology is separate from existing implementation.

Current provider: `src/lua_api/globals/lua_duration_object.rs:533`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/LuaDurationObjectAPIDocumentation.lua:181`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:84::test_patch_12_0_7_duration_objects_and_text_binding`; `tests/duration_core.rs:98::duration_core_secret_inputs_store_wrappers_and_preserve_lifecycle`; `tests/duration_core.rs:147::duration_core_secret_tainted_queries_and_mutations_reject_atomically`; `tests/duration_core.rs:245::duration_percent_tracks_clock_boundaries_and_rewind`. These are located source, not fresh execution.

Model / required proof / missing evidence: Exercise same object before/after clock replacement, future/active/expired boundaries and clock movement, rooted clock identity and independent environments; authenticate secret clock/time arguments separately.

### L89 `scriptobjects-DurationObject-HasExpired-089` — ALREADY-COVERED-BY-12.0.5-CAPABILITY

Source: `DurationObject:HasExpired`

Occurrence input: `added:DurationObject.HasExpired`, historical status `best-effort` (not conclusion).

Assessment: Bounded zero-span HasExpired behavior is independently accepted by 12.0.5; general nonzero/manual-clock/native behavior is not implied.

Current provider: `src/lua_api/globals/lua_duration_object/core.rs:392`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/LuaDurationObjectAPIDocumentation.lua:335`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:84::test_patch_12_0_7_duration_objects_and_text_binding`; `tests/duration_core.rs:147::duration_core_secret_tainted_queries_and_mutations_reject_atomically`; `tests/duration_core.rs:245::duration_percent_tracks_clock_boundaries_and_rewind`; `tests/duration_core.rs:376::duration_core_manual_progress_rate_and_rewind`. These are located source, not fresh execution.

Model / required proof / missing evidence: Reuse only zero-span expiry proof; broader clock and secret matrices stay open.

Inherited capability: `zero-span-charge-durations`, recorded proof `bounded-independent-pass`, tests `tests/cooldown_probes/charge_duration.rs`, `tests/duration_core.rs`. Gate validity: **yes, 12.0.7 includes retail-12-0-5 (Cargo.toml:120)**; only the recorded scope carries forward, not later policy or missing native/strict-profile proof. Recorded scope: B90 prose-2026-03-12-023/026: at maximum charges all three charge-duration queries return a nonnil zero-span duration object from the shared explicit charge-state producer; zero-span objects report expired, elapsed fraction1 and remaining fraction0 for default, future-start, reset and explicit zero intervals, with a below-maximum nonzero control. 'Fully elapsed' is concretized as HasExpired/fractions, HasStarted deliberately false. Configured charge state only; no automatic progression, native rate/security policy, secret-configured spans or other profiles.

### L90 `scriptobjects-DurationObject-HasStarted-090` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `DurationObject:HasStarted`

Occurrence input: `added:DurationObject.HasStarted`, historical status `best-effort` (not conclusion).

Assessment: Duration proxy already stores clock handle and uses duration-core queries. Addition chronology is separate from existing implementation.

Current provider: `src/lua_api/globals/lua_duration_object/core.rs:391`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/LuaDurationObjectAPIDocumentation.lua:366`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:84::test_patch_12_0_7_duration_objects_and_text_binding`; `tests/duration_core.rs:147::duration_core_secret_tainted_queries_and_mutations_reject_atomically`; `tests/duration_core.rs:245::duration_percent_tracks_clock_boundaries_and_rewind`; `tests/duration_core.rs:376::duration_core_manual_progress_rate_and_rewind`. These are located source, not fresh execution.

Model / required proof / missing evidence: Exercise same object before/after clock replacement, future/active/expired boundaries and clock movement, rooted clock identity and independent environments; authenticate secret clock/time arguments separately.

### L91 `scriptobjects-DurationObject-IsActive-091` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `DurationObject:IsActive`

Occurrence input: `added:DurationObject.IsActive`, historical status `best-effort` (not conclusion).

Assessment: Duration proxy already stores clock handle and uses duration-core queries. Addition chronology is separate from existing implementation.

Current provider: `src/lua_api/globals/lua_duration_object/core.rs:393`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/LuaDurationObjectAPIDocumentation.lua:382`; **later cache: may postdate 12.0.7**.

Existing tests: `src/lua_api/workarounds/temporary/inert_global_defaults.rs:263::installs_region_social_and_group_defaults`; `src/lua_api/workarounds/temporary/pool_constructor_defaults.rs:369::installs_pool_constructor_defaults`; `src/loader/tests/wow_api_globals/startup_globals.rs:84::test_patch_12_0_7_duration_objects_and_text_binding`; `tests/blizzard_text_status_bar_loads.rs:196::present_in_every_screen_eager_discovery`. These are located source, not fresh execution.

Model / required proof / missing evidence: Exercise same object before/after clock replacement, future/active/expired boundaries and clock movement, rooted clock identity and independent environments; authenticate secret clock/time arguments separately.

### L92 `scriptobjects-DurationObject-SetClock-092` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `DurationObject:SetClock`

Occurrence input: `added:DurationObject.SetClock`, historical status `best-effort` (not conclusion).

Assessment: Duration proxy already stores clock handle and uses duration-core queries. Addition chronology is separate from existing implementation.

Current provider: `src/lua_api/globals/lua_duration_object.rs:522`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/LuaDurationObjectAPIDocumentation.lua:421`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:84::test_patch_12_0_7_duration_objects_and_text_binding`; `tests/duration_core.rs:98::duration_core_secret_inputs_store_wrappers_and_preserve_lifecycle`; `tests/duration_core.rs:147::duration_core_secret_tainted_queries_and_mutations_reject_atomically`; `tests/duration_core.rs:245::duration_percent_tracks_clock_boundaries_and_rewind`. These are located source, not fresh execution.

Model / required proof / missing evidence: Exercise same object before/after clock replacement, future/active/expired boundaries and clock movement, rooted clock identity and independent environments; authenticate secret clock/time arguments separately.

### L93 `scriptobjects-DurationTextBinding-CanFormatText-093` — MODELABLE

Source: `DurationTextBinding:CanFormatText`

Occurrence input: `added:DurationTextBinding.CanFormatText`, historical status `best-effort` (not conclusion).

Assessment: CanFormatText currently returns true unconditionally; GetFormattedText has simplified numeric/default formatting and configuration-based updates, not a complete expiration/zero-duration/options formatter.

Current provider: `src/c_api/duration_text_binding.rs:155`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:22`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:84::test_patch_12_0_7_duration_objects_and_text_binding`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: binding configuration plus explicit clock, interval and elapsed schedule; format remaining value using configured expired/zero text, time modifier and one documented formatter. Test before-start/active/zero/expired, disabled/re-enabled cadence, invalid formatter failure and real FontString text. INFERRED: rounding, modifier units, expiry precedence and CanFormatText invalid-state predicate.

### L94 `scriptobjects-DurationTextBinding-CanUpdateFontString-094` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `DurationTextBinding:CanUpdateFontString`

Occurrence input: `added:DurationTextBinding.CanUpdateFontString`, historical status `best-effort` (not conclusion).

Assessment: Binding stores duration, font string, enabled flag and configuration fields; existing setters/getters and enable/disable hooks are real bounded state, not full native formatting proof.

Current provider: `src/c_api/duration_text_binding.rs:156`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:36`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:84::test_patch_12_0_7_duration_objects_and_text_binding`; `tests/duration_text_binding_copy.rs:344::duration_binding_assign_validates_atomically_clears_absent_values_and_handles_self_assignment`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove non-default setter/getter round trips, nil/default handling, font-string identity, object isolation and exact arity; for enable/disable verify actual tick suppression/resumption. INFERRED: validation/default policy not pinned historically.

### L95 `scriptobjects-DurationTextBinding-Disable-095` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `DurationTextBinding:Disable`

Occurrence input: `added:DurationTextBinding.Disable`, historical status `best-effort` (not conclusion).

Assessment: Binding stores duration, font string, enabled flag and configuration fields; existing setters/getters and enable/disable hooks are real bounded state, not full native formatting proof.

Current provider: `src/c_api/duration_text_binding.rs:157`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:74`; **later cache: may postdate 12.0.7**.

Existing tests: `src/addon_enable_state.rs:83::read_addon_enable_overrides_parses_addons_txt`; `src/ptr/compat_bootstrap.rs:128::patch_12_1_post_load_reapplies_epoch_enums_after_generated_docs_reset`; `src/lua_api/workarounds/temporary/inert_global_defaults.rs:263::installs_region_social_and_group_defaults`; `src/lua_api/workarounds/temporary/inert_global_defaults.rs:324::preserves_existing_members`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove non-default setter/getter round trips, nil/default handling, font-string identity, object isolation and exact arity; for enable/disable verify actual tick suppression/resumption. INFERRED: validation/default policy not pinned historically.

### L96 `scriptobjects-DurationTextBinding-Enable-096` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `DurationTextBinding:Enable`

Occurrence input: `added:DurationTextBinding.Enable`, historical status `best-effort` (not conclusion).

Assessment: Binding stores duration, font string, enabled flag and configuration fields; existing setters/getters and enable/disable hooks are real bounded state, not full native formatting proof.

Current provider: `src/c_api/duration_text_binding.rs:158`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:83`; **later cache: may postdate 12.0.7**.

Existing tests: `src/addon_enable_state.rs:83::read_addon_enable_overrides_parses_addons_txt`; `src/iced_app/render_hit_grid_tests.rs:16::scroll_offsets_move_cached_hit_target_without_moving_chrome_or_logical_rect`; `src/iced_app/render_hit_grid_tests.rs:118::nested_scroll_offsets_accumulate_in_cached_hits_and_clip_outside_outer_viewport`; `src/iced_app/render_hit_grid_tests.rs:215::layout_move_updates_cached_hit_grid_rects`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove non-default setter/getter round trips, nil/default handling, font-string identity, object isolation and exact arity; for enable/disable verify actual tick suppression/resumption. INFERRED: validation/default policy not pinned historically.

### L97 `scriptobjects-DurationTextBinding-GetDuration-097` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `DurationTextBinding:GetDuration`

Occurrence input: `added:DurationTextBinding.GetDuration`, historical status `best-effort` (not conclusion).

Assessment: Binding stores duration, font string, enabled flag and configuration fields; existing setters/getters and enable/disable hooks are real bounded state, not full native formatting proof.

Current provider: `src/c_api/duration_text_binding.rs:160`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:92`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:84::test_patch_12_0_7_duration_objects_and_text_binding`; `src/loader/tests/wow_api_globals/startup_globals.rs:184::test_patch_12_1_duration_binding_reference_lifetime_and_identity`; `tests/animation_anim.rs:335::animation_delays`; `tests/animation_factory.rs:6::create_animation_optional_arguments_return_one_owned_object`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove non-default setter/getter round trips, nil/default handling, font-string identity, object isolation and exact arity; for enable/disable verify actual tick suppression/resumption. INFERRED: validation/default policy not pinned historically.

### L98 `scriptobjects-DurationTextBinding-GetExpiredText-098` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `DurationTextBinding:GetExpiredText`

Occurrence input: `added:DurationTextBinding.GetExpiredText`, historical status `best-effort` (not conclusion).

Assessment: Binding stores duration, font string, enabled flag and configuration fields; existing setters/getters and enable/disable hooks are real bounded state, not full native formatting proof.

Current provider: `src/c_api/duration_text_binding.rs:161`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:106`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:84::test_patch_12_0_7_duration_objects_and_text_binding`; `tests/duration_text_binding_copy.rs:186::duration_binding_assign_preserves_receiver_and_configuration_handles`; `tests/duration_text_binding_copy.rs:259::duration_binding_copy_updates_text_with_independent_configuration`; `tests/duration_text_binding_copy.rs:344::duration_binding_assign_validates_atomically_clears_absent_values_and_handles_self_assignment`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove non-default setter/getter round trips, nil/default handling, font-string identity, object isolation and exact arity; for enable/disable verify actual tick suppression/resumption. INFERRED: validation/default policy not pinned historically.

### L99 `scriptobjects-DurationTextBinding-GetFontString-099` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `DurationTextBinding:GetFontString`

Occurrence input: `added:DurationTextBinding.GetFontString`, historical status `best-effort` (not conclusion).

Assessment: Binding stores duration, font string, enabled flag and configuration fields; existing setters/getters and enable/disable hooks are real bounded state, not full native formatting proof.

Current provider: `src/c_api/duration_text_binding.rs:162`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:120`; **later cache: may postdate 12.0.7**.

Existing tests: `src/c_api/duration_text_binding.rs:353::duration_binding_availability_preserves_client_versions`; `src/loader/tests/button_text.rs:4::test_inherited_button_text_defaults_to_center_anchor_when_anchors_omitted`; `src/loader/tests/xml_basics.rs:116::test_inherited_button_text_is_available_immediately_after_load`; `src/loader/tests/wow_api_globals/frames_and_attributes.rs:373::test_get_font_string_metatable_exposes_text_methods`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove non-default setter/getter round trips, nil/default handling, font-string identity, object isolation and exact arity; for enable/disable verify actual tick suppression/resumption. INFERRED: validation/default policy not pinned historically.

### L100 `scriptobjects-DurationTextBinding-GetFormattedText-100` — MODELABLE

Source: `DurationTextBinding:GetFormattedText`

Occurrence input: `added:DurationTextBinding.GetFormattedText`, historical status `best-effort` (not conclusion).

Assessment: CanFormatText currently returns true unconditionally; GetFormattedText has simplified numeric/default formatting and configuration-based updates, not a complete expiration/zero-duration/options formatter.

Current provider: `src/c_api/duration_text_binding.rs:163`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:134`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:84::test_patch_12_0_7_duration_objects_and_text_binding`; `src/loader/tests/wow_api_globals/startup_globals.rs:772::test_patch_12_1_safe_global_bridges`; `tests/numeric_rule_formatter.rs:155::numeric_rule_formatter_updates_duration_binding_text`; `tests/duration_text_binding_copy.rs:7::duration_binding_secret_duration_preserves_font_string_boundary`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: binding configuration plus explicit clock, interval and elapsed schedule; format remaining value using configured expired/zero text, time modifier and one documented formatter. Test before-start/active/zero/expired, disabled/re-enabled cadence, invalid formatter failure and real FontString text. INFERRED: rounding, modifier units, expiry precedence and CanFormatText invalid-state predicate.

### L101 `scriptobjects-DurationTextBinding-GetTimeModifier-101` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `DurationTextBinding:GetTimeModifier`

Occurrence input: `added:DurationTextBinding.GetTimeModifier`, historical status `best-effort` (not conclusion).

Assessment: Binding stores duration, font string, enabled flag and configuration fields; existing setters/getters and enable/disable hooks are real bounded state, not full native formatting proof.

Current provider: `src/c_api/duration_text_binding.rs:182`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:179`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:84::test_patch_12_0_7_duration_objects_and_text_binding`; `tests/duration_text_binding_copy.rs:186::duration_binding_assign_preserves_receiver_and_configuration_handles`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove non-default setter/getter round trips, nil/default handling, font-string identity, object isolation and exact arity; for enable/disable verify actual tick suppression/resumption. INFERRED: validation/default policy not pinned historically.

### L102 `scriptobjects-DurationTextBinding-GetUpdateInterval-102` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `DurationTextBinding:GetUpdateInterval`

Occurrence input: `added:DurationTextBinding.GetUpdateInterval`, historical status `best-effort` (not conclusion).

Assessment: Binding stores duration, font string, enabled flag and configuration fields; existing setters/getters and enable/disable hooks are real bounded state, not full native formatting proof.

Current provider: `src/c_api/duration_text_binding.rs:183`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:193`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:84::test_patch_12_0_7_duration_objects_and_text_binding`; `tests/duration_text_binding_copy.rs:186::duration_binding_assign_preserves_receiver_and_configuration_handles`; `tests/duration_text_binding_copy.rs:259::duration_binding_copy_updates_text_with_independent_configuration`; `tests/duration_text_binding_copy.rs:382::duration_binding_assignment_supports_actual_custom_aura_button_duration_text`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove non-default setter/getter round trips, nil/default handling, font-string identity, object isolation and exact arity; for enable/disable verify actual tick suppression/resumption. INFERRED: validation/default policy not pinned historically.

### L103 `scriptobjects-DurationTextBinding-GetZeroDurationText-103` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `DurationTextBinding:GetZeroDurationText`

Occurrence input: `added:DurationTextBinding.GetZeroDurationText`, historical status `best-effort` (not conclusion).

Assessment: Binding stores duration, font string, enabled flag and configuration fields; existing setters/getters and enable/disable hooks are real bounded state, not full native formatting proof.

Current provider: `src/c_api/duration_text_binding.rs:184`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:207`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:84::test_patch_12_0_7_duration_objects_and_text_binding`; `tests/duration_text_binding_copy.rs:186::duration_binding_assign_preserves_receiver_and_configuration_handles`; `tests/duration_text_binding_copy.rs:259::duration_binding_copy_updates_text_with_independent_configuration`; `tests/duration_text_binding_copy.rs:344::duration_binding_assign_validates_atomically_clears_absent_values_and_handles_self_assignment`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove non-default setter/getter round trips, nil/default handling, font-string identity, object isolation and exact arity; for enable/disable verify actual tick suppression/resumption. INFERRED: validation/default policy not pinned historically.

### L104 `scriptobjects-DurationTextBinding-IsEnabled-104` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `DurationTextBinding:IsEnabled`

Occurrence input: `added:DurationTextBinding.IsEnabled`, historical status `best-effort` (not conclusion).

Assessment: Binding stores duration, font string, enabled flag and configuration fields; existing setters/getters and enable/disable hooks are real bounded state, not full native formatting proof.

Current provider: `src/c_api/duration_text_binding.rs:189`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:236`; **later cache: may postdate 12.0.7**.

Existing tests: `src/lua_api/workarounds/temporary/store_public_defaults.rs:67::installs_store_public_defaults`; `src/lua_api/workarounds/temporary/kiosk_namespace_defaults.rs:40::installs_inert_kiosk_defaults`; `src/lua_api/workarounds/temporary/kiosk_namespace_defaults.rs:71::preserves_existing_kiosk_members`; `src/lua_api/globals/create_frame/template_chain/parser_inline_sequence.rs:247::keeps_local_prelude_with_following_if_block`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove non-default setter/getter round trips, nil/default handling, font-string identity, object isolation and exact arity; for enable/disable verify actual tick suppression/resumption. INFERRED: validation/default policy not pinned historically.

### L105 `scriptobjects-DurationTextBinding-SetDuration-105` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `DurationTextBinding:SetDuration`

Occurrence input: `added:DurationTextBinding.SetDuration`, historical status `best-effort` (not conclusion).

Assessment: Binding stores duration, font string, enabled flag and configuration fields; existing setters/getters and enable/disable hooks are real bounded state, not full native formatting proof.

Current provider: `src/c_api/duration_text_binding.rs:191`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:250`; **later cache: may postdate 12.0.7**.

Existing tests: `src/c_api/duration_text_binding.rs:353::duration_binding_availability_preserves_client_versions`; `src/loader/tests/wow_api_globals/startup_globals.rs:84::test_patch_12_0_7_duration_objects_and_text_binding`; `src/loader/tests/wow_api_globals/startup_globals.rs:184::test_patch_12_1_duration_binding_reference_lifetime_and_identity`; `src/loader/tests/wow_api_globals/runtime_subsystems.rs:174::test_animation_runtime_exposes_core_configuration_methods`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove non-default setter/getter round trips, nil/default handling, font-string identity, object isolation and exact arity; for enable/disable verify actual tick suppression/resumption. INFERRED: validation/default policy not pinned historically.

### L106 `scriptobjects-DurationTextBinding-SetExpiredText-106` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `DurationTextBinding:SetExpiredText`

Occurrence input: `added:DurationTextBinding.SetExpiredText`, historical status `best-effort` (not conclusion).

Assessment: Binding stores duration, font string, enabled flag and configuration fields; existing setters/getters and enable/disable hooks are real bounded state, not full native formatting proof.

Current provider: `src/c_api/duration_text_binding.rs:193`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:272`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:84::test_patch_12_0_7_duration_objects_and_text_binding`; `tests/duration_text_binding_copy.rs:186::duration_binding_assign_preserves_receiver_and_configuration_handles`; `tests/duration_text_binding_copy.rs:259::duration_binding_copy_updates_text_with_independent_configuration`; `tests/duration_text_binding_copy.rs:344::duration_binding_assign_validates_atomically_clears_absent_values_and_handles_self_assignment`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove non-default setter/getter round trips, nil/default handling, font-string identity, object isolation and exact arity; for enable/disable verify actual tick suppression/resumption. INFERRED: validation/default policy not pinned historically.

### L107 `scriptobjects-DurationTextBinding-SetFontString-107` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `DurationTextBinding:SetFontString`

Occurrence input: `added:DurationTextBinding.SetFontString`, historical status `best-effort` (not conclusion).

Assessment: Binding stores duration, font string, enabled flag and configuration fields; existing setters/getters and enable/disable hooks are real bounded state, not full native formatting proof.

Current provider: `src/c_api/duration_text_binding.rs:194`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:283`; **later cache: may postdate 12.0.7**.

Existing tests: `src/c_api/duration_text_binding.rs:353::duration_binding_availability_preserves_client_versions`; `src/loader/tests/wow_api_globals/startup_globals.rs:84::test_patch_12_0_7_duration_objects_and_text_binding`; `tests/numeric_rule_formatter.rs:155::numeric_rule_formatter_updates_duration_binding_text`; `tests/duration_text_binding_tick.rs:62::automatic_binding_copy_assign_and_reset_have_independent_schedules`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove non-default setter/getter round trips, nil/default handling, font-string identity, object isolation and exact arity; for enable/disable verify actual tick suppression/resumption. INFERRED: validation/default policy not pinned historically.

### L108 `scriptobjects-DurationTextBinding-SetTimeModifier-108` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `DurationTextBinding:SetTimeModifier`

Occurrence input: `added:DurationTextBinding.SetTimeModifier`, historical status `best-effort` (not conclusion).

Assessment: Binding stores duration, font string, enabled flag and configuration fields; existing setters/getters and enable/disable hooks are real bounded state, not full native formatting proof.

Current provider: `src/c_api/duration_text_binding.rs:200`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:329`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:84::test_patch_12_0_7_duration_objects_and_text_binding`; `tests/duration_text_binding_copy.rs:186::duration_binding_assign_preserves_receiver_and_configuration_handles`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove non-default setter/getter round trips, nil/default handling, font-string identity, object isolation and exact arity; for enable/disable verify actual tick suppression/resumption. INFERRED: validation/default policy not pinned historically.

### L109 `scriptobjects-DurationTextBinding-SetUpdateInterval-109` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `DurationTextBinding:SetUpdateInterval`

Occurrence input: `added:DurationTextBinding.SetUpdateInterval`, historical status `best-effort` (not conclusion).

Assessment: Binding stores duration, font string, enabled flag and configuration fields; existing setters/getters and enable/disable hooks are real bounded state, not full native formatting proof.

Current provider: `src/c_api/duration_text_binding.rs:213`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:349`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:84::test_patch_12_0_7_duration_objects_and_text_binding`; `tests/duration_text_binding_tick.rs:43::automatic_binding_disable_reenable_and_configuration_invalidate_interval`; `tests/duration_text_binding_tick.rs:62::automatic_binding_copy_assign_and_reset_have_independent_schedules`; `tests/duration_text_binding_copy.rs:186::duration_binding_assign_preserves_receiver_and_configuration_handles`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove non-default setter/getter round trips, nil/default handling, font-string identity, object isolation and exact arity; for enable/disable verify actual tick suppression/resumption. INFERRED: validation/default policy not pinned historically.

### L110 `scriptobjects-DurationTextBinding-SetZeroDurationText-110` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `DurationTextBinding:SetZeroDurationText`

Occurrence input: `added:DurationTextBinding.SetZeroDurationText`, historical status `best-effort` (not conclusion).

Assessment: Binding stores duration, font string, enabled flag and configuration fields; existing setters/getters and enable/disable hooks are real bounded state, not full native formatting proof.

Current provider: `src/c_api/duration_text_binding.rs:214`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/DurationTextBindingObjectAPIDocumentation.lua:360`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:84::test_patch_12_0_7_duration_objects_and_text_binding`; `tests/duration_text_binding_copy.rs:186::duration_binding_assign_preserves_receiver_and_configuration_handles`; `tests/duration_text_binding_copy.rs:259::duration_binding_copy_updates_text_with_independent_configuration`; `tests/duration_text_binding_copy.rs:344::duration_binding_assign_validates_atomically_clears_absent_values_and_handles_self_assignment`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove non-default setter/getter round trips, nil/default handling, font-string identity, object isolation and exact arity; for enable/disable verify actual tick suppression/resumption. INFERRED: validation/default policy not pinned historically.

### L111 `scriptobjects-DurationTextFormattingOptions-GetAddRemainingText-111` — BLOCKED

Source: `DurationTextFormattingOptions:GetAddRemainingText`

Occurrence input: `added:DurationTextFormattingOptions.GetAddRemainingText`, historical status `best-effort` (not conclusion).

Assessment: No retained cache declaration or current simulator provider for this exact receiver; startup test explicitly expects the guessed factories to return nil. That is absence proof, not implemented methods.

Current provider: No exact receiver/symbol provider located; neighboring subsystem only, no callable credit..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_0_7_duration_objects_and_text_binding. These are located source, not fresh execution.

Model / required proof / missing evidence: Missing evidence: historical 12.0.7 type/factory reachability, receiver identity, defaults and method declarations. Do not invent CreateDurationTextFormattingOptions/CreateDurationTextRawValue. If authenticated, smallest model would store duration-type/addRemaining flags or one seconds scalar with ms conversion; test factory reachability, non-default round trips and object independence.

### L112 `scriptobjects-DurationTextFormattingOptions-GetDurationType-112` — BLOCKED

Source: `DurationTextFormattingOptions:GetDurationType`

Occurrence input: `added:DurationTextFormattingOptions.GetDurationType`, historical status `best-effort` (not conclusion).

Assessment: No retained cache declaration or current simulator provider for this exact receiver; startup test explicitly expects the guessed factories to return nil. That is absence proof, not implemented methods.

Current provider: No exact receiver/symbol provider located; neighboring subsystem only, no callable credit..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_0_7_duration_objects_and_text_binding. These are located source, not fresh execution.

Model / required proof / missing evidence: Missing evidence: historical 12.0.7 type/factory reachability, receiver identity, defaults and method declarations. Do not invent CreateDurationTextFormattingOptions/CreateDurationTextRawValue. If authenticated, smallest model would store duration-type/addRemaining flags or one seconds scalar with ms conversion; test factory reachability, non-default round trips and object independence.

### L113 `scriptobjects-DurationTextFormattingOptions-SetAddRemainingText-113` — BLOCKED

Source: `DurationTextFormattingOptions:SetAddRemainingText`

Occurrence input: `added:DurationTextFormattingOptions.SetAddRemainingText`, historical status `best-effort` (not conclusion).

Assessment: No retained cache declaration or current simulator provider for this exact receiver; startup test explicitly expects the guessed factories to return nil. That is absence proof, not implemented methods.

Current provider: No exact receiver/symbol provider located; neighboring subsystem only, no callable credit..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_0_7_duration_objects_and_text_binding. These are located source, not fresh execution.

Model / required proof / missing evidence: Missing evidence: historical 12.0.7 type/factory reachability, receiver identity, defaults and method declarations. Do not invent CreateDurationTextFormattingOptions/CreateDurationTextRawValue. If authenticated, smallest model would store duration-type/addRemaining flags or one seconds scalar with ms conversion; test factory reachability, non-default round trips and object independence.

### L114 `scriptobjects-DurationTextFormattingOptions-SetDurationType-114` — BLOCKED

Source: `DurationTextFormattingOptions:SetDurationType`

Occurrence input: `added:DurationTextFormattingOptions.SetDurationType`, historical status `best-effort` (not conclusion).

Assessment: No retained cache declaration or current simulator provider for this exact receiver; startup test explicitly expects the guessed factories to return nil. That is absence proof, not implemented methods.

Current provider: No exact receiver/symbol provider located; neighboring subsystem only, no callable credit..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_0_7_duration_objects_and_text_binding. These are located source, not fresh execution.

Model / required proof / missing evidence: Missing evidence: historical 12.0.7 type/factory reachability, receiver identity, defaults and method declarations. Do not invent CreateDurationTextFormattingOptions/CreateDurationTextRawValue. If authenticated, smallest model would store duration-type/addRemaining flags or one seconds scalar with ms conversion; test factory reachability, non-default round trips and object independence.

### L115 `scriptobjects-DurationTextRawValue-GetMilliseconds-115` — BLOCKED

Source: `DurationTextRawValue:GetMilliseconds`

Occurrence input: `added:DurationTextRawValue.GetMilliseconds`, historical status `best-effort` (not conclusion).

Assessment: No retained cache declaration or current simulator provider for this exact receiver; startup test explicitly expects the guessed factories to return nil. That is absence proof, not implemented methods.

Current provider: No exact receiver/symbol provider located; neighboring subsystem only, no callable credit..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `tests/seconds_formatter_configuration.rs:210::seconds_formatter_configuration_is_independent_and_has_exact_arity`; `tests/seconds_formatter_configuration.rs:244::seconds_formatter_configuration_rejects_invalid_values_atomically`; `tests/seconds_formatter_configuration.rs:270::seconds_formatter_configuration_survives_gc_and_preserves_existing_methods`; `tests/seconds_formatter_format.rs:111::seconds_formatter_format_rejects_invalid_state_without_mutating_it`. These are located source, not fresh execution.

Model / required proof / missing evidence: Missing evidence: historical 12.0.7 type/factory reachability, receiver identity, defaults and method declarations. Do not invent CreateDurationTextFormattingOptions/CreateDurationTextRawValue. If authenticated, smallest model would store duration-type/addRemaining flags or one seconds scalar with ms conversion; test factory reachability, non-default round trips and object independence.

### L116 `scriptobjects-DurationTextRawValue-GetSeconds-116` — BLOCKED

Source: `DurationTextRawValue:GetSeconds`

Occurrence input: `added:DurationTextRawValue.GetSeconds`, historical status `best-effort` (not conclusion).

Assessment: No retained cache declaration or current simulator provider for this exact receiver; startup test explicitly expects the guessed factories to return nil. That is absence proof, not implemented methods.

Current provider: No exact receiver/symbol provider located; neighboring subsystem only, no callable credit..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `src/lua_api/workarounds/temporary/date_and_time_defaults.rs:185::preserves_existing_date_and_time_provider`; `src/lua_api/workarounds/temporary/game_time_calendar_invites.rs:111::installs_missing_game_time_globals`; `tests/c_map_api.rs:410::test_get_seconds_until_daily_reset`; `tests/c_map_api.rs:419::test_get_seconds_until_weekly_reset`. These are located source, not fresh execution.

Model / required proof / missing evidence: Missing evidence: historical 12.0.7 type/factory reachability, receiver identity, defaults and method declarations. Do not invent CreateDurationTextFormattingOptions/CreateDurationTextRawValue. If authenticated, smallest model would store duration-type/addRemaining flags or one seconds scalar with ms conversion; test factory reachability, non-default round trips and object independence.

### L117 `scriptobjects-DurationTextRawValue-SetMilliseconds-117` — BLOCKED

Source: `DurationTextRawValue:SetMilliseconds`

Occurrence input: `added:DurationTextRawValue.SetMilliseconds`, historical status `best-effort` (not conclusion).

Assessment: No retained cache declaration or current simulator provider for this exact receiver; startup test explicitly expects the guessed factories to return nil. That is absence proof, not implemented methods.

Current provider: No exact receiver/symbol provider located; neighboring subsystem only, no callable credit..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `tests/seconds_formatter_configuration.rs:59::seconds_formatter_evaluation_uses_current_configuration`; `tests/seconds_formatter_configuration.rs:210::seconds_formatter_configuration_is_independent_and_has_exact_arity`; `tests/seconds_formatter_configuration.rs:244::seconds_formatter_configuration_rejects_invalid_values_atomically`; `tests/seconds_formatter_configuration.rs:270::seconds_formatter_configuration_survives_gc_and_preserves_existing_methods`. These are located source, not fresh execution.

Model / required proof / missing evidence: Missing evidence: historical 12.0.7 type/factory reachability, receiver identity, defaults and method declarations. Do not invent CreateDurationTextFormattingOptions/CreateDurationTextRawValue. If authenticated, smallest model would store duration-type/addRemaining flags or one seconds scalar with ms conversion; test factory reachability, non-default round trips and object independence.

### L118 `scriptobjects-DurationTextRawValue-SetSeconds-118` — BLOCKED

Source: `DurationTextRawValue:SetSeconds`

Occurrence input: `added:DurationTextRawValue.SetSeconds`, historical status `best-effort` (not conclusion).

Assessment: No retained cache declaration or current simulator provider for this exact receiver; startup test explicitly expects the guessed factories to return nil. That is absence proof, not implemented methods.

Current provider: No exact receiver/symbol provider located; neighboring subsystem only, no callable credit..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_0_7_duration_objects_and_text_binding. These are located source, not fresh execution.

Model / required proof / missing evidence: Missing evidence: historical 12.0.7 type/factory reachability, receiver identity, defaults and method declarations. Do not invent CreateDurationTextFormattingOptions/CreateDurationTextRawValue. If authenticated, smallest model would store duration-type/addRemaining flags or one seconds scalar with ms conversion; test factory reachability, non-default round trips and object independence.

### L120 `source-context-120` — METADATA-ONLY

Source: `Widgets Removed (6):`

Assessment / exact one-sentence metadata note: Section heading and stated inventory count organize the retained excerpt only; no runtime or implementation credit.

Current provider: No exact provider located for this row; no provider is required for editorial metadata..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

### L121 `widgets-Minimap-SetBlipTexture-121` — MODELABLE

Source: `Minimap:SetBlipTexture`

Occurrence input: `removed:Minimap.SetBlipTexture`, historical status `best-effort` (not conclusion).

Assessment: Six removed methods are still in the simulator map-frame method registry; actual method-call publication and advertised metatable differ in this simulator.

Current provider: `src/lua_api/frame/methods/map_frames.rs:62`.

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `src/loader/tests/minimap_specialized.rs:5::test_minimap_texture_setters_persist_asset_state`; `src/loader/tests/wow_api_globals/startup_globals.rs:391::test_patch_12_0_7_widget_compatibility_surface`; `tests/global_frames.rs:50::test_map_frame_method_surface_is_registered`. These are located source, not fresh execution.

Model / required proof / missing evidence: Gate these six native methods at the 12.0.7 boundary, preserving older profiles only. Test actual calls and metatable visibility before/after full load, positive old-profile controls and no unrelated minimap regression. INFERRED: historical deprecation-wrapper availability; current cache absence alone is not sufficient historical proof.

### L122 `widgets-Minimap-SetCorpsePOIArrowTexture-122` — MODELABLE

Source: `Minimap:SetCorpsePOIArrowTexture`

Occurrence input: `removed:Minimap.SetCorpsePOIArrowTexture`, historical status `best-effort` (not conclusion).

Assessment: Six removed methods are still in the simulator map-frame method registry; actual method-call publication and advertised metatable differ in this simulator.

Current provider: `src/lua_api/frame/methods/map_frames.rs:66`.

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `src/loader/tests/minimap_specialized.rs:5::test_minimap_texture_setters_persist_asset_state`; `src/loader/tests/wow_api_globals/startup_globals.rs:391::test_patch_12_0_7_widget_compatibility_surface`; `tests/global_frames.rs:50::test_map_frame_method_surface_is_registered`. These are located source, not fresh execution.

Model / required proof / missing evidence: Gate these six native methods at the 12.0.7 boundary, preserving older profiles only. Test actual calls and metatable visibility before/after full load, positive old-profile controls and no unrelated minimap regression. INFERRED: historical deprecation-wrapper availability; current cache absence alone is not sufficient historical proof.

### L123 `widgets-Minimap-SetIconTexture-123` — MODELABLE

Source: `Minimap:SetIconTexture`

Occurrence input: `removed:Minimap.SetIconTexture`, historical status `best-effort` (not conclusion).

Assessment: Six removed methods are still in the simulator map-frame method registry; actual method-call publication and advertised metatable differ in this simulator.

Current provider: `src/lua_api/frame/methods/map_frames.rs:64`.

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `src/loader/tests/minimap_specialized.rs:5::test_minimap_texture_setters_persist_asset_state`; `src/loader/tests/wow_api_globals/startup_globals.rs:391::test_patch_12_0_7_widget_compatibility_surface`; `tests/global_frames.rs:50::test_map_frame_method_surface_is_registered`. These are located source, not fresh execution.

Model / required proof / missing evidence: Gate these six native methods at the 12.0.7 boundary, preserving older profiles only. Test actual calls and metatable visibility before/after full load, positive old-profile controls and no unrelated minimap regression. INFERRED: historical deprecation-wrapper availability; current cache absence alone is not sufficient historical proof.

### L124 `widgets-Minimap-SetPOIArrowTexture-124` — MODELABLE

Source: `Minimap:SetPOIArrowTexture`

Occurrence input: `removed:Minimap.SetPOIArrowTexture`, historical status `best-effort` (not conclusion).

Assessment: Six removed methods are still in the simulator map-frame method registry; actual method-call publication and advertised metatable differ in this simulator.

Current provider: `src/lua_api/frame/methods/map_frames.rs:65`.

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `src/loader/tests/minimap_specialized.rs:5::test_minimap_texture_setters_persist_asset_state`; `src/loader/tests/wow_api_globals/startup_globals.rs:391::test_patch_12_0_7_widget_compatibility_surface`; `tests/global_frames.rs:50::test_map_frame_method_surface_is_registered`. These are located source, not fresh execution.

Model / required proof / missing evidence: Gate these six native methods at the 12.0.7 boundary, preserving older profiles only. Test actual calls and metatable visibility before/after full load, positive old-profile controls and no unrelated minimap regression. INFERRED: historical deprecation-wrapper availability; current cache absence alone is not sufficient historical proof.

### L125 `widgets-Minimap-SetPlayerTexture-125` — MODELABLE

Source: `Minimap:SetPlayerTexture`

Occurrence input: `removed:Minimap.SetPlayerTexture`, historical status `best-effort` (not conclusion).

Assessment: Six removed methods are still in the simulator map-frame method registry; actual method-call publication and advertised metatable differ in this simulator.

Current provider: `src/lua_api/frame/methods/map_frames.rs:68`.

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `src/loader/tests/minimap_specialized.rs:57::test_minimap_player_texture_and_defaults_follow_runtime_state`; `src/loader/tests/wow_api_globals/startup_globals.rs:391::test_patch_12_0_7_widget_compatibility_surface`; `tests/global_frames.rs:50::test_map_frame_method_surface_is_registered`; `tests/c_map_api.rs:464::test_minimap_set_player_texture_no_error`. These are located source, not fresh execution.

Model / required proof / missing evidence: Gate these six native methods at the 12.0.7 boundary, preserving older profiles only. Test actual calls and metatable visibility before/after full load, positive old-profile controls and no unrelated minimap regression. INFERRED: historical deprecation-wrapper availability; current cache absence alone is not sufficient historical proof.

### L126 `widgets-Minimap-SetStaticPOIArrowTexture-126` — MODELABLE

Source: `Minimap:SetStaticPOIArrowTexture`

Occurrence input: `removed:Minimap.SetStaticPOIArrowTexture`, historical status `best-effort` (not conclusion).

Assessment: Six removed methods are still in the simulator map-frame method registry; actual method-call publication and advertised metatable differ in this simulator.

Current provider: `src/lua_api/frame/methods/map_frames.rs:67`.

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `src/loader/tests/minimap_specialized.rs:5::test_minimap_texture_setters_persist_asset_state`; `src/loader/tests/wow_api_globals/startup_globals.rs:391::test_patch_12_0_7_widget_compatibility_surface`; `tests/global_frames.rs:50::test_map_frame_method_surface_is_registered`. These are located source, not fresh execution.

Model / required proof / missing evidence: Gate these six native methods at the 12.0.7 boundary, preserving older profiles only. Test actual calls and metatable visibility before/after full load, positive old-profile controls and no unrelated minimap regression. INFERRED: historical deprecation-wrapper availability; current cache absence alone is not sufficient historical proof.

### L128 `source-context-128` — METADATA-ONLY

Source: `Widget Changes:`

Assessment / exact one-sentence metadata note: Section heading and stated inventory count organize the retained excerpt only; no runtime or implementation credit.

Current provider: No exact provider located for this row; no provider is required for editorial metadata..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

### L129 `widgets-Button-GetButtonState-129` — MODELABLE

Source: `Button:GetButtonState + SecretReturnsForAspect`

Occurrence input: `changed:Button.GetButtonState`, historical status `best-effort` (not conclusion).

Assessment: Current button getters push plain strings/bools; setters decode ordinary inputs and do not visibly attach ButtonState secret aspect. New delta is behavior, not documentation-only.

Current provider: `src/lua_api/frame/methods/button_anchor_hierarchy/buttons.rs:187`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleButtonAPIDocumentation.lua:72`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:391::test_patch_12_0_7_widget_compatibility_surface`; `tests/spell_casting.rs:180::action_button_down_sets_pushed_state`; `tests/spell_casting.rs:207::button_state_pushed_during_keypress`; `tests/c_action_bar_input_probes.rs:60::action_button_down_pushes_state_and_fires_cast`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: existing frame button state/enabled fields plus per-frame ButtonState secret aspect; authenticate secret arguments before mutation, mark aspect on accepted input, wrap getters only when aspect is secret. Test actual VM secrets with secure/tainted callers, secret getter identity, denied atomicity, enable/disable callbacks and other-frame controls. INFERRED: aspect reset/public-overwrite lifecycle until native evidence.

### L130 `widgets-Button-IsEnabled-130` — MODELABLE

Source: `Button:IsEnabled + SecretReturnsForAspect`

Occurrence input: `changed:Button.IsEnabled`, historical status `best-effort` (not conclusion).

Assessment: Current button getters push plain strings/bools; setters decode ordinary inputs and do not visibly attach ButtonState secret aspect. New delta is behavior, not documentation-only.

Current provider: `src/lua_api/frame/methods/button_anchor_hierarchy/buttons.rs:72`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleButtonAPIDocumentation.lua:257`; **later cache: may postdate 12.0.7**.

Existing tests: `src/lua_api/workarounds/temporary/store_public_defaults.rs:67::installs_store_public_defaults`; `src/lua_api/workarounds/temporary/kiosk_namespace_defaults.rs:40::installs_inert_kiosk_defaults`; `src/lua_api/workarounds/temporary/kiosk_namespace_defaults.rs:71::preserves_existing_kiosk_members`; `src/lua_api/globals/create_frame/template_chain/parser_inline_sequence.rs:247::keeps_local_prelude_with_following_if_block`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: existing frame button state/enabled fields plus per-frame ButtonState secret aspect; authenticate secret arguments before mutation, mark aspect on accepted input, wrap getters only when aspect is secret. Test actual VM secrets with secure/tainted callers, secret getter identity, denied atomicity, enable/disable callbacks and other-frame controls. INFERRED: aspect reset/public-overwrite lifecycle until native evidence.

### L131 `widgets-Button-SetButtonState-131` — MODELABLE

Source: `Button:SetButtonState + SecretArgumentsAddAspect`

Occurrence input: `changed:Button.SetButtonState`, historical status `best-effort` (not conclusion).

Assessment: Current button getters push plain strings/bools; setters decode ordinary inputs and do not visibly attach ButtonState secret aspect. New delta is behavior, not documentation-only.

Current provider: `src/lua_api/frame/methods/button_anchor_hierarchy/buttons.rs:173`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleButtonAPIDocumentation.lua:293`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:391::test_patch_12_0_7_widget_compatibility_surface`; `tests/key_dispatch.rs:230::action_button_key_dispatch_casts_once_through_button_down`; `tests/spell_casting.rs:180::action_button_down_sets_pushed_state`; `tests/spell_casting.rs:207::button_state_pushed_during_keypress`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: existing frame button state/enabled fields plus per-frame ButtonState secret aspect; authenticate secret arguments before mutation, mark aspect on accepted input, wrap getters only when aspect is secret. Test actual VM secrets with secure/tainted callers, secret getter identity, denied atomicity, enable/disable callbacks and other-frame controls. INFERRED: aspect reset/public-overwrite lifecycle until native evidence.

### L132 `widgets-Button-SetEnabled-132` — MODELABLE

Source: `Button:SetEnabled + SecretArgumentsAddAspect, SecretArguments NotAllowed -> AllowedWhenUntainted`

Occurrence input: `changed:Button.SetEnabled`, historical status `best-effort` (not conclusion).

Assessment: Current button getters push plain strings/bools; setters decode ordinary inputs and do not visibly attach ButtonState secret aspect. New delta is behavior, not documentation-only.

Current provider: `src/lua_api/frame/methods/button_anchor_hierarchy/buttons.rs:98`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleButtonAPIDocumentation.lua:336`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:84::test_patch_12_0_7_duration_objects_and_text_binding`; `src/loader/tests/wow_api_globals/startup_globals.rs:391::test_patch_12_0_7_widget_compatibility_surface`; `tests/blizzard_report_frame_loads.rs:186::root_directory_holds_two_files_next_to_toc`; `tests/blizzard_report_frame_glue_loads.rs:364::glue_overrides_publish_with_glue_specific_semantics`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: existing frame button state/enabled fields plus per-frame ButtonState secret aspect; authenticate secret arguments before mutation, mark aspect on accepted input, wrap getters only when aspect is secret. Test actual VM secrets with secure/tainted callers, secret getter identity, denied atomicity, enable/disable callbacks and other-frame controls. INFERRED: aspect reset/public-overwrite lifecycle until native evidence.

### L133 `widgets-EditBox-SetFont-133` — MODELABLE

Source: `EditBox:SetFont + RequiresValidFontHeight + RequiresValidFontAsset`

Occurrence input: `changed:EditBox.SetFont`, historical status `best-effort` (not conclusion).

Assessment: Current font setters accept unresolved paths and unchecked heights; 12.0.5 fontstring-setfont-shape explicitly excludes asset and height validation, so it cannot cover this 12.0.7 delta.

Current provider: `src/lua_api/frame/methods/text_attribute_event/text/formatting.rs:330`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleEditBoxAPIDocumentation.lua:677`; **later cache: may postdate 12.0.7**.

Existing tests: `src/c_api/duration_text_binding.rs:353::duration_binding_availability_preserves_client_versions`; `src/lua_api/workarounds/temporary/formatting_utility_defaults.rs:323::installs_formatting_utility_defaults`; `src/loader/tests/xml_text_region_defaults.rs:109::justify_probe_message_regions_stay_absent_with_owner_insets`; `src/loader/tests/wow_api_globals/startup_globals.rs:84::test_patch_12_0_7_duration_objects_and_text_binding`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: existing font asset resolver + explicit validated height before changing path/height/flags, shared across five receiver kinds without altering vendor Lua. Test known valid asset/nonintegral height, unknown asset, invalid/nonfinite height, unchanged state on failure and exact result tuple per receiver. INFERRED: allowable bounds, nil/flags/error policy and FontAsset domain require historical/native evidence.

### L134 `widgets-Font-SetFont-134` — MODELABLE

Source: `Font:SetFont + RequiresValidFontHeight + RequiresValidFontAsset`

Occurrence input: `changed:Font.SetFont`, historical status `best-effort` (not conclusion).

Assessment: Current font setters accept unresolved paths and unchecked heights; 12.0.5 fontstring-setfont-shape explicitly excludes asset and height validation, so it cannot cover this 12.0.7 delta.

Current provider: `src/lua_api/globals/font_strings_collection/fonts.rs:102`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleFontAPIDocumentation.lua:199`; **later cache: may postdate 12.0.7**.

Existing tests: `src/c_api/duration_text_binding.rs:353::duration_binding_availability_preserves_client_versions`; `src/lua_api/workarounds/temporary/formatting_utility_defaults.rs:323::installs_formatting_utility_defaults`; `src/loader/tests/xml_text_region_defaults.rs:109::justify_probe_message_regions_stay_absent_with_owner_insets`; `src/loader/tests/wow_api_globals/startup_globals.rs:84::test_patch_12_0_7_duration_objects_and_text_binding`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: existing font asset resolver + explicit validated height before changing path/height/flags, shared across five receiver kinds without altering vendor Lua. Test known valid asset/nonintegral height, unknown asset, invalid/nonfinite height, unchanged state on failure and exact result tuple per receiver. INFERRED: allowable bounds, nil/flags/error policy and FontAsset domain require historical/native evidence.

### L135 `widgets-FontString-SetFont-135` — MODELABLE

Source: `FontString:SetFont + RequiresValidFontHeight + RequiresValidFontAsset, arg2.Type number -> uiFontHeight`

Occurrence input: `changed:FontString.SetFont`, historical status `best-effort` (not conclusion).

Assessment: Current font setters accept unresolved paths and unchecked heights; 12.0.5 fontstring-setfont-shape explicitly excludes asset and height validation, so it cannot cover this 12.0.7 delta.

Current provider: `src/lua_api/frame/methods/text_attribute_event/text/formatting.rs:330`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleFontStringAPIDocumentation.lua:500`; **later cache: may postdate 12.0.7**.

Existing tests: `src/c_api/duration_text_binding.rs:353::duration_binding_availability_preserves_client_versions`; `src/lua_api/workarounds/temporary/formatting_utility_defaults.rs:323::installs_formatting_utility_defaults`; `src/loader/tests/xml_text_region_defaults.rs:109::justify_probe_message_regions_stay_absent_with_owner_insets`; `src/loader/tests/wow_api_globals/startup_globals.rs:84::test_patch_12_0_7_duration_objects_and_text_binding`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: existing font asset resolver + explicit validated height before changing path/height/flags, shared across five receiver kinds without altering vendor Lua. Test known valid asset/nonintegral height, unknown asset, invalid/nonfinite height, unchanged state on failure and exact result tuple per receiver. INFERRED: allowable bounds, nil/flags/error policy and FontAsset domain require historical/native evidence.

### L136 `widgets-MessageFrame-SetFont-136` — MODELABLE

Source: `MessageFrame:SetFont + RequiresValidFontHeight + RequiresValidFontAsset`

Occurrence input: `changed:MessageFrame.SetFont`, historical status `best-effort` (not conclusion).

Assessment: Current font setters accept unresolved paths and unchecked heights; 12.0.5 fontstring-setfont-shape explicitly excludes asset and height validation, so it cannot cover this 12.0.7 delta.

Current provider: `src/lua_api/frame/methods/text_attribute_event/text/formatting.rs:330`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleMessageFrameAPIDocumentation.lua:295`; **later cache: may postdate 12.0.7**.

Existing tests: `src/c_api/duration_text_binding.rs:353::duration_binding_availability_preserves_client_versions`; `src/lua_api/workarounds/temporary/formatting_utility_defaults.rs:323::installs_formatting_utility_defaults`; `src/loader/tests/xml_text_region_defaults.rs:109::justify_probe_message_regions_stay_absent_with_owner_insets`; `src/loader/tests/wow_api_globals/startup_globals.rs:84::test_patch_12_0_7_duration_objects_and_text_binding`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: existing font asset resolver + explicit validated height before changing path/height/flags, shared across five receiver kinds without altering vendor Lua. Test known valid asset/nonintegral height, unknown asset, invalid/nonfinite height, unchanged state on failure and exact result tuple per receiver. INFERRED: allowable bounds, nil/flags/error policy and FontAsset domain require historical/native evidence.

### L137 `widgets-ModelSceneActorBase-GetModelUnitGUID-137` — BLOCKED

Source: `ModelSceneActorBase:GetModelUnitGUID - ret1.ConditionalSecret`

Occurrence input: `changed:ModelSceneActorBase.GetModelUnitGUID`, historical status `exception-requested` (not conclusion).

Assessment: The removed ConditionalSecret annotation applies to a GUID query, not 3D rendering itself; no exact simulator GetModelUnitGUID provider was located. Existing occurrence exception is not proof that this non-rendering query is impossible.

Current provider: No exact receiver/symbol provider located; neighboring subsystem only, no callable credit..

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/FrameAPIModelSceneFrameActorBaseDocumentation.lua:138`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:391::test_patch_12_0_7_widget_compatibility_surface`. These are located source, not fresh execution.

Model / required proof / missing evidence: Missing evidence: historical receiver binding/identity query contract (unbound/nil, unit ownership and GUID access). Once established, a binding token/GUID-only state model may implement this without 3D; test public GUID in classified/unclassified contexts, unbound actor and no rendering dependency. Do not authorize any 3D work.

### L138 `widgets-ScrollFrame-GetHorizontalScroll-138` — MODELABLE

Source: `ScrollFrame:GetHorizontalScroll + SecretReturnsForAspect`

Occurrence input: `changed:ScrollFrame.GetHorizontalScroll`, historical status `best-effort` (not conclusion).

Assessment: Current scroll getter emits plain numeric offset; setter coerces numeric input and commits before callbacks. New secret/aspect paths are not proven by public scrolling tests.

Current provider: `src/lua_api/frame/methods/widgets/slider.rs:469`; `src/lua_api/frame/methods/widgets/slider.rs:500`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleScrollFrameAPIDocumentation.lua:10`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:391::test_patch_12_0_7_widget_compatibility_surface`; `tests/scroll_widgets.rs:404::test_scrollframe_update_scroll_child_rect_uses_resolved_subtree_bounds`; `tests/scroll_widgets.rs:446::test_scrollframe_offsets_round_trip_outside_explicit_ranges_and_notify_after_commit`; `tests/scroll_widgets.rs:514::test_scrollframe_offsets_round_trip_with_zero_cached_ranges`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: existing offsets plus per-frame ScrollOffset aspect; VM-authenticate both setters before commit, mark accepted secret offset and secret-wrap both getters; retain callback order. Test both axes, secret secure/tainted matrix, no-mutation denial, public overwrite/reset, scripts observing post-commit state and frame isolation. INFERRED: aspect clearing lifecycle.

### L139 `widgets-ScrollFrame-GetVerticalScroll-139` — MODELABLE

Source: `ScrollFrame:GetVerticalScroll + SecretReturnsForAspect`

Occurrence input: `changed:ScrollFrame.GetVerticalScroll`, historical status `best-effort` (not conclusion).

Assessment: Current scroll getter emits plain numeric offset; setter coerces numeric input and commits before callbacks. New secret/aspect paths are not proven by public scrolling tests.

Current provider: `src/lua_api/frame/methods/widgets/slider.rs:511`; `src/lua_api/frame/methods/widgets/slider.rs:542`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleScrollFrameAPIDocumentation.lua:51`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:391::test_patch_12_0_7_widget_compatibility_surface`; `src/loader/tests/wow_api_globals/runtime_subsystems.rs:56::test_scrollframe_scroll_scripts_fire_from_native_methods`; `src/loader/tests/wow_api_globals/runtime_subsystems.rs:108::test_scroll_child_resize_refreshes_parent_scroll_range`; `tests/scroll_widgets.rs:404::test_scrollframe_update_scroll_child_rect_uses_resolved_subtree_bounds`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: existing offsets plus per-frame ScrollOffset aspect; VM-authenticate both setters before commit, mark accepted secret offset and secret-wrap both getters; retain callback order. Test both axes, secret secure/tainted matrix, no-mutation denial, public overwrite/reset, scripts observing post-commit state and frame isolation. INFERRED: aspect clearing lifecycle.

### L140 `widgets-ScrollFrame-SetHorizontalScroll-140` — MODELABLE

Source: `ScrollFrame:SetHorizontalScroll SecretArguments NotAllowed -> AllowedWhenUntainted + SecretArgumentsAddAspect`

Occurrence input: `changed:ScrollFrame.SetHorizontalScroll`, historical status `best-effort` (not conclusion).

Assessment: Current scroll getter emits plain numeric offset; setter coerces numeric input and commits before callbacks. New secret/aspect paths are not proven by public scrolling tests.

Current provider: `src/lua_api/frame/methods/widgets/slider.rs:480`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleScrollFrameAPIDocumentation.lua:79`; **later cache: may postdate 12.0.7**.

Existing tests: `src/iced_app/render_hit_grid_tests.rs:16::scroll_offsets_move_cached_hit_target_without_moving_chrome_or_logical_rect`; `src/iced_app/render_hit_grid_tests.rs:118::nested_scroll_offsets_accumulate_in_cached_hits_and_clip_outside_outer_viewport`; `src/loader/tests/wow_api_globals/startup_globals.rs:391::test_patch_12_0_7_widget_compatibility_surface`; `tests/scroll_widgets.rs:66::nested_scroll_viewport_outside_parent_emits_no_visible_content`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: existing offsets plus per-frame ScrollOffset aspect; VM-authenticate both setters before commit, mark accepted secret offset and secret-wrap both getters; retain callback order. Test both axes, secret secure/tainted matrix, no-mutation denial, public overwrite/reset, scripts observing post-commit state and frame isolation. INFERRED: aspect clearing lifecycle.

### L141 `widgets-ScrollFrame-SetVerticalScroll-141` — MODELABLE

Source: `ScrollFrame:SetVerticalScroll SecretArguments NotAllowed -> AllowedWhenUntainted + SecretArgumentsAddAspect`

Occurrence input: `changed:ScrollFrame.SetVerticalScroll`, historical status `best-effort` (not conclusion).

Assessment: Current scroll getter emits plain numeric offset; setter coerces numeric input and commits before callbacks. New secret/aspect paths are not proven by public scrolling tests.

Current provider: `src/lua_api/frame/methods/widgets/slider.rs:522`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleScrollFrameAPIDocumentation.lua:103`; **later cache: may postdate 12.0.7**.

Existing tests: `src/iced_app/render_hit_grid_tests.rs:16::scroll_offsets_move_cached_hit_target_without_moving_chrome_or_logical_rect`; `src/iced_app/render_hit_grid_tests.rs:118::nested_scroll_offsets_accumulate_in_cached_hits_and_clip_outside_outer_viewport`; `src/loader/tests/wow_api_globals/startup_globals.rs:391::test_patch_12_0_7_widget_compatibility_surface`; `src/loader/tests/wow_api_globals/runtime_subsystems.rs:56::test_scrollframe_scroll_scripts_fire_from_native_methods`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: existing offsets plus per-frame ScrollOffset aspect; VM-authenticate both setters before commit, mark accepted secret offset and secret-wrap both getters; retain callback order. Test both axes, secret secure/tainted matrix, no-mutation denial, public overwrite/reset, scripts observing post-commit state and frame isolation. INFERRED: aspect clearing lifecycle.

### L142 `widgets-SimpleHTML-SetFont-142` — MODELABLE

Source: `SimpleHTML:SetFont + RequiresValidFontHeight + RequiresValidFontAsset`

Occurrence input: `changed:SimpleHTML.SetFont`, historical status `best-effort` (not conclusion).

Assessment: Current font setters accept unresolved paths and unchecked heights; 12.0.5 fontstring-setfont-shape explicitly excludes asset and height validation, so it cannot cover this 12.0.7 delta.

Current provider: `src/lua_api/frame/methods/text_attribute_event/text/formatting.rs:330`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/SimpleHTMLAPIDocumentation.lua:203`; **later cache: may postdate 12.0.7**.

Existing tests: `src/c_api/duration_text_binding.rs:353::duration_binding_availability_preserves_client_versions`; `src/lua_api/workarounds/temporary/formatting_utility_defaults.rs:323::installs_formatting_utility_defaults`; `src/loader/tests/xml_text_region_defaults.rs:109::justify_probe_message_regions_stay_absent_with_owner_insets`; `src/loader/tests/wow_api_globals/startup_globals.rs:84::test_patch_12_0_7_duration_objects_and_text_binding`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: existing font asset resolver + explicit validated height before changing path/height/flags, shared across five receiver kinds without altering vendor Lua. Test known valid asset/nonintegral height, unknown asset, invalid/nonfinite height, unchanged state on failure and exact result tuple per receiver. INFERRED: allowable bounds, nil/flags/error policy and FontAsset domain require historical/native evidence.

### L144 `source-context-144` — METADATA-ONLY

Source: `Events Added (2):`

Assessment / exact one-sentence metadata note: Section heading and stated inventory count organize the retained excerpt only; no runtime or implementation credit.

Current provider: No exact provider located for this row; no provider is required for editorial metadata..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

### L145 `events-ENCOUNTER_TIMELINE_EVENT_COLOR_CHANGED-145` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `ENCOUNTER_TIMELINE_EVENT_COLOR_CHANGED`

Occurrence input: `added:ENCOUNTER_TIMELINE_EVENT_COLOR_CHANGED`, historical status `best-effort` (not conclusion).

Assessment: Both valid-events gate and current timeline color-change notification producer exist; name acceptance alone is insufficient.

Current provider: `src/c_api/c_encounter_timeline/notifications.rs:32`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/EncounterTimelineDocumentation.lua:481`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`. These are located source, not fresh execution.

Model / required proof / missing evidence: Mutation -> listener -> exact eventID, state visible before dispatch, unchanged-color no-event control, warning/timeline/5-second trigger differences and alpha. Scope strict 12.0.7 separately from later timeline features.

### L146 `events-URL_TEXTURE_REQUEST_RESULT-146` — MODELABLE

Source: `URL_TEXTURE_REQUEST_RESULT`

Occurrence input: `added:URL_TEXTURE_REQUEST_RESULT`, historical status `best-effort` (not conclusion).

Assessment: Event name is registerable, but no corresponding URL-texture request-result producer was established.

Current provider: `src/event/valid_events_c.rs:475`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/TextureUtilsDocumentation.lua:164`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: explicit URL texture request state -> one terminal result event with documented payload; test success/failure/cancelled/no-duplicate paths and state visible in listener. INFERRED: asynchronous completion/error policies; no live network required for deterministic provider tests.

### L148 `source-context-148` — METADATA-ONLY

Source: `Event Changes:`

Assessment / exact one-sentence metadata note: Section heading and stated inventory count organize the retained excerpt only; no runtime or implementation credit.

Current provider: No exact provider located for this row; no provider is required for editorial metadata..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

### L149 `events-CHAT_MSG_COMBAT_FACTION_CHANGE-149` — MODELABLE

Source: `CHAT_MSG_COMBAT_FACTION_CHANGE - SecretInChatMessagingLockdown`

Occurrence input: `changed:CHAT_MSG_COMBAT_FACTION_CHANGE`, historical status `best-effort` (not conclusion).

Assessment: Current registration accepts these names; removing SecretInChatMessagingLockdown requires auditing payload secrecy at dispatch, not merely valid_events membership.

Current provider: `src/event/valid_events_a.rs:332`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1350`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: real existing chat producer/dispatch path with event-specific public payload policy for these nine names, not blanket declassification. Test active/inactive lockdown, actual incoming-secret payload fixture, addon caller taint retained, exact tuple and an unchanged restricted event control. INFERRED: producer input classification absent native samples.

### L150 `events-CHAT_MSG_COMBAT_HONOR_GAIN-150` — MODELABLE

Source: `CHAT_MSG_COMBAT_HONOR_GAIN - SecretInChatMessagingLockdown`

Occurrence input: `changed:CHAT_MSG_COMBAT_HONOR_GAIN`, historical status `best-effort` (not conclusion).

Assessment: Current registration accepts these names; removing SecretInChatMessagingLockdown requires auditing payload secrecy at dispatch, not merely valid_events membership.

Current provider: `src/event/valid_events_a.rs:333`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1377`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: real existing chat producer/dispatch path with event-specific public payload policy for these nine names, not blanket declassification. Test active/inactive lockdown, actual incoming-secret payload fixture, addon caller taint retained, exact tuple and an unchanged restricted event control. INFERRED: producer input classification absent native samples.

### L151 `events-CHAT_MSG_COMBAT_MISC_INFO-151` — MODELABLE

Source: `CHAT_MSG_COMBAT_MISC_INFO - SecretInChatMessagingLockdown`

Occurrence input: `changed:CHAT_MSG_COMBAT_MISC_INFO`, historical status `best-effort` (not conclusion).

Assessment: Current registration accepts these names; removing SecretInChatMessagingLockdown requires auditing payload secrecy at dispatch, not merely valid_events membership.

Current provider: `src/event/valid_events_a.rs:334`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1404`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: real existing chat producer/dispatch path with event-specific public payload policy for these nine names, not blanket declassification. Test active/inactive lockdown, actual incoming-secret payload fixture, addon caller taint retained, exact tuple and an unchanged restricted event control. INFERRED: producer input classification absent native samples.

### L152 `events-CHAT_MSG_COMBAT_XP_GAIN-152` — MODELABLE

Source: `CHAT_MSG_COMBAT_XP_GAIN - SecretInChatMessagingLockdown`

Occurrence input: `changed:CHAT_MSG_COMBAT_XP_GAIN`, historical status `best-effort` (not conclusion).

Assessment: Current registration accepts these names; removing SecretInChatMessagingLockdown requires auditing payload secrecy at dispatch, not merely valid_events membership.

Current provider: `src/event/valid_events_a.rs:335`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1431`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: real existing chat producer/dispatch path with event-specific public payload policy for these nine names, not blanket declassification. Test active/inactive lockdown, actual incoming-secret payload fixture, addon caller taint retained, exact tuple and an unchanged restricted event control. INFERRED: producer input classification absent native samples.

### L153 `events-CHAT_MSG_CURRENCY-153` — MODELABLE

Source: `CHAT_MSG_CURRENCY - SecretInChatMessagingLockdown`

Occurrence input: `changed:CHAT_MSG_CURRENCY`, historical status `best-effort` (not conclusion).

Assessment: Current registration accepts these names; removing SecretInChatMessagingLockdown requires auditing payload secrecy at dispatch, not merely valid_events membership.

Current provider: `src/event/valid_events_a.rs:337`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1486`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: real existing chat producer/dispatch path with event-specific public payload policy for these nine names, not blanket declassification. Test active/inactive lockdown, actual incoming-secret payload fixture, addon caller taint retained, exact tuple and an unchanged restricted event control. INFERRED: producer input classification absent native samples.

### L154 `events-CHAT_MSG_FILTERED-154` — MODELABLE

Source: `CHAT_MSG_FILTERED - SecretInChatMessagingLockdown`

Occurrence input: `changed:CHAT_MSG_FILTERED`, historical status `best-effort` (not conclusion).

Assessment: Current registration accepts these names; removing SecretInChatMessagingLockdown requires auditing payload secrecy at dispatch, not merely valid_events membership.

Current provider: `src/event/valid_events_a.rs:340`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1569`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: real existing chat producer/dispatch path with event-specific public payload policy for these nine names, not blanket declassification. Test active/inactive lockdown, actual incoming-secret payload fixture, addon caller taint retained, exact tuple and an unchanged restricted event control. INFERRED: producer input classification absent native samples.

### L155 `events-CHAT_MSG_LOOT-155` — MODELABLE

Source: `CHAT_MSG_LOOT - SecretInChatMessagingLockdown`

Occurrence input: `changed:CHAT_MSG_LOOT`, historical status `best-effort` (not conclusion).

Assessment: Current registration accepts these names; removing SecretInChatMessagingLockdown requires auditing payload secrecy at dispatch, not merely valid_events membership.

Current provider: `src/event/valid_events_a.rs:347`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1789`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: real existing chat producer/dispatch path with event-specific public payload policy for these nine names, not blanket declassification. Test active/inactive lockdown, actual incoming-secret payload fixture, addon caller taint retained, exact tuple and an unchanged restricted event control. INFERRED: producer input classification absent native samples.

### L156 `events-CHAT_MSG_MONEY-156` — MODELABLE

Source: `CHAT_MSG_MONEY - SecretInChatMessagingLockdown`

Occurrence input: `changed:CHAT_MSG_MONEY`, historical status `best-effort` (not conclusion).

Assessment: Current registration accepts these names; removing SecretInChatMessagingLockdown requires auditing payload secrecy at dispatch, not merely valid_events membership.

Current provider: `src/event/valid_events_a.rs:348`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:1816`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: real existing chat producer/dispatch path with event-specific public payload policy for these nine names, not blanket declassification. Test active/inactive lockdown, actual incoming-secret payload fixture, addon caller taint retained, exact tuple and an unchanged restricted event control. INFERRED: producer input classification absent native samples.

### L157 `events-CHAT_MSG_RESTRICTED-157` — MODELABLE

Source: `CHAT_MSG_RESTRICTED - SecretInChatMessagingLockdown`

Occurrence input: `changed:CHAT_MSG_RESTRICTED`, historical status `best-effort` (not conclusion).

Assessment: Current registration accepts these names; removing SecretInChatMessagingLockdown requires auditing payload secrecy at dispatch, not merely valid_events membership.

Current provider: `src/event/valid_events_a.rs:367`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ChatInfoDocumentation.lua:2344`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest model: real existing chat producer/dispatch path with event-specific public payload policy for these nine names, not blanket declassification. Test active/inactive lockdown, actual incoming-secret payload fixture, addon caller taint retained, exact tuple and an unchanged restricted event control. INFERRED: producer input classification absent native samples.

### L158 `events-CLUB_MEMBER_ADDED-158` — BLOCKED

Source: `CLUB_MEMBER_ADDED [2].Type number -> ClubMemberOpaqueId`

Occurrence input: `changed:CLUB_MEMBER_ADDED`, historical status `best-effort` (not conclusion).

Assessment: Second event argument changes number -> ClubMemberOpaqueId; generated declaration provides the alias name but no authenticated historical representation/domain, and registration alone supplies no producer. This is not safely metadata-only.

Current provider: `src/event/valid_events_a.rs:421`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ClubDocumentation.lua:1291`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`. These are located source, not fresh execution.

Model / required proof / missing evidence: Missing evidence: 12.0.7 opaque-member-ID representation, identity equality/lifetime and concrete membership-event payloads. If proven numeric alias only, use metadata-only note; otherwise model explicit club/member opaque identity and drive real mutations -> event tuple.

### L159 `events-CLUB_MEMBER_PRESENCE_UPDATED-159` — BLOCKED

Source: `CLUB_MEMBER_PRESENCE_UPDATED [2].Type number -> ClubMemberOpaqueId`

Occurrence input: `changed:CLUB_MEMBER_PRESENCE_UPDATED`, historical status `best-effort` (not conclusion).

Assessment: Second event argument changes number -> ClubMemberOpaqueId; generated declaration provides the alias name but no authenticated historical representation/domain, and registration alone supplies no producer. This is not safely metadata-only.

Current provider: `src/event/valid_events_a.rs:422`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ClubDocumentation.lua:1302`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`. These are located source, not fresh execution.

Model / required proof / missing evidence: Missing evidence: 12.0.7 opaque-member-ID representation, identity equality/lifetime and concrete membership-event payloads. If proven numeric alias only, use metadata-only note; otherwise model explicit club/member opaque identity and drive real mutations -> event tuple.

### L160 `events-CLUB_MEMBER_REMOVED-160` — BLOCKED

Source: `CLUB_MEMBER_REMOVED [2].Type number -> ClubMemberOpaqueId`

Occurrence input: `changed:CLUB_MEMBER_REMOVED`, historical status `best-effort` (not conclusion).

Assessment: Second event argument changes number -> ClubMemberOpaqueId; generated declaration provides the alias name but no authenticated historical representation/domain, and registration alone supplies no producer. This is not safely metadata-only.

Current provider: `src/event/valid_events_a.rs:423`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ClubDocumentation.lua:1314`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`. These are located source, not fresh execution.

Model / required proof / missing evidence: Missing evidence: 12.0.7 opaque-member-ID representation, identity equality/lifetime and concrete membership-event payloads. If proven numeric alias only, use metadata-only note; otherwise model explicit club/member opaque identity and drive real mutations -> event tuple.

### L161 `events-CLUB_MEMBER_ROLE_UPDATED-161` — BLOCKED

Source: `CLUB_MEMBER_ROLE_UPDATED [2].Type number -> ClubMemberOpaqueId`

Occurrence input: `changed:CLUB_MEMBER_ROLE_UPDATED`, historical status `best-effort` (not conclusion).

Assessment: Second event argument changes number -> ClubMemberOpaqueId; generated declaration provides the alias name but no authenticated historical representation/domain, and registration alone supplies no producer. This is not safely metadata-only.

Current provider: `src/event/valid_events_a.rs:424`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ClubDocumentation.lua:1325`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`. These are located source, not fresh execution.

Model / required proof / missing evidence: Missing evidence: 12.0.7 opaque-member-ID representation, identity equality/lifetime and concrete membership-event payloads. If proven numeric alias only, use metadata-only note; otherwise model explicit club/member opaque identity and drive real mutations -> event tuple.

### L162 `events-CLUB_MEMBER_UPDATED-162` — BLOCKED

Source: `CLUB_MEMBER_UPDATED [2].Type number -> ClubMemberOpaqueId`

Occurrence input: `changed:CLUB_MEMBER_UPDATED`, historical status `best-effort` (not conclusion).

Assessment: Second event argument changes number -> ClubMemberOpaqueId; generated declaration provides the alias name but no authenticated historical representation/domain, and registration alone supplies no producer. This is not safely metadata-only.

Current provider: `src/event/valid_events_a.rs:425`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/ClubDocumentation.lua:1337`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`. These are located source, not fresh execution.

Model / required proof / missing evidence: Missing evidence: 12.0.7 opaque-member-ID representation, identity equality/lifetime and concrete membership-event payloads. If proven numeric alias only, use metadata-only note; otherwise model explicit club/member opaque identity and drive real mutations -> event tuple.

### L163 `events-ENCOUNTER_END-163` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `ENCOUNTER_END + encounterUnitStatus`

Occurrence input: `changed:ENCOUNTER_END`, historical status `best-effort` (not conclusion).

Assessment: Current A_Admin.SimulateBossKill copies an explicitly supplied ordered encounterUnitStatus list before dispatch, or emits a fresh empty list when omitted; it does not derive all engaged bosses automatically. Source also requires non-secret fields.

Current provider: `src/lua_api/globals/admin_encounter.rs:53`.

Cached declaration: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/EncounterInfoDocumentation.lua:40`; **later cache: may postdate 12.0.7**.

Existing tests: `src/loader/tests/wow_api_globals/startup_globals.rs:323::test_patch_12_0_7_removal_and_event_surface`; `tests/admin_encounter_api.rs:14::test_boss_kill_fires_encounter_end`; `tests/admin_encounter_api.rs:60::test_boss_kill_copies_ordered_encounter_status_before_dispatch`; `tests/admin_encounter_api.rs:103::test_boss_kill_omitted_and_nil_status_are_fresh_empty_lists`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove multiple engaged bosses, creatureID/name/remainingHealthPercent, nonsecret fields under tainted caller, exact ENCOUNTER_END tuple, detached snapshots and empty/no-boss case. Current tests and declaration must be matched for 12.0.7 rather than accepted by occurrence status.

### L165 `source-context-165` — METADATA-ONLY

Source: `CVars Added examples from crawled page:`

Assessment / exact one-sentence metadata note: Section heading and stated inventory count organize the retained excerpt only; no runtime or implementation credit.

Current provider: No exact provider located for this row; no provider is required for editorial metadata..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

### L166 `cvars-assistedCombatReduceHighlights-166` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `assistedCombatReduceHighlights`

Occurrence input: `added:assistedCombatReduceHighlights`, historical status `implemented` (not conclusion).

Assessment: The six named additions exist in explicit CVar defaults; declaration generated Lua does not normally enumerate CVars. Defaults are simulator choices, not native historical evidence.

Current provider: `src/cvars.rs:357`; `src/cvars.rs:595`.

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `src/cvars.rs:585::patch_12_0_7_cvar_defaults_match_retail`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove all six through public GetCVar/GetCVarBool, non-default SetCVar round trips, reset and independent environments, plus 12.0.7 publication gate. INFERRED: default values and persistence/flags; missing 14 additions/5 removals remain separate blocked statement.

### L167 `cvars-developerLogFilterDebug-167` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `developerLogFilterDebug`

Occurrence input: `added:developerLogFilterDebug`, historical status `implemented` (not conclusion).

Assessment: The six named additions exist in explicit CVar defaults; declaration generated Lua does not normally enumerate CVars. Defaults are simulator choices, not native historical evidence.

Current provider: `src/cvars.rs:359`; `src/cvars.rs:596`.

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: `src/cvars.rs:585::patch_12_0_7_cvar_defaults_match_retail`. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove all six through public GetCVar/GetCVarBool, non-default SetCVar round trips, reset and independent environments, plus 12.0.7 publication gate. INFERRED: default values and persistence/flags; missing 14 additions/5 removals remain separate blocked statement.

### L168 `cvars-developerLogFilterError-168` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `developerLogFilterError`

Occurrence input: `added:developerLogFilterError`, historical status `implemented` (not conclusion).

Assessment: The six named additions exist in explicit CVar defaults; declaration generated Lua does not normally enumerate CVars. Defaults are simulator choices, not native historical evidence.

Current provider: `src/cvars.rs:360`.

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: src/cvars.rs::tests; tests/set_cvar_global.rs. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove all six through public GetCVar/GetCVarBool, non-default SetCVar round trips, reset and independent environments, plus 12.0.7 publication gate. INFERRED: default values and persistence/flags; missing 14 additions/5 removals remain separate blocked statement.

### L169 `cvars-developerLogFilterFatal-169` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `developerLogFilterFatal`

Occurrence input: `added:developerLogFilterFatal`, historical status `implemented` (not conclusion).

Assessment: The six named additions exist in explicit CVar defaults; declaration generated Lua does not normally enumerate CVars. Defaults are simulator choices, not native historical evidence.

Current provider: `src/cvars.rs:361`.

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: src/cvars.rs::tests; tests/set_cvar_global.rs. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove all six through public GetCVar/GetCVarBool, non-default SetCVar round trips, reset and independent environments, plus 12.0.7 publication gate. INFERRED: default values and persistence/flags; missing 14 additions/5 removals remain separate blocked statement.

### L170 `cvars-developerLogFilterNormal-170` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `developerLogFilterNormal`

Occurrence input: `added:developerLogFilterNormal`, historical status `implemented` (not conclusion).

Assessment: The six named additions exist in explicit CVar defaults; declaration generated Lua does not normally enumerate CVars. Defaults are simulator choices, not native historical evidence.

Current provider: `src/cvars.rs:362`.

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: src/cvars.rs::tests; tests/set_cvar_global.rs. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove all six through public GetCVar/GetCVarBool, non-default SetCVar round trips, reset and independent environments, plus 12.0.7 publication gate. INFERRED: default values and persistence/flags; missing 14 additions/5 removals remain separate blocked statement.

### L171 `cvars-developerLogFilterSpam-171` — ALREADY-IMPLEMENTED-NEEDS-PROOF

Source: `developerLogFilterSpam`

Occurrence input: `added:developerLogFilterSpam`, historical status `implemented` (not conclusion).

Assessment: The six named additions exist in explicit CVar defaults; declaration generated Lua does not normally enumerate CVars. Defaults are simulator choices, not native historical evidence.

Current provider: `src/cvars.rs:363`.

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: src/cvars.rs::tests; tests/set_cvar_global.rs. These are located source, not fresh execution.

Model / required proof / missing evidence: Prove all six through public GetCVar/GetCVarBool, non-default SetCVar round trips, reset and independent environments, plus 12.0.7 publication gate. INFERRED: default values and persistence/flags; missing 14 additions/5 removals remain separate blocked statement.

### L172 `source-context-172` — BLOCKED

Source: `(20 added, 5 removed; crawler excerpt did not include full list.)`

Assessment: Retained excerpt explicitly omits 14 of 20 CVar addition names and all five removal names; existing 131-row register cannot identify them.

Current provider: No exact receiver/symbol provider located; neighboring subsystem only, no callable credit..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: None. These are located source, not fresh execution.

Model / required proof / missing evidence: Missing evidence: full 12.0.7 page/extract or authenticated endpoint CVar diff with all 19 missing symbols. Keep aggregate claim as one blocked row; do not invent names or count 19 fabricated rows.

### L174 `deprecated-api-174` — METADATA-ONLY

Source: `Deprecated API added:`

Assessment / exact one-sentence metadata note: Section heading and stated inventory count organize the retained excerpt only; no runtime or implementation credit.

Current provider: No exact provider located for this row; no provider is required for editorial metadata..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

### L175 `deprecated-api-175` — MODELABLE

Source: `12.0.7 Deprecated_12_0_7.lua: C_ClickBindings.MakeModifiers -> MakeModifiers; C_ClickBindings.GetStringFromModifiers -> GetStringFromModifiers; C_Spell.GetMawPowerBorderAtlasBySpellID -> C_Spell.GetMawPowerRarityInfoBySpellID; GetMerchantCurrencies -> C_MerchantFrame.GetMerchantCurrencies.`

Assessment: Source removes native entry point but separately records deprecated wrappers. Simulator still registers several legacy entries; absence cannot be inferred from current stub/method inventories.

Current provider: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_Deprecated/Mainline/Deprecated_12_0_7.lua:8`; `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_Deprecated/Mainline/Deprecated_12_0_7.lua:12`.

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: src/loader/tests/wow_api_globals/startup_globals.rs; tests/spell_maw_powers.rs; tests/click_binding_spell_identifier.rs. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest change: profile-aware native publication boundary plus existing vendor deprecated wrappers controlled by loadDeprecationFallbacks, not unconditional deletion of compatibility aliases. Test old-profile positive control, strict 12.0.7 init native absence, loaded wrapper on/off, dispatch/return tuple and no loader resurrection. INFERRED: exact removal boundary until historical native-versus-wrapper evidence. Existing 12.0.5 spell-maw-powers capability proves the OLD function, not its removal.

### L176 `deprecated-api-176` — MODELABLE

Source: `Deprecated_BattleNet.lua: BNInviteFriend -> C_BattleNet.InviteFriend.`

Assessment: Source removes native entry point but separately records deprecated wrappers. Simulator still registers several legacy entries; absence cannot be inferred from current stub/method inventories.

Current provider: No exact receiver/symbol provider located; neighboring subsystem only, no callable credit..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: src/loader/tests/wow_api_globals/startup_globals.rs; tests/spell_maw_powers.rs; tests/click_binding_spell_identifier.rs. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest change: profile-aware native publication boundary plus existing vendor deprecated wrappers controlled by loadDeprecationFallbacks, not unconditional deletion of compatibility aliases. Test old-profile positive control, strict 12.0.7 init native absence, loaded wrapper on/off, dispatch/return tuple and no loader resurrection. INFERRED: exact removal boundary until historical native-versus-wrapper evidence. Existing 12.0.5 spell-maw-powers capability proves the OLD function, not its removal.

### L177 `deprecated-api-177` — MODELABLE

Source: `Deprecated_PartyInfo.lua: ConfirmReadyCheck/DemoteAssistant/DoReadyCheck/PromoteToAssistant/PromoteToLeader/SetEveryoneIsAssistant/UninviteUnit/IsGUIDInGroup -> C_PartyInfo namespace.`

Assessment: Source removes native entry point but separately records deprecated wrappers. Simulator still registers several legacy entries; absence cannot be inferred from current stub/method inventories.

Current provider: No exact receiver/symbol provider located; neighboring subsystem only, no callable credit..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: src/loader/tests/wow_api_globals/startup_globals.rs; tests/spell_maw_powers.rs; tests/click_binding_spell_identifier.rs. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest change: profile-aware native publication boundary plus existing vendor deprecated wrappers controlled by loadDeprecationFallbacks, not unconditional deletion of compatibility aliases. Test old-profile positive control, strict 12.0.7 init native absence, loaded wrapper on/off, dispatch/return tuple and no loader resurrection. INFERRED: exact removal boundary until historical native-versus-wrapper evidence. Existing 12.0.5 spell-maw-powers capability proves the OLD function, not its removal.

### L178 `deprecated-api-178` — MODELABLE

Source: `12.0.5 Deprecated_AutoComplete.lua wrappers: GetAutoCompletePresenceID/GetAutoCompleteResults/GetAutoCompleteRealms/IsRecognizedName -> C_AutoComplete namespace.`

Assessment: Source removes native entry point but separately records deprecated wrappers. Simulator still registers several legacy entries; absence cannot be inferred from current stub/method inventories.

Current provider: No exact receiver/symbol provider located; neighboring subsystem only, no callable credit..

Cached declaration: No exact generated declaration located for this row (editorial/CVars/deprecated helpers are not normally generated declarations; obsolete types/methods require historical evidence).

Existing tests: src/loader/tests/wow_api_globals/startup_globals.rs; tests/spell_maw_powers.rs; tests/click_binding_spell_identifier.rs. These are located source, not fresh execution.

Model / required proof / missing evidence: Smallest change: profile-aware native publication boundary plus existing vendor deprecated wrappers controlled by loadDeprecationFallbacks, not unconditional deletion of compatibility aliases. Test old-profile positive control, strict 12.0.7 init native absence, loaded wrapper on/off, dispatch/return tuple and no loader resurrection. INFERRED: exact removal boundary until historical native-versus-wrapper evidence. Existing 12.0.5 spell-maw-powers capability proves the OLD function, not its removal.

