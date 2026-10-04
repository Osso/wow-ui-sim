# House exterior core-fixture attachment transactions

Bounded `C_HouseExterior.SelectCoreFixtureOption(fixtureID, attachedDecorAction)` over explicit host state. Retained row `global api-C_HouseExterior-SelectCoreFixtureOption-270` adds arg2. Cached retail `HouseExteriorUIDocumentation.lua:160–168` specifies AllowedWhenUntainted, omitted Store, and mandatory reparenting for variants/recolors. `PlayerHousingConstantsDocumentation.lua:141–150` specifies Store0 and Detach1. This extends [existing exterior transactions](house-exterior-attached-decor.md), not native geometry or runtime acquisition.

## What it must do

All boxes remain unchecked: authoring only, no behavioral runs authorized.

- [ ] Start with no core selection/options. One host-supplied core family has an explicit current fixture, eligible alternatives, attachment-owner hashes and optional recolor groups. Never synthesize fixture records or derive equivalence from IDs.
- [ ] Store only placements attached directly to the replaced core's owner hash, through the existing atomic catalog transaction. Preserve unrelated/floating placements, destroyability and dye records; update known totals only.
- [ ] Detach only affected owner links; preserve placement identities, variant keys, coordinates and inventory.
- [ ] Recolor between explicitly equivalent nonnil groups reparents affected owner links to the target core regardless of Store/Detach. Both missing groups do not establish equivalence. A subsequent different-style replacement must affect the newly parented placements.
- [ ] Authenticate both original args before parsing/defaulting/model reads. Untainted secret inputs resolve; addon secret inputs reject; ordinary addon arguments remain allowed. Rejection leaves selection, placements, counts and events unchanged.
- [ ] **INFERRED:** reuse adjacent exterior policy: positive integral u32 IDs, numeric0/1 actions, explicit nil defaults Store, owned plot/mode6 eligibility, locked/invalid/missing options reject with an error, duplicate IDs use first host option, current option must exist, same-selection success without attachment work.
- [ ] **INFERRED:** reuse Success0 fixture-response publication and synchronous storage-before-response order. Commit all state before callbacks; allow real API reentry without outstanding borrows or outer overwrite. No events on rejection.

## How it works

- [C API and environment boundary](../lua-api.md)
- [Existing exterior transaction contract](house-exterior-attached-decor.md)
- [Catalog variant storage contract](housing-catalog-variants.md)

## Implementation inventory

- `src/c_api/c_housing/exterior.rs`: empty optional core selection and explicit replacement options.
- `src/c_api/c_housing/exterior/runtime.rs`: original-argument authentication and real callback registration.
- `src/c_api/c_housing/exterior/core.rs`: validated direct-owner selection/reparent transitions.
- `src/c_api/c_housing/exterior/mutation.rs`: existing shared eligibility/attachment operations, exposed only within runtime modules.
- `src/lua_api/workarounds/temporary/housing_catalog_state.lua`: obsolete SelectCoreFixtureOption no-op removed, never retained as an alternate provider.

## Tests asserting this spec

`tests/house_exterior_core_fixture.rs`: ten authored behavioral cases. Existing `tests/house_exterior.rs` needs `core_fixture: None` in its exhaustive state fixture, with no expectation/gating change. Test discovery remains the existing generated integration target.

## Known gaps (current cycle)

- [ ] Integrator must run state-only RED, final producer GREEN, applicable existing exterior controls and startup checks. Author has run none.
- [ ] Native error/result mapping, nil/coercion/range semantics, event ordering, duplicate records and unavailable-host policy remain inferred.

## Out of scope

Multiple simultaneous Base/Roof selections, child-fixture attachment ancestry/hook migration, fixture acquisition/persistence, core-options DTO publication, core attachment predicates, core-change/UI refresh events and 3D transforms. The current model has placement-owner hashes but no explicit child-fixture ancestry or native option population; do not manufacture those relations. Existing core query placeholders are not proof of this mutator's state, and the authored tests deliberately do not use them. No whole-row/native/all-profile/UI closure claimed. Modern registration keeps the existing exterior feature/profile gate; no legacy no-op compatibility path is added.
