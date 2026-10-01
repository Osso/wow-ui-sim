# Housing catalog destruction

Bounded 12.0.5 destruction slice for `C_HousingCatalog.DestroyEntry(entryVariantID, destroyAll)`, using the existing [catalog records](housing-catalog-variants.md) and independent [destroyable count](housing-destroyable-count.md). Retained [source register](../../data/patch-api/sources/12.0.5-register.json) IDs `global api-C_HousingCatalog-DestroyEntry-281` / `-282` rename the selector and change its type to `HousingCatalogEntryVariantID`. These parameter rows do not establish eligibility, mutation or native event timing. Parent reported actual compiled RED for input/tests `f94063616`: 15 cases, 2 PASS / 13 FAIL against the Lua no-op (`batch21-red-build/run*`). Producer implemented after that prerequisite; GREEN remains parent-owned.

## What it must do

### Selection and inferred eligibility

- [ ] Accept an accessible public full `(recordID, entryType, variantIdentifier)` table and a required ordinary bool `destroyAll`. Successful calls return zero Lua values. Select only the exact existing variant; never use numeric, base-entry, alternate-variant or legacy-seed lookup.
- [ ] **Explicit simulator inference:** `destroyable_instance_count` identifies the eligible subset of `num_stored`, independently supplied, not derived from storage. False removes one eligible instance; true removes all eligible instances. Reduce both counts by the same amount. For a mixed stack stored=5/eligible=3, false produces 4/2 and true on a fresh fixture produces 2/0. False followed by true also ends at 2/0, preserving the two exempt/protected instances.
- [ ] Preserve other variants, other record IDs and other entry types even when identity components overlap. Preserve every dye field and all base-entry metadata. Retain the variant record when both counts reach zero.
- [ ] **Inferred no-op policy:** missing full keys or zero eligible count succeed with zero returns, no mutation/insertion and no event. Stored instances alone do not grant deletion permission. Default environments contain no catalog records; never invent eligibility/default data.
- [ ] **Explicit simulator consistency policy:** negative `num_stored`, negative `destroyable_instance_count`, or eligible greater than stored fail before mutation, including stored=0/eligible=1. Do not clamp or repair inputs. One/all produce the same nonempty diagnostic for the same inconsistency, naming `numStored` or the destroyable count as applicable; preserve the original invalid input and all other records, and emit nothing.

### Events and errors

- [ ] Successful nonzero deletion dispatches exactly one real `HOUSING_STORAGE_ENTRY_UPDATED` callback with exactly one full variant-ID argument after both counts change and the model borrow is released. Listener queries see changed storage/eligibility and unchanged dye data. No queued duplicate event.
- [ ] **Inferred simulator timing, not native claim:** callback completes before `DestroyEntry` returns, following the existing [admin storage producer](housing-storage-entry-updated.md), implementation `5afd73d49`. Reentrant reads and same/other-variant deletions complete inside the listener; nested callbacks see each transition. Outer completion must not overwrite nested state.
- [ ] Malformed public selectors and missing/non-bool `destroyAll` fail explicitly and atomically, even when a well-formed selector would be missing. Selector fields must be ordinary finite nonnegative integers representable by the existing `i32` fields. Reject missing fields, non-table selectors, strings, fractions, nonfinite values and out-of-range values; no coercion or truncation. Error messages are nonempty; rejected calls do not mutate, insert or emit.
- [ ] Ordinary tainted addon calls succeed without clearing taint. Secret whole selectors, each nested identity field and the secret boolean reject in secure and tainted contexts without declassification, mutation or event. Secured tables retain the host VM access policy: tainted indexing and destruction reject; secure destruction remains allowed. Conservative secret rejection is a simulator limit, not native `AllowedWhenUntainted` parity.
- [ ] Mutation and callbacks remain local to the owning `WowLuaEnv`.

### Cached grounding and consumer evidence — 2026-10-01

Inspected exact local cache ranges in `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/HousingCatalogUIDocumentation.lua`:

- Lines **31–40** declare `DestroyEntry`, `SecretArguments = "AllowedWhenUntainted"`, nonnil `entryVariantID: HousingCatalogEntryVariantID` and nonnil `destroyAll: bool`. Documentation: "Attempt to delete the entry from storage"; true "deletes all entries within the stack", false "will only delete one". No return declaration follows this argument block.
- Lines **237–250** declare `GetDestroyableInstanceCount` on the same full ID. Documentation: "Returns the number of instances that can be to be destroyed in storage; These instances count towards the max storage limit". This is eligibility/storage-limit input, not simply stored count.
- Lines **481–490** declare the event's one full-ID payload and `UniqueEvent = true`, but no `SynchronousEvent = true`. Event emission/timing are not established by the parameter-change rows.

Actual consumer `Blizzard_HousingTemplates/Blizzard_HousingCatalogEntry.lua:728–856` passes popup false/true unchanged through `OnDestroyConfirmed(self.entryVariantID, destroyAll)`. Single deletion is enabled only when `self.entryInfo.destroyableInstanceCount > 0`. Bulk presentation uses a fixed amount of five and requires that destroyable count to be at least five, with a comment about a later constant update. That UI presentation is not direct evidence of mixed-stack native deletion policy; the cache does not establish whether true deletes all eligible instances, a capped batch, or handles protected/exempt instances differently. No inspected direct evidence contradicts the chosen subset policy, but the fixed-five presentation prevents treating native "all" semantics as verified.

