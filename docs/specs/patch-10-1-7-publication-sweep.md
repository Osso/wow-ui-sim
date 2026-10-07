# Patch 10.1.7 publication sweep

Account for Warcraft Wiki page 442982, revision 6473483 (2025-09-15T16:55:36Z), against current retail 12.1.0. [Audit](../wiki/investigations/patch-10-1-7-api-audit.md) owns findings and proof limits.

## What it must do

- [x] Retain 48 inventory and 55 non-inventory occurrences, including verbatim Lua example; account for all 103 source IDs exactly once.
- [x] Preserve stale header counts: 19 declared versus 28 global additions; eight declared versus three added events. Do not invent omitted rows.
- [x] Apply eighteen later registers, 10.2.5 through 12.1.0; inserting 10.2.0 is one line at the beginning after integration.
- [x] Keep fourteen precise publication gaps, not inert new functions or incompatible aliases.
- [x] Apply XML template attributes to runtime-created frames and children before OnLoad, with existing typed storage, derived overrides and no premature attribute notification; ordinary post-construction SetAttribute still notifies.
- [ ] Exact one-row negative control adds one publication failure.
- [ ] All nineteen isolated sweeps, bounded behavior and cached prefork tests, format/Mists checks and startup [] pass.

## Implementation inventory

- `tests/patch_10_1_7_publication_sweep.rs`: 48 publication/absence probes with exact fixture.
- `tests/patch_10_1_7_template_attributes.rs`: concrete inherited boolean/number/string/nil state and OnLoad ordering.
- `tests/patch_10_1_7_cached_surfaces.rs`: full cached Game template attributes and current unit-target GUID mixin.
- `src/lua_api/globals/create_frame/template_chain/attributes.rs`: reuse XML SetAttribute emission for runtime construction.
- `tools/gen_patch_wikitext_register.py` and `tools/extract_patch_non_inventory.py`: fixture-backed level-two section support and literal ping example extraction.

## Known gaps

- [ ] Fourteen inventory gaps: paid character service/name casing, unread club streams, art manifest, historical raid ping policy, listener/pending/error/receiver/world/send ping producers and TugOfWar DTO.
- [ ] Forty-five substantive extract occurrences require historical/native behavior, security, enum values, input or scrolling proof. Current cached template proof does not confer historical ping delivery credit.

## Out of scope

Historical epoch reconstruction, native parity, expanding linked pages, invented service/world data, vendor/cache/Wowless changes, classic surface changes, broad suites, agents/models, push and merge.

## Proof

[Proof ledger](../../data/patch-api/evidence/10.1.7-session-2026-10-07/p1017-proof.json) records exact command/revision/scope, result and invalidation. [Gap review](../../data/patch-api/evidence/10.1.7-session-2026-10-07/p1017-gap-review.json) and [extract scout](../../data/patch-api/evidence/10.1.7-session-2026-10-07/p1017-extract-scout.json) retain every boundary. Local targeted development evidence only.
