# B99 phase 2 — mechanical integration

Date: 2026-10-03.
Repository: `/home/osso-test/Projects/wow/wow-ui-sim`.
`SCRATCH`: `/tmp/claude-1000/-home-osso-test-Projects-wow-wow-ui-sim/f86e1850-973d-4071-a472-f5757f1d7fdc/scratchpad`.

## Result and proof boundary

Applied **26 producer operations + 7 deferred navigation-test operations**, exactly as supplied. Removed four declaration/state/default blocks to park slices 12 and 3. Moved five files into the authorized scratch directories. All replacements for each file were matched against its original current contents, checked for unique matches and non-overlap, then submitted together. No moved anchors required adaptation; no staged whole-file snapshot was copied over repository files.

`cargo fmt` exited **0**, with empty stdout/stderr. Formatter changed only `src/c_api/c_housing/exterior/runtime.rs`, an authorized edited file. No build/check/test, git command, agent, model CLI, vendor/cache mutation, deployment, or acceptance/accounting update was performed. GREEN remains unverified.

The preserved `tests/character_stats.rs` declaration remains:

```rust
#[cfg(all(feature = "retail-12-0-5", feature = "client-retail"))]
#[path = "character_stats/missing_apis.rs"]
mod missing_apis;
```

## Operations by slice

### [6] Partial prose 104 / 114 — applied 6.1–6.4

- **6.1**, `src/c_api/cooldown_duration.rs`: added `read_untainted_ignore_gcd`; authenticate actual secret flags through `unwrap_secret` under retail-12-0-5. Existing spell-specific reader remains unchanged.
- **6.2–6.3**, `src/c_api/c_action_bar.rs`: import/use authenticated reader for argument 2, propagating its error.
- **6.4**, `src/c_api/c_spell_book.rs`: authenticate argument 3 before spellbook lookup or missing-entry nil return.

No existing test expectation changed. GUID comparison and duration models were not replaced.

### [7] PDEID / special-bar membership — applied 7.5–7.6

- **7.5**, `src/lua_api/globals/missing_surface/delves_ui.rs`: replace constant 77011 with live `tiered_entrance_pde_id`, including zero.
- **7.6**, `src/c_api/c_action_bar_spell_slots.rs`: use existing public identifier reader; return effective direct membership OR explicit `special_bar_spells` membership.

No existing test expectation changed. Direct-only slot queries and existing public-only secret-input boundary remain unchanged.

### [5] Housing bundles — applied 5.4, 5.6–5.9

- **5.4**, `src/lua_api/globals/missing_surface.rs`: register Rust housing-bundle surface unconditionally.
- **5.6–5.9**, `src/lua_api/workarounds/temporary/housing_catalog_state.lua`: delete obsolete Lua bundle-copy helper, `GetBundleInfo`, `GetFeaturedBundles`, and `HousingMarketActionViewBundle` publishers.

Phase-1 seed deletion remains intact. Rust record map supplies getters and view mutation; legacy `entryIDs`/`wasViewed` extensions remain in the authored Rust module. No existing storefront assertion changed.

### [9] Core fixture / restricted outfit — applied 9.4–9.8, 9.12

- **9.4–9.5**, `src/c_api/c_housing/exterior/runtime.rs`: register/implement `SelectCoreFixtureOption` through existing argument authentication, authored core transaction, and existing committed-state response/storage publication.
- **9.6–9.8**, same file: pass API name into shared argument/target readers so core and existing exterior mutations share validation without manufacturing an `ExteriorChange` variant.
- **9.12**, `src/lua_api/workarounds/temporary/housing_catalog_state.lua`: delete obsolete core-selection no-op.

No restricted-outfit producer was invented: all four restricted-helper tests already passed in main's RED run. No existing test expectation changed. Phase-1 `core_fixture: None` fixture addition remains.

### [2] Follower display — applied 2.5

`src/lua_api/globals/group_queries.rs`: route retail-12-0-5 `UnitTreatAsPlayerForDisplay` to authored GUID-backed, authenticated resolver. Keep existing implementation under earlier-epoch cfg. Retail no longer blindly treats pet/vehicle tokens as players for display.

No identity/secrecy subsystem, nameplate renderer, CVar consumer, or row-172 secure delegate changed. No existing test expectation changed.

### [1] Private aura sound Add — applied 1.4, 1.5, 1.8

- **1.4**, `src/c_api/private_aura_sounds.rs`: register legacy Add and cfg-gated modern `AddAuraSound`.
- **1.5**, same file: removal deletes both live ID and owned payload record.
- **1.8**, same file: update module description to registration/removal model, retaining no-native-playback claim.

