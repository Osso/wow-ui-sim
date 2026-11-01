# Housing charter editability

Retail exposes `C_Housing.CanEditCharter`. The pinned `Blizzard_APIDocumentationGenerated/HousingUIDocumentation.lua:15-22` declares one non-nil boolean result.

## What it must do

- [ ] Return the host-supplied charter edit capability as one boolean; the unconfigured scenario is false.
- [ ] The original HousingCharter settings-button update reflects both false and true supplied values.

## Best-guess basis

Retail 12.1.5 build 70688 evidence verifies false when a required native housing reference, storage, or component is missing. A present component delegates to an unidentified virtual predicate; the connection to Lua boolean publication is emulated. The simulator represents the unknown predicate's result as a supplied boolean. False when unconfigured is the no-component scenario, not a claim about account ownership.

## How it works

- [Addon loading pipeline](../addon-loading-pipeline.md)

## Implementation inventory

- `src/c_api/c_housing/charter.rs`: state-backed Retail query.
- `src/c_api/c_housing.rs`: Retail registration.
- `src/lua_api/state/support_types.rs`: supplied housing capability.

## Tests asserting this spec

- `tests/blizzard_housing_charter_loads.rs`: original settings-button consumer with distinct supplied states and show/hide event registration.

## Known gaps (current cycle)

- [ ] Native component identity, ownership and eligibility rules, event timing, and runtime transitions are unknown.
- [ ] Other profiles are not published by this change.

## Out of scope

- Charter creation, signature management, and server-side authorization.
