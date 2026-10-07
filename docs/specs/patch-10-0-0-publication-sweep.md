# Patch 10.0.0 publication sweep

Account for Warcraft Wiki page 530068, revision 6789768 (2026-07-31T04:32:53Z), against current retail 12.1.0. Publication accounting is not API compatibility.

## What it must do

- [ ] Probe all 639 inventory occurrences using all 25 chronological later registers, 10.0.2 through 12.1.0, with an exact reviewed gap fixture.
- [ ] Account for all 439 nonblank extract occurrences; preserve Lua/XML examples and distinguish pending contracts from publication proof.
- [ ] Construct Scale, Path and FlipBook through animation owners; recognize owner-qualified widget-script links without changing their source identity.
- [ ] Detect an exact single-row negative control without changing IDs or count.
- [ ] Preserve prior sources, registers, ledgers, known-gap fixtures and both extractor-mode outcomes.
- [ ] Run all publication sweeps, factory regression, formatting, Mists tests check and retail startup error query.

## Out of scope

Historical epochs, native parity, broad suites, placeholders, vendor/cache edits, agents/models, push and merge. Missing service-backed behavior must remain an explicit gap rather than an autostub promoted to an implementation.

## Implementation inventory

- `data/patch-api/sources/10.0.0-*` — source, provenance, register, extract and occurrence ledger.
- `tests/patch_10_0_0_publication_sweep.rs` — prefork sweep and animation-construction regression.
- `tests/common/publication_sweep.rs` — shared classifier; no production API changes.
- `tools/gen_patch_wikitext_register.py` and `tools/extract_patch_non_inventory.py` — fixture-backed prepatch markup support.
- `data/patch-api/evidence/10.0.0-session-2026-10-07/` — per-ID review and proof.

## How it works

- [Audit](../wiki/investigations/patch-10-0-0-api-audit.md) owns counts, retained reasons and verification boundaries.
- [Prefork performance](../wiki/investigations/test-suite-performance.md) explains isolated children from the shared full-UI preload.
