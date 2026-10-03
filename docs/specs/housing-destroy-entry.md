# Housing catalog destruction

Bounded 12.0.5 destruction slice for `C_HousingCatalog.DestroyEntry(entryVariantID, destroyAll)`, using the existing [catalog records](housing-catalog-variants.md) and independent [destroyable count](housing-destroyable-count.md). Retained [source register](../../data/patch-api/sources/12.0.5-register.json) IDs `global api-C_HousingCatalog-DestroyEntry-281` / `-282` rename the selector and change its type to `HousingCatalogEntryVariantID`. These parameter rows do not establish eligibility, mutation or native event timing. Parent reported actual compiled RED for input/tests `f94063616`: 15 cases, 2 PASS / 13 FAIL against the Lua no-op (`batch21-red-build/run*`). Producer implemented after that prerequisite; bounded independent acceptance at `67b44f2c3` is recorded below.

## What it must do

### Selection and inferred eligibility

- [ ] Accept an accessible full `(recordID, entryType, variantIdentifier)` table and a required bool `destroyAll`, including authenticated actual secrets for untainted callers under the method-local boundary below. Successful calls return zero Lua values. Select only the exact existing variant; never use numeric, base-entry, alternate-variant or legacy-seed lookup.
- [ ] **Explicit simulator inference:** `destroyable_instance_count` identifies the eligible subset of `num_stored`, independently supplied, not derived from storage. False removes one eligible instance; true removes all eligible instances. Reduce both counts by the same amount. For a mixed stack stored=5/eligible=3, false produces 4/2 and true on a fresh fixture produces 2/0. False followed by true also ends at 2/0, preserving the two exempt/protected instances.
- [ ] Preserve other variants, other record IDs and other entry types even when identity components overlap. Preserve every dye field and all base-entry metadata. Retain the variant record when both counts reach zero.
- [ ] **Inferred no-op policy:** missing full keys or zero eligible count succeed with zero returns, no mutation/insertion and no event. Stored instances alone do not grant deletion permission. Default environments contain no catalog records; never invent eligibility/default data.
- [ ] **Explicit simulator consistency policy:** negative `num_stored`, negative `destroyable_instance_count`, or eligible greater than stored fail before mutation, including stored=0/eligible=1. Do not clamp or repair inputs. One/all produce the same nonempty diagnostic for the same inconsistency, naming `numStored` or the destroyable count as applicable; preserve the original invalid input and all other records, and emit nothing.

### Events and errors

- [ ] Successful nonzero deletion dispatches exactly one real `HOUSING_STORAGE_ENTRY_UPDATED` callback with exactly one full variant-ID argument after both counts change and the model borrow is released. Listener queries see changed storage/eligibility and unchanged dye data. No queued duplicate event.
- [ ] **Inferred simulator timing, not native claim:** callback completes before `DestroyEntry` returns, following the existing [admin storage producer](housing-storage-entry-updated.md), implementation `5afd73d49`. Reentrant reads and same/other-variant deletions complete inside the listener; nested callbacks see each transition. Outer completion must not overwrite nested state.
- [ ] Malformed public selectors and missing/non-bool `destroyAll` fail explicitly and atomically, even when a well-formed selector would be missing. Authenticated selector fields must be finite nonnegative integers representable by the existing `i32` fields. Reject missing fields, non-table selectors, strings, fractions, nonfinite values and out-of-range values; no coercion or truncation. Error messages are nonempty; rejected calls do not mutate, insert or emit.
- [ ] Ordinary tainted addon calls succeed without clearing taint. `DestroyEntry` alone accepts actual secret inputs for untainted callers; addon callers reject with `requires an untainted caller` without clearing taint. This replaces the historical conservative-secret requirement, not the shared catalog/Admin guards.
- [ ] Authenticate both original top-level arguments before any selector/table or boolean type error. In particular, `(false, secretBool)` denies under addon taint but reaches the table-type error securely; a secret NUM selector authenticates before the table-type error even with a malformed second argument.
- [ ] Authenticate all three original identity fields and original `destroyAll` before field integer/range/domain checks or model lookup/consistency validation. Missing, malformed, public negative or out-of-range fields cannot mask another field's or arg2's secret denial; malformed/missing arg2 cannot mask any secret field. Authenticated secure inputs still undergo ordinary validation, unknown-key no-op and consistency errors.
- [ ] Secured tables retain underlying host VM access policy: public tainted indexing/destruction rejects, secret-wrapped tables deny addon callers at authentication, and secure wrapped-table destruction remains allowed. Do not bypass `check_table_access` by accepting a wrapper.
- [ ] Root underlying table selectors before secret-wrapper allocation; survive GC before calls and during event dispatch. Event IDs are independent of caller tables: later caller-table mutation/GC must not alter event identity or model keys.
- [ ] Mutation and callbacks remain local to the owning `WowLuaEnv`.

