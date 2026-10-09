# Patch 4.0.1 publication audit

Account for the complete pinned historical retail page, revision 1271877, in `data/patch-api/sources/4.0.1-api-changes.wikitext`. [Audit implementation and evidence](../wiki/investigations/patch-4-0-1-api-audit.md).

## What it must do

- [x] Retain explicit NEW/REMOVED source occurrences and typed Breaking changes references, with source lines and unique IDs.
- [x] Keep default generator/extractor behavior unchanged; new Cataclysm handling is opt-in.
- [x] Retain all four breaking statements, navigation and both generated-build contexts outside inventory.
- [x] Probe publication/absence in cached retail Game after retail-only successors; require exact known-gap equality.
- [ ] Keep pending 4.1.0, 4.2.0, 4.3.0 and 4.3.4 placeholders in order before actual 5.0.1/later retail registers; main replaces on integration.

- [ ] Validate sealed historical source, Git commit/tree/blob pins, own recorded commands, complete row/gap accounting and own byte reproduction without resolving old Git commits.
- [ ] Reject source-response, own-log and historical-archive tampering; ignore unrelated later inventories without expanding proof scope.

## How it works

- [Audit](../wiki/investigations/patch-4-0-1-api-audit.md).

## Implementation inventory

- `src/c_api/patch_retired_members.rs`: separate 4.0.1 removed-global list; module is modern-retail gated.
- `src/c_api/mod.rs`: gated list export.
- `src/lua_api/globals/stubs/global_stubs.rs`: exclude two unused skill-header no-ops on modern retail; Classic unchanged.

- `tools/gen_patch_wikitext_register.py`: opt-in labeled inventory and typed prose references.
- `tools/extract_patch_non_inventory.py`: opt-in inventory omission without losing prose/build context.
- `tests/patch_4_0_1_publication_sweep.rs`: own retail sweep.
- `data/patch-api/sources/4.0.1-*`: pinned input and generated fixtures.
- `data/patch-api/evidence/4.0.1-session-2026-10-09/validate.py`: historical-only own validator.
- `data/patch-api/evidence/4.0.1-session-2026-10-09/pin-builder.py`: compact cryptographic snapshot capture.

## Tests asserting this spec

- `tools/test_patch_cataclysm_register.py`: concrete labeled occurrences, direction, owner category, source line, opt-in boundary and retained extract.
- `tests/patch_4_0_1_publication_sweep.rs`: current retail publication/absence only.
- `tests/patch_4_0_1_retirements.rs`: factory/cached absence on modern retail; callable original registration on Classic.

- `tools/test_patch_4_0_1_validator.py`: original/Git-unavailable evidence acceptance and disposable-copy tamper controls.

## Known gaps (current cycle)

- [x] Exclude CollapseSkillHeader/ExpandSkillHeader no-op registration on modern retail after complete caller/cache scans; Mists factory preservation passes. Other Classic client execution remains unverified.
- [ ] Four breaking statements and one historical numeric-only input contract remain native/model limits; 117 publication mismatches retained in exact gap ledger.

## Out of scope

Native 2010 client execution, complete gameplay/security parity, linked API pages not supplied locally, broad verification and final integration gates. No Classic successor credit or vendor/cache Lua modifications.
