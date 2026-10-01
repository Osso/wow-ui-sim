# Housing catalog explicit aggregates

Bounded 12.0.5 input/test contract for `HousingCatalogEntryInfo.totalNumStored` and `totalNumPlaced`, owned by `src/c_api/c_housing/catalog.rs`. Exact source rows: `structures-HousingCatalogEntryInfo-650` (`+ totalNumStored`) and `structures-HousingCatalogEntryInfo-651` (`+ totalNumPlaced`) in [the source register](../../data/patch-api/sources/12.0.5-register.json). No row-completion or native-parity claim. See [C API placement rules](../../AGENTS.md#c-api-boundary) for subsystem placement.

## What it must do

- [ ] `GetCatalogEntryInfo`, `GetCatalogEntryInfoByItem`, and `GetCatalogEntryInfoByRecordID` publish the selected base record's explicitly supplied `total_num_stored` and `total_num_placed` as Lua numbers named `totalNumStored` and `totalNumPlaced`.
- [ ] Counts are nonnegative `Option<u32>` inputs; supplied values above the signed 32-bit range remain exact. No native maximum is inferred from this simulator representation.
- [ ] **Simulator inference:** `Some(0)` publishes numeric zero; each `None` independently leaves its Lua field nil. Missing explicit data is a simulator gap, not native contract parity or a native default.
- [ ] Never derive totals from an incomplete variant map, pending placement, room membership, spent budgets or other unrelated housing state. Variant changes/removal do not change explicit totals; aggregate changes do not mutate variants or pending state.
- [ ] Every query returns independent snapshots: Lua mutation cannot change other results or input; later input mutation cannot change earlier results. Missing base records return nil even when variants survive. Environments remain independent.

### Declaration evidence and limits

Inspected cached retail `Blizzard_APIDocumentationGenerated/HousingCatalogUIDocumentation.lua:555-557` declares both fields `Type = "number", Nilable = false`. Stored total covers storage across variants and excludes unredeemed instances; placed total covers the player's houses and plots across variants. Required-number declarations do **not** establish defaults for absent simulator input. Leaving that gap nil is explicit simulator policy, not native behavior evidence.

Cached `Blizzard_Deprecated/Mainline/Deprecated_12_0_5.lua:85-87` aliases `quantity = info.totalNumStored` and `numPlaced = info.totalNumPlaced`, but computes `showQuantity` using `info.remainingRedeemable`. This slice does not model or fabricate `remainingRedeemable`, execute successful-record wrapper tests, or claim wrapper parity. Fixtures assert no fabricated redeemable field. Native probes and complete DTO parity remain unverified.

## How it works

- [API runtime context](../wiki/systems/lua-api.md).
- [Existing variant contract](housing-catalog-variants.md): distinct base/variant identities and existing selector behavior; unchanged by this slice.

## Implementation inventory

- `src/c_api/c_housing/catalog.rs`: two optional explicit aggregate inputs.
- `src/c_api/c_housing/catalog/queries.rs`: unchanged shared lookup path for all three getters.
- `src/c_api/c_housing/catalog/snapshot.rs`: shared `push_entry` publishes each supplied unsigned count directly as a Lua number; absent inputs remain absent. No variant or unrelated-state reads.
- `tests/housing_catalog_aggregates.rs`: twelve grouped behavioral fixtures, gated by `retail-12-0-5`.
- Existing construction sites in `tests/housing_catalog_base_lookups.rs` (two literals), `tests/housing_catalog_variants.rs`, `tests/housing_storage_entry_updated.rs`, and `tests/housing_destroy_entry.rs` (one literal each): both new inputs are `None`; all prior concrete fixture data preserved.
- Existing `build.rs` discovery includes top-level Rust test modules in `tests/integration.rs`; `autotests = false` remains unchanged. No new Cargo target or harness edit.

## Tests asserting this spec

Exact filter: `housing_catalog_aggregates::`; target: `integration`.

| Fixtures | Required behavior | Proof level |
| --- | --- | --- |
| `entry_info_publishes_explicit_aggregates`, `by_item_publishes_explicit_aggregates`, `by_record_id_publishes_explicit_aggregates` | Each selector publishes 37 stored / 11 placed independently of variants containing 3 and 5 stored | Written; uncompiled/unrun |
| `explicit_zero_is_not_missing_inferred_policy`, `missing_aggregates_are_nil_not_variant_sums_inferred_gap`, `each_aggregate_can_be_missing_independently_inferred_gap`, `nonnegative_counts_preserve_values_above_signed_range` | Zero/nil distinction, no variant-sum default, independent omissions, unsigned count range | Written; uncompiled/unrun |
| `variant_and_unrelated_housing_mutation_do_not_change_aggregates`, `aggregate_input_mutation_does_not_change_variants_or_pending_state` | No coupling in either direction; variant identities/counts stay distinct | Written; uncompiled/unrun |
| `snapshots_are_independent_across_lua_and_input_mutation` | All three selectors produce independent snapshots across Lua changes, input changes and GC | Written; uncompiled/unrun |
| `missing_base_record_is_nil_even_with_surviving_variants`, `aggregates_are_environment_local` | Missing-record nil and isolated explicit input | Written; uncompiled/unrun |

### Producer checkpoint / proof ledger — 2026-10-01

Parent reports compiled RED at `fe874979b`: build exit 0; twelve tests, two PASS and ten FAIL for missing aggregates. Artifacts: `/tmp/patch-12.0.5-batch24-red-*`. This producer slice does not independently rerun or inspect that behavioral proof; table entries above describe the original input checkpoint, not fresh producer results.

Producer now publishes both optional counts through shared `push_entry`, using direct `u32` to `f64` conversion without signed narrowing. `None` does not write a field; `Some(0)` writes numeric zero. Queries, variants, seed data and wrappers remain unchanged. No build, check, tests, readability, coverage, startup, delegation, push or deployment in this slice. GREEN and acceptance remain parent-owned; requirements remain unchecked pending that proof.

## Known gaps (current cycle)

- [ ] Parent GREEN and verifier on the committed producer revision; no fresh compile or behavioral result claimed here.

## Out of scope

Native nil/default/security semantics, complete `HousingCatalogEntryInfo`, redeemable data, deprecated-wrapper parity, storage/placement aggregate recomputation or synchronization, variant schema changes, serializer/output changes before RED, new Cargo targets, all-profile acceptance, shared coverage registers/wiki/PLAN and edits to `housing-catalog-variants.md` are excluded.
