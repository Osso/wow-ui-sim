# Transmog set relations

Retail and WoW Forever `C_TransmogSets` relation queries and `C_Transmog.GetAllSetAppearancesByID` read the supplied set catalog. This extends the existing [transmog filter state](transmog-sets-filter-state.md). Native details marked inferred await R88cs evidence.

## What it must do

- [ ] Project supplied base and variant records through the Retail and Forever `WardrobeSetsDataProviderMixin` consumer; distinct sets retain distinct primary appearances.
- [ ] INFERRED (R88cs pending): return catalog-backed base-set, usable-set, primary-appearance, collection, slot-source and complete appearance-item relations.
- [ ] INFERRED (R88cs pending): treat a valid, collected set as usable and a base as collected when its base or a variant is collected.
- [ ] Accept catalog data through `A_Admin.SetTransmogSetCatalog` without direct test-state writes or Lua API replacement.
- [ ] Keep the temporary empty defaults active for profiles outside Retail and WoW Forever.

## How it works

- [Transmog C API model](../wiki/systems/transmog-set-relations.md)

## Implementation inventory

- `src/c_api/c_transmog_sets.rs` — profile-gated query publication.
- `src/c_api/c_transmog_sets/catalog.rs` — catalog relation projections and admin input.
- `src/lua_api/globals/admin.rs` — registers the catalog input on `A_Admin`.
- `src/lua_api/workarounds/temporary/transmog_sets_defaults.rs` — unchanged defaults for profiles without the modeled surface.

## Tests asserting this spec

- `tests/blizzard_transmog_shared_loads.rs` — the loaded Blizzard shared provider reads two supplied set records.
- `tests/c_transmog_heirloom_api.rs` — legacy-only checks for the retained shim defaults.

## Known gaps (current cycle)

- [ ] Native behavior for relation edge cases is pending R88cs; these outputs remain INFERRED.
- [ ] Supplied tests do not exercise every relation getter through a live Wardrobe or Transmog UI action.

## Out of scope

Authoritative game set inventory, native catalog defaults, filter behavior, and relation changes outside the supplied model.
