# Housing catalog destroyable count

Bounded query slice for `C_HousingCatalog.GetDestroyableInstanceCount(entryVariantID)`, extending the [compound catalog model](housing-catalog-variants.md). Retained [12.0.5 changes](../../data/patch-api/sources/12.0.5-api-changes.txt): `global api-C_HousingCatalog-GetDestroyableInstanceCount-288` renames `entryID` to `entryVariantID`; `global api-C_HousingCatalog-GetDestroyableInstanceCount-289` changes its type to `HousingCatalogEntryVariantID`. Neither row establishes count policy. Only these two exact parameter rows receive bounded coverage; no whole-row/page completion.

## What it must do

- [x] Each explicitly supplied variant record carries an independent `destroyable_instance_count`. The full `(recordID, entryType, variantIdentifier)` selects it; two variants of one record can have distinct counts. It is not computed from `num_stored`.
- [x] **Simulator inference:** empty/default environments and missing full keys return zero, without fake records, legacy seeds, numeric selectors or alternate variant lookup. Wrong entry type or variant identifier must not alias a populated key.
- [x] Changing only stored count leaves the explicit destroyable count unchanged. Inputs belong to one environment and cannot populate another.
- [x] The query returns exactly one ordinary Lua number. Explicit fixture counts are 1 and 4, independently supplied alongside stored counts 3 and 5; these are test inputs, not production defaults.
- [x] Ordinary public selectors work from secure and tainted addon callers without clearing taint. Nested secret fields reject in both contexts without unwrapping/declassification, consistent with the prior catalog producer's conservative simulator limit. Host-installed secured tables retain VM access checks; secure lookup succeeds, tainted lookup rejects, caller taint and source data remain intact. Native secure-secret access, coercion and invalid-input behavior remain unverified.

### Cached grounding

Read 2026-10-01: `/home/osso/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/HousingCatalogUIDocumentation.lua:237–250`. Declaration: non-nil `entryVariantID: HousingCatalogEntryVariantID`, `SecretArguments = "AllowedWhenUntainted"`, one non-nil `destroyableInstanceCount: number`. Wording: "Returns the number of instances that can be to be destroyed in storage; These instances count towards the max storage limit". This describes destroyable storage instances, not every stored instance. Public-number publication is this bounded fixture contract; native return secrecy is not established by the number type alone.

The [variant spec's cached provenance](housing-catalog-variants.md#cached-declaration-provenance-2026-10-01) identifies retail 12.1.0.69933, not a native historical 12.0.5 probe. AllowedWhenUntainted does not establish the conservative rejection policy as native parity.

## How it works

- [Catalog identity/security model](housing-catalog-variants.md).
- [Housing audit ownership](../wiki/investigations/patch-12-0-5-api-audit.md#housing-destroyable-count--bounded-independent-pass).

## Implementation inventory

- `src/c_api/c_housing/catalog.rs`: adds explicit count input on `HousingCatalogVariantRecord`; default catalog remains empty.
- `tests/housing_catalog_variants.rs`: existing fixture explicitly supplies counts; nested `destroyable_count` module contains ten expectations in the existing grouped integration target.
- `src/c_api/c_housing/catalog/queries.rs`: unconditional query registration consumes the explicit count through the exact full variant key. Shared selector/table-access and integer parsers reject secret fields without unwrapping or clearing caller taint; missing keys publish one ordinary zero (simulator inference).
- `src/lua_api/workarounds/temporary/housing_catalog_state.lua`: only the overlapping `GetDestroyableInstanceCount` seeded producer is removed. Other seeds and mutation/placement/event/cart/filter providers are unchanged.

## Tests asserting this spec

Parent observed actual RED at input revision `6cd5e3812a922d4112bbfeffd0fe57e1e8ed834e`: ten cases, five PASS and five FAIL, before this producer replacement. Failures: default zero, fresh-environment isolation, distinct explicit counts, wrong-type/variant aliasing and nested-secret rejection. The five legacy-provider passes do not establish the new producer's behavior. Exact filter: `housing_catalog_variants::destroyable_count::`; existing target: `integration`. No new Cargo target. Producer GREEN and bounded independent acceptance recorded below.

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
| Parent `cargo test --test integration --no-run --message-format=json` at `6cd5e3812` | Exit 0, 135.19s; `/tmp/patch-12.0.5-batch19-red-build-result.json` | Pre-producer compile only |
| Parent `timeout 90 <integration binary> housing_catalog_variants::destroyable_count:: --nocapture --test-threads=1` at `6cd5e3812` | Exit 101, 5 PASS / 5 FAIL, 1.49s; `/tmp/patch-12.0.5-batch19-red-run.log` and `.json`; binary SHA-256 `2c06f961f453f12aae334f8c27d9c17cf615c1efbed3662fe663e674a80a7f42` | Actual prerequisite RED, not producer GREEN |
| Producer formatting: `rustfmt --edition 2024 --config skip_children=true src/c_api/c_housing/catalog/queries.rs` | Formatting only; result recorded in task report | Later query edits invalidate formatting scope |

## Known gaps (current cycle)

- [ ] Native count policy, native secure-secret parity and all-profile execution remain unverified; bounded acceptance does not close the whole source row/page.
- [ ] Secure secret access and native count policy remain unknown; no declassification or native-parity claim.

## Out of scope

- Destroy mutation, storage events and placement selection are separate slices, not prerequisites for this read query. Their future implementation may use documented simulator inferences; native probes are not a permanent implementation gate, including for Forever.
- Aggregate/search counts, filters, catalog metadata, storefront/cart and legacy wrapper migration remain unrelated owners.
- Builds, checks, delegation, push and concurrent storage-event changes are excluded from this docs reconciliation. The variants spec is untouched; this separate contract owns the count requirements and proof boundary.

## Reconciled bounded proof — 2026-10-01

Independent report: `/tmp/patch-12.0.5-housing-destroyable-count-independent-proof.md`; hash ledger: `/tmp/patch-12.0.5-housing-destroyable-count-independent-hashes.json`. Producer `3068e48d27fa104e071397be58688b0e4f7f34e3`: saved **10 count + 14 variant controls PASS**, exit 0; independent default fmt/check **exit 0**, no warnings/errors. Relevant hashes rechecked unchanged during reconciliation; later storage-event work excluded. No builds or test reruns here.

Parent `batch19-green-startup-run.json`: exit 0, `[]`, zero errors, 4.192s; not independent startup proof. Secure nested secrets conservatively reject even for secure callers: native AllowedWhenUntainted parity remains missing. Default zero and native count policy remain inference/unverified; no native or all-profile acceptance.

| File | SHA-256 |
|---|---|
| `src/c_api/c_housing/catalog.rs` | `4edb0088aca19484fc513c4dd6dee1b3844245203d1e6f2757d8fbd781c0484f` |
| `src/c_api/c_housing/catalog/queries.rs` | `cccabbd099b8111aaa35dbde7a86b4cfa3195b82768db5ac56a108244816d4f2` |
| `tests/housing_catalog_variants.rs` | `99a61a04d24ea19dbe543c9689384ce88ccba776e79cf4d3e16f9a70a9f6d2ad` |
| `src/lua_api/workarounds/temporary/housing_catalog_state.lua` | `39269d2df10880f761f5482c75a080e7ccd2dca1eb36bb487331bc57c8b42cd2` |

GREEN integration binary SHA-256 `6fa01bfe3c95c9c51a7b05fb9fea2ec13835df9a886d60c9535b220715b36170`. Historical RED binary hash remains in ledger above.
