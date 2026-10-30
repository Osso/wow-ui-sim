# Aura container carryover and private caster queries

## Contract

- `C_UnitAuras.GetRefreshCarryOverDuration` uses the same aura lookup, recast metadata, argument validation, access policy, and eligibility as `GetRefreshExtendedDuration`. It returns `min(max(expiration - now, 0), max_carryover_seconds)`, or nil when the existing refresh policy is ineligible. This is a **user-approved default, not native evidence**.
- `C_UnitAurasPrivate.GetPrivateAuraCasterGUID(unitToken, auraInstanceID)` returns the supplied private record's `sourceGUID`, or nil when the record or GUID is unknown. This is a **user-approved default, not native evidence**. The private record store remains temporary.
- Both queries are available with `retail-12-1-5`; this contract does not establish other profiles or native secret handling.

## Evidence

The Retail 12.1.5.70448 cache's `Blizzard_APIDocumentationGenerated/UnitAuraDocumentation.lua:412–429` declares the carryover query and its optional spell identifier. `Blizzard_AuraContainer/Blizzard_AuraContainerUtil.lua:13–27` consumes its result as a pandemic window. The existing [refresh-duration contract](aura-refresh-duration.md) describes the inferred metadata and eligibility policy.

The pinned `Blizzard_AuraContainer/Blizzard_AuraContainerSources.lua:72–73` calls the private caster query with a unit token and aura instance ID. No generated declaration was identified for this private query; its behavior is not native-verified.

`tests/aura_container_missing_queries.rs` exercises the original consumer methods with two supplied carryover caps and two supplied GUIDs, plus unknown private records.
