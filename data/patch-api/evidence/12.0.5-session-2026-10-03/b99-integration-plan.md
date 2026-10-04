# B99 single-worker integration plan

Verified: 2026-10-03. Planning only; this planner performed no repository writes, builds/tests, git mutations, agents, model CLIs, or operational actions. Read current files, not just historical HEAD or staged snapshots. Initial read-only HEAD: `9c0d271f440916a7a50a288a5c1653b20905f305`, with the described uncommitted work. Concurrent repository activity advanced HEAD to `aa29d7d7f06b14f33990c36ae736b98579b00b78` during planning. Final revalidation found all 44 planned input hashes and all 104 anchor counts unchanged, and all 34 new destinations still absent. Other documentation/evidence and quest-info-system changes remain outside the edit ranges; no clean-tree claim.

## Decision

**Use bounded anchored edits, never overwrite existing files with staging copies.** All 96 authored existing-file anchors still match current source exactly once. The additional chat startup-test maintenance anchor described in its handoff also matches once. No authored anchor requires correction against this snapshot. Only two cross-slice overlapping replacement ranges exist: the identical rilua pin in Cargo.toml and source URL in Cargo.lock, both shared by slices 10 and 11. Resolve them once, not sequentially.

Textual application is mechanical. Full successful integration has two known prerequisites: a fetchable rilua revision containing BOTH new VM facilities, and an authored replacement for the existing immediate-mutation ClassTalents command test. Do not silently weaken that test, claim it passes, or restore direct-state fallback. Safe fixture-maintenance edits are spelled out below; the missing real completion-lifecycle test is explicitly blocked rather than guessed.

## Slice key and input authority

| ID | Selected handoff | Staging directory |
|---|---|---|
| 1 | handoff-private-aura-sound.md | staging/private-aura-sound |
| 2 | handoff-editmode-nameplate.md — ONLY row 174 API slice | staging/editmode-nameplate |
| 3 | handoff-aura-header.md | staging/aura-header |
| 4 | handoff-nav-rekey.md | staging/nav-rekey |
| 5 | handoff-housing-bundle.md | staging/housing-bundle |
| 6 | handoff-partial-104-114.md | staging/partial-104-114 |
| 7 | handoff-pdeid-specialbar.md | staging/pdeid-specialbar |
| 8 | handoff-chat-expressions.md | staging/chat-expressions |
| 9 | handoff-fixture-restricted-outfit.md | staging/fixture-restricted-outfit |
| 10 | handoff-ambiguate-v2.md | staging/ambiguate — **apply last, after dependency bump** |
| 11 | handoff-taint-log-v2.md | staging/taint-log — **apply last, after dependency bump** |

Paths below are repo-relative; staging paths are relative to SCRATCH. Treat old handoffs, parked equipset/bundle tests, non-v2 ambiguate/taint-log packets, and row 172 secure-delegate work as excluded. No changes to vendor/cache files, build.rs, tests/integration.rs, or new Cargo test targets are required.

## Worker procedure and recommended order

1. Wait for the owning task to commit current working-tree work. This planning task does not commit it. Recheck content hashes and each exact anchor; a commit alone changes no source bytes, so matching hashes remain usable. Stop on a genuinely changed anchor rather than copying a stale full file.
2. Copy only the new-file allowlist below. For existing files, collect all selected replacements against the same current input, require one match, reject overlapping ranges, then apply in descending offset order or as one atomic multi-edit per file. Deduplicate Cargo edits as specified below. State/producer tags remain available for optional authorized RED checkpoints; no test execution occurs here.
3. Apply independent slices in this order: **6 → 7 → 5 → 9 → 8 → 2 → 1**. Complete each state/producer pair. This groups the two housing Lua deletions, keeps direct/special-bar semantics intact, and installs sound state before entry-event integration.
4. Apply **4 → 3**. Install shared aura-entry hooks before adding loaded aura-header tests; keep their no-entry sort fixtures independent. Include the existing-test maintenance for slice 3. Do not claim full acceptance until its command-lifecycle control has been authored and proven.
5. The owning task must supply one published, fetchable rilua SHA containing both helpers. Update dependency pin/lock once; then apply **10 → 11, last, after dependency bump**, skipping both packets' duplicate dependency edits. Taint-log macro wrapping must preserve all prior inserted fields. Formatting, commits and future proof belong to the authorized integrator, not this planner.

## Shared-file reconciliation

### src/c_api/mod.rs — 3, 4, 5

Different exact anchors, no overlap: `pub(crate) mod c_transmog_collection;` (3), `pub mod bag_info;`, `pub mod c_map;`, `c_chat_info::register(state)?;` (4), `pub mod c_catalog_shop_products;` (5). Insert class_talent_commands, aura_entry_ids, c_navigation, aura_entry and c_housing_bundles once. Preserve all current unrelated exports/registration calls. Slice 8 modifies c_chat_info.rs, NOT this registration anchor; it does not consume or duplicate slice 4's edit.

### src/lua_api/state/sim_state.rs and src/lua_api/state.rs — 2, 4, 5, 7, 8, 11

All field/default anchors are disjoint. Merged intent is the union below, with each packet's original cfg:

| Slice | Added/retyped state | Initial value |
|---|---|---|
| 2 | npc_follower_guids | empty HashSet |
| 4 | nearest_party_member_token; aura_entry_ids | None; default pending None/empty retired IDs |
| 5 | housing_bundles | existing simulator seed 5001, not empty storefront |
| 7 | special_bar_spells; tiered_entrance_pde_id | empty HashSet; numeric 0 |
| 8 | chat_expression_roster | None, distinct from explicit empty roster |
| 11 | cvars becomes Rc<CVarStorage>; taint_log shares that exact Rc | one new CVarStorage and one environment-local sink |

Slice 11 changes macro opening/closing braces and the cvars initializer, not the whole constructor. Use its three constructor replacements together. Preserve current active_transmog_outfit_id, instance_identity, identity_secret_guids, quest_favor, has_active_delve, eligibility request state, and every other existing field/default. Do not initialize CVarStorage twice or change roster/identity defaults to satisfy another slice's tests.

Staged state snapshots from 2/5/7/8/11 omit current active outfit and instanced-identity fields/defaults. They are unsafe whole-file replacements despite every bounded anchor matching. Exact fields/default edits appear in the appendix.

### src/lua_api/globals/state_backed_queries.rs — 4, 11

Slice 4 adds apply_entry_event before event callback dispatch inside dispatch_event_now; slice 11 reinstalls its hook at the beginning of reload_ui before PLAYER_ENTERING_WORLD dispatch. Distinct functions/anchors. Preserve both. ReloadUI's event is not one of the three aura-entry events, so reload must not consume a pending aura rekey batch. No second rekey call inside reload_ui.

### src/lua_api/workarounds/temporary/housing_catalog_state.lua — 5, 9

Slice 5 deletes the bundle seed, copy helper, GetBundleInfo, GetFeaturedBundles, and HousingMarketActionViewBundle. Slice 9 deletes only `SelectCoreFixtureOption = __wow_noop,`. Six independent ranges. Apply their union. Preserve decor market/cart/product state and every unrelated fixture method. Neither slice may copy its full Lua snapshot after the other; that would resurrect the removed publisher/state. Rust bundle registration is unconditional; exterior core callback registration is separate. Neither may be overwritten by its retired Lua placeholder.

### Cargo.toml and Cargo.lock — 10, 11: exact merged-by-intent resolution

Current shared pin is `6044544b960cd68b4b0c58bb3373412757c2caee`. BOTH packets replace precisely the same line. Choose ONE real 40-character SHA R containing:

- `rilua::table_security::transform_host_secret_string(&mut LuaState, Val, impl FnOnce(&[u8]) -> Vec<u8>) -> LuaResult<Val>`;
- `LuaState::set_tainted_table_read_hook` and its immutable, pre-slot-taint-propagation callback contract.

Exact final templates, substituting the SAME supplied R in all three positions:

```toml
rilua = { git = "https://github.com/Osso/rilua.git", rev = "R" }
```

```toml
source = "git+https://github.com/Osso/rilua.git?rev=R#R"
```

`R` is explanatory, not a valid pin; never commit R or `<NEW_RILUA_REV>`. Local inspected candidate revisions differ: ambiguate `70625a96e1556c45845fb4d030f37af3ef896877`, taint hook `1746620e08d8a7f27bef1de6fca857a934baa16c`. Neither handoff proves either revision contains both changes or is fetchable. The integration prerequisite must provide combined revision/lock resolution. Do not pick one candidate on faith, use two pins, introduce a path dependency, or downgrade later. After this single bump both original Cargo anchors intentionally no longer match: skip the packets' four duplicate Cargo operations. If supplied code revision changes dependency metadata, obtain its actual resolved lock entry rather than assuming only a URL changes.

## Current-tree drift and corrected-anchor inventory

**Zero authored anchors are stale in current source.** The appendix records each exact OLD, NEW, current line and match count; line numbers are navigation aids, not replacement authority. Read-only source hashes pin the inspected inputs.

The important drift is outside edit ranges:

- Slice 2's group_queries.rs snapshot predates current `identity_output` and state-aware `unit_identity_is_secret` routing. Replace ONLY unit_treat_as_player_for_display; retain current identity helper, guild/party name publication and secrecy logic.
- Slice 2's real/mod.rs snapshot lacks instanced_identity. Insert nameplate_display at mouse_probes without dropping instanced_identity's current cfg/export.
- Shared state snapshots lack current active outfit/instance defaults. Apply insertions only, as above.
- Slice 11's lua_api/mod.rs snapshot lacks current cast_success module/re-export. Insert taint_log at talent_state and retain CastSuccess wiring.
- No selected packet edits spell_macro_verbs.rs, widgets/mod.rs, outfit actions.rs, unit_misc.rs, instanced_identity.rs or quest-info-system.rs. Preserve /equipset, /tm, /outfit including empty-outfit dispatch, current model-vs-tooltip SetUnit dispatch, outfit selection model, cast filtering, instance secrecy, quest favor and delve instance producers without transplanting old code.

After later authorized integration, validate still-unique OLD text against the current whole file, not historical line 174 or HEAD positions. If input content changes from the hashes below, rerun static anchor preflight; this planner has not authorized mutating work.

## Semantic conflicts and ownership

### Display/identity (2 versus 6 and current working tree)

Only slice 2 replaces unit_treat_as_player_for_display. Slice 6 adds GUID equality evidence and cooldown flag handling; it does NOT modify UnitIsUnit. Follower GUID tagging changes display style only, never TargetInfo.is_player, UnitIsPlayer/UnitIsHumanPlayer, GUID identity, instanced secrecy, friendship or group membership. Keep current GUID/instance model as authoritative; do not reuse follower tags to exempt identities from secrecy. The two tests' NPC/equal-name GUID assertions can coexist.

The new display helper also changes retail pet/vehicle from unconditional true to false and rejects invalid public types; these are deliberate packet changes. There is an additional resolver boundary worth explicit coverage: old visible_party_member requires party_group_active and accepts raid aliases; new resolve_unit_is_player uses present party1..4 entries without checking party_group_active and rejects raid aliases. existing_guid_for_unit has a separate existence gate. Thus inactive-but-retained party entries can return true via the player branch even when existence/GUID lookup returns none; formerly supported raid display aliases can become false. No staged follower test covers these cases. Do not describe the change as follower-only or silently broaden/repair token semantics. If preserving those preexisting display results is required, obtain a bounded authored resolution before application; current handoff intentionally chooses the UnitIsPlayer resolver.

### Private aura sounds (1) versus aura-instance rekey (4)

Separate ID domains: private sound IDs are u32 registration handles/live IDs, with owned unit-token/spell/sound fields; aura_instance_id is i32 on player/party aura records. Rekey only AuraInfo IDs and aura_entry_ids retired/pending state. Never rewrite private sound handles or remove registrations during entry. Preserve sound allocator monotonicity and registration-to-removal lifecycle. Entry events do NOT themselves set world.encounter_in_progress, mythic_plus.is_active or sound pvp_match_active. Host must supply those contexts and the rekey plan separately; no derivation from the event name is authored.

Slice 4 rekeys before each supported dispatch route's consumers (WowLuaEnv, global/admin dispatch, LoaderEnv). Do not add another rekey at per-frame dispatch or in ClassTalent command delivery. Live auras without a supplied entry batch now fail explicitly and atomically; no old ID reuse fallback. Current owned src/tests scan found no existing entry-event fire call sites. New private-sound context tests alter host context directly and do not emit entry events, so there is no direct test conflict. Future combined entry tests must stage IDs first. Target fixture auras are not modeled by this rekey; no row-202 cooldown-viewer claim.

### Aura-header tests (3) versus rekey (4) and current cast filtering

Header cases sort existing expiration/time data and update children using UNIT_AURA, not encounter/M+/PvP entry. They do not need rekey batches. Tests must seed fresh batches only if extended to entry events. Do not add simulator sorting, vendor patches or fabricate loaded cooldown-viewer proof. ClassTalent commands use existing synchronous dispatch and secure-stack restoration; retain current CastSuccess suppression for nonplayer secret instant casts. New command events are not entry events and should not trigger rekey.

### Bundle/defaults/exterior/outfit (5, 7, 9 and current actions)

Keep housing bundle seed 5001 and compatibility entryIDs/wasViewed extensions. The empty-default/exact-six-fields/duplicate-featured-record model in an older parked bundle packet is excluded. Current housing_catalog storefront expects seed plus mutable view flag, and slice 5 preserves that. Slice 9's core_fixture starts None; its exhaustive house_exterior fixture adds None while retaining all assertions. Exterior state and bundle state are independent.

PDEID changes from fixed 77011 to live numeric zero default; do not change has_active_delve or eligibility request state. Empty special_bar_spells preserves existing direct membership; effective slots remain shadowed by macros/outfits. Special membership must NOT invent direct slots, make HasSpellActionButtons true or override /outfit actions. Current outfit actions and the restricted helper tested by slice 9 share the existing catalog, not a second index store; keep actions.rs intact.

