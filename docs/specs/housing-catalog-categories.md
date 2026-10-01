# Housing catalog category snapshots

Bounded category/subcategory DTO contract for the 12.0.5 rename, backed by explicit inputs in `src/c_api/c_housing/catalog.rs`. Runtime getter replacement is pending. See [C API architecture](../wiki/systems/lua-api.md) for subsystem context.

## What it must do

- [ ] `C_HousingCatalog.GetCatalogCategoryInfo(categoryID)` reads only the exact category map key and returns `ID`, `orderIndex`, nullable `name`/`icon`, a fresh `subcategoryIDs` list, and explicit boolean `anyStoredEntries`.
- [ ] `C_HousingCatalog.GetCatalogSubcategoryInfo(subcategoryID)` reads only the exact subcategory map key and returns `ID`, `orderIndex`, `parentCategoryID`, nullable `name`/`icon`, and explicit boolean `anyStoredEntries`.
- [ ] Both true and false survive unchanged; `anyOwnedEntries` is absent. Neither predicate is inferred from partial catalog entries, variant storage, aggregates, list membership, or another record.
- [ ] Defaults expose no category/subcategory records. Missing exact IDs return nil, never another map's record or legacy All/Featured/Decor/Seating/Lighting seeds. **Simulator inference:** nullable documented results do not establish native missing-ID behavior.
- [ ] Every query produces a fresh snapshot, including the category's nested list. Lua mutation cannot change model inputs or another snapshot. Host record mutation changes only subsequent snapshots. Environments remain isolated. These are simulator snapshot policies, not native-verified aliasing semantics.
- [ ] Public numeric selectors work in secure and tainted calls without changing caller taint. Rooted host-secret numeric selectors reject without unwrapping in both contexts; secret identity survives GC and rejection. **Conservative simulator limitation:** secure secret acceptance is not modeled despite the native declaration below.

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
- `tests/housing_catalog_categories.rs`: twelve unrun getter fixtures in the existing generated `integration` target; feature gated to `retail-12-0-5`.
- `tests/housing_catalog_variants.rs`: existing whole-state constructor now uses `..HousingCatalogState::default()` so new maps stay empty; entry/variant fixture inputs unchanged.
- `src/lua_api/workarounds/temporary/housing_catalog_state.lua:967–985`: existing seeded `GetCatalogCategoryInfo`/`GetCatalogSubcategoryInfo` producers remain untouched. `GetCategoryInfo` is not the target getter.

## Tests asserting this spec

Parent filter: `housing_catalog_categories::` in the existing `integration` target. No new Cargo target or harness edit.

| Fixtures | Expected behavior | Proof |
|---|---|---|
| `empty_category_ids_do_not_return_legacy_seeds`, `empty_subcategory_ids_do_not_return_legacy_seeds` | Empty IDs 101/102/1001, plus subcategory 1002, return nil | Written, unrun |
| `category_fields_preserve_explicit_true_and_false_under_new_name`, `subcategory_fields_preserve_explicit_true_and_false_under_new_name`, `optional_fields_are_nullable_without_losing_required_fields` | Exact DTO values, new boolean name, retired field absent, nullable optionals | Written, unrun |
| `selectors_use_exact_id_and_never_cross_category_maps`, `stored_predicates_are_not_computed_from_partial_variant_storage` | Exact-key separation; explicit predicates independent of positive variant storage | Written, unrun |
| `returned_tables_and_nested_lists_are_fresh_snapshots`, `record_mutation_changes_new_snapshots_only`, `records_are_isolated_between_environments` | Snapshot/list isolation, mutation lifetime, environment isolation | Written, unrun |
| `public_numeric_selectors_preserve_caller_taint`, `secret_numeric_selectors_reject_without_unwrapping_or_taint_changes` | Public tainted access and conservative secure/tainted secret rejection | Written, unrun |

Proof ledger: no build, check, test, verification, push, or deployment run for this input-only checkpoint. Formatting is not behavior proof. Parent must establish actual RED before a fresh producer replaces seeded getters; no source-row coverage credit claimed.

## Known gaps (current cycle)

- [ ] Parent actual RED, fresh producer implementation, and subsequent runtime acceptance remain pending.
- [ ] Native secure-secret selector acceptance remains unmodeled; current rejection deliberately differs from the declared allowance.
- [ ] `SearchCatalogCategories`/`SearchCatalogSubcategories` filtering and `HousingCategorySearchInfo` row 661 remain separately unresolved. No search calls or filter assertions exist in this slice.

## Out of scope

- Search/filter implementation, ownership derivation, seed removal, serializers and registration: separate producer work.
- Shared variant/aggregate specs, wiki, coverage, PLAN and source-row status changes: excluded from this checkpoint.
- Native execution, full catalog domain and all-profile acceptance: no evidence supplied by these fixtures.
