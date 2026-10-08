# Patch 8.1.5 publication sweep

Audit current retail publication against pinned Warcraft Wiki page 394493, revision 3789732; not historical reconstruction.

## What it must do

- [x] Pin raw source, provenance and fetch receipts; retain every API/event bullet and every non-inventory occurrence with opt-in reproducible tooling.
- [x] Discover exact current-retail gaps using later registers, with an 8.2.0 integration placeholder first.
- [x] Retire only consumer-free namespace removals; preserve globals, classic lookup and all vendor files.
- [x] Track newly acquired catalog toys until fanfare is cleared: `NeedsFanfare` and `GetToyInfo` fifth result agree, clearing preserves ownership, repeated collection does not rewrap, uncollection clears pending state.
- [x] Account for all 85 occurrences and 24 exact problematic gaps, with complete whole-word scans for all 28 removals and exact 24 → 25 negative control.
- [x] Prove all 38 publication sweeps plus factory, touched behavior, 51 parser fixtures, 38 register reproductions, formerly reproducible extracts, format and warning-clean non-vendor Mists check; dynamic validator passes.

## Implementation inventory

- `src/c_api/c_toy_box_info.rs`: pending-fanfare backing state and C API queries/mutation.
- `src/c_api/patch_retired_members.rs`: separate retail-only 8.1.5 removals.
- Existing collection producer and `C_ToyBox.GetToyInfo` read/write the same backing state.
- `tools/gen_patch_wikitext_register.py`, `tools/extract_patch_non_inventory.py`: independent opt-in `--legacy-api-bullets`.
- `data/patch-api/sources/8.1.5-*` and `data/patch-api/evidence/8.1.5-session-2026-10-08/`: source and proof.

## Tests asserting this spec

- `tests/patch_8_1_5_publication_sweep.rs`, `tests/patch_8_1_5_publication_fixes.rs`, `tests/patch_8_1_5_cached_surfaces.rs`.
- `tests/patch_8_1_5_toy_fanfare.rs`: acquisition/repetition/clear/uncollection through Lua, including unknown ID.
- Both Python parser fixture suites and evidence `validate.py`.

## Known gaps

- [ ] Publication does not prove populated outputs, signatures, security, native acquisition events or historical defaults. Toy acquisition uses the existing admin producer; native NEW_TOY_ADDED payload/timing parity is not claimed.
- [ ] Remaining publication gaps require backing producers, consumer migration, or scoped native-global publication decisions; see final per-ID review.
- [ ] 8.2.0 supersession awaits main-thread integration.
- [ ] Inherited 12.0.5/12.0.7/12.1.0 saved extracts remain non-reproducible, unchanged by this page.

## Out of scope

Vendor/cache/Wowless writes, Blizzard overrides, shims, other-page implementation, native 3D, full integration suite, session cwd changes, agents/model CLIs, push and merge.

## How it works

[Audit](../wiki/investigations/patch-8-1-5-api-audit.md).
