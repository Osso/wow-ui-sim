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

## Local proof

| Patch | Rows | OK | Gaps | Result |
|---|---:|---:|---:|---|
| 11.0.2 | 34 | 22 | 12 | PASS |
| 11.0.5 | 48 | 38 | 10 | PASS |
| 11.0.7 | 98 | 70 | 28 | PASS |
| 11.1.0 | 116 | 97 | 19 | PASS |
| 11.1.5 | 125 | 89 | 36 | PASS |
| 11.1.7 | 48 | 40 | 8 | PASS |
| 11.2.0 | 162 | 135 | 27 | PASS |
| 11.2.5 | 163 | 118 | 45 | PASS |
| 11.2.7 | 508 | 414 | 94 | PASS |
| 12.0.0 | 1010 | 989 | 21 | PASS |
| 12.0.1 | 225 | 222 | 3 | PASS |
| 12.0.5 | 363 | 352 | 11 | PASS |
| 12.0.7 | 174 | 171 | 3 | PASS |
| 12.1.0 | 778 | 773 | 5 | PASS |

All sweeps run alone at aefaa9605; later fixtures remain unchanged. Exact commands and revisions: `p1102-proof.json` in the evidence directory.
