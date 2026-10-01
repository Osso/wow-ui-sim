# Housing storage entry updated input

Bounded 12.0.5 event slice: explicit simulator input `A_Admin.SetHousingCatalogVariantStoredCount(entryVariantID, numStored)` updates an existing [catalog variant](housing-catalog-variants.md). This is not a native API or a `DestroyEntry` implementation. [Event dispatch](../event-system.md) supplies the existing synchronous admin pattern. Producer follows parent-observed compiled prerequisite RED; GREEN remains parent-owned.

## What it must do

- [ ] Select an existing variant by the complete `(recordID, entryType, variantIdentifier)` key. Change only `num_stored`; preserve independent `destroyable_instance_count`, every dye field, base entries and other variants, including another entry type with the same record/variant identifiers. Use existing Rust catalog records; no fake records, new model types or production seeds.
- [ ] Changed storage, including an explicit transition to zero, dispatches a real `HOUSING_STORAGE_ENTRY_UPDATED` frame listener before input returns. Emit exactly one argument, a full `entryVariantID` table containing the three identity fields. Listener queries observe changed storage and untouched destroyable/dye data. Do not queue a duplicate storage event.
- [ ] Release model borrows before synchronous callbacks. Reentrant reads and second input calls on both the same and another variant finish inside the listener; nested events observe their own changed state. Outer completion must not overwrite nested changes.
- [ ] **Inferred edge-only simulator policy:** repeating current count succeeds without mutation/event. Changed and unchanged successful calls return zero Lua values.
- [ ] **Explicit simulator error policy, not native validation parity:** unknown full keys and malformed selectors/counts fail with a nonempty error, without mutation, insertion or event. Selector must be an accessible public table with three nonnegative integer identity fields representable by `i32`. Count must be an ordinary, finite, nonnegative integer representable by the existing `i32` storage field. No string coercion, truncation, fake-record creation or alternate-key lookup.
- [ ] Ordinary public inputs work from tainted addon callers without clearing caller taint. Secret whole selectors, nested identity fields and counts reject safely in secure and tainted callers, without unwrapping/declassification, mutation or event. This conservative simulator boundary is not native secret-access parity.
- [ ] State changes and listeners remain local to their `WowLuaEnv`.

### Cached grounding and inference boundary

Inspected 2026-10-01: `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/HousingCatalogUIDocumentation.lua:481–490` declares `HOUSING_STORAGE_ENTRY_UPDATED`, `UniqueEvent = true`, and one nonnil `entryVariantID: HousingCatalogEntryVariantID`. **It does not declare `SynchronousEvent = true`.** The inspected PTR declaration has the same distinction. Synchronous dispatch above is an implementation-inferred simulator admin-producer policy, not a user-requested or observed native event-timing guarantee.

