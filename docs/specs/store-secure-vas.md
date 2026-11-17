# Retail StoreSecure VAS API

This contract covers Retail `C_StoreSecure` VAS lookup, request, purchase, and completion methods backed by `src/c_api/`. Native behavior is from Retail 12.1.5.70688; consumer tests load the pinned Retail 12.1.5.70783 UI cache. See the [VAS system page](../wiki/systems/store-secure-vas.md) and [StoreSecure catalog model](../wiki/systems/store-secure-catalog.md).

## What it must do

### Realm, account, and character lookups

- [x] `GetRealmList` returns an array, including an empty array when no records exist. Rows contain `virtualRealmAddress`, `realmName`, `characterCount`, and optional Boolean `pvp` / `rp`; rows sort by descending character count.
- [x] `GetVASRealmList` returns an array with the same row keys, sets every `characterCount` to zero, and sorts the source VAS realm rows by ascending realm-name bytes before publication.
- [x] `GetCharactersForRealm` requires a realm address and non-nil guild-only value, filters by realm, applies the GUID-backed guild-master filter when true, and returns an array sorted by descending level then native full-GUID order.
- [x] Character rows publish the twelve verified fields and value forms. Missing optional values remain nil; an empty GUID publishes nil.
- [x] `GetCharacterInfoByGUID` reads the separate character-object input and returns one character row or nil when no matching object is supplied.
- [x] `GetWoWAccountGUIDFromName` uses the current local account GUID when the local flag is truthy; otherwise it matches account-name bytes exactly. Empty GUIDs publish nil.
- [x] `GetEligibleRacesForVASService` publishes nil when the host has no computed result, and an array—including an empty array—for a supplied result. Rows contain `raceName`, `isAlliedRace`, and `isHeritageArmorUnlocked`.
- [x] `GetVASGuildMasterInfoForCharacterByGUID` publishes nil on a miss or a `{ guildName, guildMemberInfos }` table on a hit. Member rows contain `guid` and `memberName` in supplied order.
- [x] `GetVasServiceType` maps the verified native product types to service types and returns nil for an absent product or unmapped type.
- [ ] Restricted calls return zero values before reading arguments. Required arguments use the verified shared secret-access and truthiness rules. Exact member-specific Usage text remains UNKNOWN and is not asserted.

### Requests, purchases, and completion

- [x] Request and purchase methods use only explicit host inputs. Unknown accounts, characters, products, server responses, purchase rows, completion data, and failure state remain empty by default.
- [x] `ValidateBnetTransfer` copies the supplied account-name bytes without trimming or case-folding, advances the request sequence, and submits only when connected. A new request does not clear the saved transfer snapshot, and the request does not emit a synchronous validation event.
- [x] A correlated BNet success replaces the saved transfer GUID, ordered account-name list, and name-to-GUID map before one nil-valued `VAS_TRANSFER_VALIDATION_UPDATE`; a matching error emits one signed integer without replacing these values. A stale response changes nothing.
- [x] `GetBnetTransferInfo` reads no arguments and returns exactly two values: the saved GUID as a string or nil, then a 1-based array of account-name strings. An empty saved GUID publishes nil, and an empty account list publishes an empty array.
- [x] `RequestRealmGuildMasterInfo` collects current character GUIDs for the realm and submits only when connected. Its response updates guild-master records before one `STORE_GUILD_MASTER_INFO_RECEIVED` realm number.
- [x] `RequestCharacterGuildFollowInfo` accepts a GUID and realm, then submits only when connected. Its response updates follow-info state before `STORE_GUILD_FOLLOW_INFO_RECEIVED` with a GUID and `{ transferredRealm?, factionChanged }`.
- [x] `OpenNydusLink` opens only a nonempty link from the matching entry; a missing entry or link does nothing.
- [x] `GetConfirmationInfo` publishes six populated values from a selected purchase record, with price parts derived from the native unsigned amounts. It does not consume confirmation state.
- [x] `GetUnrevokedBoostInfo` publishes the verified three-string result only when the supplied product, character, and realm records resolve; an absent result is not given a fabricated tuple.
- [x] `PurchaseProduct` accepts only supplied products whose host purchase predicate passes, advances the request sequence, and consumes the supplied external transaction ID before connection-gated submission.
- [x] `PurchaseProductConfirm` requires a selected record with state 9 and secondary status 0. It prepares confirmation or cancel state only for the verified accepted inputs; cents are narrowed to 16 bits and resulting values of 100 or more are rejected.
- [x] `PurchaseVASProduct` validates the supplied product, character, service fields, and transfer destination before preparing an asynchronous request. It does not claim server acceptance.
- [x] `GetVASCompletionInfo` publishes four values for a resolved completion record. When a product resolves but its character does not, it marks completion pending and may submit a follow-up request instead of publishing a fabricated tuple.
- [x] `GetFailureInfo` returns the mapped code or nil, followed by nil. `AckFailure` clears only the active marker and submits its acknowledgement only when connected.
- [x] `ClearPreGeneratedExternalTransactionID` clears the supplied transaction value and returns zero values. Unknown source data stays empty by default; UNKNOWN absence return paths are not asserted.