### Chat/Ambiguate/taint (8, 10, 11)

Group expansion reads explicit chat_expression_roster, not synthesized party/group identity data. None preserves group tags; an explicit empty list removes known group tags. Secret flags are NeverSecret; secret text/name accepts the distinct AllowedWhenTainted policy. Do not substitute unwrap_secret across all arguments or declassify transformed results. These producers have different input contracts and do not replace each other.

Taint-log observes VM table reads without borrowing SimState and without changing return value/caller taint. It must retain the same CVar Rc as SimState, install only after app data exists, and update the two manually constructed app-data fixtures via WowLuaAppData::new. Existing ReloadUI means dispatch in the existing VM, not native VM recreation. Additional loaded Lua code may create extra records only when taintLog is enabled; exact-record tests use isolated environments and explicit 0→1 gating, not startup-wide logging.

## Required existing-test maintenance, not all present in staging

### Slice 8: startup namespace unit test — mechanical and epoch-aware

The handoff's unconditional expected-icon update would break earlier epochs, because expressions::register is called only under retail-12-0-5 registration. Use all three epoch-aware exact OLD/NEW replacements in the appendix for `src/loader/tests/wow_api_globals/startup_namespaces.rs::test_startup_bootstrap_namespaces_exist`. Current anchors match once. Assert concrete texture markup for the new producer; earlier epochs observe API type `nil` rather than calling a removed publisher. The first edit is the handoff's required expectation update with corrected epoch scope; the other two are planned test-fixture maintenance. These edits are absent from edits.json and must not be forgotten. Removal of the temporary expression publisher is unconditional, so an older-epoch literal-identity assertion would also be wrong. Older-profile c_chat_info_probes and installs_chat_info_no_state_defaults still call this removed API; their broader-profile scope needs corresponding expectation authoring before older-profile GREEN can be claimed. Retail calls returning plain `hello` remain unchanged; preserves_existing_chat_info_provider remains a valid explicit-provider control.

### Slice 3: five fixture maintenance edits — mechanical

`tests/admin_spec_talent_api.rs::test_trait_config_mapping_tracks_active_loadout`: replace the single bare-environment SwitchToLoadoutByIndex(2) fixture call with `C_ClassTalents.LoadConfig(C_ClassTalents.GetConfigIDsBySpecID(66)[2], true)`. Its mapping assertions stay unchanged.

For the non-selectable-glow case in hero_talents.rs and three named rendering.rs cases, establish Protection through host fixture state (`player.active_spec_index = 2; talents.switch_to_spec(66)`) and remove the Lua SwitchToSpecializationByName("Protection") fixture call. Exact function-scoped prefixes and replacements are in the appendix. No renderer assertions change; A_Admin.SetSpec alone is insufficient. These five maintenance edits are proposed integration resolutions, not staged packet edits or executed tests.

### Slice 3: command completion control — unresolved authoring prerequisite

`tests/hero_talents.rs::test_class_talents_switch_methods_update_seeded_spec_and_loadout_state` currently assumes four bare-environment SwitchTo* commands immediately mutate spec/loadout and expects `202,102,301`. New producer emits command events; without loaded vendor listeners no transition occurs. This is a definite semantic conflict, not an anchor mismatch. The staged helper tests exercise real UI initiation/taint neutrality and a pending spec cast, not completion. A correct replacement must load the real cached helper/UI, assert command delivery and pending activation, drive the modeled completion lifecycle, then assert actual selected spec/config. Preserve the old direct-state behavior only under its pre-12.0.5 provider if historical coverage is retained. Do not cfg-disable retail coverage without a replacement, reduce expected values to unchanged state as a fake completion test, monkey-patch callbacks, or restore direct-state fallback. No exact completion-test replacement is supplied by the packet; single mechanical worker cannot invent it. This blocks a full-green completion claim, not application of other slices.

### Slice 4: older-profile navigation scope

Remove only the existing nearest-token-nil assertions already listed in its edits. Its replacement entrypoint is retail-12-0-5 gated, while removal of the temporary publisher is unconditional. Consequently blizzard_quest_navigation_consumes_c_super_track_namespace's function-presence expectation is unavailable on older epochs. Separate that expectation by actual provider scope; do not invent a nil fallback. This is a known broader-profile verification gap, not an 11-slice textual collision.

## Future proof plan — not executed here

Use existing generated `integration` target: filters are module prefixes, not `--test <new filename>`. Confirm current selected profile/epoch explicitly; the authored historical recipe is `python3 scripts/build-host.py --build-host local --no-default-features --features profile-retail,retail-12-0-5 --test --test integration FILTER -- --nocapture`. This is a future command template, not proof that it ran. Batch related controls; do not invoke full Wowless/self-test. Library tests use --lib/appropriate lib filter separately.

New module filters and existing controls appear in the generated table below. Most new modules are retail-12-0-5 gated; housing_bundle_structures intentionally has all-profile tests and only its exact-shape case is gated. private_aura_sound_add_context has six common cases and two later-client-only modern AddAuraSound cases. Counts below are annotations, not tests discovered or passed. All-source compilation/check/startup/profile proof remains unperformed.

A final future ledger must identify the integrated revision, changed scope, command and actual outcome. No previous packet's historical passing report proves the combined new revision. New dependencies require corresponding controls, and the unresolved lifecycle test must be settled before presenting the combined branch as passing/ready. This plan adds no audit acceptance or native-parity credit.

## File × slice matrix — every existing edited file

A = authored packet edit; M = required/proposed existing-test maintenance absent from staging. A/M means both in the same file. Dependency rows are overlapping and must be deduplicated.
| Existing path | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| `Cargo.lock` | — | — | — | — | — | — | — | — | — | A | A |
| `Cargo.toml` | — | — | — | — | — | — | — | — | — | A | A |
| `docs/specs/action-bar-membership.md` | — | — | — | — | — | — | A | — | — | — | — |
| `docs/specs/ambiguate-context.md` | — | — | — | — | — | — | — | — | — | A | — |
| `docs/specs/delves-api-inputs.md` | — | — | — | — | — | — | A | — | — | — | — |
| `docs/specs/private-aura-sound-removal.md` | A | — | — | — | — | — | — | — | — | — | — |
| `docs/specs/spellbook-cooldown-duration.md` | — | — | — | — | — | A | — | — | — | — | — |
| `docs/specs/unit-identity-equality.md` | — | — | — | — | — | A | — | — | — | — | — |
| `src/c_api/c_action_bar.rs` | — | — | — | — | — | A | — | — | — | — | — |
| `src/c_api/c_action_bar_spell_slots.rs` | — | — | — | — | — | — | A | — | — | — | — |
| `src/c_api/c_chat_info.rs` | — | — | — | — | — | — | — | A | — | — | — |
| `src/c_api/c_housing/exterior.rs` | — | — | — | — | — | — | — | — | A | — | — |
| `src/c_api/c_housing/exterior/mutation.rs` | — | — | — | — | — | — | — | — | A | — | — |
| `src/c_api/c_housing/exterior/runtime.rs` | — | — | — | — | — | — | — | — | A | — | — |
| `src/c_api/c_spell_book.rs` | — | — | — | — | — | A | — | — | — | — | — |
| `src/c_api/cooldown_duration.rs` | — | — | — | — | — | A | — | — | — | — | — |
| `src/c_api/mod.rs` | — | — | A | A | A | — | — | — | — | — | — |
| `src/c_api/private_aura_sounds.rs` | A | — | — | — | — | — | — | — | — | — | — |
| `src/loader/addon_modules.rs` | — | — | — | — | — | — | — | — | — | — | A |
| `src/loader/tests/wow_api_globals/startup_namespaces.rs` | — | — | — | — | — | — | — | M | — | — | — |
| `src/lua_api/env.rs` | — | — | — | — | — | — | — | — | — | — | A |
| `src/lua_api/env_events.rs` | — | — | — | A | — | — | — | — | — | — | — |
| `src/lua_api/env_init/mod.rs` | — | — | — | — | — | — | — | — | — | — | A |
| `src/lua_api/globals/group_queries.rs` | — | A | — | — | — | — | — | — | — | — | — |
| `src/lua_api/globals/missing_surface.rs` | — | — | — | — | A | — | — | — | — | — | — |
| `src/lua_api/globals/missing_surface/delves_ui.rs` | — | — | — | — | — | — | A | — | — | — | — |
| `src/lua_api/globals/missing_surface/traits/class_talents.rs` | — | — | A | — | — | — | — | — | — | — | — |
| `src/lua_api/globals/real/ambiguate.rs` | — | — | — | — | — | — | — | — | — | A | — |
| `src/lua_api/globals/real/mod.rs` | — | A | — | — | — | — | — | — | — | — | — |
| `src/lua_api/globals/state_backed_queries.rs` | — | — | — | A | — | — | — | — | — | — | A |
| `src/lua_api/globals/unit_probes.rs` | — | A | — | — | — | — | — | — | — | — | — |
| `src/lua_api/loader_env.rs` | — | — | — | A | — | — | — | — | — | — | — |
| `src/lua_api/mod.rs` | — | — | — | — | — | — | — | — | — | — | A |
| `src/lua_api/state.rs` | — | A | — | A | A | — | A | A | — | — | A |
| `src/lua_api/state/sim_state.rs` | — | A | — | A | A | — | A | A | — | — | A |
| `src/lua_api/workarounds/temporary/c_chat_info_defaults.rs` | — | — | — | — | — | — | — | A | — | — | — |
| `src/lua_api/workarounds/temporary/housing_catalog_state.lua` | — | — | — | — | A | — | — | — | A | — | — |
| `src/lua_api/workarounds/temporary/navigation_defaults.rs` | — | — | — | A | — | — | — | — | — | — | — |
| `tests/admin_spec_talent_api.rs` | — | — | M | — | — | — | — | — | — | — | — |
| `tests/c_navigation_probes.rs` | — | — | — | A | — | — | — | — | — | — | — |
| `tests/hero_talents.rs` | — | — | M | — | — | — | — | — | — | — | — |
| `tests/hero_talents/rendering.rs` | — | — | M | — | — | — | — | — | — | — | — |
| `tests/house_exterior.rs` | — | — | — | — | — | — | — | — | A | — | — |
| `tests/private_aura_sound_removal.rs` | A | — | — | — | — | — | — | — | — | — | — |

The unresolved replacement of hero_talents::test_class_talents_switch_methods_update_seeded_spec_and_loadout_state belongs to the already-listed slice-3 tests/hero_talents.rs maintenance row. Its OLD function exists; there is no authorized exact NEW completion-lifecycle body to apply yet.

## New filters and control suites

| Slice | New integration module filters (test annotations) | Existing integration controls |
|---|---|---|
| 1 | `private_aura_sound_add_context::` (8) | `private_aura_sound_removal::`, `private_aura_anchors::`, `unit_auras_private::` |
| 2 | `follower_nameplate_display::` (7) | `unit_api::`, `unit_comparison_permissions::`, `unit_token_identity_secrecy::`, `instanced_identity::` |
| 3 | `secure_aura_header_helpers::` (5) | `admin_spec_talent_api::test_trait_config_mapping_tracks_active_loadout`, `hero_talents::`, `hero_talents::rendering::`, `class_talents_config::`, `talent_change_events::` |
| 4 | `patch_12_0_5_navigation_aura_entry::` (8) | `c_navigation_probes::`, `aura_instance_ids::`, `aura_api::`, `aura_table_shape::`, `blizzard_quest_navigation_loads::` |
| 5 | `housing_bundle_structures::` (6) | `housing_catalog::housing_catalog_storefront_and_market_methods_use_seeded_state`, `housing_catalog_aggregates::`, `housing_catalog_base_lookups::` |
| 6 | `retail_12_0_5_partial_104_114::` (5) | `cooldown_probes::ignore_gcd::`, `unit_comparison_permissions::`, `unit_api::test_unit_is_unit`, `cooldown_widget::`, `spell_book_cooldown_outputs::` |
| 7 | `pdeid_specialbar::` (8) | `action_bar_membership::`, `action_spell_slot_identifiers::`, `delves_api_inputs::`, `delves_ui::`, `delve_instance_state::` |
| 8 | `chat_expressions::` (9) | `c_chat_info_probes::`, `chat_messaging_lockdown::`, `chat_frame_util::` |
| 9 | `house_exterior_core_fixture::` (10); `restricted_outfit_index::` (4) | `house_exterior::`, `house_exterior_attached_decor::`, `patch_12_0_5_outfit_catalog::`, `outfit_action_command::`, `blizzard_restricted_addon_environment_loads::` |
| 10 | `ambiguate_secret_name::` (5) | `ambiguate_context::`, `system_api::`, `secret_value_security::` |
| 11 | `retail_12_0_5_taint_log::` (10) | `security_api::`, `security_state_drivers::`, `table_util::`, `secret_string_formatting::`, `retail_12_0_5_addon_table_freeze::` |

Library controls: startup_namespaces::test_startup_bootstrap_namespaces_exist (8); c_chat_info_defaults::tests::{installs_chat_info_no_state_defaults,preserves_existing_chat_info_provider} (8); navigation_defaults::tests::{installs_safe_empty_navigation_defaults,preserves_existing_navigation_provider} (4); affected env_init/addon_modules app-data fixtures (11). Match the actual library module-qualified names when invoking --lib. These are separate from the generated integration target. Broader-profile navigation/chat gaps above remain open.

Combined current-work controls to retain: `cast_events_identity::`, `instanced_identity::`, `unit_token_identity_secrecy::`, `outfit_action_command::`, `mouse_tm_commands::`, `equipment_set_command::`, `delve_instance_state::`, `quest_reward_favor::`, `spell_macro_verbs::`, `cast_events_identity::model_identity::`, and `pending_transmog_cost::`. Model SetUnit/actor identity controls are in the inspected `mod model_identity` inside cast_events_identity. No tests have been executed by this planner.

