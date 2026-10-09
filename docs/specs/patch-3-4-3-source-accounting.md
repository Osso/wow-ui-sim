# Wrath Classic 3.4.3 source accounting

Account for frozen Warcraft Wiki page `152751`/revision `5983024` as Wrath Classic TOC `30403`, without importing retail 10.1.7 members or claiming collection parity. [Ledger](../../data/patch-api/sources/3.4.3-page-coverage.json); [audit and proof boundaries](../wiki/investigations/patch-3-4-3-api-audit.md).

## What it must do

- [x] Preserve response identity/hash, exact wikitext, reproducible plaintext and every nonblank source row. [Proof ledger](../../data/patch-api/evidence/3.4.3-session-2026-10-09/source-proof.json): seven targeted tests and extraction pass at `84274cf67`.
- [x] Keep both summary bullets UNPROVEN, with no invented API inventory, positive publication/behavior credit or cross-client successor; reject mutated accounting.
- [x] Distinguish supported Wrath profile and historical documentation-only cache from native/runtime proof; preserve configured interface 38001 separately from source TOC 30403.
- [x] Replay sealed own historical inputs and derive counts independently of later global audits/cache/tool changes. Own source validator passes; altered proof-log seal rejected and original bytes restored.

## How it works

- [Source/prose/profile coverage matrix](../wiki/investigations/patch-3-4-3-api-audit.md).

## Implementation inventory

- `data/patch-api/sources/3.4.3-api-changes.{wikitext,txt}` — exact source and plaintext.
- `data/patch-api/sources/3.4.3-page-coverage.json` — Wrath-labeled source ledger, not publication register.
- `data/patch-api/evidence/3.4.3-session-2026-10-09/` — pin/response, historical tooling, cache observation, validator, tests and seals.

## Tests asserting this spec

`python3 data/patch-api/evidence/3.4.3-session-2026-10-09/test_source_accounting.py` checks serialized contracts and mutations. `python3 data/patch-api/evidence/3.4.3-session-2026-10-09/validate.py` replays sealed source/profile accounting; `python3 tools/extract_patch_non_inventory.py --patch 3.4.3 --text-only --canonical-patch-navigation --check` checks recorded extraction flags. Neither command measures simulator behavior.

## Known gaps (current cycle)

- [ ] Unspecified subset of retail 10.1.7 changes in Wrath Classic: linked page not expanded; member/signature/output/event/security contracts unproven.
- [ ] Unenumerated collection API additions for mounts/pets/toys/heirlooms: publication, behavior and native parity unproven.

## Out of scope

Linked source reconstruction, shared ClientLine expansion, new runtime models without specified behavior, cache sync/vendor edits, positive empty-inventory compatibility credit, retail supersession/retirement, other page audits, broad verification and operational changes. Main owns integration after 4.0.1 and final gates. Documentation-only local cache prevents a publisher-loaded Wrath probe; supported profile itself is not missing.