### B72 authored security requirements — 2026-10-03

- [ ] Genuine host secret NUM in each identity field independently and all three together accepts securely for one/all; actual secret BOOL false and true select one/all, including combined numeric/table secrets; actual secret TABLE selects the exact full key.
- [ ] Secure success returns zero values, preserves secure state, changes only the selected variant and emits exactly one full-ID synchronous event. Unrelated variants/environment, dye/base metadata, existing pending request and event queues remain unchanged.
- [ ] Paired addon calls deny each/all secret fields, each secret bool and secret tables with the host authentication diagnostic. Rejections preserve counts, every dye/base field, pending request, dispatch count, event queue and caller taint.
- [ ] Top-argument and all-original-fields/arg2 precedence matrices cover type, missing, finite/integer/range/domain, unknown-key and inconsistent-model boundaries before mutation.

B72 actual compiled RED is accepted; producer `7f7d0fe8a` follows it. GREEN and independent verifier acceptance remain pending. Scope is `DestroyEntry` alone: shared catalog selectors, Admin and conservative guards remain unchanged. These tests assert simulator host-secret behavior, not mixed-stack native policy, full placement or security-global parity.

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
- `src/c_api/c_housing/catalog/destroy_input.rs`: B72 producer `7f7d0fe8a` authenticates both original top-level arguments before selector/boolean type errors, and all three original identity fields before field type, integer/range/domain or model validation. The underlying selector and original fields are stack-rooted across allocations; `check_table_access` retains underlying table policy. Stack top is restored on success/error. This is a DestroyEntry-only AllowedWhenUntainted boundary, awaiting GREEN/verifier proof.
- `src/c_api/c_housing/catalog/storage.rs`: DestroyEntry now uses that method-local reader. Existing exact-variant count validation, eligible-subset mutation and admin event publication after releasing the model borrow remain unchanged. Shared catalog/Admin/pending inputs, cancellation equivalence, payload rooting and no post-dispatch write remain unchanged; no taint clearing.
- `src/lua_api/workarounds/temporary/housing_catalog_state.lua`: exact `DestroyEntry` no-op removed; unrelated seeded policies unchanged. No fallback.

## Tests asserting this spec

All cases require a callable API before exercising it, so invalid-input rejection cannot pass solely because registration is missing. Actual frame `RegisterEvent` listeners capture count/dye observations; assertions after the public call establish callback completion rather than trusting potentially swallowed listener errors. Reentrant trace and completion are checked outside the callback. Rust assertions check all fixture identities/counts, every dye field, exact base metadata, record retention and absence of queued duplicates.

| Cases in `housing_destroy_entry` | Observable contract |
|---|---|
| `false_destroys_one_eligible_instance_before_synchronous_event`, `true_destroys_all_eligible_not_all_stored_in_mixed_stack`, `one_then_all_reduces_both_counts_and_preserves_exempt_instances` | Fresh 5/3 mixed stack, zero returns, state-before-event and inferred subset policy |
| `fully_eligible_stack_retains_zero_record_after_one_or_all`, `full_key_isolates_variant_type_and_record` | Zero record retention, exact compound identity and preserved metadata/dyes |
| `missing_keys_and_explicit_zero_eligibility_are_noops`, `empty_catalog_stays_empty_without_invented_permission` | No eligibility/default data invention, no-op/no-event behavior |
| `malformed_public_selectors_fail_atomically`, `destroy_all_is_required_public_boolean_even_for_missing_keys`, `inconsistent_counts_fail_explicitly_without_clamping_mutating_or_emitting` | Explicit atomic public/model validation, consistent one/all inconsistency diagnostics |
| `listener_reads_and_reenters_same_and_other_variant_before_outer_return`, `public_addon_call_preserves_taint`, `guarded_selector_retains_vm_access_policy_without_mutation_on_rejection`, `destruction_and_events_are_environment_local` | Reentrant timing, ordinary addon calls, public VM guard behavior, environment isolation |

