# B99 Phase 1 — state/tests integrated; RED unexecuted

Date: 2026-10-03. Base HEAD: `aa29d7d7f06b14f33990c36ae736b98579b00b78`; changes remain in the working tree.

## Scope and proof

Applied slices in order **6 → 7 → 5 → 9 → 8 → 2 (row 174 only) → 1 → 4 → 3 → 12**, collecting shared-file edits against original bytes and issuing one nonoverlapping multi-edit per existing file. Added **33 files** (11 test modules, 11 source modules, 11 specs); changed **19 existing files** through **41 anchored edits**. New tests contain 78 test annotations, including conditional modern-sound cases.

All **82 selected authored anchors**, eight proposed plan-maintenance anchors, and two deferred combat-expectation anchors matched exactly once. Every remaining exact OLD also matches once after formatting. **No mismatched anchors; no merge-by-intent replacement needed.** Stale whole-file snapshots were never copied over existing repository files.

`cargo fmt` exited **0** on this final Rust tree. Compilation and behavioral RED are **NOT VERIFIED**: no cargo build/check/test was run. Static inspection checked module dependencies, required visibility, exhaustive changed struct fixtures, and the listed EventQueue/private-field/eval/string hazards. No staged-test API corrections were needed. New code contains no warning-suppression attributes; unused producer warnings may remain at this deliberate RED checkpoint.

No git add/commit/stash/checkout, agents, model CLIs, vendor edits, or operational changes. `Cargo.toml`, `Cargo.lock`, `build.rs`, and `tests/integration.rs` are byte-identical to the starting tree. Slices 10/11 and row 172 were not applied. No audit acceptance/accounting files changed.

## Phase 2 application authority

`SCRATCH` = `/tmp/claude-1000/-home-osso-test-Projects-wow-wow-ui-sim/f86e1850-973d-4071-a472-f5757f1d7fdc/scratchpad`.

Each slice below names its exact withheld operations, target file, anchor, and original handoff code location. **All new modules and their declarations already exist**; do not copy existing staged snapshots or reapply declarations. The complete byte-preserving remaining OLD/NEW replacements are additionally in `$SCRATCH/b99-phase2-deferred-edits.json`: 35 producer operations plus 15 deferred expectation/fixture operations, identified by `slice` and `index`. Slice 8 operation 1 is narrowed to its source-header change because `mod expressions` is already installed. Combat expectation edits are separately in the original JSON named below; combat adapters have no supplied patch.

For a Phase 2 file with several replacements, match all OLD blocks against the same current file, reject overlaps, then apply together. `b99-phase1-actions.json` records applied operations. The integration plan remains the authority for epoch scope, loaded-helper completion gaps and shared-state ownership. Expected RED below is prediction, not observed failure evidence.

## [6] Partial prose 104 / 114

**Input:** `$SCRATCH/handoff-partial-104-114.md`; staging `$SCRATCH/staging/partial-104-114/`.
**Handoff producer-code section:** Exact proposed edits.

**Files added:**
- `tests/retail_12_0_5_partial_104_114.rs`

**State / DOC applied:** No source-state changes. Applied DOC follow-ups in `docs/specs/unit-identity-equality.md` and `docs/specs/spellbook-cooldown-duration.md`. Existing GUID and cooldown models remain unchanged.

**WITHHELD producers:**

| Operation | File | OLD anchor / affected code | Handoff code location |
|---|---|---|---|
| 6.1 | `src/c_api/cooldown_duration.rs` | `pub(crate) fn read_ignore_gcd(state: &LuaState, index: i32) -> bool {` | `handoff-partial-104-114.md:125`; exact OLD/NEW, operation 1 |
| 6.2 | `src/c_api/c_action_bar.rs` | `use super::cooldown_duration::{read_ignore_gcd, select_cooldown_duration_times};` | `handoff-partial-104-114.md:151`; exact OLD/NEW, operation 2 |
| 6.3 | `src/c_api/c_action_bar.rs` | `let ignore_gcd = read_ignore_gcd(state, 2);` | `handoff-partial-104-114.md:163`; exact OLD/NEW, operation 3 |
| 6.4 | `src/c_api/c_spell_book.rs` | `fn c_spell_book_get_spell_book_item_cooldown_duration(state: &mut LuaState) -> LuaResult<u32> {` | `handoff-partial-104-114.md:175`; exact OLD/NEW, operation 4 |

