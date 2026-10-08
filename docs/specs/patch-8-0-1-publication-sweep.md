# Patch 8.0.1 publication audit

## Source and scope

Audit Warcraft Wiki page 149302, revision 1463362 (2023-07-20T21:26:53Z), refetched 2026-10-08. The register retains nested namespace additions, named prose removals and every added/removed event occurrence. Extract retains migration prose, replacement guidance and references; linked resources are not expanded. Parser changes are opt-in `--bfa-prepatch`, distinct from the parallel 8.1.5 bullet parser.

## Required behavior

- [x] Prefork full-UI sweep probes every register occurrence using the shared raw/ordinary lookup, event-registration and later-register supersession classifier. Publication is not signature, payload, security or native behavior parity.
- [ ] Every failure has its exact literal, observation, expected publication and bounded reason in the coverage ledger/review.
- [ ] Every candidate removal has untruncated whole-word qualified/bare cached and src/tests scans plus newer-register inspection. Current consumers must remain reachable.
- [ ] Historical saved extracts/registers reproduce; inherited extraction failures remain explicitly recorded without rewriting inputs.
- [ ] Portable validator derives counts from retained artifacts and scopes mutable historical inputs to the base Git revision.

## Explicit world-position model

`C_Map.GetMapPosFromWorldPos(continentID, worldPosition, overrideUiMapID?)` is backed by environment-local, map-ID-keyed world rectangles under `c_api::map_world_coordinates`. No default native geography is fabricated from art dimensions.

- [x] Publish a real raw function, parse required numeric coordinates and optional numeric map ID.
- [x] Project a finite point inside an explicit known map's world rectangle into normalized coordinates; preserve reversed axes. Return map ID plus a vector exposing x/y/GetXY.
- [x] Honor continent and override ID. No matching rectangle, out-of-bounds point, degenerate/nonfinite rectangle or overlapping automatic matches returns no values.
- [x] Rectangle mutations affect later queries; environments remain isolated.
- [ ] Native automatic selection among overlapping map hierarchies, secret-argument parity and real geography capture remain unproven. Existing `GetWorldPosFromMapPos` art-based approximation is not a round-trip oracle for these new explicit inputs and remains outside this change.

## Proof

- `tests/patch_8_0_1_publication_sweep.rs` — cached publication only.
- `tests/patch_8_0_1_map_coordinates.rs` — concrete affine projection, input/missing-state boundaries and isolation.
- [Audit evidence](../../data/patch-api/evidence/8.0.1-session-2026-10-08/) — discovery, scans, receipts, reproduction and validator.
- [Wiki audit](../wiki/investigations/patch-8-0-1-api-audit.md) — final coverage matrix and unresolved contracts.

No shims, vendor patches, agents, full integration suite, push or merge are authorized.
