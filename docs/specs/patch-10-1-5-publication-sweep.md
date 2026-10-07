# Patch 10.1.5 publication sweep

Account for Warcraft Wiki page 396196, revision 3807695 (2023-08-02T17:30:43Z). Default retail carries 12.1.0, not reconstructed 10.1.5.

## Requirements

- [x] Parse all 101 inventory occurrences, including level-two headings; retain source header mismatches rather than dropping rows.
- [x] Probe publication/absence using twenty chronological later registers, 10.1.7 through 12.1.0 (10.1.7 added at integration; gaps unchanged).
- [x] Retail/PTR must not fabricate unused C_CampaignInfo.UsesNormalQuestIcons or register RequestArtifactCompletionHistory. Classic registrations, archaeology availability/data queries and cached Blizzard Lua remain unchanged.
- [x] Require exact reviewed publication gaps and exhaustive patch-page-coverage/v1 source accounting. Publication, event registration and default values do not imply signatures, populated outputs, security or native parity.
- [x] Preserve existing registers byte-identically and generated extracts reproducibly; prove parser extensions with behavioral fixtures. External MediaWiki/crawler plaintext captures remain byte-identical, not recast as generator outputs.

## Tests

- `tests/patch_10_1_5_publication_sweep.rs` — cached Game publication/absence and exact gap fixture.
- `tests/patch_10_1_5_publication_fixes.rs` — repeated namespace lookup retirement and retained successor publication.
- `tests/patch_10_1_5_cached_surfaces.rs` — same retirement after cached Game load.
- `tools/test_gen_patch_wikitext_register.py`, `tools/test_extract_patch_non_inventory.py` — level-two inventory and retained enum/constant/structure output fixtures.

## Out of scope

Historical epochs, placeholders, native parity, linked-page expansion, vendor/cache changes, 3D rendering, full suites, push, merge and agents/models.

## Evidence

[Page audit](../wiki/investigations/patch-10-1-5-api-audit.md) owns per-ID boundaries and local proof.

## Local proof

Runtime/test revision `2474d039a`: twenty isolated exact sweeps, retirement RED/GREEN, eight history regressions, campaign cases, one cached Game prefork, Mists history preservation, negative control 32 → 33, formatting, Mists test check zero non-vendor warnings and exit-0 startup `[]`. Twenty byte-identical registers, seventeen generated extracts and 122 preserved baseline inputs. External later captures are not extractor outputs. [Proof ledger](../../data/patch-api/evidence/10.1.5-session-2026-10-07/p1015-proof.json) owns commands, hashes and exact scope. Local development proof is not native or independent acceptance.

| Patch | Rows | OK | Gaps | Isolated exit |
|---|---:|---:|---:|---:|
| 10.1.5 | 101 | 69 | 32 | 0 |
| 10.2.0 | 150 | 120 | 30 | 0 |
| 10.2.5 | 59 | 45 | 14 | 0 |
| 10.2.6 | 220 | 200 | 20 | 0 |
| 10.2.7 | 104 | 68 | 36 | 0 |
| 11.0.0 | 495 | 329 | 166 | 0 |
| 11.0.2 | 34 | 22 | 12 | 0 |
| 11.0.5 | 48 | 38 | 10 | 0 |
| 11.0.7 | 98 | 70 | 28 | 0 |
| 11.1.0 | 116 | 97 | 19 | 0 |
| 11.1.5 | 125 | 89 | 36 | 0 |
| 11.1.7 | 48 | 40 | 8 | 0 |
| 11.2.0 | 162 | 135 | 27 | 0 |
| 11.2.5 | 163 | 118 | 45 | 0 |
| 11.2.7 | 508 | 414 | 94 | 0 |
| 12.0.0 | 1010 | 989 | 21 | 0 |
| 12.0.1 | 225 | 222 | 3 | 0 |
| 12.0.5 | 363 | 352 | 11 | 0 |
| 12.0.7 | 174 | 171 | 3 | 0 |
| 12.1.0 | 778 | 773 | 5 | 0 |
