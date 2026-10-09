# Wrath Classic 3.4.1 source accounting

Account frozen Warcraft Wiki page `379792`/revision `3656581` as Wrath Classic TOC 30401, preserving explicit inventory and unresolved prose/signatures without linked reconstruction. [Ledger](../../data/patch-api/sources/3.4.1-page-coverage.json); [audit](../wiki/investigations/patch-3-4-1-api-audit.md).

## What it must do

- [x] Preserve literal response identity, hashes, wikitext and all 376 nonblank rows; reject source/revision/accounting tampering.
- [x] Derive 333 inventory occurrences and six reconciled headers, retaining `LogFps` as a command inside the CVar added header and all 62 explicit default records.
- [x] Account both substantive prose rows, including raw widget-equivalence line 235 omitted by stock extraction, and 211 unspecified global signatures; reject fabricated behavior/native credit.
- [x] Keep supported Wrath profile/interface 38001 separate from source TOC 30401; permit no foreign supersession or positive runtime/native observation.
- [x] Reproduce exact recorded extraction and replay sealed own historical inputs independently of later registers/shared tools/Git.

## How it works

- [Literal inventory/prose/signature matrix and proof boundaries](../wiki/investigations/patch-3-4-1-api-audit.md).

## Implementation inventory

- `data/patch-api/sources/3.4.1-api-changes.{wikitext,txt}` — frozen raw and reproducible limited extract.
- `data/patch-api/sources/3.4.1-page-coverage.json` — inventory/source/prose/signature ledger, not runtime publication register.
- `data/patch-api/evidence/3.4.1-session-2026-10-09/` — response/pin, own historical tooling, profile observation, validator/tests/seals/receipts.

## Tests asserting this spec

`python3 data/patch-api/evidence/3.4.1-session-2026-10-09/test_source_accounting.py`: seven targeted serialized accounting tests. Own `validate.py`: sealed historical replay. `python3 tools/extract_patch_non_inventory.py --patch 3.4.1 --text-only --canonical-patch-navigation --check`: exact recorded flags, no runtime proof. [Proof ledger](../../data/patch-api/evidence/3.4.1-session-2026-10-09/source-proof.json).

`python3 data/patch-api/evidence/3.4.1-session-2026-10-09/replay_controls.py`: at `e6ed237fa`, both serialized ledger/log tamper controls reject at exact seals with exit 1, restore original bytes/hashes, and relocated Git-free archive replay exits 0. Exact receipts/member hashes in `portable-controls.log`; source/test/tool/validator bytes remain unchanged.

## Known gaps (current cycle)

- [ ] Runtime/native publication/removal for 333 enumerated occurrences; 211 signatures, 44 event payload/dispatch contracts, 77 CVar persistence/effect contracts and one command grammar/output contract unproven.
- [ ] Unspecified retail 10.0.2 subset and linked retail 10.0.0 widget equivalence remain UNPROVEN.

## Out of scope

Linked documentation reconstruction, shared classifier expansion, native/runtime probes, cache/vendor/Wowless writes, speculative modeled behavior, shims/fallbacks, retirement, other audits, provider/model/retry changes, broad/final gates and operational integration. Parent integrates after independently owned 3.4.2.
