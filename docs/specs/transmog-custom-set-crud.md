# Transmog custom-set CRUD

Bounded per-environment custom-set storage lives in `src/c_api/c_transmog_collection.rs`. Contracts come from `data/patch-api/sources/12.0.0-register.json`; see [C API architecture](../lua-api.md).

## What it must do

- [ ] `NewCustomSet(name, icon, itemTransmogInfoList)` returns one generated numeric ID for accepted fixtures; distinct creations remain distinct.
- [ ] `GetCustomSets()` returns a fresh array of numeric IDs. `GetCustomSetInfo(id)` returns name and numeric icon; `GetCustomSetItemTransmogInfoList(id)` returns a fresh array of fresh item tables.
- [ ] `ModifyCustomSet(id, list)`, `RenameCustomSet(id, name)`, and `DeleteCustomSet(id)` return zero values and change only the selected record.
- [ ] Copy appearanceID, secondaryAppearanceID and illusionID values on input and output; isolate environments.

Simulator policies: initially empty storage, monotonically generated IDs, owned copied values, integer-ID conversion matching existing transmog APIs. Parse the complete list before changing storage; malformed rows or missing numeric fields produce explicit errors. Missing lookup returns zero values, permitted by pinned mayreturnnothing; missing mutator IDs raise an explicit error. These policies do not establish native validation or defaults.

## How it works

- [API subsystem](../lua-api.md)

## Implementation inventory

- `src/c_api/c_transmog_collection.rs`: owned records, item values and seven handlers.
- `src/c_api/mod.rs`: retail-12-0-0 module gate.
- `src/lua_api/state/sim_state.rs`: per-environment model field.
- `src/lua_api/state.rs`: empty initialization.
- `src/lua_api/globals/missing_surface/transmog_collection.rs`: adds handlers after existing functions on the same namespace table.

## Tests asserting this spec

`tests/c_namespace_noop_replacements.rs`: four `custom_set_crud_*` tests at `ef9bbe70c`; RED 0/4, initial NewCustomSet returning one nil. GREEN pending at implementation commit. Error policies and missing-ID behavior are not covered by these four tests.

## Known gaps (current cycle)

- [ ] Independent verification and cross-profile proof remain pending.

## Out of scope

Native limits, validation, defaults, success/failure policy, events, hyperlinks, slot semantics, persistence and security: not established by the bounded fixtures. No category or unrelated transmog behavior changes.
