# Supplied StorePublic group eligibility

`C_StorePublic.DoesGroupHavePurchaseableProducts` reads explicit group eligibility observations. Source belongs under `src/c_api/`; [model and limits](../wiki/systems/store-public-eligibility.md).

## What it must do

- [x] Publish only on Retail, PTR, Forever, Mists, Era and Anniversary. Wrath is a declaration-negative control, not a claim about native Wrath.
- [x] Return exactly one non-null boolean for each supplied group ID, independent of secure-product membership and catalog DTOs.
- [x] Keep observations environment-local and updateable. Missing observations raise `missing-input: C_StorePublic.DoesGroupHavePurchaseableProducts groupID=<id> has no supplied eligibility observation`, never false.
- [x] Let the selected original subscription helper choose subscription, game time or web redirect according to independent supplied observations, preserving subscription priority and transitions.
- [x] Keep group eligibility independent from store availability and visibility; see the [separate Store API contract](store-api-availability.md). Do not derive eligibility from either availability flag.

## How it works

- [Environment-local model and proof boundary](../wiki/systems/store-public-eligibility.md).

## Implementation inventory

- `src/c_api/store_public_eligibility.rs`: supplied observations and profile-gated query.
- `src/c_api/registration.rs`: namespace registration.
- `src/lua_api/state/sim_state.rs`, `src/lua_api/state.rs`: per-environment input storage.
- `src/lua_api/workarounds/temporary/store_public_defaults.rs`: visibility state and compatibility defaults only where no source-backed `IsEnabled` producer is registered.

## Tests asserting this spec

- `tests/store_public_eligibility.rs`: selected original helper, supplied state, shape, updates, missing input, isolation and Wrath absence. Recording sinks prove helper control flow only.
- `src/lua_api/workarounds/temporary/store_public_defaults.rs::tests`: unchanged enabled/visibility assertions and the legacy group-501 assertion with explicit supplied eligibility; Wrath asserts query absence.

## Known gaps (current cycle)

None for the supplied-input contract. [Final proof](../wiki/systems/store-public-eligibility.md#final-proof-2026-10-26) covers all six declared profiles, Wrath absence, seven unchanged stock pairs and seven zero-warning all-target checks.

## Out of scope

- Native policy for membership, ownership, region, eligibility flags, readiness, numeric coercion or secret arguments. Retail/PTR/Forever declarations include `SecretArguments = "AllowedWhenUntainted"`; Mists/Era/Anniversary do not. This lane does not infer parity.
- Working storefront, purchases, secure StoreUI acceptance and classic `SelectSubscriptionProduct`. Classic inbound Lua lacks that endpoint; recording sinks do not remove the blocker.

## Declaration provenance

Each profile's pinned `Blizzard_APIDocumentationGenerated/StorePublicUIDocumentation.lua` declares `Environment = "All"` at line 6. Retail/PTR/Forever lines 11–24 declare required numeric groupID at line 17 and non-null boolean at line 22. Mists/Era/Anniversary lines 11–23 declare numeric groupID at line 16 and boolean at line 21. Declarations establish shape, not an eligibility algorithm or an unsupplied default.