## New-file allowlist and collision inventory

Copy ONLY these repo-relative paths from their selected staging directory. All 34 paths are absent from the current repo; no destination is supplied by two selected slices. No new-file name/path collision within the eleven selected packets or against current repo. Files with common generic basenames such as core.rs/inputs.rs are distinct qualified destinations. Existing-file snapshots and JSON/patch/manifests are NOT new repo files. Parked unselected staging packets are not integration inputs.

| New destination | Slice | Current repo |
|---|---|---|
| `docs/specs/action-bar-special-membership.md` | 7 | absent |
| `docs/specs/ambiguate-secret-name.md` | 10 | absent |
| `docs/specs/aura-entry-instance-ids.md` | 4 | absent |
| `docs/specs/chat-expressions.md` | 8 | absent |
| `docs/specs/follower-nameplate-display.md` | 2 | absent |
| `docs/specs/house-exterior-core-fixture.md` | 9 | absent |
| `docs/specs/housing-bundle-structures.md` | 5 | absent |
| `docs/specs/navigation-nearest-party-token.md` | 4 | absent |
| `docs/specs/private-aura-sound-add-context.md` | 1 | absent |
| `docs/specs/restricted-outfit-index.md` | 9 | absent |
| `docs/specs/tiered-entrance-pdeid.md` | 7 | absent |
| `src/c_api/aura_entry.rs` | 4 | absent |
| `src/c_api/aura_entry_ids.rs` | 4 | absent |
| `src/c_api/c_chat_info/expressions.rs` | 8 | absent |
| `src/c_api/c_housing/exterior/core.rs` | 9 | absent |
| `src/c_api/c_housing_bundles.rs` | 5 | absent |
| `src/c_api/c_navigation.rs` | 4 | absent |
| `src/c_api/class_talent_commands.rs` | 3 | absent |
| `src/c_api/private_aura_sounds/add.rs` | 1 | absent |
| `src/c_api/private_aura_sounds/inputs.rs` | 1 | absent |
| `src/lua_api/globals/real/nameplate_display.rs` | 2 | absent |
| `src/lua_api/taint_log.rs` | 11 | absent |
| `tests/ambiguate_secret_name.rs` | 10 | absent |
| `tests/chat_expressions.rs` | 8 | absent |
| `tests/follower_nameplate_display.rs` | 2 | absent |
| `tests/house_exterior_core_fixture.rs` | 9 | absent |
| `tests/housing_bundle_structures.rs` | 5 | absent |
| `tests/patch_12_0_5_navigation_aura_entry.rs` | 4 | absent |
| `tests/pdeid_specialbar.rs` | 7 | absent |
| `tests/private_aura_sound_add_context.rs` | 1 | absent |
| `tests/restricted_outfit_index.rs` | 9 | absent |
| `tests/retail_12_0_5_partial_104_114.rs` | 6 | absent |
| `tests/retail_12_0_5_taint_log.rs` | 11 | absent |
| `tests/secure_aura_header_helpers.rs` | 3 | absent |

## Exact-anchor ledger

96 authored replacements + 1 handoff-required chat assertion + 7 proposed mechanical maintenance replacements = 104 operations before deduplicating the two repeated Cargo lines. Every listed OLD matches once at the inspected current bytes. Original OLD blocks were read from handoffs/edits JSON; corrected maintenance is explicitly labeled. JSON below preserves whitespace/newline bytes exactly. Do not copy stale staged snapshots. The Cargo templates are prerequisites to resolve once, not two independent application operations.

| Slice/edit | Existing path | Current OLD start line | Matches | Classification |
|---|---|---|---|---|
| 1.1 | `src/c_api/private_aura_sounds.rs` | 3 | 1 | ### 1. STATE — `src/c_api/private_aura_sounds.rs` |
| 1.2 | `src/c_api/private_aura_sounds.rs` | 12 | 1 | ### 2. STATE — `src/c_api/private_aura_sounds.rs` |
| 1.3 | `src/c_api/private_aura_sounds.rs` | 10 | 1 | ### 3. PRODUCER — `src/c_api/private_aura_sounds.rs` |
| 1.4 | `src/c_api/private_aura_sounds.rs` | 26 | 1 | ### 4. PRODUCER — `src/c_api/private_aura_sounds.rs` |
| 1.5 | `src/c_api/private_aura_sounds.rs` | 62 | 1 | ### 5. PRODUCER — `src/c_api/private_aura_sounds.rs` |
| 1.6 | `tests/private_aura_sound_removal.rs` | 257 | 1 | ### 6. STATE — `tests/private_aura_sound_removal.rs` |
| 1.7 | `docs/specs/private-aura-sound-removal.md` | 94 | 1 | ### 7. DOC — `docs/specs/private-aura-sound-removal.md` |
| 1.8 | `src/c_api/private_aura_sounds.rs` | 1 | 1 | ### 8. PRODUCER — `src/c_api/private_aura_sounds.rs` |
| 2.1 | `src/lua_api/state/sim_state.rs` | 252 | 1 | ### STATE: `src/lua_api/state/sim_state.rs` |
| 2.2 | `src/lua_api/state.rs` | 230 | 1 | ### STATE: `src/lua_api/state.rs` |
| 2.3 | `src/lua_api/globals/unit_probes.rs` | 68 | 1 | ### PRODUCER: `src/lua_api/globals/unit_probes.rs` |
| 2.4 | `src/lua_api/globals/real/mod.rs` | 43 | 1 | ### PRODUCER: `src/lua_api/globals/real/mod.rs` |
| 2.5 | `src/lua_api/globals/group_queries.rs` | 647 | 1 | ### PRODUCER: `src/lua_api/globals/group_queries.rs` |
| 3.1 | `src/c_api/mod.rs` | 130 | 1 | packet edit |
| 3.2 | `src/lua_api/globals/missing_surface/traits/class_talents.rs` | 10 | 1 | packet edit |
| 3.3 | `src/lua_api/globals/missing_surface/traits/class_talents.rs` | 236 | 1 | packet edit |
| 3.4 | `src/lua_api/globals/missing_surface/traits/class_talents.rs` | 254 | 1 | packet edit |
| 3.5 | `src/lua_api/globals/missing_surface/traits/class_talents.rs` | 267 | 1 | packet edit |
| 3.6 | `src/lua_api/globals/missing_surface/traits/class_talents.rs` | 282 | 1 | packet edit |
| 3.7 | `src/lua_api/globals/missing_surface/traits/class_talents.rs` | 106 | 1 | packet edit |
| 3.8 | `tests/admin_spec_talent_api.rs` | 277 | 1 | planned maintenance |
| 3.9 | `tests/hero_talents.rs` | 392 | 1 | planned maintenance |
| 3.10 | `tests/hero_talents/rendering.rs` | 4 | 1 | planned maintenance |
| 3.11 | `tests/hero_talents/rendering.rs` | 154 | 1 | planned maintenance |
| 3.12 | `tests/hero_talents/rendering.rs` | 228 | 1 | planned maintenance |
| 4.1 | `src/c_api/mod.rs` | 17 | 1 | state |
| 4.2 | `src/lua_api/state/sim_state.rs` | 323 | 1 | state |
| 4.3 | `src/lua_api/state.rs` | 289 | 1 | state |
| 4.4 | `src/c_api/mod.rs` | 81 | 1 | producer |
| 4.5 | `src/c_api/mod.rs` | 233 | 1 | producer |
| 4.6 | `src/lua_api/env_events.rs` | 232 | 1 | producer |
| 4.7 | `src/lua_api/globals/state_backed_queries.rs` | 68 | 1 | producer |
| 4.8 | `src/lua_api/loader_env.rs` | 152 | 1 | producer |
| 4.9 | `src/lua_api/workarounds/temporary/navigation_defaults.rs` | 31 | 1 | producer |
| 4.10 | `src/lua_api/workarounds/temporary/navigation_defaults.rs` | 59 | 1 | state |
| 4.11 | `src/lua_api/workarounds/temporary/navigation_defaults.rs` | 67 | 1 | state |
| 4.12 | `src/lua_api/workarounds/temporary/navigation_defaults.rs` | 73 | 1 | state |
| 4.13 | `tests/c_navigation_probes.rs` | 15 | 1 | state |
| 4.14 | `tests/c_navigation_probes.rs` | 17 | 1 | state |
| 4.15 | `tests/c_navigation_probes.rs` | 25 | 1 | state |
| 4.16 | `tests/c_navigation_probes.rs` | 35 | 1 | state |
| 5.1 | `src/c_api/mod.rs` | 43 | 1 | ### [state/producer] `src/c_api/mod.rs` |
| 5.2 | `src/lua_api/state/sim_state.rs` | 345 | 1 | ### [state] `src/lua_api/state/sim_state.rs` |
| 5.3 | `src/lua_api/state.rs` | 309 | 1 | ### [state] `src/lua_api/state.rs` |
| 5.4 | `src/lua_api/globals/missing_surface.rs` | 259 | 1 | ### [producer] `src/lua_api/globals/missing_surface.rs` |
| 5.5 | `src/lua_api/workarounds/temporary/housing_catalog_state.lua` | 125 | 1 | ### [state] `src/lua_api/workarounds/temporary/housing_catalog_state.lua` |
| 5.6 | `src/lua_api/workarounds/temporary/housing_catalog_state.lua` | 526 | 1 | ### [producer] `src/lua_api/workarounds/temporary/housing_catalog_state.lua` |
| 5.7 | `src/lua_api/workarounds/temporary/housing_catalog_state.lua` | 907 | 1 | ### [producer] `src/lua_api/workarounds/temporary/housing_catalog_state.lua` |
| 5.8 | `src/lua_api/workarounds/temporary/housing_catalog_state.lua` | 914 | 1 | ### [producer] `src/lua_api/workarounds/temporary/housing_catalog_state.lua` |
| 5.9 | `src/lua_api/workarounds/temporary/housing_catalog_state.lua` | 946 | 1 | ### [producer] `src/lua_api/workarounds/temporary/housing_catalog_state.lua` |
| 6.1 | `src/c_api/cooldown_duration.rs` | 9 | 1 | producer |
| 6.2 | `src/c_api/c_action_bar.rs` | 3 | 1 | producer |
| 6.3 | `src/c_api/c_action_bar.rs` | 81 | 1 | producer |
| 6.4 | `src/c_api/c_spell_book.rs` | 470 | 1 | producer |
| 6.5 | `docs/specs/unit-identity-equality.md` | 67 | 1 | state/spec |
| 6.6 | `docs/specs/spellbook-cooldown-duration.md` | 36 | 1 | state/spec |
| 7.1 | `src/lua_api/state/sim_state.rs` | 117 | 1 | ### 1. [state] `src/lua_api/state/sim_state.rs` |
| 7.2 | `src/lua_api/state/sim_state.rs` | 198 | 1 | ### 2. [state] `src/lua_api/state/sim_state.rs` |
| 7.3 | `src/lua_api/state.rs` | 133 | 1 | ### 3. [state] `src/lua_api/state.rs` |
| 7.4 | `src/lua_api/state.rs` | 189 | 1 | ### 4. [state] `src/lua_api/state.rs` |
| 7.5 | `src/lua_api/globals/missing_surface/delves_ui.rs` | 287 | 1 | ### 5. [producer] `src/lua_api/globals/missing_surface/delves_ui.rs` |
| 7.6 | `src/c_api/c_action_bar_spell_slots.rs` | 73 | 1 | ### 6. [producer] `src/c_api/c_action_bar_spell_slots.rs` |
| 7.7 | `docs/specs/delves-api-inputs.md` | 73 | 1 | ### 7. [documentation] `docs/specs/delves-api-inputs.md` |
| 7.8 | `docs/specs/action-bar-membership.md` | 7 | 1 | ### 8. [documentation] `docs/specs/action-bar-membership.md` |
| 7.9 | `docs/specs/action-bar-membership.md` | 76 | 1 | ### 9. [documentation] `docs/specs/action-bar-membership.md` |
| 7.10 | `docs/specs/action-bar-membership.md` | 81 | 1 | ### 10. [documentation] `docs/specs/action-bar-membership.md` |
| 8.1 | `src/c_api/c_chat_info.rs` | 1 | 1 | packet edit |
| 8.2 | `src/c_api/c_chat_info.rs` | 15 | 1 | packet edit |
| 8.3 | `src/lua_api/state/sim_state.rs` | 49 | 1 | packet edit |
| 8.4 | `src/lua_api/state.rs` | 62 | 1 | packet edit |
| 8.5 | `src/lua_api/workarounds/temporary/c_chat_info_defaults.rs` | 27 | 1 | packet edit |
| 8.6 | `src/loader/tests/wow_api_globals/startup_namespaces.rs` | 87 | 1 | planned maintenance |
| 8.7 | `src/loader/tests/wow_api_globals/startup_namespaces.rs` | 44 | 1 | planned maintenance |
| 8.8 | `src/loader/tests/wow_api_globals/startup_namespaces.rs` | 69 | 1 | planned maintenance |
| 9.1 | `src/c_api/c_housing/exterior.rs` | 27 | 1 | state |
| 9.2 | `src/c_api/c_housing/exterior.rs` | 35 | 1 | state |
| 9.3 | `src/c_api/c_housing/exterior/runtime.rs` | 3 | 1 | producer |
| 9.4 | `src/c_api/c_housing/exterior/runtime.rs` | 26 | 1 | producer |
| 9.5 | `src/c_api/c_housing/exterior/runtime.rs` | 78 | 1 | producer |
| 9.6 | `src/c_api/c_housing/exterior/runtime.rs` | 91 | 1 | producer |
| 9.7 | `src/c_api/c_housing/exterior/runtime.rs` | 115 | 1 | producer |
| 9.8 | `src/c_api/c_housing/exterior/runtime.rs` | 146 | 1 | producer |
| 9.9 | `src/c_api/c_housing/exterior/mutation.rs` | 89 | 1 | producer |
| 9.10 | `src/c_api/c_housing/exterior/mutation.rs` | 122 | 1 | producer |
| 9.11 | `src/c_api/c_housing/exterior/mutation.rs` | 199 | 1 | producer |
| 9.12 | `src/lua_api/workarounds/temporary/housing_catalog_state.lua` | 899 | 1 | producer |
| 9.13 | `tests/house_exterior.rs` | 175 | 1 | state |
| 10.1 | `src/lua_api/globals/real/ambiguate.rs` | 1 | 1 | ### [producer] `src/lua_api/globals/real/ambiguate.rs` |
| 10.2 | `Cargo.toml` | 26 | 1 | ### [producer prerequisite] `Cargo.toml:26` |
| 10.3 | `Cargo.lock` | 4994 | 1 | ### [producer prerequisite] `Cargo.lock:4994` |
| 10.4 | `docs/specs/ambiguate-context.md` | 55 | 1 | ### [state] `docs/specs/ambiguate-context.md:55` |
| 11.1 | `src/lua_api/mod.rs` | 53 | 1 | state |
| 11.2 | `src/lua_api/state/sim_state.rs` | 58 | 1 | state |
| 11.3 | `src/lua_api/state.rs` | 16 | 1 | state |
| 11.4 | `src/lua_api/state.rs` | 44 | 1 | state |
| 11.5 | `src/lua_api/state.rs` | 478 | 1 | state |
| 11.6 | `src/lua_api/env.rs` | 25 | 1 | state |
| 11.7 | `src/lua_api/env.rs` | 44 | 1 | state |
| 11.8 | `src/lua_api/env.rs` | 118 | 1 | producer |
| 11.9 | `src/lua_api/globals/state_backed_queries.rs` | 107 | 1 | producer |
| 11.10 | `src/lua_api/env_init/mod.rs` | 294 | 1 | state / existing-test |
| 11.11 | `src/loader/addon_modules.rs` | 253 | 1 | state / existing-test |
| 11.12 | `Cargo.toml` | 26 | 1 | producer / dependency |
| 11.13 | `Cargo.lock` | 4994 | 1 | producer / dependency |

