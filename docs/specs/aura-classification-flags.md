# Aura classification flags

Public aura tables expose five ordinary boolean classifications from the existing `AuraInfo` inputs in `src/lua_api/game_data.rs`. Raid and nameplate-only classifications are independent explicit inputs, not inferred from helpfulness. Query publication remains in `src/lua_api/globals/auras.rs`; see [Lua API architecture](../lua-api.md).

## What it must do

- [ ] Return `isHelpful`, `isHarmful`, `isRaid`, `isNameplateOnly`, and `isFromPlayerOrPlayerPet` as ordinary Lua booleans for which `issecretvalue` returns false, including addon-tainted queries.
- [ ] Publish `isHelpful` from `is_helpful`, `isHarmful` from its complement, `isRaid` from `is_raid`, `isNameplateOnly` from `is_nameplate_only`, and `isFromPlayerOrPlayerPet` from its existing explicit input. Helpful/non-raid and harmful/raid inputs must both retain their classifications.
- [ ] Preserve true and false nameplate-only and from-player flags independently of helpfulness and raid status. From-player publication must use the explicit boolean, not reclassify `source_unit`.
- [ ] Slot, filtered-index, and instance-ID queries must agree for the same aura and preserve caller addon taint. Returned tables must be independent snapshots: mutating one must not alter other results or host inputs.
- [ ] Preserve existing fixture outputs while making constructor values explicit: previous helpful fixtures retain `is_raid=true`, previous harmful fixtures retain `is_raid=false`, and all existing constructors retain `is_nameplate_only=false`.

### Evidence and inferred fixture values

Retained row [`prose-2026-03-12-029`](../../data/patch-api/sources/12.0.5-register.json), source line **29**, says: “The isHelpful, isHarmful, isRaid, isNameplateOnly, and isFromPlayerOrPlayerPet booleans on aura data tables are no longer secret.” This supports public representation of these five fields only. It does not establish native aura classification rules or defaults.

The new inputs are ordinary public host booleans, not secret-aware values. Existing constructor values deliberately preserve the simulator's prior outputs (`isRaid` previously followed helpfulness; nameplate-only was always false). Those legacy/seed values are inferred simulator fixture policy, **not native data**, and do not impose coupling on explicitly populated auras. Existing source-unit-derived seed values for from-player remain fixture policy; new tests intentionally supply explicit values that disagree with source-unit spelling.

## How it works

- [Lua API architecture](../lua-api.md)
- [Existing public aura query model](unit-aura-instance-enumeration.md)

## Implementation inventory

- `src/lua_api/game_data.rs` — existing `AuraInfo` with explicit `is_raid` and `is_nameplate_only`; paladin, party buff/debuff, and player-pool constructors retain legacy values.
- `src/lua_api/globals/admin.rs` — admin aura constructor explicitly retains its prior helpfulness-based raid value and false nameplate-only value; no new admin input surface.
- `src/lua_api/globals/auras.rs` — fixed target fixtures and local test constructor retain explicit legacy values; table writer is unchanged at this input checkpoint.
- `src/loader/tests/wow_api_tooltip_helpers.rs`, `tests/c_unit_auras_admin.rs`, `tests/aura_api.rs`, `patch-tests/patch_12_1/aura_container.rs` — existing test constructors retain explicit legacy values.
- `tests/aura_table_shape.rs` — existing shape assertions retained; new host-populated player/party fixtures exercise independent classifications, public booleans, taint, and snapshots without adding a Cargo target.

## Tests asserting this spec

Existing five `aura_table_shape` tests retain their expectations. The target debuff assertion now describes its explicit fixture classification rather than asserting a general helpfulness/raid coupling.

New grouped filters, both **unrun**:

- `aura_table_shape::explicit_classification_flags_are_public_independent_and_query_consistent` — host-populated player and party helpful/non-raid and harmful/raid fixtures, independently varied nameplate/from-player flags, all five boolean/secrecy assertions, slot/index/instance-ID consistency, and exact addon-taint preservation. Fixed target aura instance 1 is a legacy query control, not a host-populated target input.
- `aura_table_shape::classification_query_snapshots_do_not_mutate_host_or_other_results` — independent slot/instance-ID/index results for player, party, and the fixed target control; mutate all five returned flags, retain other snapshots and host inputs, and preserve addon taint.

Parent-owned compiled RED commands (documented, **not executed**):

```text
cargo test --test integration aura_table_shape::explicit_classification_flags_are_public_independent_and_query_consistent -- --exact
cargo test --test integration aura_table_shape::classification_query_snapshots_do_not_mutate_host_or_other_results -- --exact
```

Existing-control filter: `cargo test --test integration aura_table_shape::`. Constructor-compilation controls also include `c_unit_auras_admin::`, `aura_api::`, and the PTR `patch_12_1_audit` target; no execution or cross-profile proof is claimed here.

## Known gaps (current cycle)

- [ ] Parent must compile and observe actual behavioral RED before a fresh producer change. Current writer still derives `isRaid` from helpfulness and publishes constant false `isNameplateOnly`; the first new test is intended to expose this mismatch, not yet proven to fail.
- [ ] After RED, publish the two explicit fields and obtain focused GREEN plus retained controls. No writer/producer changes, test execution, builds, checks, coverage, push, or deployment occur in this slice.
- [ ] Target queries currently read fixed Rust fixtures, with no host-populated target aura store. Tests preserve that pathway rather than adding or pretending to exercise target inputs.

## Out of scope

- Secret quantitative aura fields, native classification/default inference, private-aura modeling, or native-client parity: the retained row does not establish those contracts.
- New filtering semantics (including RAID/nameplate filters), acquisition, events, admin setters, storage relocation, or refactors: this slice changes inputs/tests/spec only.
- Shared wiki, coverage, PLAN, audit accounting, unrelated changes, delegation, build/check/test execution, push, and deployment: expressly excluded.
