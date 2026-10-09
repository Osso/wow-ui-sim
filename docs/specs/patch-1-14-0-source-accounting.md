# Patch 1.14.0 literal SOURCE accounting

Account frozen Warcraft Wiki page71995/revision710568 without turning historical inventory or linked baseline claims into simulator/native compatibility. [Audit mechanics and limits](../wiki/investigations/patch-1-14-0-api-audit.md).

## What it must do

- [x] Verify exact legacy-2026-10-09 manifest, registry, response identity, timestamp, raw bytes and hashes before derivation.
- [x] Preserve every nonblank literal row, inventory occurrence/direction, header/count, PTR build caption, script label, template and link boundary.
- [x] Preserve malformed/underqualified widget identities literally; do not invent aliases, arguments, returns, payloads, CVar defaults or modeled coverage.
- [x] Keep the foreign2.5.2 synchronization claim and navigation `like` separate from pending/actual same-Era successors, none applied here.
- [x] Reject each omitted accounting record, fabricated coverage, source-content/identity tamper and serialized ledger/log changes.
- [x] Replay copied historical SOURCE and opt-in register/default extractor bytes without Git, build target, current tools or network; preserve default-generator failure and exact restoration of original sealed bytes.

## How it works

- [Wiki audit](../wiki/investigations/patch-1-14-0-api-audit.md).

## Implementation inventory

- `data/patch-api/evidence/1.14.0-session-2026-10-09/audit.py` — page-local SOURCE derivation/validation; no simulator changes.
- `data/patch-api/evidence/1.14.0-session-2026-10-09/ledger.json` — literal boundaries, unknown contracts and zero measurements.
- `data/patch-api/evidence/1.14.0-session-2026-10-09/historical-tools/` — unchanged copied generator/extractor; existing `--skip-plain-scripts-label` opt-in.
- `data/patch-api/evidence/1.14.0-session-2026-10-09/seals.json` — immutable original SOURCE map; subsequent receipts separate.

## Tests asserting this spec

- `data/patch-api/evidence/1.14.0-session-2026-10-09/test_source_accounting.py` — literal SOURCE/omission/content/credit controls.
- `data/patch-api/evidence/1.14.0-session-2026-10-09/test_portable.py` — fresh copied replay and disk tamper/restoration controls.

## Known gaps (current cycle)

- [ ] Native historical client state/behavior, signatures/payloads/defaults and security contracts for723 inventory occurrences.
- [ ] Linked baseline, community notes and source diff contents; unexpanded API documentation templates.
- [ ] Main-owned ordered successor integration, native observations and final gates.

## Out of scope

Runtime changes, publication-as-model credit, inferred aliases/defaults/API behavior, foreign-client inclusion proof, shared-tool changes, dependency/toolchain/network/vendor/CASC/Wowless changes and integration/final acceptance.
