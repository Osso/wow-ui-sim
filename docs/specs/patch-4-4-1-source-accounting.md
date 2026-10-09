# Cataclysm Classic 4.4.1 source accounting

Account for Warcraft Wiki page 608915, revision 6234252, TOC 40401 at pinned source commit `5960d3b73c48895dc216c4ecf1afccc8b51aa074`. [Source-only ledger](../../data/patch-api/sources/4.4.1-page-coverage.json) and [audit](../wiki/investigations/patch-4-4-1-api-audit.md) distinguish exact source proof from unsupported runtime claims.

## What it must do

- [x] Preserve returned revision/content identity, exact source/response/text bytes, serialized ledger hash and all 19 nonblank source rows (17 metadata/markup, two occurrences).
- [x] Preserve navigation/resources/TOC/both linked diffs, 4.4.0 → 4.4.1 direction, four literal `x` counts and `xxx xx 2024`; reject omitted rows, invented numbers/date and reversed direction.
- [x] Reject runtime credit, retail profile, register/supersession and shared-pin mutations; portable validator depends on committed artifacts, not changing HEAD/shared-file hashes.
- [ ] Added `C_SpecializationInfo.GetNumSpecializationsForClassID` publication/behavior in Cataclysm Classic: **UNPROVEN**, missing supported Cata profile.
- [ ] Removed `GetNumSpecializationsForClassID` absence in Cataclysm Classic: **UNPROVEN**, missing supported Cata profile. No retirement of the live retail global.

## How it works

- [Exact row/proof matrix and runtime boundary](../wiki/investigations/patch-4-4-1-api-audit.md).
- [Merged 4.4.2 precedent](patch-4-4-2-source-accounting.md).

## Implementation inventory

- `data/patch-api/sources/4.4.1-api-changes.wikitext`: exact pinned source.
- `data/patch-api/sources/4.4.1-api-changes.txt`: reproduced non-inventory text.
- `data/patch-api/sources/4.4.1-page-coverage.json`: source-only occurrence/metadata ledger, not a runtime register.
- `data/patch-api/evidence/4.4.1-session-2026-10-08/`: returned response, provenance, validator and final receipt.

## Tests asserting this spec

`python3 -B data/patch-api/evidence/4.4.1-session-2026-10-08/validate.py`: four tests, positive external serialized contracts and 19 negative cases. `python3 -B tools/extract_patch_non_inventory.py --patch 4.4.1 --text-only --canonical-patch-navigation --check`: plaintext reproduction. `python3 -B tools/check_patch_validators.py HEAD`: clean and synthetic later-audit portability gate. Final committed receipt records commands/revisions/results; none establishes runtime compatibility.

## Known gaps (current cycle)

- [ ] Both occurrence runtime contracts remain UNPROVEN: no Cata publication/removal, signature, output, security, behavior, UI-load or native proof.

## Out of scope

New runtime/profile/shared classifier, Cargo builds, runtime/native probes, Mists/retail stand-ins, live retail retirement, retail supersession/publication registers and external diff expansion. Source accounting cannot establish any of them.
