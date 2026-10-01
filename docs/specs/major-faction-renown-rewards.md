# Major-faction renown rewards

Explicit pair-keyed inputs for `C_MajorFactions.GetRenownRewardsForLevel`; no producer is implemented in this slice. Cached primary source: `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/MajorFactionsDocumentation.lua`, query lines 95–109, structure lines 265–297. See [C API boundary](../lua-api.md).

## What it must do

- [ ] Start with an empty reward map; registering factions/levels must not fabricate rewards.
- [ ] For explicit `(majorFactionID, renownLevel)` inputs, publish three fixture rows preserving sequence, identity, all declared optional fields, and `isCollected = true`, `false`, or absent.
- [ ] Keep neighboring faction and level pairs isolated; reflect row updates/removal on subsequent queries.
- [ ] **Inferred:** an absent pair for a registered faction/level returns an empty table. Unknown-faction eligibility is not specified.

### Literal declared types

Query: required `majorFactionID: number`, `renownLevel: number`; required return `rewards: table<MajorFactionRenownRewardInfo>`. `SecretArguments = AllowedWhenUntainted` is declared but not tested here.

| Fields | Declared type | Nilable |
| --- | --- | --- |
| `renownRewardID`, `uiOrder` | `number` | false |
| `isAccountUnlock` | `bool` | false |
| `itemID`, `spellID`, `mountID`, `transmogID`, `transmogSetID`, `titleMaskID`, `transmogIllusionSourceID` | `number` | true |
| `icon` | `fileID` | true |
| `name`, `description`, `toastDescription` | `cstring` | true |
| `rewardType` | **`number`, not a declared enum** | true |
| `isCollected` | `bool` | true |

Rust inputs represent IDs/fileID as `i64`, order/reward type as `i32`, strings as `String`, and nilable values as `Option`. Integer representation and stored sequence order are bounded model choices, not native-verified numeric constraints or ordering rules.

## How it works

- [Lua API/state architecture](../lua-api.md)

## Implementation inventory

- `src/c_api/c_major_factions/renown_rewards.rs`: explicit `RenownRewardInfo` inputs; no record default or Lua producer.
- `src/c_api/c_major_factions.rs`: public type export only; existing registrations unchanged.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs`: public pair-keyed map initialized empty, unconditionally; no publication gate implied.
- `src/lua_api/workarounds/temporary/major_faction_display_defaults.rs`: existing empty-table fallback retained unchanged until producer work.

## Tests asserting this spec

`tests/major_faction_renown_rewards.rs`: five grouped tests gated by `client-retail` and `retail-12-0-5`, querying the actual registered namespace function. No Cargo manifest or existing test registry edits.

Actual compilation and behavioral RED are **pending**; no tests/checks run in this slice. Intended first RED: `expected three modeled renown rewards` because the retained fallback returns `{}` instead of three explicit fixture rows. Empty/default controls may already pass; they do not establish publication.

## Known gaps (current cycle)

- [ ] Parent build batch must establish compilation and actual RED before producer implementation.
- [ ] Implement state-backed publication and retire only the matching fallback in a later slice.
- [ ] Future native probe: populated and absent pairs, optional omissions/false, ordering, integer limits, unknown-faction eligibility, and secret argument policy.

## Out of scope

Native parity, fabricated production rewards, eligibility rules, events, vendor changes, producer changes, fallback removal, other-profile publication, and push/deploy are excluded from this input-only slice.
