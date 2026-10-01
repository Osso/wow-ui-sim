# Housing catalog destroyable count

Input/tests-first slice for `C_HousingCatalog.GetDestroyableInstanceCount(entryVariantID)`, extending the [compound catalog model](housing-catalog-variants.md). Retained [12.0.5 changes](../../data/patch-api/sources/12.0.5-api-changes.txt): `global api-C_HousingCatalog-GetDestroyableInstanceCount-288` renames `entryID` to `entryVariantID`; `global api-C_HousingCatalog-GetDestroyableInstanceCount-289` changes its type to `HousingCatalogEntryVariantID`. Neither row establishes count policy. No source-row completion credit.

## What it must do

- [ ] Each explicitly supplied variant record carries an independent `destroyable_instance_count`. The full `(recordID, entryType, variantIdentifier)` selects it; two variants of one record can have distinct counts. It is not computed from `num_stored`.
- [ ] **Simulator inference:** empty/default environments and missing full keys return zero, without fake records, legacy seeds, numeric selectors or alternate variant lookup. Wrong entry type or variant identifier must not alias a populated key.
- [ ] Changing only stored count leaves the explicit destroyable count unchanged. Inputs belong to one environment and cannot populate another.
- [ ] The query returns exactly one ordinary Lua number. Explicit fixture counts are 1 and 4, independently supplied alongside stored counts 3 and 5; these are test inputs, not production defaults.
- [ ] Ordinary public selectors work from secure and tainted addon callers without clearing taint. Nested secret fields reject in both contexts without unwrapping/declassification, consistent with the prior catalog producer's conservative simulator limit. Host-installed secured tables retain VM access checks; secure lookup succeeds, tainted lookup rejects, caller taint and source data remain intact. Native secure-secret access, coercion and invalid-input behavior remain unverified.

### Cached grounding

Read 2026-10-01: `/home/osso/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/HousingCatalogUIDocumentation.lua:237–250`. Declaration: non-nil `entryVariantID: HousingCatalogEntryVariantID`, `SecretArguments = "AllowedWhenUntainted"`, one non-nil `destroyableInstanceCount: number`. Wording: "Returns the number of instances that can be to be destroyed in storage; These instances count towards the max storage limit". This describes destroyable storage instances, not every stored instance. Public-number publication is this bounded fixture contract; native return secrecy is not established by the number type alone.

The [variant spec's cached provenance](housing-catalog-variants.md#cached-declaration-provenance-2026-10-01) identifies retail 12.1.0.69933, not a native historical 12.0.5 probe. AllowedWhenUntainted does not establish the conservative rejection policy as native parity.

## How it works

- [Catalog identity/security model](housing-catalog-variants.md).
- [Housing audit ownership](../wiki/investigations/patch-12-0-5-api-audit.md#Housing catalog variant producer — pending GREEN).

## Implementation inventory

- `src/c_api/c_housing/catalog.rs`: adds explicit count input on `HousingCatalogVariantRecord`; default catalog remains empty.
- `tests/housing_catalog_variants.rs`: existing fixture explicitly supplies counts; nested `destroyable_count` module contains ten expectations in the existing grouped integration target.
- `src/c_api/c_housing/catalog/queries.rs` and `src/lua_api/workarounds/temporary/housing_catalog_state.lua`: deliberately unchanged. The existing query still reads legacy seeded `numStored` and ignores `entryType`; the new input is not consumed yet.

## Tests asserting this spec

All ten cases are **unrun**, neither RED nor GREEN claimed. Parent must compile/run these against the unchanged provider and observe actual behavioral RED before production query/seed edits. Exact filter: `housing_catalog_variants::destroyable_count::`; existing target: `integration`. No new Cargo target.

| Case in `destroyable_count` | Observable contract |
|---|---|
| `default_returns_zero`, `missing_record_returns_zero` | Empty/missing inferred zero |
| `variants_have_distinct_explicit_counts`, `wrong_type_or_variant_does_not_alias` | Independent exact compound-key counts |
| `changing_storage_does_not_change_destroyable_count`, `fresh_environments_are_isolated` | Storage independence and environment isolation |
| `returns_one_ordinary_number`, `public_selector_preserves_addon_taint` | One usable number; ordinary tainted caller |
| `nested_secrets_reject_without_declassification`, `guarded_selector_preserves_vm_access_policy` | Conservative secret rejection and host VM guard |

### Proof ledger

| Command / scope | Result | Invalidation |
|---|---|---|
| Initial `git status --short` / `git rev-parse HEAD` | Base `157d15cef796ccd11b3e6c04f80b57d0700728cb`; existing variants spec and audit wiki dirty, excluded from this slice | Concurrent revisions not behavioral proof |
| `rustfmt --edition 2024 --config skip_children=true src/c_api/c_housing/catalog.rs tests/housing_catalog_variants.rs` | Formatting only; exit recorded by task report | Later changes to either file invalidate formatting scope |
| Filter `housing_catalog_variants::destroyable_count::` in `integration` | Not run; parent RED pending | No build/test proof exists for these inputs |

## Known gaps (current cycle)

- [ ] Parent actual RED, subsequent producer replacement, GREEN and acceptance remain pending. Existing providers are not evidence for this contract.
- [ ] Secure secret access and native count policy remain unknown; no declassification or native-parity claim.

## Out of scope

- Destroy mutation, storage events and placement selection are separate slices, not prerequisites for this read query. Their future implementation may use documented simulator inferences; native probes are not a permanent implementation gate, including for Forever.
- Aggregate/search counts, filters, catalog metadata, storefront/cart and legacy wrapper migration remain unrelated owners.
- Builds, checks, delegation, push, source/accounting updates and concurrent docs/verifier outputs are excluded from this input-only task. The concurrently edited variants spec is untouched; this separate contract owns the new count requirements.
