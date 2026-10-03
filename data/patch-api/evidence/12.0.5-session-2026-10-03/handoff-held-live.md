# Held-live audit handoff — 2026-10-03

## Scope and verdict

Named ten source occurrences audited against the retained register/text, current cached declarations, live Rust producers, existing behavioral tests, and the supplied prebuilt integration binary. Authored files stay in scratch; no repository edits, implementation changes, git mutations, service operations, or cache/vendor edits performed. No cargo, agents, Supervisor, model CLIs, or rebuilds used.

| Row | Proposed accounting | Verdict |
|---|---|---|
| 342 | bounded-coverage; tooltip-unit-buff-security | Existing tests prove removed arg1 NeverSecret within live player/party helpful-aura domain. |
| 347 | bounded-coverage; tooltip-unit-debuff-security | Existing tests prove removed arg1 NeverSecret within live player/party harmful-aura domain. |
| 278 | bounded-coverage; housing-pending-decor | Existing tests prove full variant selector, including variants sharing a base. Argument-name change is declaration metadata, not an additional runtime behavior. |
| 279 | bounded-coverage; housing-pending-decor | Existing tests prove full variant identity reaches actual pending state. No full-placement claim. |
| 281 | bounded-coverage; housing-destroy-entry | Existing tests prove full variant selector targets live deletion. Argument-name change is declaration metadata. |
| 282 | bounded-coverage; housing-destroy-entry | Existing tests prove record/type/variant isolation. Native mixed-stack policy remains unproved. |
| 291 | bounded-coverage; mount-spell-identifier | Existing tests prove public numeric and explicitly registered string identifiers against live mount records; not complete native SpellIdentifier/AllowedWhenTainted parity. |
| 650 | partial-development-green; housing-catalog-aggregates | Explicit totalNumStored publication proved. Missing input omits cached required number; real conformance gap, not absent test or placeholder. |
| 651 | partial-development-green; housing-catalog-aggregates | Explicit totalNumPlaced publication proved. Same independently demonstrated missing-input gap. |
| prose-2026-03-25-088 | metadata-only, explicitly superseded historical proposal; link unit-comparison-permissions | Do not implement historical blanket secret return. March31 policy has existing bounded permission and identity proof. Metadata-only classifies the obsolete proposal, not the current API. |

None of these producers is constant/no-op. Global bounded credit concerns the retained input/name/type delta, not whole-function/native/all-profile completion. Aggregate rows remain partial, not complete.

## Exact source and cached declarations

Register: `data/patch-api/sources/12.0.5-register.json`, atomic `change` and `source_lines`. Text: `data/patch-api/sources/12.0.5-api-changes.txt`, one-based lines below. Cache prefix: `/home/osso-test/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/`. Cache contents are observations, not independently established 12.0.5 build provenance.

| Row | Exact retained change | Cached declaration |
|---|---|---|
| 342 | `- arg1 NeverSecret` | `TooltipInfoDocumentation.lua:1239-1256`: GetUnitBuff, `AllowedWhenUntainted`; nonnil `unitToken: UnitTokenRestrictedForAddOns`, `index: luaIndex`, optional `filter: AuraFilters`; `RequiresUnitAuraAccess`, `SecretWhenUnitAuraRestricted`, `MayReturnNothing`; required TooltipData when returned. No arg1 NeverSecret. |
| 347 | `- arg1 NeverSecret` | Same contract for GetUnitDebuff at `TooltipInfoDocumentation.lua:1279-1296`. No arg1 NeverSecret. |
| 278 | `# arg1.Name catalogEntryID -> catalogEntryVariantID` | `HousingBasicModeUIDocumentation.lua:185-193`: StartPlacingNewDecor, `AllowedWhenUntainted`, required `catalogEntryVariantID: HousingCatalogEntryVariantID`. |
| 279 | `# arg1.Type HousingCatalogEntryID -> HousingCatalogEntryVariantID` | Same declaration. Full compound key at `HousingCatalogConstantsDocumentation.lua:87-95`: recordID, entryType, variantIdentifier. |
| 281 | `# arg1.Name entryID -> entryVariantID` | `HousingCatalogUIDocumentation.lua:31-40`: DestroyEntry, `AllowedWhenUntainted`, required `entryVariantID: HousingCatalogEntryVariantID`, required `destroyAll: bool`. |
| 282 | `# arg1.Type HousingCatalogEntryID -> HousingCatalogEntryVariantID` | Same declaration; destroyAll: “If true, deletes all entries within the stack; If false, will only delete one”. Full compound key as above. |
| 291 | `# arg1.Type number -> SpellIdentifier` | `MountJournalDocumentation.lua:249-261`: GetMountFromSpell, `AllowedWhenTainted`, required `spellID: SpellIdentifier`, nullable numeric mountID. |
| 650 | `+ totalNumStored` | `HousingCatalogUIDocumentation.lua:555`: required number, total stored across variants excluding unredeemed instances. |
| 651 | `+ totalNumPlaced` | `HousingCatalogUIDocumentation.lua:557`: required number, total placed across houses/plots/variants. |
| prose-2026-03-25-088 | `UnitIsUnit now returns secrets if either unit in the comparison is "targettarget" or "focustarget".` | `UnitDocumentation.lua:2321-2336`: RequiresComparableUnitTokens, SecretWhenUnitComparisonRestricted, AllowedWhenUntainted, two required UnitTokens, required bool result. Cache does not independently settle denied-call arity. |

