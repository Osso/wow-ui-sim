# Patch 1.15.9 literal SOURCE accounting

Account frozen page 685352 / revision 6780591 / timestamp `2026-07-24T15:29:35Z` offline, preserving literal source and independent Classic Era history. [Audit and proof limits](../wiki/investigations/patch-1-15-9-api-audit.md).

## What it must do

- [x] Validate exact response/raw hashes, revision identity, manifest and 101-page registry through 1.0.0; retain all 37 nonblank rows with literal spelling and occurrences.
- [x] Derive Classic Era/Season of Discovery/Hardcore and TOC 11509 from literal text; keep configured Era/Anniversary 11507 separate from native measurement.
- [x] Account six headers, 14 contracts, four API occurrences, three quoted TBC chat/CVar examples and one partial return removal; reject omitted rows/contracts/headers, foreign history and invented credit.
- [x] Keep unexpanded diffs and unspecified signatures/event/state/security/native contracts UNPROVEN; no Era defaults inferred from TBC examples.
- [x] Replay compact sealed historical inputs in a fresh copied no-Git/target process; reject serialized ledger/log tampering and restore exact bytes/hashes without rewriting original seals.

## How it works

- [Literal coverage matrix and source boundary](../wiki/investigations/patch-1-15-9-api-audit.md).

## Implementation inventory

- `data/patch-api/evidence/1.15.9-session-2026-10-09/audit.py` — own source-specific ledger derivation/validation, not a shared parser or simulator.
- Same directory: frozen source/response/manifest/registry/configuration/tools, `ledger.json`, `test_source_accounting.py`, targeted RED/GREEN logs.

## Tests asserting this spec

`python3 -B data/patch-api/evidence/1.15.9-session-2026-10-09/test_source_accounting.py`: eight source-only fixtures; no simulator invocation. RED/GREEN retained. Historical validator exit 0 and copied SOURCE8/8 at `308aeda25`; serialized ledger/log each reject/restored exactly. Original 18 seals unchanged; 20-member copied archive and five separate receipt seals recorded in `portable-proof.json`, `portable-controls.log` and `receipt-seals.json`. No final gates.

## Known gaps (current cycle)

- [ ] Unexpanded linked diffs, UnitAura remaining return layout, full C_CVar signatures and native behaviors remain UNPROVEN.
- [ ] Main-owned actual successors/current-native/integration and any final acceptance remain unperformed by this source slice.

## Out of scope

Runtime/profile/default/shim/alias/fallback changes, sealed 2.5.6 receipt backfill, foreign history supersession, shared parser/extractor changes, native/model measurements, canonical/vendor/cache/Wowless writes, network, broad/check/lint/readability/coverage/startup/final gates, delegation/model CLI and operational actions. PLAN.md updates excluded from this commit under explicit plan skill policy.
