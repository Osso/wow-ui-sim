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

### Unknown native seam

- **VERIFIED (pinned Retail 12.1.5.70783 UI):** `Blizzard_Shared_StoreUISecure.lua` calls `GetBnetTransferInfo` near lines 3995 and 4115. One caller assigns the first result to `bnetAccountGUID`; another assigns the second result to `gameAccounts`. These assignments show caller use, not the native result contract.
- **VERIFIED (simulator source):** Retail sets `modeledStoreVas`, so the runtime-bootstrap VAS fallback is skipped; `src/c_api/` has no `GetBnetTransferInfo` registration.
- **UNKNOWN (R88al pending):** its native arguments, result values, backing source, events, and clearing rules remain unresolved. Do not infer behavior from the caller variable names or include the cross-BNet purchase path in the parity claim.

### Tests

**VERIFIED (test source):** `tests/store_secure_vas.rs` covers explicit lookup rows and row fields, ordering, empty and absent values, product-type mapping, correlated responses, original account/guild/StoreUI consumers, purchase preparation, and completion. The contract lists the tests for each API group.

## Sources

- [Retail StoreSecure VAS contract](../../specs/store-secure-vas.md).
- [Retail StoreSecure catalog and list state](store-secure-catalog.md).
- `src/c_api/store_vas_model.rs` and `src/c_api/store_vas_model/responses.rs` — VAS inputs, request records, and response callbacks.
- `src/c_api/store_catalog_model.rs` — product records used by the VAS service-type lookup.
- `src/c_api/c_store_secure_vas.rs` and child modules — Retail Lua API behavior.
- `src/c_api/c_store_secure.rs` and `src/c_api/mod.rs` — Retail surface selection and module registration.
- `src/lua_api/env_init/bootstrap.rs` — selects the Retail VAS model.
- `src/lua_api/env_init/runtime_surface_bootstrap.lua` — non-Retail compatibility path.
- Native evidence: R88ad.3–6 and R88g.3.q1 in the private task handoff; R88g.3.q1 establishes registration only. R88al remains pending for `GetBnetTransferInfo`. Consumer sources are from the pinned Retail 12.1.5.70783 cache.

## See Also

- [[store-secure-catalog]] — catalog, availability, and list state.
- [[store-api-availability]] — public Store status mirrors.
- [[consumer-selection-and-red-proof]] — selected consumer and producer-subtracted proof rules.