**Test filters:** `retail_12_0_5_partial_104_114::`.

**Expected RED:** Three predicted failures: `ignore_gcd_secure_secret_flags_select_action_and_book_intervals` returns GCD start 30 instead of individual start 12; `ignore_gcd_tainted_secret_flags_are_denied_without_losing_taint` observes successful pcall; `ignore_gcd_book_authenticates_flag_before_missing_entry_return` returns nil before authentication. The same-name GUID and duration-to-widget tests are expected GREEN controls, not guaranteed RED.

**Existing tests / deferred maintenance:** None.

## [7] PDEID / special-bar membership

**Input:** `$SCRATCH/handoff-pdeid-specialbar.md`; staging `$SCRATCH/staging/pdeid-specialbar/`.
**Handoff producer-code section:** Exact replacements.

**Files added:**
- `tests/pdeid_specialbar.rs`
- `docs/specs/action-bar-special-membership.md`
- `docs/specs/tiered-entrance-pdeid.md`

**State / DOC applied:** Added `SimState.special_bar_spells: HashSet<u32>` with empty default under retail-12-0-5; unconditional `tiered_entrance_pde_id: u32` default 0. Applied DOC edits to `docs/specs/delves-api-inputs.md` and all three edits to `docs/specs/action-bar-membership.md`.

**WITHHELD producers:**

| Operation | File | OLD anchor / affected code | Handoff code location |
|---|---|---|---|
| 7.5 | `src/lua_api/globals/missing_surface/delves_ui.rs` | `fn get_tiered_entrance_pde_id(state: &mut LuaState) -> LuaResult<u32> {` | `handoff-pdeid-specialbar.md:118`; exact OLD/NEW, operation 5 |
| 7.6 | `src/c_api/c_action_bar_spell_slots.rs` | `fn is_on_bar_or_special_bar(state: &mut LuaState) -> LuaResult<u32> {` | `handoff-pdeid-specialbar.md:137`; exact OLD/NEW, operation 6 |

**Test filters:** `pdeid_specialbar::`.

**Expected RED:** Eight predicted failures: PDEID default/live/isolation cases still receive 77011; special-only, alias, direct/macro/outfit-shadow, read-only/isolation and public-recovery cases still see direct-only membership.

**Existing tests / deferred maintenance:** None; existing direct-slot, outfit and Delves expectations retained.

## [5] Housing bundles

**Input:** `$SCRATCH/handoff-housing-bundle.md`; staging `$SCRATCH/staging/housing-bundle/`.
**Handoff producer-code section:** Exact existing-file edits.

**Files added:**
- `tests/housing_bundle_structures.rs`
- `src/c_api/c_housing_bundles.rs`
- `docs/specs/housing-bundle-structures.md`

**State / DOC applied:** Added `c_housing_bundles` module and `SimState.housing_bundles` defaulting to typed seed 5001. Applied the handoff's explicitly STATE-tagged deletion of `local __wow_housing_seeded_bundle_state`; retained every producer-tagged Lua helper/publisher and withheld Rust registration.

**WITHHELD producers:**

| Operation | File | OLD anchor / affected code | Handoff code location |
|---|---|---|---|
| 5.4 | `src/lua_api/globals/missing_surface.rs` | `c_api::c_housing::register_c_housing_surface(state)?;` | `handoff-housing-bundle.md:82`; exact OLD/NEW, operation 4 |
| 5.6 | `src/lua_api/workarounds/temporary/housing_catalog_state.lua` | `local function __wow_housing_copy_bundle_info(bundle_product_id)` | `handoff-housing-bundle.md:121`; exact OLD/NEW, operation 6 |
| 5.7 | `src/lua_api/workarounds/temporary/housing_catalog_state.lua` | `GetBundleInfo = function(bundleCatalogShopProductID)` | `handoff-housing-bundle.md:144`; exact OLD/NEW, operation 7 |
| 5.8 | `src/lua_api/workarounds/temporary/housing_catalog_state.lua` | `GetFeaturedBundles = function()` | `handoff-housing-bundle.md:156`; exact OLD/NEW, operation 8 |
| 5.9 | `src/lua_api/workarounds/temporary/housing_catalog_state.lua` | `HousingMarketActionViewBundle = function(bundleProductID)` | `handoff-housing-bundle.md:169`; exact OLD/NEW, operation 9 |

