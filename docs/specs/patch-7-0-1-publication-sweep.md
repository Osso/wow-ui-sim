# Patch 7.0.1 publication sweep

Audit [pinned source](../../data/patch-api/sources/7.0.1-api-changes.wikitext), pageid 336026 revision 3241525. This revision redirects to Patch 7.0.3/API changes; it contains no API inventory. [Audit mechanics and proof](../wiki/investigations/patch-7-0-1-api-audit.md).

## What it must do

- [ ] Reproduce the empty register and retained redirect extract from the pinned revision without expanding the target page.
- [ ] Account for the single redirect context row as metadata-only, with no runtime capability credit.
- [ ] Execute the prefork sweep against the exact empty inventory and gap set; record zero observations explicitly.
- [ ] Preserve historical artifacts and validate proof from any checkout after later audits add registers.

## How it works

- [Audit](../wiki/investigations/patch-7-0-1-api-audit.md).
- [Historical validator portability](../wiki/investigations/patch-audit-validator-portability.md).

## Implementation inventory

- `tests/patch_7_0_1_publication_sweep.rs`: cached full-UI publication sweep, with 7.0.3 integration placeholder first.
- `tests/data/patch_7_0_1_sweep_known_gaps.json`: empty expected gap set.
- `data/patch-api/sources/7.0.1-*`: pinned source, provenance, extract, empty register and context ledger.
- `data/patch-api/evidence/7.0.1-session-2026-10-08/validate.py`: portable evidence validator.

## Tests asserting this spec

- `cargo test --test prefork_full_ui -- patch_7_0_1`.
- Evidence `validate.py`, source reproduction and existing Python fixture scripts.

## Known gaps (current cycle)

- [ ] Targeted verification and evidence sealing pending.

## Out of scope

Target-page API and behavior audit belongs to the parallel 7.0.3 audit. Zero rows here does not establish target-page coverage. No runtime retirements, shims, backing-model changes or vendor edits are needed for a redirect.
