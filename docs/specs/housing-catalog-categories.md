# Housing catalog category snapshots

Bounded category/subcategory DTO contract for the 12.0.5 rename, backed by explicit inputs in `src/c_api/c_housing/catalog.rs`. Runtime getters read these maps; bounded independent acceptance at producer `bbbacf8f0` is recorded below. Full catalog/native security acceptance remains open. See [C API architecture](../wiki/systems/lua-api.md) for subsystem context.

## What it must do

- [x] `C_HousingCatalog.GetCatalogCategoryInfo(categoryID)` reads only the exact category map key and returns `ID`, `orderIndex`, nullable `name`/`icon`, a fresh `subcategoryIDs` list, and explicit boolean `anyStoredEntries`.
- [x] `C_HousingCatalog.GetCatalogSubcategoryInfo(subcategoryID)` reads only the exact subcategory map key and returns `ID`, `orderIndex`, `parentCategoryID`, nullable `name`/`icon`, and explicit boolean `anyStoredEntries`.
- [x] Both true and false survive unchanged; `anyOwnedEntries` is absent. Neither predicate is inferred from partial catalog entries, variant storage, aggregates, list membership, or another record.
- [x] Defaults expose no category/subcategory records. Missing exact IDs return nil, never another map's record or legacy All/Featured/Decor/Seating/Lighting seeds. **Simulator inference:** nullable documented results do not establish native missing-ID behavior.
- [x] Every query produces a fresh snapshot, including the category's nested list. Lua mutation cannot change model inputs or another snapshot. Host record mutation changes only subsequent snapshots. Environments remain isolated. These are simulator snapshot policies, not native-verified aliasing semantics.
- [x] Public numeric selectors work in secure and tainted calls without changing caller taint. Rooted host-secret numeric selectors reject without unwrapping in both contexts; secret identity survives GC and rejection. **Conservative simulator limitation:** secure secret acceptance is not modeled despite the native declaration below.

### Source and guard evidence

The retained [12.0.5 source register](../../data/patch-api/sources/12.0.5-register.json) records exactly:

| Source row | Delta |
|---|---|
| `structures-HousingCatalogCategoryInfo-643` | `anyOwnedEntries -> anyStoredEntries` |
| `structures-HousingCatalogSubcategoryInfo-659` | `anyOwnedEntries -> anyStoredEntries` |

Cached declaration inspected at `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/HousingCatalogUIDocumentation.lua`: getters at lines 110–122 and 202–214; DTOs at 524–535 and 581–592. This current cache is corroborating shape evidence, not an exact native 12.0.5 execution probe. Both selectors are nonnullable `number`, results nullable `info`, and `SecretArguments = "AllowedWhenUntainted"`. Required DTO fields are nonnullable; `name` is nullable `cstring`, `icon` nullable `textureAtlas`. `subcategoryIDs` is a required numeric table. Category/subcategory predicate descriptions say respectively “True if the player owns anything that falls under this category” and “True if the player owns anything that falls under this subcategory”. These descriptions do not supply a derivation algorithm from partial simulator data.

Existing `src/c_api/c_housing/catalog/input.rs::read_public_integer` rejects anything except public `Val::Num`, explicitly reporting that secret access is not modeled; it never unwraps or clears taint. This contract retains that conservative policy rather than claiming `AllowedWhenUntainted` parity. Tests exercise both secure and tainted rejection with rooted host-secret fixtures, not Lua-created placeholder secrets.

## How it works

- [Lua API architecture](../wiki/systems/lua-api.md)
- [Client profiles](../wiki/systems/client-profiles.md)

## Implementation inventory

- `src/c_api/c_housing/catalog.rs`: empty-default category/subcategory maps; IDs are map keys. Records contain explicit order, optional name/icon, category child IDs or subcategory parent ID, and independent stored predicate.
- `src/c_api/c_housing/catalog/categories.rs`: public integer guards and exact distinct-map lookups; absent records return nil without fallback.
- `src/c_api/c_housing/catalog/snapshot.rs`: fresh stack-rooted category/subcategory DTOs and nested numeric lists; optional name/icon omitted and stored predicates copied explicitly.
- `src/c_api/c_housing/catalog/queries.rs`: both getters registered unconditionally alongside existing catalog methods.
- `tests/housing_catalog_categories.rs`: twelve getter fixtures in the existing generated `integration` target; feature gated to `retail-12-0-5`.
- `tests/housing_catalog_variants.rs`: existing whole-state constructor now uses `..HousingCatalogState::default()` so new maps stay empty; entry/variant fixture inputs unchanged.
- `src/lua_api/workarounds/temporary/housing_catalog_state.lua`: only seeded `GetCatalogCategoryInfo`/`GetCatalogSubcategoryInfo` publishers removed. Seeded category/subcategory search and unrelated storefront/cart behavior remain pending separate work. `GetCategoryInfo` is not the target getter.

## Tests asserting this spec

Parent filter: `housing_catalog_categories::` in the existing `integration` target. No new Cargo target or harness edit.

