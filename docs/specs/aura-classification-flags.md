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
- `src/lua_api/globals/auras.rs` — shared table writer publishes all five classifications as plain `Val::Bool` from explicit aura inputs (harmfulness complements helpfulness), without player overrides, secret unwrapping, taint clearing, or new declassification. Fixed target fixtures and local test constructor retain explicit legacy values.
- `src/loader/tests/wow_api_tooltip_helpers.rs`, `tests/c_unit_auras_admin.rs`, `tests/aura_api.rs`, `patch-tests/patch_12_1/aura_container.rs` — existing test constructors retain explicit legacy values.
- `tests/aura_table_shape.rs` — existing shape assertions retained; new host-populated player/party fixtures exercise independent classifications, public booleans, taint, and snapshots without adding a Cargo target.

Existing complete constructor sites updated (14 total):

- `src/lua_api/game_data.rs`: `apply_player_aura_spell`, `make_party_buff`, `make_party_debuff`, `build_auras_from_indices` (four).
- `src/lua_api/globals/auras.rs`: `target_fixture_auras` (two), `tests::plain_helpful_aura` (one); `src/lua_api/globals/admin.rs`: `build_admin_aura` (one).
- `src/loader/tests/wow_api_tooltip_helpers.rs`: `flash_of_light_aura` (one); `tests/c_unit_auras_admin.rs`: `admin_aura` (one). `admin_buff` and the struct-update `dispellable_debuff` inherit the updated constructor values.
- `tests/aura_api.rs`: `test_c_unit_auras_filters_aura_instances_by_polarity_and_player_source` (two); `patch-tests/patch_12_1/aura_container.rs`: `seed_player_filter_auras` (two).

New complete constructor sites: `tests/aura_table_shape.rs::{helpful_non_raid_fixture,harmful_raid_fixture}`. Party fixtures clone these inputs, then explicitly reverse nameplate-only and from-player values.

## Tests asserting this spec

Existing five `aura_table_shape` tests retain their expectations. The target debuff assertion now describes its explicit fixture classification rather than asserting a general helpfulness/raid coupling.

Grouped regression filters (producer GREEN not run):

- `aura_table_shape::explicit_classification_flags_are_public_independent_and_query_consistent` — host-populated player and party helpful/non-raid and harmful/raid fixtures, independently varied nameplate/from-player flags, all five boolean/secrecy assertions, slot/index/instance-ID consistency, and exact addon-taint preservation. Fixed target aura instance 1 is a legacy query control, not a host-populated target input.
- `aura_table_shape::classification_query_snapshots_do_not_mutate_host_or_other_results` — independent slot/instance-ID/index results for player, party, and the fixed target control; mutate all five returned flags, retain other snapshots and host inputs, and preserve addon taint.

Parent-owned focused commands:

```text
cargo test --test integration aura_table_shape::explicit_classification_flags_are_public_independent_and_query_consistent -- --exact
cargo test --test integration aura_table_shape::classification_query_snapshots_do_not_mutate_host_or_other_results -- --exact
```

Parent-reported compiled RED after public import fix `cdd1d729e`: `/tmp/patch-12.0.5-batch28-red-fixed-*`, grouped `aura_table_shape::` result **6 PASS / 1 FAIL**, failing on the player `isRaid` value mismatch. Initial RED build failed on the private `game_data` import; it did not compile successfully. Inputs: `f7166567e`, `663b9ad7c`, `4243827e2`, plus import fix `cdd1d729e`. Producer has not rerun these commands.

Existing-control filter: `cargo test --test integration aura_table_shape::`. Constructor-compilation controls also include `c_unit_auras_admin::`, `aura_api::`, and the PTR `patch_12_1_audit` target; no producer GREEN or cross-profile proof is claimed here.

## Known gaps (current cycle)

- [x] Parent observed compiled behavioral RED after fixing the private import: player `isRaid` mismatch, 6 PASS / 1 FAIL.
- [x] Producer changed the shared classification writer to use `is_raid`, `is_nameplate_only`, and `is_from_player_or_player_pet` directly; existing constructors and fixed target inputs remain unchanged.
- [ ] Parent GREEN and verifier acceptance remain pending. Requirement checkboxes above remain open until that proof; producer slice performs formatting and commit only, without builds, tests, checks, readability, startup, or delegation.
- [ ] Target queries currently read fixed Rust fixtures, with no host-populated target aura store. Tests preserve that pathway rather than adding or pretending to exercise target inputs.

## Out of scope

- Secret quantitative aura fields, native classification/default inference, private-aura modeling, or native-client parity: the retained row does not establish those contracts.
- New filtering semantics (including RAID/nameplate filters), acquisition, private auras, events, lifecycle, admin setters, storage relocation, or unrelated refactors: producer slice changes only the required shared classification read path and dedicated spec.
- Shared wiki, coverage, PLAN, audit accounting, unrelated changes, delegation, build/check/test execution, push, and deployment: expressly excluded.
