# Active LFG dungeon name

PTR `C_LFGInfo.GetActiveLFGDungeonName()` queries the existing instance ID and LFD catalog. The pinned 12.1.5 declaration adds a no-argument function returning one non-nil string; it does not define inactive or invalid-state behavior. [LFG implementation](../../src/c_api/c_lfg_info.rs).

## What it must do

- [ ] Publish the query only on PTR; earlier retail must retain absence through normal namespace lookup before and after bootstrap.
- [ ] Resolve only `world.instance_lfg_dungeon_id` against `lfd_dungeons`; return the matching catalog name, not the world instance label or proposal name.
- [ ] Read current state on every call: instance-ID changes and catalog renames must affect subsequent results.
- [ ] Return one empty string when no instance LFG ID is set, including proposal-only state.
- [ ] Raise an error naming the API, unknown ID, and catalog when a set ID has no matching entry; leave state unchanged and allow recovery after a valid ID is assigned.
- [ ] Preserve existing instance and LFG queries.

The instance-ID-only selection, empty inactive result, and unknown-ID error are chosen simulator semantics, not verified native behavior. `in_instance` is not an additional selection condition: a set instance LFG ID is the sole lookup key.

## How it works

- [Existing instance queries](../../src/lua_api/globals/instance_info.rs)
- [LFD catalog defaults](../../src/lua_api/state/defaults/lfg.rs)

## Implementation inventory

- `src/c_api/c_lfg_info.rs` — profile publication and state-backed query.
- `src/loader/tests/wow_api_globals/patch_12_1_5_active_lfg_dungeon.rs` — focused query and profile tests.
- `src/loader/tests/wow_api_globals/mod.rs` — existing grouped test-module wiring.

## Tests asserting this spec

- `src/loader/tests/wow_api_globals/patch_12_1_5_active_lfg_dungeon.rs`
- `tests/instance_info.rs`
- `tests/c_lfg_info_probes.rs`

## Known gaps (current cycle)

- [ ] Confirm native inactive/unknown-ID semantics and error behavior with a PTR probe.

## Out of scope

Queue/proposal selection, new state setters, caching, queue redesign, security/taint semantics, audit artifact generation, and native validation conformance. Tests drive existing state directly.