## Exact edits — machine-readable appendix

Apply all edits for one file against its ORIGINAL input. Do not evaluate later oldText against earlier replacements in the same file. For slices 10/11 omit Cargo edits after the unified resolved bump. For slice 3 also resolve the missing command-completion test described above; the five M fixture edits below do not replace it.

### Slice 1: private-aura-sound

```json
[
  {
    "path": "src/c_api/private_aura_sounds.rs",
    "classification": "### 1. STATE — `src/c_api/private_aura_sounds.rs`",
    "oldText": "use std::collections::HashSet;",
    "newText": "mod inputs;\npub use inputs::{AuraSoundRegistration, PrivateAuraSoundRegistrations};"
  },
  {
    "path": "src/c_api/private_aura_sounds.rs",
    "classification": "### 2. STATE — `src/c_api/private_aura_sounds.rs`",
    "oldText": "#[derive(Default)]\npub struct PrivateAuraSoundRegistrations {\n    /// Explicit live IDs only. The ID domain and removal policies are simulator inferences.\n    pub live_ids: HashSet<u32>,\n}\n\n",
    "newText": ""
  },
  {
    "path": "src/c_api/private_aura_sounds.rs",
    "classification": "### 3. PRODUCER — `src/c_api/private_aura_sounds.rs`",
    "oldText": "const API_NAME: &str = \"C_UnitAuras.RemovePrivateAuraAppliedSound\";",
    "newText": "mod add;\n\nconst API_NAME: &str = \"C_UnitAuras.RemovePrivateAuraAppliedSound\";"
  },
  {
    "path": "src/c_api/private_aura_sounds.rs",
    "classification": "### 4. PRODUCER — `src/c_api/private_aura_sounds.rs`",
    "oldText": "    // The cached 12.1 deprecated chunk aliases legacy to modern after bootstrap.",
    "newText": "    table_set_rust_fn_static(\n        state,\n        namespace,\n        \"AddPrivateAuraAppliedSound\",\n        add::add_private,\n    )?;\n    #[cfg(feature = \"retail-12-1-0\")]\n    table_set_rust_fn_static(state, namespace, \"AddAuraSound\", add::add_modern)?;\n    // The cached 12.1 deprecated chunk aliases legacy to modern after bootstrap."
  },
  {
    "path": "src/c_api/private_aura_sounds.rs",
    "classification": "### 5. PRODUCER — `src/c_api/private_aura_sounds.rs`",
    "oldText": "    borrow_state_mut(state)?\n        .private_aura_sound_registrations\n        .live_ids\n        .remove(&id);",
    "newText": "    let mut sim = borrow_state_mut(state)?;\n    let sounds = &mut sim.private_aura_sound_registrations;\n    sounds.live_ids.remove(&id);\n    sounds.registrations.remove(&id);"
  },
  {
    "path": "tests/private_aura_sound_removal.rs",
    "classification": "### 6. STATE — `tests/private_aura_sound_removal.rs`",
    "oldText": "        live_ids: HashSet::from([101, 303]),",
    "newText": "        live_ids: HashSet::from([101, 303]),\n        ..Default::default()"
  },
  {
    "path": "docs/specs/private-aura-sound-removal.md",
    "classification": "### 7. DOC — `docs/specs/private-aura-sound-removal.md`",
    "oldText": "- [ ] Add acquisition remains unmodeled; host-seeded removal does not establish Add-to-Remove lifecycle.",
    "newText": "- [ ] Separate [Add context model](private-aura-sound-add-context.md) awaits parent proof/acceptance; historical host-seeded removal proof does not establish acquisition or Add-to-Remove lifecycle."
  },
  {
    "path": "src/c_api/private_aura_sounds.rs",
    "classification": "### 8. PRODUCER — `src/c_api/private_aura_sounds.rs`",
    "oldText": "//! INFERRED host-declared sound registrations; native acquisition and playback unknown.",
    "newText": "//! INFERRED sound registration/removal model; no native acquisition or playback parity."
  }
]
```

### Slice 2: editmode-nameplate

```json
[
  {
    "path": "src/lua_api/state/sim_state.rs",
    "classification": "### STATE: `src/lua_api/state/sim_state.rs`",
    "oldText": "    pub current_focus: Option<TargetInfo>,",
    "newText": "    pub current_focus: Option<TargetInfo>,\n    /// Explicit host NPC-follower identities for player-style display.\n    /// INFERRED: no followers until host input; does not change human identity.\n    #[cfg(feature = \"retail-12-0-5\")]\n    pub npc_follower_guids: std::collections::HashSet<String>,"
  },
  {
    "path": "src/lua_api/state.rs",
    "classification": "### STATE: `src/lua_api/state.rs`",
    "oldText": "            current_focus: $runtime.current_focus,",
    "newText": "            current_focus: $runtime.current_focus,\n            #[cfg(feature = \"retail-12-0-5\")]\n            npc_follower_guids: HashSet::new(),"
  },
  {
    "path": "src/lua_api/globals/unit_probes.rs",
    "classification": "### PRODUCER: `src/lua_api/globals/unit_probes.rs`",
    "oldText": "fn resolve_unit_is_player(sim: &SimState, token: &str) -> bool {",
    "newText": "pub(crate) fn resolve_unit_is_player(sim: &SimState, token: &str) -> bool {"
  },
  {
    "path": "src/lua_api/globals/real/mod.rs",
    "classification": "### PRODUCER: `src/lua_api/globals/real/mod.rs`",
    "oldText": "pub mod mouse_probes;",
    "newText": "pub mod mouse_probes;\n#[cfg(feature = \"retail-12-0-5\")]\npub(crate) mod nameplate_display;"
  },
  {
    "path": "src/lua_api/globals/group_queries.rs",
    "classification": "### PRODUCER: `src/lua_api/globals/group_queries.rs`",
    "oldText": "fn unit_treat_as_player_for_display(state: &mut LuaState) -> LuaResult<u32> {\n    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();\n    let treat_as_player = {\n        let st = borrow_state(state)?;\n        match unit.as_str() {\n            \"player\" | \"pet\" | \"vehicle\" => true,\n            \"target\" => st\n                .current_target\n                .as_ref()\n                .map(|target| target.is_player)\n                .unwrap_or(false),\n            \"focus\" => st\n                .current_focus\n                .as_ref()\n                .map(|target| target.is_player)\n                .unwrap_or(false),\n            other => visible_party_member(&st, other).is_some(),\n        }\n    };\n    state.push(Val::Bool(treat_as_player));\n    Ok(1)\n}\n",
    "newText": "#[cfg(feature = \"retail-12-0-5\")]\nfn unit_treat_as_player_for_display(state: &mut LuaState) -> LuaResult<u32> {\n    super::real::nameplate_display::unit_treat_as_player_for_display(state)\n}\n\n#[cfg(not(feature = \"retail-12-0-5\"))]\nfn unit_treat_as_player_for_display(state: &mut LuaState) -> LuaResult<u32> {\n    let unit = Option::<String>::from_stack(state, 1)?.unwrap_or_default();\n    let treat_as_player = {\n        let st = borrow_state(state)?;\n        match unit.as_str() {\n            \"player\" | \"pet\" | \"vehicle\" => true,\n            \"target\" => st\n                .current_target\n                .as_ref()\n                .map(|target| target.is_player)\n                .unwrap_or(false),\n            \"focus\" => st\n                .current_focus\n                .as_ref()\n                .map(|target| target.is_player)\n                .unwrap_or(false),\n            other => visible_party_member(&st, other).is_some(),\n        }\n    };\n    state.push(Val::Bool(treat_as_player));\n    Ok(1)\n}\n"
  }
]
```

### Slice 3: aura-header

```json
[
  {
    "path": "src/c_api/mod.rs",
    "classification": "",
    "oldText": "pub(crate) mod c_transmog_collection;\n",
    "newText": "pub(crate) mod c_transmog_collection;\n#[cfg(feature = \"retail-12-0-5\")]\npub(crate) mod class_talent_commands;\n"
  },
  {
    "path": "src/lua_api/globals/missing_surface/traits/class_talents.rs",
    "classification": "",
    "oldText": "fn current_config_ids(",
    "newText": "#[cfg(not(feature = \"retail-12-0-5\"))]\nfn current_config_ids("
  },
  {
    "path": "src/lua_api/globals/missing_surface/traits/class_talents.rs",
    "classification": "",
    "oldText": "fn c_class_talents_switch_to_loadout_by_name(",
    "newText": "#[cfg(not(feature = \"retail-12-0-5\"))]\nfn c_class_talents_switch_to_loadout_by_name("
  },
  {
    "path": "src/lua_api/globals/missing_surface/traits/class_talents.rs",
    "classification": "",
    "oldText": "fn c_class_talents_switch_to_loadout_by_index(",
    "newText": "#[cfg(not(feature = \"retail-12-0-5\"))]\nfn c_class_talents_switch_to_loadout_by_index("
  },
  {
    "path": "src/lua_api/globals/missing_surface/traits/class_talents.rs",
    "classification": "",
    "oldText": "fn c_class_talents_switch_to_specialization_by_name(",
    "newText": "#[cfg(not(feature = \"retail-12-0-5\"))]\nfn c_class_talents_switch_to_specialization_by_name("
  },
  {
    "path": "src/lua_api/globals/missing_surface/traits/class_talents.rs",
    "classification": "",
    "oldText": "fn c_class_talents_switch_to_specialization_by_index(",
    "newText": "#[cfg(not(feature = \"retail-12-0-5\"))]\nfn c_class_talents_switch_to_specialization_by_index("
  },
  {
    "path": "src/lua_api/globals/missing_surface/traits/class_talents.rs",
    "classification": "",
    "oldText": "fn register_c_class_talents_action_fns(\n",
    "newText": "#[cfg(feature = \"retail-12-0-5\")]\nfn register_c_class_talents_action_fns(\n    state: &mut LuaState,\n    table_ref: rilua::vm::gc::arena::GcRef<rilua::vm::table::Table>,\n) -> LuaResult<()> {\n    crate::c_api::class_talent_commands::register(state, table_ref)\n}\n\n#[cfg(not(feature = \"retail-12-0-5\"))]\nfn register_c_class_talents_action_fns(\n"
  },
  {
    "path": "tests/admin_spec_talent_api.rs",
    "classification": "planned maintenance: config-mapping fixture",
    "oldText": "            C_ClassTalents.SwitchToLoadoutByIndex(2)",
    "newText": "            C_ClassTalents.LoadConfig(C_ClassTalents.GetConfigIDsBySpecID(66)[2], true)"
  },
  {
    "path": "tests/hero_talents.rs",
    "classification": "planned maintenance: test_non_selectable_hero_nodes_do_not_show_selectable_glow",
    "oldText": "fn test_non_selectable_hero_nodes_do_not_show_selectable_glow() {\n    let env = env_with_full_ui();\n    let result: String = env\n        .eval(\n            r#\"\n            C_ClassTalents.SwitchToSpecializationByName(\"Protection\")",
    "newText": "fn test_non_selectable_hero_nodes_do_not_show_selectable_glow() {\n    let env = env_with_full_ui();\n    {\n        let mut state = env.state().borrow_mut();\n        state.player.active_spec_index = 2;\n        state.talents.switch_to_spec(66);\n    }\n    let result: String = env\n        .eval(\n            r#\""
  },
  {
    "path": "tests/hero_talents/rendering.rs",
    "classification": "planned maintenance: test_class_talent_edges_render_below_visible_talent_buttons",
    "oldText": "fn test_class_talent_edges_render_below_visible_talent_buttons() {\n    let env = env_with_full_ui();\n    let result: String = env\n        .eval(\n            r#\"\n            C_ClassTalents.SwitchToSpecializationByName(\"Protection\")",
    "newText": "fn test_class_talent_edges_render_below_visible_talent_buttons() {\n    let env = env_with_full_ui();\n    {\n        let mut state = env.state().borrow_mut();\n        state.player.active_spec_index = 2;\n        state.talents.switch_to_spec(66);\n    }\n    let result: String = env\n        .eval(\n            r#\""
  },
  {
    "path": "tests/hero_talents/rendering.rs",
    "classification": "planned maintenance: test_button_frame_level_change_relevels_connected_edges_on_update",
    "oldText": "fn test_button_frame_level_change_relevels_connected_edges_on_update() {\n    let env = env_with_full_ui();\n    let result: String = env\n        .eval(\n            r#\"\n            C_ClassTalents.SwitchToSpecializationByName(\"Protection\")",
    "newText": "fn test_button_frame_level_change_relevels_connected_edges_on_update() {\n    let env = env_with_full_ui();\n    {\n        let mut state = env.state().borrow_mut();\n        state.player.active_spec_index = 2;\n        state.talents.switch_to_spec(66);\n    }\n    let result: String = env\n        .eval(\n            r#\""
  },
  {
    "path": "tests/hero_talents/rendering.rs",
    "classification": "planned maintenance: test_hero_spec_content_spec_image_anchors_to_spec_name",
    "oldText": "fn test_hero_spec_content_spec_image_anchors_to_spec_name() {\n    // Regression test for xml_layer_batch two-pass ordering bug.\n    // Before fix (commit 1cae5342), xml_layer_batch collected all textures first\n    // then appended fontstrings, so SpecImage's SetPoint ran before SpecName FontString\n    // existed, causing parent[\"SpecName\"] to be nil and anchoring to the parent frame.\n    let env = env_with_full_ui();\n    let result: String = env\n        .eval(\n            r#\"\n            C_ClassTalents.SwitchToSpecializationByName(\"Protection\")",
    "newText": "fn test_hero_spec_content_spec_image_anchors_to_spec_name() {\n    // Regression test for xml_layer_batch two-pass ordering bug.\n    // Before fix (commit 1cae5342), xml_layer_batch collected all textures first\n    // then appended fontstrings, so SpecImage's SetPoint ran before SpecName FontString\n    // existed, causing parent[\"SpecName\"] to be nil and anchoring to the parent frame.\n    let env = env_with_full_ui();\n    {\n        let mut state = env.state().borrow_mut();\n        state.player.active_spec_index = 2;\n        state.talents.switch_to_spec(66);\n    }\n    let result: String = env\n        .eval(\n            r#\""
  }
]
```

