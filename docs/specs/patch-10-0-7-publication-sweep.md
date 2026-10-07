# Patch 10.0.7 publication sweep

Account for Warcraft Wiki page 108565, revision 1063344 (2023-05-09T20:31:43Z), against current retail 12.1.0. [Page audit](../wiki/investigations/patch-10-0-7-api-audit.md) owns source accounting and proof boundaries.

## What it must do

- [x] Retain 70 inventory occurrences and all 2,214 non-inventory occurrences, including the complete Structures and Type Changes tail. Later type-event headings must not overwrite publication inventories.
- [x] Apply twenty-one chronological later registers from master, 10.1.5 through 12.1.0; reserve 10.1.0 at the list start without depending on its unmerged branch.
- [x] Require exactly 26 reviewed publication gaps. Publication, absence and event registration do not establish signatures, populated outputs, security, event payloads or native/historical parity.
- [x] Prevent repeated ordinary lookup from fabricating eighteen removed, unused retail namespace members; preserve C_Social.GetFriends/GetFriendInfo and unmodified Blizzard deprecation wrappers.
- [ ] Preserve classic legacy lookup independently of retail retirement; existing module gate excludes classic profiles and a Mists behavior test awaits execution.
- [ ] Preserve all prior registers, sources, ledgers and gap fixtures byte-identically; retain every previous extract-check result under both example modes. Artifact verification pending.

## How it works

- [Page audit](../wiki/investigations/patch-10-0-7-api-audit.md) — occurrence accounting, retirements and proof scopes.
- [Client profiles](../wiki/systems/client-profiles.md) — existing retail/classic gates.

## Implementation inventory

- `tools/gen_patch_wikitext_register.py` — publication inventory parser, stopping before historical type annotations.
- `tools/extract_patch_non_inventory.py` — retained non-inventory rendering; Structures exit reuses identical 10.1.0 rendering.
- `src/c_api/patch_retired_members.rs` — existing retail-gated removed-member markers.
- `data/patch-api/sources/10.0.7-*` — pinned source, provenance, inventory and exhaustive coverage ledger.
- `tests/data/patch_10_0_7_sweep_known_gaps.json` — exact reviewed gap set.

## Tests asserting this spec

- `tests/patch_10_0_7_publication_sweep.rs` — cached Game publication/absence and exact fixture.
- `tests/patch_10_0_7_publication_fixes.rs` — repeated bare-environment lookup and retained social queries.
- `tests/patch_10_0_7_cached_surfaces.rs` — same assertions after cached Game startup.
- `tests/patch_10_0_7_classic_surfaces.rs` — Mists legacy lookup preservation.
- `tools/test_gen_patch_wikitext_register.py`, `tools/test_extract_patch_non_inventory.py` — two reproduced source-loss boundaries.

## Known gaps (current cycle)

- [ ] Twenty-six publication gaps require concrete producers or supported current graphics-CVar state; [per-ID review](../../data/patch-api/evidence/10.0.7-session-2026-10-07/p1007-gap-review.json) owns reasons.
- [ ] 2,203 substantive extract occurrences need occurrence-specific behavioral proof; [scout](../../data/patch-api/evidence/10.0.7-session-2026-10-07/p1007-extract-scout.json) retains literal statements and boundaries.
- [ ] Integration adds 10.1.0 and recomputes fixtures. Read-only comparison finds no intersection with the 26 retained gap IDs.

## Out of scope

Historical epoch reconstruction, new placeholders, linked-page expansion, vendor/cache edits, native parity, full suites, agents/models, push and merge. Documentation/accounting completion does not close retained compatibility gaps.
