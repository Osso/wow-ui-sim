# Housing catalog explicit aggregates

Bounded 12.0.5 input/test contract for `HousingCatalogEntryInfo.totalNumStored` and `totalNumPlaced`, owned by `src/c_api/c_housing/catalog.rs`. Exact source rows: `structures-HousingCatalogEntryInfo-650` (`+ totalNumStored`) and `structures-HousingCatalogEntryInfo-651` (`+ totalNumPlaced`) in [the source register](../../data/patch-api/sources/12.0.5-register.json). No row-completion or native-parity claim. See [C API placement rules](../../AGENTS.md#c-api-boundary) for subsystem placement.

## What it must do

- [x] `GetCatalogEntryInfo`, `GetCatalogEntryInfoByItem`, and `GetCatalogEntryInfoByRecordID` publish the selected base record's explicitly supplied `total_num_stored` and `total_num_placed` as Lua numbers named `totalNumStored` and `totalNumPlaced`.
- [x] Counts are nonnegative `Option<u32>` inputs; supplied values above the signed 32-bit range remain exact. No native maximum is inferred from this simulator representation.
- [x] **Simulator inference:** `Some(0)` publishes numeric zero; each `None` independently leaves its Lua field nil. Missing explicit data is a simulator gap, not native contract parity or a native default.
- [x] Never derive totals from an incomplete variant map, pending placement, room membership, spent budgets or other unrelated housing state. Variant changes/removal do not change explicit totals; aggregate changes do not mutate variants or pending state.
- [x] Every query returns independent snapshots: Lua mutation cannot change other results or input; later input mutation cannot change earlier results. Missing base records return nil even when variants survive. Environments remain independent.

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
- `tests/housing_catalog_aggregates.rs`: thirteen grouped behavioral fixtures, gated by `retail-12-0-5`; twelve prior fixtures plus one independently accepted saved raw-output coverage extension.
- Existing construction sites in `tests/housing_catalog_base_lookups.rs` (two literals), `tests/housing_catalog_variants.rs`, `tests/housing_storage_entry_updated.rs`, and `tests/housing_destroy_entry.rs` (one literal each): both new inputs are `None`; all prior concrete fixture data preserved.
- Existing `build.rs` discovery includes top-level Rust test modules in `tests/integration.rs`; `autotests = false` remains unchanged. No new Cargo target or harness edit.

## Tests asserting this spec

Exact filter: `housing_catalog_aggregates::`; target: `integration`.

| Fixtures | Required behavior | Proof level |
| --- | --- | --- |
| `entry_info_publishes_explicit_aggregates`, `by_item_publishes_explicit_aggregates`, `by_record_id_publishes_explicit_aggregates` | Each selector publishes 37 stored / 11 placed independently of variants containing 3 and 5 stored | Saved GREEN; independently inspected, not rerun |
| `explicit_zero_is_not_missing_inferred_policy`, `missing_aggregates_are_nil_not_variant_sums_inferred_gap`, `each_aggregate_can_be_missing_independently_inferred_gap`, `nonnegative_counts_preserve_values_above_signed_range` | Zero/nil distinction, no variant-sum default, independent omissions, unsigned count range | Saved GREEN; independently inspected, not rerun |
| `variant_and_unrelated_housing_mutation_do_not_change_aggregates`, `aggregate_input_mutation_does_not_change_variants_or_pending_state` | No coupling in either direction; variant identities/counts stay distinct | Saved GREEN; independently inspected, not rerun |
| `snapshots_are_independent_across_lua_and_input_mutation` | All three selectors produce independent snapshots across Lua changes, input changes and GC | Saved GREEN; independently inspected, not rerun |
| `missing_base_record_is_nil_even_with_surviving_variants`, `aggregates_are_environment_local` | Missing-record nil and isolated explicit input | Saved GREEN; independently inspected, not rerun |

### Reconciled batch24 bounded proof — 2026-10-01

Inputs `fe874979b`; producer `72795fbfa`. [Independent proof](/tmp/patch-12.0.5-housing-aggregates-independent-proof.md) inspected saved compiler/runtime artifacts, not rerun behavior. Compiled RED: build exit 0, **2 PASS / 10 FAIL**. All ten failures stop at `explicit totalNumStored mismatch`; they do not individually isolate placed-field failures. Missing-aggregate and missing-base controls pass.

Saved GREEN: **12 aggregate + 14 base + 24 variants/count + 11 storage + 15 destruction = 76 PASS** across five nonempty selections. Parent normal startup exits **0**, JSON `[]`, zero unique errors/occurrences; saved evidence, not independent execution or nonempty-wrapper proof. Runtime ledger: `/tmp/patch-12.0.5-batch24-green-runs.json`; startup: `/tmp/patch-12.0.5-batch24-green-startup-run.json`.

Fresh independent `cargo fmt --check` and `cargo check` each exit **0** at docs `2973774c5`, with relevant Rust/config identical to producer; no warnings/errors. Gate ledger: `/tmp/patch-12.0.5-batch24-independent-checks.json`. Source audit covers shared serializer, unchanged selector guards/registration, rooting and preserved constructor data; not native aggregate secrecy or all-profile execution.

### Reconciled batch27 raw output proof — 2026-10-01

Exact [source register](../../data/patch-api/sources/12.0.5-register.json) rows `structures-HousingCatalogEntryInfo-653` through `-657` remove, respectively, `showQuantity`, `quantity`, `numPlaced`, `customizations`, and `dyeIDs`. Already-bounded row `-652` removes `entryID`, retained here as a control.

- [x] `raw_entry_snapshots_omit_removed_fields_after_legacy_injection` exercises all three raw getters on the populated base fixture with `Some(37)` stored and `Some(11)` placed. It asserts concrete current identity, item, name, trophy flag and aggregate fields, plus `rawget == nil` for all five removed names and `entryID`; not absence on a missing record.
- [x] Injecting legacy fields into each returned Lua snapshot leaves subsequent fresh results from all three getters populated and alias-free, while the injected snapshots retain their local fields.

Coverage extension for existing, unchanged production behavior; no fabricated RED or new producer/input behavior. Exact parent GREEN filter `housing_catalog_aggregates::` selects **12 prior + 1 bounded extension = 13 tests** in target `integration`. [Independent batch27 appendix](/tmp/patch-12.0.5-housing-category-search-independent-proof.md#separately-authorized-batch27-appendix--tests-only-9cdb13a59) accepts saved **13 PASS / 0 FAIL** at tests-only `9cdb13a59`, including the new fixture. Saved compilation exits **0**; integration SHA256 matches runtime metadata. Fresh independent fmt exits **0**; production unchanged, so batch26 independent check remains valid. No independent build/test/startup rerun. Prior batch24 proof does not cover this new fixture; batch26 **78 PASS/startup 0 []** is a separate execution, not a combined 91-test run. Exact rows **653–657** gain bounded raw output absence/injection-freshness coverage; row652 remains a control. Input removals645/646 receive no credit.

Boundary is the raw C API in `WowLuaEnv::new()`, not the cached deprecated wrapper, which deliberately adds backward-compatible aliases and remains untouched. Extra unknown selector fields being ignored is distinct from output-field absence; this test imposes no new selector type-shape requirement. No complete DTO, native or all-profile claim.

## Known gaps (current cycle)

- [ ] Cached required native numbers remain a missing-data simulator gap when input is `None`; nil is not a native default. Native maximum/default/secrecy unverified.
- [ ] Automatic synchronization, variant derivation, complete DTO and successful nonempty deprecated-wrapper compatibility remain absent/unproven; `remainingRedeemable` is not modeled here.
- [ ] Exact rows 650/651 retain audit-pending with bounded explicit-field proof links, not whole-row closure. Audit **IN PROGRESS**.

## Out of scope

Native nil/default/security semantics, complete `HousingCatalogEntryInfo`, redeemable data, deprecated-wrapper parity, storage/placement aggregate recomputation or synchronization, variant schema changes, new Cargo targets, all-profile acceptance, PLAN and edits to `housing-catalog-variants.md` are excluded.
