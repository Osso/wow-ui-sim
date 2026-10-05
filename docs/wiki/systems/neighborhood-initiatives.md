# Neighborhood initiatives

Host-backed `HousingState.initiative` owns active/viewing GUIDs, per-neighborhood initiative records and activity logs, required level, access entitlement and current party's qualifying neighborhood membership. No native catalog, persistence or progression engine is synthesized.

## Runtime

`src/c_api/c_neighborhood_initiative/` publishes detached documented DTOs. Tasks inherit tracked flags from the existing tracked-ID set. Chat links are explicit host strings, looked up in the active neighborhood; unknown links return an empty string. Required level compares against `SimState.player.level`; access remains a separate host entitlement.

Request calls initialize missing records in the choosing stage and defer delivery to the next timer tick. This preserves cached dashboard `OnShow` ordering: request first, register listeners second. Replies mark only the requested neighborhood loaded, then synchronously dispatch the documented zero-payload event if that neighborhood is still viewed. Changed active selection refreshes the dashboard; repeated identical writes do not redispatch.

Market-shop policy and house-finder selected plot are fields in housing state, registered in `src/c_api/c_housing/market.rs`. A click does not imply purchase or teleport.

## Unsupported diagnostics

Cached exterior documentation and consumers contain no fixture-debug signatures or payloads. `temporary/house_exterior_debug.rs` publishes two nil-returning unavailable-data workarounds. Retire once native contract evidence and GUID/selected-fixture diagnostic state exist. Publication closure is not native diagnostic behavior proof.

## Sources

- [Publication contract](../../specs/patch-12-0-0-publication-sweep.md#housinginitiative-closure-contract)
- Cached `NeighborhoodInitiativeDocumentation.lua`, `HousingUIDocumentation.lua`, `HouseExteriorUIDocumentation.lua` and `Blizzard_HousingDashboardInitiatives.lua`.

## See Also

- [[patch-12-0-0-api-audit]] — publication inventory and historical diagnostic gaps
