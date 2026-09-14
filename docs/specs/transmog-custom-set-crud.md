# Transmog custom-set CRUD

Bounded per-environment custom-set storage lives in `src/c_api/c_transmog_collection.rs`. Contracts come from `data/patch-api/sources/12.0.0-register.json`; see [C API architecture](../lua-api.md).

## What it must do

- [x] `NewCustomSet(name, icon, itemTransmogInfoList)` returns one generated numeric ID for accepted fixtures; distinct creations remain distinct.
- [x] `GetCustomSets()` returns a fresh array of numeric IDs. `GetCustomSetInfo(id)` returns name and numeric icon; `GetCustomSetItemTransmogInfoList(id)` returns a fresh array of fresh item tables.
- [x] `ModifyCustomSet(id, list)`, `RenameCustomSet(id, name)`, and `DeleteCustomSet(id)` return zero values and change only the selected record.
- [x] Copy appearanceID, secondaryAppearanceID and illusionID values on input and output; isolate environments.

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

`tests/c_namespace_noop_replacements.rs`: four `custom_set_crud_*` tests at `ef9bbe70c`; RED 0/4, initial `NewCustomSet` returning one nil. Runtime `df6264867` reached GREEN; `/tmp/verify-custom-set-crud-ledger.json` records 4/4 each on retail 12.0.0/12.0.5/12.0.7, reusing exact 12.0.0 bytes and running later profiles fresh. Final metadata proof at `2531a0e61` records 15,011 fresh hashes, zero stale, 86 renewals, 35 additions, seven credits, and validator exit 0 with all 3,410 rows matching. Error policies and missing-ID behavior are not covered by these four tests.

## Known gaps (current cycle)

Readability reported `read_items` length and local-`Vec` loop signals. Main reviewed them and rejected a mandatory refactor; this is not a clean-readability claim.

## Out of scope

Native validation, error policy, defaults, limits, events, persistence, hyperlinks and consumer behavior remain unproven. Security and slot semantics remain outside the bounded fixtures. No category or unrelated transmog behavior changes; broad audit remains open and eight Mists AccountStore failures remain unresolved.