### B72 exact tests — compiled RED, GREEN pending

| Exact test | Authored boundary |
|---|---|
| `secure_secret_numeric_fields_accept_each_and_all_full_key_components` | Each/all actual secret NUM fields, fresh secure one/all mutation, event, unrelated environment |
| `addon_secret_numeric_fields_deny_each_and_all_components_atomically` | Paired numeric denial and atomicity |
| `secure_secret_boolean_true_and_false_select_all_or_one` | Actual BOOL false/true, public/all-secret-field/wrapped-table selectors |
| `addon_secret_boolean_true_and_false_deny_atomically` | Paired BOOL denial and atomicity |
| `secure_secret_table_selector_accepts_exact_key_and_one_event` | Actual TABLE full-key secure one/all mutation |
| `addon_secret_table_selector_denies_atomically` | Paired TABLE denial, two exact variants |
| `original_top_arguments_authenticate_before_selector_or_boolean_type_errors` | Original arg authentication before selector/bool errors, including false arg1/secret arg2 |
| `all_original_fields_and_boolean_authenticate_before_field_domain_or_model_errors` | Every field/arg2 before malformed, missing, negative, out-of-range and model errors |
| `secret_wrapped_guarded_selector_retains_underlying_table_access_policy` | Public underlying-table guard and secure wrapped-table acceptance |
| `secret_table_roots_survive_gc_and_event_identity_does_not_alias_caller` | Pre-call/callback GC, wrapper roots, caller/event/model identity independence |

Shared real fixture helpers inject host wrappers, verify actual secret payloads (NUM/BOOL/TABLE), root underlying tables before allocation, seed a pending-request sentinel directly without placement execution, and assert preserved catalog/dye/base/pending/queue observations. `LuaApi` is imported for host `state()` access. Existing grouped integration module remains; no new target. Original destruction/count/full-key/error/reentry/listener tests retained except the superseded combined conservative-secret test.

### Proof ledger and producer gate

- Baseline clean revision `5afd73d498340952f0e2a2c59ae1a718f92d5b6a`; model/query/event producer and cached consumer inspection only.
- Parent-reported actual RED at input/tests `f94063616`: 15 cases, 2 PASS / 13 FAIL from the existing Lua no-op; build compiled successfully. Artifacts `batch21-red-build/run*` were not independently inspected by this implementer. Parent filter: `housing_destroy_entry::` in existing target `integration`.
- `rustfmt --edition 2024 --config skip_children=true src/c_api/c_housing/catalog/storage.rs src/c_api/c_housing/catalog/queries.rs`: exit 0 on producer files; formatting only. Subsequent module-doc wording change does not affect formatting.
- Historical producer checkpoint: added with no local build/check/test execution. Superseded by bounded independent acceptance below; requirement checkboxes do not imply native parity.
- `rustfmt --edition 2024 --config skip_children=true tests/housing_destroy_entry.rs`: exit 0 on the new test file; formatting only, not compilation or behavioral proof. Later test edits invalidate formatting scope.
- Producer slice authorizes formatting and commit only; no builds, checks, delegation or push. Existing tests, source accounting, other specs and vendor/cache definitions unchanged.

Historical B72 inputs-only checkpoint (`1b0e7ab99` plus `0f7f47297`): producer-side formatting was not compilation; historical 15-case proof did not cover the new requirements.

### B72 implementation and accepted compiled RED — 2026-10-03

Producer `7f7d0fe8a` follows accepted actual RED at `0f7f47297aa9309cb1ae220c4e33b7f5660916a7`, with inputs `1b0e7ab99` + `0f7f47297`: **24 tests, 14 retained PASS / 10 new FAIL**, harness time **7.33s**. Saved build result records **261.349678s**, exit **0**, zero compiler diagnostics; run wrapper records exit **101**, **7.364796727s** elapsed. Artifacts: `/tmp/patch-12.0.5-batch72-red-build-result.json` and `/tmp/patch-12.0.5-batch72-red-run.{json,stdout,stderr}`. Build/run JSON identify the same integration executable hash. Provenance is dirty-combined; protected unowned source was neither touched nor inspected in this audit.

