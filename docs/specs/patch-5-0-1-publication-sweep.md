# Patch 5.0.1 publication sweep

Audit historical retail pageid 554410, revision 5344081, as the literal `#REDIRECT [[Patch 5.0.4/API changes]]`. [Audit](../wiki/investigations/patch-5-0-1-api-audit.md).

## What it must do

- [x] Preserve the supplied response and literal redirect without following it.
- [x] Generate empty inventory/header counts and the sole metadata-only context ID using existing default tools.
- [x] Define a zero-row retail prefork sweep and empty expected-gap list; actual integrated 5.0.4 register first, then 5.1.0 and newer retail registers.
- [x] Reject a fabricated entry; retain empty `{}` observations as harness execution, not API proof.
- [x] Reproduce all saved sources, preserving exactly the three inherited extraction failures; pass Python fixtures, own/publication sweeps, format and non-vendor-warning-clean Mists check.
- [x] Seal historical inputs and retain the historical clean/synthetic-later validator gates; current integrated-head gates require separate coordinator proof.

## How it works

[Audit and retained evidence](../wiki/investigations/patch-5-0-1-api-audit.md); [validator portability](../wiki/investigations/patch-audit-validator-portability.md).

## Implementation inventory

- `data/patch-api/sources/5.0.1-*` — source, provenance, empty register, extract and ledger.
- `tests/patch_5_0_1_publication_sweep.rs`, `tests/data/patch_5_0_1_sweep_known_gaps.json` — zero-row prefork case.
- `data/patch-api/evidence/5.0.1-session-2026-10-08/` — pinned receipts, proofs and historical validator.

## Tests asserting this spec

Own prefork filter `patch_5_0_1`, `publication_sweep`, Python fixture programs, retained `validate.py`, and `tools/check_patch_validators.py`.

## Known gaps (current cycle)

No API or behavior statements in this source. Empty inventory does not establish that the patch changed no APIs.

## Out of scope

Destination reconstruction (separate p504-page), runtime or retirement changes, native parity, full-suite execution, push and merge.