### Profile boundary

- [x] Register the modeled VAS surface only for Retail. Preserve existing non-Retail behavior; do not promote Retail findings to Mists or other profiles.
- [x] The original Retail StoreUI realm, character, BNet transfer, purchase, and completion consumers and the account selector use the registered methods after runtime bootstrap. The BNet transfer path is verified only for the tested local request/response inputs; it does not claim live server behavior.

## How it works

- [Retail StoreSecure VAS state and response flow](../wiki/systems/store-secure-vas.md).
- [Retail StoreSecure catalog and list state](../wiki/systems/store-secure-catalog.md).
- [Store API availability](../wiki/systems/store-api-availability.md).
- Native findings: private handoff `pr46-49-decomp-answers.md`, R88ad.3–6, R88g.3.q1, R88al, and the cited R88q / R88w bullets. R88al leaves the initial snapshot and context-dependent return-secret classification outside this claim.

## Implementation inventory

- `src/c_api/store_vas_model.rs` and `src/c_api/store_vas_model/responses.rs` — empty-default host inputs and response updates.
- `src/c_api/store_catalog_model.rs` — product records used by the VAS service-type lookup.
- `src/c_api/c_store_secure_vas.rs` and its child modules — Retail VAS registration and Lua publication.
- `src/c_api/c_store_secure.rs` and `src/c_api/mod.rs` — Retail surface selection and module registration.
- `src/lua_api/env_init/bootstrap.rs` — selects the Retail VAS model.
- `src/lua_api/env_init/runtime_surface_bootstrap.lua` — retains legacy VAS defaults only outside Retail.
- `tests/store_secure_vas.rs` — selected Retail consumer and host-row tests.

## Tests asserting this spec

- `tests/store_secure_vas.rs::retail_store_vas_realm_and_character_consumers_use_explicit_rows` — original Retail realm/character panel; callable wrong-producer RED checks cover each lookup producer.
- `tests/store_secure_vas.rs::retail_store_vas_lookup_apis_publish_explicit_host_rows` and `retail_store_vas_service_type_mapping_matches_product_types` — complete row fields, ordering, empty/missing values, and every verified product-type mapping.
- `tests/store_secure_vas.rs::retail_store_bnet_validation_uses_the_original_account_selector`, `retail_store_bnet_error_preserves_account_map_and_notifies_original_selector`, `retail_store_bnet_validation_request_preserves_the_transfer_snapshot`, and `retail_store_bnet_transfer_info_uses_original_transfer_consumer` — account validation and the original `Blizzard_Shared_StoreUISecure.lua` BNet transfer purchase consumer; the latter covers two success snapshots, error, stale response, empty GUID, and callable-producer RED.
- `tests/store_secure_vas.rs::retail_store_vas_guild_master_request_uses_the_original_panel` and `retail_store_vas_guild_follow_request_uses_the_original_character_callback` — guild request/response behavior.
- `tests/store_secure_vas.rs::retail_store_purchase_confirmation_checks_selected_state_and_cents`, `retail_store_vas_purchase_uses_original_continue_button`, and `retail_store_vas_completion_uses_original_store_ui_handler` — purchase, confirmation, and completion consumers.
- `tests/store_secure_vas.rs::retail_store_vas_transfer_completion_and_missing_character_request_are_modeled` and `retail_store_vas_purchase_and_completion_use_explicit_host_inputs` — supplied completion, failure, and empty-default inputs.

## Known gaps (current cycle)

- [ ] The native snapshot before any validation remains UNKNOWN. The simulator keeps this host input empty by default; tests do not claim a native initial value.
- [ ] Return-secret classification depends on publication and execution context and is not asserted without secret-harness coverage. Restricted-call entry and every VAS member's required-argument error path also remain unverified; member-specific Usage text remains UNKNOWN and is not asserted.

## Out of scope

- Mists or other non-Retail native VAS parity.
- Real network transport, server-side eligibility computation, server purchase settlement, and complete response lifecycle guarantees.
