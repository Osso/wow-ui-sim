# Patch 6.2.2 publication sweep

Audit the pinned Warcraft Wiki pageid 122147, revision 6209268. Source contains only the `apichanges` navigation template, not a redirect or API inventory. See [audit](../wiki/investigations/patch-6-2-2-api-audit.md).

## What it must do

- [ ] Preserve the complete pinned wikitext and revision provenance.
- [ ] Reproduce an empty register and a navigation-only extract with existing tool defaults.
- [ ] Account for the single extract context row as metadata-only, without runtime credit.
- [ ] Execute the zero-row prefork publication sweep with an empty known-gap set.
- [ ] Keep validator register and prior-validator scopes fixed at recorded Git revisions, independent of checkout location and later audits.

## How it works

- [Audit and evidence](../wiki/investigations/patch-6-2-2-api-audit.md).
- [Historical validator scope](../wiki/investigations/patch-audit-validator-portability.md).

## Implementation inventory

- `tests/patch_6_2_2_publication_sweep.rs` — zero-row prefork case; 6.2.4 and 7.0.1 placeholders precede 7.0.3 and later registers.
- `tests/data/patch_6_2_2_sweep_known_gaps.json` — empty expected gap set.
- `data/patch-api/sources/6.2.2-*` — source, provenance, extract, register, and metadata ledger.
- `data/patch-api/evidence/6.2.2-session-2026-10-08/validate.py` — read-only historical evidence gate.

## Tests asserting this spec

- `tests/patch_6_2_2_publication_sweep.rs`.
- `data/patch-api/evidence/6.2.2-session-2026-10-08/validate.py`.

## Known gaps (current cycle)

None in the pinned page. An empty source inventory is not proof that patch 6.2.2 changed no APIs.

## Out of scope

Reconstructing undocumented patch changes, following navigation/transclusions as new audit sources, runtime changes, retirements, native-client parity, and full integration tests.
