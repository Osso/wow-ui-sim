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

Existing five `aura_table_shape` tests retain boolean-field coverage. The target debuff assertion describes its explicit fixture classification rather than asserting a general helpfulness/raid coupling. The old player shape test now installs a deterministic helpful/non-raid host record with source `player` and explicit from-player false, then asserts returned name/source identity and explicit flag equality.

Grouped regression filters:

- `aura_table_shape::explicit_classification_flags_are_public_independent_and_query_consistent` — host-populated player and party helpful/non-raid and harmful/raid fixtures, independently varied nameplate/from-player flags, all five boolean/secrecy assertions, slot/index/instance-ID consistency, and exact addon-taint preservation. Fixed target aura instance 1 is a legacy query control, not a host-populated target input.
- `aura_table_shape::classification_query_snapshots_do_not_mutate_host_or_other_results` — independent slot/instance-ID/index results for player, party, and the fixed target control; mutate all five returned flags, retain other snapshots and host inputs, and preserve addon taint.

Parent-owned focused commands:

```text
cargo test --test integration aura_table_shape::explicit_classification_flags_are_public_independent_and_query_consistent -- --exact
cargo test --test integration aura_table_shape::classification_query_snapshots_do_not_mutate_host_or_other_results -- --exact
```

Parent-reported compiled RED after public import fix `cdd1d729e`: `/tmp/patch-12.0.5-batch28-red-fixed-*`, grouped `aura_table_shape::` result **6 PASS / 1 FAIL**, failing on the player `isRaid` value mismatch. Initial RED build failed on the private `game_data` import; it did not compile successfully. Inputs: `f7166567e`, `663b9ad7c`, `4243827e2`, plus import fix `cdd1d729e`. Producer has not rerun these commands.

### Compiled producer result and old-test correction — 2026-10-01

Inspected parent artifacts `/tmp/patch-12.0.5-batch28-green-revision.txt`, `-green-build-result.json`, `-green-run-0.log`, and `-green-runs.json`: revision `bff26b9c112885736ff227d39e0e32380a53bfdd`, build exit **0**, focused `aura_table_shape::` execution exit **101**, **6 PASS / 1 FAIL**. Both new explicit-classification and snapshot tests passed. The sole focused failure was `aura_table_exposes_boolean_flag_shape`, old line **87**, `assert!(from_player)`. These are saved parent results, not a fresh execution by this correction slice; the artifact's GREEN label does not mean the focused group passed.

The prior expectation was invalid: `SimState::seed_default_game_state` uses `default_player_buffs`, which selects a clock-dependent subset from `BUFF_POOL`. `build_auras_from_indices` assigns instance IDs starting at 1 and explicit from-player flags from the selected source. Party-sourced Arcane Intellect (`party2`), Mark of the Wild (`party3`), and Battle Shout (`party1`) precede other player-source entries in the pool and can occupy slot 1 with an explicit false flag. The old writer forced true for player-unit queries, masking this input distinction. The failure log does not identify which party aura was selected.

Intermediate correction `16e2945a3` compared the clock-selected host record and recorded saved 7/7 PASS, but independent source review found its source identity assertion incomplete: player queries force sourceUnit `player`, whereas the selected host record can be party-sourced. Final fixture fix `e652d9610` replaces clock selection with a deterministic explicit record; assertions and production identity remain unchanged.

### Reconciled batch28 bounded partial proof — 2026-10-01

Evidence: `/tmp/patch-12.0.5-aura-classification-independent-proof.md` and `/tmp/patch-12.0.5-aura-classification-followup-proof.md`, read in full. Final followup resolves only fixture reliability and accepts bounded explicit-input publication/snapshot behavior.

| Scope | Result | Proof level |
|---|---|---|
| Initial private-import compile failure; corrected behavioral RED | Compile failure has no RED credit; corrected 6 PASS / 1 FAIL at player isRaid mismatch | Saved parent artifacts |
| Producer `bff26b9c1` | New explicit/snapshot cases PASS; focused 6/7 with old forced-from-player assertion failure | Saved parent runtime |
| Deterministic fixture `e652d9610`, shared build `7926dfdfe` | Seven shape/classification/snapshot cases PASS | Saved batch29 red-fixed build/run entry 1; final independent followup PASS, not rerun |
| Default formatting/production compilation | Fresh followup fmt exit 0; prior check exit 0 reused across 1,488 identical source/Cargo/build/config hashes | Snapshot-scoped independent gates; not current whole-worktree proof |
| Broader controls | Unique 121 PASS / 2 FAIL out of 123; overlapping private filter adds zero unique tests | Historical saved runtime, BROAD NOT GREEN |
| Startup | Exit 0, `[]` | Saved producer startup; no fresh startup |

Unresolved controls: `on_update_modes::on_update_modes_process_actual_managed_aura_dirty_phases` (managed dirty phase) and `unit_auras_private::native_unit_event_dispatch_respects_unit_filter` (nativeSpecialization payload/unit dispatch). Exact failed Lua assertions are unidentified. **PREEXISTING UNPROVEN**: no pre-producer baseline establishes their origin; focused followup neither fixes nor supersedes them.

Shared build revision `7926dfdfe19b2ee236f14f8084153b604dc1c4d5` binds the saved seven PASS to integration SHA-256 `c96aa74ceffe25c1461c5a97b73a35e322ad8babecc1d2a0175c0e6300b240c8`. Its private-anchor run is excluded. Followup source hashes: `auras.rs` `0d010b9d645f4875e9daa0b674f1d163ac969a51e08e5dd0194ece30327c86bd`, `game_data.rs` `931e22d3519ab6057ba2e58b817db9a471a32c83b04856fa3baa417fdc7ea68b`; fixture hash `4a160c4c8a42e1556ad778416f8e5718e78660a6445a2037343696c429559271`. Later concurrent bytes are not covered.

Exact retained row `prose-2026-03-12-029` receives bounded partial proof only and remains audit-pending: ordinary explicit host inputs do not establish the whole native five-boolean secrecy delta. Accounting remains **268 pending / 80 bounded / 14 partial = 362**, source SHA unchanged. Native combat/secret policies remain untested; no generic declassification, native parity, all-profile execution, public target-input model, or whole-row/page credit. Overall goal **INPROGRESS**.

## Known gaps (current cycle)

- [x] Parent observed compiled behavioral RED after fixing the private import: player `isRaid` mismatch, 6 PASS / 1 FAIL.
- [x] Producer changed the shared classification writer to use `is_raid`, `is_nameplate_only`, and `is_from_player_or_player_pet` directly; existing constructors and fixed target inputs remain unchanged.
- [x] Deterministic fixture followup independently accepted bounded saved seven PASS; initial source finding resolved without weakening assertions.
- [ ] Whole native five-boolean secrecy delta, combat/secret policies and broader acceptance remain open; requirement checkboxes above do not imply native verification.
- [ ] Target queries currently read fixed Rust fixtures, with no host-populated target aura store. Tests preserve that pathway rather than adding or pretending to exercise target inputs.

## Out of scope

- Secret quantitative aura fields, native classification/default inference, private-aura modeling, or native-client parity: the retained row does not establish those contracts.
- New filtering semantics (including RAID/nameplate filters), acquisition, private auras, events, lifecycle, admin setters, storage relocation, or unrelated refactors: producer slice changes only the required shared classification read path and dedicated spec.
- This docs reconciliation changes only this spec, wiki index/log and the exact retained row's bounded proof note. Private-anchor specs/code/tests, PLAN, other shared docs, delegation, builds/tests, push and deployment remain excluded.