**Test filters:** `housing_bundle_structures::`.

**Expected RED:** Six predicted failures. IMPORTANT: unlike the handoff's all-Lua-deletions checkpoint, Phase 1 retains old Lua publishers. Their references to the removed seed now index nil when called; Rust bundle callbacks remain unregistered. These are behavioral failures, not proof of the new serializer. Existing `housing_catalog::housing_catalog_storefront_and_market_methods_use_seeded_state` is also expected to fail temporarily at bundle lookup; its expectations were not changed.

**Existing tests / deferred maintenance:** No expectation changes authored. Preserve `tests/housing_catalog.rs::housing_catalog_storefront_and_market_methods_use_seeded_state` for Phase 2 restoration.

## [9] Core fixture / restricted outfit helper

**Input:** `$SCRATCH/handoff-fixture-restricted-outfit.md`; staging `$SCRATCH/staging/fixture-restricted-outfit/`.
**Handoff producer-code section:** Exact current-tree edits (Edit N).

**Files added:**
- `tests/house_exterior_core_fixture.rs`
- `tests/restricted_outfit_index.rs`
- `src/c_api/c_housing/exterior/core.rs`
- `docs/specs/restricted-outfit-index.md`
- `docs/specs/house-exterior-core-fixture.md`

**State / DOC applied:** Added optional `HouseExteriorState.core_fixture` plus `ExteriorCoreFixture` and `ExteriorCoreFixtureOption`; derived empty default remains None. Added runtime `core` module. Exposed only `mutation::{update_attachments,validate_host,find_affected_placements}` as pub(super), required by the new module. Added `core_fixture: None` to `tests/house_exterior.rs::modern_fixture::exterior_fixture`; no assertions changed. Existing outfit catalog/actions remain untouched.

**WITHHELD producers:**

| Operation | File | OLD anchor / affected code | Handoff code location |
|---|---|---|---|
| 9.4 | `src/c_api/c_housing/exterior/runtime.rs` | `"SelectFixtureOption",` registration tuple | `handoff-fixture-restricted-outfit.md:181`; exact OLD/NEW, operation 4 |
| 9.5 | `src/c_api/c_housing/exterior/runtime.rs` | `fn select_fixture_option(state: &mut LuaState) -> LuaResult<u32> {` | `handoff-fixture-restricted-outfit.md:201`; exact OLD/NEW, operation 5 |
| 9.6 | `src/c_api/c_housing/exterior/runtime.rs` | `let (target, action) = read_arguments(state, change)?;` | `handoff-fixture-restricted-outfit.md:230`; exact OLD/NEW, operation 6 |
| 9.7 | `src/c_api/c_housing/exterior/runtime.rs` | `read_arguments: change: ExteriorChange parameter/authentication block` | `handoff-fixture-restricted-outfit.md:244`; exact OLD/NEW, operation 7 |
| 9.8 | `src/c_api/c_housing/exterior/runtime.rs` | `fn read_target(value: Val, change: ExteriorChange) -> LuaResult<u32>` | `handoff-fixture-restricted-outfit.md:270`; exact OLD/NEW, operation 8 |
| 9.12 | `src/lua_api/workarounds/temporary/housing_catalog_state.lua` | `SelectCoreFixtureOption = __wow_noop,` | `handoff-fixture-restricted-outfit.md:337`; exact OLD/NEW, operation 12 |

**Test filters:** `house_exterior_core_fixture::`, `restricted_outfit_index::`.

**Expected RED:** Ten core cases predicted RED: old SelectCoreFixtureOption no-op neither commits selection/Store/Detach/recolor nor validates inputs or publishes responses; same-selection case also expects a response. Four actual cached restricted-helper cases may already pass; no missing outfit producer was established. Cache/load failures are separate fixture blockers.

**Existing tests / deferred maintenance:** None. Only the exhaustive `modern_fixture::exterior_fixture` struct construction changed.

## [8] Chat expressions

**Input:** `$SCRATCH/handoff-chat-expressions.md`; staging `$SCRATCH/staging/chat-expressions/`.
**Handoff producer-code section:** Exact edits against CURRENT HEAD.

**Files added:**
- `tests/chat_expressions.rs`
- `src/c_api/c_chat_info/expressions.rs`
- `docs/specs/chat-expressions.md`

