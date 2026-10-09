# Retail Patch 5.1.0 publication sweep

Audit the pinned 2012 retail [source](../../data/patch-api/sources/5.1.0-api-changes.provenance.json), including its separately pinned diff. [Audit](../wiki/investigations/patch-5-1-0-api-audit.md) records implementation and proof boundaries.

## What it must do

- [x] Account for all register occurrences and extract statements without equating publication with behavior.
- [x] Apply only later retail registers, starting with merged 5.2.0, 5.3.0, 5.4.0 and 5.4.1; never 5.5.x Classic.
- [x] Keep consumer-free `C_PetJournal.GetSummonedPetID` and `SummonPetByID` absent under raw and repeated ordinary lookup; retain GUID successors and current journal publication.
- [x] Read/update/clear a concrete existing action/spell loss-control interval through the cached legacy global wrapper.
- [x] Reproduce saved sources/registers and pass clean-checkout/later-audit validators using committed, pinned inputs.

## How it works

- [Audit and evidence](../wiki/investigations/patch-5-1-0-api-audit.md).
- [Validator portability](../wiki/investigations/patch-audit-validator-portability.md).

## Implementation inventory

- `tools/gen_patch_wikitext_register.py`: opt-in summary, transcluded inventory, owner-specific handlers and code-wrapped removals.
- `src/c_api/patch_retired_members.rs`: two retail-only ID spelling retirements.
- `tests/patch_5_1_0_publication_sweep.rs`: current retail publication/absence.
- `tests/patch_5_1_0_behavior.rs`: bare/full-LoD absence and existing interval backing.
- `data/patch-api/sources/5.1.0-page-coverage.json`: exhaustive occurrence/prose ledger.

## Tests asserting this spec

- `tests/patch_5_1_0_publication_sweep.rs`.
- `tests/patch_5_1_0_behavior.rs`.
- `tools/test_patch_5_1_register.py`.
- `data/patch-api/evidence/5.1.0-session-2026-10-08/validate.py`.

## Known gaps (current cycle)

- [ ] Exact pending API occurrences remain in `tests/data/patch_5_1_0_sweep_known_gaps.json`; reasons in the ledger and session gap review.
- [ ] Historical loss-control notification, instance chat routing, owned-pet identity/summons, restricted-environment security and automatic cooldown visibility remain unproved.

## Out of scope

Native 2012 gameplay/return/security parity, Classic history, vendor edits, compatibility shims, and full integration suite. Current interval fixtures are not native crowd-control captures.
