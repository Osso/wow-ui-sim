# Patch 4.1.0 publication and source accounting

Account for the pinned historical retail [source](../../data/patch-api/sources/4.1.0-api-changes.wikitext), revision 4094671. Publication, prose/signature contracts and native parity are separate claims. See [audit](../wiki/investigations/patch-4-1-0-api-audit.md).

## What it must do

- [x] Opt-in parsing retains NEW groups, whitespace-padded references, multi-name removals and changed combat-log events with original source lines; default generator bytes remain unchanged.
- [x] Every register identity and retained prose statement has explicit coverage or a precise gap.
- [x] The sweep applies actual later retail registers, ordered [4.2.0](patch-4-2-0-publication-sweep.md), [4.3.0](patch-4-3-0-publication-sweep.md), [4.3.4](patch-4-3-4-publication-sweep.md), then 5.0.1 onward. Sole overlap: 4.3.0 removes `IsIPv6Available`, yielding 51 matches / 30 gaps; removal proves neither an IPv6 model nor native parity.
- [x] Evidence validation derives counts from sealed source/accounting files and rejects tampering.

- [x] Retail raw and ordinary `GetPetHappiness` lookups are absent; non-retail default/seeded pet-state outputs are unchanged.

## How it works

- [Audit](../wiki/investigations/patch-4-1-0-api-audit.md).
- [Publication helper](../../tests/common/publication_sweep.rs).

## Implementation inventory

- `tools/gen_patch_wikitext_register.py`: opt-in Cataclysm change-bullet parser.
- `tools/test_patch_4_1_register.py`: concrete output and default-byte compatibility fixture.
- `data/patch-api/sources/4.1.0-*`: frozen source, register, extraction and accounting.

## Tests asserting this spec

- `tools/test_patch_4_1_register.py`.
- `tests/pet_stats.rs`: retail absence and non-retail pet-state behavior.
- `tests/patch_4_1_0_publication_sweep.rs`: cached retail publication/absence.

## Known gaps (current cycle)

Historical capture remains 81 inventory occurrences, 50 matches / 31 gaps; 86 extract rows (78 metadata / eight unproven prose), four signatures; 171 ledger IDs (50 bounded / 43 pending / 78 metadata). Current publication accounting is 51 matches / 30 gaps, not a rewrite of that frozen ledger. See [proof separation and pending acceptance](../wiki/investigations/patch-4-1-0-api-audit.md#integration-boundary).

- [ ] Current retail observations and historical prose/signature contracts require independent accounting; callable publication alone is insufficient.

## Out of scope

Native 2011 client parity without native receipts; Classic successor credit; vendor changes; network acquisition; main-thread final acceptance gates.
