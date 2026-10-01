# Patch 12.0.5 structure inputs

Three empty C API backing inputs publish modeled structure snapshots through bounded mainline 12.0.5+ queries. Source authority is the cached retail `Blizzard_APIDocumentationGenerated` files only: `PvpInfoDocumentation.lua` (109–115, 1676–1695), `TransmogItemsDocumentation.lua` (196–209, 1163–1177), and `TransmogOutfitInfoDocumentation.lua` (521–536, 1003–1019), under `~/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/`. Fixture IDs are arbitrary test-local inputs, not production data.

## What it must do

- [ ] `structures-PvpBrawlInfo-667`: zero-argument `C_PvP.GetActiveBrawlInfo()` publishes one nilable record. Fields: `brawlID`, `name`, `shortDescription`, `longDescription`, `canQueue`, `minLevel`, `maxLevel`, `groupsAllowed`, `crossFactionAllowed`, nullable `timeLeftUntilNextChange`, enum `brawlType`, string sequence `mapNames`, `includesAllArenas`, `minItemLevel`, explicit boolean `shouldHideRewardIcon`. No `SecretArguments` metadata on this query. Empty input returns one nil (closest reading of the declared nilable return, pending native proof).
- [ ] `structures-TransmogAppearanceSourceInfoData-671`: `C_TransmogCollection.GetAppearanceSourceInfo(itemModifiedAppearanceID)` publishes `category`, `itemAppearanceID`, `canHaveIllusion`, `icon`, `isCollected`, `itemLink`, `transmoglink`, nullable `sourceType`, `itemSubclass`, explicit boolean `ignoreModelAttachmentChecksForIllusion`. Only `sourceType` is optional. One required numeric argument; `MayReturnNothing = true`, `SecretArguments = "AllowedWhenUntainted"`.
- [ ] `structures-ViewedTransmogOutfitSlotInfo-678`: `C_TransmogOutfitInfo.GetViewedOutfitSlotInfo(slot, type, option)` publishes `transmogID`, enum `displayType`, `isTransmogrified`, `hasPending`, `isPendingCollected`, `canTransmogrify`, enum `warning`, `warningText`, enum `error`, `errorText`, nullable fileID `texture`, enum `sheatheCategory`. Three required enum arguments; `MayReturnNothing = true`, `SecretArguments = "AllowedWhenUntainted"`.
- [ ] State pass-through assumptions: exact input-key selection, no cross-key leakage, fresh mutable Lua snapshots without changing inputs, optional-field omission, and replacement/removal reflected by subsequent queries. Zero returns for missing `MayReturnNothing` records is an inference. Numeric fixture enum values test pass-through, not native enum meaning or range validation.
- [ ] Appearance and viewed-slot queries accept secret arguments only from untainted callers through existing VM security; ordinary arguments remain usable by tainted callers. No generic declassification, taint clearing, or inferred permission checks.

## How it works

- [C API placement and runtime state](../../AGENTS.md#c-api-boundary).
- [Lua API architecture](../lua-api.md).

## Implementation inventory

- `src/c_api/c_pvp/brawl_info.rs`: explicit `PvpBrawlInfo` record.
- `src/c_api/c_transmog_collection/appearance_source_info.rs`: explicit `AppearanceSourceInfo` record.
- `src/c_api/c_transmog_outfit_info/viewed_slot_info.rs`: explicit `ViewedOutfitSlotInfo` record.
- `src/c_api/c_pvp/active_brawl.rs`: `GetActiveBrawlInfo` snapshot producer.
- `src/c_api/c_transmog_collection/appearance_sources.rs`: exact source-ID query.
- `src/c_api/c_transmog_outfit_info/viewed_slots.rs`: exact `(slot, type, option)` query.
- Owning parent registrations replace lazy namespace providers for only these three keys under `all(retail-12-0-5, any(profile-retail, client-ptr))`. Unrelated registrations and other-profile defaults remain unchanged.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs`: public type access, `active_brawl: None`, empty `transmog_appearance_sources` and `viewed_outfit_slots`, gated to `retail-12-0-5`.

## Tests asserting this spec

`tests/patch_12_0_5_structure_inputs.rs`: one grouped integration file, three focused tests per domain (nine total). Calls actual registered queries, requires populated tables before checking fields, exercises both boolean values/two sheathe values, identity, optional fields, snapshots, keys and removal. Fixtures are gated to `all(retail-12-0-5, any(profile-retail, client-ptr))`, allowing historical mainline builds without requiring current `client-retail`. Empty input storage may be shared; this does not claim cross-profile publication. No new Cargo target declaration.

## Known gaps (current cycle)

- [x] Actual parent batch-8 behavioral RED at `693883c77003589b24b9a555141ea51a3e71e1e9`: 0/9 pass, 9/9 fail. `/tmp/patch-12.0.5-batch8-new-model-runs.json` binds the successful-build binary SHA-256 `33fb7f19a888209821fa80417201d1b5bc6a6f2899377813335df62b3e671924`; `/tmp/patch-12.0.5-batch8-new-model-red-2.log` records populated table failures before field assertions, separate transmog zero-return arity failures, and brawl populated/removal boundary failure. Empty brawl one-nil control alone already passed.
- [x] Implemented three producers after that RED: all declared fields, optional omission, fresh tables/map array, exact integer keys, one brawl nil versus zero transmog results, authenticated secret unwrap per transmog selector before numeric decoding. No inferred records or booleans.
- [x] Batch9 `4f9e1607c`: successful compilation, `patch_12_0_5_structure_inputs::` 9/9 PASS, exit 0. `/tmp/patch-12.0.5-batch9-build-result.json` and `/tmp/patch-12.0.5-batch9-runs.json` bind revision/artifact; `/tmp/patch-12.0.5-batch9-integration-3.log` records three cases per producer: complete records/contrasting values, empty/removal arity, and snapshots/key isolation/secret controls where applicable. These are supplied-state model behaviors, not native parity.
- [ ] Independent producer audit 119 report `/tmp/patch-12.0.5-batch9-independent-proof.md`, final current-default fmt/check and startup after new query producers remain pending.
- [ ] Required-argument validation tests remain deferred: schema establishes requiredness, not exact native error/coercion behavior. Current numeric decoding rejects missing/non-number, non-finite, fractional and out-of-storage-range selectors rather than truncating to a different key; native validation/error wording remains an inference. Enum ranges are not validated.
- [ ] Future native probes: populated/empty return arity, all field values and optional omissions, enum ranges, snapshot mutation and exact secret-caller behavior. Cached shapes do not prove these simulator policies.

## Out of scope

SimState/schema changes, tests weakening, production game data, events or queue policy, outfit selection/editing, argument coercion heuristics, vendor changes, generic security changes, major-faction/GC/DamageMeter/RecentAllies/housing work, Cargo/build/check/test/readability/broad-gate/push/deploy execution.