**State / DOC applied:** Added retail-12-0-5 `chat_expression_roster: Option<Vec<(u8,String)>>`, default None, and `mod expressions`. Kept original source header, existing register return, and identity Lua publisher. The declaration was split from the handoff's combined producer/header edit.

**WITHHELD producers:**

| Operation | File | OLD anchor / affected code | Handoff code location |
|---|---|---|---|
| 8.1 | `src/c_api/c_chat_info.rs` | `//! Explicit Retail chat messaging restriction query and producer guard. (header only; module installed)` | `handoff-chat-expressions.md:49`; exact OLD/NEW, operation 1 |
| 8.2 | `src/c_api/c_chat_info.rs` | `in_chat_messaging_lockdown,` | `handoff-chat-expressions.md:60`; exact OLD/NEW, operation 2 |
| 8.5 | `src/lua_api/workarounds/temporary/c_chat_info_defaults.rs` | `installChatInfoDefault("ReplaceIconAndGroupExpressions", function(text)` | `handoff-chat-expressions.md:102`; exact OLD/NEW, operation 5 |

**Test filters:** `chat_expressions::`.

**Expected RED:** Nine predicted failures: identity publisher does not expand icon/group text or respect disabling flags, ignores strict argument validation/NeverSecret authentication, and does not transform secret text. Explicitly empty roster and None remain distinct host inputs.

**Existing tests / deferred maintenance:** Deferred `src/loader/tests/wow_api_globals/startup_namespaces.rs::test_startup_bootstrap_namespaces_exist` (all three epoch-aware plan edits). Older-epoch `tests/c_chat_info_probes.rs::chat_info_temporary_defaults_are_available` and `src/lua_api/workarounds/temporary/c_chat_info_defaults.rs::tests::installs_chat_info_no_state_defaults` need provider-scope review after publisher deletion; `preserves_existing_chat_info_provider` remains unchanged.

## [2] Row 174 follower display only

**Input:** `$SCRATCH/handoff-editmode-nameplate.md`; staging `$SCRATCH/staging/editmode-nameplate/`.
**Handoff producer-code section:** Exact existing-file edits (PRODUCER group_queries).

**Files added:**
- `tests/follower_nameplate_display.rs`
- `src/lua_api/globals/real/nameplate_display.rs`
- `docs/specs/follower-nameplate-display.md`

**State / DOC applied:** Added retail-12-0-5 `npc_follower_guids` with empty HashSet default and `real::nameplate_display` module. Made existing `unit_probes::resolve_unit_is_player` pub(crate), visibility only, required by the new module. No row-172 secure-delegate work, identity/secrecy model edits, or live display wiring applied.

**WITHHELD producers:**

| Operation | File | OLD anchor / affected code | Handoff code location |
|---|---|---|---|
| 2.5 | `src/lua_api/globals/group_queries.rs` | `fn unit_treat_as_player_for_display(state: &mut LuaState) -> LuaResult<u32> {` | `handoff-editmode-nameplate.md:111`; exact OLD/NEW, operation 5 |

**Test filters:** `follower_nameplate_display::`.

**Expected RED:** Seven predicted failures: marked NPCs remain false; GUID/isolation/friendliness/taint/secure-secret cases lack follower-aware results; old pet/vehicle blanket true and permissive public-input conversion violate the final missing-unit/type case.

**Existing tests / deferred maintenance:** None identified. Phase 2 deliberately also changes pet/vehicle/type behavior. The authored resolver has inactive-retained-party and raid-alias differences noted in the plan; no extra token-policy fix was invented.

## [1] Private aura sound registration state

**Input:** `$SCRATCH/handoff-private-aura-sound.md`; staging `$SCRATCH/staging/private-aura-sound/`.
**Handoff producer-code section:** Exact existing-file edits (numbered PRODUCER sections).

**Files added:**
- `tests/private_aura_sound_add_context.rs`
- `src/c_api/private_aura_sounds/add.rs`
- `src/c_api/private_aura_sounds/inputs.rs`
- `docs/specs/private-aura-sound-add-context.md`

**State / DOC applied:** Moved registration state into new `inputs` module and re-exported `AuraSoundRegistration` and `PrivateAuraSoundRegistrations`. New state includes owned payload map, pvp_match_active=false and next_id=Some(1); live_ids remains public. Added both `inputs` and `add` declarations, without registering callbacks. Added `..Default::default()` to `tests/private_aura_sound_removal.rs::host_replacement_changes_the_current_removal_set`. Applied DOC edit to `docs/specs/private-aura-sound-removal.md`.

