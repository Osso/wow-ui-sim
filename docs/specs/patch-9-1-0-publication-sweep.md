# Patch 9.1.0 publication sweep

Account for page 17181 revision 167980 against current retail 12.1.0, not historical reconstruction.

## Required behavior

- [x] Retain pinned source, provenance flags, extract and all 179 inventory occurrences.
- [x] Parse the plain widget Scripts label only with an opt-in flag; preserve earlier registers.
- [x] Review exact cached publication gaps and later-register supersession.
- [x] Account for every extract/context ID without granting publication-only behavior credit.
- [x] Prove all publication sweeps, negative control, parser fixtures, formatting, Mists tests check and retail startup [].

## Proof boundary

192 IDs: 61 current absences, 65 publication/registration-only rows, 56 pending rows (53 exact gaps plus three summary contracts), ten metadata rows. Twenty-one unused namespace retirements closed; post-load tooltip placeholder remains classic-only. Two historical CVar defaults differ without current-state rewrites. Thirty sweeps have combined current-scope proof; first aggregate run's own 9.1.0 failure was resolved by the subsequent focused run, not hidden as an aggregate pass. See [audit](../wiki/investigations/patch-9-1-0-api-audit.md).

## Exclusions

No vendor/cache edits, placeholders, historical reconstruction, broad suites, agents/models, push or merge. Retail retirements preserve classic profiles and deprecation wrappers. Concurrent 9.1.5 register remains read-only and is inserted by integration.
