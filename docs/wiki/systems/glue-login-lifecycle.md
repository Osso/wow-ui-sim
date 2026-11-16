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

**VERIFIED (simulator source):** the clear operation sets the Aurora state to 1, removes realm-list availability, resets connection-info input, clears the pending login step and connected flag, and removes a stored login error. The Retail-only `disconnect_state_is_authoritative` marker makes `GetState` use this model snapshot after disconnect even when `Login` was not submitted; it is a simulator marker, not a native field. **VERIFIED (current regression test source; run pending):** the direct-screen original-consumer case checks public `GetState` before and after disconnect. `RetailGlueState::default` starts the cancellation marker as false. That local default and any native reset writer remain **UNKNOWN** beyond the R88aa evidence.

### Character-list pending state

**VERIFIED (simulator source):** `character_list_completion_state` is an integer and its empty default is 0. **VERIFIED (simulator source and Retail consumer test):** the getter returns true when the state is 0 and false when it is nonzero. The model default is not evidence of native startup state.

**VERIFIED (simulator source):** the modeled successful-connection path stores 0 before it fires argumentless `CHARACTER_LIST_RETRIEVING`. Full-list processing stores 0 and then 1 before it fires `CHARACTER_LIST_UPDATE`. **VERIFIED (R88aa):** two reset routines and `GetCharacterListUpdate` also write 0; **UNMODELED:** those three reset paths are not wired into this simulator flow. The current consumer tests use two host-backed roster GUIDs, an absent GUID, an empty roster, and a host-seeded state 2; execution is pending. State 2 tests the verified generic nonzero predicate and is not a claimed native writer. The native initial value and any indirect writers remain **UNKNOWN**.

### Consumer and verification boundary

**VERIFIED (local Retail cache 12.1.5.70783):** the selected Mainline Glue TOC loads `Mainline/CharacterSelect.lua`. The original `OnShow` calls `C_Login.DisconnectFromServer` at line 211. The original queue-status handler calls `IsCharacterListUpdatePending` at line 697, and another cached handler calls it at line 689. **INFERRED (R88aa):** those callers match native Retail 12.1.5.70688; the cached Lua build is not byte-matched to that native build. The task's line 720 is not the line in the local cache.

**VERIFIED (previous candidate):** removing both Retail registrations made the earlier original-consumer tests fail inside their cached consumer calls. **PENDING (current candidate):** execute separate no-op Disconnect and constant-true pending-getter mutations against the revised unchanged test source. Preserve each patch, build receipt, exact list/run, and executable/dylib/CVar hashes in the lane handoff.

### Implementation and tests

**VERIFIED (source tree):** `src/c_api/c_login.rs` owns Retail gating and the offline login/list progression; `src/c_api/c_login/retail_lifecycle.rs` owns Retail Glue registration, cancellation state, completion state, and the focused cleanup unit test.

**VERIFIED (test source):** `tests/glue_login.rs` invokes the original Retail `OnShow` and queue-status consumers. The new direct-screen test does not call `C_Login.Login` before it checks public `GetState`; the disconnect state tests use both connecting and connected login states. The cancellation condition is seeded false through a host-state seam before tests assert its notification; this does not establish native initialization. The queue tests use two default host-backed WoW accounts, an absent roster GUID, an empty roster, and a host-seeded nonzero value. **INFERRED from test source until runs complete:** these fixtures reject constant/no-op producers; exact mutation transcripts are pending.

**VERIFIED (test source and prior receipt):** `tests/glue_character_select.rs` no longer treats missing `C_Login.DisconnectFromServer` as an expected error; its retained StoreGlue test exercises shared state and the original update consumer.

## Sources

- [Glue login contract](../../specs/glue-login-flow.md) — native contract, simulator policy, and open evidence boundaries.
- [Consumer selection and RED proof](../investigations/consumer-selection-and-red-proof.md#retail-glue-login-consumers-and-pending-list-cache-call-site-r88aa) — selected cache consumer and mutation proof.
- [Client profiles](client-profiles.md) — profile-specific runtime loading.
- [Lua API](lua-api.md) — C API and Lua registration boundary.

## See Also

- [[event-system]] — event publication and script dispatch.
- [[client-profiles]] — active-profile selection.
- [[lua-api]] — public Lua API registration.
