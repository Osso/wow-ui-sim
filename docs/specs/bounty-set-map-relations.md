# Bounty-set map associations

`C_Map.GetBountySetMaps` queries explicit simulator-owned bounty-set membership. Current Retail/PTR cached `MapDocumentation.lua` declares required numeric `bountySetID`, a non-nil table of numeric map IDs and `AllowedWhenUntainted`; cached `MapUtil` forwards/memoizes the result and `WorldMapBountyBoard` consumes it with `ipairs`. [Source boundary](../wiki/investigations/patch-8-1-0-api-audit.md). Historical8.1 evidence is name-only, not this current contract.

## What it must do

- [x] Return exactly one fresh numeric sequence for explicitly supplied set membership, preserving supplied values/order rather than deriving membership from map hierarchy/faction metadata.
- [x] Return an empty sequence when no modeled association exists. This is simulator absence policy respecting the non-nil sequence contract, not native unknown-ID evidence.
- [x] Keep normal defaults free of invented bounty memberships; a test's supplied relation is not production/native catalog data.
- [x] Reflect owned-input updates/removal in subsequent queries while preserving previous snapshots and caller mutation isolation.
- [x] Validate required numeric inputs without wrong-type coercion or lossy identity aliasing; apply documented secret authentication before validation/lookup and preserve wrappers/caller taint.
- [x] Register the modeled query for current Retail/PTR channels. No inferred earlier-profile backport or revival of retired `GetBountySetIDForMap`.

## How it works

- [C API ownership](../../AGENTS.md#c-api-boundary)
- [Current handoff](../wiki/investigations/integrated-source-and-factory-proof-2026-10-09.md)

## Implementation inventory

- `SimState.bounty_set_maps`: explicit membership input, initialized empty after authentic initial public-query RED; no default associations. `src/c_api/c_map_bounties.rs` projects those explicit inputs through guarded C API registration and a fresh sequence result; Retail and PTR focused runs each report 11 passed; both normal-session publication/empty-model/retired-ID/Calendar-preservation probes report exit0/`[]`. [Independent GREEN](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/bounty-set-map-relations-2026-10-11/green-independent-report.md) PASS: 11 logical cases across Retail/PTR, 22 executions; normal assertions and Rust checks verified.
- `src/c_api/c_map.rs`: existing map surface registration seam.

## Tests asserting this spec

- `tests/c_map_bounty_set_maps.rs`: initial six public-query absence/shape, snapshots, validation/security tests executed RED at `e09b0c1d1`: 0 passed / 6 failed, exit101. Public query violates required non-nil table; setup/compile succeeded.
- Five explicit-association/update/mutation/anti-alias/actual-MapUtil controls now target real owned inputs. Expanded `9602fa8fb` RED executed before getter producer: 0 passed / 11 failed, exit101, including the actual unmodified MapUtil consumer's non-table result after source execution. Later assertions remain unproved until GREEN.

## Known gaps (current cycle)

- [ ] Expanded RED's independently computed digest-comparison output was lost; authentication receipt gap remains explicit. Behavioral RED11 is confirmed; initial RED6 and all GREEN/publication artifact authentication are verified. No gap concealment or old-payload reread.
- [x] Normal-session publication verified in Retail/PTR; only current Bounty row removed. [Independent owning-prefork proof](https://github.com/Osso/wow-ui-sim/blob/183edc381/data/patch-api/evidence/bounty-set-map-relations-2026-10-11/publication/publication-independent-report.md): one case, 145 classified rows/90 OK/55 current gaps; frozen historical and earlier Calendar receipts unchanged.

## Out of scope

Invented/default memberships, native catalog acquisition, native ordering/unknown-ID fidelity, full bounty-board lifecycle, retired APIs and unproved profile matrices. Calendar's generated normal-session data policy does not authorize Bounty seeding. These boundaries distinguish this explicit-input query from missing catalogs or native acceptance; parent handoff remains open.
