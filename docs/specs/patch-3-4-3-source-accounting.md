# Wrath Classic 3.4.3 source accounting

Account for frozen Warcraft Wiki page `152751`/revision `5983024` as Wrath Classic TOC `30403`, without importing retail 10.1.7 members or claiming collection parity. [Ledger](../../data/patch-api/sources/3.4.3-page-coverage.json); [audit and proof boundaries](../wiki/investigations/patch-3-4-3-api-audit.md).

## What it must do

- [x] Preserve response identity/hash, exact wikitext, reproducible plaintext and every nonblank source row. [Proof ledger](../../data/patch-api/evidence/3.4.3-session-2026-10-09/source-proof.json): seven targeted tests and extraction pass at `84274cf67`.
- [x] Keep both summary bullets UNPROVEN, with no invented API inventory, positive publication/behavior credit or cross-client successor; reject mutated accounting.
- [x] Distinguish supported Wrath profile and historical documentation-only cache from native/runtime proof; preserve configured interface 38001 separately from source TOC 30403.
- [x] Replay sealed own historical inputs and derive counts independently of later global audits/cache/tool changes. Independent verifier 216 reports fresh historical replay PASS: 16 seals, seven rows/five metadata/two UNPROVEN summary contracts, zero explicit API occurrences/runtime observations. Original `48ab8e1c3` sealed inputs unchanged; retained 7/7 GREEN and tamper proof remain applicable to source accounting only. No native/current-runtime or positive empty-API parity credit.

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

Linked source reconstruction, shared ClientLine expansion, new runtime models without specified behavior, cache sync/vendor edits, positive empty-inventory compatibility credit, retail supersession/retirement, other page audits, broad verification and operational changes. Main owns integration after 4.0.1; shared portable gate, actual-master integration and CI remain pending, not a completed full handoff. [Historical cache observation](../wiki/investigations/patch-3-4-3-api-audit.md#profile-and-cache-evidence) records 42 entries including one empty `.missing` marker, not 42 usable documentation payloads; sealed observation bytes stay unchanged. This cache cannot establish publisher-loaded proof or unsupported-client status. Actual Wrath 38001 architecture differs from native Classic 30403.
