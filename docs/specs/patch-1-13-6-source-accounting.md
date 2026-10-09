# Frozen Era 1.13.6 SOURCE accounting

Account for the literal [frozen page](../../data/patch-api/evidence/1.13.6-session-2026-10-09/source.wikitext), page461367/revision4435603. [Audit](../wiki/investigations/patch-1-13-6-api-audit.md) describes evidence boundaries. This is an accounting contract, not historical runtime compatibility.

## What it must do

- [x] Validate exact manifest/registry/response/body identity, timestamp and hashes.
- [x] Retain every nonblank row, three CVar occurrences, both headings, numerical header, summary, links and templates; omit no literal source boundary.
- [x] Keep unspecified defaults/signatures/effects, unexpanded links and configured11507/source11306 identities separate.
- [x] Preserve own SOURCE RED/GREEN, no-Git copied replay, historical default bytes, disk ledger/log rejection and exact restoration; seal original inputs immutably and later receipts separately.
- [x] Keep ordered same-Era successors pending; publication does not create model/native credit.
- [x] Observe all three current bare-Era getter results, raw C_CVar registration and unknown-name control separately; strict exact current gaps, no historical behavior claim.

## How it works

- [Frozen-page audit](../wiki/investigations/patch-1-13-6-api-audit.md).

## Implementation inventory

- `data/patch-api/evidence/1.13.6-session-2026-10-09/audit.py`: independent literal ledger derivation and seal validation.
- `data/patch-api/evidence/1.13.6-session-2026-10-09/replay_defaults.py`: historical unchanged extraction replay.
- `data/patch-api/evidence/1.13.6-session-2026-10-09/historical-tools/`: immutable snapshots, shared tools unchanged.
- `patch-tests/patch_1_13_6_factory.rs`: bounded current Era getters via narrow publication probe; `Cargo.toml` registers only this test target.

## Tests asserting this spec

- `data/patch-api/evidence/1.13.6-session-2026-10-09/test_source_accounting.py`: seven SOURCE fixtures and omission/fabrication controls.
- `data/patch-api/evidence/1.13.6-session-2026-10-09/test_portable.py`: three copied replay/seal/restoration tests.
- `patch-tests/patch_1_13_6_factory.rs`: exact current getter mismatches and nonexistent-CVar control.

## Known gaps (current cycle)

- [ ] Native11306 publication, allowed values/defaults, effects, persistence, security and linked content remain UNPROVEN.
- [ ] Main-owned ordered successor integration and acceptance.

## Out of scope

Production runtime edits, foreign-history supersession, linked expansion, fabricated source defaults, vendor/Wowless/CASC/cache/dependency/network changes and final gates.
