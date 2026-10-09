# Patch 5.5.2 publication sweep

Audit the pinned Warcraft Wiki page 686956 revision 6778080 as Mists Classic, not retail MoP. [Audit and proof boundaries](../wiki/investigations/patch-5-5-2-api-audit.md).

## What it must do

- [x] Account for every register and extract row; an empty inventory confers no positive API coverage.
- [x] Execute under `client-mists`, current interface 50504 and the Mists cache; ancestor TOC 50502 is the same client line.
- [x] Load real cached Mists SharedXMLBase and SharedXML without Lua errors.
- [x] Use only 5.5.3 and 5.5.4 as later same-line registers; leave retail chains untouched.
- [x] Reject an injected inventory row against the empty-inventory count.
- [x] Preserve all retail publication observations and pass the portable validator gate.

## How it works

- [5.5.4 harness decision](../wiki/investigations/patch-5-5-4-api-audit.md).
- [Validator portability](../wiki/investigations/patch-audit-validator-portability.md).

## Implementation inventory

- `tests/patch_5_5_2_publication_sweep.rs`: Mists-only publication boundary.
- `data/patch-api/sources/5.5.2-*`: pinned source, register and metadata accounting.
- `data/patch-api/evidence/5.5.2-session-2026-10-08/`: reproduction, scans, proof receipts and validator.

## Tests asserting this spec

- `tests/patch_5_5_2_publication_sweep.rs`.
- `tests/publication_sweep_client_lines.rs`: retail client-line controls.
- Evidence `validate.py`: historical source, receipt and accounting assertions.

## Known gaps (current cycle)

None in the source-listed inventory. Committed proof at `0a030682a` passes the [branch gate](../../data/patch-api/evidence/5.5.2-session-2026-10-08/gate-summary.json): 44/44 clean and 45/45 after the synthetic later audit. All requested tests/checks and both negative controls pass within the documented boundaries.

## Out of scope

Linked GitHub comparisons are unexpanded external boundaries. No API identities or removals appear in this revision. No positive API, full-Game startup or native signature/output/security parity is claimed.
