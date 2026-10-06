# Patch 12.1.0 page audit

Row-by-row audit of the [Patch 12.1.0/API changes](https://warcraft.wiki.gg/wiki/Patch_12.1.0/API_changes) page. Ledger and SSOT for row status: [12.1.0-page-coverage.json](../../../data/patch-api/sources/12.1.0-page-coverage.json). Earlier occurrence-level work on 12.1 is in [patch-12-1-api-audit](patch-12-1-api-audit.md); its statuses are cross-references, not proof for this ledger. Method follows the [12.0.7 audit](patch-12-0-7-api-audit.md).

## Source

- Plaintext extract (333 non-blank lines): notes, blue posts, enumerations, structures, deprecated API.
- The extract drops six collapsed tables. They come from the raw wikitext (revision 6886719) via [12.1.0-wikitext-register.json](../../../data/patch-api/sources/12.1.0-wikitext-register.json): 778 symbols across Global API, FrameXML, ScriptObjects, Widgets, Events and CVars.
- 1,111 rows in total. Triage and batch plans: [evidence directory](../../../data/patch-api/evidence/12.1.0-session-2026-10-04/).

## Round 1 — proof batches C01–C10 — 2026-10-04

**1,022 pending / 24 bounded / 1 partial / 64 metadata.**

Tests and specs only, no producer change: AuraContainer/AuraButton creation, FrameXML helper moves, TOC `[Bootstrap]`, OnUpdate modes, XML mixins, event-registration aspect, inheritance, aura groups and options, aura sound removal.

- Master: 8 new tests passed / 0 failed, startup `lua-errors` `[]`, `cargo fmt --check` exit 0 ([log](../../../data/patch-api/evidence/12.1.0-session-2026-10-04/round-1-master-green.log.txt)). Seventeen reused existing tests ran only in the integration worktree ([result](../../../data/patch-api/evidence/12.1.0-session-2026-10-04/p1210-r1-result.md)).
- [Review](../../../data/patch-api/evidence/12.1.0-session-2026-10-04/p1210-r1-review.md): ACCEPT WITH QUALIFICATIONS — 24 bounded, 1 partial, 2 pending; no vacuous tests.
- Pending: disabled-addon bootstrap (not exercised); AuraContainer intrinsic event-registration restriction (observed mask 0 — a real gap, modelable).
- The OnUpdate rows rest on four passing tests; `on_update_modes_process_actual_managed_aura_dirty_phases` was failing before this audit and still is.

## Round 2 — publication sweep — 2026-10-04

**391 pending / 151 bounded / 505 partial / 64 metadata.**

One data-driven test over the 778 inventory symbols ([spec](../../specs/patch-12-1-0-publication-sweep.md)): 631 match the page, 147 do not.

- 504 added/changed symbols are published: partial, "publication only" — a looser credit rule than the rest of the audit, chosen for breadth.
- 127 removed symbols are absent by raw and ordinary lookup: bounded.
- 147 gaps stay pending and are baselined in `tests/data/patch_12_1_0_sweep_known_gaps.json`, so the test fails on any change to the gap set. Roughly: 85 FrameXML helpers not found (many `*_LoadUI` / `Show…Frame` functions, possibly a load-on-demand bootstrap loading gap), 35 missing Global API functions, 20 removed symbols still published, 15 widget methods, events and CVars.

Master: sweep test passed, startup `lua-errors` `[]`, `cargo fmt --check` exit 0 ([log](../../../data/patch-api/evidence/12.1.0-session-2026-10-04/round-2-master-green.log.txt)). Per-symbol output: [result](../../../data/patch-api/evidence/12.1.0-session-2026-10-04/p1210-sweep-result.json). No independent review.

## Round 3 — closing sweep gaps — 2026-10-04

**265 pending / 157 bounded / 625 partial / 64 metadata.**

Sweep gaps 147 → 21 at `616bf37ab` ([result](../../../data/patch-api/evidence/12.1.0-session-2026-10-04/p1210-r3-sweep-result.json)). 127 rows credited by the round 2 rule; `PlayerChoiceToggle_TryShow-1027` dropped back to pending.

- **FrameXML helpers (75):** root cause was the test preload helper (`tests/common/prefork_full_ui_preload.rs`), which skipped load-on-demand `[Bootstrap]` files that real startup runs. It now uses startup discovery. `IsPlayerAtEffectiveMaxLevel` and `UIParent_ManageFramePositions` are no longer published natively under 12.1.0.
- **Added Global API (31):** implemented over existing state, with behavior tests in `tests/patch_12_1_0_added_globals.rs`. Guessed behaviors are marked `INFERRED` in code. `C_Browser.CloseFullscreenBrowser` is a permanent no-op. The roleset workaround was replaced by a Rust `C_Roleset`.
- **Widgets, events, CVars (18):** `RadialProgress` animation type, `SecondsFormatter:GetRounding`, CVar adds/removals (new CVar defaults of `"0"` are guesses), new events.
- **`C_PvP.JoinRandomTrainingGround`:** now marked removed.

Remaining 21 gaps:
- 8 removed Global API and `RaidNotice_*`, `getglobal`, `setglobal` (14 total): Blizzard's own deprecated files (`Deprecated_12_1_0.lua`, `Blizzard_DeprecatedBattleNet`, `Blizzard_DeprecatedRaidWarning`) republish them because `loadDeprecationFallbacks` defaults to 1. Likely the real client too; the sweep's "absent" expectation may be too strict.
- `PlayerChoiceToggle_TryShow-1027`: the page lists it as both added (868) and removed.
- `EventUtil.AreVariablesLoaded`: defined by the simulator's `shared_bootstrap.lua` and survives Blizzard's `EventUtil = {}`; not traced.
- `MacroFrame_SaveMacro`: `Blizzard_MacroUI` loads at startup.
- `EncounterJournal_OpenToTieredEntrance`: only in the full LoD addon. `ShouldDisplaySpellCooldown`: only a mixin method.
- `Frame:ResizeToBoundsRect`: undocumented behavior. `CHAT_MSG_*`: wildcard, unprobeable.

Not caused by this round, still failing: `c_spell_static_fallbacks` (expects a function retail removed in 12.0.7), `wowforever_cooldown_categories::forever_cooldown_categories_preserve_other_profiles`, `on_update_modes_process_actual_managed_aura_dirty_phases`.

`prefork_full_ui` at `616bf37ab`: 2,009 passed / 7 failed. All 7 also fail at the pre-round base `d1a2a0250`: four wardrobe tests (`itemModifiedAppearanceID requires a number`), two `Deprecated_HousingCatalog` legacy-field wrappers, `catalog_shop` (`product_provider_empty`). No independent review.

## Round 4 — extract rows: enums, structs, deprecated wrappers — 2026-10-04

**149 pending / 225 bounded / 658 partial / 79 metadata** (after roleset/CVar/singles credits and the strict-removals fix).

Plan: [extract scout](../../../data/patch-api/evidence/12.1.0-session-2026-10-04/p1210-extract-scout.md) classified the 244 non-sweep pending rows into 15 batches.

- 15 editorial rows → metadata-only; roleset getters row credited.
- **Enums (58, bounded):** `tests/patch_12_1_0_enums.rs` checks exact values and Meta against cached generated docs. 11 values were wrong because `__wow_fill_enum` assigned max+1 (broke flag enums and mid-table inserts); 12 enums now publish explicit doc pairs from `src/c_api/patch_12_1_0_enums.rs`.
- **Structs (26, partial):** `tests/patch_12_1_0_struct_shapes.rs` checks field presence/type against cached docs. Producer fixes: `hideAnswerArt`, `canAttachPet`, `friendLevel`, `classFilename` from class ID, `overrideTooltipSpellID`. `TieredEntranceTierInfo` uses `queueAsLFG` (cached docs; no Blizzard consumer reads either name).
- **Deprecated wrappers (9, bounded):** `tests/patch_12_1_0_deprecated_wrappers.rs` (prefork_full_ui) proves cached deprecated Lua installs and delegates.
- **Strict removals fixed (`b5b31ee5b`):** `strict_removals.lua` ran after startup and deleted wrappers cached `Blizzard_Deprecated/Mainline/Deprecated_12_1_0.lua` installs. Root cause: generic `C_*` namespace fallbacks (runtime surface and a copy in `housing_catalog_state.lua`) fabricated removed keys, plus three still-registered natives. Removed keys are now marked at namespace setup (`mark_namespace_keys_removed`), natives unregistered, the post-startup deletion deleted. Dye wrappers 390/391 credited; 4 sweep rows (`C_DyeColor.GetDyeColorForItem[Location]`, `C_Housing.IsInsideOwnHouse`, `C_SuperTrack.GetNextWaypointForMap`) now join the Blizzard-republished gaps (sweep 25 gaps).
- **Open:** `GetInventorySlotInfo` comes from `Deprecated_PaperDoll` (same folder name in the exported UI source); the simulator never loads it because the folder lacks the `Blizzard_` prefix. Unknown whether the real client loads it.

Master `10118708c`: sweep, enum, struct, added-globals, surface-closures and deprecated-wrapper tests GREEN, each run alone. Pre-existing failures noted by the enums agent, unchanged by this round: `test_patch_12_0_0_transmog_situation_enum_values`, `edit_mode_profile_option_enums_match_blizzard_docs`, `unit_frame_edit_mode_setting_meta_includes_big_defensive_icon_size`. No independent review.
- **Roleset, CVar scope, singles (9):** roleset filtering drives visibility (two INFERRED semantics); session-only `tooltipShowAuraSpellIDs`; `CreateFrame("WorldFrame")` now rejected (real bug). Blocked: 19-arg chat filter (cached Blizzard Lua passes 14), VectorGraphics method absence (needs per-type dispatch).

Master `b5b31ee5b`: sweep, enum, bootstrap, 45 `patch_12_1_0_` integration and 49 `test_patch_12_1_` lib tests GREEN; `prefork_full_ui` 2,011 passed / 7 failed, the same 7 pre-existing failures as round 3.
- **Sweep rule for deprecation fallbacks (`6a3634401`):** a removed symbol now passes when its value is a Lua function from a cached Blizzard deprecation file — what the client publishes with `loadDeprecationFallbacks=1`. 14 rows credited; gaps 25 → 11. Native aliases (`C_Housing.IsInsideOwnHouse`, `C_SuperTrack.GetNextWaypointForMap`, `RemovePrivateAuraAppliedSound`, `GetInspectSpecialization`) have no Lua source and stay gaps. Ledger 135 pending / 239 bounded / 658 partial / 79 metadata.
- **Aura filters and secret context (11, partial):** `src/c_api/aura_filter.rs` parses filter components for `GetUnitAuraInstanceIDs`/`IsAuraFilteredOutByInstanceID`; `src/c_api/unit_aura_access.rs` makes RequiresUnitAuraAccess APIs error for tainted callers and secret-wraps AuraData and `UNIT_AURA` update info under restriction. Seven INFERRED choices (in code). Not modeled: secret vector from `GetUnitAuras`, scalar secret outputs, legacy `UnitAura`/`UnitBuff`/`UnitDebuff` gating, new filter components in index/slot APIs. `spell_api::test_spell_get_spell_charges` also fails on master (pre-existing).
- **Forbidden aspects (9, partial; `4a303f6d0`):** dispatch now skips tainted handlers on restricted hierarchies (13 dispatch sites use `get_dispatch_script`; HookScript chains checked per handler); aspect method gates reject tainted callers only, because Blizzard's secure `UpdateEventRegistrations` registers events under EventRegistrations. Pending: layout-driven OnSizeChanged (not dispatched at all), aura-secret forbidden state, context-access error text. `wowforever_table_does_not_leak_into_earlier_profiles` fails at `d1a2a0250` too (pre-existing).
- **Managed AuraContainer (48, partial; `ef0843889..8e01da096`):** harness drives the real cached template from addon code with simulator aura state. Simulator fixes found on the way: intrinsic `OnEvent_Intrinsic` never received events (dispatch ran only the normal binding), numeric step curves interpolated linearly (broke duration text), secure env kept pre-created placeholder frames after XML replacement, `CreateSecureDelegate` was an identity stub. Pending: addon `CreateFrame("AuraButton")` rejection, tooltip disable via `SetMouseMotionEnabled` (hit-test ignores motion flag), aura sounds (no aura-change subsystem).
- **Unit identity secrecy (10, partial; `2f36c7ddc`):** exposed a rilua GC bug — `intern_hashed` returned strings marked dead but not yet swept, so the sweep freed live strings (`unit_frame_layer_probe` failed only under GC). Fixed in rilua `intern-resurrect-dead` (`a76ffa8`, pushed to Osso/rilua); wow-ui-sim pin moved from `6044544` (`25d7680ea`).
- Flaky under full parallel lib runs: `cast_completion::duration_tests::unit_cast_duration_clears_before_completion_callbacks` (1 of 3 full runs; passes alone and in module).
- **Struct producers (20, partial; `44c4ec6da..a938b7602`):** state-backed `PlaySoundWithOptions` (records volume), cooldown viewer cooldowns (temporary Lua defaults removed), LFG active entry, `GetPetInfoTableByPetID` with doc field names `tradable`/`unique`, `CHAT_MSG_*` payload 18 `DiscordChatInfo` (Blizzard's chat handler reads `arg18.userID` unconditionally).
- **Aura-secret access (5, partial; `28a40b1b8..501d6f13a`):** tainted frame-method callers on access-restricted objects are denied while auras are secret (`frame_id_from_stack`); query methods stay callable. **EventUtil root cause:** a source patch prepended `if EventUtil ~= nil then return end` to Blizzard's `EventUtil.lua`, so the simulator bootstrap copy (with `AreVariablesLoaded`) won on every profile; removed — sweep gaps 11 → 10.
- **Leftovers (9, partial; `965dad7e4`):** OnSizeChanged now fires from the per-tick layout pass (it never fired on layout-driven resizes before); aura sounds play on add/application/removal; addon `CreateFrame("AuraButton")` errors (intrinsics without `allowUntaintedCreation`); hover requires mouse motion; `Frame:ResizeToBoundsRect` implemented (sweep gaps 10 → 9). Blocked: SVG and radial-mask rendering, private script object partitions, CastingBar taint route, `C_PingSecure.GetTargetPingReceiver`.

## Session integration proof — 2026-10-04

Full integration suite (full-UI preload tests run separately) at the session-start commit `d1a2a0250` vs master `d7928bf05`: 30 → 27 failures, **0 new** ([before](../../../data/patch-api/evidence/12.1.0-session-2026-10-04/integ-failures-d1a2a0250.txt), [after](../../../data/patch-api/evidence/12.1.0-session-2026-10-04/integ-failures-d7928bf05.txt)). An intermediate run caught 21 regressions from the strict-removals fix (18 `GetInventorySlotInfo`, 2 RecruitAFriend, 1 bootstrap boundary); root cause: the client treats every addon listed in `ui-toc-list.txt` as built-in, but the simulator required a `Blizzard_` prefix, so `Deprecated_PaperDoll` (which republishes `GetInventorySlotInfo`) never loaded. Built-in = `Blizzard_` prefix or top-level folder in the profile manifest (`blizzard_ui_sync::is_builtin_addon_folder`). Lib: same 6 pre-existing failures; `prefork_full_ui`: same 7; all three patch sweeps, enum and bootstrap tests GREEN; startup lua-errors `[]`.

Re-verified at `16682b415` (after 12.0.5/12.0.7 gap closures, club model, 12.0.0 sweep, console registry, tooltip GC fix): full integration 26 failures vs 30 at `d1a2a0250`, **0 new** ([list](../../../data/patch-api/evidence/12.1.0-session-2026-10-04/integ-failures-16682b415.txt)); lib and `prefork_full_ui` unchanged pre-existing failures; four patch sweeps GREEN; startup `[]`. The 12.0.0 sweep merge briefly broke `tooltip_item_context::tainted_secrets_all_four_positions…`: bisect pointed at the retirement commit, but the cause was unrooted tooltip tables collected during `CreateColor` callbacks (GC timing shift); fixed by rooting (`16682b415`).

Re-verified at `5168a5282` (after 12.0.0 producers, enum/deprecated batches, 11.x retirement + successor fixes, cast-duration rounding fix): full integration 25 failures vs 30 at `d1a2a0250`, **0 new** ([list](../../../data/patch-api/evidence/12.1.0-session-2026-10-04/integ-failures-5168a5282.txt)); lib 6 pre-existing; `prefork_full_ui` 2016/3 (7 at session start); four sweeps, both enum tests, alias and bootstrap tests GREEN alone; startup `[]`.
