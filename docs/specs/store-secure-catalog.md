# Retail StoreSecure catalog and list state

This contract covers the Retail `C_StoreSecure` catalog, availability, currency, dynamic-price, and list-request methods implemented under `src/c_api/`. Native findings are from Retail 12.1.5.70688 (R88ad.1/R88ad.2); original consumer tests use the pinned Retail 12.1.5.70783 UI cache. **VERIFIED (pinned Retail 12.1.5.70783 generated API directory):** it has no `C_StoreSecure` declaration; the original consumers provide the selected call sites. See the [system page](../wiki/systems/store-secure-catalog.md) for the state model.

## What it must do

### Catalog reads

- [ ] `GetProductGroups` returns one fresh array of `{ groupID, parentGroupID }` rows. It applies the native display and eligible-product filters, preserves response order, and returns an empty array when no rows pass.
- [ ] `GetProductGroupInfo` returns fresh metadata for the first matching group. A miss returns zero Lua values.
- [ ] `GetProducts` returns ordered entry IDs after support, session, hidden-entry, and product-ID filters. `includeHiddenProducts` bypasses only the hidden-entry filter; a miss returns an empty array.
- [ ] `GetEntryInfo` resolves an Entry ID to its linked supported Product and returns a fresh entry and `sharedData` table. A missing or rejected row returns one nil.
- [x] VERIFIED (Retail 12.1.5.70688 R88ad.1/R88ad.2; `tests/store_secure_catalog.rs`): the `GetEntryInfo` snapshot sets `alreadyOwned` to false and copies it; it does not read a separate ownership result.
- [ ] `GetProductInfo` returns a fresh product and `sharedData` table for the first supported Product row. A missing or rejected row returns one nil.
- [ ] `sharedData` publishes the verified numeric, Boolean, optional, color, card, and deliverable fields. Absent optional fields stay nil. Runtime prices, catalog rows, and metadata remain host supplied; defaults do not add demo records.
- [ ] When `overrideTextColor` is supplied, `GetProductInfo` returns a new table built from `ColorMixin` and sets only `r`, `g`, and `b` to each native byte component multiplied by the single-precision reciprocal of 255. It does not set alpha; absent color stays nil.

### Availability, lists, currency, and requests

- [ ] Availability, product-list readiness, purchase-list readiness, purchase rows, dynamic-price rows, and currency ID start empty or false. Availability and list readiness are separate state.
- [ ] Availability responses store the supplied Boolean before a zero-payload `STORE_STATUS_CHANGED` event.
- [ ] A successful product-list response replaces the catalog, stores currency ID, clears region lock and response error, and sets product-list readiness before a zero-payload `STORE_PRODUCTS_UPDATED` event. A failed response clears group/product rows, clears readiness, stores the error, and sets region lock only for result code 3.
- [ ] A purchase-list response sets readiness from its result code, copies rows only on success, and queues a zero-payload `STORE_PURCHASE_LIST_UPDATED` event after the state change.
- [ ] A disconnect clears availability before resetting the verified product/purchase readiness, region-lock, response-error, currency-ID, dynamic-price, and connection state.
- [ ] `HasDistributionList` is false without a connection or supplied connection distribution state. No default connection state is invented.
- [ ] `HasPurchaseInProgress` is true when any row is unblocked, has status other than 6, and has adjusted expiry later than adjusted current time. The query does not consume rows.
- [ ] `GetCurrencyID` starts at zero. `GetCurrencyInfo` returns nil for ID zero or a missing metadata row; otherwise it returns a fresh `{ currencyID, sharedData }` table with the verified metadata fields.
- [ ] `IsDynamicBundle` reads the supplied product marker. `HasDynamicPriceData` checks supplied product-keyed data. A dynamic-price response stores data before a one-number `DYNAMIC_BUNDLE_PRICE_UPDATED` event.
- [ ] `GetProductList` increments its 32-bit request counter on each normal call, including while disconnected. It submits only while under the limit of 5 and connected; the counter resets after the unsigned 2000 ms elapsed check. It returns zero Lua values and does not wait for a response.
- [ ] `GetPurchaseList` submits only while connected, returns zero Lua values, and does not wait for a response.

### Profile boundary

- [ ] Register this modeled surface only for Retail. Keep the prior bootstrap path for uncovered profiles; Mists behavior remains UNKNOWN and is not treated as native evidence.
- [ ] Keep the methods available in `secureenv` after the original Retail `Blizzard_EnvironmentCleanup` path.

## How it works

- [Retail StoreSecure state and response flow](../wiki/systems/store-secure-catalog.md).
- [Existing Store availability inputs](../wiki/systems/store-api-availability.md).
- [Anchor and Lua API systems](../lua-api.md).

## Implementation inventory

- `src/c_api/store_catalog_model.rs` — host inputs, query predicates, and response updates.
- `src/c_api/c_store_secure_catalog.rs` — Retail C API registration and Lua publication.
- `src/c_api/c_store_secure.rs` — Retail selection and retained non-Retail group path.
- `src/c_api/c_store_glue.rs` and `src/c_api/store_glue_model.rs` — shared callbacks and request submissions.
- `src/lua_api/env_init/runtime_surface_bootstrap.lua` — skips the replaced Retail guesses and retains other-profile bootstrap behavior.

## Tests asserting this spec

- `tests/store_secure_catalog.rs` — original Retail StoreUI and CatalogShop consumers, empty defaults, response transitions/events, currency formatting, ColorMixin RGB output, and request/list selectors.
- `tests/store_api.rs` — zero currency ID and verified disconnect state with explicit host responses.
- `tests/store_group_contract.rs` — original EnvironmentCleanup path and secure-environment method availability.
- `tests/catalog_shop.rs`, `tests/store_tree.rs`, and `tests/store_anchor_regression.rs` — Retail Store consumers receive explicit catalog inputs for presentation checks.

## Known gaps (current cycle)

- [ ] Add direct tests for restricted execution, secret argument access, pending-error consumption, and no-argument methods with extra/wrong-type arguments.
- [ ] The full disconnect cleanup of catalog and purchase row arrays, disconnect notification order, native currency metadata rows, server response timing, and the complete distribution-state writer remain UNKNOWN; keep those inputs empty until supplied.
- [ ] Mists and other non-Retail native catalog behavior is UNKNOWN; this change does not promote Retail findings to those profiles.

## Out of scope

- Real network transport, server catalog contents, account eligibility, purchase settlement, and complete VAS purchase lifecycle.
- Native-parity claims for PTR, WoW Forever, Mists, Era, Anniversary, or Wrath.