Register marks prose088 `superseded-or-pre-release-proposal`. Later text141-144, March 31, 2026: either base token permits comparison (player, pet, vehicle, mouseover, target, softenemy, softfriend, softinteract, focus, none, npc, questnpc); otherwise a group/group-pet token permits a noncompound, non-nameplate counterpart other than targettarget/focustarget. “All other comparisons are disallowed and return nil.” Source088 must stay preserved verbatim; its blanket secret-return sentence is not the final contract.

## Producers and existing behavioral assertions

### 342 — GetUnitBuff

Producer: `src/c_api/c_tooltip_info_indexed_aura.rs:19,23-50,92-113`. Original unit/index/filter are authenticated with unwrap_secret before any parser/lookup. Helpful polarity selects current player/party stores. Selected aura payload passes through `src/lua_api/globals/missing_surface/tooltip_info/mod.rs:109-114`; this is not an empty-tooltip shim.

Existing proof in `tests/tooltip_unit_buff_security.rs`:
- `secure_secret_unit_selects_actual_party_not_player` (:514): UBCheck expects **Party one outside source** at index1 and **Party one player source** at index2 using secret UBUnit. UBCheck (:99-108) asserts three actual lines and `data.lines[1].leftText == name`.
- `secure_combined_secret_selectors_return_meaningful_content_and_true_miss` (:540): secret unit/index/filter returns Party one player source; filtered second row has `next(data.lines) == nil`.
- `tainted_secret_each_position_denied_before_lookup_or_public_parse` (:549): UBReject covers secret unit plus invalid public index and secret index/filter plus invalid earlier arguments. UBReject (:144-154) asserts pcall failure, “requires an untainted caller”, API context and unchanged stack taint. UBTainted stamps/checks UnitBuffFixture (:133-142).
- `live_host_mutation_removal_and_environment_isolation` (:446) and rooted GC proof (:608) distinguish a live producer from constant/player fallback.

Verdict (a), bounded input-delta proof complete. Aura-access/restricted-output policy is separate, unmodeled scope; no new tests required for arg1 removal.

### 347 — GetUnitDebuff

Producer: same indexed module `:20,27-50,92-113`, harmful polarity and live harmful stores. Harmful target fixture deliberately excluded.

Existing proof in `tests/tooltip_unit_debuff_security.rs`:
- `secure_original_secret_unit_index_filter_and_nil_drive_real_selection` (:514): UDCheck expects Party one outside source for secret unit; Player own source for secret index/filter; Party one player source for combined secrets; a genuine filtered miss. UDCheck (:99-108) asserts `data.lines[1].leftText == name` and three actual lines.
- `tainted_secret_each_position_authenticates_before_earlier_malformed_parse` (:533): invalid earlier public arguments plus later secrets still produce authentication rejection. UDReject (:145-155) asserts untainted-caller gate and preserves stack taint; UDTainted stamps/checks UnitDebuffFixture.
- `live_mutation_removal_and_clear_refresh_snapshots_without_cross_unit_leaks` (:452) and GC/root proof (:618) prevent constant/empty/player-fallback credit.

Verdict (a), bounded input-delta proof complete. No aura-access/restricted-output/native/general-unit claim; no new tests needed.

### 278 / 279 — StartPlacingNewDecor