`HousingCatalogConstantsDocumentation.lua:86–96` defines the compound variant identifier with `recordID`, `entryType` and `variantIdentifier`. Retained [12.0.5 register](../../data/patch-api/sources/12.0.5-register.json) rows `events-HOUSING_STORAGE_ENTRY_UPDATED-555` / `-556` rename the first argument from `entryID` to `entryVariantID` and change its type from `HousingCatalogEntryID` to `HousingCatalogEntryVariantID`. These rows establish payload shape, not native emission conditions, uniqueness/coalescing behavior or destruction policy. Cached provenance limits are recorded in the [catalog contract](housing-catalog-variants.md#cached-declaration-provenance-2026-10-01).

The admin name and explicit count input define the simulator input surface. Mutation-before-callback ordering, synchronous timing, change-only emission, error/no-op rules and conservative secret rejection are implementation-inferred simulator policies, not native behavior claims. No fabricated automatic native destroy transition is inferred from this event declaration.

## How it works

- [Catalog identity and snapshot contract](housing-catalog-variants.md).
- [Independent destroyable-count contract](housing-destroyable-count.md).
- [Existing event subsystem](../event-system.md).
- [Party admin transition pattern](party-connection.md) and [quest confirmation pattern](quest-accept-confirmation.md).

## Implementation inventory

- `tests/housing_storage_entry_updated.rs`: eleven input-only behavioral fixtures using existing Rust catalog types, actual `RegisterEvent` listeners and unchanged public catalog queries. Automatically discovered by the existing grouped `integration` harness; no Cargo target added.
- `src/c_api/c_housing/catalog.rs`: unchanged existing model types; wires shared guards and storage producer.
- `src/c_api/c_housing/catalog/input.rs`: catalog selector/access and public integer guards extracted without changing query policy; storage input adds nonnegative validation before mutation.
- `src/c_api/c_housing/catalog/queries.rs`: existing snapshots/count reads reuse extracted guards.
- `src/c_api/c_housing/catalog/storage.rs`: updates only `num_stored` on the exact existing variant, releases the borrow, then synchronously publishes a rooted full-ID table on changes. No event queue or post-callback overwrite.
- `src/c_api/c_housing/catalog/snapshot.rs`: existing full-ID serializer reused; table stays stack-rooted across field allocation and dispatch, stack top restored on dispatch success/error.
- `src/lua_api/globals/admin_party_target_helpers.rs`: reference pattern releases state borrow before `dispatch_event_now`; unchanged.
- `src/lua_api/globals/admin_quests.rs`: reference explicit confirmation input; unchanged.
- `src/lua_api/globals/admin.rs`: registers the producer unconditionally alongside the existing unconditional catalog surface; no new profile gates, taint clearing or alternate producer path.

## Tests asserting this spec

All eleven cases first require the setter to be callable. Invalid-input `pcall` rejection cannot accidentally pass because registration is missing. Listener observations are captured and asserted after dispatch; reentrant completion is also asserted outside the callback, so swallowed callback errors cannot count as success. Rust checks exact record counts, storage values, independent destroyable counts, all dye fields, unchanged base metadata and absence of a queued duplicate event.

| Case in `housing_storage_entry_updated` | Observable contract |
|---|---|
| `changed_storage_publishes_one_full_id_after_state_before_return` | Exact one-argument full ID, changed state inside listener, decrease to zero, zero returns |
| `full_key_distinguishes_variant_and_entry_type` | Distinct variant and type transitions, independent destroyability/dyes |
| `identical_count_is_inferred_noop_with_zero_returns` | Initial/repeated same-count no event, zero returns |
| `unknown_full_keys_error_without_inserting_or_emitting` | Missing record/variant/type errors, no fake variant |
| `malformed_selectors_error_without_mutation_or_event` | Missing/wrong/fractional fields and non-table selectors reject |
| `malformed_negative_fractional_and_nonfinite_counts_error_atomically` | Missing/wrong/negative/fractional/nonfinite/out-of-range counts reject |
| `listener_can_read_and_reenter_input_before_outer_return` | Same-key and other-key nested inputs dispatch/read synchronously without retained borrow |
| `public_addon_input_preserves_caller_taint` | Public addon-created selector/count succeeds without clearing taint |
| `secret_selectors_and_counts_reject_secure_and_tainted_callers` | Host-wrapped secrets reject at whole-selector, each nested field and count boundary |
| `storage_input_and_event_are_environment_local` | Populated environments retain independent storage/listeners |
| `empty_catalog_rejects_unknown_key_without_fake_records` | Default empty model stays empty after explicit rejection |

### Proof ledger

- Initial clean checkout: `3068e48d27fa104e071397be58688b0e4f7f34e3`; inspected model already includes independent destroyable count. Existing count/variant source, fixtures, contracts and accounting remain untouched.
- `rustfmt --edition 2024 --config skip_children=true tests/housing_storage_entry_updated.rs`: exit 0 on the new test file; formatting only, not compilation or behavioral proof. Later test edits invalidate that formatting scope.
- Parent RED at tests `19dfb2674f5763815a6780fb273107b956cc63e6`: `cargo test --test integration --no-run --message-format=json`, exit 0 in 62.74s (`/tmp/patch-12.0.5-batch20-red-build-result.json`, companion build log/json). Saved bounded binary command with `housing_storage_entry_updated:: --nocapture --test-threads=1` exits 101, 0/11 PASS; every failure reaches the explicit missing-setter prerequisite, not eleven independently exercised behavior boundaries (`/tmp/patch-12.0.5-batch20-red-run.log` / `.json`). Inspected before producer implementation; not rerun.
- `rustfmt --edition 2024 --config skip_children=true` on the six changed Rust producer/guard/registration files: exit 0; formatting only, not compilation or behavioral proof.
- Producer source implements the contract above; no builds, checks or behavioral execution in this producer task. Format/commit only. Parent owns targeted GREEN and acceptance; no assertion marked passing.

## Known gaps (current cycle)

- [ ] Parent targeted GREEN and acceptance remain pending after producer registration; saved prerequisite RED is established, not behavioral proof of the implemented producer.
- [ ] Native emission conditions, coalescing/uniqueness, timing, invalid-input and secret-access behavior remain unknown. No native, all-profile, source-row or whole-page completion credit.

## Out of scope

`DestroyEntry`, automatic destruction/native emission conditions, placement, dyes mutation, destroyable-count/variant-accounting changes, filters/search refresh and all other housing producers. No edits to current count/variant fixtures or accounting owned by 184. Producer task changes only catalog guard sharing, exact-variant storage input/registration, serializer visibility and this spec: no builds, checks, delegation, push or wiki/accounting updates. Event declaration availability alone does not establish implementation on earlier profiles; test gating follows the existing 12.0.5 catalog fixtures, without adding a production feature gate.