No existing removal assertion changed. Phase-1 `..Default::default()` fixture construction remains.

### [4] Navigation / aura-entry rekey — applied 4.5–4.16

- **4.5**, `src/c_api/mod.rs`: register retail-12-0-5 navigation provider.
- **4.6**, `src/lua_api/env_events.rs`: apply entry transition before event consumers.
- **4.7**, `src/lua_api/globals/state_backed_queries.rs`: apply transition before callback dispatch.
- **4.8**, `src/lua_api/loader_env.rs`: apply transition before collecting/delivering listeners.
- **4.9**, `src/lua_api/workarounds/temporary/navigation_defaults.rs`: remove nearest-token nil placeholder.
- **4.10–4.12**, same file: apply authored unit-test maintenance.
- **4.13–4.16**, `tests/c_navigation_probes.rs`: apply authored integration-test maintenance.

No provider-preservation assertion changed. No extra entry notification, fallback, token inference, or private-sound rekey was added.

### [3] Aura headers / ClassTalentHelper — parked

Skipped **all 3.2–3.7 producer operations** and **all 3.8–3.12 fixture operations**.

`tests/hero_talents.rs::test_class_talents_switch_methods_update_seeded_spec_and_loadout_state` uses bare `WowLuaEnv::new()` and asserts immediate loadout/spec mutation. Authored producer only dispatches commands to vendor callbacks: without loaded UI it cannot supply those mutations. Handoff explicitly excludes cold lazy loading and completed specialization cast, and says staged tests prove initiation/neutrality, not completion. No honest completed-lifecycle replacement is supplied. Preserved the existing test unchanged; did not weaken, delete, cfg-disable, or add direct-state fallback.

Main's recorded RED shows all five new slice-3 tests failing without producers, so retaining that test file would retain known failures. Moved:

- `tests/secure_aura_header_helpers.rs` → `$SCRATCH/parked-aura-header/secure_aura_header_helpers.rs`.
- `src/c_api/class_talent_commands.rs` → `$SCRATCH/parked-aura-header/class_talent_commands.rs`.

Removed its cfg + module declaration from `src/c_api/mod.rs`. Slice had no new spec or state fields to remove. Five existing fixture edits remain unapplied because their producer is parked.

**Aura-ordering RED reading:** `b99-red.log` records an empty order at line 136 and `left: "" / right: "313,312,311"` at line 163. Both real cached Lua/XML files exist at the test's exact `Blizzard_RestrictedAddOnEnvironment` path. Cached XML declares the hidden header template and its OnShow handler; cached Lua calls Update on Show, gates event updates/registration on visibility, and maps zero expiration to `math.huge` before sorting. Thus evidence points to header fixture/script/visibility/update initialization producing no visible children, not a demonstrated permanent-aura comparator failure or missing cache path. Fixture also clears setup Lua errors before assertions. Exact runtime cause is unproven; no fixture/vendor change was made to mask it. Parked files retain original assertions for later diagnosis.

### [8] Chat expressions — excluded

Applied **none** of 8.1, 8.2, 8.5 or deferred startup-namespace edits 8.6–8.8. Did not restore module, state, or moved files. Unpublished rilua helper remains outside this round.

### [12] Group combat restrictions — parked

No producer exists in supplied packet. Moved:

- `tests/group_combat_restrictions.rs` → `$SCRATCH/parked-combat-restrictions/group_combat_restrictions.rs`.
- `src/c_api/c_party_info/combat_restrictions.rs` → `$SCRATCH/parked-combat-restrictions/combat_restrictions.rs`.
- `docs/specs/group-combat-restrictions.md` → `$SCRATCH/parked-combat-restrictions/group-combat-restrictions.md`.

Removed cfg + `pub mod combat_restrictions` from `src/c_api/c_party_info.rs`; removed documented `group_restrictions` field/cfg from `src/lua_api/state/sim_state.rs` and its cfg/default from `src/lua_api/state.rs`. Skipped both deferred startup-global expectation edits. No Lua adapters or denial contracts fabricated.

## Every existing test whose expectation changed

| File / exact test | Old meaning | New meaning |
|---|---|---|
| `tests/c_navigation_probes.rs::navigation_fallbacks_return_safe_empty_defaults` | Six default results, including unconditional nearest-token nil | Five unchanged temporary defaults; nearest-token contract owned by authenticated host-backed navigation tests. Removed only its tuple component, query, and nil assertion (4.13–4.16). |
| `src/lua_api/workarounds/temporary/navigation_defaults.rs::tests::installs_safe_empty_navigation_defaults` | Six-result tuple also requires placeholder nearest-token nil | Five-result tuple covers remaining temporary defaults only (4.10–4.12). |

