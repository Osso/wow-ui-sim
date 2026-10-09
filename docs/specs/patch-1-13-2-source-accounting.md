# Patch 1.13.2 frozen SOURCE accounting

Bounded Classic Era source audit from the [immutable pin](../../data/patch-api/evidence/1.13.2-session-2026-10-09/source-pin.json). [Audit wiki](../wiki/investigations/patch-1-13-2-api-audit.md) separates literal SOURCE accounting, current model observations and proof epochs. No native parity inferred.

## What it must do

- [x] Preserve exact frozen manifest/response/wikitext identity and hashes plus manifest-linked101-page registry.
- [x] Account for every raw row, inventory/signature/default/prose/headings/counts/captions/template/link/reference occurrence without expansion, invented aliases or default credit.
- [x] Derive totals and reject every omission/fabricated capability; replay unchanged default register bytes and extractor failure.
- [x] Copy historical replay without Git/target/current tools, reject disk ledger/log tampers and restore exact bytes under original seals.
- [x] Keep original seals immutable; later receipts separate.
- [ ] Apply actual same-Era identity/direction precedence separately, with no foreign/model/native closure or original-seal mutation.
- [x] Test only existing current Era generic CVar state mutations separately from historical signatures/defaults/effects/native parity.

## How it works

- [Audit and coverage matrix](../wiki/investigations/patch-1-13-2-api-audit.md).

## Implementation inventory

- `data/patch-api/evidence/1.13.2-session-2026-10-09/audit.py`: page-local literal accounting and validator; copied historical tools unchanged.
- Same evidence directory: immutable inputs, current-state review, ledger and separated epochs.
- Existing `src/cvars.rs` and `src/lua_api/globals/set_cvar_verb.rs`: generic mutable storage/read model; no runtime change.

## Tests asserting this spec

- Same evidence directory `test_source_accounting.py`: eight SOURCE fixtures, every occurrence omission and fabricated-credit controls.
- `patch-tests/patch_1_13_2_cvar_state.rs`: own bare Era generic CVar storage transitions; no source default/effect/native/signature credit.
- Same evidence directory `test_portable.py`: fresh copied SOURCE8/default replay and both disk seal rejections/exact restorations.
- Same evidence directory `test_successors.py`: actual same-Era order/overlaps, every omission, foreign-credit rejection and original seals unchanged.

## Known gaps (current cycle)

- [ ] Historical/native contracts remain UNPROVEN; name-factory/registration does not close modeled gaps.
- [ ] Main owns integration/native/final acceptance. Separate exact actual-successor receipt does not change frozen original queue or establish semantic closure.

## Out of scope

Retail1.12.0 parallel audit and all foreign history semantic supersession; linked/transcluded expansion; guessed signatures/defaults/aliases; native security/server/loaded-UI parity. Main integrates1.13.2 before1.12.0 without merging histories semantically. No other checkout/vendor/cache/network changes, broad/check/lint/readability/coverage/final gates, delegation/push/merge/deploy.