### Slice 4: nav-rekey

```json
[
  {
    "path": "src/c_api/mod.rs",
    "classification": "state",
    "oldText": "pub mod bag_info;",
    "newText": "#[cfg(feature = \"retail-12-0-5\")]\npub mod aura_entry_ids;\npub mod bag_info;"
  },
  {
    "path": "src/lua_api/state/sim_state.rs",
    "classification": "state",
    "oldText": "    pub player: PlayerState,",
    "newText": "    /// Explicit host-selected nearest party token; no roster/distance inference.\n    #[cfg(feature = \"retail-12-0-5\")]\n    pub nearest_party_member_token: Option<String>,\n    /// Host-provided identifiers for the next encounter/M+/PvP entry.\n    #[cfg(feature = \"retail-12-0-5\")]\n    pub aura_entry_ids: crate::c_api::aura_entry_ids::AuraEntryIds,\n    pub player: PlayerState,"
  },
  {
    "path": "src/lua_api/state.rs",
    "classification": "state",
    "oldText": "            player: PlayerState::seeded(),",
    "newText": "            #[cfg(feature = \"retail-12-0-5\")]\n            nearest_party_member_token: None,\n            #[cfg(feature = \"retail-12-0-5\")]\n            aura_entry_ids: crate::c_api::aura_entry_ids::AuraEntryIds::default(),\n            player: PlayerState::seeded(),"
  },
  {
    "path": "src/c_api/mod.rs",
    "classification": "producer",
    "oldText": "pub mod c_map;",
    "newText": "pub mod c_map;\n#[cfg(feature = \"retail-12-0-5\")]\npub(crate) mod c_navigation;\n#[cfg(feature = \"retail-12-0-5\")]\npub(crate) mod aura_entry;"
  },
  {
    "path": "src/c_api/mod.rs",
    "classification": "producer",
    "oldText": "    c_chat_info::register(state)?;",
    "newText": "    c_chat_info::register(state)?;\n    #[cfg(feature = \"retail-12-0-5\")]\n    c_navigation::register(state)?;"
  },
  {
    "path": "src/lua_api/env_events.rs",
    "classification": "producer",
    "oldText": "    pub fn fire_event_with_args(&self, event: &str, args: &[Val]) -> Result<()> {\n        #[cfg(feature = \"retail-12-0-0\")]",
    "newText": "    pub fn fire_event_with_args(&self, event: &str, args: &[Val]) -> Result<()> {\n        #[cfg(feature = \"retail-12-0-5\")]\n        crate::c_api::aura_entry::apply_entry_event(self.lua.borrow_mut().state_mut(), event)?;\n        #[cfg(feature = \"retail-12-0-0\")]"
  },
  {
    "path": "src/lua_api/globals/state_backed_queries.rs",
    "classification": "producer",
    "oldText": "    #[cfg(feature = \"retail-12-0-0\")]\n    super::real::event_callbacks::dispatch_event_callbacks(state, event_name, args)?;",
    "newText": "    #[cfg(feature = \"retail-12-0-5\")]\n    crate::c_api::aura_entry::apply_entry_event(state, event_name)?;\n    #[cfg(feature = \"retail-12-0-0\")]\n    super::real::event_callbacks::dispatch_event_callbacks(state, event_name, args)?;"
  },
  {
    "path": "src/lua_api/loader_env.rs",
    "classification": "producer",
    "oldText": "        let listeners = self.with_state(|state| {\n            Ok::<Vec<u64>, crate::Error>",
    "newText": "        let listeners = self.with_state(|state| {\n            #[cfg(feature = \"retail-12-0-5\")]\n            crate::c_api::aura_entry::apply_entry_event(state, event)?;\n            Ok::<Vec<u64>, crate::Error>"
  },
  {
    "path": "src/lua_api/workarounds/temporary/navigation_defaults.rs",
    "classification": "producer",
    "oldText": "installNavigationDefault(\"GetNearestPartyMemberToken\", function()\n    return nil\nend)\n\n",
    "newText": ""
  },
  {
    "path": "src/lua_api/workarounds/temporary/navigation_defaults.rs",
    "classification": "state",
    "oldText": "let result: (bool, i32, bool, i32, bool, bool)",
    "newText": "let result: (bool, i32, bool, i32, bool)"
  },
  {
    "path": "src/lua_api/workarounds/temporary/navigation_defaults.rs",
    "classification": "state",
    "oldText": "                    C_Navigation.GetNearestPartyMemberToken() == nil,\n",
    "newText": ""
  },
  {
    "path": "src/lua_api/workarounds/temporary/navigation_defaults.rs",
    "classification": "state",
    "oldText": "assert_eq!(result, (false, 0, false, 0, true, true));",
    "newText": "assert_eq!(result, (false, 0, false, 0, true));"
  },
  {
    "path": "tests/c_navigation_probes.rs",
    "classification": "state",
    "oldText": "        nearest_token_is_nil,\n",
    "newText": ""
  },
  {
    "path": "tests/c_navigation_probes.rs",
    "classification": "state",
    "oldText": "): (bool, i32, bool, i32, bool, bool) = env",
    "newText": "): (bool, i32, bool, i32, bool) = env"
  },
  {
    "path": "tests/c_navigation_probes.rs",
    "classification": "state",
    "oldText": "                C_Navigation.GetNearestPartyMemberToken() == nil,\n",
    "newText": ""
  },
  {
    "path": "tests/c_navigation_probes.rs",
    "classification": "state",
    "oldText": "    assert!(nearest_token_is_nil);\n",
    "newText": ""
  }
]
```

### Slice 5: housing-bundle

```json
[
  {
    "path": "src/c_api/mod.rs",
    "classification": "### [state/producer] `src/c_api/mod.rs`",
    "oldText": "pub mod c_catalog_shop_products;\n",
    "newText": "pub mod c_catalog_shop_products;\npub mod c_housing_bundles;\n"
  },
  {
    "path": "src/lua_api/state/sim_state.rs",
    "classification": "### [state] `src/lua_api/state/sim_state.rs`",
    "oldText": "    pub catalog_shop_products: crate::c_api::c_catalog_shop_products::CatalogShopProducts,\n",
    "newText": "    pub catalog_shop_products: crate::c_api::c_catalog_shop_products::CatalogShopProducts,\n    /// One bundle record store; preserves the existing simulator storefront seed.\n    pub housing_bundles: crate::c_api::c_housing_bundles::HousingBundles,\n"
  },
  {
    "path": "src/lua_api/state.rs",
    "classification": "### [state] `src/lua_api/state.rs`",
    "oldText": "            catalog_shop_products: Default::default(),\n",
    "newText": "            catalog_shop_products: Default::default(),\n            housing_bundles: Default::default(),\n"
  },
  {
    "path": "src/lua_api/globals/missing_surface.rs",
    "classification": "### [producer] `src/lua_api/globals/missing_surface.rs`",
    "oldText": "    c_api::c_housing::register_c_housing_surface(state)?;\n",
    "newText": "    c_api::c_housing::register_c_housing_surface(state)?;\n    c_api::c_housing_bundles::register_c_housing_bundles(state)?;\n"
  },
  {
    "path": "src/lua_api/workarounds/temporary/housing_catalog_state.lua",
    "classification": "### [state] `src/lua_api/workarounds/temporary/housing_catalog_state.lua`",
    "oldText": "local __wow_housing_seeded_bundle_state = {\n  [5001] = {\n    productID = 5001,\n    price = 500,\n    originalPrice = nil,\n    entryIDs = { 1001, 1002 },\n    decorEntries = {\n      { decorID = 1001, quantity = 1 },\n      { decorID = 1002, quantity = 1 },\n    },\n    nonDecorProducts = {},\n    canPreview = true,\n    wasViewed = false,\n  },\n}\n",
    "newText": ""
  },
  {
    "path": "src/lua_api/workarounds/temporary/housing_catalog_state.lua",
    "classification": "### [producer] `src/lua_api/workarounds/temporary/housing_catalog_state.lua`",
    "oldText": "local function __wow_housing_copy_bundle_info(bundle_product_id)\n  local bundle = __wow_housing_seeded_bundle_state[bundle_product_id]\n  if not bundle then\n    return nil\n  end\n  local info = __wow_housing_clone_table(bundle)\n  info.entryIDs = __wow_housing_clone_table(bundle.entryIDs)\n  info.decorEntries = {}\n  for index, decor_entry in ipairs(bundle.decorEntries) do\n    info.decorEntries[index] = __wow_housing_clone_table(decor_entry)\n  end\n  info.nonDecorProducts = __wow_housing_clone_table(bundle.nonDecorProducts)\n  return info\nend\n",
    "newText": ""
  },
  {
    "path": "src/lua_api/workarounds/temporary/housing_catalog_state.lua",
    "classification": "### [producer] `src/lua_api/workarounds/temporary/housing_catalog_state.lua`",
    "oldText": "  GetBundleInfo = function(bundleCatalogShopProductID)\n    return __wow_housing_copy_bundle_info(bundleCatalogShopProductID)\n  end,\n",
    "newText": ""
  },
  {
    "path": "src/lua_api/workarounds/temporary/housing_catalog_state.lua",
    "classification": "### [producer] `src/lua_api/workarounds/temporary/housing_catalog_state.lua`",
    "oldText": "  GetFeaturedBundles = function()\n    local featured = { __wow_housing_copy_bundle_info(5001) }\n    return featured\n  end,\n",
    "newText": ""
  },
  {
    "path": "src/lua_api/workarounds/temporary/housing_catalog_state.lua",
    "classification": "### [producer] `src/lua_api/workarounds/temporary/housing_catalog_state.lua`",
    "oldText": "  HousingMarketActionViewBundle = function(bundleProductID)\n    local bundle = __wow_housing_seeded_bundle_state[bundleProductID]\n    if not bundle then\n      return false\n    end\n    bundle.wasViewed = true\n    return true\n  end,\n",
    "newText": ""
  }
]
```

### Slice 6: partial-104-114