| Fixtures | Expected behavior | Proof |
|---|---|---|
| `empty_category_ids_do_not_return_legacy_seeds`, `empty_subcategory_ids_do_not_return_legacy_seeds` | Empty IDs 101/102/1001, plus subcategory 1002, return nil | Saved GREEN PASS; independently inspected |
| `category_fields_preserve_explicit_true_and_false_under_new_name`, `subcategory_fields_preserve_explicit_true_and_false_under_new_name`, `optional_fields_are_nullable_without_losing_required_fields` | Exact DTO values, new boolean name, retired field absent, nullable optionals | Saved GREEN PASS; independently inspected |
| `selectors_use_exact_id_and_never_cross_category_maps`, `stored_predicates_are_not_computed_from_partial_variant_storage` | Exact-key separation; explicit predicates independent of positive variant storage | Saved GREEN PASS; independently inspected |
| `returned_tables_and_nested_lists_are_fresh_snapshots`, `record_mutation_changes_new_snapshots_only`, `records_are_isolated_between_environments` | Snapshot/list isolation, mutation lifetime, environment isolation | Saved GREEN PASS; independently inspected |
| `public_numeric_selectors_preserve_caller_taint`, `secret_numeric_selectors_reject_without_unwrapping_or_taint_changes` | Public tainted access and conservative secure/tainted secret rejection | Saved GREEN PASS; independently inspected |

### Producer checkpoint — 2026-10-01

Input commit `63b53dfe5`; parent reports actual compiled `batch25-red-*`: twelve selected, two PASS / ten FAIL, covering seeded leakage, missing explicit records and silently accepted secrets. The fixture table above describes the historical input-only checkpoint, not current GREEN evidence.

Producer implemented exact empty-map lookups, explicit DTO snapshots and unconditional registration; removed only the two matching temporary publishers. No taint clearing or secret unwrapping. Conservative rejection remains **partial native coverage** for `AllowedWhenUntainted`.

### Reconciled batch25 bounded proof — 2026-10-01

Independent report: `/tmp/patch-12.0.5-housing-categories-independent-proof.md`. Inputs `63b53dfe5`, producer `bbbacf8f05d6ded38be356d907d59b764542dd9b`. Saved compiled RED **2 PASS / 10 FAIL** among twelve selected: public caller-taint and exact-selector controls already passed; failures reproduce seeded leakage, ignored explicit input, stale snapshots and secret acceptance. Saved GREEN **12 categories + 12 aggregates + 14 base + 24 variants/count + 4 cart = 66 PASS**, all selected runs nonempty. Both builds exit 0; compiler records emit normal and grouped integration executables. Cart controls are not category search proof.

Saved parent startup exits **0**, stdout `[]`, zero unique/occurrence Lua errors, 290 Blizzard addons; inspected, not independently rerun. No housing panel interaction or third-party addon proof. GREEN integration hash `fa4f6476d6475b89549625240b3e62d4d25ccede6aadbb360e142914b3a88c16` and normal executable hash `89af35c47ac9ab6f843a8666afa2d7fec574155c72ac2bb469d986f5d15b0fef` were independently recomputed before gates. Historical RED executable was overwritten; compiler records do not cryptographically bind revision to binary.

Fresh independent default `cargo fmt --check` and `cargo check` each exit **0** at clean producer; ledger `/tmp/patch-12.0.5-batch25-independent-checks.json`, full logs `/tmp/patch-12.0.5-batch25-independent-fmt.log` and `/tmp/patch-12.0.5-batch25-independent-check.log`. Source/readability review passes for producer changes. Stack rooting/barriers and unconditional registration are source-reviewed; no getter GC-pressure stress or other-profile execution. Fractional/out-of-range/nonnumeric selectors are source-inspected, not independently exercised by these twelve fixtures. Rooted secret fixture includes GC and rejection, but not native AllowedWhenUntainted acceptance.

Accept explicit empty-default maps, distinct exact keys, required/nullable DTO fields, explicit true/false `anyStoredEntries` independent of variants, absent `anyOwnedEntries`, fresh nested snapshots, host-mutation lifetime and environment isolation. Missing-ID nil and snapshot policies remain simulator inferences. Exact rename rows **643/659** gain **bounded-coverage** only; row **661** remains audit-pending. Accounting becomes **274 pending / 74 bounded / 14 partial = 362**; audit **IN PROGRESS**, all source IDs and text SHA retained.

Later concurrent `editor_mode_contexts` record/fixture edits and category search inputs are excluded. Gates establish historical clean producer proof, not current dirty checkout acceptance.

## Known gaps (current cycle)

- [ ] Full catalog and housing panel interaction acceptance remain open; bounded producer acceptance above does not close them.
- [ ] Native secure-secret selector acceptance remains unmodeled; current rejection deliberately differs from the declared allowance.
- [ ] `SearchCatalogCategories`/`SearchCatalogSubcategories` filtering and `HousingCategorySearchInfo` row 661 remain separately unresolved. No search calls or filter assertions exist in this slice. Seeded searches still return 101/102/1001/1002 while empty getter maps return nil; transitional search/getter mismatch remains until the next batch. Later search inputs are excluded.

## Out of scope

- Search/filter implementation, ownership derivation, other seed removal and unrelated storefront/cart behavior: separate work.
- Shared variant/aggregate specs, PLAN and dedicated search spec: excluded from this acceptance.
- Native execution, full catalog domain and all-profile acceptance: no evidence supplied by these fixtures.
