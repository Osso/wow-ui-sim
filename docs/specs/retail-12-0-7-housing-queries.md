# Retail 12.0.7 housing names, doors and floors — B18/B20/B21

Source rows [034–036](../../data/patch-api/sources/12.0.7-api-changes.txt) add three queries. Cached retail HousingCatalogUI, HousingCustomizeModeUI and HousingLayoutUI documentation supplies signatures and `AllowedWhenUntainted`; cache may postdate 12.0.7 and is not native historical proof.

## What it must do

- [x] Under `retail-12-0-7`, names read the subcategory's explicit parent and both live labels, returning exactly two public strings.
- [x] Door compatibility reads the explicit `(roomGUID, componentID)` type set; floor permission reads the explicit signed floor-index map. Queries return one public boolean without mutation.
- [x] Authenticate all arguments and extras using `unwrap_secret` before selector validation. Untainted callers may use secrets; tainted callers cannot, including on misses or malformed preceding arguments.
- [x] Outputs follow live updates and environments remain independent.
- [x] INFERRED: empty host state has no names or permissions; missing relation/label or malformed exact-i32 selector returns no names / false, never synthesized service data. Negative floor indices are supported.

## How it works

- [Lua API](../lua-api.md)
- [Widget/state architecture](../widget-system.md)

## Implementation inventory

- `src/c_api/c_housing.rs` — feature-gated registration.
- `src/c_api/c_housing/patch_12_0_7.rs` — live producers.
- `src/c_api/patch_12_0_7_inputs.rs` — shared argument authentication and exact-integer selection.
- `src/lua_api/state/support_types.rs` — empty door/floor host inputs; existing catalog records supply labels and parent relation.

## Tests asserting this spec

- `tests/p1207_housing_names.rs` — three public API cases.
- `tests/p1207_housing_door.rs` — three public API cases.
- `tests/p1207_housing_floor.rs` — three public API cases.

## Known gaps (current cycle)

- [ ] Historical 12.0.7 signature provenance and native execution remain unproved.
- [ ] Catalog tooltip consumer assumes every referenced subcategory has both labels. Hosts must supply consistent records; missing-label tests do not claim that tooltip path is safe.

## Out of scope

- Housing ownership/mode/service derivation, automatic permission updates, localization, UI interaction and other-profile proof.
- No vendor/cache changes. Empty catalog contains no entries referencing missing labels; no producer excluded on empty-default grounds.