```json
[
  {
    "path": "src/c_api/cooldown_duration.rs",
    "classification": "producer",
    "oldText": "pub(crate) fn read_ignore_gcd(state: &LuaState, index: i32) -> bool {\n    cfg!(feature = \"retail-12-0-5\") && matches!(stack_val(state, index), Val::Bool(true))\n}\n",
    "newText": "pub(crate) fn read_ignore_gcd(state: &LuaState, index: i32) -> bool {\n    cfg!(feature = \"retail-12-0-5\") && matches!(stack_val(state, index), Val::Bool(true))\n}\n\n/// Action/book flags use the cached AllowedWhenUntainted argument policy.\npub(crate) fn read_untainted_ignore_gcd(state: &LuaState, index: i32) -> rilua::LuaResult<bool> {\n    if !cfg!(feature = \"retail-12-0-5\") {\n        return Ok(false);\n    }\n    let input = rilua::table_security::unwrap_secret(state, stack_val(state, index))?;\n    // Inferred compatibility: retain the existing true-only rule for public values.\n    Ok(matches!(input, Val::Bool(true)))\n}\n"
  },
  {
    "path": "src/c_api/c_action_bar.rs",
    "classification": "producer",
    "oldText": "use super::cooldown_duration::{read_ignore_gcd, select_cooldown_duration_times};",
    "newText": "use super::cooldown_duration::{read_untainted_ignore_gcd, select_cooldown_duration_times};"
  },
  {
    "path": "src/c_api/c_action_bar.rs",
    "classification": "producer",
    "oldText": "    let ignore_gcd = read_ignore_gcd(state, 2);",
    "newText": "    let ignore_gcd = read_untainted_ignore_gcd(state, 2)?;"
  },
  {
    "path": "src/c_api/c_spell_book.rs",
    "classification": "producer",
    "oldText": "fn c_spell_book_get_spell_book_item_cooldown_duration(state: &mut LuaState) -> LuaResult<u32> {\n    let Some(spell_id) = read_duration_spellbook_entry(state) else {\n        state.push(Val::Nil);\n        return Ok(1);\n    };\n    let ignore_gcd = super::cooldown_duration::read_ignore_gcd(state, 3);\n",
    "newText": "fn c_spell_book_get_spell_book_item_cooldown_duration(state: &mut LuaState) -> LuaResult<u32> {\n    let ignore_gcd = super::cooldown_duration::read_untainted_ignore_gcd(state, 3)?;\n    let Some(spell_id) = read_duration_spellbook_entry(state) else {\n        state.push(Val::Nil);\n        return Ok(1);\n    };\n"
  },
  {
    "path": "docs/specs/unit-identity-equality.md",
    "classification": "state/spec",
    "oldText": "Adding pet/vehicle/raid/remote identities, changing `UnitExists` or GUID generation, token normalization, general coercion/error redesign, other secret-value API rules and all-profile/native parity. Tokens without a modeled GUID do not compare equal, even where another compatibility query reports presence; this correction does not invent identities for those unsupported domains.\n",
    "newText": "Adding pet/vehicle/raid/remote identities, changing `UnitExists` or GUID generation, token normalization, general coercion/error redesign, other secret-value API rules and all-profile/native parity. Tokens without a modeled GUID do not compare equal, even where another compatibility query reports presence; this correction does not invent identities for those unsupported domains.\n\n## Authored follow-up for prose 2026-03-25-104 (not executed)\n\n- [ ] `tests/retail_12_0_5_partial_104_114.rs::unit_permissions_compare_distinct_guids_despite_identical_names`: existing target/focus snapshots with identical names and distinct GUIDs compare false symmetrically; public FocusUnit restores same-GUID equality. No additional token identities or permission rules.\n\nThe introductory March 25 row remains partial, consistent with March 31 prose141. Existing base/group/residual matrices are bounded, not native/all-token/all-profile proof. Canonical token bounds, lexical compound/nameplate classification, missing/nil behavior and zero-return arity remain inferred simulator policies. March 25 prose088's earlier secret-result proposal is superseded by prose104–107 / March31 prose141–144; do not reinstate it.\n"
  },
  {
    "path": "docs/specs/spellbook-cooldown-duration.md",
    "classification": "state/spec",
    "oldText": "- Pet book entries, macro spell resolution, dynamic spellbook redesign, numeric spellbook-cooldown modernization, charges/loss-of-control, secrecy/security, vendor behavior, and non-default rates.\n",
    "newText": "- Pet book entries, macro spell resolution, dynamic spellbook redesign, numeric spellbook-cooldown modernization, charges/loss-of-control, secrecy/security, vendor behavior, and non-default rates.\n\n## Authored follow-up for prose 2026-03-25-114 (not executed)\n\n- [ ] Query-selected objects from action/spell/spellbook drive actual Cooldown widgets; removing the individual interval clears new ignoreGCD=true objects, while false retains GCD and old true snapshots remain usable.\n- [ ] On retail12.0.5+, action/book ignoreGCD uses VM authentication per cached AllowedWhenUntainted: secure secret booleans select the same intervals as plain booleans; tainted secret flags error without clearing caller taint.\n- [ ] Authenticate book ignoreGCD before invalid-entry nil return; secure invalid-entry queries still return nil.\n\nTests: `tests/retail_12_0_5_partial_104_114.rs`. INFERRED: retain the prior literal-true-only public conversion policy and earlier-epoch argument-ignore policy; this is not native type/coercion validation. Interval selection remains the existing inferred later-end/individual-only model. No additional SimState fields.\n\nScope is only the new flag for action/book and ordinary widget consumption for all three APIs. Action-slot and book slot/bank secret authentication remain unmodeled by these duration producers. Spell's distinct cached AllowedWhenTainted policy is unresolved and deliberately unchanged: applying the untainted-only helper there would misrepresent its contract. Restricted-output secrecy, pet book entries, native behavior, real Blizzard consumers, earlier-epoch proof and independent acceptance remain open. Row114 stays partial; these authored additions carry no execution credit.\n"
  }
]
```

### Slice 7: pdeid-specialbar

```json
[
  {
    "path": "src/lua_api/state/sim_state.rs",
    "classification": "### 1. [state] `src/lua_api/state/sim_state.rs`",
    "oldText": "    pub action_bars: HashMap<u32, u32>,",
    "newText": "    pub action_bars: HashMap<u32, u32>,\n    /// INFERRED host-declared resolved spell IDs on special bars; empty by default.\n    /// Independent of direct assignments and bar visibility.\n    #[cfg(feature = \"retail-12-0-5\")]\n    pub special_bar_spells: HashSet<u32>,"
  },
  {
    "path": "src/lua_api/state/sim_state.rs",
    "classification": "### 2. [state] `src/lua_api/state/sim_state.rs`",
    "oldText": "    pub has_active_delve: bool,",
    "newText": "    pub has_active_delve: bool,\n    /// Explicit host entrance PDEID; INFERRED zero default, not a native sentinel.\n    pub tiered_entrance_pde_id: u32,"
  },
  {
    "path": "src/lua_api/state.rs",
    "classification": "### 3. [state] `src/lua_api/state.rs`",
    "oldText": "            action_bars: $collections.action_bars,",
    "newText": "            action_bars: $collections.action_bars,\n            #[cfg(feature = \"retail-12-0-5\")]\n            special_bar_spells: HashSet::new(),"
  },
  {
    "path": "src/lua_api/state.rs",
    "classification": "### 4. [state] `src/lua_api/state.rs`",
    "oldText": "            has_active_delve: false,",
    "newText": "            has_active_delve: false,\n            tiered_entrance_pde_id: 0,"
  },
  {
    "path": "src/lua_api/globals/missing_surface/delves_ui.rs",
    "classification": "### 5. [producer] `src/lua_api/globals/missing_surface/delves_ui.rs`",
    "oldText": "fn get_tiered_entrance_pde_id(state: &mut LuaState) -> LuaResult<u32> {\n    state.push(Val::Num(77011.0));\n    Ok(1)\n}",
    "newText": "fn get_tiered_entrance_pde_id(state: &mut LuaState) -> LuaResult<u32> {\n    let pde_id = crate::lua_api::methods::borrow_state(state)?.tiered_entrance_pde_id;\n    state.push(Val::Num(f64::from(pde_id)));\n    Ok(1)\n}"
  },
  {
    "path": "src/c_api/c_action_bar_spell_slots.rs",
    "classification": "### 6. [producer] `src/c_api/c_action_bar_spell_slots.rs`",
    "oldText": "fn is_on_bar_or_special_bar(state: &mut LuaState) -> LuaResult<u32> {\n    // Special-bar membership and native AllowedWhenTainted access remain unmodeled.\n    let has_slots = query_public_direct_spell_membership(state, \"C_ActionBar.IsOnBarOrSpecialBar\")?;\n    state.push(Val::Bool(has_slots));\n    Ok(1)\n}",
    "newText": "fn is_on_bar_or_special_bar(state: &mut LuaState) -> LuaResult<u32> {\n    // AllowedWhenTainted is NOT AllowedWhenUntainted: retain the bounded public reader.\n    // Native secret-input permission/result secrecy remain unmodeled.\n    let spell_id = super::c_spell::read_public_spell_identifier_at(\n        state,\n        1,\n        \"C_ActionBar.IsOnBarOrSpecialBar\",\n    )?;\n    let is_on_bar = match spell_id {\n        Some(id) => {\n            let sim = borrow_state(state)?;\n            // INFERRED union of effective direct slots and explicit special membership.\n            effective_spell_slots(&sim, id).next().is_some() || sim.special_bar_spells.contains(&id)\n        }\n        None => false,\n    };\n    state.push(Val::Bool(is_on_bar));\n    Ok(1)\n}"
  },
  {
    "path": "docs/specs/delves-api-inputs.md",
    "classification": "### 7. [documentation] `docs/specs/delves-api-inputs.md`",
    "oldText": "- Row 260 and all other Delves methods: unchanged.",
    "newText": "- Row 260 is specified separately in [Tiered entrance PDEID](tiered-entrance-pdeid.md); all other Delves methods remain outside this input slice."
  },
  {
    "path": "docs/specs/action-bar-membership.md",
    "classification": "### 8. [documentation] `docs/specs/action-bar-membership.md`",
    "oldText": "## What it must do",
    "newText": "The acceptance below records the earlier direct-only slice. [Explicit special-bar membership](action-bar-special-membership.md) specifies the subsequent host-state extension; it does not grant native security or acquisition credit.\n\n## What it must do"
  },
  {
    "path": "docs/specs/action-bar-membership.md",
    "classification": "### 9. [documentation] `docs/specs/action-bar-membership.md`",
    "oldText": "- [ ] Special-bar membership has no defined contract/model in this slice. Direct assignment coverage cannot close row245 or establish special-bar behavior.",
    "newText": "- [ ] Native special-bar acquisition/classification remains undefined. [Explicit special-bar membership](action-bar-special-membership.md) adds an INFERRED host set; direct assignment coverage alone cannot close row245 or establish native special-bar behavior."
  },
  {
    "path": "docs/specs/action-bar-membership.md",
    "classification": "### 10. [documentation] `docs/specs/action-bar-membership.md`",
    "oldText": "- Special bars, active page/visibility, vehicle/possess/pet/stance/bonus/override/temporary-bar membership: no bounded model or native definition supplied.",
    "newText": "- Native special-bar acquisition, active page/visibility, and vehicle/possess/pet/stance/bonus/override/temporary-bar classification: no native definition supplied. Explicit host membership is specified by the companion extension."
  }
]
```

### Slice 8: chat-expressions

```json
[
  {
    "path": "src/c_api/c_chat_info.rs",
    "classification": "",
    "oldText": "//! Explicit Retail chat messaging restriction query and producer guard.\n",
    "newText": "//! Explicit Retail chat messaging restrictions and text expression producer.\n\nmod expressions;\n"
  },
  {
    "path": "src/c_api/c_chat_info.rs",
    "classification": "",
    "oldText": "        in_chat_messaging_lockdown,\n    )\n",
    "newText": "        in_chat_messaging_lockdown,\n    )?;\n    expressions::register(state, namespace)\n"
  },
  {
    "path": "src/lua_api/state/sim_state.rs",
    "classification": "",
    "oldText": "    pub chat_messaging_lockdown: bool,\n",
    "newText": "    pub chat_messaging_lockdown: bool,\n    /// INFERRED chat-expansion roster: None preserves group tags; Some uses\n    /// ordered (subgroup, verbatim name) rows, independent of synthetic party data.\n    #[cfg(feature = \"retail-12-0-5\")]\n    pub chat_expression_roster: Option<Vec<(u8, String)>>,\n"
  },
  {
    "path": "src/lua_api/state.rs",
    "classification": "",
    "oldText": "            chat_messaging_lockdown: false,\n",
    "newText": "            chat_messaging_lockdown: false,\n            #[cfg(feature = \"retail-12-0-5\")]\n            chat_expression_roster: None,\n"
  },
  {
    "path": "src/lua_api/workarounds/temporary/c_chat_info_defaults.rs",
    "classification": "",
    "oldText": "installChatInfoDefault(\"ReplaceIconAndGroupExpressions\", function(text)\n    return text\nend)\n\n",
    "newText": ""
  },
  {
    "path": "src/loader/tests/wow_api_globals/startup_namespaces.rs",
    "classification": "maintenance (handoff, epoch-aware)",
    "oldText": "    assert_eq!(replaced_message, \"{rt1} hello\");",
    "newText": "    #[cfg(feature = \"retail-12-0-5\")]\n    assert_eq!(\n        replaced_message,\n        \"|TInterface\\\\TargetingFrame\\\\UI-RaidTargetingIcon_1:0|t hello\"\n    );\n    #[cfg(not(feature = \"retail-12-0-5\"))]\n    assert_eq!(replaced_message, \"nil\");"
  },
  {
    "path": "src/loader/tests/wow_api_globals/startup_namespaces.rs",
    "classification": "planned maintenance: historical API absence",
    "oldText": "fn test_startup_bootstrap_namespaces_exist() {\n    let env = WowLuaEnv::new().unwrap();",
    "newText": "fn test_startup_bootstrap_namespaces_exist() {\n    let env = WowLuaEnv::new().unwrap();\n    let expression_query = if cfg!(feature = \"retail-12-0-5\") {\n        \"C_ChatInfo.ReplaceIconAndGroupExpressions(\\\"{rt1} hello\\\")\"\n    } else {\n        \"type(C_ChatInfo.ReplaceIconAndGroupExpressions)\"\n    };"
  },
  {
    "path": "src/loader/tests/wow_api_globals/startup_namespaces.rs",
    "classification": "planned maintenance: select observable epoch contract",
    "oldText": "        .eval(\n            r#\"\n            local enabled, pollSeconds, balanceEnabled = C_WowTokenPublic.GetCommerceSystemStatus()\n            return type(C_ChatInfo.PerformEmote),\n                C_ChatInfo.ReplaceIconAndGroupExpressions(\"{rt1} hello\"),\n                C_ChatInfo.AreOutgoingAddonChatMessagesRestricted(),\n                type(C_Navigation.GetDistance),\n                C_Navigation.GetDistance(),\n                type(C_Navigation.GetFrame()),\n                type(C_WowTokenPublic.GetCommerceSystemStatus),\n                enabled,\n                pollSeconds,\n                balanceEnabled\n            \"#,\n        )\n",
    "newText": "        .eval(\n            &r#\"\n            local enabled, pollSeconds, balanceEnabled = C_WowTokenPublic.GetCommerceSystemStatus()\n            return type(C_ChatInfo.PerformEmote),\n                __CHAT_EXPRESSION_QUERY__,\n                C_ChatInfo.AreOutgoingAddonChatMessagesRestricted(),\n                type(C_Navigation.GetDistance),\n                C_Navigation.GetDistance(),\n                type(C_Navigation.GetFrame()),\n                type(C_WowTokenPublic.GetCommerceSystemStatus),\n                enabled,\n                pollSeconds,\n                balanceEnabled\n            \"#\n            .replace(\"__CHAT_EXPRESSION_QUERY__\", expression_query),\n        )\n"
  }
]
```