**WITHHELD producers:**

| Operation | File | OLD anchor / affected code | Handoff code location |
|---|---|---|---|
| 1.4 | `src/c_api/private_aura_sounds.rs` | `// The cached 12.1 deprecated chunk aliases legacy to modern after bootstrap.` | `handoff-private-aura-sound.md:79`; exact OLD/NEW, operation 4 |
| 1.5 | `src/c_api/private_aura_sounds.rs` | `borrow_state_mut(state)?` existing live-ID removal chain | `handoff-private-aura-sound.md:99`; exact OLD/NEW, operation 5 |
| 1.8 | `src/c_api/private_aura_sounds.rs` | `//! INFERRED host-declared sound registrations; native acquisition and playback unknown.` | `handoff-private-aura-sound.md:142`; exact OLD/NEW, operation 8 |

**Test filters:** `private_aura_sound_add_context::`.

**Expected RED:** Six common / eight with retail-12-1-0 predicted failures: Add cannot allocate/store payloads or complete recovery/lifecycle; secure and secret-input positive paths fail; modern/deprecated alias cases lack modern registration. Missing Add must not be counted as successful denial validation. Existing removal still removes live IDs only, not payload records.

**Existing tests / deferred maintenance:** None. The one removal fixture construction edit preserves all its assertions.

## [4] Navigation / aura-entry rekey

**Input:** `$SCRATCH/handoff-nav-rekey.md`; staging `$SCRATCH/staging/nav-rekey/`.
**Handoff producer-code section:** Exact existing-file replacements (numbered sections).

**Files added:**
- `tests/patch_12_0_5_navigation_aura_entry.rs`
- `src/c_api/aura_entry_ids.rs`
- `src/c_api/aura_entry.rs`
- `src/c_api/c_navigation.rs`
- `docs/specs/aura-entry-instance-ids.md`
- `docs/specs/navigation-nearest-party-token.md`

**State / DOC applied:** Added retail-12-0-5 nearest_party_member_token=None and AuraEntryIds default with pending=None/empty retired set. Added declarations for all three new modules: aura_entry_ids, c_navigation, aura_entry. Did not register navigation, add event hooks, delete nearest-token publisher, or change any existing navigation assertions.

**WITHHELD producers:**

| Operation | File | OLD anchor / affected code | Handoff code location |
|---|---|---|---|
| 4.5 | `src/c_api/mod.rs` | `c_chat_info::register(state)?;` | `handoff-nav-rekey.md:146`; exact OLD/NEW, operation 5 |
| 4.6 | `src/lua_api/env_events.rs` | `pub fn fire_event_with_args(&self, event: &str, args: &[Val]) -> Result<()> {` | `handoff-nav-rekey.md:160`; exact OLD/NEW, operation 6 |
| 4.7 | `src/lua_api/globals/state_backed_queries.rs` | `super::real::event_callbacks::dispatch_event_callbacks(state, event_name, args)?;` with its cfg | `handoff-nav-rekey.md:176`; exact OLD/NEW, operation 7 |
| 4.8 | `src/lua_api/loader_env.rs` | `let listeners = self.with_state(\|state\| { (LoaderEnv dispatch)` | `handoff-nav-rekey.md:192`; exact OLD/NEW, operation 8 |
| 4.9 | `src/lua_api/workarounds/temporary/navigation_defaults.rs` | `installNavigationDefault("GetNearestPartyMemberToken", function()` | `handoff-nav-rekey.md:208`; exact OLD/NEW, operation 9 |

**Test filters:** `patch_12_0_5_navigation_aura_entry::`.

**Expected RED:** Eight predicted failures: nil placeholder cannot return selected token and wrongly accepts addon calls; encounter/M+/PvP dispatch still exposes old IDs, accepts invalid batches, and does not rekey shared player/party stores. Host batches remain pending. Private sound IDs are a separate domain and untouched.

**Existing tests / deferred maintenance:** Deferred `tests/c_navigation_probes.rs::navigation_fallbacks_return_safe_empty_defaults` (four edits) and `src/lua_api/workarounds/temporary/navigation_defaults.rs::tests::installs_safe_empty_navigation_defaults` (three edits). Earlier-epoch `tests/blizzard_quest_navigation_loads.rs::blizzard_quest_navigation_consumes_c_super_track_namespace` needs provider-scope adjustment after placeholder deletion; no authored replacement supplied. Preserve `preserves_existing_navigation_provider`.