Producer: `src/c_api/c_housing/basic_mode.rs:28-41`; exact key selects `housing.catalog.variants` and records `housing.pending_new_decor`. `basic_mode/pending_input.rs:10-45` authenticates original selector and all original fields before field validation.

Existing proof in `tests/housing_pending_decor.rs`:
- `repeated_and_replacement_requests_preserve_full_identity` (:171): base record1001, Decor variants1/2 and Room variant1 update pending to each exact key; mutating caller table after selection cannot change copied pending identity. `assert_pending` (:58-63) asserts Rust key equality AND live IsPlacingNewDecor.
- `replacement_request_changes_record_identity` (:195): otherwise matching full key for record1002 selects that distinct live record.
- `secure_secret_numeric_fields_accept_actual_full_variant_request` (:427) and `secure_secret_table_selector_accepts_and_copies_full_identity_across_gc` (:498): secret key components/tables select variant1 versus2; result arity zero, caller remains secure, counts unchanged and second environment remains empty.
- `all_original_fields_authenticate_before_any_public_parse_domain_or_model_lookup` (:571): tainted original fields produce the untainted-caller error before malformed earlier fields, with pending unchanged.

Verdict (a) for BOTH rows: full variant identity has actual state effect. Placement eligibility/parser/cancel are inferred; finished placement, reservation, instance creation and placement events not proved or required by these selector annotations. No new tests needed.

### 281 / 282 — DestroyEntry

Producer: `src/c_api/c_housing/catalog/storage.rs:25-54`, registration `catalog/queries.rs:36`; full-key lookup mutates only chosen variant stored/destroyable counts, then publishes storage update. `catalog/destroy_input.rs:18-59` authenticates selector/destroyAll and all key fields before validation.

Existing proof in `tests/housing_destroy_entry.rs`:
- `full_key_isolates_variant_type_and_record` (:279): delete secondID, roomID, otherID; event counts become (6,1), (5,0), (4,0), while firstID stays (5,3). `assert_model` proves all four live records, not only returned DTO shape.
- `false_destroys_one_eligible_instance_before_synchronous_event` (:195): firstID becomes (4,2), event visible before return.
- `secure_secret_table_selector_accepts_exact_key_and_one_event` (:700) and secure numeric/boolean cases (:644/:673): authenticated key drives real mutation to (4,2) or (2,0); event1 contains that key, isolated environment unchanged.
- `original_top_arguments_authenticate_before_selector_or_boolean_type_errors` (:726) and all-fields case (:748): atomic tainted-secret denial before ordinary validation/model errors.

Verdict (a) for BOTH identifier/name rows. `true_destroys_all_eligible_not_all_stored_in_mixed_stack` (:210) explicitly proves inferred eligible-subset behavior, NOT cached “all entries” native parity. No new selector-proof tests needed.

### 291 — GetMountFromSpell

Producer: `src/c_api/c_mount_spell_lookup.rs:14,17-32`; public identifier reader/explicit aliases feed a live scan of `world.mounts`. It does not call the C_Spell companion or return a canned mount.

Existing proof in `tests/mount_spell_identifier.rs`:
- `numeric_brown_horse_spell_returns_existing_mount` (:90): CheckMount(458,6); actual relation exists.
- `explicit_name_alias_uses_lowercase_registry_key` (:127) and `link_shaped_alias_uses_registered_value_not_embedded_spell` (:144): explicit strings resolve the registered value, not invented native grammar.
- `existing_relation_mount_id_and_spell_updates_are_live` (:177): numeric/alias queries change 6 → 6006; changing spell to999998 makes458 miss and999998 return6006.
- `removal_of_actual_mount_record_invalidates_numeric_and_alias_queries` (:210): remove mount6; numeric458 and alias become nil while40192 still returns107.
- `actual_vm_secrets_reject_as_unmodeled_policy_in_secure_and_tainted_calls` (:306): retained negative policy, NOT AllowedWhenTainted implementation proof.

CheckMount (:33-48) calls actual C_MountJournal function and asserts `result == expected`. Verdict (a) for bounded public identifier support; full native identifier/secret permissions remain open. No new bounded tests needed.

### 650 / 651 — aggregate fields

Producer: `src/c_api/c_housing/catalog.rs:35-44` holds explicit Option<u32> totals; `catalog/snapshot.rs:104-122` publishes supplied numbers into actual base record DTO; three live getters in `catalog/queries.rs:73-140` select the base record. No variant sums fabricated.

