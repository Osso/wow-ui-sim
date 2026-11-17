# Retail StoreSecure VAS

Retail `C_StoreSecure` VAS support uses explicit host inputs and the existing StoreSecure state. It models local query, request, purchase, and completion behavior; it does not supply live server data or claim server acceptance.

## Content

### Consumer boundary

- **VERIFIED (Retail 12.1.5.70688, R88ad.3–6):** VAS lookups, request callbacks, purchase preparation, and completion results have native argument, return, and event contracts.
- **VERIFIED (pinned Retail UI cache 12.1.5.70783):** `Blizzard_Shared_StoreUISecure.lua` consumes realm, character, guild, purchase, and completion methods. `CharacterServicesAccountSelector.lua` consumes account GUID lookup and BNet validation.
- **VERIFIED (test source):** `tests/store_secure_vas.rs` loads the Retail StoreUI and GlueXML closures, checks original Lua source paths, and exercises their realm, account, guild, purchase, and completion call sites.

### Host state and requests

- **VERIFIED (simulator source):** `StoreVasState` is nested under `StoreSecureState` and defaults to empty inputs. It stores realm and character rows, account GUID maps, eligibility results, guild data, purchase data, completion data, failure state, and submitted requests.
- **VERIFIED (simulator source):** `StoreVasRequest` records typed request inputs. Request methods do not perform network I/O; connection-gated submissions update simulator request state. Unknown host inputs do not create records.
- **VERIFIED (simulator source):** the Retail VAS registration is separate from the catalog model. Non-Retail profiles retain their existing bootstrap behavior; Retail VAS findings do not establish Mists behavior.

### Response order

- **VERIFIED (simulator source and tests):** a correlated BNet success replaces the account map before the validation event; an error sends its numeric code without replacing the map; a stale sequence does nothing.
- **VERIFIED (simulator source and tests):** guild-master and guild-follow callbacks update their cached records before sending their respective events.
- **VERIFIED (simulator source and tests):** completion lookup publishes only supplied product and character data. A missing character can prepare a follow-up request. Failure acknowledgement clears the active marker; external transaction ID clearing removes only the supplied value.

### Tests

`tests/store_secure_vas.rs` covers explicit lookup rows, empty and absent data, correlated responses, original StoreUI callbacks, purchase request preparation, and the original completion handler. The spec maps each public contract to this test file.

## Sources

- [Retail StoreSecure VAS contract](../../specs/store-secure-vas.md).
- [Retail StoreSecure catalog and list state](store-secure-catalog.md).
- `src/c_api/store_vas_model.rs` — VAS inputs, request records, and response callbacks.
- `src/c_api/c_store_secure_vas.rs` and child modules — Retail Lua API behavior.
- `src/c_api/c_store_secure.rs` — Retail surface selection.
- `src/lua_api/env_init/runtime_surface_bootstrap.lua` — non-Retail compatibility path.
- Native evidence: R88ad.3–6 in the private task handoff; consumer sources are from the pinned Retail 12.1.5.70783 cache.

## See Also

- [[store-secure-catalog]] — catalog, availability, and list state.
- [[store-api-availability]] — public Store status mirrors.
- [[consumer-selection-and-red-proof]] — selected consumer and producer-subtracted proof rules.