**No other existing test expectation changed.** The five class-talent fixture edits, chat startup expectations, and combat startup expectations were skipped. Existing `hero_talents` immediate-mutation control remains unchanged.

**Older-epoch navigation inspection:** `tests/blizzard_quest_navigation_loads.rs::blizzard_quest_navigation_consumes_c_super_track_namespace` only checks nearest-token function presence, not nil output. Default features select `client-retail`, which includes cumulative retail-12-0-5 support; newly registered Rust provider preserves that presence. Left test unchanged. Earlier profiles without retail-12-0-5 lose the removed placeholder, as handoff explicitly states; this cross-profile gap remains reported, not hidden with a fallback or unrelated test change.

## GREEN filters for main

Use existing local integration runner, one filter per invocation:

```text
python3 scripts/build-host.py --build-host local --test --test integration FILTER -- --nocapture
```

| Slice | FILTER | Cases from phase-1 RED |
|---|---|---|
| 6 | `retail_12_0_5_partial_104_114::` | 5 |
| 7 | `pdeid_specialbar::` | 8 |
| 5 | `housing_bundle_structures::` | 6 |
| 9 | `house_exterior_core_fixture::` | 10 |
| 9 control | `restricted_outfit_index::` | 4 |
| 2 | `follower_nameplate_display::` | 7 |
| 1 | `private_aura_sound_add_context::` | 8 with current retail-12-1-0 |
| 4 | `patch_12_0_5_navigation_aura_entry::` | 8 |

Retained new scope: **56 cases**; no post-edit execution here. Do not include parked `secure_aura_header_helpers::` or `group_combat_restrictions::`, or removed chat expressions. For exact historical retail-12-0-5 proof, main can select `--no-default-features --features profile-retail,retail-12-0-5`; GUI-gated controls need their actual GUI-enabled feature configuration rather than empty-filter passes.

## Existing controls most likely affected

| Area | Filters / controls |
|---|---|
| Housing | `housing_catalog::` (especially `housing_catalog_storefront_and_market_methods_use_seeded_state`), `house_exterior::`; shared serializers, bundle action, validation, storage and callback reentry. |
| Sound | `private_aura_sound_removal::`; host live IDs, owned records, deprecated alias and removal lifecycle. |
| Navigation | `c_navigation_probes::`, `blizzard_quest_navigation_loads::`; library unit controls `navigation_defaults::tests::`, including `preserves_existing_navigation_provider`. |
| Bars / Delves | `action_bar_membership::`, `delves_ui::`, `delves_api_inputs::`; direct/macro/outfit membership and PDEID consumers. |
| Talents, identity, duration, display | `hero_talents::` including its unchanged immediate-mutation control and rendering submodule; `admin_spec_talent_api::`; `unit_comparison_permissions::`, `unit_api::`, `cooldown_probes::` including ignore-GCD/duration cases, `spell_book_cooldown_outputs::`, `cooldown_widget::`, `duration_core::`. Existing group/unit query consumers, party aliases, target/focus display and startup surface checks should also be covered; no dedicated existing UnitTreatAsPlayerForDisplay test was found outside the new follower suite in the inspected tests/global sources. |

Entry dispatch now intentionally requires an exact fresh host batch for nonempty aura stores at encounter/M+/PvP entry. Existing entry-event tests/consumers must stage that host input; do not mistake this explicit contract for a transient failure. Loaded cooldown-viewer refresh, native randomness, full nameplate/CVar consumer behavior and talent completion are not established by this mechanical work.

## Proof ledger

| Evidence | Scope | Result |
|---|---|---|
| Unique-anchor and overlap preflight | 33 selected deferred operations plus four removals, grouped by current file | Every OLD unique; no overlap; no adapted anchors. |
| Exact reconstruction | Repository immediately after edits, before formatter | All 33 operations and four removals match expected source exactly. |
| Skipped-anchor inspection | Slice-3 producer + five fixture edits | All 11 OLD anchors still present; edits not applied. |
| Existing RED log + cached source inspection | Parked aura-header fixtures | Empty child lists observed by main; cache paths exist; fixture/update boundary suspected, not runtime-proven. |
| `cargo fmt` | Final integrated Rust tree | Exit 0; only authorized exterior runtime formatting changed; syntax/formatting only. |
| Compilation, GREEN, lint/check, startup/runtime acceptance | Not authorized | NOT RUN. |

Scratch evidence: `b99-phase2-actions.json`, `b99-phase2-preimages.json`, `b99-phase2-prefmt.json`, `b99-phase2-fmt-result.txt`. These contain actual current-tree inputs, not staged replacement snapshots.
