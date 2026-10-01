# Major-faction renown rewards

State-backed `C_MajorFactions.GetRenownRewardsForLevel` snapshots from explicit pair-keyed inputs. Cached primary source: `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/MajorFactionsDocumentation.lua`, query lines 95–109, `MajorFactionRenownRewardInfo` declaration. See [C API boundary](../lua-api.md).

## What it must do

- [x] Start with an empty reward map; registering factions/levels must not fabricate rewards.
- [x] Publish explicit `(majorFactionID, renownLevel)` rows in stored vector order, with required fields and every declared optional field, preserving false versus absent.
- [x] Keep neighboring pairs isolated; subsequent queries reflect row updates/removal and return independent tables/rows.
- [x] Require numeric selectors. Authenticate secret unwrapping separately for each argument through the existing VM helper without clearing caller taint; tainted callers may pass public numbers.
- [x] Publish `isCollected` only for mainline retail/PTR 12.0.5+; preserve earlier/nonmainline field absence and existing query availability.

Checkboxes record implementation, not GREEN proof. **Inferred:** any absent pair returns an empty table; do not impose faction/level eligibility checks. Fractional selectors do not alias integer keys. These are bounded model choices, not native-verified behavior.

### Literal declared types

Query: required `majorFactionID: number`, `renownLevel: number`; required return `rewards: table<MajorFactionRenownRewardInfo>`. Source declares `SecretArguments = AllowedWhenUntainted`.

| Fields | Declared type | Nilable |
| --- | --- | --- |
| `renownRewardID`, `uiOrder` | `number` | false |
| `isAccountUnlock` | `bool` | false |
| `itemID`, `spellID`, `mountID`, `transmogID`, `transmogSetID`, `titleMaskID`, `transmogIllusionSourceID` | `number` | true |
| `icon` | `fileID` | true |
| `name`, `description`, `toastDescription` | `cstring` | true |
| `rewardType` | **`number`, not a declared enum** | true |
| `isCollected` | `bool` | true |

Rust inputs represent IDs/fileID as `i64`, order/reward type as `i32`, strings as `String`, and nilable values as `Option`. Integer representation and stored sequence order are bounded model choices, not native numeric constraints or ordering rules.

## Implementation inventory

- `src/c_api/c_major_factions/renown_rewards.rs`: existing `RenownRewardInfo`, strict numeric selector decoding, exact pair lookup, cloned snapshot publication, optional-field/profile gates.
- `src/c_api/c_major_factions.rs`: owning namespace registers the query unconditionally, consistent with existing C API registration and the former cross-profile default.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs`: existing empty pair-keyed map; unchanged by producer implementation.
- `src/lua_api/workarounds/temporary/major_faction_display_defaults.rs`: only the guarded `GetRenownRewardsForLevel` fallback removed; other defaults retained.

Only `isCollected` publication requires both `retail-12-0-5` and mainline `profile-retail`/`client-ptr`. No feature expansion or query-availability gate is introduced. Empty maps continue returning empty tables across profiles; earlier-profile publication remains without `isCollected` even when an explicit row supplies it.

## Behavioral proof ledger

Actual batch8 RED at `693883c77003589b24b9a555141ea51a3e71e1e9`, following input `766272cdc` and mainline fixture gate `5dfea9d6a`:

- Command: `timeout 90 target/debug/deps/integration-a11e89d240f9bd0c major_faction_renown_rewards:: --nocapture --test-threads=1`.
- `/tmp/patch-12.0.5-batch8-new-model-runs.json` binds the successful build artifact to that revision, SHA256 `33fb7f19a888209821fa80417201d1b5bc6a6f2899377813335df62b3e671924`, count 5, exit 101.
- `/tmp/patch-12.0.5-batch8-new-model-red-1.log`: **2/5 PASS, 3/5 FAIL**. Empty default and missing registered pair pass. Failures are `expected three modeled renown rewards`, `pair-specific reward counts`, and `initial modeled reward count`; compilation/missing-function errors are not the failure boundary.

`tests/major_faction_renown_rewards.rs` now has **nine** grouped cases under its unchanged mainline 12.0.5+ fixture gate. Four cases were added before handler changes: public selectors from tainted callers, secret selectors individually/both from secure versus tainted callers, required numeric inputs/non-truncation, and independent snapshot mutations. Existing five fixtures were not weakened. Added cases have no executed RED/GREEN evidence; parent owns the next build/proof. Original RED remains historical evidence, not proof for changed producer code.

## Known gaps (current cycle)

- [ ] Parent-owned compilation and GREEN for filter `major_faction_renown_rewards::` (nine cases).
- [ ] Parent-owned earlier/nonmainline regression proof for empty defaults and `isCollected` absence; current fixtures run only mainline 12.0.5+.
- [ ] Future native probe: populated/absent/unknown pairs, optional omissions/false, snapshot behavior, ordering, numeric limits, and secret policy.

Secret controls exercise VM-owned wrappers and stack taint only. Source annotation grounds the intended access policy; native secret provenance, output secrecy, general confidentiality, and real-client parity are unclaimed.

## Out of scope

New state/factions/production records, eligibility rules, events, shared structure specs, Cargo changes, vendor edits, other API changes, push/deploy, and worker-run builds/tests/checks/readability/broad gates.
