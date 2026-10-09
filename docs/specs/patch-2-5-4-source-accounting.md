# TBC Classic Patch 2.5.4 SOURCE accounting

Account for frozen Warcraft Wiki page 515131/revision 4967736, timestamp 2023-06-20T22:06:32Z, in the separate TBC Classic 205xx history. Source lives in `data/patch-api/sources/2.5.4-api-changes.wikitext`. This is SOURCE/contract accounting, not API parity. [Audit](../wiki/investigations/patch-2-5-4-api-audit.md).

## What it must do
- [x] Validate original response/content/manifest/registry hashes and identity; retain registry through 1.0.0. Identity and historical replay pass.
- [x] Account for every nonblank literal row, API occurrence, numerical header and known/unspecified signature; preserve summary/transclusion/linked-source boundaries without expanding them. Own eight tests pass, including every row/contract omission control.
- [x] Keep linked/native/state contracts precisely UNPROVEN; no positive empty-inventory credit or blanket unsupported-API diagnosis. Contract/profile credit mutations rejected.
- [x] Separate literal source TOC from actual configured interfaces; allow only pending same-TBC successors, no foreign supersession. Configuration/history fixtures pass.
- [x] Replay sealed original snapshots/tools/ledger/proof after copying without Git/target/current tools/files; reject serialized tampering and restore exact bytes/hashes. [Copied proof and three serialized controls](../../data/patch-api/evidence/2.5.4-session-2026-10-09/portable-proof.json) pass at `ae9cb498a`, 27 original seals.

## How it works
- [Coverage and proof matrix](../wiki/investigations/patch-2-5-4-api-audit.md).

## Implementation inventory
- `data/patch-api/sources/2.5.4-*` — source and literal SOURCE ledger.
- `data/patch-api/evidence/2.5.4-session-2026-10-09/` — own frozen inputs, configured snapshots, historical tools, targeted tests and replay.

## Tests asserting this spec
`data/patch-api/evidence/2.5.4-session-2026-10-09/test_source_accounting.py` tests serialized source contracts only. [Source proof ledger](../../data/patch-api/evidence/2.5.4-session-2026-10-09/source-proof.json) retains exact RED revision and GREEN byte scope at `80a50f32c`; [portable proof](../../data/patch-api/evidence/2.5.4-session-2026-10-09/portable-proof.json) records argv/cwd/environment/revision for copied replay and serialized ledger/proof/log rejection-restoration. These are source-accounting receipts, not native acceptance.

## Known gaps (current cycle)
- [ ] Native publication/absence, argument/return, payload, security, lifecycle and state behavior remain unmeasured. Main owns native gates and integration of pending 2.5.5/2.5.6 references.

## Out of scope
Runtime/profile/shared-classifier edits, shims/new models/fallbacks, linked-source expansion, cross-client supersession, vendor/cache/Wowless changes, broad/check/lint/profile/startup/full-suite/final/native gates, delegation and operational changes.
