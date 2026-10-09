# Patch 5.5.1 publication sweep

Audit the pinned Warcraft Wiki page 686954 revision 6778077 as Mists Classic, not retail MoP. [Audit and proof boundaries](../wiki/investigations/patch-5-5-1-api-audit.md).

## What it must do

- [x] Account for every register and extract row; an empty inventory confers no positive API coverage.
- [x] Execute under `client-mists`, current interface 50504 and the Mists cache; ancestor TOC 50501 is the same client line.
- [x] Load real cached Mists SharedXMLBase and SharedXML without Lua errors.
- [x] Use only 5.5.2, 5.5.3 and 5.5.4 as later same-line registers; leave retail chains untouched.
- [x] Reject an injected inventory row against the empty-inventory count.
- [x] Preserve all retail publication observations and pass the portable validator gate.

## How it works

- [5.5.4 harness decision](../wiki/investigations/patch-5-5-4-api-audit.md).
- [Validator portability](../wiki/investigations/patch-audit-validator-portability.md).

## Implementation inventory

- `tests/patch_5_5_1_publication_sweep.rs`: Mists-only publication boundary.
- `data/patch-api/sources/5.5.1-*`: pinned source, register and metadata accounting.
- `data/patch-api/evidence/5.5.1-session-2026-10-08/`: reproduction, scans, proof receipts and validator.

## Tests asserting this spec

- `tests/patch_5_5_1_publication_sweep.rs`.
- `tests/publication_sweep_client_lines.rs`: retail client-line controls.
- Evidence `validate.py`: historical source, receipt and accounting assertions.

## Known gaps (current cycle)

None in the source-listed inventory. Own validator passes with 157 sealed inputs. Branch gate at `ea5fb94cd` passes 45/45 clean and 46/46 after the synthetic later audit; [gate summary](../../data/patch-api/evidence/5.5.1-session-2026-10-08/gate-summary.json). All requested runtime checks and negative controls pass. Three exact inherited extract-reproduction failures remain outside this page’s scope.

## Out of scope

Linked GitHub comparisons are unexpanded external boundaries. No API identities or removals appear in this revision. No positive API, full-Game startup or native signature/output/security parity is claimed.
