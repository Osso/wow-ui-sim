# Retail StoreSecure VAS API

This contract covers Retail `C_StoreSecure` VAS lookup, request, purchase, and completion methods backed by `src/c_api/`. Native behavior is from Retail 12.1.5.70688; consumer tests load the pinned Retail 12.1.5.70783 UI cache. See the [StoreSecure model](../wiki/systems/store-secure-catalog.md) for the existing Store state boundary.

## What it must do

### Realm, account, and character lookups

- [ ] `GetRealmList` returns one fresh array, including an empty array when no records exist. Rows contain `virtualRealmAddress`, `realmName`, `characterCount`, and optional Boolean `pvp` / `rp`; rows sort by descending character count.
- [ ] `GetVASRealmList` returns one fresh array with the same row keys, sets every `characterCount` to zero, and sorts the source VAS realm rows by ascending realm-name bytes before publication.
- [ ] `GetCharactersForRealm` requires a realm address and non-nil guild-only value, filters by realm, applies the GUID-backed guild-master filter when true, and returns a fresh array sorted by descending level then native full-GUID order.
- [ ] Character rows publish the twelve verified fields and value forms. Missing optional values remain nil; an empty GUID publishes nil.
- [ ] `GetCharacterInfoByGUID` reads the separate character-object input and returns one character row or nil when no matching object is supplied.
- [ ] `GetWoWAccountGUIDFromName` uses the current local account GUID when the local flag is truthy; otherwise it matches account-name bytes exactly. Empty GUIDs publish nil.
- [ ] `GetEligibleRacesForVASService` publishes nil when the host has no computed result, and a fresh array—including an empty array—for a supplied result. Rows contain `raceName`, `isAlliedRace`, and `isHeritageArmorUnlocked`.
- [ ] `GetVASGuildMasterInfoForCharacterByGUID` publishes nil on a miss or a fresh `{ guildName, guildMemberInfos }` table on a hit. Member rows contain `guid` and `memberName` in supplied order.
- [ ] `GetVasServiceType` maps the verified native product types to service types and returns nil for an absent product or unmapped type.
- [ ] Restricted calls return zero values before reading arguments. Required arguments use the verified shared secret-access and truthiness rules. Exact member-specific Usage text remains UNKNOWN and is not asserted.

### Requests, purchases, and completion

- [ ] Request and purchase methods use only explicit host inputs. Unknown accounts, characters, products, server responses, purchase rows, completion data, and failure state remain empty by default.
- [ ] `ValidateBnetTransfer`, `RequestRealmGuildMasterInfo`, and `RequestCharacterGuildFollowInfo` prepare asynchronous requests; they do not synthesize their response events or wait for responses.
- [ ] `PurchaseVASProduct` returns the verified local preparation result and records accepted requests only when connected. It does not claim server acceptance.
- [ ] `GetVASCompletionInfo`, `GetFailureInfo`, `AckFailure`, confirmation methods, and Nydus-link handling publish only the verified fields and state transitions. No inferred public return is asserted for UNKNOWN absence paths.

### Profile boundary

- [ ] Register the modeled VAS surface only for Retail. Preserve existing non-Retail behavior; do not promote Retail findings to Mists or other profiles.
- [ ] The original Retail StoreUI and character-service consumers use the registered methods after runtime bootstrap.

## How it works

- [Retail StoreSecure catalog and list state](../wiki/systems/store-secure-catalog.md).
- [Store API availability](../wiki/systems/store-api-availability.md).
- Native findings: private handoff `pr46-49-decomp-answers.md`, R88ad.3–6 and the cited R88q / R88w bullets.

## Implementation inventory

- `src/c_api/store_vas_model.rs` — empty-default VAS host inputs.
- `src/c_api/c_store_secure_vas.rs` and its child modules — Retail VAS registration and Lua publication.
- `src/c_api/c_store_secure.rs` — Retail surface selection.
- `src/lua_api/env_init/runtime_surface_bootstrap.lua` — retains legacy VAS defaults only outside Retail.
- `tests/store_secure_vas.rs` — selected Retail consumer and host-row tests.

## Tests asserting this spec

- `tests/store_secure_vas.rs` — original `StoreVASValidationFrame_Init`, realm selector, and character selector consumers; lookup row shapes and empty cases.
- Additional request, purchase, and completion assertions are added with their implementation commits.

## Known gaps (current cycle)

- [ ] Implement and test R88ad.4 asynchronous requests and correlated responses.
- [ ] Implement and test R88ad.5 purchase and confirmation paths.
- [ ] Implement and test R88ad.6 completion and failure paths.
- [ ] Keep complete network delivery, server acceptance, unknown cleanup guards, and every unknown absence return outside the parity claim.

## Out of scope

- Mists or other non-Retail native VAS parity.
- Real network transport, server-side eligibility computation, server purchase settlement, and complete response lifecycle guarantees.