## [3] Aura headers / ClassTalentHelper

**Input:** `$SCRATCH/handoff-aura-header.md`; staging `$SCRATCH/staging/aura-header/`.
**Handoff producer-code section:** Exact existing-file replacements (Path: class_talents.rs).

**Files added:**
- `tests/secure_aura_header_helpers.rs`
- `src/c_api/class_talent_commands.rs`

**State / DOC applied:** Added class_talent_commands module under retail-12-0-5. No new SimState fields. All four existing direct SwitchTo* implementations and registration remain unchanged; no vendor sorting or Lua changes. All five proposed existing fixture edits remain deferred because they are not needed to compile. No spec file was staged for this slice.

**WITHHELD producers:**

| Operation | File | OLD anchor / affected code | Handoff code location |
|---|---|---|---|
| 3.2 | `src/lua_api/globals/missing_surface/traits/class_talents.rs` | `fn current_config_ids(` | `handoff-aura-header.md:38`; exact OLD/NEW, operation 2 |
| 3.3 | `src/lua_api/globals/missing_surface/traits/class_talents.rs` | `fn c_class_talents_switch_to_loadout_by_name(` | `handoff-aura-header.md:50`; exact OLD/NEW, operation 3 |
| 3.4 | `src/lua_api/globals/missing_surface/traits/class_talents.rs` | `fn c_class_talents_switch_to_loadout_by_index(` | `handoff-aura-header.md:62`; exact OLD/NEW, operation 4 |
| 3.5 | `src/lua_api/globals/missing_surface/traits/class_talents.rs` | `fn c_class_talents_switch_to_specialization_by_name(` | `handoff-aura-header.md:74`; exact OLD/NEW, operation 5 |
| 3.6 | `src/lua_api/globals/missing_surface/traits/class_talents.rs` | `fn c_class_talents_switch_to_specialization_by_index(` | `handoff-aura-header.md:86`; exact OLD/NEW, operation 6 |
| 3.7 | `src/lua_api/globals/missing_surface/traits/class_talents.rs` | `fn register_c_class_talents_action_fns(` | `handoff-aura-header.md:98`; exact OLD/NEW, operation 7 |

**Test filters:** `secure_aura_header_helpers::`.

**Expected RED:** Two cached aura-order tests may already pass. Three helper tests predicted RED: direct mutation bypasses real UI callbacks/pending spec cast; primed real loadout field stays unchanged; addon observer error is never delivered. Cache/UI setup failures must be separated from behavioral RED.

**Existing tests / deferred maintenance:** Deferred: `tests/admin_spec_talent_api.rs::test_trait_config_mapping_tracks_active_loadout`; `tests/hero_talents.rs::test_non_selectable_hero_nodes_do_not_show_selectable_glow`; `tests/hero_talents/rendering.rs::{test_class_talent_edges_render_below_visible_talent_buttons,test_button_frame_level_change_relevels_connected_edges_on_update,test_hero_spec_content_spec_image_anchors_to_spec_name}`. Also `tests/hero_talents.rs::test_class_talents_switch_methods_update_seeded_spec_and_loadout_state` needs a real cached-helper completion-lifecycle replacement: none is authored. Do not weaken its immediate-mutation assertions or substitute unchanged state as completion proof.

## [12] Group combat restrictions (last)

**Input:** `$SCRATCH/handoff-combat-restrictions.md`; staging `$SCRATCH/staging/combat-restrictions/`.
**Handoff producer-code section:** Staged additions and deliberate hold / Required integration.

**Files added:**
- `tests/group_combat_restrictions.rs`
- `src/c_api/c_party_info/combat_restrictions.rs`
- `docs/specs/group-combat-restrictions.md`

**State / DOC applied:** Added retail-12-0-5 public c_party_info::combat_restrictions host-only module and SimState.group_restrictions with empty assistant_names/is_raid=false. Added all three exact state/domain edits; no Lua adapter, query rewrite, callback guard, events or historical global was added.

**WITHHELD producers:**

