# Patch 11.0.2 publication sweep

## Contract

Audit all 34 retained inventory occurrences against default retail 12.1.0. All thirteen later registers, 11.0.5 through 12.1.0, supersede chronologically. Exact gap IDs must match observed failures; publication is not signature, output, security or behavior parity. Cached deprecation wrappers remain unchanged.

## Bounded closures

Three unused removed members remain absent after repeated ordinary/raw lookup. Neighbor namespaces stay callable; classic profiles retain prior registration. `C_Item.IsItemBindToAccountUntilEquip` reads intrinsic item bonding metadata (ItemBind 9), accepting ID, numeric string and item hyperlink. Unknown IDs/invalid strings return false. This does not model per-instance binding, ownership or refund eligibility. Existing secret argument boundary is reused; no new security parity claim.

## Tests

- `tests/patch_11_0_2_publication_sweep.rs`: all source inventory rows, chronological supersession and exact fixture.
- `tests/patch_11_0_2_publication_fixes.rs`: repeated lookup and concrete generated item metadata.
- `tests/patch_11_0_2_cached_surfaces.rs`: unchanged full cached Game preload retains retirements, neighbors and item query without new Lua errors.

## Sources

- `data/patch-api/sources/11.0.2-api-changes.provenance.json`
- `data/patch-api/evidence/11.0.2-session-2026-10-06/p1102-retirement-consumers.json`
- Cached `Blizzard_APIDocumentationGenerated/ItemConstantsDocumentation.lua:179–195`: ItemBind 9 is ToBnetAccountUntilEquipped.
