# Patch 1.13.3 frozen SOURCE accounting

Bounded Classic Era page audit from the [immutable source pin](../../data/patch-api/evidence/1.13.3-session-2026-10-09/source-pin.json). [Audit wiki](../wiki/investigations/patch-1-13-3-api-audit.md) records derivation/proof epochs. Source accounting does not grant native or runtime parity.

## What it must do

- [x] Validate exact frozen page/revision/timestamp, response/content hashes and manifest-linked remaining registry.
- [x] Preserve every nonblank raw row, all literal inventory/signature fragments, prose, headings/counts/captions/navigation and template/link/reference boundaries.
- [x] Replay historical default bytes/error unchanged, including 22-entry default register versus 27 literal inventory occurrences; never invent omitted signatures, defaults or aliases.
- [x] Derive totals/statuses/omission controls from source and fixtures; reject omission and fabricated credit.
- [x] Replay copied historical source with no Git/target/current tools; reject serialized ledger/log tampering and restore exact bytes under unchanged original seals.
- [x] Keep original seals immutable and later GREEN receipts separate.
- [x] Test only the grounded current Era NPC health-values subset separately from SOURCE/native credit.

## How it works

- [Audit and coverage matrix](../wiki/investigations/patch-1-13-3-api-audit.md).

## Implementation inventory

- `data/patch-api/evidence/1.13.3-session-2026-10-09/audit.py`: page-local lossless accounting, historical defaults and validator.
- Same evidence directory: frozen inputs, own state/model review, original ledger and proof epochs.
- Existing `src/lua_api/globals/utility_system_spell/spell_api.rs`: current state-backed health reads; no runtime change authorized without reproduced grounded gap.

## Tests asserting this spec

- `data/patch-api/evidence/1.13.3-session-2026-10-09/test_source_accounting.py`: nine literal SOURCE fixtures.
- `patch-tests/patch_1_13_3_npc_health.rs`: existing current bare Era health reads across three NPC snapshot mutations; not native/signature parity.
- `data/patch-api/evidence/1.13.3-session-2026-10-09/test_portable.py`: copied SOURCE/default replay and serialized ledger/log rejection/exact restoration.

## Known gaps (current cycle)

- [ ] Main-owned same-Era1.13.4 integration must precede1.13.3; no successor register applied by this page audit.
- [ ] All historical runtime/native/signature/security/default contracts remain UNPROVEN.

## Out of scope

Foreign history, linked/transcluded expansion, fabricated signatures/defaults/models, runtime name-factory promotion, successor integration and native/loaded-UI parity. Main integrates concurrent same-Era 1.13.4 before 1.13.3. No other checkout/network/vendor changes, broad/check/lint/readability/coverage/final gates, delegation/push/merge/deploy.