No authored existing-file producer replacement exists. `handoff-combat-restrictions.md`, **Staged additions and deliberate hold** / **Required integration** explicitly withholds seven Lua adapters: PromoteToLeader, PromoteToAssistant, DemoteAssistant, SetEveryoneIsAssistant, ConvertToParty, ConvertToRaid, ConfirmConvertToRaid. Existing leadership adapter anchors are the same named callback bodies in `src/c_api/c_party_info.rs`; conversion callbacks are absent. Still held: VM caller classification/original argument authentication; per-member assistant and leader readers; explicit kind in `IsInRaid`, `GetNumRaidMembers`, `C_PartyInfo.GetActiveGroupType`, `IsPartyFull`; source-backed denial handling. No Lua denial shape/code is supplied. **This is an authoring blocker, not an exact-anchor patch ready for Phase 2.** Do not fabricate return/error contracts.

**Test filters:** `group_combat_restrictions::`.

**Expected RED:** Eight predicted failures: current leader/everyone/assistant producers mutate during combat; individual promotion/demotion uses everyone flag; three conversion callbacks are absent or have no live effects. Tests exercise actual C_PartyInfo calls and preserve the existing combat/chat-lockdown independence.

**Existing tests / deferred maintenance:** Deferred two exact edits to `src/loader/tests/wow_api_globals/startup_globals.rs::test_patch_12_0_7_safe_global_bridges`, supplied in `staging/combat-restrictions/deferred-startup-expectations.json`. Explicit raid-kind integration will require revisiting `tests/party_raid_probes.rs::is_in_raid_true_when_six_or_more_members`, `tests/admin_party_api.rs::test_get_num_raid_members_counts_raid_including_player`, and `tests/party_loot_method.rs::master_selection_resolves_modeled_party_and_player_roster_identity`. Audit `tests/secure_group_headers.rs::secure_raid_group_header_spawns_raid_unit_children` (SetPartySize(7)); replacement expectations are not authored.

## Phase-boundary resolutions

- New-module declarations tagged producer by authors were applied under the explicit Phase 1 module rule: sound `add`, follower `nameplate_display`, exterior `core`, navigation `c_navigation`/`aura_entry`, chat `expressions`, and talent commands. They are inert until the withheld registrations/dispatch are applied.
- Four visibility-only edits were required to compile new module references: follower `resolve_unit_is_player` and exterior `update_attachments`, `validate_host`, `find_affected_placements`. Their function bodies are unchanged. **No registration, guard, dispatch or behavior edit inside an existing Rust function was required or applied.**
- Navigation's seven STATE-tagged existing-test edits were deferred because they remove expectations, not compile dependencies; this follows the explicit Phase 1 instruction. Chat's three expectation edits and talents' five fixture edits likewise remain deferred. Only two existing test constructions changed for added struct fields.
- Housing Lua seed deletion was applied because the handoff explicitly marks it STATE. Its helper and three publishers remain withheld, so their existing calls currently reference absent seed state. Phase 2 must retire those publishers and install the Rust producer together; do not call this checkpoint a passing existing storefront baseline.
- Cached aura sorting and restricted-outfit helper tests are controls that may already be GREEN. Do not force every new test to fail or invent a producer solely for a RED count. Real talent completion and combat-denial adapters remain unresolved beyond this mechanical Phase 1.

## Final proof ledger

| Evidence | Scope / revision | Result |
|---|---|---|
| Read-only HEAD/status and anchor preflight | Clean starting tree at aa29d7d7f; selected handoffs and proposed maintenance | 92 exact OLD checks, each unique; all 33 destinations absent before addition |
| Anchored edits + new-file inventory | Current working tree, selected ten slices only | 41 operations; 33 added / 19 changed paths; no unintended modified path |
| Static dependency/API/fixture inspection | Current added modules/tests and changed existing declarations | Four visibility dependencies addressed; two exhaustive fixtures updated; no known prohibited test API usage or new suppressions |
| `cargo fmt` | Final Rust working tree; no later Rust edits | Exit 0; parsing/formatting only |
| Remaining OLD anchor checks; protected-path comparison | After formatter, same working tree | All 50 deferred replacements unique; Cargo manifests, build discovery and integration harness unchanged |
| Compilation / behavioral RED / broader suites | Not authorized | NOT RUN; no compile/pass/fail claim |

Report: `$SCRATCH/b99-phase1.md`. Phase 2 still needs authorized compilation and observed behavioral RED before any GREEN or completion claim.
