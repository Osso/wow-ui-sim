# Patch 6.1.0 publication sweep

Audit the pinned [6.1.0 page](../../data/patch-api/sources/6.1.0-api-changes.provenance.json), recording current-retail publication/absence independently from historical behavior. [Audit](../wiki/investigations/patch-6-1-0-api-audit.md) describes accounting and evidence.

## What it must do

- [ ] Retain all four explicit API identities and every nonblank supplemental extract occurrence, with precise coverage or problematic reasons.
- [ ] Probe the three legacy recap globals and SendChatMessage against later-register supersession; require the exact known-gap set, not a count-only allowance.
- [ ] Detect a missing-API negative control without changing unaffected observations.
- [ ] Reproduce registers using recorded opt-in flags and preserve old extracts' outcomes, including inherited failures.
- [ ] Validate historical proof from another checkout and after newer audits merge, without absolute-path equality or comparisons to moving source/evidence digests.

## How it works

- [Audit and portability boundaries](../wiki/investigations/patch-6-1-0-api-audit.md).
- [Historical validation conventions](../wiki/investigations/patch-audit-validator-portability.md).

## Implementation inventory

- `tests/patch_6_1_0_publication_sweep.rs` — prefork publication/absence sweep and ordered integration placeholders.
- `tests/common/publication_sweep.rs` — unchanged shared classifier, supersession and exact known-gap comparison.
- `tests/data/patch_6_1_0_sweep_known_gaps.json` — exact retained SendChatMessage absence mismatch.
- `tools/gen_patch_wikitext_register.py` — opt-in colon-prefixed standalone API bullet parser.
- `tools/extract_patch_non_inventory.py` — borrowed opt-in unexpanded patch-diff reference rendering.
- `data/patch-api/sources/6.1.0-page-coverage.json` — per-occurrence coverage SSOT.
- `data/patch-api/evidence/6.1.0-session-2026-10-08/validate.py` — read-only historical gate.

## Tests asserting this spec

- `tests/patch_6_1_0_publication_sweep.rs`.
- `tools/test_gen_patch_wikitext_register.py` and `tools/test_extract_patch_non_inventory.py`.
- Evidence `validate.py`, negative-control register/results and source reproduction receipts.

## Known gaps (current cycle)

- [ ] SendChatMessage is still simulator-published despite later historical removal metadata; active callers preclude retirement.
- [ ] Legacy recap event/link backing, automatic death-watch lifecycle, historical Collections consolidation, SocialUI integration and native invalid-UTF8 rejection remain precisely recorded rather than credited as implemented.

## Out of scope

- Native-client parity without native evidence; hypothetical signatures/return values are not modeled.
- Reproducing a historical TGA/PNG client crash.
- Expanding the separate automated-diff transclusion or external service/network operations.
- Vendor/Wowless edits, shims, full integration suite, pushing or merging.