Existing proof in `tests/housing_catalog_aggregates.rs`:
- `entry_info_publishes_explicit_aggregates` (:96), `by_item_publishes_explicit_aggregates` (:101), `by_record_id_publishes_explicit_aggregates` (:106): assertAggregates expects stored37/placed11, independently of variant counts3/5. Helper asserts `info.totalNumStored == stored` and `info.totalNumPlaced == placed`.
- `aggregate_input_mutation_does_not_change_variants_or_pending_state` (:172): host changes totals to41/0; all getters publish41/0, variants remain3/5 and pending unchanged.
- `explicit_zero_is_not_missing_inferred_policy` (:111), large unsigned values (:135), snapshot independence (:195): supplied zero/large numbers and live host changes are not constants or per-variant sums.
- `each_aggregate_can_be_missing_independently_inferred_gap` (:127): supplied(37,None) yields37/nil; supplied(None,11) yieldsnil/11. `missing_aggregates_are_nil_not_variant_sums_inferred_gap` (:119) proves nil/nil even with nonzero variants.

Both added fields have bounded publication proof, but BOTH cached declarations require numbers on populated DTOs. Missing-host-data conformance is a real known implementation gap already covered by behavioral tests/spec. Verdict: partial, not placeholder and not a missing-test case. A new PASS test cannot make omitted fields nonnil; do not add duplicate tests or silently default unknown counts to zero. Existing aggregate spec owns the gap.

### prose-2026-03-25-088 — final UnitIsUnit policy

Producer: `src/lua_api/globals/unit_misc.rs:340-436`; VM-authenticated secret tokens, symmetric base/group permission helpers, current GUID equality; denial at421 returns zero results. Earlier scout line numbers are stale; current locations used here.

Existing proof:
- `tests/unit_comparison_permissions.rs::comparison_permissions_base_tokens_allow_even_restricted_counterparts`: all12 base tokens × four restricted counterparts in both orders, expected false for absent identities; targeting player/focus produces true. In particular player/targettarget and player/focustarget are permitted booleans, not blanket secrets.
- `comparison_permissions_group_and_pet_tokens_allow_simple_counterparts`: both directions, live party2 target/focus aliases true, party1/party2 false.
- `comparison_permissions_group_and_pet_tokens_deny_restricted_counterparts`: both directions expected None for compound/nameplate/targettarget/focustarget; `select('#', UnitIsUnit('party1','nameplate1')) == 0`.
- `comparison_permissions_nonbase_pairs_are_denied_even_when_identical`: residual nonbase6×6 pairs and invalid groups yield None symmetrically.
- `comparison_permissions_typed_secrets_require_untainted_callers`: secret player token compares true; tainted secret inputs pcall-fail; public player/player stays true and UnitComparisonProbe stack taint is retained.
- `tests/unit_api.rs::test_unit_is_unit_target_and_focus_alias_player_symmetrically` (:607), `test_unit_is_unit_retarget_and_clear_change_alias_identity` (:632), `test_unit_is_unit_party_alias_disappears_after_roster_shrink` (:669): real same-GUID aliases compare true; retargeting/clearing or roster removal changes relevant comparisons to false while retained snapshots survive.

Verdict (a) for bounded FINAL policy plus explicit historical-supersession classification. Final nil-expression denial is proved; explicit one-nil native return arity is not. No new tests or historical secret-output model needed.

## Proposed capability entry text

Update existing entries; do not create duplicate capabilities. Exact scope sentences and complete entries are in the staged accounting candidate. Proposed id/symbol/spec/tests mapping:

| ID | Symbols | Spec | Tests |
|---|---|---|---|
| tooltip-unit-buff-security | C_TooltipInfo.GetUnitBuff | docs/specs/tooltip-unit-buff-security.md | tests/tooltip_unit_buff_security.rs |
| tooltip-unit-debuff-security | C_TooltipInfo.GetUnitDebuff | docs/specs/tooltip-unit-debuff-security.md | tests/tooltip_unit_debuff_security.rs |
| housing-pending-decor | C_HousingBasicMode.StartPlacingNewDecor, IsPlacingNewDecor, CancelActiveEditing | docs/specs/housing-pending-decor.md | tests/housing_pending_decor.rs |
| housing-destroy-entry | C_HousingCatalog.DestroyEntry | docs/specs/housing-destroy-entry.md | tests/housing_destroy_entry.rs |
| mount-spell-identifier | C_MountJournal.GetMountFromSpell | docs/specs/mount-spell-identifier.md | tests/mount_spell_identifier.rs |
| housing-catalog-aggregates | HousingCatalogEntryInfo.totalNumStored, HousingCatalogEntryInfo.totalNumPlaced | docs/specs/housing-catalog-aggregates.md | tests/housing_catalog_aggregates.rs |
| unit-comparison-permissions | UnitIsUnit | docs/specs/unit-identity-equality.md | tests/unit_comparison_permissions.rs, tests/unit_api.rs |

