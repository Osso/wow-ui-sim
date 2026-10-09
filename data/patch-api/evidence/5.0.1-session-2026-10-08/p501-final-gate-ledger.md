# Patch 5.0.1 final acceptance

Redirect source revision 5344081 is not the Git proof revision. Base Git `83e5d3c3b2643a360bf32e73f343faa987eeb39b` contains the supplied fetch receipt. Source/test/tool/Cargo proof identities are pinned at `f86fe61ebef0c9cbcff753ae83ef607656e736da`; own and publication executions have matching identities even where their Git receipt revisions differ.

## Commits

| Commit | Coherent change |
|---|---|
| 29a642ca8 | Literal redirect artifacts, zero-row retail sweep, spec/wiki boundary |
| f86fe61eb | Independent Mists target selection |
| 59140d35d | Reproduction, Python/format/Mists receipts, historical validator |
| 800a7c725 | Complete own-session seal membership |
| b0c401984 | Own/publication observations and formal fabricated-entry rejection |
| e1653e5c7 | Exact observed negative-control command exit |
| 4de100305 | 112 own-session records sealed with historical Git tree/blob pins |

## Commands and proof

See `p501-command-ledger.md` and individual `.proof.json` records for complete argv/revision/identity/exit/log seals. Runtime/source checks are not rerun for evidence-only commits.

`python3 -B data/patch-api/evidence/5.0.1-session-2026-10-08/validate.py` at `4de10030584362eb4c1fe37a6e512b4e6e5d555a`: PASS, 68 registers, 65 extracts, 64 sweep/factory cases, 58 historical prior validators. Source-response and own-log tampering independently fail at exact own-session seals; both restore byte-identically and the restored validator passes. Complete control outputs: `p501-seal-tamper.json`.

`python3 -B tools/check_patch_validators.py 4de10030584362eb4c1fe37a6e512b4e6e5d555a`: PASS, clean 59/59 and synthetic later audit 60/60, zero failures and no oversized evidence. Complete report: `p501-validator-gate.log`; revision/command/log seal: `p501-validator-gate.proof.json`; compact summary: `p501-validator-gate-summary.json`. Gate ran asynchronously once; no runtime/check/reproduction commands were rerun for this final evidence/docs step.

The pending-sealing notes in immutable historical `p501-command-ledger.md` describe its commit-time state and are superseded here. All substantive proof inputs remain sealed and unchanged. Final readback checks clean Git state, unchanged source/test/tool/Cargo identities, preserved wiki baseline and evidence sizes; they are direct administrative acceptance, not another broad gate.

## Boundary

Own inventory/header counts are empty, the sole source context is metadata-only, and both own observations are `{}`. Those observations are not positive API or behavior evidence. The fabricated fixture is control data only, rejected before observation. Destination 5.0.4 remains separately owned. No runtime, retirement, vendor or shared-tool edits; no full-suite execution, push, merge or delegation.
