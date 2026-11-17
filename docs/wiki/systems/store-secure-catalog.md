# Retail StoreSecure catalog

This system models Retail `C_StoreSecure` catalog and list state as host-supplied input. It replaces seeded catalog guesses only on Retail; it does not supply live server rows.

## Content

### Evidence and consumer boundary

- **VERIFIED (Retail 12.1.5.70688, R88ad.1/R88ad.2):** group, entry, product, availability, currency, purchase-list, dynamic-price, and request paths have the documented filters, return shapes, state changes, and notification order.
- **VERIFIED (pinned Retail UI cache 12.1.5.70783):** `Blizzard_Shared_StoreUITemplates.lua:88,281-286` consumes group metadata and product groups; `Blizzard_Shared_StoreUISecure.lua:451,599,657-716,1120,1200-1222,1305-1310,1590-1638` consumes currency, entries, group rows, list readiness, requests, and response events. `Blizzard_CatalogShop.lua:19-33,396-407,760` consumes product details, list readiness, and currency.
- **VERIFIED (pinned Retail 12.1.5.70783 generated API directory and native R88ad findings):** no `C_StoreSecure` declarations are present; method argument and result behavior is sourced from R88ad, while the original consumer call sites establish runtime use.
- **VERIFIED (Retail consumer source):** `Blizzard_EnvironmentCleanup/EnvironmentCleanup.lua:3` clears the public `C_StoreSecure` binding. The cleanup regression checks that catalog methods remain usable in `secureenv` before and after post-load restoration.

### State and response flow

- **VERIFIED (simulator source):** `StoreSecureState` lives inside `StoreGlueState`. Its default catalog, purchase rows, dynamic-price map, and currency map are empty; readiness flags are false, currency ID is zero, and connection distribution readiness is absent.
- **VERIFIED (simulator source):** the Retail response entry points accept host values. Availability writes before `STORE_STATUS_CHANGED`; product-list response state and ordered catalog rows write before `STORE_PRODUCTS_UPDATED`; purchase rows/readiness write before its notification is queued; dynamic-price data writes before `DYNAMIC_BUNDLE_PRICE_UPDATED`.
- **VERIFIED (Retail 12.1.5.70688 R88ad.2 and simulator source):** a successful or failed purchase-list response queues a zero-payload notification when its callback is installed. Pending responses coalesce into one dispatch.
- **INFERRED (simulator policy):** `WowLuaEnv::fire_on_update_timed` flushes the purchase-list notification after login advancement and before frame OnUpdate handlers. Native evidence establishes queued delivery after the state change, but not this exact dispatch boundary.
- **VERIFIED (Retail 12.1.5.70688 and simulator source):** the disconnect input clears availability before resetting the verified list readiness, region-lock, response-error, currency-ID, dynamic-price, and connection fields. The full cleanup of catalog and purchase rows is UNKNOWN and is not asserted.
- **VERIFIED (Retail 12.1.5.70688):** `GetProductGroups` returns fresh group rows after display/eligible-product filtering; `GetProducts` returns filtered entry IDs; `GetEntryInfo` and `GetProductInfo` publish fresh detail tables. The `GetEntryInfo` snapshot copies `alreadyOwned=false`; it does not read a separate ownership result. Missing group metadata returns zero values; missing entry/product detail returns one nil.
- **VERIFIED (Retail 12.1.5.70688):** availability and list readiness are separate. Purchase-in-progress uses cached purchase rows and adjusted expiry time; it is not derived from a VAS action or a readiness flag.
- **VERIFIED (Retail 12.1.5.70688):** currency ID zero or missing metadata produces nil currency info. `GetProductList` counts calls while disconnected, submits at most five requests per unsigned 2000 ms interval, and returns no values; `GetPurchaseList` submits only when connected and returns no values.
- **VERIFIED (Retail 12.1.5.70688 R88ad.2 and simulator source):** a supplied product override color is published as a new table created from `ColorMixin`; the StoreSecure producer sets normalized `r`, `g`, and `b` and does not set alpha. The pinned `StoreCardMixin:UpdateCard` passes those three channels to `ProductName:SetTextColor`; an absent color remains nil.
- **UNKNOWN:** actual catalog/currency rows, per-account eligibility, server timing, full disconnect cleanup of catalog/purchase rows, disconnect notification order, full distribution-state writer/lifetime, and cross-response order. Their host inputs stay empty unless a test or host explicitly supplies them.

### Profile boundary

- **VERIFIED (simulator source):** `c_store_secure_catalog::register` selects the modeled surface only when `client_profile::ACTIVE` is Retail. The Lua bootstrap skips the replaced Retail catalog/list guesses only for that profile.
- **UNKNOWN:** native Mists catalog semantics. Mists and other uncovered profiles retain their prior bootstrap path; that path is not promoted as native behavior.

### Tests

`tests/store_secure_catalog.rs` runs the original StoreUI and CatalogShop consumer code with explicit host records and empty-default state. `tests/catalog_shop.rs`, `tests/store_tree.rs`, and `tests/store_anchor_regression.rs` supply explicit Retail readiness/catalog inputs for presentation tests. `tests/store_group_contract.rs` checks the original cleanup path in `secureenv`. The former default-demo group assertions are retired; see the [historical policy page](../../specs/store-product-group-shape.md).

## Sources

- [StoreSecure catalog contract](../../specs/store-secure-catalog.md).
- [Store API availability system](store-api-availability.md) — separate `C_StorePublic` and CatalogShop switches.
- R88ad.1/R88ad.2 native findings, Retail 12.1.5.70688 (task evidence packet).
- `src/c_api/store_catalog_model.rs` — host input and response state.
- `src/c_api/c_store_secure_catalog.rs` — Retail C API readers and requests.
- `src/c_api/c_store_secure.rs` — Retail selection and retained non-Retail group path.
- `src/lua_api/env_init/runtime_surface_bootstrap.lua` — profile-scoped bootstrap retirement.
- Original Retail consumers listed above, from pinned cache 12.1.5.70783.

## See Also

- [[store-api-availability]] — separate public Store status mirrors.
- [[store-secure-pool-constructors]] — historical Store UI pool investigation and source checkpoints.
- [[consumer-selection-and-red-proof]] — consumer selection and producer-subtracted RED proof.
