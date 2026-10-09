# Patch 4.1.0 publication and source accounting

Account for the pinned historical retail [source](../../data/patch-api/sources/4.1.0-api-changes.wikitext), revision 4094671. Publication, prose/signature contracts and native parity are separate claims. See [audit](../wiki/investigations/patch-4-1-0-api-audit.md).

## What it must do

- [ ] Opt-in parsing retains NEW groups, whitespace-padded references, multi-name removals and changed combat-log events with original source lines; default generator bytes remain unchanged.
- [ ] Every register identity and retained prose statement has explicit coverage or a precise gap.
- [ ] The sweep applies only later retail registers, ordered 4.2.0, 4.3.0, 4.3.4 then 5.0.1 onward; queued pages receive no fabricated successor credit.
- [ ] Evidence validation derives counts from sealed source/accounting files and rejects tampering.

## How it works

- [Audit](../wiki/investigations/patch-4-1-0-api-audit.md).
- [Publication helper](../../tests/common/publication_sweep.rs).

## Implementation inventory

- `tools/gen_patch_wikitext_register.py`: opt-in Cataclysm change-bullet parser.
- `tools/test_patch_4_1_register.py`: concrete output and default-byte compatibility fixture.
- `data/patch-api/sources/4.1.0-*`: frozen source, register, extraction and accounting.

## Tests asserting this spec

- `tools/test_patch_4_1_register.py`.

## Known gaps (current cycle)

- [ ] Current retail observations and historical prose/signature contracts require independent accounting; callable publication alone is insufficient.

## Out of scope

Native 2011 client parity without native receipts; Classic successor credit; vendor changes; network acquisition; main-thread final acceptance gates.
