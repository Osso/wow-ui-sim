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

**VERIFIED (simulator source):** the clear operation sets the Aurora state to 1, removes realm-list availability, resets connection-info input, clears the pending login step and connected flag, and removes a stored login error. The Retail-only `disconnect_state_is_authoritative` marker makes `GetState` use this model snapshot after disconnect even when `Login` was not submitted; it is a simulator marker, not a native field. **VERIFIED (current Retail test at HEAD `74ffa9dca6ee7993a184c5a66b244232b8645f6a`):** the direct-screen original-consumer case checks public `GetState` before and after disconnect. **VERIFIED (simulator source):** `RetailGlueState::default` sets the local cancellation marker to false. **UNKNOWN (native):** cancellation initialization and reset writers beyond the R88aa conditional.

### Character-list pending state

**VERIFIED (simulator source):** `character_list_completion_state` is an integer and its empty default is 0. **VERIFIED (simulator source and Retail consumer test):** the getter returns true when the state is 0 and false when it is nonzero. The model default is not evidence of native startup state.

**VERIFIED (simulator source):** the modeled successful-connection path stores 0 before it fires argumentless `CHARACTER_LIST_RETRIEVING`. Full-list processing stores 0 and then 1 before it fires `CHARACTER_LIST_UPDATE`. **VERIFIED (R88aa):** two reset routines and `GetCharacterListUpdate` also write 0; **UNMODELED:** those three reset paths are not wired into this simulator flow. **VERIFIED (current Retail tests at HEAD `74ffa9dca6ee7993a184c5a66b244232b8645f6a`):** original queue-status consumers pass with two distinct host-backed WoW GUIDs, absent/empty cases, completion states 0/1, and host-seeded state 2. The two backed rows have an explicit `WaitingOnQueue` VAS host record so the UI uses its queue branch; the test does not claim `IsCharacterVASLocked` behavior. A constant-true pending getter makes both discriminating tests fail. State 2 checks the generic nonzero predicate and is not a claimed native writer. Native initial completion and indirect writers remain **UNKNOWN**.

### Consumer and verification boundary

**VERIFIED (local Retail cache 12.1.5.70783):** the selected Mainline Glue TOC loads `Mainline/CharacterSelect.lua`. The original `OnShow` calls `C_Login.DisconnectFromServer` at line 211. The original queue-status handler calls `IsCharacterListUpdatePending` at line 697, and another cached handler calls it at line 689. The same `OnShow` clears `GetDisconnectOnLogout` before it calls Disconnect; a repeated `OnShow` does not call Disconnect again until the host reseeds that selector. **INFERRED (R88aa):** those callers match native Retail 12.1.5.70688; the cached Lua build is not byte-matched to that native build. The task's line 720 is not the line in the local cache.

**VERIFIED (selector mutants at code/test HEAD `74ffa9dca6ee7993a184c5a66b244232b8645f6a`):** no-op Disconnect, constant-true pending-getter, and P1 projection mutants fail their exact original-consumer tests. Six exact integration GREEN tests pass, including one-shot event order and StoreGlue replacement. **VERIFIED (cleanup unit at HEAD `41047ceedec6b85b0af18d4138713aebf0d5991c`):** stored login error and disconnect state clear. **VERIFIED (five-profile compile matrix):** PTR, Forever, Mists, Wrath, and Retail complete with zero warnings in that order; the first three receipts at `74ffa9d` are source-equivalent because the diff to `41047ceed` changes only documentation. **VERIFIED (Retail runtime):** `--no-addons --no-saved-vars lua-errors` prints `[]`. See the linked investigation and handoff report for hashes.

### Implementation and tests

**VERIFIED (source tree):** `src/c_api/c_login.rs` owns Retail gating and the offline login/list progression; `src/c_api/c_login/retail_lifecycle.rs` owns Retail Glue registration, cancellation state, completion state, and the focused cleanup unit test.

**VERIFIED (source and current test runs):** `tests/glue_login.rs` invokes the original Retail `OnShow` and queue-status consumers. The direct-screen test checks public `GetState` without calling Login; disconnect tests cover pending and connected attempts; the cancellation condition is seeded false and the test asserts only its conditional notification, not native initialization. The queue tests assert two backed roster GUIDs, absent/empty cases, and 0/1/2 getter inputs. The host `WaitingOnQueue` records keep row rendering on the selected queue branch; `IsCharacterVASLocked` remains unmodeled and unclaimed. Six exact integration tests and the focused cleanup unit pass. The five-profile compile matrix has zero warnings, and Retail `lua-errors` returns `[]`.

**VERIFIED (current optional-tests integration run at HEAD `74ffa9dca6ee7993a184c5a66b244232b8645f6a`):** `tests/glue_character_select.rs` no longer treats missing `C_Login.DisconnectFromServer` as an expected error; `storeglue_r88q_host::shared_boolean_writers_drive_original_glue_update` passes with shared state and the original update consumer.

## Sources

- [Glue login contract](../../specs/glue-login-flow.md) — native contract, simulator policy, and open evidence boundaries.
- [Consumer selection and RED proof](../investigations/consumer-selection-and-red-proof.md#retail-glue-login-consumers-and-pending-list-cache-call-site-r88aa) — selected cache consumer and mutation proof.
- [Client profiles](client-profiles.md) — profile-specific runtime loading.
- [Lua API](lua-api.md) — C API and Lua registration boundary.

## See Also

- [[event-system]] — event publication and script dispatch.
- [[client-profiles]] — active-profile selection.
- [[lua-api]] — public Lua API registration.