### Slice 9: fixture-restricted-outfit

```json
[
  {
    "path": "src/c_api/c_housing/exterior.rs",
    "classification": "state",
    "oldText": "    pub selected_size: Option<i32>,",
    "newText": "    /// Host-selected core family; empty unless explicitly supplied.\n    pub core_fixture: Option<ExteriorCoreFixture>,\n    pub selected_size: Option<i32>,"
  },
  {
    "path": "src/c_api/c_housing/exterior.rs",
    "classification": "state",
    "oldText": "#[derive(Clone, Debug, PartialEq)]\npub struct ExteriorSizeOption {",
    "newText": "/// One explicit core selection and its eligible replacements; no native population.\n#[derive(Clone, Debug, PartialEq)]\npub struct ExteriorCoreFixture {\n    pub selected_fixture_id: u32,\n    pub options: Vec<ExteriorCoreFixtureOption>,\n}\n\n#[derive(Clone, Debug, PartialEq)]\npub struct ExteriorCoreFixtureOption {\n    pub fixture_id: u32,\n    /// Attachment parent identity in ExteriorDecorPlacement, supplied by the host.\n    pub owner_hash: u32,\n    /// Shared group means variants of the same style; None never implies equivalence.\n    pub recolor_group: Option<u32>,\n    pub is_locked: bool,\n    pub is_invalid: bool,\n}\n\n#[derive(Clone, Debug, PartialEq)]\npub struct ExteriorSizeOption {"
  },
  {
    "path": "src/c_api/c_housing/exterior/runtime.rs",
    "classification": "producer",
    "oldText": "#[path = \"mutation.rs\"]\nmod mutation;",
    "newText": "#[path = \"core.rs\"]\nmod core;\n#[path = \"mutation.rs\"]\nmod mutation;"
  },
  {
    "path": "src/c_api/c_housing/exterior/runtime.rs",
    "classification": "producer",
    "oldText": "        (\n            \"SelectFixtureOption\",",
    "newText": "        (\n            \"SelectCoreFixtureOption\",\n            select_core_fixture_option as rilua::RustFn,\n        ),\n        (\n            \"SelectFixtureOption\","
  },
  {
    "path": "src/c_api/c_housing/exterior/runtime.rs",
    "classification": "producer",
    "oldText": "fn select_fixture_option(state: &mut LuaState) -> LuaResult<u32> {",
    "newText": "fn select_core_fixture_option(state: &mut LuaState) -> LuaResult<u32> {\n    let (target, action) = read_arguments(state, core::API)?;\n    let (response, stored_variants) = {\n        let mut sim = borrow_state_mut(state)?;\n        core::update_core_fixture(&mut sim.housing, target, action)?\n    };\n    // INFERRED: share the existing fixture-response and storage publication policy.\n    publish_exterior_response(\n        state,\n        ExteriorChange::Fixture.event(),\n        response,\n        stored_variants,\n    )\n}\n\nfn select_fixture_option(state: &mut LuaState) -> LuaResult<u32> {"
  },
  {
    "path": "src/c_api/c_housing/exterior/runtime.rs",
    "classification": "producer",
    "oldText": "    let (target, action) = read_arguments(state, change)?;",
    "newText": "    let (target, action) = read_arguments(state, change.api())?;"
  },
  {
    "path": "src/c_api/c_housing/exterior/runtime.rs",
    "classification": "producer",
    "oldText": "    change: ExteriorChange,\n) -> LuaResult<(u32, AttachedDecorAction)> {\n    // Authenticate BOTH originals before type checks, defaults, or model access.\n    let target = authenticate(state, stack_val(state, 1), change.api(), 1)?;\n    let action = authenticate(state, stack_val(state, 2), change.api(), 2)?;\n    let target = read_target(target, change)?;\n    let action = read_action(action, change.api(), 2)?;",
    "newText": "    api: &str,\n) -> LuaResult<(u32, AttachedDecorAction)> {\n    // Authenticate BOTH originals before type checks, defaults, or model access.\n    let target = authenticate(state, stack_val(state, 1), api, 1)?;\n    let action = authenticate(state, stack_val(state, 2), api, 2)?;\n    let target = read_target(target, api)?;\n    let action = read_action(action, api, 2)?;"
  },
  {
    "path": "src/c_api/c_housing/exterior/runtime.rs",
    "classification": "producer",
    "oldText": "fn read_target(value: Val, change: ExteriorChange) -> LuaResult<u32> {\n    match value {\n        Val::Num(number) if is_positive_integral_u32(number) => Ok(number as u32),\n        _ => Err(runtime_error(format!(\n            \"{}: argument 1 must be a positive integral u32 number\",\n            change.api()\n        ))),",
    "newText": "fn read_target(value: Val, api: &str) -> LuaResult<u32> {\n    match value {\n        Val::Num(number) if is_positive_integral_u32(number) => Ok(number as u32),\n        _ => Err(runtime_error(format!(\n            \"{api}: argument 1 must be a positive integral u32 number\"\n        ))),"
  },
  {
    "path": "src/c_api/c_housing/exterior/mutation.rs",
    "classification": "producer",
    "oldText": "fn update_attachments(",
    "newText": "pub(super) fn update_attachments("
  },
  {
    "path": "src/c_api/c_housing/exterior/mutation.rs",
    "classification": "producer",
    "oldText": "fn validate_host(",
    "newText": "pub(super) fn validate_host("
  },
  {
    "path": "src/c_api/c_housing/exterior/mutation.rs",
    "classification": "producer",
    "oldText": "fn find_affected_placements(",
    "newText": "pub(super) fn find_affected_placements("
  },
  {
    "path": "src/lua_api/workarounds/temporary/housing_catalog_state.lua",
    "classification": "producer",
    "oldText": "  SelectCoreFixtureOption = __wow_noop,\n",
    "newText": ""
  },
  {
    "path": "tests/house_exterior.rs",
    "classification": "state",
    "oldText": "        HouseExteriorState {\n            selected_size: Some(3),",
    "newText": "        HouseExteriorState {\n            core_fixture: None,\n            selected_size: Some(3),"
  }
]
```

### Slice 10: ambiguate

```json
[
  {
    "path": "src/lua_api/globals/real/ambiguate.rs",
    "classification": "### [producer] `src/lua_api/globals/real/ambiguate.rs`",
    "oldText": "//! Deterministic public-name shortening with the 12.0.5 secret-context boundary.\n\npub(crate) fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {\n    let source = format!(\n        r#\"\nlocal rejectSecretContext = {reject_secret_context}\nlocal isSecretContext = issecretvalue\nfunction Ambiguate(fullName, context)\n    if rejectSecretContext and isSecretContext(context) then\n        error(\"Ambiguate argument #2 must not be secret\", 2)\n    end\n    if context == \"none\" then\n        return fullName\n    end\n    return string.match(fullName, \"^(.-)%-.+$\") or fullName\nend\n\"#,\n        reject_secret_context = cfg!(feature = \"retail-12-0-5\"),\n    );\n    lua.exec(&source)?;\n    Ok(())\n}",
    "newText": "//! Ambiguate's existing name mapping with the 12.0.5 secret-input boundary.\n\nuse crate::lua_bridge::stack_val;\nuse rilua::table_security::{is_secret_value, transform_host_secret_string};\nuse rilua::vm::state::LuaState;\nuse rilua::{LuaApiMut, LuaResult, Val, runtime_error};\n\npub(crate) fn register_all(lua: &mut rilua::Lua) -> crate::Result<()> {\n    if cfg!(feature = \"retail-12-0-5\") {\n        LuaApiMut::register_function(lua, \"Ambiguate\", ambiguate)?;\n    } else {\n        // Earlier epochs retain their existing Lua provider, not the new permission.\n        lua.exec(\n            r#\"\nfunction Ambiguate(fullName, context)\n    if context == \"none\" then\n        return fullName\n    end\n    return string.match(fullName, \"^(.-)%-.+$\") or fullName\nend\n\"#,\n        )?;\n    }\n    Ok(())\n}\n\nfn ambiguate(state: &mut LuaState) -> LuaResult<u32> {\n    let full_name = stack_val(state, 1);\n    let context = stack_val(state, 2);\n    if is_secret_value(state, context) {\n        return Err(runtime_error(\"Ambiguate argument #2 must not be secret\"));\n    }\n    let keep_full_name = context_is_none(state, context);\n    let result = if is_secret_value(state, full_name) {\n        transform_secret_name(state, full_name, keep_full_name)?\n    } else {\n        ambiguate_public_name(state, full_name, keep_full_name)?\n    };\n    state.push(result);\n    Ok(1)\n}\n\nfn transform_secret_name(state: &mut LuaState, name: Val, keep_full_name: bool) -> LuaResult<Val> {\n    // INFERRED: secret fullName must carry the declared cstring type, and\n    // secret-derived output remains secret for either caller. The VM helper\n    // authenticates its private payload and wraps the result without changing\n    // caller taint or exposing an unwrapped value to Lua.\n    transform_host_secret_string(state, name, |bytes| {\n        let result = if keep_full_name {\n            bytes\n        } else {\n            shorten_name(bytes)\n        };\n        result.to_vec()\n    })\n    .map_err(|error| {\n        runtime_error(format!(\n            \"Ambiguate argument #1 secret must contain a string: {error}\"\n        ))\n    })\n}\n\nfn ambiguate_public_name(state: &mut LuaState, name: Val, keep_full_name: bool) -> LuaResult<Val> {\n    if keep_full_name {\n        return Ok(name);\n    }\n    let bytes = read_name_bytes(state, name)?;\n    let shortened = shorten_name(&bytes);\n    if shortened.len() == bytes.len() {\n        return Ok(name);\n    }\n    Ok(Val::Str(state.gc.intern_string(shortened)))\n}\n\nfn context_is_none(state: &LuaState, context: Val) -> bool {\n    let Val::Str(reference) = context else {\n        return false;\n    };\n    state\n        .gc\n        .string_arena\n        .get(reference)\n        .is_some_and(|string| string.data() == b\"none\")\n}\n\nfn read_name_bytes(state: &LuaState, value: Val) -> LuaResult<Vec<u8>> {\n    match value {\n        Val::Str(reference) => state\n            .gc\n            .string_arena\n            .get(reference)\n            .map(|string| string.data().to_vec())\n            .ok_or_else(|| runtime_error(\"Ambiguate name string has been collected\")),\n        // Faithful to rilua string.match's existing numeric coercion.\n        Val::Num(_) => Ok(format!(\"{value}\").into_bytes()),\n        _ => Err(runtime_error(\n            \"bad argument #1 to 'string.match' (string expected)\",\n        )),\n    }\n}\n\nfn shorten_name(bytes: &[u8]) -> &[u8] {\n    // Faithful byte-level port of string.match(name, '^(.-)%-.+$') or name:\n    // the first hyphen qualifies only when followed by at least one byte.\n    let prefix_end = bytes.iter().position(|byte| *byte == b'-');\n    match prefix_end.filter(|index| index + 1 < bytes.len()) {\n        Some(index) => &bytes[..index],\n        None => bytes,\n    }\n}"
  },
  {
    "path": "Cargo.toml",
    "classification": "### [producer prerequisite] `Cargo.toml:26`",
    "oldText": "rilua = { git = \"https://github.com/Osso/rilua.git\", rev = \"6044544b960cd68b4b0c58bb3373412757c2caee\" }",
    "newText": "rilua = { git = \"https://github.com/Osso/rilua.git\", rev = \"<NEW_RILUA_REV>\" }"
  },
  {
    "path": "Cargo.lock",
    "classification": "### [producer prerequisite] `Cargo.lock:4994`",
    "oldText": "source = \"git+https://github.com/Osso/rilua.git?rev=6044544b960cd68b4b0c58bb3373412757c2caee#6044544b960cd68b4b0c58bb3373412757c2caee\"",
    "newText": "source = \"git+https://github.com/Osso/rilua.git?rev=<NEW_RILUA_REV>#<NEW_RILUA_REV>\""
  },
  {
    "path": "docs/specs/ambiguate-context.md",
    "classification": "### [state] `docs/specs/ambiguate-context.md:55`",
    "oldText": "- Row 416 `AllowedWhenTainted` remains unmodeled: no secret fullName acceptance, decoding, or argument-1 policy claims.",
    "newText": "- Historical B74 acceptance excludes row 416 `AllowedWhenTainted`: no secret fullName acceptance, decoding, or argument-1 policy credit. The separately authored [secret-name slice](ambiguate-secret-name.md) targets row 416; its unchecked requirements and inferred output policy do not broaden B74 proof."
  }
]
```

### Slice 11: taint-log

