# Retail StoreSecure VAS API

This contract covers Retail `C_StoreSecure` VAS lookup, request, purchase, and completion methods backed by `src/c_api/`. Native behavior is from Retail 12.1.5.70688; consumer tests load the pinned Retail 12.1.5.70783 UI cache. See the [StoreSecure model](../wiki/systems/store-secure-catalog.md) for the existing Store state boundary.

## What it must do

### Realm, account, and character lookups

- [x] `GetRealmList` returns one fresh array, including an empty array when no records exist. Rows contain `virtualRealmAddress`, `realmName`, `characterCount`, and optional Boolean `pvp` / `rp`; rows sort by descending character count.
- [x] `GetVASRealmList` returns one fresh array with the same row keys, sets every `characterCount` to zero, and sorts the source VAS realm rows by ascending realm-name bytes before publication.
- [x] `GetCharactersForRealm` requires a realm address and non-nil guild-only value, filters by realm, applies the GUID-backed guild-master filter when true, and returns a fresh array sorted by descending level then native full-GUID order.
- [x] Character rows publish the twelve verified fields and value forms. Missing optional values remain nil; an empty GUID publishes nil.
- [x] `GetCharacterInfoByGUID` reads the separate character-object input and returns one character row or nil when no matching object is supplied.
- [x] `GetWoWAccountGUIDFromName` uses the current local account GUID when the local flag is truthy; otherwise it matches account-name bytes exactly. Empty GUIDs publish nil.
- [x] `GetEligibleRacesForVASService` publishes nil when the host has no computed result, and a fresh array—including an empty array—for a supplied result. Rows contain `raceName`, `isAlliedRace`, and `isHeritageArmorUnlocked`.
- [x] `GetVASGuildMasterInfoForCharacterByGUID` publishes nil on a miss or a fresh `{ guildName, guildMemberInfos }` table on a hit. Member rows contain `guid` and `memberName` in supplied order.
- [x] `GetVasServiceType` maps the verified native product types to service types and returns nil for an absent product or unmapped type.
- [ ] Restricted calls return zero values before reading arguments. Required arguments use the verified shared secret-access and truthiness rules. Exact member-specific Usage text remains UNKNOWN and is not asserted.

### Requests, purchases, and completion

- [ ] Request and purchase methods use only explicit host inputs. Unknown accounts, characters, products, server responses, purchase rows, completion data, and failure state remain empty by default.
- [ ] `ValidateBnetTransfer` copies the supplied account-name bytes without trimming or case-folding, advances the request sequence, and submits only when connected. It does not emit a synchronous validation event.
- [ ] A correlated BNet success replaces the supplied destination account and account-name map before one nil-valued `VAS_TRANSFER_VALIDATION_UPDATE`; a correlated error emits one signed integer and does not replace the map. Stale responses do nothing.
- [ ] `RequestRealmGuildMasterInfo` collects current character GUIDs for the realm and submits only when connected. Its response updates guild-master records before one `STORE_GUILD_MASTER_INFO_RECEIVED` realm number.
- [ ] `RequestCharacterGuildFollowInfo` accepts a GUID and realm, then submits only when connected. Its response updates follow-info state before `STORE_GUILD_FOLLOW_INFO_RECEIVED` with a GUID and `{ transferredRealm?, factionChanged }`.
- [ ] `OpenNydusLink` opens only a nonempty link from the matching entry; a missing entry or link does nothing.
- [ ] `GetConfirmationInfo` publishes six populated values from a selected purchase record, with price parts derived from the native unsigned amounts. It does not consume confirmation state.
- [ ] `GetUnrevokedBoostInfo` publishes the verified three-string result only when the supplied product, character, and realm records resolve; an absent result is not given a fabricated tuple.
- [ ] `PurchaseProduct` accepts only supplied products whose host purchase predicate passes, advances the request sequence, and consumes the supplied external transaction ID before connection-gated submission.
- [ ] `PurchaseProductConfirm` requires a selected record with state 9 and secondary status 0. It prepares confirmation or cancel state only for the verified accepted inputs; cents are narrowed to 16 bits and resulting values of 100 or more are rejected.
- [ ] `PurchaseVASProduct` validates the supplied product, character, service fields, and transfer destination before preparing an asynchronous request. It does not claim server acceptance.
- [ ] `GetVASCompletionInfo` publishes four values for a resolved completion record. When a product resolves but its character does not, it marks completion pending and may submit a follow-up request instead of publishing a fabricated tuple.
- [ ] `GetFailureInfo` returns the mapped code or nil, followed by nil. `AckFailure` clears only the active marker and submits its acknowledgement only when connected.
- [ ] `ClearPreGeneratedExternalTransactionID` clears the supplied transaction value and returns zero values. Unknown source data stays empty by default; UNKNOWN absence return paths are not asserted.

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

## Known gaps (current cycle)

- [ ] Verify the R88ad.4 request and correlated-response contract against the selected Retail consumers.
- [ ] Verify R88ad.5 purchase and confirmation paths.
- [ ] Verify R88ad.6 completion and failure paths.
- [ ] Keep complete network delivery, server acceptance, unknown cleanup guards, and every unknown absence return outside the parity claim.

## Out of scope

- Mists or other non-Retail native VAS parity.
- Real network transport, server-side eligibility computation, server purchase settlement, and complete response lifecycle guarantees.
