# Table freezing

`table.freeze` and `table.isfrozen` expose shallow table immutability through modeled Lua APIs, independently of rilua's recursive GC-freeze optimization. The pinned [12.1.5 register](../../data/patch-api/sources/12.1.5-register.json) changes the argument type from `LuaValueReference` to `table`; PTR additionally returns the frozen table. Both functions already exist in the pinned earlier contract.

## What it must do

- [x] Publish both functions for the `retail-12-1-0` epoch and later; do not remove them from earlier retail when enabling PTR.
- [x] PTR `freeze(t)` returns exactly one result identical to `t`; earlier retail returns no results. Repeated freezing is idempotent.
- [x] `isfrozen(t)` returns exactly one boolean reflecting the VM flag, false before freezing and true afterward.
- [x] Preserve reads, iteration, array length, and object identity.
- [x] Reject assignment, `rawset`, `table.insert/remove/sort`, global `tinsert/tremove/wipe`, and PTR `table.removeunordered/removevalue` on already-frozen tables without partial mutation. Mutable-table operations retain ordinary behavior.
- [ ] Freeze only the supplied table. Referenced tables, keys, metatables, closure environments and captured state remain mutable; freezing an addon namespace must not freeze `_G` or later addon state.
- [ ] Keep ordinary GC tracing and collection: reachable values survive collection, but freezing does not permanently pin the table or its graph.
- [x] **Simulator validation choice:** both profiles require real Lua tables and reject missing/nil/non-table arguments. The broader earlier `LuaValueReference` domain is not claimed.

## How it works

- [Lua API](../lua-api.md)
- [Client profiles](client-profiles.md)
- [Other table extensions](lua-table-extensions.md)

## Implementation inventory

- `src/lua_api/globals/real/table_freeze.rs`: registration, profile return arity, shallow read-only/query calls, guarded native mutators.
- `src/lua_api/globals/real/table_extensions.rs`: shared table lookup and guarded PTR mutators.
- `src/lua_api/globals/utility_system_spell/mod.rs`: guarded global table mutation aliases.
- `src/lua_api/globals/register.rs`: epoch-scoped API installation.
- Rilua table immutability: root-only read-only state and write guards, separate from recursive GC freezing.

## Tests asserting this spec

- `src/loader/tests/wow_api_globals/patch_12_1_5_table_freeze.rs`: grouped library tests for publication/arity, read preservation, mutation rejection, shallow graphs, and argument errors.
- `tests/table_util.rs`: addon callback/later-load boundary and collection regression.

## Known gaps (current cycle)

- [ ] Earlier non-table `LuaValueReference` inputs and cross-addon ownership/taint behavior remain unverified.
- [ ] Shallow semantics are supported by QuestieTDB's direct Classic Era 1.15.9.68808 probe (`docs/table.freeze.md` in `Questie/QuestieTDB`), not a retail native probe. Retail unchanged Syndicator's namespace freeze followed by callback cleanup establishes the simulator regression boundary.
- [ ] Final dependency pin, focused verification and retail startup replay pending.
- [ ] The VM can forward missing-key writes through `__newindex`; this does not mutate the frozen table itself. Metatable replacement, reentrant freezing during comparator callbacks, and arbitrary Rust-side mutation paths are not covered by these tests.

## Out of scope

Unfreezing, redesign of the recursive GC optimization, broad VM hardening, new ownership enforcement, audit-manifest credit, and broader profile conformance are excluded from this correction.