```json
[
  {
    "path": "src/lua_api/mod.rs",
    "classification": "state",
    "oldText": "pub(crate) mod talent_state;",
    "newText": "#[cfg(feature = \"retail-12-0-5\")]\npub mod taint_log;\npub(crate) mod talent_state;"
  },
  {
    "path": "src/lua_api/state/sim_state.rs",
    "classification": "state",
    "oldText": "    pub cvars: CVarStorage,",
    "newText": "    pub cvars: std::rc::Rc<CVarStorage>,\n    #[cfg(feature = \"retail-12-0-5\")]\n    pub taint_log: std::rc::Rc<crate::lua_api::taint_log::TaintLog>,"
  },
  {
    "path": "src/lua_api/state.rs",
    "classification": "state",
    "oldText": "    ($collections:ident, $runtime:ident) => {\n        Self {",
    "newText": "    ($collections:ident, $runtime:ident) => {{\n        let cvars = ::std::rc::Rc::new(CVarStorage::new());\n        #[cfg(feature = \"retail-12-0-5\")]\n        let taint_log = ::std::rc::Rc::new(crate::lua_api::taint_log::TaintLog::new(\n            ::std::rc::Rc::clone(&cvars),\n        ));\n        Self {"
  },
  {
    "path": "src/lua_api/state.rs",
    "classification": "state",
    "oldText": "            cvars: CVarStorage::new(),",
    "newText": "            cvars,\n            #[cfg(feature = \"retail-12-0-5\")]\n            taint_log,"
  },
  {
    "path": "src/lua_api/state.rs",
    "classification": "state",
    "oldText": "            simulator_exit_requested: false,\n        }\n    };\n}",
    "newText": "            simulator_exit_requested: false,\n        }\n    }};\n}"
  },
  {
    "path": "src/lua_api/env.rs",
    "classification": "state",
    "oldText": "    pub(crate) sim_state: Rc<RefCell<SimState>>,",
    "newText": "    pub(crate) sim_state: Rc<RefCell<SimState>>,\n    #[cfg(feature = \"retail-12-0-5\")]\n    pub(crate) taint_log: Rc<super::taint_log::TaintLog>,"
  },
  {
    "path": "src/lua_api/env.rs",
    "classification": "state",
    "oldText": "    fn new(sim_state: Rc<RefCell<SimState>>) -> Self {\n        Self {\n            sim_state,",
    "newText": "    pub(crate) fn new(sim_state: Rc<RefCell<SimState>>) -> Self {\n        #[cfg(feature = \"retail-12-0-5\")]\n        let taint_log = Rc::clone(&sim_state.borrow().taint_log);\n        Self {\n            sim_state,\n            #[cfg(feature = \"retail-12-0-5\")]\n            taint_log,"
  },
  {
    "path": "src/lua_api/env.rs",
    "classification": "producer",
    "oldText": "        lua.state_mut().set_app_data(WowLuaAppData::new(state));\n        lua",
    "newText": "        lua.state_mut().set_app_data(WowLuaAppData::new(state));\n        #[cfg(feature = \"retail-12-0-5\")]\n        super::taint_log::install(lua.state_mut());\n        lua"
  },
  {
    "path": "src/lua_api/globals/state_backed_queries.rs",
    "classification": "producer",
    "oldText": "fn reload_ui(state: &mut LuaState) -> LuaResult<u32> {\n    dispatch_event_now(",
    "newText": "fn reload_ui(state: &mut LuaState) -> LuaResult<u32> {\n    // This lifecycle dispatches a reload event; it does not recreate the VM.\n    #[cfg(feature = \"retail-12-0-5\")]\n    crate::lua_api::taint_log::install(state);\n    dispatch_event_now("
  },
  {
    "path": "src/lua_api/env_init/mod.rs",
    "classification": "state / existing-test",
    "oldText": "        lua.state_mut().set_app_data(WowLuaAppData {\n            sim_state: Rc::new(RefCell::new(SimState::default())),\n            lua: None,\n            font_system: None,\n            on_update_cache_dirty: true,\n            hot_literals: None,\n            global_slots: None,\n            addon_modules: None,\n        });",
    "newText": "        lua.state_mut().set_app_data(WowLuaAppData::new(Rc::new(RefCell::new(\n            SimState::default(),\n        ))));"
  },
  {
    "path": "src/loader/addon_modules.rs",
    "classification": "state / existing-test",
    "oldText": "        lua.state_mut().set_app_data(WowLuaAppData {\n            sim_state: Rc::new(RefCell::new(SimState::default())),\n            lua: None,\n            font_system: None,\n            on_update_cache_dirty: true,\n            hot_literals: None,\n            global_slots: None,\n            addon_modules: None,\n        });",
    "newText": "        lua.state_mut().set_app_data(WowLuaAppData::new(Rc::new(RefCell::new(\n            SimState::default(),\n        ))));"
  },
  {
    "path": "Cargo.toml",
    "classification": "producer / dependency",
    "oldText": "rilua = { git = \"https://github.com/Osso/rilua.git\", rev = \"6044544b960cd68b4b0c58bb3373412757c2caee\" }",
    "newText": "rilua = { git = \"https://github.com/Osso/rilua.git\", rev = \"<NEW_RILUA_REV>\" }"
  },
  {
    "path": "Cargo.lock",
    "classification": "producer / dependency",
    "oldText": "source = \"git+https://github.com/Osso/rilua.git?rev=6044544b960cd68b4b0c58bb3373412757c2caee#6044544b960cd68b4b0c58bb3373412757c2caee\"",
    "newText": "source = \"git+https://github.com/Osso/rilua.git?rev=<NEW_RILUA_REV>#<NEW_RILUA_REV>\""
  }
]
```

## Inspected input hashes

Source SHA-256, including current working-tree edits. These pin static applicability, not compile/test correctness. A source change after planning invalidates its hash/anchor evidence.

| Path | SHA-256 |
|---|---|
| `Cargo.lock` | `6133aff6edf9872abfb9330a061da8184d60850e380b97bff055f9c8681f0055` |
| `Cargo.toml` | `6704a600a431bd7e27d8a7aea1c9c3a8ddc20354f3d67f856ed930c83cda1fbd` |
| `docs/specs/action-bar-membership.md` | `6821bd0bc809bc10fa57ae187fa6643f2f595fac200dc024e06b36c174b5c5e0` |
| `docs/specs/ambiguate-context.md` | `219c8fe3d0761fb03388600415e4ca771f816015fe2cc72ce895889dd9cff37c` |
| `docs/specs/delves-api-inputs.md` | `391674b26b70dc0eeb1e7e0fca4a60dec3b2e8e474b08f2303dfb747618a0f17` |
| `docs/specs/private-aura-sound-removal.md` | `92d18aa74f6fdf6c54d0f3682829b8b430d1fedf11c899df6eef6a3d3086b97f` |
| `docs/specs/spellbook-cooldown-duration.md` | `633e7b6169d12790e6c3886347995853704ff00b80a7b9acdc8bbe16608fedb7` |
| `docs/specs/unit-identity-equality.md` | `4e7ae3cd85a43438262cfe3ad737d850c5f6f826d37441791a0906f3ee52284f` |
| `src/c_api/c_action_bar.rs` | `d5596d5f37087c8c71be39e336d2ea8d352ff6238bf231e3b6f800f8b04d9b46` |
| `src/c_api/c_action_bar_spell_slots.rs` | `16073cf8b5788f9d81dbc4499c50e9d197f1bf637c9eb480e71036f82c638374` |
| `src/c_api/c_chat_info.rs` | `8b14868ce255546810dbe1872ecb891c377a85a1ee620d51ba6ad02d5df8f998` |
| `src/c_api/c_housing/exterior.rs` | `db5860715343f90b7a2a68c5fcd25c79c323aa37400103d9db8ec743094409e8` |
| `src/c_api/c_housing/exterior/mutation.rs` | `bb4a8cc016ddc0571a7e69689ef019f9282a5022b30ef632c5e1fc0c944ee814` |
| `src/c_api/c_housing/exterior/runtime.rs` | `64c2f466d07e9de27abb3a027cadbafe863f98d42b2d1eb0ba3d70f819bac407` |
| `src/c_api/c_spell_book.rs` | `e7d10eaadcb540a98144426fb7e0223489cd1fce3e8be9dfbc9544681fd8491c` |
| `src/c_api/cooldown_duration.rs` | `6119652b5a46c6131a48211776ef1b539d75edac0ad35326dcc23f63cef6d894` |
| `src/c_api/mod.rs` | `3fae115ee3219aa20129bcdf0fdf6eaaf281bf205a7855ad0b08672dbf771656` |
| `src/c_api/private_aura_sounds.rs` | `1458cf8291ba2975da298ea91a3e425c43393045469cae1055035f19d96b5f83` |
| `src/loader/addon_modules.rs` | `ee2e90579d4c62d4e51c98bb054381883d0efa7337222ecef58397163f1d42e9` |
| `src/loader/tests/wow_api_globals/startup_namespaces.rs` | `d8e85f71ee7d0b7a1519ce204e9d7ab80b45f46db0bdfb834731af7eabb9b99d` |
| `src/lua_api/env.rs` | `6e0b4a71cd0282400a0b9f351327b71c29e9f2f529620be854655bfd773f8b59` |
| `src/lua_api/env_events.rs` | `47951284430b41aac64c114b020b39c3b86bbf31b417ca7d6a09f3acee82d9ad` |
| `src/lua_api/env_init/mod.rs` | `14040a0997f0e1ab78726af30cd1caf529c98041a668fa682c19ebf82ca78cde` |
| `src/lua_api/globals/group_queries.rs` | `588a235909b5700a81294118c20b071e26709db97b437138c4d19672041cb791` |
| `src/lua_api/globals/missing_surface.rs` | `da3738caa135f88fbe6e93046c26b8c7b492353b42d90057ad35df478b66e269` |
| `src/lua_api/globals/missing_surface/delves_ui.rs` | `ced56cec1b730bd585d5509927c3f14aa94ec0348aa1ccd3ec4a3bbff9895046` |
| `src/lua_api/globals/missing_surface/traits/class_talents.rs` | `e37544ce32a250c56a8a13994a7065eaff9c1653745b3a223d442e11a57836ca` |
| `src/lua_api/globals/real/ambiguate.rs` | `5b4af8e5f00f0d421f83aeb7f0a3b1d50a3d2683ee9b9383085fb3d0c390414c` |
| `src/lua_api/globals/real/mod.rs` | `127d4062ed655357b06a6755ce6e97c42623ff49c0e62a8520e3b8aefd989231` |
| `src/lua_api/globals/state_backed_queries.rs` | `5ebaaaf7028938cd45622a679c444a453a59991ea5ae43d50d8c3b7f7fc3df78` |
| `src/lua_api/globals/unit_probes.rs` | `7c6918670c333a3230f9b02559a37a5a27ae07fdb1934d90242871b2f3f3952d` |
| `src/lua_api/loader_env.rs` | `92b0638108bed656a730c7e1a204e5c86b85d20734ba767612cfb16b39bba037` |
| `src/lua_api/mod.rs` | `5f82acfe877435b8fecab69389da85b37d007ad92a7afe22f000d3a21b67c656` |
| `src/lua_api/state.rs` | `215ab574003e2f44ebc0918d04274cd7c35f0ed94505b7dd8f08dd01301f3de5` |
| `src/lua_api/state/sim_state.rs` | `9cd1de9134a60783c74aa351894de06338ca59886c31bbf14682af2bf1bcb0ee` |
| `src/lua_api/workarounds/temporary/c_chat_info_defaults.rs` | `efe05338a18f6bfa00b6867d4ef338a22a8b42993674888cebab430f2665a4fc` |
| `src/lua_api/workarounds/temporary/housing_catalog_state.lua` | `bbb8fea8bccf02df8d4d1a63d1b96216f56f67e4d09601688559467e01e6a028` |
| `src/lua_api/workarounds/temporary/navigation_defaults.rs` | `b831fe0a77dd378eee939f44e333d759107a1b3a7ec1b1465b00ae24aaad0245` |
| `tests/admin_spec_talent_api.rs` | `b357f08becdfba1e69d37677df5a9199e2dea6a4e20306cf91091c49f60ba2c1` |
| `tests/c_navigation_probes.rs` | `ca28f0e9d7546e79cc598a2942b21b88761abc650f65622740aaa141b2d0edd0` |
| `tests/hero_talents.rs` | `c75e251cb9b9e49b97b34af5b008d319a470251318231011b9b48ec539881abc` |
| `tests/hero_talents/rendering.rs` | `c32d5c7c576eb8a17439c968aac0707ceaf02d13ba32cd1b0a7474feae0dae35` |
| `tests/house_exterior.rs` | `251ccf1856bbc1d031fb3b80e5b8631ea057e9506fd6e8406164d7d25bb4e6ca` |
| `tests/private_aura_sound_removal.rs` | `653c6da07e72c74b77cfd4069087b1fd2472328effff13cd817b91437ff78ef1` |

## Planner proof ledger

| Static inspection | Scope | Result |
|---|---|---|
| Exact-text occurrence and in-memory range application | 104 listed operations over 44 current files; Cargo duplicates coalesced once | Each OLD matches once; no other overlapping ranges |
| New-path existence and ownership inventory | 34 selected new destinations | All absent; none shared by two selected slices |
| JSON appendix parsing | Eleven packets, 104 operations | Round-trip parses; whitespace-preserving exact replacements |
| Final SHA-256/anchor revalidation after concurrent activity | All 44 planned source inputs, 104 OLD texts, 34 new destinations | Same hashes; each OLD still matches once; new paths remain absent at observed HEAD aa29d7d7f06b14f33990c36ae736b98579b00b78 |
| Read-only git status comparison | Initial vs final repository status | Changed concurrently: implementation files became tracked/committed and other documentation/evidence changed; not caused by this planner. No global unchanged/clean-tree claim |
| Build/test/format/acceptance/remote dependency verification | None executed | Not proven; lifecycle and combined published VM revision remain prerequisites |

This is a completed planning artifact, NOT a claim that applying all packets yields a passing build or closes an audit row. Staged code and proposed maintenance still require the owning integrator's execution proof.
