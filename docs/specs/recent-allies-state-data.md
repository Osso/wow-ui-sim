# Recent Allies state-data rename

Bounded 12.0.5 audit slice `structures-RecentAllyStateData-669`: replace `hasFriendRequestPending` with `friendRequestSentThisSession` in explicit Recent Allies snapshots. Primary contract: `/home/osso/.cache/wow-ui-sim/blizzard-ui/retail/AddOns/Blizzard_APIDocumentationGenerated/RecentAlliesDocumentation.lua`, `GetRecentAllies` and its nested structures. [Audit context](../wiki/investigations/patch-12-0-5-api-audit.md).

## What it must do

- [ ] Own an explicit C API input, initially disabled with no rows; never synthesize production allies.
- [ ] Query only `C_RecentAllies.GetRecentAllies()` with zero arguments. Disabled input returns zero values; enabled empty input returns exactly one empty table.
- [ ] Publish complete `RecentAllyData` rows with `stateData`, `characterData`, and `interactionData`, including explicit interaction sequences and context tables.
- [ ] Publish distinct true/false `friendRequestSentThisSession` values with no `hasFriendRequestPending` alias. Preserve all other required values and optional nils.
- [ ] Return independent nested snapshots; Lua mutations must not alter input or subsequent results.
- [ ] Limit this modeled surface and fixtures to `retail-12-0-5` plus mainline `profile-retail`/`client-ptr`.

### Declared nested fields

| Structure | Required fields | Optional fields |
| --- | --- | --- |
| `RecentAllyStateData` | `isOnline`, `isDND`, `isAFK`, `isConvertedLegacyFriend`, `friendRequestSentThisSession`: bool | `pinExpirationDate`: time_t; `currentLocation`: string |
| `RecentAllyCharacterData` | `guid`: WOWGUID; `name`, `fullName`, `realmName`: string; `level`, `classID`, `raceID`: number; `sex`: UnitSex | none |
| `RecentAllyInteractionData` | `interactions`: table of RecentAllyInteraction | `note`: string |
| `RecentAllyInteraction` | `type`: RolodexType; `description`: cstring; `timestamp`: time_t; `contextData`: RecentAllyInteractionContextData | none |
| `RecentAllyInteractionContextData` | none | `itemID`, `activityDifficultyID`, `activityDifficultyLevel`: number; `locationName`: cstring |

**Source evidence:** `GetRecentAllies` has no declared arguments, returns nonnil `table<RecentAllyData>`, and declares `RequiresRecentAllies = true`. The predicate declares `FailureMode = "ReturnNothing"`.

**Inference, not native evidence:** one explicit `enabled` boolean models predicate eligibility; disabled is initial simulator policy. Enabled with no supplied rows returns an empty table. Stored vector order, independent snapshots, and integer Rust representations (`i64` timestamps/item IDs; `i32` numbers/enum values) are bounded model choices, not native ordering, numeric-limit or ownership claims. No support/readiness state is invented.

## How it works

- [C API boundary and environment state](../lua-api.md).
- [12.0.5 audit evidence boundaries](../wiki/investigations/patch-12-0-5-api-audit.md).

## Implementation inventory

- `src/c_api/c_recent_allies.rs`: explicit input and all nested rows; no registration or serializer.
- `src/c_api/mod.rs`: gated public input module.
- `src/lua_api/state/sim_state.rs`: per-environment input storage.
- `src/lua_api/state.rs`: disabled, empty input initialization.
- `tests/recent_allies_state_data.rs`: four actual-query cases in the existing generated integration harness; no new Cargo target.

## Tests asserting this spec

`tests/recent_allies_state_data.rs` covers disabled populated input, enabled empty input, two populated rows with opposite renamed flags and concrete nested/optional data, and deep result mutation versus input and later snapshots.

Proof ledger: no compilation, RED, GREEN, check, or runtime execution in this slice. Fixtures call the actual registered query without test-time replacement. Query producer remains unchanged intentionally; parent owns build and actual RED before implementing publication. All checkboxes remain unverified. Formatting is not behavioral proof.

## Known gaps (current cycle)

- [ ] Parent: build grouped fixtures and capture actual RED; implement only this query's registration/serializer after RED, then establish GREEN and final gates.
- [ ] Future native probe: query disabled/enabled empty/populated states; record return counts, nested nils, renamed true/false flags, ordering and mutations across repeated queries. Native eligibility and default state remain unknown.

## Out of scope

Lookup, search, mutation, readiness/support queries, events, location preference, production records, earlier/nonmainline behavior, vendor/cache edits, Cargo targets, delegation, push, and worker-run builds/checks/runtime tests. No full-audit completion or native parity credit.
