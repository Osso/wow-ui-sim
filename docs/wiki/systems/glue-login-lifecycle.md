# Retail Glue login lifecycle

**VERIFIED (R88aa and simulator source):** Retail Glue publishes two login/list APIs from the modeled login state. This page separates current simulator behavior from native facts that remain unknown.

## Content

### API scope

**VERIFIED (R88aa and Retail consumer tests):** Glue registers `C_Login.DisconnectFromServer` and the global `IsCharacterListUpdatePending`. The disconnect function has no arguments and returns no values. The pending getter has no arguments and returns one Boolean. Both registrations are removed when the screen changes to Game; the getter is a global, not a `C_Login` member. **VERIFIED (R88aa):** native Glue publication uses flag 3; the simulator test surface does not expose publication flags.

**VERIFIED (simulator source):** `src/c_api/c_login/retail_lifecycle.rs::update_screen_registration` uses ordinary Lua API registration. This lane does not manually publish the APIs to the secure environment.

### Disconnect state transition

**VERIFIED (simulator source and Retail consumer test):** `DisconnectFromServer` first clears the modeled connection state, then fires:

1. `AUTO_REALM_SWAP(false)`.
2. `LAUNCHER_LOGIN_STATUS_CHANGED` with no arguments when cancellation was not already recorded.
3. `LOGIN_STATE_CHANGED` with no arguments.

**VERIFIED (simulator source and Retail state-cleanup unit test):** the clear operation sets the Aurora state to 1, removes realm-list availability, resets connection-info input, clears the pending login step and connected flag, and removes a stored login error. `RetailGlueState::default` starts the cancellation marker as false. That local default and any native reset writer remain **UNKNOWN** beyond the R88aa evidence.

### Character-list pending state

**VERIFIED (simulator source):** `character_list_completion_state` is an integer and its empty default is 0. **VERIFIED (simulator source and Retail consumer test):** the getter returns true when the state is 0 and false when it is nonzero. The model default is not evidence of native startup state.

**VERIFIED (simulator source and Retail consumer test):** the modeled successful-connection path stores 0 before it fires argumentless `CHARACTER_LIST_RETRIEVING`. Full-list processing stores 0 and then 1 before it fires `CHARACTER_LIST_UPDATE`. The test observes the original queue-status handler defer while the list is pending and process the completed list. The exact initial native value and reset writers outside these modeled paths remain **UNKNOWN**.

### Consumer and verification boundary

**VERIFIED (local Retail cache 12.1.5.70783):** the selected Mainline Glue TOC loads `Mainline/CharacterSelect.lua`. The original `OnShow` calls `C_Login.DisconnectFromServer` at line 211. The original queue-status handler calls `IsCharacterListUpdatePending` at line 697, and another cached handler calls it at line 689. **INFERRED (R88aa):** those callers match native Retail 12.1.5.70688; the cached Lua build is not byte-matched to that native build. The task's line 720 is not the line in the local cache.

**VERIFIED (hash-bound mutation and GREEN runs):** the mutation removes only the two Retail registrations. With the unchanged test source, each original-consumer test fails at its consumer call; the producer-backed tests pass. The source hash, mutation patch hash, executable/dylib hashes, CVar snapshots, and runtime transcripts are recorded in the lane handoff.

### Implementation and tests

**VERIFIED (source tree):** `src/c_api/c_login.rs` owns Retail gating and the offline login/list progression; `src/c_api/c_login/retail_lifecycle.rs` owns Retail Glue registration, cancellation state, completion state, and the focused cleanup unit test.

**VERIFIED (test source and receipts):** `tests/glue_login.rs` exercises the original Retail `OnShow` and queue-status consumers, event ordering/payloads, return arity, list state, and Game-scope absence. `tests/glue_character_select.rs` no longer treats missing `C_Login.DisconnectFromServer` as an expected error; its retained StoreGlue test exercises shared state and the original update consumer.

## Sources

- [Glue login contract](../../specs/glue-login-flow.md) — native contract, simulator policy, and open evidence boundaries.
- [Consumer selection and RED proof](../investigations/consumer-selection-and-red-proof.md#retail-glue-login-consumers-and-pending-list-cache-call-site-r88aa) — selected cache consumer and mutation proof.
- [Client profiles](client-profiles.md) — profile-specific runtime loading.
- [Lua API](lua-api.md) — C API and Lua registration boundary.

## See Also

- [[event-system]] — event publication and script dispatch.
- [[client-profiles]] — active-profile selection.
- [[lua-api]] — public Lua API registration.
