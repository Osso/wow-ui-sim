# Patch 8.1.0 publication sweep

Audit pinned Warcraft Wiki page 464337, revision 4462737 against current retail, not historical reconstruction. [Audit](../wiki/investigations/patch-8-1-0-api-audit.md) records models, exact gaps and proof.

## What it must do

- [x] Refetch and pin raw source, provenance and fetch receipt; retain every API/extract occurrence and both rename identities. Preserve CVar header counts and build/citation context.
- [x] Reuse the 8.1.5 opt-in bullet parser without a competing implementation; keep rename and CVar-table additions separately opt-in. Every formerly reproducible saved artifact must reproduce.
- [x] Enforce exact reviewed publication gaps through a prefork sweep, with 8.1.5 then 8.2.0 integration placeholders followed by 8.2.5 and later master registers.
- [x] Implement supplied civil calendar comparison with documented rhs-relative sign, chronological component ordering, weekday independence and no mutation/clock defaults.
- [x] Retire only proven unused retail members; retain current cached consumers and live simulator provider callers. Preserve Mists paths and unmodified Blizzard Lua.
- [x] Retain complete whole-word qualified/bare cached and source/test scans plus later re-addition checks, including unmerged 8.1.5/8.2.0 snapshots.
- [ ] Record passing targeted sweep/area/parser/reproduction/Mists/format gates and artifact acceptance, without a full integration suite.
- [x] Validator must work after merge and later audits: no absolute checkout assertions; original shared-input checks compare pinned git revisions; counts derive from files.

## Implementation inventory

- `data/patch-api/sources/8.1.0-*` — pinned source, extract, register and occurrence ledger.
- `src/c_api/c_date_and_time.rs` — supplied civil-time value model and comparison.
- `src/c_api/patch_retired_members.rs` — retail-only three-member list.
- `tests/patch_8_1_0_*.rs`, `tests/data/patch_8_1_0_sweep_known_gaps.json` — bare/cached/classic behavior and exact publication gap enforcement.
- `data/patch-api/evidence/8.1.0-session-2026-10-08/` — exhaustive scans, gap review, reproduction, validator and command receipts.

## Tests asserting this spec

- `tests/patch_8_1_0_publication_sweep.rs`.
- `tests/patch_8_1_0_calendar_compare.rs`.
- `tests/patch_8_1_0_publication_fixes.rs`, `tests/patch_8_1_0_cached_surfaces.rs`.
- `tools/test_extract_patch_non_inventory.py`, `tools/test_gen_patch_wikitext_register.py`.
- Evidence `validate.py` and recorded scoped calendar/map/date-provider/configuration-provider/Mists gates.

## Known gaps

- [ ] 60 exact publication gaps retain per-ID model, identity, metadata, policy, request-lifecycle or protected-provider reasons.
- [ ] Two literal `?` source statements specify no implementable contract; retained as uncertainty, never behavioral credit.
- [ ] Current publication is not signature, populated-output, payload, security or historical parity proof. Calendar invalid civil-date/native security parity remains unverified.
- [ ] Inherited 12.0.5/12.0.7/12.1.0 saved extracts remain non-reproducible; unchanged failure boundaries are recorded.
- [ ] Main thread must replace the 8.1.5/8.2.0 placeholders during ordered integration and refresh any superseded observations/ledgers.

## Out of scope

No invented API defaults or shim shortcuts, no vendor/cache/Wowless/WowlessData edits, no Blizzard Lua monkey-patching, no other-page implementation, no session-cwd changes, push, merge, agents/models or full integration suite. Host has no WoW install; CASC texture acceptance is not claimed.
