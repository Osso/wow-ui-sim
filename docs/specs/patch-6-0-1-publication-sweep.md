# Patch 6.0.1 publication sweep

Audit Warcraft Wiki pageid 3058 revision 31159 as the literal redirect to Patch 6.0.2/API changes. [Audit](../wiki/investigations/patch-6-0-1-api-audit.md).

## What it must do

- [x] Pin the complete fresh response, wikitext and revision provenance without following the redirect.
- [x] Reproduce the empty inventory register and redirect extract with existing default tools.
- [x] Account for the sole context ID as metadata-only, with no runtime credit.
- [x] Define a zero-row prefork sweep with an empty expected-gap fixture.
- [x] Place queued 6.0.2, 6.1.0 and 6.2.0 placeholders before merged 6.2.2, 6.2.4 and newer registers.
- [ ] Pass requested targeted proofs and the fresh-checkout/later-audit validator gate.

## How it works

- [Audit and evidence](../wiki/investigations/patch-6-0-1-api-audit.md).
- [Validator portability](../wiki/investigations/patch-audit-validator-portability.md).

## Implementation inventory

- `tests/patch_6_0_1_publication_sweep.rs` — prefork case, no API observations.
- `tests/data/patch_6_0_1_sweep_known_gaps.json` — empty expected gaps.
- `data/patch-api/sources/6.0.1-*` — pinned source, provenance, register, extract and ledger.
- `data/patch-api/evidence/6.0.1-session-2026-10-08/` — complete proofs and read-only validator.

## Tests asserting this spec

- `cargo test --test prefork_full_ui -- patch_6_0_1`.
- Evidence `validate.py` and `tools/check_patch_validators.py`.

## Known gaps (current cycle)

None in the redirect page. Zero inventory is not evidence that patch 6.0.1 changed no APIs.

## Out of scope

Destination-page reconstruction, inferred undocumented changes, runtime modifications, retirements, native-client parity and full integration suites.
