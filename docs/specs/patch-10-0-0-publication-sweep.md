# Patch 10.0.0 publication sweep

Account for Warcraft Wiki page 530068, revision 6789768 (2026-07-31T04:32:53Z), against current retail 12.1.0. [Page audit](../wiki/investigations/patch-10-0-0-api-audit.md) owns occurrence accounting and proof boundaries. Publication accounting is not API compatibility.

## What it must do

- [x] Probe all 639 inventory occurrences using all 25 chronological later registers, 10.0.2 through 12.1.0, requiring exactly 173 reviewed gaps.
- [x] Account for all 439 nonblank extract occurrences and four inventory-context lines; preserve Lua examples and distinguish pending statements from publication proof.
- [x] Construct Scale, Path and FlipBook through animation owners; recognize owner-qualified widget-script links without changing source identities. Missing endpoints remain gaps.
- [x] Account for every API on shared changed lines with its source line and annotations; preserve prior generation through explicit expansion mode.
- [x] Detect the exact single-row GetUnitEmpowerHoldAtMaxTime negative control: 173 → 174 gaps, unchanged row count/IDs.
- [x] Preserve all 158 prior inputs, 50 extraction-mode outcomes and byte-identical regeneration of all 26 registers.
- [x] Verify all publication sweeps, factory/helper regressions, formatting, Mists tests check and exit-0 retail startup `[]`.

## How it works

- [Audit](../wiki/investigations/patch-10-0-0-api-audit.md) — counts, source-loss boundaries, retained reasons and verification.
- [Prefork performance](../wiki/investigations/test-suite-performance.md) — isolated children from the shared full-UI preload.

## Implementation inventory

- `data/patch-api/sources/10.0.0-*` — source, provenance, register, extract and 1,082-ID ledger.
- `tests/patch_10_0_0_publication_sweep.rs` — prefork sweep and animation-construction regression.
- `tests/common/publication_sweep.rs` — shared classifier; no production API changes.
- `tools/gen_patch_wikitext_register.py` and `tools/extract_patch_non_inventory.py` — fixture-backed prepatch markup/shared-reference support.
- `data/patch-api/evidence/10.0.0-session-2026-10-07/` — per-ID review, source/caller scans, proof ledger and artifact validator.

## Tests asserting this spec

- `tests/patch_10_0_0_publication_sweep.rs` — current cached publication/absence and owner construction.
- `tools/test_gen_patch_wikitext_register.py` — inline Commands, owner-qualified handler links and shared annotations.
- `tools/test_extract_patch_non_inventory.py` — layout clearing and verbatim examples.
- `data/patch-api/evidence/10.0.0-session-2026-10-07/validate.py` — accounting/preservation/negative-control artifacts.
- Existing 10.2.0/10.2.5 interface probes, loaded alias attribution and cached 12.0.0 successor test — shared-helper regressions.

## Known gaps

- [ ] 173 publication gaps: [per-ID review](../../data/patch-api/evidence/10.0.0-session-2026-10-07/p1000-gap-review.json) owns reasons.
- [ ] 424 pending extract statements/examples and three inventory-section migration summaries; no signature, populated output, payload, security, native or historical parity credit.
- [ ] Eleven serialized CVar default comparisons differ from the historical page, including two absent variables. Publication accounting does not erase these differences.

## Out of scope

Historical epochs, native parity, broad suites, placeholders, vendor/cache edits, agents/models, push and merge. Missing service-backed behavior remains explicit rather than an autostub promoted to an implementation. This audit makes no runtime API changes or retirements.