The ten new cases fail at conservative secret rejection or authentication-precedence boundaries. They do not demonstrate ten downstream GC/event/atomicity failures: secure secret calls fail before those later assertions. Source inspection of only `storage.rs` and `destroy_input.rs` establishes the narrow producer wiring/order, not successful execution. **GREEN and independent verifier remain pending.** Existing storage/event/count/cancel-equivalence behavior is retained, not newly accepted by this producer.

Exact rows **281/282 remain audit-pending**; current **177 pending / 164 bounded / 14 partial / 7 metadata**, 362 IDs and 77 capabilities remain unchanged. No approval for broad-row, native, all-profile, full-placement, whole-domain or whole-goal credit. Cached full-variant declaration and BOOL true-all/false-one wording ground the call shape; eligible mixed-stack, defaults/no-op, errors and synchronous events remain explicit simulator inferences. Historical 15-case old-fixture/conservative secure-secret rejection proof remains historical and is superseded only after later GREEN acceptance, not by producer presence.

### Historical reconciled bounded proof — 2026-10-01

[Independent report](/tmp/patch-12.0.5-housing-destruction-independent-proof.md) accepts producer `67b44f2c36c12c86cc1e10c334d010041239e7c9` and tests `f94063616c0490bae295e2eeb3936910a535138d`: saved **15 destruction + 11 storage + 24 catalog = 50 PASS**, independently inspected, not rerun. Fresh default `cargo fmt --check` and `cargo check` both exit **0** at the unchanged producer snapshot. These gates cover default cumulative retail features, not historical 12.0.5-only or all-profile execution. Actual RED was **2 PASS / 13 FAIL**, not thirteen independently reached downstream boundaries.

| Capability | Bounded proof / remaining limit |
|---|---|
| Full variant selector and required boolean | Behavioral PASS: exact record/type/variant isolation, zero returns, atomic malformed/public/model rejection |
| Eligible one/all mutation and missing/zero no-op | Behavioral PASS: mixed 5/3 stack to 4/2 or 2/0, exempt instances and metadata retained; subset/no-op/consistency policies inferred, not native all-stack proof |
| Mutation before synchronous callback and nested deletion | Behavioral PASS: one full-ID event per change, reentrant reads/deletions, no queued duplicate or outer overwrite; native timing/coalescing unknown |
| Taint, secret inputs and secured tables | Historical Behavioral PASS for ordinary addon taint and real host guards; historical conservative secure/tainted secret rejection belongs to the old fixture; supersession awaits later B72 GREEN acceptance and grants no current secret-acceptance credit |
| Publisher rooting and error cleanup | Source inspection only; no forced-GC or dispatch-error behavioral proof |

Parent `batch21-green-startup-run.json` records exit **0**, stdout **[]**, 4.2461s; saved parent evidence, **not independent startup proof**. Exact [coverage](../../data/patch-api/sources/12.0.5-page-coverage.json) rows `global api-C_HousingCatalog-DestroyEntry-281`/`-282` alone link bounded selector rename/full variant-argument coverage. Their audit-pending classification is retained; all **362 IDs**, source hash and unrelated rows/counts remain **308 pending / 40 bounded / 14 partial**. No native/all-profile or whole-row/page closure.

## Known gaps (current cycle)

- [x] B72 actual compiled RED before the method-local producer change accepted: 14 PASS / 10 FAIL; not GREEN acceptance.
- [ ] B72 GREEN secure secret acceptance/addon denial, ordering, atomicity and rooting proof (main-owned).
- [ ] Native mixed-stack eligibility, fixed-five UI/batch meaning, invalid-input errors, secret access and event timing remain unknown.

Historical 15-case RED and GREEN above remain explicit historical evidence, not B72 completion.

## Out of scope

Native instance policy/probes, storage-limit aggregates, search refresh/filtering, placement, full entry DTOs, `CanDestroyEntry`, legacy argument compatibility, popup behavior, asynchronous/coalesced native events and whole source-row/page or all-profile completion. Those require separate evidence. B72 security production scope is `DestroyEntry` alone, not shared catalog/Admin/pending inputs, conservative guards or security-global behavior. This audit changes only the four authorized documentation files.
