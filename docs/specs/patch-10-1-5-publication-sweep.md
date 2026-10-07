# Patch 10.1.5 publication sweep

Account for Warcraft Wiki page 396196, revision 3807695 (2023-08-02T17:30:43Z). Default retail carries 12.1.0, not reconstructed 10.1.5.

## Requirements

- [x] Parse all 101 inventory occurrences, including level-two headings; retain source header mismatches rather than dropping rows.
- [x] Probe publication/absence using nineteen chronological later registers, 10.2.0 through 12.1.0. Insert independently integrated 10.1.7 first when available.
- [x] Retail/PTR must not fabricate unused C_CampaignInfo.UsesNormalQuestIcons. Classic registrations and cached Blizzard Lua remain unchanged.
- [ ] Require exact reviewed publication gaps and exhaustive patch-page-coverage/v1 source accounting. Publication, event registration and default values do not imply signatures, populated outputs, security or native parity.
- [ ] Preserve all existing registers byte-identically and extracts reproducibly; prove parser extensions with behavioral fixtures.

## Tests

- `tests/patch_10_1_5_publication_sweep.rs` — cached Game publication/absence and exact gap fixture.
- `tests/patch_10_1_5_publication_fixes.rs` — repeated namespace lookup retirement and retained successor publication.
- `tests/patch_10_1_5_cached_surfaces.rs` — same retirement after cached Game load.
- `tools/test_gen_patch_wikitext_register.py`, `tools/test_extract_patch_non_inventory.py` — level-two inventory and retained enum/constant/structure output fixtures.

## Out of scope

Historical epochs, placeholders, native parity, linked-page expansion, vendor/cache changes, 3D rendering, full suites, push, merge and agents/models.

## Evidence

[Page audit](../wiki/investigations/patch-10-1-5-api-audit.md) owns per-ID boundaries and local proof.
