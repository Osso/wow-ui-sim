# Patch 4.0.1 publication audit

Account for the complete pinned historical retail page, revision 1271877, in `data/patch-api/sources/4.0.1-api-changes.wikitext`. [Audit implementation and evidence](../wiki/investigations/patch-4-0-1-api-audit.md).

## What it must do

- [x] Retain explicit NEW/REMOVED source occurrences and typed Breaking changes references, with source lines and unique IDs.
- [x] Keep default generator/extractor behavior unchanged; new Cataclysm handling is opt-in.
- [x] Retain all four breaking statements, navigation and both generated-build contexts outside inventory.
- [x] Probe publication/absence in cached retail Game after retail-only successors; require exact known-gap equality.
- [x] Use actual retail 4.1.0, 4.2.0, 4.3.0 and 4.3.4 successor registers in order before 5.0.1/later retail registers; exclude Classic successors.

- [x] Validate sealed historical source, Git commit/tree/blob pins, own recorded commands, complete row/gap accounting and own byte reproduction without resolving old Git commits.
- [x] Reject source-response, own-log and historical-archive tampering; ignore unrelated later inventories without expanding proof scope.
- [x] Exclude CollapseSkillHeader/ExpandSkillHeader no-op registration on modern retail after complete caller/cache scans; Mists factory preservation passes. Other Classic client execution remains unverified.

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

- [ ] Four source breaking statements and the separate GetItemCooldown numeric-only signature remain UNPROVEN. Current accounting: 419 publication rows, 304 current matches and 115 gaps. Only CanTransform/Transform additions are superseded by 4.1 removal; this earns no model credit. Sealed historical accounting remains 302/117, with negative 118.

## Current receipts and acceptance boundary

[Integrated discovery receipts](../../data/patch-api/evidence/4.0.1-session-2026-10-09/integrated/) are separate from [historical evidence](../../data/patch-api/evidence/4.0.1-session-2026-10-09/). [Portable validator v2](../../data/patch-api/evidence/4.0.1-session-2026-10-09/portable-validator-v2.md) removes only the repository-target temporary-workspace dependency; exact v1 validator/context and historical archive/evidence seals remain preserved.

Main publication passes 70/70; Mists check/build exit 0, but built startup is not yet checked. Independent formatting scope covers 212 files; Python fixtures 75/75 and own v2 fresh-root/no-target/no-Git fixtures 5/5 pass. Register outputs reproduce 74/74 with all process exits captured; extracts reproduce 74/77, with three known inherited failures (12.0.5/12.0.7 mismatch, 12.1.0 error/no output). Shared portable gate, current negative control, runtime smoke, CI and full suite remain pending; no all-green acceptance claim.

## Out of scope

Native 2010 client execution, complete gameplay/security parity, linked API pages not supplied locally, broad verification and final integration gates. No Classic successor credit or vendor/cache Lua modifications.
