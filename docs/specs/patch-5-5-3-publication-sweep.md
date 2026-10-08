# Patch 5.5.3 API page audit

Audit the pinned [source](../../data/patch-api/sources/5.5.3-api-changes.wikitext) as `mists-classic`, against the current `client-mists` profile. TOC 50503 is an ancestor build of the 50504 profile; same client line.

## What it must do

- Preserve zero inventory entries/header counts and account for all four metadata rows without API coverage credit.
- Load real cached Mists SharedXML for the empty-inventory discovery proof; reject wrong-profile execution through the shared classifier.
- Use only 5.5.4 in this page's later-register chain; leave retail registers and observations unchanged.
- Preserve blocker history with the explicit ancestor-build resolution.
- Reproduce saved registers/extracts and validate committed proof in clean and unrelated-later-audit checkouts.

## Tests asserting this spec

- `tests/patch_5_5_3_publication_sweep.rs`: current Mists profile/cache and SharedXML-only publication boundary.
- `tests/patch_5_5_4_publication_sweep.rs`: Mists client-line controls.
- `tests/publication_sweep_client_lines.rs`: retail isolation and profile rejection.
- `data/patch-api/evidence/5.5.3-session-2026-10-08/validate.py`: source accounting, historical inputs and retained proof.

## Known gaps and exclusions

No inventory contracts to implement or retire. Linked GitHub comparisons remain unexpanded external boundaries. Empty inventory establishes neither positive API coverage nor native signature/output/security parity. SharedXML-only discovery is not full-Game startup proof. No runtime, cache, vendor or tooling behavior changes are required.

## How it works

- [Audit](../wiki/investigations/patch-5-5-3-api-audit.md).
- [5.5.4 harness decision](../wiki/investigations/patch-5-5-4-api-audit.md).
- [Validator portability](../wiki/investigations/patch-audit-validator-portability.md).