Scope sentences, respectively: authenticated live helpful aura input removal; authenticated live harmful aura input removal; authenticated full-key pending request, not finished placement; authenticated full-key selected destruction, not native mixed-stack semantics; explicit public aliases into live mount records, not native secret permission/grammar; supplied base aggregate publication with missing-required-field gap; final March31 symmetric permissions/live GUID equality superseding March25 proposal, with nil-expression versus native arity qualification. Detailed exclusions are preserved in the per-row verdicts and staged sentences.

## Observed proof ledger

Read-only HEAD: `ced356e95d3b992734c9f97c7fdabd787f06197f`. Working tree is shared/dirty: HEAD is NOT a claim of clean source or compiled revision.

Supplied binary SHA256, unchanged before/after every run: `ad981f4b75fb89a3272eaeef2d6161ba551221d76d10755297d7084f74ddda1d`. Command for each row below: `target/debug/deps/integration-8ea324359263a4d2 <filter> --test-threads=1`, with 90-second process bound. Four independent processes maximum. No retry necessary. All exit0, zero failures, empty stderr.

| Filter | PASS |
|---|---:|
| housing_pending_decor:: | 20 |
| housing_destroy_entry:: | 24 |
| mount_spell_identifier:: | 16 |
| tooltip_unit_buff_security:: | 18 |
| tooltip_unit_debuff_security:: | 15 |
| housing_catalog_aggregates:: | 13 |
| unit_comparison_permissions:: | 6 |
| unit_api::test_unit_is_unit | 6 |
| Total distinct existing cases | 118 |

Artifacts: `held-live-observed/runs.json`, one stdout/stderr pair per filter, `inputs-before.json`, `input-drift.json` (empty), `revision.txt`. Relevant producer/test hashes stayed unchanged across execution. Prebuilt binary's build-source equivalence was not independently established here; results prove the identified binary's cases, not newly compiled current-tree/final acceptance. Existing independent historical ledgers stay preserved in capability entries. No new startup/format/check/native/all-profile proof claimed.

Source text SHA256: `4da3872aa566695f46e2dacd4e79992f5b06be9541f0d19cf0e8dba45cea8329`; register SHA256: `eaea58ae8adf215587cea6de12349b3586fcb2520a4c8aefd4d7cee5406046ed`. Both match accounting provenance.

## Staging, new tests and integration limits

Staged candidate: `staging/held-live/data/patch-api/sources/12.0.5-page-coverage.json`, mirroring repo path. Only seven existing capability scope sentences and ten named source row capability/status/note fields are proposed changes. No new capability or mutation to retained implementation/compiled-revision/proof/gate/ledger fields.

Machine-readable selected updates and baseline SHA: `held-live-observed/accounting-proposal.json`. Apply only those named updates against the latest shared file, not a wholesale stale snapshot overwrite. Existing specs contain historical pending wording; this handoff is an accounting proposal, not applied acceptance/spec reconciliation.

**New test files: none. New specs: none.** All seven existing specs already own the bounded contracts and exclusions. Existing tests already assert the requested live-state deltas; aggregate required-field gaps already have tests. No uncovered bounded behavior justified duplicate authoring. New-test expected-PASS and constant/placeholder counterexample declarations: not applicable because no new tests were authored. Existing counterexamples are meaningful: aura content/unit changes defeat empty or player-only producers; variant replacement/deletion defeats constant/no-op or base-only producers; mount mutation/removal defeats constants; aggregate37/11→41/0 defeats canned counts; UnitIsUnit true/false/nil matrices and live identity changes defeat constant returns.

No row proposed as full native support. No aggregate conformance fix, token grammar invention, secret fallback, real placement or commerce work authorized or performed.
