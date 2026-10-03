# Table freezing

`table.freeze` and `table.isfrozen` expose shallow table immutability through modeled Lua APIs, independently of rilua's recursive GC-freeze optimization. The pinned [12.1.5 register](../../data/patch-api/sources/12.1.5-register.json) changes the argument type from `LuaValueReference` to `table`; PTR additionally returns the frozen table. Both functions already exist in the pinned earlier contract.

## What it must do

- [x] Publish both functions for the `retail-12-0-5` epoch and later; do not remove them from earlier retail when enabling PTR.
- [x] PTR `freeze(t)` returns exactly one result identical to `t`; earlier retail returns no results. Repeated freezing is idempotent.
- [x] `isfrozen(t)` returns exactly one boolean reflecting the VM flag, false before freezing and true afterward.
- [x] Preserve reads, iteration, array length, and object identity.
- [x] Reject assignment, `rawset`, `table.insert/remove/sort`, global `tinsert/tremove/wipe`, and PTR `table.removeunordered/removevalue` on already-frozen tables without partial mutation. Mutable-table operations retain ordinary behavior.
- [x] Freeze only the supplied table. Referenced tables, keys, metatables, closure environments and captured state remain mutable; freezing an addon namespace must not freeze `_G` or later addon state.
- [x] Keep ordinary GC tracing and collection: reachable values survive collection, but freezing does not permanently pin the table or its graph.
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

## Development proof and independent bounded acceptance — 2026-10-03

Commit `a4cce2db1`. RED: not observed: default retail includes retail-12-1-0, where the old gate already registered it. GREEN: 1/1 inside a 404/404 run with control suites; `cargo fmt --check` exit0; startup `lua-errors` `[]`. This section supersedes any wording above that describes the slice as staged, unapplied or unrun.

Main accepts an independent GPT-6.1-sol source review (no test rerun): **ACCEPT WITH QUALIFICATIONS**, [report](../../data/patch-api/evidence/12.0.5-session-2026-10-03/b98-verify-events-commands.md) SHA256 `72c3dbad682b576a045792d498f279ba54e7ef7b6094d5364da3d8a5817b4ac3`. Partial until a strict 12.0.5 build executes the test. Requirement checkboxes are left as authored; the report lists which are earned and to what bound. Bounded simulator proof, not native parity.

[Page accounting](../../data/patch-api/sources/12.0.5-page-coverage.json): prose-2026-03-25-094 partial-development-green under capability `table-freeze-addon-read-only`; **119 capabilities/362 IDs; 42 pending /262 bounded /23 partial /35 metadata**.

## Known gaps (current cycle)

- [ ] Earlier non-table `LuaValueReference` inputs and cross-addon ownership/taint behavior remain unverified.
- [ ] Shallow semantics are supported by QuestieTDB's direct Classic Era 1.15.9.68808 probe (`docs/table.freeze.md` in `Questie/QuestieTDB`), not a retail native probe. Retail unchanged Syndicator's namespace freeze followed by callback cleanup establishes the simulator regression boundary.
- `748d4df0c` pins published rilua `1788318`; at `9b8d44b06`, independent default-profile verification passes table utilities 11/11 (including the two-addon lifecycle regression), freeze library tests 4/4, attribute recovery 11/11, `cargo fmt --check` and `cargo check`. Rilua has independent shallow-GC/write proof. See [bounded retail replay and residual failures](../wiki/investigations/patch-12-1-5-api-audit.md#table-freeze-corrective-boundary); this is not zero-error or full native conformance.
- [ ] The VM can forward missing-key writes through `__newindex`; this does not mutate the frozen table itself. Metatable replacement, reentrant freezing during comparator callbacks, and arbitrary Rust-side mutation paths are not covered by these tests.

## Out of scope

Unfreezing, redesign of the recursive GC optimization, broad VM hardening, new ownership enforcement, audit-manifest credit, and broader profile conformance are excluded from this correction.