**All eligible is explicitly simulator inference for mixed protected/exempt stacks; native policy remains unknown.** Records do not model individual instance flags. This slice treats the independent count as the eligible subset; it does not claim every native storage-limit-counting instance or every protected/exempt category follows that rule. Cache provenance is the [catalog spec's current retail cache](housing-catalog-variants.md#cached-declaration-provenance-2026-10-01), not a historical native 12.0.5 observation.

## How it works

- [Existing catalog identity/model and queries](housing-catalog-variants.md).
- [Independent eligibility input/query](housing-destroyable-count.md).
- [Distinct admin storage producer and event inference](housing-storage-entry-updated.md).
- [Event dispatch](../event-system.md).

## Implementation inventory

- `tests/housing_destroy_entry.rs`: new grouped behavioral module, automatically discovered by existing `integration` harness; no new Cargo target.
- `src/c_api/c_housing/catalog.rs`: unchanged existing record inputs; sufficient for this slice.
- `src/c_api/c_housing/catalog/queries.rs`: unconditional Rust `DestroyEntry` registration alongside existing full-ID queries.
- `src/c_api/c_housing/catalog/{input,snapshot}.rs`: unchanged shared selector/VM access guards and rooted serializers.
- `src/c_api/c_housing/catalog/storage.rs`: validates public arguments and consistent counts, decrements only the exact variant's stored/eligible counts, then uses the existing admin event publisher after releasing the model borrow. Both producers retain the payload root across nested callbacks; destruction performs no post-dispatch write and never clears taint.
- `src/lua_api/workarounds/temporary/housing_catalog_state.lua`: exact `DestroyEntry` no-op removed; unrelated seeded policies unchanged. No fallback.

## Tests asserting this spec

All cases require a callable API before exercising it, so invalid-input rejection cannot pass solely because registration is missing. Actual frame `RegisterEvent` listeners capture count/dye observations; assertions after the public call establish callback completion rather than trusting potentially swallowed listener errors. Reentrant trace and completion are checked outside the callback. Rust assertions check all fixture identities/counts, every dye field, exact base metadata, record retention and absence of queued duplicates.

| Cases in `housing_destroy_entry` | Observable contract |
|---|---|
| `false_destroys_one_eligible_instance_before_synchronous_event`, `true_destroys_all_eligible_not_all_stored_in_mixed_stack`, `one_then_all_reduces_both_counts_and_preserves_exempt_instances` | Fresh 5/3 mixed stack, zero returns, state-before-event and inferred subset policy |
| `fully_eligible_stack_retains_zero_record_after_one_or_all`, `full_key_isolates_variant_type_and_record` | Zero record retention, exact compound identity and preserved metadata/dyes |
| `missing_keys_and_explicit_zero_eligibility_are_noops`, `empty_catalog_stays_empty_without_invented_permission` | No eligibility/default data invention, no-op/no-event behavior |
| `malformed_public_selectors_fail_atomically`, `destroy_all_is_required_public_boolean_even_for_missing_keys`, `inconsistent_counts_fail_explicitly_without_clamping_mutating_or_emitting` | Explicit atomic public/model validation, consistent one/all inconsistency diagnostics |
| `listener_reads_and_reenters_same_and_other_variant_before_outer_return`, `public_addon_call_preserves_taint`, `secret_selector_fields_and_boolean_reject_secure_and_tainted_callers`, `guarded_selector_retains_vm_access_policy_without_mutation_on_rejection`, `destruction_and_events_are_environment_local` | Reentrant timing, ordinary addon calls, conservative secret and VM guard behavior, environment isolation |

### Proof ledger and producer gate

- Baseline clean revision `5afd73d498340952f0e2a2c59ae1a718f92d5b6a`; model/query/event producer and cached consumer inspection only.
- Parent-reported actual RED at input/tests `f94063616`: 15 cases, 2 PASS / 13 FAIL from the existing Lua no-op; build compiled successfully. Artifacts `batch21-red-build/run*` were not independently inspected by this implementer. Parent filter: `housing_destroy_entry::` in existing target `integration`.
- `rustfmt --edition 2024 --config skip_children=true src/c_api/c_housing/catalog/storage.rs src/c_api/c_housing/catalog/queries.rs`: exit 0 on producer files; formatting only. Subsequent module-doc wording change does not affect formatting.
- Producer added with no local build/check/test execution. Parent must establish GREEN on the committed producer. Requirements checkboxes remain pending proof; no source-row credit or acceptance claim.
- `rustfmt --edition 2024 --config skip_children=true tests/housing_destroy_entry.rs`: exit 0 on the new test file; formatting only, not compilation or behavioral proof. Later test edits invalidate formatting scope.
- Producer slice authorizes formatting and commit only; no builds, checks, delegation or push. Existing tests, source accounting, other specs and vendor/cache definitions unchanged.

## Known gaps (current cycle)

- [x] Parent reports actual compiled RED before producer implementation.
- [ ] Parent-owned bounded GREEN/security/event acceptance remain pending.
- [ ] Native mixed-stack eligibility, fixed-five UI/batch meaning, invalid-input errors, secret access and event timing remain unknown.

## Out of scope

Native instance policy/probes, storage-limit aggregates, search refresh/filtering, placement, full entry DTOs, `CanDestroyEntry`, legacy argument compatibility, popup behavior, asynchronous/coalesced native events and whole source-row/page or all-profile completion. Those require separate evidence; this slice changes only the destruction producer, its exact no-op replacement and this contract.
