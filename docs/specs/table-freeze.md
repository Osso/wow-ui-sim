# Table freezing

`table.freeze` and `table.isfrozen` expose rilua's frozen-object state through modeled Lua APIs. The pinned [12.1.5 register](../../data/patch-api/sources/12.1.5-register.json) changes the argument type from `LuaValueReference` to `table`; PTR additionally returns the frozen table. Both functions already exist in the pinned earlier contract.

## What it must do

- [x] Publish both functions for the `retail-12-1-0` epoch and later; do not remove them from earlier retail when enabling PTR.
- [x] PTR `freeze(t)` returns exactly one result identical to `t`; earlier retail returns no results. Repeated freezing is idempotent.
- [x] `isfrozen(t)` returns exactly one boolean reflecting the VM flag, false before freezing and true afterward.
- [x] Preserve reads, iteration, array length, and object identity.
- [x] Reject assignment, `rawset`, `table.insert/remove/sort`, global `tinsert/tremove/wipe`, and PTR `table.removeunordered/removevalue` on already-frozen tables without partial mutation. Mutable-table operations retain ordinary behavior.
- [x] **Simulator traversal choice:** recursively freeze table values, table keys, and private metatables; cycles terminate. Tests use isolated graphs and confirm nested reads survive collection.
- [x] **Simulator validation choice:** both profiles require real Lua tables and reject missing/nil/non-table arguments. The broader earlier `LuaValueReference` domain is not claimed.

## How it works

- [Lua API](../lua-api.md)
- [Client profiles](client-profiles.md)
- [Other table extensions](lua-table-extensions.md)

## Implementation inventory

- `src/lua_api/globals/real/table_freeze.rs`: registration, profile return arity, VM freeze/query calls, guarded native mutators.
- `src/lua_api/globals/real/table_extensions.rs`: shared table lookup and guarded PTR mutators.
- `src/lua_api/globals/utility_system_spell/mod.rs`: guarded global table mutation aliases.
- `src/lua_api/globals/register.rs`: epoch-scoped API installation.
- Pinned rilua `b638756`: existing recursive GC freeze traversal and assignment/rawset enforcement; unchanged.

## Tests asserting this spec

- `src/loader/tests/wow_api_globals/patch_12_1_5_table_freeze.rs`: four grouped library tests per profile for publication/arity, read preservation, mutation rejection, isolated recursive graphs, and argument errors.

## Known gaps (current cycle)

- [ ] Native shallow-versus-recursive semantics, earlier non-table `LuaValueReference` inputs, and secret/taint behavior remain unverified.
- [ ] The reused VM walk also traverses closure environments/upvalues and userdata environments/metatables. An ordinary closure can reach `_G`; freezing such a graph can freeze shared runtime state. No shared tables/metatables or such closures are frozen in these fixtures. This is the VM traversal model, not a claim about native WoW behavior.
- [ ] The VM can forward missing-key writes through `__newindex`; this does not mutate the frozen table itself. Metatable replacement, reentrant freezing during comparator callbacks, and arbitrary Rust-side mutation paths are not covered by these tests.

## Out of scope

External rilua changes, unfreezing, broad VM hardening, native security enforcement, audit-manifest credit, and broader profile conformance are excluded from this bounded implementation.
