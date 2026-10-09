# Historical retail Patch 3.0.3 source accounting

Bounded audit of frozen page 560990/revision 5407654 (2010-03-28T14:26:47Z), from the committed legacy manifest/input pair. [Audit and implementation](../wiki/investigations/patch-3-0-3-api-audit.md).

## What it must do

- [x] Preserve exact three CVar spellings, definition prose and original line numbers behind an opt-in flag; default recorded bytes and integrated 3.3.3/3.3.5/4.0.1 outputs remain identical.
- [x] Account every nonblank raw/rendered line, header, inventory and signature boundary without linked expansion or behavioral credit.
- [x] Replay original ledger, gaps, parsers, model-source review and logs from sealed copied inputs in a fresh process without Git, target or mutable current files; reject serialized tampering and restore exact bytes.
- [x] Retain full handoff registry through 1.0.0, separate actual retail successors from ordered queued placeholders; never use Classic supersession.

## How it works

- [Own audit](../wiki/investigations/patch-3-0-3-api-audit.md).

## Implementation inventory

- `tools/gen_patch_wikitext_register.py`: opt-in `--legacy-cvar-definitions`.
- `data/patch-api/sources/3.0.3-*`: exact source, provenance, register, extract and current source ledger.
- `data/patch-api/evidence/3.0.3-session-2026-10-09/`: immutable original ledger, gap records and historical replay inputs.

## Tests asserting this spec

- `tools/test_patch_3_0_3_source.py`: literal serialized contracts, malformed-definition boundary and bounded recorded-output compatibility.
- `tools/test_patch_3_0_3_validator.py`: copied historical replay and tamper/restoration boundary.

## Separate supported retail factory measurement

- [x] Add standalone `tests/patch_3_0_3_factory.rs`, retail-only target, exact own three-name inventory and fabricated unknown-CVar boundary using the shared classifier. No spelling correction or aliases.
- [x] Discover with empty known gaps: RED at `2f8f3596d` observes one match/two gaps; fabricated unknown nil/nil control passes. [Fresh measurement ledger](../../data/patch-api/evidence/3.0.3-factory-2026-10-09/measurement-ledger.json) preserves exact results.
- [x] Targeted reviewed GREEN at `7ac3902cf`: 3/3 exit 0; all three sweep observations byte-identical to discovery. Only offline `--no-default-features --features client-retail`, no loaded UI/cache or native acceptance. [Exact revision/cwd/env/argv/full streams/hashes](../../data/patch-api/evidence/3.0.3-factory-2026-10-09/development-proof-ledger.json).

| Exact source name | Bare retail published | Observed value / default | Source default |
|---|---|---|---|
| `syncronizeConfig` | No | nil / nil | Unspecified |
| `synchronizeBindings` | Yes | `"1"` / `"1"` | Unspecified |
| `synchronizeMacros` | No | nil / nil | Unspecified |

Original 16 seals and eight ledger IDs remain historical source accounting. Separate current factory results cannot prove 0/1 synchronization, server/native/persistence effects or source defaults; arbitrary local CVar assignment is not synchronization.

## Known gaps (current cycle)

- [ ] Historical/native publication remains UNPROVEN; separate supported retail factory observes one published/two absent, not historical parity.
- [ ] Three flag-synchronization effects UNPROVEN: UI settings, bindings and macros. No default, endpoint, trigger, scope, conflict resolution, persistence or event contract specified.

## Development proof

[Exact command/revision ledger](../../data/patch-api/evidence/3.0.3-session-2026-10-09/development-proof-ledger.json): own source tests RED exit1 (two missing-flag failures / one template control pass) at `e2cf94de1`, GREEN exit0 3/3 at `ce0a0e863`; historical tests RED exit1 2/2 missing-validator failures at `f7582f153`, GREEN exit0 2/2 at `e50955ee3`. Seventeen serialized tamper/restoration controls, missing-gap restoration, seventeen omission controls and two overclaim/spelling controls pass. Original 16 seals/93 snapshots remain frozen; later documentation does not invalidate these scopes. No runtime or final acceptance.

## Out of scope

Linked forum contracts, correcting `syncronizeConfig`, callable signatures absent from the page, invented synchronization service, shims/fallbacks, runtime/vendor/cache/Wowless edits, Classic history and native/integration acceptance. Main owns integration and native acceptance; only targeted development tests are authorized here.
