# Patch 7.0.3 publication sweep

Audit pinned [page 549091, revision 5295335](../../data/patch-api/sources/7.0.3-api-changes.provenance.json). [Wiki audit](../wiki/investigations/patch-7-0-3-api-audit.md) owns implementation and proof details.

## What it must do

- [ ] Probe every registered identity, distinguishing publication/absence from behavioral parity and applying later registers chronologically.
- [ ] Account for every retained extract statement; keep unspecified domains and historical contracts explicit rather than inventing shims.
- [ ] Persist recipe search text, filter the existing learned catalogue case-insensitively, clear with nil, reject invalid input, and publish list updates.
- [ ] Preserve default recipe-list order/results when search is empty.
- [ ] Keep consumer-free retail GetMountInfo/GetMountInfoExtra absent on raw, ordinary and repeated lookup; retain current successors and live legacy Summon callers.
- [ ] Preserve classic surfaces and all merged publication sweep expectations.
- [ ] Validate fixed historical register scope and file-derived counts from any checkout, including after later audits merge.
- [ ] Reproduce all saved registers/extracts using recorded flags, retaining only the three inherited extract failures.

## How it works

- [Audit and evidence](../wiki/investigations/patch-7-0-3-api-audit.md).
- [Validator portability](../wiki/investigations/patch-audit-validator-portability.md).

## Implementation inventory

- `tools/gen_patch_wikitext_register.py`: opt-in Legion nested inventories, normalized owners, bare removals and rename pairs.
- `tools/extract_patch_non_inventory.py`: opt-in removal of pure nested inventory lines only.
- `src/c_api/c_trade_skill_filter.rs`: modeled recipe search and catalogue filtering.
- `src/lua_api/state_types/crafting.rs`: per-environment search text.
- `src/lua_api/globals/missing_surface/professions.rs`: profession registration delegates filter APIs to C API module.
- `src/c_api/patch_retired_members.rs`: retail-only two-member absence boundary.
- `tests/common/publication_sweep.rs`: shared publication probe.
- `data/patch-api/sources/7.0.3-*`: pinned inputs and occurrence ledger.
- `data/patch-api/evidence/7.0.3-session-2026-10-08/validate.py`: historical read-only gate.

## Tests asserting this spec

- `tests/patch_7_0_3_publication_sweep.rs`.
- `tests/patch_7_0_3_behavior.rs`.
- `tools/test_gen_patch_wikitext_register.py`.
- `tools/test_extract_patch_non_inventory.py`.

## Known gaps (current cycle)

- [ ] Exact unresolved publication identities and per-statement reasons remain in the occurrence ledger and gap review; no native parity claim.
- [ ] Integrate the first-position 7.1.0 placeholder after p710-page merges.

## Out of scope

Historical Legion server/UI reconstruction, 3D nameplates/cameras/models, unspecified Garrison filters and defunct-stat member catalogs, native locale collation/reagent search, vendor changes, full integration suite, push, merge and agents. Current APIs with cached consumers, simulator callers or later re-additions must not be retired merely to make this sweep pass.
